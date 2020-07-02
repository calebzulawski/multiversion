pub struct CpuFeatures(&'static [&'static str]);

const fn const_slice_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

impl CpuFeatures {
    pub const fn none() -> Self {
        // FIXME: remove unnecessary const once #73862 merged
        const NO_FEATURES: &'static [&'static str] = &[];
        Self(NO_FEATURES)
    }

    pub const unsafe fn new(features: &'static [&'static str]) -> Self {
        Self(features)
    }

    pub const fn supports(&self, feature: &str) -> bool {
        let mut i = 0;
        while i < self.0.len() {
            if const_slice_eq(self.0[i].as_bytes(), feature.as_bytes()) {
                return true;
            }
            i += 1;
        }
        false
    }
}

macro_rules! detect_cpu_features {
    { $($feature:tt),* $(,)? } => {
        {
            if $crate::are_cpu_features_detected!($($feature),*) {
                Some(CpuFeatures::new(&[$($feature),*]))
            } else {
                None
            }
        }
    }
}

/// Detects CPU features.
///
/// When the `std` feature is enabled, this macro operates like the standard library detection
/// macro for the current target (e.g. [`is_x86_feature_detected`]), but accepts multiple arguments.
///
/// When the `std` feature is not enabled, this macro detects if the feature is
/// enabled during compilation, using the [`cfg`] attribute.
///
/// [`is_x86_feature_detected`]: https://doc.rust-lang.org/std/macro.is_x86_feature_detected.html
/// [`cfg`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_feature
#[cfg(any(feature = "std", doc))]
#[macro_export]
macro_rules! are_cpu_features_detected {
    { $feature:tt $(,)? } => {
        {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            { is_x86_feature_detected!($feature) }
            #[cfg(target_arch = "arm")]
            { is_arm_feature_detected!($feature) }
            #[cfg(target_arch = "aarch64")]
            { is_aarch64_feature_detected!($feature) }
            #[cfg(target_arch = "powerpc")]
            { is_powerpc_feature_detected!($feature) }
            #[cfg(target_arch = "powerpc64")]
            { is_powerpc64_feature_detected!($feature) }
            #[cfg(target_arch = "mips")]
            { is_mips_feature_detected!($feature) }
            #[cfg(target_arch = "mips64")]
            { is_mips64_feature_detected!($feature) }
            #[cfg(not(any(
                target_arch = "x86",
                target_arch = "x86_64",
                target_arch = "arm",
                target_arch = "aarch64",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "mips",
                target_arch = "mips64",
            )))]
            { compile_error!("Unsupported architecture. Expected x86, x86_64, arm, aarch64, powerpc, powerpc64, mips, or mips64.") }
        }
    };
    { $first:tt, $($features:tt),+ $(,)? } => {
        $crate::are_cpu_features_detected!($first) $(&& $crate::are_cpu_features_detected!($features))*
    }
}
#[cfg(not(any(feature = "std", doc)))]
#[macro_export]
macro_rules! are_cpu_features_detected {
    { $($features:tt),+ } => {
        {
            #[cfg(all( $(target_feature = $features),* ))]
            { true }
            #[cfg(not(all( $(target_feature = $features),* )))]
            { false }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn equal() {
        assert!(const_slice_eq("a".as_bytes(), "a".as_bytes()));
        assert!(!const_slice_eq("a".as_bytes(), "b".as_bytes()));
        assert!(!const_slice_eq("a".as_bytes(), "ab".as_bytes()));
        assert!(const_slice_eq("foo".as_bytes(), "foo".as_bytes()));
        assert!(!const_slice_eq("foo".as_bytes(), "foobar".as_bytes()));
        assert!(!const_slice_eq("foo".as_bytes(), "bar".as_bytes()));
    }

    #[test]
    fn contains() {
        let features = unsafe { CpuFeatures::new(&["a", "b", "foo", "bar", "baz!"]) };
        assert!(features.supports("a"));
        assert!(features.supports("foo"));
        assert!(features.supports("baz!"));
        assert!(!features.supports("baz"));
        assert!(!features.supports("foobar"));
    }
}
