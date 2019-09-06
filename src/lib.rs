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

enum Architecture {
    X86,
}

impl Parse for Architecture {
    fn parse(input: ParseStream) -> Result<Self> {
        let arch: Ident = input.parse()?;
        match arch.to_string().as_str() {
            "x86" => Ok(Self::X86),
            _ => Err(Error::new(arch.span(), "expected 'x86'")),
        }
    }
}

struct FeatureSet {
    paren: token::Paren,
    features: Punctuated<LitStr, token::Comma>,
}

impl Parse for FeatureSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            paren: parenthesized!(content in input),
            features: Punctuated::parse_terminated(&content)?,
        })
    }
}

struct FunctionArm {
    feature: FeatureSet,
    arrow: token::FatArrow,
    function: Ident,
}

impl Parse for FunctionArm {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            feature: input.parse()?,
            arrow: input.parse()?,
            function: input.parse()?,
        })
    }
}

struct DefaultFunction {
    default: token::Default,
    function: Ident,
}

impl Parse for DefaultFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            default: input.parse()?,
            function: input.parse()?,
        })
    }
}

struct SpecializeBlock {
    specialize: Ident,
    arch: Architecture,
    brace: token::Brace,
    arms: Punctuated<FunctionArm, token::Comma>,
}

impl Parse for SpecializeBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            specialize: expect_token(&input, "specialize")?,
            arch: input.parse()?,
            brace: braced!(content in input),
            arms: Punctuated::parse_terminated(&content)?,
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
        while !input.peek(Token![default]) {
            specializations.push_value(input.parse()?);
            specializations.push_punct(input.parse()?);
        }
        Ok(Self {
            specializations: specializations,
            default_fn: input.parse()?,
        })
    }
}

impl Dispatcher {
    fn to_tokens_from_signature(&self, signature: &Signature) -> TokenStream {
        let mut feature_detection = TokenStream::new();
        for s in &self.specializations {
            let (cfg, has_feature) = match s.arch {
                Architecture::X86 => (
                    quote! {
                        #[cfg(any(target_arch == "x86", target_arch == "x86_64")) ]
                    },
                    quote! { is_x86_feature_detected! },
                ),
            };
            for f in &s.arms {
                let first_feature = f
                    .feature
                    .features
                    .iter()
                    .nth(0)
                    .expect("empty feature list");
                let rest_features = f.feature.features.iter().skip(1);
                let function = &f.function;
                feature_detection.extend(quote! {
                    #cfg
                    {
                        if #has_feature(#first_feature) #( && #has_feature(#rest_features) )* {
                            return #function
                        }
                    }
                });
            }
            let default_function = &self.default_fn.function;
            feature_detection.extend(quote! { return #default_function });
        }
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
            fn (#(#argument_ty),*) -> #returns
        };
        quote! {
            fn __get_fn() -> #function_type {
                #feature_detection
            }
            static mut __dispatched_fn: crate::atomic::Atomic<Option<#function_type>> = Atomic::new(None);
            if let Some(__current_fn) = __dispatched_fn.load(crate::atomic::Ordering::SeqCst) {
                __current_fn(#(#argument_names),*)
            } else {
                let __current_fn = __get_fn();
                __dispatched_fn.store(__current_fn, crate::atomic::Ordering::SeqCst);
                __current_fn(#(#argument_names).*)
            }
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
