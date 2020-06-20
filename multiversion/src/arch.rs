/// A trait that indicates presence of a set of CPU features.
///
/// To create types that implement this trait, use [`cpu_type`].
///
/// # Safety
/// Types implementing `Cpu` must uphold the guarantee of only being safely constructible if the
/// required features are supported by the CPU.
///
/// # Example
/// The `Cpu` trait can be used to prove existence of CPU features:
///
/// ```
/// use multiversion::{Cpu, cpu_type, target};
///
/// // A type requiring no CPU features
/// cpu_type! { NoFeatures }
///
/// // A type for detecting AVX and AVX2 support
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// cpu_type! { Avx2 : "avx", "avx2" }
///
/// // This function is unsafe to call without first checking for AVX and AVX2
/// #[target("[x86|x86_64]+avx+avx2")]
/// unsafe fn uses_avx2() {
///     println!("This function uses AVX and AVX2!");
/// }
///
/// // This function is safe because `Avx2` can only be safely constructed by
/// // detecting AVX and AVX2.
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// fn uses_avx2_safe(_: Avx2) {
///     unsafe { uses_avx2() }
/// }
///
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// fn main() {
///     // NoFeatures does not require any features and is always supported
///     assert!(NoFeatures::detect().is_some());
///
///     // Detecting `Avx2` allows calling the function
///     if let Some(features) = Avx2::detect() {
///         uses_avx2_safe(features);
///     }
/// }
/// ```
///
/// [`cpu_type`]: macro.cpu_type.html
pub unsafe trait Cpu: Copy {
    type FeatureList: AsRef<[&'static str]>;

    /// The CPU features required by this type.
    const FEATURES: Self::FeatureList;

    /// Creates the CPU feature type without detecting features.
    unsafe fn new() -> Self;

    /// Returns the CPU feature type if all features are detected, or `None` otherwise.
    fn detect() -> Option<Self>;
}

/// Detects features, using `target_feature` if no_std
#[cfg(feature = "runtime_dispatch")]
#[doc(hidden)]
#[macro_export]
macro_rules! detect_features {
    { $feature:tt } => {
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
    { $first:tt, $($features:tt),+ } => {
        $crate::detect_features!($first) $(&& $crate::detect_features!($features))*
    }
}
#[cfg(not(feature = "runtime_dispatch"))]
#[doc(hidden)]
#[macro_export]
macro_rules! detect_features {
    { $($features:tt),+ } => {
        {
            #[cfg(all( $(target_feature = $features),* ))]
            { true }
            #[cfg(not(all( $(target_feature = $features),* )))]
            { false }
        }
    }
}

/// Counts the number of token trees.
#[doc(hidden)]
#[macro_export]
macro_rules! count_tts {
    { $v:tt } => { 1 };
    { $first:tt $($rest:tt)+ } => { $crate::count_tts! { $($rest)* } + 1 };
}

/// Defines types that indicates presence of a set of CPU features.
///
/// The generated types implement [`Cpu`].  For example, a type `Avx2` that detects AVX and AVX2:
/// ```
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// multiversion::cpu_type! { Avx2 : "avx", "avx2" }
/// ```
///
/// [`Cpu`]: trait.Cpu.html
#[macro_export]
macro_rules! cpu_type {
    { $name:ident } => {
        #[derive(Copy, Clone, Debug, Default)]
        struct $name;

        unsafe impl $crate::Cpu for $name {
            type FeatureList = [&'static str; 0];
            const FEATURES: Self::FeatureList = [];

            unsafe fn new() -> Self {
                Self
            }

            fn detect() -> Option<Self> {
                Some(Self)
            }
        }
    };
    { $name:ident : $($features:tt),+ $(,)? } => {
        #[derive(Copy, Clone, Debug)]
        struct $name($crate::detail::UnsafeConstructible);

        unsafe impl $crate::Cpu for $name {
            type FeatureList = [&'static str; $crate::count_tts!( $($features)* )];
            const FEATURES: Self::FeatureList = [$($features),*];

            unsafe fn new() -> Self {
                Self($crate::detail::UnsafeConstructible::new())
            }

            fn detect() -> Option<Self> {
                if $crate::detect_features!($($features),*) {
                    unsafe { Some(Self::new()) }
                } else {
                    None
                }
            }
        }
    }
}

/// Unstable interfaces for internal use
#[doc(hidden)]
pub mod detail {
    #[derive(Copy, Clone, Debug)]
    pub struct UnsafeConstructible(());

    impl UnsafeConstructible {
        pub unsafe fn new() -> Self {
            Self(())
        }
    }

    pub const fn equal(a: &[u8], b: &[u8]) -> bool {
        const fn equal_impl(a: &[u8], b: &[u8], index: usize) -> bool {
            if a.len() != b.len() {
                false
            } else if index == a.len() {
                true
            } else {
                a[index] == b[index] && equal_impl(a, b, index + 1)
            }
        }

        equal_impl(a, b, 0)
    }

    pub const fn contains(slice: &[&str], value: &str) -> bool {
        const fn contains_impl(slice: &[&str], value: &str, index: usize) -> bool {
            if index == slice.len() {
                false
            } else {
                equal(slice[index].as_bytes(), value.as_bytes())
                    || contains_impl(slice, value, index + 1)
            }
        }

        contains_impl(slice, value, 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn equal() {
        use detail::equal;
        assert!(equal("a".as_bytes(), "a".as_bytes()));
        assert!(!equal("a".as_bytes(), "b".as_bytes()));
        assert!(!equal("a".as_bytes(), "ab".as_bytes()));
        assert!(equal("foo".as_bytes(), "foo".as_bytes()));
        assert!(!equal("foo".as_bytes(), "foobar".as_bytes()));
        assert!(!equal("foo".as_bytes(), "bar".as_bytes()));
    }

    #[test]
    fn contains() {
        use detail::contains;
        let features = ["a", "b", "foo", "bar", "baz!"];
        assert!(contains(&features, "a"));
        assert!(contains(&features, "foo"));
        assert!(contains(&features, "baz!"));
        assert!(!contains(&features, "baz"));
        assert!(!contains(&features, "foobar"));
    }

    #[test]
    fn generic() {
        cpu_type! { Generic }
        assert!(<Generic as Cpu>::FEATURES.is_empty());
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn x86() {
        cpu_type! { Avx : "avx" }
        cpu_type! { Avx2 : "avx", "avx2" }
        assert_eq!(<Avx as Cpu>::FEATURES, ["avx"]);
        assert_eq!(<Avx2 as Cpu>::FEATURES, ["avx", "avx2"]);
    }
}
