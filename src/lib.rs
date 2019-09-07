extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{
    braced, parenthesized, parse_macro_input, token, FnArg, Ident, LitStr, Signature, Token,
};

fn expect_token(input: &ParseStream, token: &str) -> Result<Ident> {
    let ident: Ident = input.parse()?;
    if ident != token {
        Err(Error::new(ident.span(), format!("expected '{}'", token)))
    } else {
        Ok(ident)
    }
}

struct FeatureSet {
    _paren: token::Paren,
    features: Punctuated<LitStr, token::Comma>,
}

impl Parse for FeatureSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            _paren: parenthesized!(content in input),
            features: Punctuated::parse_terminated(&content)?,
        })
    }
}

struct FunctionArm {
    feature: FeatureSet,
    _arrow: token::FatArrow,
    function: Ident,
}

impl Parse for FunctionArm {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            feature: input.parse()?,
            _arrow: input.parse()?,
            function: input.parse()?,
        })
    }
}

struct DefaultFunction {
    _default: token::Default,
    _arrow: token::FatArrow,
    function: Ident,
}

impl Parse for DefaultFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            _default: input.parse()?,
            _arrow: input.parse()?,
            function: input.parse()?,
        })
    }
}

struct SpecializeBlock {
    _specialize: Ident,
    _paren: token::Paren,
    arch: Punctuated<LitStr, token::Comma>,
    _brace: token::Brace,
    arms: Punctuated<FunctionArm, token::Comma>,
    default_fn: Option<DefaultFunction>,
}

impl Parse for SpecializeBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let arch_content;
        let arms_content;
        let specialize: Ident = expect_token(&input, "specialize")?;
        let paren = parenthesized!(arch_content in input);
        let arch: Punctuated<LitStr, token::Comma> = Punctuated::parse_terminated(&arch_content)?;

        let brace = braced!(arms_content in input);
        let mut arms: Punctuated<FunctionArm, token::Comma> = Punctuated::new();
        while !arms_content.is_empty() && !arms_content.peek(Token![default]) {
            arms.push_value(arms_content.parse()?);
            arms.push_punct(arms_content.parse()?);
        }
        let default_fn: Option<DefaultFunction> =
            if !arms_content.is_empty() && arms_content.peek(Token![default]) {
                Some(arms_content.parse()?)
            } else {
                None
            };
        if !arms_content.is_empty() {
            let _trailing_comma: token::Comma = arms_content.parse()?;
        }

        Ok(Self {
            _specialize: specialize,
            _paren: paren,
            arch: arch,
            _brace: brace,
            arms: arms,
            default_fn: default_fn,
        })
    }
}

struct Dispatcher {
    specializations: Punctuated<SpecializeBlock, token::Comma>,
    default_fn: DefaultFunction,
}

impl Parse for Dispatcher {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut specializations: Punctuated<SpecializeBlock, token::Comma> = Punctuated::new();
        while !input.is_empty() && !input.peek(Token![default]) {
            specializations.push_value(input.parse()?);
            specializations.push_punct(input.parse()?);
        }
        let default_fn: DefaultFunction = input.parse()?;
        if !input.is_empty() {
            let _trailing_comma: token::Comma = input.parse()?;
        }
        Ok(Self {
            specializations: specializations,
            default_fn: default_fn,
        })
    }
}

impl Dispatcher {
    fn to_tokens_from_signature(&self, signature: &Signature) -> TokenStream {
        let mut feature_detection = TokenStream::new();
        for s in &self.specializations {
            let mut arms = TokenStream::new();
            for f in &s.arms {
                let function = &f.function;
                if f.feature.features.is_empty() {
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
                        let first_feature = f.feature.features.first().unwrap();
                        let rest_features = f.feature.features.iter().skip(1);
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
            if let Some(default_fn) = &s.default_fn {
                let function = &default_fn.function;
                arms.extend(quote! { return #function });
            }
            let arch = s.arch.iter();
            feature_detection.extend(quote! {
                #[cfg(any(#(target_arch = #arch),*))]
                {
                    #arms
                }
            });
        }
        let default_function = &self.default_fn.function;
        feature_detection.extend(quote! { return #default_function });
        let argument_names = signature
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
        let argument_ty = signature
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
        let returns = &signature.output;
        let function_type = quote! {
            fn (#(#argument_ty),*) #returns
        };
        quote! {
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
        }
    }
}

struct MultiVersion {
    signature: Signature,
    dispatcher: Dispatcher,
}

impl ToTokens for MultiVersion {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let signature = &self.signature;
        let dispatcher = self.dispatcher.to_tokens_from_signature(&signature);
        let generated = quote! {
            #signature {
                #dispatcher
            }
        };
        tokens.extend(generated);
    }
}

impl Parse for MultiVersion {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            signature: Signature {
                constness: None,
                asyncness: None,
                unsafety: input.parse().ok(),
                abi: input.parse().ok(),
                fn_token: input.parse()?,
                ident: input.parse()?,
                generics: input.parse()?,
                paren_token: parenthesized!(content in input),
                inputs: Punctuated::parse_terminated(&content)?,
                variadic: None,
                output: input.parse()?,
            },
            dispatcher: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn multiversion(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let multiversion = parse_macro_input!(input as MultiVersion);
    multiversion.into_token_stream().into()
}
