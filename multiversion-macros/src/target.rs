use crate::safe_inner::process_safe_inner;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::convert::TryInto;
use syn::{parse_quote, Attribute, Error, ItemFn, Lit, LitStr, Result};

include!(concat!(env!("OUT_DIR"), "/implied_features.rs"));

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Target {
    architecture: String,
    features: Vec<String>,
}

impl Target {
    pub(crate) fn parse(s: &LitStr) -> Result<Self> {
        let value = s.value();

        let mut it = value.as_str().split('+');

        let architecture = it
            .next()
            .ok_or_else(|| Error::new(s.span(), "expected architecture specifier"))?
            .to_string();

        if architecture.is_empty()
            || !architecture
                .chars()
                .all(|x| x.is_alphanumeric() || x == '_')
        {
            return Err(Error::new(s.span(), "invalid architecture specifier"));
        };

        let specified_features = it
            .map(|x| {
                if x.is_empty() {
                    Err(Error::new(s.span(), "feature string cannot be empty"))
                } else {
                    Ok(x.to_string())
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let mut features = Vec::new();
        for feature in specified_features {
            features.extend(
                implied_features(&architecture, &feature)
                    .iter()
                    .map(ToString::to_string),
            );
        }
        features.sort_unstable();
        features.dedup();

        Ok(Self {
            architecture,
            features,
        })
    }

    pub fn features_string(&self) -> String {
        self.features.join("_").replace('.', "")
    }

    pub fn has_features_specified(&self) -> bool {
        !self.features.is_empty()
    }

    pub fn target_arch(&self) -> Attribute {
        let arch = &self.architecture;
        parse_quote! {
            #[cfg(target_arch = #arch)]
        }
    }

    pub fn target_feature(&self) -> Vec<Attribute> {
        self.features
            .iter()
            .map(|feature| {
                parse_quote! {
                    #[target_feature(enable = #feature)]
                }
            })
            .collect()
    }

    pub fn fn_attrs(&self) -> Vec<Attribute> {
        let mut attrs = self.target_feature();
        attrs.push(self.target_arch());
        attrs
    }

    pub fn features_enabled(&self) -> TokenStream {
        let feature = self.features.iter();
        quote! {
            true #( && core::cfg!(target_feature = #feature) )*
        }
    }

    pub fn features_detected(&self) -> TokenStream {
        let feature = self.features.iter();
        let is_feature_detected = format_ident!(
            "is_{}_feature_detected",
            match self.architecture.as_str() {
                "x86_64" => "x86",
                f => f,
            }
        );
        quote! {
            true #( && std::#is_feature_detected!(#feature) )*
        }
    }
}

impl std::convert::TryFrom<&Lit> for Target {
    type Error = Error;

    fn try_from(lit: &Lit) -> Result<Self> {
        match lit {
            Lit::Str(s) => Self::parse(s),
            _ => Err(Error::new(lit.span(), "expected literal string")),
        }
    }
}

pub(crate) fn make_target_fn(target: Option<Lit>, func: ItemFn) -> Result<TokenStream> {
    let target = target.as_ref().map(|s| s.try_into()).transpose()?;
    let functions = make_target_fn_items(target.as_ref(), func)?;
    Ok(quote! { #(#functions)* })
}

pub(crate) fn make_target_fn_items(
    target: Option<&Target>,
    mut func: ItemFn,
) -> Result<Vec<ItemFn>> {
    // Create the function
    if let Some(target) = target {
        let target_arch = target.target_arch();
        let target_feature = target.target_feature();
        func = parse_quote! { #target_arch #(#target_feature)* #func };
    }
    process_safe_inner(func)
}

#[cfg(test)]
mod test {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn parse_no_features() {
        let s = LitStr::new("x86", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(target.architecture, "x86");
        assert!(target.features.is_empty());
    }

    #[test]
    fn parse_features() {
        let s = LitStr::new("x86_64+sse4.2+xsave", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(target.architecture, "x86_64");
        assert_eq!(target.features, vec!["sse4.2", "xsave"]);
    }

    #[test]
    fn parse_extra_plus_start() {
        let s = LitStr::new("+x86+sse4.2+xsave", Span::call_site());
        Target::parse(&s).unwrap_err();
    }

    #[test]
    fn parse_extra_plus_end() {
        let s = LitStr::new("x86+sse4.2+xsave+", Span::call_site());
        Target::parse(&s).unwrap_err();
    }

    #[test]
    fn generate_target_arch() {
        let s = LitStr::new("x86+avx", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.target_arch(),
            parse_quote! { #[cfg(target_arch = "x86")] }
        );
    }

    #[test]
    fn generate_single_target_feature() {
        let s = LitStr::new("x86+avx", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.target_feature(),
            vec![parse_quote! { #[target_feature(enable = "avx")] }]
        );
    }

    #[test]
    fn generate_multiple_target_feature() {
        let s = LitStr::new("x86+avx+xsave", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.target_feature(),
            vec![
                parse_quote! { #[target_feature(enable = "avx")] },
                parse_quote! { #[target_feature(enable = "xsave")] }
            ]
        );
    }
}
