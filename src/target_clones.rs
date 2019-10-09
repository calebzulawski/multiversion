extern crate proc_macro;

use crate::dispatcher::Dispatcher;
use crate::target::{parse_target_string, Target};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{
    token, Attribute, Block, Expr, FnArg, Ident, ItemFn, LitStr, Signature, Token, Visibility,
};

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

impl FunctionClone<'_> {
    pub fn name(&self) -> Ident {
        Ident::new(
            &if self.target.is_some() && self.target.as_ref().unwrap().has_features_specified() {
                format!(
                    "feature_{}",
                    self.target.as_ref().unwrap().features_string()
                )
            } else {
                "default".to_string()
            },
            Span::call_site(),
        )
    }

    pub fn unsafety(&self) -> Option<token::Unsafe> {
        if self.target.is_some() && self.target.as_ref().unwrap().has_features_specified() {
            Some(token::Unsafe {
                span: Span::call_site(),
            })
        } else {
            self.signature.unsafety
        }
    }
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

        // Outer fn has a descriptive name and is unsafe if features are specified
        let mut outer_signature = signature.clone();
        outer_signature.ident = self.name();
        outer_signature.unsafety = self.unsafety();

        // Inner fn signature matches the original fn signature
        let inner_signature = signature.clone();
        let inner_function_ident = &inner_signature.ident;
        let body = &self.body;
        let argument_names = &outer_signature
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

        // We create an inner and an outer function here.
        //
        // The outer function is invoked by the dispatcher. It must be marked unsafe since it uses
        // the `target_feature` attribute.
        //
        // The inner function contains the actual implementation of the function. This solves two
        // problems. First, this function has the same safety as the outermost function, allowing
        // normal safety guarantees within the unsafe outer function. Second, this function has the
        // same name as the outermost function, allowing recursion to skip nested feature
        // detection.
        tokens.extend(quote! {
            #target_arch
            #target_features
            pub #outer_signature {
                #[inline(always)]
                #inner_signature
                #body

                #inner_function_ident(#(#argument_names),*)
            }
        });
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
        let name = &signature.ident;
        let clones = (&self.clones).iter();
        let dispatcher = &self.dispatcher;
        tokens.extend(quote! {
            #visibility mod #name {
                use super::*;
                #(#clones)*
            }
            #(#attributes)*
            #visibility #signature {
                #dispatcher
            }
        });
    }
}

impl<'a> TargetClones<'a> {
    pub fn new(config: Config, func: &'a ItemFn) -> Self {
        let module = func.sig.ident.clone();
        let prepend_module = move |name| syn::parse2::<Expr>(quote! { #module::#name }).unwrap();
        let mut clones = Vec::new();
        let mut functions = Vec::new();
        for target in config.targets {
            clones.push(FunctionClone {
                target: Some(target.clone()),
                signature: func.sig.clone(),
                body: func.block.as_ref(),
            });
            functions.push((target, prepend_module(clones.last().unwrap().name())));
        }
        // push default
        clones.push(FunctionClone {
            target: None,
            signature: func.sig.clone(),
            body: func.block.as_ref(),
        });
        let default = prepend_module(clones.last().unwrap().name());

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
