use crate::target::Target;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, FnArg, Signature};

pub(crate) struct Dispatcher {
    pub signature: Signature,
    pub functions: Vec<(Target, Expr)>,
    pub default: Expr,
}

impl ToTokens for Dispatcher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let function_arguments = self.signature.inputs.iter();
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
                    #default
                }
            }
        };
        tokens.extend(quote! {
            use std::sync::atomic::{AtomicPtr, Ordering};
            type __fn_ty = unsafe fn (#(#argument_ty),*) #returns;
            #[cold]
            unsafe fn __resolver_fn (#(#function_arguments),*) #returns {
                fn __get_fn() -> __fn_ty {
                    #feature_detection
                }
                let __current_fn = __get_fn();
                __DISPATCHED_FN.store(__current_fn as *mut (), Ordering::Relaxed);
                __current_fn(#(#argument_names),*)
            }
            static __DISPATCHED_FN: AtomicPtr<()> = AtomicPtr::new(__resolver_fn as *mut ());
            let __current_ptr = __DISPATCHED_FN.load(Ordering::Relaxed);
            unsafe {
                let __current_fn = std::mem::transmute::<*mut (), __fn_ty>(__current_ptr);
                __current_fn(#(#argument_names),*)
            }
        });
    }
}
