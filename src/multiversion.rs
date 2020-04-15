use crate::dispatcher::Dispatcher;
use crate::target::Target;
use crate::util;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::convert::{TryFrom, TryInto};
use syn::{
    parse_quote, spanned::Spanned, Error, Ident, ItemFn, Lit, Meta, MetaList, NestedMeta, Path,
};

// Parses an attribute meta into Options
macro_rules! meta_parser {
    {
        $list:expr => [$($key:literal => $var:ident,)*]
    } => {
        $(let mut $var = None;)*
        for element in $list {
            match element {
                NestedMeta::Meta(meta) => match meta {
                    Meta::NameValue(nv) => {
                        let name = nv.path.get_ident()
                            .ok_or(Error::new(nv.path.span(), "unexpected key"))?
                            .to_string();
                        match name.as_str() {
                            $(
                                $key => {
                                    if $var.is_none() {
                                        $var = Some(&nv.lit);
                                    } else {
                                        Err(Error::new(nv.path.span(), "key already provided"))?
                                    }
                                }
                            )*
                            _ => Err(Error::new(nv.path.span(), "unexpected key"))?
                        };
                    }
                    _ => Err(Error::new(meta.span(), "expected name-value pair"))?
                }
                NestedMeta::Lit(lit) => Err(Error::new(
                    lit.span(),
                    "unexpected literal, expected name-value pair",
                ))?,
            }
        }
    }
}

enum Specialization {
    Clone {
        target: Target,
        func: Option<Ident>,
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
}

impl TryFrom<Function> for Dispatcher {
    type Error = Error;

    fn try_from(func: Function) -> Result<Self, Self::Error> {
        let normalized_sig = util::normalize_signature(&func.func.sig)?;
        let args = util::args_from_signature(&normalized_sig)?;
        let fn_params = util::fn_params(&func.func.sig);
        Ok(Self {
            specializations: func
                .specializations
                .iter()
                .map(|specialization| match specialization {
                    Specialization::Clone { target, .. } => crate::dispatcher::Specialization {
                        target: target.clone(),
                        block: func.func.block.as_ref().clone(),
                        normalize: false,
                    },
                    Specialization::Override {
                        target,
                        func,
                        is_unsafe,
                    } => crate::dispatcher::Specialization {
                        target: target.clone(),
                        block: if *is_unsafe {
                            parse_quote! {
                                {
                                    unsafe { #func::<#(#fn_params),*>(#(#args),*) }
                                }
                            }
                        } else {
                            parse_quote! {
                                {
                                    #func::<#(#fn_params),*>(#(#args),*)
                                }
                            }
                        },
                        normalize: true,
                    },
                })
                .collect(),
            attrs: func.func.attrs,
            vis: func.func.vis,
            sig: func.func.sig,
            default: *func.func.block,
        })
    }
}

impl TryFrom<ItemFn> for Function {
    type Error = Error;

    fn try_from(func: ItemFn) -> Result<Self, Self::Error> {
        let attrs = func.attrs;
        let mut multiversioned = Function {
            specializations: Vec::new(),
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
                "clone" => {
                    meta_parser! {
                        &nested => [
                            "target" => target,
                            "fn" => func,
                        ]
                    }
                    multiversioned.specializations.push(Specialization::Clone {
                        target: target
                            .ok_or(Error::new(nested.span(), "expected key 'target'"))?
                            .try_into()?,
                        func: func
                            .map(|lit| match lit {
                                Lit::Str(s) => s.parse(),
                                _ => Err(Error::new(lit.span(), "expected literal string")),
                            })
                            .transpose()?,
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
                                .ok_or(Error::new(nested.span(), "expected key 'target'"))?
                                .try_into()?,
                            func: match func
                                .ok_or(Error::new(nested.span(), "expected key 'fn'"))?
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
