extern crate proc_macro;

use crate::dispatcher::{Dispatcher, Specialization};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Block, FnArg, Ident, ItemFn, LitStr, Signature};

pub(crate) struct Config {
    specializations: Punctuated<SpecializeBlock, token::Comma>,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            specializations: Punctuated::parse_terminated(&input)?,
        })
    }
}

struct SpecializeBlock {
    _specialize: Ident,
    _paren: token::Paren,
    arch: Punctuated<LitStr, token::Comma>,
    _brace: token::Brace,
    features: Punctuated<FeatureSet, token::Comma>,
}

impl Parse for SpecializeBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let specialize: Ident = input.parse()?;
        if specialize != "specialize" {
            return Err(Error::new(specialize.span(), "expected 'specialize'"));
        }
        let arch_content;
        let feature_content;
        Ok(Self {
            _specialize: specialize,
            _paren: parenthesized!(arch_content in input),
            arch: Punctuated::parse_terminated(&arch_content)?,
            _brace: braced!(feature_content in input),
            features: Punctuated::parse_terminated(&feature_content)?,
        })
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

struct FunctionClone<'a> {
    architectures: Option<Vec<&'a LitStr>>,
    features: Option<Vec<&'a LitStr>>,
    signature: Signature,
    body: &'a Block,
}

impl ToTokens for FunctionClone<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let arch_cfg = if let Some(arch) = &self.architectures {
            quote! { #[cfg(any(#(target_arch = #arch),*))] }
        } else {
            TokenStream::new()
        };
        let signature = &self.signature;
        let body = &self.body;
        tokens.extend(if let Some(features) = &self.features {
            let feature_cfg = quote! { #(#[target_feature(enable = #features)])* };
            if signature.unsafety.is_some() {
                quote! { #arch_cfg #feature_cfg #signature #body }
            } else {
                let mut unsafe_signature = signature.clone();
                unsafe_signature.unsafety = Some(token::Unsafe {
                    span: Span::call_site(),
                });
                unsafe_signature.ident = Ident::new("__unsafe_fn", Span::call_site());
                let argument_names = &signature
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
                quote! {
                    #arch_cfg
                    #signature {
                        #feature_cfg
                        #unsafe_signature #body
                        unsafe {
                            __unsafe_fn(#(#argument_names),*)
                        }
                    }
                }
            }
        } else {
            quote! {
                #arch_cfg
                #signature #body
            }
        });
    }
}

pub(crate) struct MultiClones<'a> {
    signature: &'a Signature,
    clones: Vec<FunctionClone<'a>>,
    dispatcher: Dispatcher<'a>,
}

impl ToTokens for MultiClones<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let signature = &self.signature;
        let clones = (&self.clones).iter();
        let dispatcher = &self.dispatcher;
        tokens.extend(quote! {
            #signature {
                #(#clones)*
                #dispatcher
            }
        });
    }
}

impl<'a> MultiClones<'a> {
    pub fn new(config: &'a Config, func: &'a ItemFn) -> Self {
        let mut clones = Vec::new();
        let mut specializations = Vec::new();
        let mut id: u64 = 0;
        let mut new_signature = move || {
            let mut signature = func.sig.clone();
            signature.ident = Ident::new(&format!("__clone_{}", id), Span::call_site());
            id += 1;
            signature
        };
        for s in &config.specializations {
            let architectures = s.arch.iter().collect::<Vec<_>>();
            let mut functions = Vec::new();
            for f in &s.features {
                let features = f.features.iter().collect::<Vec<_>>();
                clones.push(FunctionClone {
                    architectures: Some(architectures.clone()),
                    features: Some(features.clone()),
                    signature: new_signature(),
                    body: func.block.as_ref(),
                });
                functions.push((features, clones.last().unwrap().signature.ident.clone()));
            }

            // push default
            clones.push(FunctionClone::<'a> {
                architectures: Some(architectures.clone()),
                features: None,
                signature: new_signature(),
                body: func.block.as_ref(),
            });

            // push specialization
            specializations.push(Specialization {
                architectures: architectures,
                functions: functions,
                default: Some(clones.last().unwrap().signature.ident.clone()),
            });
        }
        // push global default
        clones.push(FunctionClone {
            architectures: None,
            features: None,
            signature: new_signature(),
            body: func.block.as_ref(),
        });
        let default = clones.last().unwrap().signature.ident.clone();

        Self {
            signature: &func.sig,
            clones: clones,
            dispatcher: Dispatcher {
                signature: &func.sig,
                specializations: specializations,
                default: default,
            },
        }
    }
}
