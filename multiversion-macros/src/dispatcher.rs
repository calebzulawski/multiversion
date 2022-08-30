use crate::{target::Target, util};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, punctuated::Punctuated, token, Attribute, Block, Error, Ident, ItemFn, ItemMod,
    LitStr, Result, Signature, Visibility,
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
    pub selected_target: Option<Ident>,
}

impl Dispatcher {
    // Create functions for each target
    fn feature_fns(&self) -> Result<Vec<ItemFn>> {
        let mut fns = Vec::new();
        for target in &self.targets {
            // This function will always be unsafe, regardless of the safety of the multiversioned
            // function.
            //
            // This could accidentally allow unsafe operations to end up in functions that appear
            // safe, but the default function below will catch any misuses of unsafe, since it
            // inherits the original function safety.
            //
            // When target_feature 1.1 is available, this function can also use the original
            // function safety.
            let mut attrs = self.inner_attrs.clone();
            attrs.extend(target.fn_attrs());
            let block = if let Some(selected_target) = &self.selected_target {
                let features = target.features_slice();
                let block = &self.func.block;
                parse_quote! {
                    {
                        const #selected_target: multiversion::features::TargetFeatures = unsafe { multiversion::features::TargetFeatures::with_features(#features) };
                        #block
                    }
                }
            } else {
                self.func.block.clone()
            };
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
        let block = if let Some(selected_target) = &self.selected_target {
            let block = &self.func.block;
            parse_quote! {
                {
                    const #selected_target: multiversion::features::TargetFeatures = multiversion::features::TargetFeatures::new();
                    #block
                }
            }
        } else {
            self.func.block.clone()
        };
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

pub(crate) fn derive_dispatcher(
    targets: Punctuated<LitStr, token::Comma>,
    dispatcher: ItemMod,
) -> Result<TokenStream> {
    if dispatcher.content.is_none() || !dispatcher.content.as_ref().unwrap().1.is_empty() {
        return Err(Error::new(
            dispatcher.content.unwrap().0.span,
            "expected an empty module",
        ));
    }

    let targets = targets
        .iter()
        .map(Target::parse)
        .collect::<Result<Vec<_>>>()?;

    let attrs = dispatcher.attrs;
    let vis = dispatcher.vis;
    let ident = dispatcher.ident;

    let target_fn_name = |target: &Target| {
        Ident::new(
            &format!("features_{}", target.features_string()),
            Span::call_site(),
        )
    };

    let target_fns = targets.iter().map(|target| {
        let target_arch = target.target_arch();
        let target_features = target.target_feature();
        let name = target_fn_name(target);
        quote! {
            #target_arch
            #(#target_features)*
            unsafe fn #name<Output>(f: impl FnOnce() -> Output) -> Output
            {
                f()
            }
        }
    });

    let target_arm = targets.iter().enumerate().map(|(i, target)| {
        let name = target_fn_name(target);
        let index = i + 1;
        quote! { #index => unsafe { Self::#name(f) }, }
    });

    let detects = targets.iter().enumerate().map(|(i, target)| {
        let detect = target.features_detected();
        let index = i + 1;
        quote! {
            if #detect {
                return #index;
            }
        }
    });

    Ok(quote! {
        #(#attrs)* #vis mod #ident {
            #[doc(hidden)]
            pub struct Dispatcher;

            impl Dispatcher {
                pub fn dispatch() -> usize {
                    #[cold]
                    fn detect() -> usize {
                        #(#detects)*
                        0
                    }

                    use core::sync::atomic::{AtomicUsize, Ordering};
                    static SELECTED: AtomicUsize = AtomicUsize::new(usize::MAX);
                    let selected = SELECTED.load(Ordering::Relaxed);
                    if selected == usize::MAX {
                        let selected = detect();
                        SELECTED.store(selected, Ordering::Relaxed);
                        selected
                    } else {
                        selected
                    }
                }

                #(#target_fns)*

                pub fn none<Output>(f: impl FnOnce() -> Output) -> Output
                {
                    f()
                }
            }

            #[doc(hidden)]
            #[macro_export]
            macro_rules! dispatch {
                { $dispatcher:ty, $expr:expr } => {
                    match <$dispatcher>::detect() {
                        0 => Self::none(f),
                        #(#target_arm)*
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    }
                }
            }

            #[doc(hidden)]
            pub use dispatch;
        }
    })
}
