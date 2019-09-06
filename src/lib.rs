extern crate proc_macro;

use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, parse_macro_input, token, Ident, LitStr, Signature, Token};

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

struct MultiVersion {
    signature: Signature,
    specializations: Punctuated<SpecializeBlock, token::Comma>,
    default_fn: DefaultFunction,
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
        while !input.peek(Token![default]) {
            specializations.push_value(input.parse()?);
            specializations.push_punct(input.parse()?);
        }

        Ok(Self {
            signature: signature,
            specializations: specializations,
            default_fn: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn multiversion(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let multiversion = parse_macro_input!(input as MultiVersion);
    proc_macro::TokenStream::new()
}
