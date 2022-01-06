use crate::safe_inner::process_safe_inner;
use crate::static_dispatch::process_static_dispatch;
use crate::target_cfg::process_target_cfg;
use proc_macro2::TokenStream;
use quote::quote;
use std::convert::TryInto;
use syn::{parse_quote, Attribute, Error, ItemFn, Lit, LitStr, Path, Result};

#[derive(Clone, Debug)]
pub(crate) struct Target {
    architectures: Vec<String>,
    features: Vec<String>,
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.architectures == other.architectures && self.features == other.features
    }
}

impl Target {
    pub(crate) fn parse(s: &LitStr) -> Result<Self> {
        let value = s.value();

        let mut it = value.as_str().split('+');

        let arch_specifier = it
            .next()
            .filter(|x| !x.is_empty())
            .ok_or_else(|| Error::new(s.span(), "expected architecture specifier"))?;
        let architectures = if arch_specifier.starts_with('[') && arch_specifier.ends_with(']') {
            arch_specifier[1..arch_specifier.len() - 1]
                .split('|')
                .map(|x| {
                    if x.is_empty() {
                        Err(Error::new(s.span(), "architecture string cannot be empty"))
                    } else {
                        Ok(x.to_string())
                    }
                })
                .collect::<Result<Vec<_>>>()?
        } else if arch_specifier
            .chars()
            .all(|x| x.is_alphanumeric() || x == '_')
        {
            vec![arch_specifier.to_string()]
        } else {
            return Err(Error::new(s.span(), "invalid architecture specifier"));
        };

        let mut features = it
            .map(|x| {
                if x.is_empty() {
                    Err(Error::new(s.span(), "feature string cannot be empty"))
                } else {
                    Ok(x.to_string())
                }
            })
            .collect::<Result<Vec<_>>>()?;
        features.sort_unstable();
        features.dedup();

        Ok(Self {
            architectures,
            features,
        })
    }

    pub fn arches(&self) -> impl Iterator<Item = &str> {
        self.architectures.iter().map(String::as_str)
    }

    pub fn features_string(&self) -> String {
        self.features.join("_").replace('.', "")
    }

    pub fn has_features_specified(&self) -> bool {
        !self.features.is_empty()
    }

    pub fn target_arch(&self) -> Attribute {
        let arch = self.architectures.iter().map(|x| x.as_str());
        parse_quote! {
            #[cfg(any(#(target_arch = #arch),*))]
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

    pub fn features_detected(&self, crate_path: &Path) -> TokenStream {
        if self.features.is_empty() {
            quote! { true }
        } else {
            let features = &self.features;
            quote! { #crate_path::are_cpu_features_detected!(#(#features),*) }
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
    // Rewrite #[target_cfg] and #[static_dispatch]
    process_target_cfg(target.cloned(), &mut func.block)?;
    process_static_dispatch(&mut func, target)?;

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
    fn parse_single_arch_no_features() {
        let s = LitStr::new("x86", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(target.architectures, vec!["x86"]);
        assert!(target.features.is_empty());
    }

    #[test]
    fn parse_multiple_arch_no_features() {
        let s = LitStr::new("[arm|aarch64|mips|mips64]", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.architectures,
            vec!["arm", "aarch64", "mips", "mips64"]
        );
        assert_eq!(target.features.len(), 0);
    }

    #[test]
    fn parse_single_arch_with_features() {
        let s = LitStr::new("x86_64+sse4.2+xsave", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(target.architectures, vec!["x86_64"]);
        assert_eq!(target.features, vec!["sse4.2", "xsave"]);
    }

    #[test]
    fn parse_multiple_arch_with_features() {
        let s = LitStr::new("[powerpc|powerpc64]+altivec+power8", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(target.architectures, vec!["powerpc", "powerpc64"]);
        assert_eq!(target.features, vec!["altivec", "power8"]);
    }

    #[test]
    fn parse_missing_arch_close() {
        let s = LitStr::new("[x86+sse4.2+xsave", Span::call_site());
        Target::parse(&s).unwrap_err();
    }

    #[test]
    fn parse_malformed_arch() {
        let s = LitStr::new("[x86|]+sse4.2+xsave", Span::call_site());
        Target::parse(&s).unwrap_err();
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
    fn generate_single_target_arch() {
        let s = LitStr::new("x86+avx", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.target_arch(),
            parse_quote! { #[cfg(any(target_arch = "x86"))] }
        );
    }

    #[test]
    fn generate_multiple_target_arch() {
        let s = LitStr::new("[x86|x86_64]+avx", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.target_arch(),
            parse_quote! { #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] }
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
