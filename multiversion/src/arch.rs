/// A trait that indicates presence of a set of CPU features.
///
/// To create types that implement this trait, use [`cpu_token_type`].
///
/// # Safety
/// Types implementing `CpuToken` must uphold the guarantee of only being safely constructible if the
/// required features are supported by the CPU.
///
/// # Example
/// The `CpuToken` trait can be used to prove existence of CPU features:
///
/// ```
/// use multiversion::{CpuToken, cpu_token_type, target};
///
/// // A type requiring no CPU features
/// cpu_token_type! { NoFeatures }
///
/// // A type for detecting AVX and AVX2 support
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// cpu_token_type! { Avx2 : "avx", "avx2" }
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
/// [`cpu_token_type`]: macro.cpu_token_type.html
pub unsafe trait CpuToken: Copy {
    /// The CPU features required by this type.
    const FEATURES: &'static [&'static str];

    /// Creates the CPU feature type without detecting features.
    unsafe fn new() -> Self;

    /// Returns the CPU feature type if all features are detected, or `None` otherwise.
    fn detect() -> Option<Self>;

    /// Converts a token into a token containing a subset of available CPU features.
    fn into_subset<T: CpuToken>() -> Option<T> {
        if detail::subset(Self::FEATURES, T::FEATURES) {
            unsafe { Some(T::new()) }
        } else {
            None
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

/// Defines types that indicates presence of a set of CPU features.
///
/// The generated types implement [`CpuToken`].  For example, a type `Avx2` that detects AVX and AVX2:
/// ```
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// multiversion::cpu_token_type! { Avx2 : "avx", "avx2" }
/// ```
///
/// [`CpuToken`]: trait.CpuToken.html
#[macro_export]
macro_rules! cpu_token_type {
    { $name:ident } => {
        #[derive(Copy, Clone, Debug, Default)]
        struct $name;

        unsafe impl $crate::CpuToken for $name {
            const FEATURES: &'static [&'static str] = &[];

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

        unsafe impl $crate::CpuToken for $name {
            const FEATURES: &'static [&'static str] = &[$($features),*];

            unsafe fn new() -> Self {
                Self($crate::detail::UnsafeConstructible::new())
            }

            fn detect() -> Option<Self> {
                if $crate::are_cpu_features_detected!($($features),*) {
                    unsafe { Some(Self::new()) }
                } else {
                    None
                }
            }
        }
    }
}

/// Tests if a type implementing [`CpuToken`] supports a feature.
///
/// ```
/// use multiversion::{CpuToken, cpu_token_type, cpu_token_has_features};
///
/// #[cfg(target_arch = "x86_64")]
/// fn main() {
///     cpu_token_type! { SomeCpu: "sse4.1", "avx", "avx2" }
///
///     const HAS_AVX: bool = cpu_token_has_features! { type SomeCpu: "avx" };
///     assert!(HAS_AVX);
///
///     // Check if a value supports a featre
///     if let Some(cpu) = SomeCpu::detect() {
///         let has_avx512f = cpu_token_has_features! { cpu: "avx512f" };
///         assert!(!has_avx512f);
///     }
/// }
/// ```
///
/// [`CpuToken`]: trait.CpuToken.html
#[macro_export]
macro_rules! cpu_token_has_features {
    { type $cpu:ty : $($feature:tt),+ $(,)? } => {
        {
            $( $crate::detail::contains(&<$cpu as $crate::CpuToken>::FEATURES, $feature) )&*
        }
    };
    { $cpu:ident : $($feature:tt),+ $(,)? } => {
        {
            #[inline(always)]
            fn supports<T: $crate::CpuToken>(_: T) -> bool {
                $crate::cpu_token_has_features!{ type T: $($feature)* }
            }
            supports($cpu)
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

    pub const fn subset(source: &[&str], subset: &[&str]) -> bool {
        const fn subset_impl(source: &[&str], subset: &[&str], index: usize) -> bool {
            if index == subset.len() {
                true
            } else {
                contains(source, subset[index]) && subset_impl(source, subset, index + 1)
            }
        }
        subset_impl(source, subset, 0)
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
    fn subset() {
        use detail::subset;
        let features = ["a", "b", "foo"];
        assert!(subset(&features, &["foo", "a", "b"]));
        assert!(subset(&features, &["foo", "b"]));
        assert!(subset(&features, &["foo", "foo"]));
        assert!(subset(&features, &["a"]));
        assert!(subset(&features, &["a"]));
        assert!(!subset(&features, &["a", "bar"]));
        assert!(!subset(&features, &["bar"]));
    }

    #[test]
    fn generic() {
        cpu_token_type! { Generic }
        assert!(<Generic as CpuToken>::FEATURES.is_empty());
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn x86() {
        cpu_token_type! { Avx : "avx" }
        cpu_token_type! { Avx2 : "avx", "avx2" }
        assert_eq!(<Avx as CpuToken>::FEATURES, ["avx"]);
        assert_eq!(<Avx2 as CpuToken>::FEATURES, ["avx", "avx2"]);
    }
}
