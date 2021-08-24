use crate::dispatcher::Dispatcher;
use crate::target::Target;
use crate::util;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};
use syn::{
    parse::Parser, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma, Error,
    ItemFn, Lit, LitStr, Meta, NestedMeta, Path, ReturnType, Type,
};

fn meta_path_string(meta: &Meta) -> Result<String, Error> {
    meta.path()
        .get_ident()
        .ok_or_else(|| Error::new(meta.path().span(), "expected identifier, got path"))
        .map(ToString::to_string)
}

fn meta_kv_value(meta: Meta) -> Result<Lit, Error> {
    if let Meta::NameValue(nv) = meta {
        Ok(nv.lit)
    } else {
        Err(Error::new(meta.span(), "expected name-value pair"))
    }
}

fn meta_map(meta: Meta) -> Result<MetaMap, Error> {
    if let Meta::List(l) = meta {
        l.nested.try_into()
    } else {
        Err(Error::new(meta.span(), "unexpected value"))
    }
}

fn lit_str(lit: Lit) -> Result<LitStr, Error> {
    if let Lit::Str(s) = lit {
        Ok(s)
    } else {
        Err(Error::new(lit.span(), "expected string"))
    }
}

fn lit_bool(lit: Lit) -> Result<bool, Error> {
    if let Lit::Bool(b) = lit {
        Ok(b.value)
    } else {
        Err(Error::new(lit.span(), "expected string"))
    }
}

struct MetaMap {
    map: HashMap<String, Meta>,
    span: Span,
}

impl TryFrom<Punctuated<NestedMeta, Comma>> for MetaMap {
    type Error = Error;

    fn try_from(meta: Punctuated<NestedMeta, Comma>) -> Result<Self, Self::Error> {
        let mut map = HashMap::new();
        let span = meta.span();
        for meta in meta.into_iter() {
            let meta = if let NestedMeta::Meta(m) = meta {
                Ok(m)
            } else {
                Err(Error::new(meta.span(), "expected meta, got literal"))
            }?;

            let key = meta_path_string(&meta)?;
            if map.contains_key(&key) {
                return Err(Error::new(meta.path().span(), "key already provided"));
            }
            map.insert(key, meta);
        }
        Ok(Self { map, span })
    }
}

impl MetaMap {
    fn try_remove(&mut self, key: &str) -> Option<Meta> {
        self.map.remove(key)
    }

    fn remove(&mut self, key: &str) -> Result<Meta, Error> {
        self.map
            .remove(key)
            .ok_or_else(|| Error::new(self.span, format!("expected key `{}`", key)))
    }

    fn span(&self) -> Span {
        self.span
    }

    fn finish(self) -> Result<(), Error> {
        if let Some((_, v)) = self.map.into_iter().next() {
            Err(Error::new(v.span(), "unexpected key"))
        } else {
            Ok(())
        }
    }
}

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

impl TryFrom<Meta> for Specialization {
    type Error = Error;

    fn try_from(meta: Meta) -> Result<Self, Self::Error> {
        match meta_path_string(&meta)?.as_str() {
            "clone" => Ok(Self::Clone {
                target: Target::parse(&lit_str(meta_kv_value(meta)?)?)?,
            }),
            "alternative" => {
                let mut map = meta_map(meta)?;
                let target = Target::parse(&lit_str(meta_kv_value(map.remove("target")?)?)?)?;
                let func = lit_str(meta_kv_value(map.remove("fn")?)?)?.parse()?;
                let is_unsafe = map
                    .try_remove("unsafe")
                    .map(|x| lit_bool(meta_kv_value(x)?))
                    .unwrap_or(Ok(false))?;
                map.finish()?;
                Ok(Self::Override {
                    target,
                    func,
                    is_unsafe,
                })
            }
            _ => Err(Error::new(meta.span(), "expected `clone` or `alternative`")),
        }
    }
}

struct Function {
    specializations: Vec<Specialization>,
    func: ItemFn,
    associated: bool,
    crate_path: Path,
}

impl Function {
    fn new(attr: Punctuated<NestedMeta, Comma>, func: ItemFn) -> Result<Self, Error> {
        let mut map = MetaMap::try_from(attr)?;

        let specializations = if let Some(clones) = map.try_remove("clones") {
            if let Meta::List(list) = clones {
                list.nested
                    .into_iter()
                    .map(|x| {
                        if let NestedMeta::Lit(lit) = x {
                            let target = Target::parse(&lit_str(lit)?)?;
                            Ok(Specialization::Clone { target })
                        } else {
                            Err(Error::new(x.span(), "expected target string"))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            } else {
                Err(Error::new(
                    clones.span(),
                    "expected list of function clone targets",
                ))
            }
        } else if let Some(versions) = map.try_remove("versions") {
            if let Meta::List(list) = versions {
                list.nested
                    .into_iter()
                    .map(|x| {
                        if let NestedMeta::Meta(meta) = x {
                            meta.try_into()
                        } else {
                            Err(Error::new(x.span(), "unexpected value"))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            } else {
                Err(Error::new(
                    versions.span(),
                    "expected list of function versions",
                ))
            }
        } else {
            Err(Error::new(map.span(), "expected `clones` or `versions`"))
        }?;

        let associated = map
            .try_remove("associated_fn")
            .map(|x| lit_bool(meta_kv_value(x)?))
            .unwrap_or(Ok(func.sig.receiver().is_some()))?;
        let crate_path = map
            .try_remove("crate_path")
            .map(|x| lit_str(meta_kv_value(x)?)?.parse())
            .unwrap_or(Ok(parse_quote!(multiversion)))?;
        map.finish()?;
        Ok(Self {
            specializations,
            associated,
            crate_path,
            func,
        })
    }
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

pub(crate) fn make_multiversioned_fn(
    attr: TokenStream,
    func: ItemFn,
) -> Result<TokenStream, syn::Error> {
    if let ReturnType::Type(_, ty) = &func.sig.output {
        if let Type::ImplTrait(_) = **ty {
            return Err(Error::new(
                ty.span(),
                "cannot multiversion function with `impl Trait` return type",
            ));
        }
    }

    let parser = Punctuated::parse_terminated;
    let attr = parser.parse2(attr)?;
    let function = Function::new(attr, func)?;
    let dispatcher: Dispatcher = function.try_into()?;
    Ok(dispatcher.to_token_stream())
}
