use crate::target::Target;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, Signature};

pub(crate) struct Dispatcher {
    pub signature: Signature,
    pub functions: Vec<(Target, Ident)>,
    pub default: Ident,
}

impl ToTokens for Dispatcher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let feature_detection = {
            let defaulted_arches = self.functions.iter().filter_map(|(target, _)| {
                if !target.has_features_specified() {
                    Some(target.arch_as_str())
                } else {
                    None
                }
            });
            let return_if_detected = self.functions.iter().map(|(target, function)| {
                let target_arch = target.target_arch();
                let features_detected = target.features_detected();
                quote! {
                    #target_arch
                    {
                        if #features_detected {
                            return #function
                        }
                    }
                }
            });
            let default = &self.default;
            quote! {
                #(#return_if_detected)*
                #[cfg(not(any(#(target_arch = #defaulted_arches),*)))]
                {
                    return #default
                }
            }
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
        tokens.extend(quote! {
            use std::sync::atomic::{AtomicUsize, Ordering};
            type __fn_ty = fn (#(#argument_ty),*) #returns;
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
