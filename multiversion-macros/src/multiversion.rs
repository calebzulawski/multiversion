use crate::dispatcher::Dispatcher;
use crate::target::Target;
use crate::util;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::convert::{TryFrom, TryInto};
use syn::{
    parse_quote, spanned::Spanned, Error, Ident, ItemFn, Lit, Meta, MetaList, NestedMeta, Path,
};

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
    cpu_token: Option<Ident>,
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
            cpu_token: item.cpu_token,
        })
    }
}

impl TryFrom<ItemFn> for Function {
    type Error = Error;

    fn try_from(mut func: ItemFn) -> Result<Self, Self::Error> {
        let associated = crate::util::is_associated_fn(&mut func);
        let attrs = func.attrs;
        let mut multiversioned = Function {
            specializations: Vec::new(),
            associated,
            crate_path: parse_quote!(multiversion),
            cpu_token: None,
            func: ItemFn {
                attrs: Vec::new(),
                ..func
            },
        };

        for attr in attrs {
            // if not in meta list form, ignore the attribute
            let MetaList { path, nested, .. } = if let Ok(Meta::List(list)) = attr.parse_meta() {
                list
            } else {
                multiversioned.func.attrs.push(attr);
                continue;
            };

            // if meta path isn't just an ident, ignore the attribute
            let path = if let Some(ident) = path.get_ident() {
                ident
            } else {
                multiversioned.func.attrs.push(attr);
                continue;
            };

            // parse the attribute
            match path.to_string().as_str() {
                "crate_path" => {
                    meta_parser! {
                        &nested => [
                            "path" => crate_path,
                        ]
                    }
                    if let Lit::Str(crate_path) = crate_path
                        .ok_or_else(|| Error::new(nested.span(), "expected key 'path'"))?
                    {
                        multiversioned.crate_path = crate_path.parse()?;
                    } else {
                        return Err(Error::new(crate_path.span(), "expected literal string"));
                    }
                }
                "cpu_features_token" => {
                    meta_parser! {
                        &nested => [
                            "name" => name,
                        ]
                    }

                    let name = if let Lit::Str(name) =
                        name.ok_or_else(|| Error::new(nested.span(), "expected key 'path'"))?
                    {
                        name.parse()?
                    } else {
                        return Err(Error::new(name.span(), "expected literal string"));
                    };
                    multiversioned.cpu_token = Some(name);
                }
                "clone" => {
                    meta_parser! {
                        &nested => [
                            "target" => target,
                        ]
                    }
                    multiversioned.specializations.push(Specialization::Clone {
                        target: target
                            .ok_or_else(|| Error::new(nested.span(), "expected key 'target'"))?
                            .try_into()?,
                    });
                }
                "specialize" => {
                    meta_parser! {
                        &nested => [
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
                }
                _ => {
                    multiversioned.func.attrs.push(attr);
                    continue;
                }
            }
        }

        Ok(multiversioned)
    }
}

pub(crate) fn make_multiversioned_fn(func: ItemFn) -> Result<TokenStream, syn::Error> {
    let function: Function = func.try_into()?;
    let dispatcher: Dispatcher = function.try_into()?;
    Ok(dispatcher.to_token_stream())
}
