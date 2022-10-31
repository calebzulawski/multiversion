use crate::{target::Target, util};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, Attribute, Block, Error, Ident, ItemFn, Result, Signature, Visibility};

pub(crate) fn feature_fn_name(ident: &Ident, target: Option<&Target>) -> Ident {
    if let Some(target) = target {
        if target.has_features_specified() {
            return Ident::new(
                &format!("{}_{}_version", ident, target.features_string()),
                ident.span(),
            );
        }
    }

    // If this is a default fn, it doesn't have a dedicated static dispatcher
    Ident::new(&format!("{}_default_version", ident), ident.span())
}

pub(crate) enum DispatchMethod {
    Default,
    Static,
    Direct,
    Indirect,
}

pub(crate) struct Dispatcher {
    pub dispatcher: DispatchMethod,
    pub inner_attrs: Vec<Attribute>,
    pub targets: Vec<Target>,
    pub func: ItemFn,
}

impl Dispatcher {
    // Create functions for each target
    fn feature_fns(&self) -> Result<Vec<ItemFn>> {
        let make_block = |target: Option<&Target>| {
            let block = &self.func.block;
            let features_init = if let Some(target) = target {
                let features = target.features_slice();
                quote! { unsafe { multiversion::target::TargetFeatures::with_features(#features) } }
            } else {
                quote! { multiversion::target::TargetFeatures::new() }
            };
            let feature_attrs = if let Some(target) = target {
                target.target_feature()
            } else {
                Vec::new()
            };
            let features = if let Some(target) = target {
                let s = target
                    .features()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                s.join(",")
            } else {
                String::new()
            };
            parse_quote! {
                {
                    #[allow(unused)]
                    pub mod __multiversion {
                        pub const FEATURES: multiversion::target::TargetFeatures = #features_init;

                        macro_rules! inherit_target {
                            { $f:item } => { #(#feature_attrs)* $f }
                        }

                        macro_rules! cfg_selected {
                            { [$cfg:meta] $($attached:tt)* } => { #[multiversion::target::cfg_selected_impl(#features, $cfg)] $($attached)* };
                        }

                        macro_rules! cfg_attr_selected {
                            { [$cfg:meta, $attr:meta] $($attached:tt)* } => { #[multiversion::target::cfg_attr_selected_impl(#features, $cfg, $attr)] $($attached)* };
                        }

                        pub(crate) use inherit_target;
                        pub(crate) use cfg_selected;
                        pub(crate) use cfg_attr_selected;
                    }
                    #block
                }
            }
        };

        let mut fns = Vec::new();
        for target in &self.targets {
            // This function will always be unsafe, regardless of the safety of the multiversioned
            // function.
            //
            // This could accidentally allow unsafe operations to end up in functions that appear
            // safe, but the deny lint should catch it.
            //
            // When target_feature 1.1 is available, this function can also use the original
            // function safety.
            let mut attrs = self.inner_attrs.clone();
            attrs.extend(target.fn_attrs());
            if self.func.sig.unsafety.is_none() {
                attrs.push(parse_quote!(#[deny(unsafe_op_in_unsafe_fn)]));
            }
            let block = make_block(Some(target));
            fns.push(ItemFn {
                attrs,
                vis: Visibility::Inherited,
                sig: Signature {
                    ident: feature_fn_name(&self.func.sig.ident, Some(target)),
                    unsafety: parse_quote! { unsafe },
                    ..self.func.sig.clone()
                },
                block,
            });
        }

        // Create default fn
        let mut attrs = self.inner_attrs.clone();
        attrs.push(parse_quote! { #[inline(always)] });
        let block = make_block(None);
        fns.push(ItemFn {
            attrs,
            vis: self.func.vis.clone(),
            sig: Signature {
                ident: feature_fn_name(&self.func.sig.ident, None),
                ..self.func.sig.clone()
            },
            block,
        });

        Ok(fns)
    }

    fn static_dispatcher_fn(&self) -> Block {
        let fn_params = util::fn_params(&self.func.sig);
        let (_, argument_names) = util::normalize_signature(&self.func.sig);
        let maybe_await = self.func.sig.asyncness.map(|_| util::await_tokens());
        let return_if_detected = self.targets.iter().filter_map(|target| {
            if target.has_features_specified() {
                let target_arch = target.target_arch();
                let features_enabled = target.features_enabled();
                let function = feature_fn_name(&self.func.sig.ident, Some(target));
                Some(quote! {
                    #target_arch
                    {
                        if #features_enabled {
                            return unsafe { #function::<#(#fn_params),*>(#(#argument_names),*)#maybe_await }
                        }
                    }
                })
            } else {
                None
            }
        });
        let default_fn = feature_fn_name(&self.func.sig.ident, None);
        parse_quote! {
            {
                #(#return_if_detected)*
                #default_fn::<#(#fn_params),*>(#(#argument_names),*)#maybe_await
            }
        }
    }

    fn indirect_dispatcher_fn(&self) -> Result<Block> {
        if !cfg!(feature = "std") {
            return Err(Error::new(
                Span::call_site(),
                "indirect function dispatch only available with the `std` cargo feature",
            ));
        }
        if !util::fn_params(&self.func.sig).is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "indirect function dispatch does not support type generic or const generic parameters",
            ));
        }
        if self.func.sig.asyncness.is_some() {
            return Err(Error::new(
                Span::call_site(),
                "indirect function dispatch does not support async functions",
            ));
        }
        if util::impl_trait_present(&self.func.sig) {
            return Err(Error::new(
                Span::call_site(),
                "indirect function dispatch does not support impl trait",
            ));
        }

        let fn_ty = util::fn_type_from_signature(&Signature {
            unsafety: parse_quote! { unsafe },
            ..self.func.sig.clone()
        })?;
        let (normalized_signature, argument_names) = util::normalize_signature(&self.func.sig);

        let feature_detection = {
            let return_if_detected = self.targets.iter().filter_map(|target| {
                if target.has_features_specified() {
                    let target_arch = target.target_arch();
                    let features_detected = target.features_detected();
                    let function = feature_fn_name(&self.func.sig.ident, Some(target));
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
            let default_fn = feature_fn_name(&self.func.sig.ident, None);
            quote! {
                fn __get_fn() -> #fn_ty {
                    #(#return_if_detected)*
                    #default_fn
                };
            }
        };
        let resolver_signature = Signature {
            ident: Ident::new("__resolver_fn", Span::call_site()),
            ..normalized_signature
        };
        Ok(parse_quote! {
            {
                use core::sync::atomic::{AtomicPtr, Ordering};
                #[cold]
                #resolver_signature {
                    #feature_detection
                    let __current_fn = __get_fn();
                    __DISPATCHED_FN.store(__current_fn as *mut (), Ordering::Relaxed);
                    unsafe { __current_fn(#(#argument_names),*) }
                }
                static __DISPATCHED_FN: AtomicPtr<()> = AtomicPtr::new(__resolver_fn as *mut ());
                let __current_ptr = __DISPATCHED_FN.load(Ordering::Relaxed);
                // Safety: the pointer is a fn pointer, so we can transmute it back to its original
                // representation.
                #[allow(clippy::undocumented_unsafe_blocks)]
                unsafe {
                    let __current_fn = core::mem::transmute::<*mut (), #fn_ty>(__current_ptr);
                    __current_fn(#(#argument_names),*)
                }
            }
        })
    }

    fn direct_dispatcher_fn(&self) -> Result<Block> {
        if !cfg!(feature = "std") {
            return Err(Error::new(
                Span::call_site(),
                "indirect function dispatch only available with the `std` cargo feature",
            ));
        }

        let fn_params = util::fn_params(&self.func.sig);
        let (_, argument_names) = util::normalize_signature(&self.func.sig);
        let maybe_await = self.func.sig.asyncness.map(|_| util::await_tokens());
        let ordered_targets = self
            .targets
            .iter()
            .filter(|target| target.has_features_specified())
            .collect::<Vec<_>>();

        let detect_index = {
            let detect_feature = ordered_targets.iter().enumerate().map(|(index, target)| {
                let index = index + 1; // 0 is default features
                let target_arch = target.target_arch();
                let features_detected = target.features_detected();
                quote! {
                    #target_arch
                    {
                        if #features_detected {
                            return #index
                        }
                    }
                }
            });
            quote! {
                fn __detect_index() -> usize {
                    #[cold]
                    fn __detect() -> usize {
                        #(#detect_feature)*
                        0
                    }

                    use core::sync::atomic::{AtomicUsize, Ordering};
                    static SELECTED: AtomicUsize = AtomicUsize::new(usize::MAX);
                    let selected = SELECTED.load(Ordering::Relaxed);
                    if selected == usize::MAX {
                        let selected = __detect();
                        SELECTED.store(selected, Ordering::Relaxed);
                        selected
                    } else {
                        selected
                    }
                }
            }
        };

        let call_function = |function| {
            quote! {
                unsafe { #function::<#(#fn_params),*>(#(#argument_names),*)#maybe_await }
            }
        };

        let match_arm = ordered_targets.iter().enumerate().map(|(index, target)| {
            let index = index + 1; // 0 is default features
            let target_arch = target.target_arch();
            let function = feature_fn_name(&self.func.sig.ident, Some(target));
            let arm = call_function(function);
            quote! {
                #target_arch
                #index => #arm,
            }
        });
        let call_default = call_function(feature_fn_name(&self.func.sig.ident, None));
        Ok(parse_quote! {
            {
                #detect_index
                match __detect_index() {
                    #(#match_arm)*
                    0 => #call_default,
                    _ => unsafe { core::hint::unreachable_unchecked() },
                }
            }
        })
    }

    fn create_fn(&self) -> Result<ItemFn> {
        let block = match self.dispatcher {
            DispatchMethod::Default => {
                if cfg!(feature = "std") {
                    if !crate::util::fn_params(&self.func.sig).is_empty()
                        || self.func.sig.asyncness.is_some()
                        || util::impl_trait_present(&self.func.sig)
                    {
                        self.direct_dispatcher_fn()?
                    } else {
                        let indirect = self.indirect_dispatcher_fn()?;
                        let direct = self.direct_dispatcher_fn()?;
                        parse_quote! {
                            {
                                #[cfg(not(any(
                                    target_feature = "retpoline",
                                    target_feature = "retpoline-indirect-branches",
                                    target_feature = "retpoline-indirect-calls",
                                )))]
                                #indirect
                                #[cfg(any(
                                    target_feature = "retpoline",
                                    target_feature = "retpoline-indirect-branches",
                                    target_feature = "retpoline-indirect-calls",
                                ))]
                                #direct
                            }
                        }
                    }
                } else {
                    self.static_dispatcher_fn()
                }
            }
            DispatchMethod::Static => self.static_dispatcher_fn(),
            DispatchMethod::Direct => self.direct_dispatcher_fn()?,
            DispatchMethod::Indirect => self.indirect_dispatcher_fn()?,
        };

        let (normalized_signature, _) = util::normalize_signature(&self.func.sig);
        let feature_fns = self.feature_fns()?;
        Ok(ItemFn {
            attrs: self.func.attrs.clone(),
            vis: self.func.vis.clone(),
            sig: normalized_signature,
            block: Box::new(parse_quote! {
                {
                    #(#feature_fns)*
                    #block
                }
            }),
        })
    }
}

impl ToTokens for Dispatcher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self.create_fn() {
            Ok(val) => val.into_token_stream(),
            Err(err) => err.to_compile_error(),
        })
    }
}
