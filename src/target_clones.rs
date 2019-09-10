extern crate proc_macro;

use crate::dispatcher::Dispatcher;
use crate::target::{parse_target_string, Target};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{token, Attribute, Block, Ident, ItemFn, LitStr, Signature, Token, Visibility};

pub(crate) struct Config {
    targets: Vec<Target>,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut targets = Vec::new();
        for target_string in Punctuated::<LitStr, Token![,]>::parse_terminated(&input)? {
            targets.extend(parse_target_string(&target_string)?)
        }
        Ok(Self { targets: targets })
    }
}

struct FunctionClone<'a> {
    target: Option<Target>,
    signature: Signature,
    body: &'a Block,
}

impl ToTokens for FunctionClone<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let target_arch = self
            .target
            .as_ref()
            .map_or(TokenStream::new(), |x| x.target_arch());
        let target_features = self
            .target
            .as_ref()
            .map_or(TokenStream::new(), |x| x.target_features());
        let signature = &self.signature;
        let mut unsafe_signature = signature.clone();
        unsafe_signature.unsafety = Some(token::Unsafe {
            span: Span::call_site(),
        });
        let body = &self.body;
        tokens.extend(quote! { #target_arch #target_features #unsafe_signature #body });
    }
}

pub(crate) struct TargetClones<'a> {
    attributes: &'a Vec<Attribute>,
    visibility: &'a Visibility,
    clones: Vec<FunctionClone<'a>>,
    dispatcher: Dispatcher,
}

impl ToTokens for TargetClones<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attributes = (&self.attributes).iter();
        let visibility = &self.visibility;
        let signature = &self.dispatcher.signature;
        let clones = (&self.clones).iter();
        let dispatcher = &self.dispatcher;
        tokens.extend(quote! {
            #(#attributes)*
            #visibility #signature {
                #(#clones)*
                #dispatcher
            }
        });
    }
}

impl<'a> TargetClones<'a> {
    pub fn new(config: Config, func: &'a ItemFn) -> Self {
        let mut clones = Vec::new();
        let mut functions = Vec::new();
        let mut id: u64 = 0;
        let mut new_signature = move || {
            let mut signature = func.sig.clone();
            signature.ident = Ident::new(&format!("__clone_{}", id), Span::call_site());
            id += 1;
            signature
        };
        for target in config.targets {
            clones.push(FunctionClone {
                target: Some(target.clone()),
                signature: new_signature(),
                body: func.block.as_ref(),
            });
            functions.push((target, clones.last().unwrap().signature.ident.clone()));
        }
        // push default
        clones.push(FunctionClone {
            target: None,
            signature: new_signature(),
            body: func.block.as_ref(),
        });
        let default = clones.last().unwrap().signature.ident.clone();

        Self {
            attributes: &func.attrs,
            visibility: &func.vis,
            clones: clones,
            dispatcher: Dispatcher {
                signature: func.sig.clone(),
                functions: functions,
                default: default,
            },
        }
    }
}
