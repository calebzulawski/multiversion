use crate::{target::Target, util};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, visit_mut::VisitMut, Attribute, Block, Ident, ItemFn, ItemUse, Lit, Meta,
    MetaNameValue, NestedMeta, Result, Signature, UseName, UsePath, UseRename, UseTree, Visibility,
};

fn feature_fn_name(ident: &Ident, target: Option<&Target>) -> Ident {
    if let Some(target) = target {
        if target.has_features_specified() {
            Ident::new(
                &format!(
                    "__multiversion_{}_feature_{}",
                    ident,
                    target.features_string()
                ),
                Span::call_site(),
            )
        } else {
            Ident::new(
                &format!("__multiversion_{}_default", ident),
                Span::call_site(),
            )
        }
    } else {
        Ident::new(
            &format!("__multiversion_{}_default", ident),
            Span::call_site(),
        )
    }
}

pub(crate) struct Specialization {
    pub target: Target,
    pub block: Block,
    pub normalize: bool,
}

impl Specialization {
    fn make_fn(&self, vis: &Visibility, sig: &Signature) -> Result<ItemFn> {
        let target_string = self.target.target_string();
        let target_attr: Attribute = parse_quote! { #[multiversion::target(#target_string)] };
        let fn_name = feature_fn_name(&sig.ident, Some(&self.target));
        if sig.unsafety.is_some() {
            // If the function is already unsafe, just tag it with the target attribute
            Ok(ItemFn {
                attrs: vec![
                    parse_quote! { #[inline] },
                    parse_quote! { #[doc(hidden)] },
                    target_attr,
                ],
                vis: vis.clone(),
                sig: Signature {
                    ident: fn_name,
                    ..sig.clone()
                },
                block: Box::new(self.block.clone()),
            })
        } else {
            // If the function isn't unsafe, nest an unsafe fn in it
            let fn_params = crate::util::fn_params(&sig);
            let maybe_await = sig.asyncness.map(|_| util::await_tokens());
            let unsafe_sig = Signature {
                ident: Ident::new("__unsafe_fn", Span::call_site()),
                unsafety: parse_quote! { unsafe },
                ..if self.normalize {
                    crate::util::normalize_signature(sig)?
                } else {
                    sig.clone()
                }
            };
            let outer_sig = Signature {
                ident: fn_name,
                ..crate::util::normalize_signature(sig)?
            };
            let args = util::args_from_signature(&outer_sig)?;
            let block = &self.block;
            Ok(ItemFn {
                attrs: vec![
                    parse_quote! { #[inline(always)] },
                    parse_quote! { #[doc(hidden)] },
                    self.target.target_arch(),
                ],
                vis: vis.clone(),
                block: Box::new(parse_quote! {
                    {
                        #target_attr
                        #[safe_inner]
                        #unsafe_sig
                        #block
                        unsafe { __unsafe_fn::<#(#fn_params),*>(#(#args),*)#maybe_await }
                    }
                }),
                sig: outer_sig,
            })
        }
    }
}

pub(crate) struct Dispatcher {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub sig: Signature,
    pub specializations: Vec<Specialization>,
    pub default: Block,
}

impl Dispatcher {
    // Create an attribute that disables an expression if we're on an architecture with a
    // specialized default
    fn cfg_if_not_defaulted(&self) -> Attribute {
        let mut defaulted_arches = Vec::new();
        for arches in self
            .specializations
            .iter()
            .filter_map(|Specialization { target, .. }| {
                if !target.has_features_specified() {
                    Some(target.arches_as_str())
                } else {
                    None
                }
            })
        {
            defaulted_arches.extend(arches);
        }
        parse_quote! { #[cfg(not(any(#(target_arch = #defaulted_arches),*)))] }
    }

    // Create specialized functions for arch/feature sets
    fn feature_fns(&self) -> Result<Vec<ItemFn>> {
        let mut fns = self
            .specializations
            .iter()
            .map(|x| x.make_fn(&self.vis, &self.sig))
            .collect::<Result<Vec<_>>>()?;

        // Create default fn
        fns.push({
            ItemFn {
                attrs: vec![
                    parse_quote! { #[inline(always)] },
                    parse_quote! { #[doc(hidden)] },
                    self.cfg_if_not_defaulted(),
                ],
                vis: self.vis.clone(),
                sig: Signature {
                    ident: feature_fn_name(&self.sig.ident, None),
                    ..self.sig.clone()
                },
                block: Box::new({
                    // Rewrite static dispatches, since this doesn't use the target attribute
                    let mut block = self.default.clone();
                    HelperAttributeVisitor { target: None }.visit_block_mut(&mut block);
                    block
                }),
            }
        });

        Ok(fns)
    }

    fn dispatcher_fn(&self) -> Result<ItemFn> {
        let fn_params = util::fn_params(&self.sig);
        let normalized_signature = util::normalize_signature(&self.sig)?;
        let argument_names = util::args_from_signature(&normalized_signature)?;
        let block: Block = if cfg!(feature = "runtime_dispatch")
            && fn_params.is_empty()
            && self.sig.asyncness.is_none()
        {
            // Dispatching from an atomic fn pointer occurs when the following is true:
            //   * runtime-dispatching is enabled
            //   * the function is not generic
            //   * the function is not async
            let fn_ty = util::fn_type_from_signature(&self.sig)?;
            let feature_detection = {
                let return_if_detected =
                    self.specializations
                        .iter()
                        .filter_map(|Specialization { target, .. }| {
                            if target.has_features_specified() {
                                let target_arch = target.target_arch();
                                let features_detected = target.features_detected();
                                let function = feature_fn_name(&self.sig.ident, Some(&target));
                                Some(quote! {
                                    #target_arch
                                    {
                                        if #features_detected {
                                            return #function
                                        }
                                    }
                                })
                            } else {
                                None
                            }
                        });
                let default_fn = feature_fn_name(&self.sig.ident, None);
                quote! {
                    fn __get_fn<#(#fn_params),*>() -> #fn_ty {
                        #(#return_if_detected)*
                        #default_fn
                    };
                }
            };
            let resolver_signature = Signature {
                ident: Ident::new("__resolver_fn", Span::call_site()),
                ..normalized_signature.clone()
            };
            parse_quote! {
                {
                    use core::sync::atomic::{AtomicPtr, Ordering};
                    #[cold]
                    #resolver_signature {
                        #feature_detection
                        let __current_fn = __get_fn();
                        __DISPATCHED_FN.store(__current_fn as *mut (), Ordering::Relaxed);
                        __current_fn(#(#argument_names),*)
                    }
                    static __DISPATCHED_FN: AtomicPtr<()> = AtomicPtr::new(__resolver_fn as *mut ());
                    let __current_ptr = __DISPATCHED_FN.load(Ordering::Relaxed);
                    unsafe {
                        let __current_fn = core::mem::transmute::<*mut (), #fn_ty>(__current_ptr);
                        __current_fn(#(#argument_names),*)
                    }
                }
            }
        } else {
            // Dispatch the function via branching if runtime-dispatching is disabled, or it is
            // generic/async/impl Trait
            let maybe_await = self.sig.asyncness.map(|_| util::await_tokens());
            let return_if_detected =
                self.specializations
                    .iter()
                    .filter_map(|Specialization { target, .. }| {
                        if target.has_features_specified() {
                            let target_arch = target.target_arch();
                            let features_detected = target.features_detected();
                            let function = feature_fn_name(&self.sig.ident, Some(&target));
                            Some(quote! {
                                #target_arch
                                {
                                    if #features_detected {
                                        return #function::<#(#fn_params),*>(#(#argument_names),*)#maybe_await
                                    }
                                }
                            })
                        } else {
                            None
                        }
                    });
            let default_fn = feature_fn_name(&self.sig.ident, None);
            parse_quote! {
                {
                    #(#return_if_detected)*
                    #default_fn::<#(#fn_params),*>(#(#argument_names),*)#maybe_await
                }
            }
        };
        Ok(ItemFn {
            attrs: self.attrs.clone(),
            vis: self.vis.clone(),
            sig: normalized_signature,
            block: Box::new(block),
        })
    }
}

impl ToTokens for Dispatcher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let feature_fns = self.feature_fns().unwrap();
        let dispatcher_fn = self.dispatcher_fn().unwrap();
        tokens.extend(quote! {
            #(#feature_fns)*
            #dispatcher_fn
        });
    }
}

pub(crate) struct HelperAttributeVisitor {
    pub target: Option<Target>,
}

impl HelperAttributeVisitor {
    fn rebuild_use_tree(&self, tree: &UseTree) -> ItemUse {
        fn finish(
            idents: Vec<&Ident>,
            name: &Ident,
            rename: &Ident,
            target: Option<&Target>,
        ) -> ItemUse {
            let fn_name = feature_fn_name(&name, target);
            if idents.is_empty() {
                parse_quote! { use #fn_name as #rename; }
            } else {
                parse_quote! { use #(#idents)::*::#fn_name as #rename; }
            }
        }
        fn detail<'a>(
            tree: &'a UseTree,
            mut idents: Vec<&'a Ident>,
            target: Option<&Target>,
        ) -> ItemUse {
            match tree {
                UseTree::Path(UsePath { ident, tree, .. }) => {
                    idents.push(ident);
                    detail(tree, idents, target)
                }
                UseTree::Name(UseName { ref ident }) => finish(idents, ident, ident, target),
                UseTree::Rename(UseRename { ident, rename, .. }) => {
                    finish(idents, ident, rename, target)
                }
                _ => panic!("unsupported use statement for #[static_dispatch]"),
            }
        }
        detail(tree, Vec::new(), self.target.as_ref())
    }

    fn target_cfg_value(&self, nested: &NestedMeta) -> bool {
        match nested {
            NestedMeta::Meta(meta) => match meta {
                Meta::Path(_) => panic!("not expecting path"),
                Meta::NameValue(MetaNameValue { path, lit, .. }) => {
                    if path.is_ident("target") {
                        if let Lit::Str(s) = lit {
                            let test_target = Some(Target::parse(s).unwrap());
                            test_target == self.target
                        } else {
                            panic!("expected string literal")
                        }
                    } else {
                        panic!("unknown key");
                    }
                }
                Meta::List(list) => {
                    if list.path.is_ident("not") {
                        assert_eq!(
                            list.nested.len(),
                            1,
                            "expected a single target_cfg predicate"
                        );
                        !self.target_cfg_value(list.nested.first().unwrap())
                    } else if list.path.is_ident("any") {
                        list.nested.iter().any(|x| self.target_cfg_value(x))
                    } else if list.path.is_ident("all") {
                        list.nested.iter().all(|x| self.target_cfg_value(x))
                    } else {
                        panic!("unknown path");
                    }
                }
            },
            NestedMeta::Lit(_) => panic!("not expecting literal"),
        }
    }
}

impl VisitMut for HelperAttributeVisitor {
    fn visit_item_use_mut(&mut self, i: &mut ItemUse) {
        let before = i.attrs.len();
        i.attrs
            .retain(|attr| *attr != parse_quote! { #[static_dispatch] });
        if i.attrs.len() < before {
            *i = self.rebuild_use_tree(&i.tree);
        }
    }

    fn visit_attribute_mut(&mut self, i: &mut Attribute) {
        if let Ok(Meta::List(list)) = i.parse_meta() {
            if list.path.is_ident("target_cfg") {
                assert_eq!(
                    list.nested.len(),
                    1,
                    "expected a single target_cfg predicate"
                );
                *i = if self.target_cfg_value(list.nested.first().unwrap()) {
                    parse_quote! { #[cfg(not(any()))] }
                } else {
                    parse_quote! { #[cfg(any())] }
                };
            }
        }
    }
}
