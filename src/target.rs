use crate::safe_inner::process_safe_inner;
use crate::static_dispatch::process_static_dispatch;
use crate::target_cfg::process_target_cfg;
use once_cell::sync::Lazy;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use std::convert::TryInto;
use syn::{
    parse::Parse, parse::ParseStream, parse_quote, spanned::Spanned, Attribute, Error, ItemFn, Lit,
    LitStr, Result,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
enum Architecture {
    X86,
    X86_64,
    Arm,
    Aarch64,
    Mips,
    Mips64,
    PowerPC,
    PowerPC64,
}

impl Architecture {
    fn new(name: &str, span: Span) -> Result<Self> {
        match name {
            "x86" => Ok(Architecture::X86),
            "x86_64" => Ok(Architecture::X86_64),
            "arm" => Ok(Architecture::Arm),
            "aarch64" => Ok(Architecture::Aarch64),
            "mips" => Ok(Architecture::Mips),
            "mips64" => Ok(Architecture::Mips64),
            "powerpc" => Ok(Architecture::PowerPC),
            "powerpc64" => Ok(Architecture::PowerPC64),
            _ => Err(Error::new(span, format!("unknown architecture '{}'", name))),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Architecture::X86 => "x86",
            Architecture::X86_64 => "x86_64",
            Architecture::Arm => "arm",
            Architecture::Aarch64 => "aarch64",
            Architecture::Mips => "mips",
            Architecture::Mips64 => "mips64",
            Architecture::PowerPC => "powerpc",
            Architecture::PowerPC64 => "powerpc64",
        }
    }

    #[cfg(feature = "runtime_dispatch")]
    fn feature_detector(&self) -> TokenStream {
        match self {
            Architecture::X86 => quote! { is_x86_feature_detected! },
            Architecture::X86_64 => quote! { is_x86_feature_detected! },
            Architecture::Arm => quote! { is_arm_feature_detected! },
            Architecture::Aarch64 => quote! { is_aarch64_feature_detected! },
            Architecture::Mips => quote! { is_mips_feature_detected! },
            Architecture::Mips64 => quote! { is_mips64_feature_detected! },
            Architecture::PowerPC => quote! { is_powerpc_feature_detected! },
            Architecture::PowerPC64 => quote! { is_powerpc64_feature_detected! },
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Target {
    architectures: Vec<Architecture>,
    features: Vec<String>,
    span: proc_macro2::Span,
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.architectures == other.architectures && self.features == other.features
    }
}

impl Target {
    pub(crate) fn parse(s: &LitStr) -> Result<Self> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"^(?:\[(?P<arches>\w+(?:\|\w+)*)\]|(?P<arch>\w+))(?P<features>(?:\+[\w.]+)+)?$",
            )
            .unwrap()
        });
        let owned = s.value();
        let captures = RE
            .captures(&owned)
            .ok_or_else(|| Error::new(s.span(), "invalid target string"))?;
        let features = captures.name("features").map_or(Vec::new(), |x| {
            let mut v = x
                .as_str()
                .split('+')
                .skip(1)
                .map(|x| x.to_string())
                .collect::<Vec<_>>();
            v.sort_unstable();
            v.dedup();
            v
        });
        if let Some(arch) = captures.name("arch") {
            Ok(Self {
                architectures: vec![Architecture::new(arch.as_str(), s.span())?],
                features,
                span: s.span(),
            })
        } else {
            let mut arches = captures
                .name("arches")
                .ok_or_else(|| Error::new(s.span(), "invalid target string"))?
                .as_str()
                .split('|')
                .map(|arch| Architecture::new(arch, s.span()))
                .collect::<Result<Vec<_>>>()?;
            arches.sort_unstable();
            arches.dedup();
            Ok(Self {
                architectures: arches,
                features,
                span: s.span(),
            })
        }
    }

    pub fn target_string(&self) -> LitStr {
        let arches = if self.architectures.len() > 1 {
            format!(
                "[{}]",
                self.architectures
                    .iter()
                    .map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join("|")
            )
        } else {
            self.architectures.first().unwrap().as_str().to_string()
        };
        let string = if self.features.is_empty() {
            arches
        } else {
            format!("{}+{}", arches, self.features.join("+"))
        };
        LitStr::new(&string, self.span)
    }

    pub fn arches_as_str(&self) -> Vec<&'static str> {
        self.architectures.iter().map(|x| x.as_str()).collect()
    }

    pub fn features_string(&self) -> String {
        self.features.join("_").replace(".", "")
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

    pub fn features_detected(&self) -> TokenStream {
        if self.features.is_empty() {
            quote! {
                true
            }
        } else {
            let arches = self.architectures.iter().map(|x| {
                let arch = x.as_str();
                let features = self.features.iter();
                #[cfg(feature = "runtime_dispatch")]
                {
                    let feature_detector = x.feature_detector();
                    quote! {
                        #[cfg(target_arch = #arch)]
                        {
                            #( #feature_detector(#features) )&&*
                        }
                    }
                }
                #[cfg(not(feature = "runtime_dispatch"))]
                quote! {
                    #[cfg(target_arch = #arch)]
                    {
                        cfg!(all(#(target_feature = #features),*))
                    }
                }
            });
            quote! { { #(#arches)* } }
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

pub(crate) fn make_target_fn(target: Option<Lit>, mut func: ItemFn) -> Result<TokenStream> {
    let target = target.as_ref().map(|s| s.try_into()).transpose()?;

    // Rewrite #[target_cfg] and #[static_dispatch]
    process_target_cfg(target.clone(), &mut func.block)?;
    process_static_dispatch(&mut func, target.as_ref())?;

    // Create the function
    if let Some(target) = target {
        let target_arch = target.target_arch();
        let target_feature = target.target_feature();
        func = parse_quote! { #target_arch #(#target_feature)* #func };
    }
    let functions = process_safe_inner(func)?;
    Ok(quote! { #(#functions)* })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_single_arch_no_features() {
        let s = LitStr::new("x86", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(target.architectures, vec![Architecture::X86]);
        assert_eq!(target.features.is_empty(), true);
    }

    #[test]
    fn parse_multiple_arch_no_features() {
        let s = LitStr::new("[arm|aarch64|mips|mips64]", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.architectures,
            vec![
                Architecture::Arm,
                Architecture::Aarch64,
                Architecture::Mips,
                Architecture::Mips64
            ]
        );
        assert_eq!(target.features.len(), 0);
    }

    #[test]
    fn parse_single_arch_with_features() {
        let s = LitStr::new("x86_64+sse4.2+xsave", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(target.architectures, vec![Architecture::X86_64]);
        assert_eq!(target.features, vec!["sse4.2", "xsave"]);
    }

    #[test]
    fn parse_multiple_arch_with_features() {
        let s = LitStr::new("[powerpc|powerpc64]+altivec+power8", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.architectures,
            vec![Architecture::PowerPC, Architecture::PowerPC64]
        );
        assert_eq!(target.features, vec!["altivec", "power8"]);
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

    #[test]
    fn generate_single_features_detect() {
        let s = LitStr::new("x86+avx", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.features_detected().to_string(),
            {
                #[cfg(feature = "runtime_dispatch")]
                {
                    quote! {
                        {
                            #[cfg(target_arch = "x86")]
                            {
                                is_x86_feature_detected!("avx")
                            }
                        }
                    }
                }
                #[cfg(not(feature = "runtime_dispatch"))]
                {
                    quote! {
                        {
                            #[cfg(target_arch = "x86")]
                            {
                                cfg!(all(target_feature = "avx"))
                            }
                        }
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn generate_multiple_features_detect() {
        let s = LitStr::new("[x86|x86_64]+avx+xsave", Span::call_site());
        let target = Target::parse(&s).unwrap();
        assert_eq!(
            target.features_detected().to_string(),
            {
                #[cfg(feature = "runtime_dispatch")]
                {
                    quote! {
                        {
                            #[cfg(target_arch = "x86")]
                            {
                                is_x86_feature_detected!("avx") && is_x86_feature_detected!("xsave")
                            }
                            #[cfg(target_arch = "x86_64")]
                            {
                                is_x86_feature_detected!("avx") && is_x86_feature_detected!("xsave")
                            }
                        }
                    }
                }
                #[cfg(not(feature = "runtime_dispatch"))]
                {
                    quote! {
                        {
                            #[cfg(target_arch = "x86")]
                            {
                                cfg!(all(target_feature = "avx", target_feature = "xsave"))
                            }
                            #[cfg(target_arch = "x86_64")]
                            {
                                cfg!(all(target_feature = "avx", target_feature = "xsave"))
                            }
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
