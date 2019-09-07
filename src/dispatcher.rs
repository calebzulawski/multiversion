use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, LitStr, Signature};

pub(crate) struct Dispatcher<'a> {
    pub signature: &'a Signature,
    pub specializations: Vec<Specialization<'a>>,
    pub default: Ident,
}

pub(crate) struct Specialization<'a> {
    pub architectures: Vec<&'a LitStr>,
    pub functions: Vec<(Vec<&'a LitStr>, Ident)>,
    pub default: Option<Ident>,
}

impl ToTokens for Specialization<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut arms = TokenStream::new();
        for f in &self.functions {
            let function = &f.1;
            if f.0.is_empty() {
                arms.extend(quote! { return #function });
            } else {
                for (arch, detect) in vec![
                    (
                        quote! { any(target_arch = "x86", target_arch = "x86_64") },
                        quote! { is_x86_feature_detected! },
                    ),
                    (
                        quote! { target_arch = "arm" },
                        quote! { is_arm_feature_detected! },
                    ),
                    (
                        quote! { target_arch = "aarch64" },
                        quote! { is_aarch64_feature_detected! },
                    ),
                    (
                        quote! { target_arch = "mips" },
                        quote! { is_mips_feature_detected! },
                    ),
                    (
                        quote! { target_arch = "mips64" },
                        quote! { is_mips64_feature_detected! },
                    ),
                    (
                        quote! { target_arch = "powerpc" },
                        quote! { is_powerpc_feature_detected! },
                    ),
                    (
                        quote! { target_arch = "powerpc64" },
                        quote! { is_powerpc64_feature_detected! },
                    ),
                ] {
                    let first_feature = f.0.first().unwrap();
                    let rest_features = f.0.iter().skip(1);
                    arms.extend(quote! {
                        #[cfg(#arch)]
                        {
                            if #detect(#first_feature) #( && #detect(#rest_features) )* {
                                return #function
                            }
                        }
                    });
                }
            }
        }
        if let Some(default) = &self.default {
            arms.extend(quote! { return #default });
        }
        let arch = &self.architectures;
        tokens.extend(quote! {
            #[cfg(any(#(target_arch = #arch),*))]
            {
                #arms
            }
        });
    }
}

impl ToTokens for Dispatcher<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let specializations = &self.specializations;
        let default = &self.default;
        let feature_detection = quote! {
            #(#specializations)*
            return #default
        };
        let argument_names = &self
            .signature
            .inputs
            .iter()
            .map(|x| {
                if let FnArg::Typed(p) = x {
                    p.pat.as_ref()
                } else {
                    unimplemented!("member fn not supported")
                }
            })
            .collect::<Vec<_>>();
        let argument_ty = &self
            .signature
            .inputs
            .iter()
            .map(|x| {
                if let FnArg::Typed(p) = x {
                    p.ty.as_ref()
                } else {
                    unimplemented!("member fn not supported")
                }
            })
            .collect::<Vec<_>>();
        let returns = &self.signature.output;
        let function_type = quote! {
            fn (#(#argument_ty),*) #returns
        };
        tokens.extend(quote! {
            use std::sync::atomic::{AtomicUsize, Ordering};
            type __fn_ty = #function_type;
            fn __get_fn() -> __fn_ty {
                #feature_detection
            }
            static __DISPATCHED_FN: AtomicUsize = AtomicUsize::new(0usize);
            let mut __current_ptr = __DISPATCHED_FN.load(Ordering::SeqCst);
            if __current_ptr == 0 {
                __current_ptr = unsafe { std::mem::transmute(__get_fn()) };
                __DISPATCHED_FN.store(__current_ptr, Ordering::SeqCst);
            }
            let __current_fn = unsafe { std::mem::transmute::<usize, __fn_ty>(__current_ptr) };
            __current_fn(#(#argument_names),*)
        });
    }
}
