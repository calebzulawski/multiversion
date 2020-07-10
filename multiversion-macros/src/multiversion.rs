use crate::dispatcher::Dispatcher;
use crate::meta::{parse_attributes, parse_crate_path};
use crate::target::Target;
use crate::util;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::convert::{TryFrom, TryInto};
use syn::{parse_quote, spanned::Spanned, Error, ItemFn, Lit, Meta, NestedMeta, Path};

enum Specialization {
    Clone {
        target: Target,
    },
    Override {
        target: Target,
        func: Path,
        is_unsafe: bool,
    },
}

struct Function {
    specializations: Vec<Specialization>,
    func: ItemFn,
    associated: bool,
    crate_path: Path,
}

impl TryFrom<Function> for Dispatcher {
    type Error = Error;

    fn try_from(item: Function) -> Result<Self, Self::Error> {
        let (_, args) = util::normalize_signature(&item.func.sig);
        let fn_params = util::fn_params(&item.func.sig);
        Ok(Self {
            specializations: item
                .specializations
                .iter()
                .map(|specialization| match specialization {
                    Specialization::Clone { target, .. } => crate::dispatcher::Specialization {
                        target: target.clone(),
                        block: item.func.block.as_ref().clone(),
                        normalize: false,
                    },
                    Specialization::Override {
                        target,
                        func,
                        is_unsafe,
                    } => {
                        let call = quote! { #func::<#(#fn_params),*>(#(#args),*) };
                        let call = if item.associated {
                            quote! { Self::#call }
                        } else {
                            call
                        };
                        crate::dispatcher::Specialization {
                            target: target.clone(),
                            block: if *is_unsafe {
                                parse_quote! {
                                    { unsafe { #call } }
                                }
                            } else {
                                parse_quote! {
                                    { #call }
                                }
                            },
                            normalize: true,
                        }
                    }
                })
                .collect(),
            attrs: item.func.attrs,
            vis: item.func.vis,
            sig: item.func.sig,
            default: *item.func.block,
            associated: item.associated,
            crate_path: item.crate_path,
        })
    }
}

impl TryFrom<ItemFn> for Function {
    type Error = Error;

    fn try_from(mut func: ItemFn) -> Result<Self, Self::Error> {
        let associated = crate::util::is_associated_fn(&mut func);
        let mut multiversioned = Function {
            specializations: Vec::new(),
            associated,
            crate_path: parse_quote!(multiversion),
            func: ItemFn {
                attrs: Vec::new(),
                ..func
            },
        };

        multiversioned.func.attrs = parse_attributes(func.attrs.drain(..), |path, nested| {
            Ok(match path.to_string().as_str() {
                "crate_path" => {
                    multiversioned.crate_path = parse_crate_path(nested)?;
                    true
                }
                "clone" => {
                    meta_parser! {
                        nested => [
                            "target" => target,
                        ]
                    }
                    multiversioned.specializations.push(Specialization::Clone {
                        target: target
                            .ok_or_else(|| Error::new(nested.span(), "expected key 'target'"))?
                            .try_into()?,
                    });
                    true
                }
                "specialize" => {
                    meta_parser! {
                        nested => [
                            "target" => target,
                            "fn" => func,
                            "unsafe" => is_unsafe,
                        ]
                    }
                    multiversioned
                        .specializations
                        .push(Specialization::Override {
                            target: target
                                .ok_or_else(|| Error::new(nested.span(), "expected key 'target'"))?
                                .try_into()?,
                            func: match func
                                .ok_or_else(|| Error::new(nested.span(), "expected key 'fn'"))?
                            {
                                Lit::Str(s) => s.parse(),
                                lit => Err(Error::new(lit.span(), "expected literal string")),
                            }?,
                            is_unsafe: is_unsafe.map_or(Ok(false), |lit| match lit {
                                Lit::Bool(b) => Ok(b.value),
                                lit => Err(Error::new(lit.span(), "expected literal bool")),
                            })?,
                        });
                    true
                }
                _ => false,
            })
        })?;

        Ok(multiversioned)
    }
}

pub(crate) fn make_multiversioned_fn(func: ItemFn) -> Result<TokenStream, syn::Error> {
    let function: Function = func.try_into()?;
    let dispatcher: Dispatcher = function.try_into()?;
    Ok(dispatcher.to_token_stream())
}
