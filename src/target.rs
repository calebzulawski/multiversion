use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{Error, LitStr, Result};

#[derive(PartialEq, Clone, Debug)]
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
    architecture: Architecture,
    features: Vec<String>,
}

impl Target {
    pub fn arch_as_str(&self) -> &str {
        return self.architecture.as_str();
    }

    pub fn has_features_specified(&self) -> bool {
        return !self.features.is_empty();
    }

    pub fn target_arch(&self) -> TokenStream {
        let arch = self.architecture.as_str();
        quote! {
            #[cfg(target_arch = #arch)]
        }
    }

    pub fn target_features(&self) -> TokenStream {
        let features = self.features.iter();
        quote! {
            #(#[target_feature(enable = #features)])*
        }
    }

    pub fn features_detected(&self) -> TokenStream {
        if let Some(first_feature) = self.features.first() {
            let rest_features = self.features.iter().skip(1);
            let feature_detector = self.architecture.feature_detector();
            quote! {
                #feature_detector(#first_feature) #( && #feature_detector(#rest_features) )*
            }
        } else {
            quote! {
                true
            }
        }
    }
}

pub(crate) fn parse_target_string(s: &LitStr) -> Result<Vec<Target>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"^(?:\[(?P<arches>\w+(?:\|\w+)*)\]|(?P<arch>\w+))(?P<features>(?:\+\w+)+)?$"
        )
        .unwrap();
    }
    let owned = s.value();
    let captures = RE
        .captures(&owned)
        .ok_or(Error::new(s.span(), "invalid target string"))?;
    let features = captures.name("features").map_or(Vec::new(), |x| {
        x.as_str()
            .split('+')
            .skip(1)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
    });
    if let Some(arch) = captures.name("arch") {
        Ok(vec![Target {
            architecture: Architecture::new(arch.as_str(), s.span())?,
            features: features,
        }])
    } else {
        captures
            .name("arches")
            .unwrap()
            .as_str()
            .split('|')
            .map(|arch| {
                Ok(Target {
                    architecture: Architecture::new(arch, s.span())?,
                    features: features.clone(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_single_arch_no_features() {
        let s = LitStr::new("x86", Span::call_site());
        let targets = parse_target_string(&s).unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].architecture, Architecture::X86);
        assert_eq!(targets[0].features.len(), 0);
    }

    #[test]
    fn parse_multiple_arch_no_features() {
        let s = LitStr::new("[arm|aarch64|mips|mips64]", Span::call_site());
        let targets = parse_target_string(&s).unwrap();
        assert_eq!(targets.len(), 4);
        assert_eq!(targets[0].architecture, Architecture::Arm);
        assert_eq!(targets[0].features.len(), 0);
        assert_eq!(targets[1].architecture, Architecture::Aarch64);
        assert_eq!(targets[1].features.len(), 0);
        assert_eq!(targets[2].architecture, Architecture::Mips);
        assert_eq!(targets[2].features.len(), 0);
        assert_eq!(targets[3].architecture, Architecture::Mips64);
        assert_eq!(targets[3].features.len(), 0);
    }

    #[test]
    fn parse_single_arch_with_features() {
        let s = LitStr::new("x86_64+avx2+xsave", Span::call_site());
        let targets = parse_target_string(&s).unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].architecture, Architecture::X86_64);
        assert_eq!(targets[0].features.len(), 2);
        assert_eq!(targets[0].features[0], "avx2");
        assert_eq!(targets[0].features[1], "xsave");
    }

    #[test]
    fn parse_multiple_arch_with_features() {
        let s = LitStr::new("[powerpc|powerpc64]+altivec+power8", Span::call_site());
        let targets = parse_target_string(&s).unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].architecture, Architecture::PowerPC);
        assert_eq!(targets[0].features.len(), 2);
        assert_eq!(targets[0].features[0], "altivec");
        assert_eq!(targets[0].features[1], "power8");
        assert_eq!(targets[1].architecture, Architecture::PowerPC64);
        assert_eq!(targets[1].features.len(), 2);
        assert_eq!(targets[1].features[0], "altivec");
        assert_eq!(targets[1].features[1], "power8");
    }

    #[test]
    fn generate_target_arch() {
        let s = LitStr::new("x86+avx", Span::call_site());
        let target = parse_target_string(&s).unwrap().pop().unwrap();
        assert_eq!(
            target.target_arch().to_string(),
            quote! { #[cfg(target_arch = "x86")] }.to_string()
        );
    }

    #[test]
    fn generate_single_target_feature() {
        let s = LitStr::new("x86+avx", Span::call_site());
        let target = parse_target_string(&s).unwrap().pop().unwrap();
        assert_eq!(
            target.target_features().to_string(),
            quote! { #[target_feature(enable = "avx")] }.to_string()
        );
    }

    #[test]
    fn generate_multiple_target_feature() {
        let s = LitStr::new("x86+avx+xsave", Span::call_site());
        let target = parse_target_string(&s).unwrap().pop().unwrap();
        assert_eq!(
            target.target_features().to_string(),
            quote! { #[target_feature(enable = "avx")] #[target_feature(enable = "xsave")] }
                .to_string()
        );
    }

    #[test]
    fn generate_single_features_detect() {
        let s = LitStr::new("x86+avx", Span::call_site());
        let target = parse_target_string(&s).unwrap().pop().unwrap();
        assert_eq!(
            target.features_detected().to_string(),
            quote! { is_x86_feature_detected!("avx") }.to_string()
        );
    }

    #[test]
    fn generate_multiple_features_detect() {
        let s = LitStr::new("x86+avx+xsave", Span::call_site());
        let target = parse_target_string(&s).unwrap().pop().unwrap();
        assert_eq!(
            target.features_detected().to_string(),
            quote! { is_x86_feature_detected!("avx") && is_x86_feature_detected!("xsave") }
                .to_string()
        );
    }
}
