use crate::{target::Target, util};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, visit_mut::VisitMut, Attribute, Block, Ident, ItemFn, ItemUse, Result, Signature,
    UseName, UsePath, UseRename, UseTree, Visibility,
};

fn feature_fn_name(target: Option<&Target>) -> Ident {
    if let Some(target) = target {
        if target.has_features_specified() {
            Ident::new(
                &format!("__feature_{}", target.features_string()),
                Span::call_site(),
            )
        } else {
            Ident::new("__default", Span::call_site())
        }
    } else {
        Ident::new("__default", Span::call_site())
    }
}

fn feature_mod_name(ident: &Ident) -> Ident {
    Ident::new(&format!("__static_dispatch_{}", ident), Span::call_site())
}

pub(crate) struct Specialization {
    pub target: Target,
    pub block: Block,
}

impl Specialization {
    fn make_fn(&self, sig: &Signature) -> Result<ItemFn> {
        let target_string = self.target.target_string();
        let target_attr: Attribute = parse_quote! { #[multiversion::target(#target_string)] };
        // If features are specified, this isn't a default function
        if self.target.has_features_specified() {
            if sig.unsafety.is_some() {
                // If the function is already unsafe, just tag it with the target attribute
                Ok(ItemFn {
                    attrs: vec![
                        parse_quote! { #[inline] },
                        parse_quote! { #[doc(hidden)] },
                        target_attr,
                    ],
                    vis: parse_quote! { pub(super) },
                    sig: Signature {
                        ident: Ident::new(
                            &format!("__feature_{}", self.target.features_string()),
                            Span::call_site(),
                        ),
                        ..sig.clone()
                    },
                    block: Box::new(self.block.clone()),
                })
            } else {
                // If the function isn't unsafe, nest an unsafe fn in it
                let maybe_await = sig.asyncness.map(|_| util::await_tokens());
                let unsafe_sig = Signature {
                    ident: Ident::new("__unsafe_fn", Span::call_site()),
                    unsafety: parse_quote! { unsafe },
                    ..sig.clone()
                };
                let args = util::args_from_signature(sig)?;
                let block = &self.block;
                Ok(ItemFn {
                    attrs: vec![
                        parse_quote! { #[inline(always)] },
                        parse_quote! { #[doc(hidden)] },
                        self.target.target_arch(),
                    ],
                    vis: parse_quote! { pub(super) },
                    sig: Signature {
                        ident: Ident::new(
                            &format!("__feature_{}", self.target.features_string()),
                            Span::call_site(),
                        ),
                        ..sig.clone()
                    },
                    block: Box::new(parse_quote! {
                        {
                            #target_attr
                            #[safe_inner]
                            #unsafe_sig
                            #block
                            unsafe { __unsafe_fn(#(#args),*)#maybe_await }
                        }
                    }),
                })
            }
        } else {
            // If no features are specified, this is just a default fn for a specific arch
            Ok(ItemFn {
                attrs: vec![
                    parse_quote! { #[inline(always)] },
                    parse_quote! { #[doc(hidden)] },
                    target_attr,
                ],
                vis: parse_quote! { pub(super) },
                sig: Signature {
                    ident: feature_fn_name(None),
                    ..sig.clone()
                },
                block: Box::new(self.block.clone()),
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
            .map(|x| x.make_fn(&self.sig))
            .collect::<Result<Vec<_>>>()?;

        // Create default fn
        fns.push({
            ItemFn {
                attrs: vec![
                    parse_quote! { #[inline(always)] },
                    parse_quote! { #[doc(hidden)] },
                    self.cfg_if_not_defaulted(),
                ],
                vis: parse_quote! { pub(super) },
                sig: Signature {
                    ident: feature_fn_name(None),
                    ..self.sig.clone()
                },
                block: Box::new({
                    // Rewrite static dispatches, since this doesn't use the target attribute
                    let mut block = self.default.clone();
                    StaticDispatchVisitor { target: None }.visit_block_mut(&mut block);
                    block
                }),
            }
        });

        Ok(fns)
    }

    fn dispatcher_fn(&self) -> Result<ItemFn> {
        let fn_ty_params = self.sig.generics.type_params().collect::<Vec<_>>();
        let argument_names = util::args_from_signature(&self.sig)?;
        let block: Block = if fn_ty_params.is_empty() && self.sig.asyncness.is_none() {
            let fn_ty = util::fn_type_from_signature(&self.sig);
            let feature_detection = {
                let feature_mod_name = feature_mod_name(&self.sig.ident);
                let return_if_detected =
                    self.specializations
                        .iter()
                        .map(|Specialization { target, .. }| {
                            let target_arch = target.target_arch();
                            let features_detected = target.features_detected();
                            let function = feature_fn_name(Some(&target));
                            quote! {
                                #target_arch
                                {
                                    if #features_detected {
                                        return #feature_mod_name::#function
                                    }
                                }
                            }
                        });
                let cfg_if_not_defaulted = self.cfg_if_not_defaulted();
                let default_fn = feature_fn_name(None);
                quote! {
                    fn __get_fn<#(#fn_ty_params),*>() -> #fn_ty {
                        #(#return_if_detected)*
                        #cfg_if_not_defaulted
                        {
                            #feature_mod_name::#default_fn
                        }
                    };
                }
            };
            let resolver_signature = Signature {
                ident: Ident::new("__resolver_fn", Span::call_site()),
                ..self.sig.clone()
            };
            // Not a generic fn, so use a static atomic ptr
            parse_quote! {
                {
                    use std::sync::atomic::{AtomicPtr, Ordering};
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
                        let __current_fn = std::mem::transmute::<*mut (), #fn_ty>(__current_ptr);
                        __current_fn(#(#argument_names),*)
                    }
                }
            }
        } else {
            // A generic, async, or impl Trait, so just call it directly
            let maybe_await = self.sig.asyncness.map(|_| util::await_tokens());
            let feature_mod_name = feature_mod_name(&self.sig.ident);
            let return_if_detected =
                self.specializations
                    .iter()
                    .map(|Specialization { target, .. }| {
                        let target_arch = target.target_arch();
                        let features_detected = target.features_detected();
                        let function = feature_fn_name(Some(&target));
                        quote! {
                            #target_arch
                            {
                                if #features_detected {
                                    return #feature_mod_name::#function(#(#argument_names),*)#maybe_await
                                }
                            }
                        }
                    });
            let cfg_if_not_defaulted = self.cfg_if_not_defaulted();
            let default_fn = feature_fn_name(None);
            parse_quote! {
                {
                    #(#return_if_detected)*
                    #cfg_if_not_defaulted
                    {
                        #feature_mod_name::#default_fn(#(#argument_names),*)#maybe_await
                    }
                }
            }
        };
        Ok(ItemFn {
            attrs: self.attrs.clone(),
            vis: self.vis.clone(),
            sig: self.sig.clone(),
            block: Box::new(block),
        })
    }
}

impl ToTokens for Dispatcher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.vis;
        let mod_name = feature_mod_name(&self.sig.ident);
        let feature_fns = self.feature_fns().unwrap();
        let dispatcher_fn = self.dispatcher_fn().unwrap();
        tokens.extend(quote! {
            #[doc(hidden)]
            #vis mod #mod_name {
                use super::*;
                #(#feature_fns)*
            }
            #dispatcher_fn
        });
    }
}

pub(crate) struct StaticDispatchVisitor {
    pub target: Option<Target>,
}

impl StaticDispatchVisitor {
    fn rebuild_use_tree(&self, tree: &UseTree) -> ItemUse {
        fn finish(
            idents: Vec<&Ident>,
            name: &Ident,
            rename: &Ident,
            target: Option<&Target>,
        ) -> ItemUse {
            let mod_name = feature_mod_name(&name);
            let fn_name = feature_fn_name(target);
            if idents.is_empty() {
                parse_quote! { use #mod_name::#fn_name as #rename; }
            } else {
                parse_quote! { use #(#idents)::*::#mod_name::#fn_name as #rename; }
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
}

impl VisitMut for StaticDispatchVisitor {
    fn visit_item_use_mut(&mut self, i: &mut ItemUse) {
        let before = i.attrs.len();
        i.attrs
            .retain(|attr| *attr != parse_quote! { #[static_dispatch] });
        if i.attrs.len() < before {
            *i = self.rebuild_use_tree(&i.tree);
        }
    }
}
