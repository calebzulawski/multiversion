use crate::dispatcher::Dispatcher;
use crate::target::Target;
use crate::util;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::convert::TryInto;
use syn::{
    parse::Parse, parse::ParseStream, parse_quote, punctuated::Punctuated, spanned::Spanned, token,
    Error, Ident, ItemFn, Lit, LitStr, Meta, MetaList, NestedMeta, Path,
};

enum Specialization2 {
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

enum ParseError {
    WrongAttr,
    Invalid(Error),
}

impl std::convert::From<Error> for ParseError {
    fn from(e: Error) -> Self {
        Self::Invalid(e)
    }
}

macro_rules! meta_parser {
    {
        $list:ident = [$($key:literal => $var:ident,)*]
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

impl std::convert::TryFrom<Meta> for Specialization2 {
    type Error = ParseError;

    fn try_from(meta: Meta) -> Result<Self, Self::Error> {
        if let Meta::List(MetaList { path, nested, .. }) = &meta {
            if let Some(attr) = path.get_ident() {
                match attr.to_string().as_str() {
                    "clone" => {
                        meta_parser! {
                            nested = [
                                "target" => target,
                                "fn" => func,
                            ]
                        }
                        Ok(Specialization2::Clone {
                            target: target
                                .ok_or(Error::new(nested.span(), "expected key 'target'"))?
                                .try_into()?,
                            func: func
                                .map(|lit| match lit {
                                    Lit::Str(s) => s.parse(),
                                    _ => Err(Error::new(lit.span(), "expected literal string")),
                                })
                                .transpose()?,
                        })
                    }
                    "override" => {
                        meta_parser! {
                            nested = [
                                "target" => target,
                                "fn" => func,
                                "unsafe" => is_unsafe,
                            ]
                        }
                        Ok(Specialization2::Override {
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
                        })
                    }
                    _ => Err(ParseError::WrongAttr),
                }
            } else {
                Err(ParseError::WrongAttr)
            }
        } else {
            Err(ParseError::WrongAttr)
        }
    }
}

pub(crate) struct Specialization {
    target: Target,
    _fat_arrow_token: token::FatArrow,
    unsafety: Option<token::Unsafe>,
    path: Path,
}

impl Parse for Specialization {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let target_str = input.parse::<LitStr>()?;
        Ok(Self {
            target: Target::parse(&target_str)?,
            _fat_arrow_token: input.parse()?,
            unsafety: input.parse()?,
            path: input.parse()?,
        })
    }
}

pub(crate) struct Config {
    specializations: Punctuated<Specialization, token::Comma>,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        Ok(Self {
            specializations: input.parse_terminated(Specialization::parse)?,
        })
    }
}

pub(crate) fn make_multiversioned_fn(
    config: Config,
    func: ItemFn,
) -> Result<TokenStream, syn::Error> {
    let normalized_sig = util::normalize_signature(&func.sig)?;
    let args = util::args_from_signature(&normalized_sig)?;
    let fn_params = util::fn_params(&func.sig);
    Ok(Dispatcher {
        attrs: func.attrs,
        vis: func.vis,
        sig: func.sig.clone(),
        specializations: config
            .specializations
            .iter()
            .map(
                |Specialization {
                     target,
                     path,
                     unsafety,
                     ..
                 }| crate::dispatcher::Specialization {
                    target: target.clone(),
                    block: if unsafety.is_some() {
                        parse_quote! {
                            {
                                unsafe { #path::<#(#fn_params),*>(#(#args),*) }
                            }
                        }
                    } else {
                        parse_quote! {
                            {
                                #path::<#(#fn_params),*>(#(#args),*)
                            }
                        }
                    },
                    normalize: true,
                },
            )
            .collect(),
        default: *func.block,
    }
    .to_token_stream())
}
