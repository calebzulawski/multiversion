use crate::dispatcher::Dispatcher;
use crate::target::parse_target_string;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parenthesized, Expr, Ident, LitStr, Signature, Token, Visibility};

pub(crate) struct MultiVersion {
    visibility: Visibility,
    dispatcher: Dispatcher,
}

impl ToTokens for MultiVersion {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let visibility = &self.visibility;
        let signature = &self.dispatcher.signature;
        let dispatcher = &self.dispatcher;
        tokens.extend(quote! {
            #visibility #signature {
                #dispatcher
            }
        });
    }
}

impl Parse for MultiVersion {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let visibility: Visibility = input.parse()?;
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
        let mut functions = Vec::new();
        while !input.is_empty() && !input.peek(Token![default]) {
            let target_string: LitStr = input.parse()?;
            let _arrow: Token![=>] = input.parse()?;
            let function: Ident = input.parse()?;
            let _comma: Token![,] = input.parse()?;
            functions.extend(
                parse_target_string(&target_string)?
                    .drain(..)
                    .map(|t| (t, syn::parse2::<Expr>(function.to_token_stream()).unwrap())),
            );
        }
        let _default: Token![default] = input.parse()?;
        let _arrow: Token![=>] = input.parse()?;
        let default: Expr = input.parse()?;
        if !input.is_empty() {
            let _trailing_comma: Token![,] = input.parse()?;
        }
        Ok(Self {
            visibility: visibility,
            dispatcher: Dispatcher {
                signature: signature,
                functions: functions,
                default: default,
            },
        })
    }
}
