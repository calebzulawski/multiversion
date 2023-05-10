use crate::{target::Target, util};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{
    parse_quote, Attribute, Block, Error, Expr, Ident, ItemFn, Result, Signature, Visibility,
};

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
    Ident::new(&format!("{ident}_default_version"), ident.span())
}

fn unsafe_fn_safe_block(f: ItemFn) -> ItemFn {
    let safe_fn = ItemFn {
        vis: Visibility::Inherited,
        sig: Signature {
            unsafety: None,
            ident: Ident::new("__safe_inner", f.sig.ident.span()),
            ..f.sig.clone()
        },
        ..f.clone()
    };

    let (unsafe_sig, args) = crate::util::normalize_signature(&f.sig);
    let maybe_await = f.sig.asyncness.map(|_| crate::util::await_tokens());
    let safe_ident = &safe_fn.sig.ident;
    let fn_params = crate::util::fn_params(&unsafe_sig);
    ItemFn {
        block: parse_quote! {
            {
                #[inline(always)]
                #safe_fn
                #safe_ident::<#(#fn_params),*>(#(#args),*)#maybe_await
            }
        },
        sig: unsafe_sig,
        ..f
    }
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
            let features = target.map(|t| t.features()).unwrap_or(&[]);
            let features_init = quote! {
                (multiversion::target_features::CURRENT_TARGET)#(.with_feature_str(#features))*
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
                    #[doc(hidden)] // https://github.com/rust-lang/rust/issues/111415
                    #[allow(unused)]
                    pub mod __multiversion {
                        pub const FEATURES: multiversion::target::Target = #features_init;

                        macro_rules! inherit_target {
                            { $f:item } => { #(#feature_attrs)* $f }
                        }

                        macro_rules! target_cfg {
                            { [$cfg:meta] $($attached:tt)* } => { #[multiversion::target::target_cfg_impl(#features, $cfg)] $($attached)* };
                        }

                        macro_rules! target_cfg_attr {
                            { [$cfg:meta, $attr:meta] $($attached:tt)* } => { #[multiversion::target::target_cfg_attr_impl(#features, $cfg, $attr)] $($attached)* };
                        }

                        macro_rules! target_cfg_f {
                            { $cfg:meta } => { multiversion::target::target_cfg_f_impl!(#features, $cfg) };
                        }

                        macro_rules! match_target {
                            { $($arms:tt)* } => { multiversion::target::match_target_impl!{ #features $($arms)* } }
                        }

                        pub(crate) use inherit_target;
                        pub(crate) use target_cfg;
                        pub(crate) use target_cfg_attr;
                        pub(crate) use target_cfg_f;
                        pub(crate) use match_target;
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
            // For now, nest a safe copy in the unsafe version. This is imperfect, but sound.
            //
            // When target_feature 1.1 is available, this function can instead use the original
            // function safety.
            let mut f = unsafe_fn_safe_block(ItemFn {
                attrs: self.inner_attrs.clone(),
                vis: Visibility::Inherited,
                sig: Signature {
                    ident: feature_fn_name(&self.func.sig.ident, Some(target)),
                    unsafety: parse_quote! { unsafe },
                    ..self.func.sig.clone()
                },
                block: make_block(Some(target)),
            });
            f.attrs.extend(target.fn_attrs());
            fns.push(f);
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

    fn call_target_fn(&self, target: Option<&Target>) -> Expr {
        let function = feature_fn_name(&self.func.sig.ident, target);
        let fn_params = util::fn_params(&self.func.sig);
        let (_, argument_names) = util::normalize_signature(&self.func.sig);
        let maybe_await = self.func.sig.asyncness.map(|_| util::await_tokens());
        parse_quote! {
            unsafe { #function::<#(#fn_params),*>(#(#argument_names),*)#maybe_await }
        }
    }

    fn static_dispatcher_fn(&self) -> Block {
        let return_if_detected = self.targets.iter().filter_map(|target| {
            if target.has_features_specified() {
                let target_arch = target.target_arch();
                let features_enabled = target.features_enabled();
                let call = self.call_target_fn(Some(target));
                Some(quote! {
                    #target_arch
                    {
                        if #features_enabled {
                            return #call
                        }
                    }
                })
            } else {
                None
            }
        });
        let call_default = self.call_target_fn(None);
        parse_quote! {
            {
                #(#return_if_detected)*
                #call_default
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
                "direct function dispatch only available with the `std` cargo feature",
            ));
        }

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

        let match_arm = ordered_targets.iter().enumerate().map(|(index, target)| {
            let index = index + 1; // 0 is default features
            let target_arch = target.target_arch();
            let arm = self.call_target_fn(Some(target));
            quote! {
                #target_arch
                #index => #arm,
            }
        });
        let call_default = self.call_target_fn(None);
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
        //
        // First, we determine which dispatcher to use.
        //
        // If the dispatcher is unspecified, decide on the following criteria:
        // * If the std feature is not enabled, dispatch statically, since we can't do CPU feature
        //   detection.
        // * If the function is generic, async, or has impl Trait, use direct dispatch, since we
        //   can't take a function pointer.
        // * If any retpoline features are enabled use direct dispatch, since retpolines hurt
        //   performance of indirect dispatch significantly.
        // * Otherwise, prefer indirect dispatch, since it appears to have better performance on
        //   average.  On machines with worse branch prediction, it may be significantly better.
        //
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

        // If we already know that the current build target supports the best function choice, we
        // can skip dispatching entirely.
        //
        // Here we check for one of two possibilities:
        // * If the globally enabled features (the target-feature or target-cpu codegen options)
        //   already support the highest priority function, skip dispatch entirely and call that
        //   function.
        // * If the current target isn't specified in the multiversioned list at all, we can skip
        //   dispatch entirely and call the default function.
        //
        // In these cases, the default function is called instead.
        let best_targets = self
            .targets
            .iter()
            .rev()
            .map(|t| (t.arch(), t))
            .collect::<HashMap<_, _>>();
        let mut skips = Vec::new();
        for (arch, target) in best_targets.iter() {
            let feature = target.features();
            skips.push(quote! {
                all(target_arch = #arch, #(target_feature = #feature),*)
            });
        }
        let specified_arches = best_targets.keys().collect::<Vec<_>>();
        let call_default = self.call_target_fn(None);
        let (normalized_signature, _) = util::normalize_signature(&self.func.sig);
        let feature_fns = self.feature_fns()?;
        Ok(ItemFn {
            attrs: self.func.attrs.clone(),
            vis: self.func.vis.clone(),
            sig: normalized_signature,
            block: Box::new(parse_quote! {
                {
                    #(#feature_fns)*

                    #[cfg(any(
                        not(any(#(target_arch = #specified_arches),*)),
                        #(#skips),*
                    ))]
                    { return #call_default }

                    #[cfg(not(any(
                        not(any(#(target_arch = #specified_arches),*)),
                        #(#skips),*
                    )))]
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
