use crate::dispatcher::{Dispatcher, Specialization};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Ident, LitStr, Signature, Token};

struct FunctionArm {
    _paren: token::Paren,
    features: Punctuated<LitStr, token::Comma>,
    _arrow: token::FatArrow,
    function: Ident,
}

impl Parse for FunctionArm {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            _paren: parenthesized!(content in input),
            features: Punctuated::parse_terminated(&content)?,
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
        let specialize: Ident = input.parse()?;
        if specialize != "specialize" {
            return Err(Error::new(specialize.span(), "expected 'specialize'"));
        }
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

pub(crate) struct MultiVersion {
    signature: Signature,
    specializations: Punctuated<SpecializeBlock, token::Comma>,
    default_fn: DefaultFunction,
}

impl ToTokens for MultiVersion {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let signature = &self.signature;
        let dispatcher = Dispatcher {
            signature: signature,
            specializations: self
                .specializations
                .iter()
                .map(|s| Specialization {
                    architectures: s.arch.iter().collect(),
                    functions: s
                        .arms
                        .iter()
                        .map(|a| (a.features.iter().collect(), a.function.clone()))
                        .collect(),
                    default: s.default_fn.as_ref().map(|x| x.function.clone()),
                })
                .collect(),
            default: self.default_fn.function.clone(),
        };
        tokens.extend(quote! {
            #signature {
                #dispatcher
            }
        });
    }
}

impl Parse for MultiVersion {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let signature = Signature {
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
        };
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
            signature: signature,
            specializations: specializations,
            default_fn: default_fn,
        })
    }
}
