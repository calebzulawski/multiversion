#![allow(clippy::needless_doctest_main)]
//! This crate provides the [`target`]  and [`multiversion`] attributes for implementing
//! function multiversioning.
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
//! Targets for the [`target`] and [`multiversion`] attributes are specified as a combination of
//! architecture (as specified in the [`target_arch`] attribute) and feature (as specified in the
//! [`target_feature`] attribute). A target can be specified as:
//! * `"arch"`
//! * `"arch+feature"`
//! * `"arch+feature1+feature2"`
//!
//! The following are some valid target specification strings:
//! * `"x86"` (matches the `"x86"` architecture)
//! * `"x86_64+avx+avx2"` (matches the `"x86_64"` architecture with the `"avx"` and `"avx2"`
//! features)
//! * `"arm+neon"` (matches the `arm` architecture with the `"neon"` feature
//!
//! # Example
//! The following example is a good candidate for optimization with SIMD.  The function `square`
//! optionally uses the AVX instruction set extension on x86 or x86-64.  The SSE instruction set
//! extension is part of x86-64, but is optional on x86 so the square function optionally detects
//! that as well.  This is automatically implemented by the [`multiversion`] attribute.
//!
//! The following works by compiling multiple *clones* of the function with various features enabled
//! and detecting which to use at runtime. If none of the targets match the current CPU (e.g. an older
//! x86-64 CPU, or another architecture such as ARM), a clone without any features enabled is used.
//! ```
//! use multiversion::multiversion;
//!
//! #[multiversion(targets("x86_64+avx", "x86+sse"))]
//! fn square(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v;
//!     }
//! }
//! ```
//!
//! [`target`]: attr.target.html
//! [`multiversion`]: attr.multiversion.html
//! [`target_arch`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch
//! [`target_feature`]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_feature

/// Provides function multiversioning.
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
/// # Implementation details
/// The `direct` and `indirect` function version dispatcher performs function selection on the
/// first invocation. This is implemented with a static atomic variable containing the selected
/// function.
///
/// This implementation has a few benefits:
/// * The function selector is typically only invoked once.  Subsequent calls are reduced to an
/// atomic load.
/// * If called in multiple threads, there is no contention.  Both threads may perform feature
/// detection, but the atomic ensures these are synchronized correctly.
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
    /// Returns the selected target as a [`TargetFeatures`].  This macro only works in a
    /// function marked with [`multiversion`].
    ///
    /// # Example
    /// ```
    /// use multiversion::{multiversion, target::selected_target};
    ///
    /// #[multiversion(targets = "simd")]
    /// fn foo() {
    ///     if selected_target!().supports("avx") {
    ///         println!("AVX detected");
    ///     } else {
    ///         println!("AVX not detected");
    ///     }
    /// }
    pub use multiversion_macros::selected_target;

    /// Equivalent to `#[cfg]`, but considers `target_feature`s detected at runtime.
    pub use multiversion_macros::target_cfg;

    /// Equivalent to `#[cfg_attr]`, but considers `target_feature`s detected at runtime.
    pub use multiversion_macros::target_cfg_attr;

    #[doc(hidden)]
    pub use multiversion_macros::{target_cfg_attr_impl, target_cfg_impl};

    mod features;
    pub use features::*;
}
