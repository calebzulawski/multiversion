#![allow(clippy::needless_doctest_main)]
//! This crate provides the [`multiversion`] attribute for implementing function multiversioning.
//!
//! Many CPU architectures have a variety of instruction set extensions that provide additional
//! functionality. Common examples are single instruction, multiple data (SIMD) extensions such as
//! SSE and AVX on x86/x86-64 and NEON on ARM/AArch64. When available, these extended features can
//! provide significant speed improvements to some functions. These optional features cannot be
//! haphazardly compiled into programs–executing an unsupported instruction will result in a
//! crash.
//!
//! **Function multiversioning** is the practice of compiling multiple versions of a function
//! with various features enabled and safely detecting which version to use at runtime.
//!
//! # Cargo features
//! There is one cargo feature, `std`, enabled by default.  When enabled, [`multiversion`] will
//! use CPU feature detection at runtime to dispatch the appropriate function. Disabling this
//! feature will only allow compile-time function dispatch using `#[cfg(target_feature)]` and can
//! be used in `#[no_std]` crates.
//!
//! # Capabilities
//! The intention of this crate is to allow nearly any function to be multiversioned.
//! The following cases are not supported:
//! * functions that use `self` or `Self`
//! * `impl Trait` return types (arguments are fine)
//!
//! If any other functions do not work please file an issue on GitHub.
//!
//! # Target specification strings
//! Targets are specified as a combination of architecture (as specified in [`target_arch`]) and
//! feature (as specified in [`target_feature`]).
//!
//! A target can be specified as:
//! * `"arch"`
//! * `"arch+feature"`
//! * `"arch+feature1+feature2"`
//!
//! A particular CPU can also be specified with a slash:
//! * `"arch/cpu"`
//! * `"arch/cpu+feature"`
//!
//! The following are some valid target specification strings:
//! * `"x86"` (matches the `"x86"` architecture)
//! * `"x86_64+avx+avx2"` (matches the `"x86_64"` architecture with the `"avx"` and `"avx2"`
//! features)
//! * `"x86_64/x86-64-v2"` (matches the `"x86_64"` architecture with the `"x86-64-v2"` CPU)
//! * `"x86/i686+avx"` (matches the `"x86"` architecture with the `"i686"` CPU and `"avx"`
//! feature)
//! * `"arm+neon"` (matches the `arm` architecture with the `"neon"` feature
//!
//! A complete list of available target features and CPUs is available in the [`target-features`
//! crate documentation](target_features::docs).
//!
//! [`target`]: attr.target.html
//! [`multiversion`]: attr.multiversion.html
//! [`target_arch`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch
//! [`target_feature`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_feature

/// Provides function multiversioning.
///
/// The annotated function is compiled multiple times, once for each target, and the
/// best target is selected at runtime.
///
/// Options:
/// * `targets`
///   * Takes a list of targets, such as `targets("x86_64+avx2", "x86_64+sse4.1")`.
///   * Target priority is first to last.  The first matching target is used.
///   * May also take a special value `targets = "simd"` to automatically multiversion for common
///     SIMD target features.
/// * `attrs`
///   * Takes a list of attributes to attach to each target clone function.
/// * `dispatcher`
///   * Selects the preferred dispatcher. Defaults to `default`.
///     * `default`: If the `std` feature is enabled, uses either `direct` or `indirect`,
///       attempting to choose the fastest choice.  If the `std` feature is not enabled, uses `static`.
///     * `static`: Detects features at compile time from the enabled target features.
///     * `indirect`: Detect features at runtime, and dispatches with an indirect function call.
///       Cannot be used for generic functions, `async` functions, or functions that take or return an
///       `impl Trait`.  This is usually the default.
///     * `direct`: Detects features at runtime, and dispatches with direct function calls. This is
///       the default on functions that do not support indirect dispatch, or in the presence of
///       indirect branch exploit mitigations such as retpolines.
///
/// # Example
/// This function is a good candidate for optimization using SIMD.
/// The following compiles `square` three times, once for each target and once for the generic
/// target.  Calling `square` selects the appropriate version at runtime.
///
/// ```
/// use multiversion::multiversion;
///
/// #[multiversion(targets("x86_64+avx", "x86+sse"))]
/// fn square(x: &mut [f32]) {
///     for v in x {
///         *v *= *v
///     }
/// }
/// ```
///
/// This example is similar, but targets all supported SIMD instruction sets (not just the two shown above):
///
/// ```
/// use multiversion::multiversion;
///
/// #[multiversion(targets = "simd")]
/// fn square(x: &mut [f32]) {
///     for v in x {
///         *v *= *v
///     }
/// }
/// ```
///
/// # Notes on dispatcher performance
///
/// ### Feature detection is performed only once
/// The `direct` and `indirect` dispatchers perform function selection on the first invocation.
/// This is implemented with a static atomic variable containing the selected function.
///
/// This implementation has a few benefits:
/// * The function selector is typically only invoked once.  Subsequent calls are reduced to an
/// atomic load.
/// * If called in multiple threads, there is no contention.  Both threads may perform feature
/// detection, but the atomic ensures these are synchronized correctly.
///
/// ### Dispatcher elision
/// If the optimal set of features is already known to exist at compile time, the entire dispatcher
/// is elided. For example, if the highest priority target requires `avx512f` and the function is
/// compiled with `RUSTFLAGS=-Ctarget-cpu=skylake-avx512`, the function is not multiversioned and
/// the highest priority target is used.
///
/// [`target`]: attr.target.html
/// [`multiversion`]: attr.multiversion.html
pub use multiversion_macros::multiversion;

/// Provides a less verbose equivalent to the `cfg(target_arch)` and `target_feature` attributes.
///
/// A function tagged with `#[target("x86_64+avx+avx2")]`, for example, is equivalent to a
/// function tagged with each of:
/// * `#[cfg(target_arch = "x86_64")]`
/// * `#[target_feature(enable = "avx")]`
/// * `#[target_feature(enable = "avx2")]`
///
/// The [`target`] attribute is intended to be used in tandem with the [`multiversion`] attribute
/// to produce hand-written multiversioned functions.
///
/// [`target`]: attr.target.html
/// [`multiversion`]: attr.multiversion.html
pub use multiversion_macros::target;

/// Inherit the `target_feature` attributes of the selected target in a multiversioned function.
///
/// # Example
/// ```
/// use multiversion::{multiversion, inherit_target};
/// #[multiversion(targets = "simd")]
/// fn select_sum() -> unsafe fn(x: &mut[f32]) -> f32 {
///     #[inherit_target]
///     unsafe fn sum(x: &mut[f32]) -> f32 {
///         x.iter().sum()
///     }
///     sum as unsafe fn(&mut[f32]) -> f32
/// }
pub use multiversion_macros::inherit_target;

/// Information related to the current target.
pub mod target {
    // used by docs
    #[allow(unused)]
    use super::*;

    /// Get the selected target in a multiversioned function.
    ///
    /// Returns the selected target as a [`Target`].
    ///
    /// This macro only works in a function marked with [`multiversion`].
    ///
    /// # Example
    /// ```
    /// use multiversion::{multiversion, target::selected_target};
    ///
    /// #[multiversion(targets = "simd")]
    /// fn foo() {
    ///     if selected_target!().supports_feature_str("avx") {
    ///         println!("AVX detected");
    ///     } else {
    ///         println!("AVX not detected");
    ///     }
    /// }
    pub use multiversion_macros::selected_target;

    /// Equivalent to `#[cfg]`, but considers `target_feature`s detected at runtime.
    ///
    /// This macro only works in a function marked with [`multiversion`].
    pub use multiversion_macros::target_cfg;

    /// Equivalent to `#[cfg_attr]`, but considers `target_feature`s detected at runtime.
    ///
    /// This macro only works in a function marked with [`multiversion`].
    pub use multiversion_macros::target_cfg_attr;

    /// Match the selected target.
    ///
    /// Matching is done at compile time, as if by `#[cfg]`. Target matching considers both
    /// detected features and statically-enabled features. Arms that do not match are not
    /// compiled.
    ///
    /// This macro only works in a function marked with [`multiversion`].
    ///
    /// # Example
    /// ```
    /// use multiversion::{multiversion, target::match_target};
    ///
    /// #[multiversion(targets = "simd")]
    /// fn foo() {
    ///     match_target! {
    ///         "x86_64+avx" => println!("x86-64 with AVX"),
    ///         "aarch64+neon" => println!("AArch64 with Neon"),
    ///         _ => println!("another architecture"),
    ///     }
    /// }
    /// ```
    pub use multiversion_macros::match_target;

    /// Equivalent to `cfg!`, but considers `target_feature`s detected at runtime.
    ///
    /// This macro only works in a function marked with [`multiversion`].
    pub use multiversion_macros::target_cfg_f;

    #[doc(hidden)]
    pub use multiversion_macros::{
        match_target_impl, target_cfg_attr_impl, target_cfg_f_impl, target_cfg_impl,
    };

    #[doc(no_inline)]
    pub use target_features::Target;
}

#[doc(hidden)]
pub use target_features;
