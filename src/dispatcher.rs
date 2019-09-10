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
        let function_arguments = &self.signature.inputs.iter();
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
                let function_arguments = function_arguments.clone();
                let argument_names = argument_names.clone();
                quote! {
                    #target_arch
                    {
                        // wrap the function in an unsafe function with the same signature
                        unsafe fn unsafe_wrapper(#(#function_arguments),*) #returns {
                            #function(#(#argument_names),*)
                        }
                        if #features_detected {
                            return unsafe_wrapper
                        }
                    }
                }
            });
            let default = &self.default;
            let function_arguments = function_arguments.clone();
            let argument_names = argument_names.clone();
            quote! {
                #(#return_if_detected)*
                #[cfg(not(any(#(target_arch = #defaulted_arches),*)))]
                {
                    unsafe fn unsafe_wrapper(#(#function_arguments),*) #returns {
                        #default(#(#argument_names),*)
                    }
                    return unsafe_wrapper
                }
            }
        };
        tokens.extend(quote! {
            use std::sync::atomic::{AtomicUsize, Ordering};
            type __fn_ty = unsafe fn (#(#argument_ty),*) #returns;
            fn __get_fn() -> __fn_ty {
                #feature_detection
            }
            static __DISPATCHED_FN: AtomicUsize = AtomicUsize::new(0usize);
            let mut __current_ptr = __DISPATCHED_FN.load(Ordering::SeqCst);
            if __current_ptr == 0 {
                __current_ptr = unsafe { std::mem::transmute(__get_fn()) };
                __DISPATCHED_FN.store(__current_ptr, Ordering::SeqCst);
            }
            unsafe {
                let __current_fn = std::mem::transmute::<usize, __fn_ty>(__current_ptr);
                __current_fn(#(#argument_names),*)
            }
        });
    }
}
