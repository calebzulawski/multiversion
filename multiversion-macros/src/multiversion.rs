use crate::dispatcher::{DispatchMethod, Dispatcher};
use crate::target::Target;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::collections::HashMap;
use syn::{
    parse::Parser, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma, Error,
    ItemFn, Lit, LitStr, Meta, NestedMeta, ReturnType, Type,
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

fn lit_str(lit: &Lit) -> Result<&LitStr, Error> {
    if let Lit::Str(s) = lit {
        Ok(s)
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
    let mut map = MetaMap::try_from(parser.parse2(attr)?)?;

    let targets = if let Some(targets) = map.try_remove("targets") {
        if let Meta::List(list) = targets {
            list.nested
                .into_iter()
                .map(|x| {
                    if let NestedMeta::Lit(lit) = &x {
                        let target = Target::parse(lit_str(lit)?)?;
                        if target.has_features_specified() {
                            Ok(target)
                        } else {
                            Err(Error::new(x.span(), "target must have features specified"))
                        }
                    } else {
                        Err(Error::new(x.span(), "expected target string"))
                    }
                })
                .collect::<Result<Vec<_>, _>>()
        } else if let Meta::NameValue(nv) = targets {
            match lit_str(&nv.lit)?.value().as_str() {
                "simd" => {
                    let targets = [
                        // "x86_64+avx512f+avx512bw+avx512cd+avx512dq+avx512vl",
                        "x86_64+avx2+fma",
                        "x86_64+sse4.2",
                        // "x86+avx512f+avx512bw+avx512cd+avx512dq+avx512vl",
                        "x86+avx2+fma",
                        "x86+sse4.2",
                        "x86+sse2",
                        "aarch64+neon",
                        // "arm+neon",
                        // "mips+msa",
                        // "mips64+msa",
                        // "powerpc+vsx",
                        // "powerpc+altivec",
                        // "powerpc64+vsx",
                        // "powerpc64+altivec",
                    ];
                    targets
                        .iter()
                        .map(|x| Target::parse(&LitStr::new(x, nv.lit.span())))
                        .collect()
                }
                _ => Err(Error::new(nv.lit.span(), "expected \"simd\"")),
            }
        } else {
            Err(Error::new(
                targets.span(),
                "expected list of function clone targets",
            ))
        }
    } else {
        Err(Error::new(map.span(), "expected `targets`"))
    }?;

    let inner_attrs = if let Some(attrs) = map.try_remove("attrs") {
        if let Meta::List(list) = attrs {
            Ok(list
                .nested
                .into_iter()
                .map(|x| {
                    parse_quote! { #[#x] }
                })
                .collect())
        } else {
            Err(Error::new(attrs.span(), "expected list of attributes"))
        }
    } else {
        Ok(Vec::new())
    }?;

    let dispatcher = map
        .try_remove("dispatcher")
        .map(|x| {
            let s = meta_kv_value(x)?;
            match lit_str(&s)?.value().as_str() {
                "default" => Ok(DispatchMethod::Default),
                "static" => Ok(DispatchMethod::Static),
                "direct" => Ok(DispatchMethod::Direct),
                "indirect" => Ok(DispatchMethod::Indirect),
                _ => Err(Error::new(
                    s.span(),
                    "expected `default`, `static`, `direct`, or `indirect`",
                )),
            }
        })
        .unwrap_or_else(|| Ok(DispatchMethod::Default))?;

    map.finish()?;

    Ok(Dispatcher {
        targets,
        func,
        inner_attrs,
        dispatcher,
    }
    .to_token_stream())
}
