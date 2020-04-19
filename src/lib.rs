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
//! There is one cargo feature, `runtime_dispatch`, enabled by default.  When enabled,
//! [`multiversion`] will use CPU feature detection at runtime to dispatch the appropriate function,
//! which requires the `std` crate.  Disabling this feature will only allow compile-time function
//! dispatch using `#[cfg(target_feature)]` and can be used in `#[no_std]` crates.
//!
//! # Capabilities
//! The intention of this crate is to allow any function to be multiversioned.  If any functions do
//! not work, please file an issue on GitHub.
//!
//! # Target specification strings
//! Targets for the [`target`] and [`multiversion`] attributes are specified as a combination of
//! architecture (as specified in the `target_arch` attribute) and feature (as specified in the
//! `target_feature` attribute). A single architecture can be specified as:
//! * `"arch"`
//! * `"arch+feature"`
//! * `"arch+feature1+feature2"`
//!
//! while multiple architectures can be specified as:
//! * `"[arch1|arch2]"`
//! * `"[arch1|arch2]+feature"`
//! * `"[arch1|arch2]+feature1+feature2"`
//!
//! The following are all valid target specification strings:
//! * `"x86"` (matches the `"x86"` architecture)
//! * `"x86_64+avx+avx2"` (matches the `"x86_64"` architecture with the `"avx"` and `"avx2"`
//! features)
//! * `"[mips|mips64|powerpc|powerpc64]"` (matches any of the `"mips"`, `"mips64"`, `"powerpc"` or
//! `"powerpc64"` architectures)
//! * `"[arm|aarch64]+neon"` (matches either the `"arm"` or `"aarch64"` architectures with the
//! `"neon"` feature)
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
//! #[multiversion]
//! #[clone(target = "[x86|x86_64]+avx")]
//! #[clone(target = "x86+sse")]
//! fn square(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v;
//!     }
//! }
//! ```
//!
//! The following produces a nearly identical function, but instead of cloning the function, the
//! implementations are manually specified. This is typically more useful when the implementations
//! aren't identical, such as when using explicit SIMD instructions instead of relying on compiler
//! optimizations.
//! ```
//! use multiversion::{multiversion, target};
//!
//! #[target("[x86|x86_64]+avx")]
//! unsafe fn square_avx(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v;
//!     }
//! }
//!
//! #[target("x86+sse")]
//! unsafe fn square_sse(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v;
//!     }
//! }
//!
//! #[multiversion]
//! #[specialize(target = "[x86|x86_64]+avx", fn = "square_avx", unsafe = true)]
//! #[specialize(target = "x86+sse", fn = "square_sse", unsafe = true)]
//! fn square(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v;
//!     }
//! }
//!
//! # fn main() {}
//! ```
//!
//! # Static dispatching
//! Sometimes it may be useful to call multiversioned functions from other multiversioned functions.
//! In these situations it would be inefficient to perform feature detection multiple times.
//! Additionally, the runtime detection prevents the function from being inlined.  In this situation,
//! the `#[static_dispatch]` helper attribute allows bypassing feature detection.
//!
//! The `#[static_dispatch]` attribute accepts the following arguments:
//! * `fn`: path to the function to static dispatch.
//! * `rename` (optional): the binding to use for the statically dispatched function.  If not
//!   provided, the function name is used.
//!
//! Caveats:
//! * The caller function must exactly match an available feature set in the called function.  A
//!   function compiled for `x86_64+avx+avx2` cannot statically dispatch a function compiled for
//!   `x86_64+avx`.  A function compiled for `x86_64+avx` may statically dispatch a function
//!   compiled for `[x86|x86_64]+avx`, since an exact feature match exists for that architecture.
//! * The receiver (`self`, `&self`, etc.) must be provided as the first argument to the statically
//!   dispatched function, e.g. `foo(bar)` rather than `bar.foo()`.
//! ```
//! # mod fix { // doctests do something weird with modules, this fixes it
//! use multiversion::multiversion;
//!
//! #[multiversion]
//! #[clone(target = "[x86|x86_64]+avx")]
//! #[clone(target = "x86+sse")]
//! fn square(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v
//!     }
//! }
//!
//! #[multiversion]
//! #[clone(target = "[x86|x86_64]+avx")]
//! #[clone(target = "x86+sse")]
//! #[static_dispatch(fn = "square")]
//! fn square_plus_one(x: &mut [f32]) {
//!     square(x); // this function call bypasses feature detection
//!     for v in x {
//!         *v += 1.0;
//!     }
//! }
//!
//! # }
//! ```
//!
//! # Conditional compilation
//! The `#[cfg]` attribute allows conditional compilation based on the target architecture and
//! features, however this does not take into account additional features specified by
//! `#[target_feature]`.  In this scenario, the `#[target_cfg]` helper attribute provides
//! conditional compilation in functions tagged with [`multiversion`] or [`target`].
//!
//! The `#[target_cfg]` attribute supports `all`, `any`, and `not` (just like `#[cfg]`) and
//! supports the following keys:
//! * `target`: takes a target specification string as a value and is true if the target matches
//! the function's target
//!
//! ```
//! #[multiversion::multiversion]
//! #[clone(target = "[x86|x86_64]+avx")]
//! #[clone(target = "[arm|aarch64]+neon")]
//! fn print_arch() {
//!     #[target_cfg(target = "[x86|x86_64]+avx")]
//!     println!("avx");
//!
//!     #[target_cfg(target = "[arm|aarch64]+neon")]
//!     println!("neon");
//!
//!     #[target_cfg(not(any(target = "[x86|x86_64]+avx", target = "[arm|aarch64]+neon")))]
//!     println!("generic");
//! }
//! ```
//!
//! [`target`]: attr.target.html
//! [`multiversion`]: attr.multiversion.html

extern crate proc_macro;

#[macro_use]
mod meta;

mod dispatcher;
mod multiversion;
mod safe_inner;
mod static_dispatch;
mod target;
mod target_cfg;
mod util;

use quote::ToTokens;
use syn::{parse::Nothing, parse_macro_input, ItemFn};

/// Provides function multiversioning.
///
/// Functions are selected in order, calling the first matching target.  The function tagged by the
/// attribute is the generic implementation that does not require any specific architecture or
/// features.
///
/// # Helper attributes
/// * `#[clone]`
///   * Clones the function for the specified target.
///   * Arguments:
///     * `target`: the target specification of the clone
/// * `#[specialize]`
///   * Specializes the function for the specified target by calling `function`.
///   * Arguments:
///     * `target`: the target specification of the specialization
///     * `fn`: path to the function specializing the tagged function
///     * `unsafe` (optional): indicates whether the specialization function is `unsafe`, but safe to
///       call for this target.
///       Functions tagged with the [`target`] attribute must be `unsafe`, so marking `unsafe = true`
///       indicates that the safety contract is fulfilled and`function` is safe to call on the specified
///       target.  If `function` is unsafe for any other reason, remember to mark the tagged function
///       `unsafe` as well.
/// * `#[static_dispatch]`
///   * Statically dispatches another multiversioned function, see [static dispatching].
///
/// # Examples
/// ## Cloning
/// The following compiles `square` three times, once for each target and once for the generic
/// target.  Calling `square` selects the appropriate version at runtime.
/// ```
/// use multiversion::multiversion;
///
/// #[multiversion]
/// #[clone(target = "[x86|x86_64]+avx")]
/// #[clone(target = "x86+sse")]
/// fn square(x: &mut [f32]) {
///     for v in x {
///         *v *= *v
///     }
/// }
/// ```
///
/// ## Specialization
/// This example creates a function `where_am_i` that prints the detected CPU feature.
/// ```
/// use multiversion::multiversion;
///
/// fn where_am_i_avx() {
///     println!("avx");
/// }
///
/// fn where_am_i_sse() {
///     println!("sse");
/// }
///
/// fn where_am_i_neon() {
///     println!("neon");
/// }
///
/// #[multiversion]
/// #[specialize(target = "[x86|x86_64]+avx", fn  = "where_am_i_avx")]
/// #[specialize(target = "x86+sse", fn = "where_am_i_sse")]
/// #[specialize(target = "[arm|aarch64]+neon", fn = "where_am_i_neon")]
/// fn where_am_i() {
///     println!("generic");
/// }
///
/// # fn main() {}
/// ```
/// ## Making `target_feature` functions safe
/// This example is the same as the above example, but calls `unsafe` specialized functions.  Note
/// that the `where_am_i` function is still safe, since we know we are only calling specialized
/// functions on supported CPUs.  In this example, the [`target`] attribute is used for
/// convenience.
/// ```
/// use multiversion::{multiversion, target};
///
/// #[target("[x86|x86_64]+avx")]
/// unsafe fn where_am_i_avx() {
///     println!("avx");
/// }
///
/// #[target("x86+sse")]
/// unsafe fn where_am_i_sse() {
///     println!("sse");
/// }
///
/// #[target("[arm|aarch64]+neon")]
/// unsafe fn where_am_i_neon() {
///     println!("neon");
/// }
///
/// #[multiversion]
/// #[specialize(target = "[x86|x86_64]+avx", fn = "where_am_i_avx", unsafe = true)]
/// #[specialize(target = "x86+sse", fn = "where_am_i_sse", unsafe = true)]
/// #[specialize(target = "[arm|aarch64]+neon", fn = "where_am_i_neon")]
/// fn where_am_i() {
///     println!("generic");
/// }
///
/// # fn main() {}
/// ```
///
/// # Static dispatching
/// The [`multiversion`] attribute allows functions called inside the function to be statically dispatched.
/// Additionally, functions created with this attribute can themselves be statically dispatched.
/// See [static dispatching] for more information.
///
/// # Conditional compilation
/// The [`multiversion`] attribute supports conditional compilation with the `#[target_cfg]` helper
/// attribute. See [conditional compilation] for more information.
///
/// # Implementation details
/// The function version dispatcher consists of a function selector and an atomic function pointer.
/// Initially the function pointer will point to the function selector. On invocation, this selector
/// will then choose an implementation, store a pointer to it in the atomic function pointer for later
/// use and then pass on control to the chosen function. On subsequent calls, the chosen function
/// will be called without invoking the function selector.
///
/// Some comments on the benefits of this implementation:
/// * The function selector is only invoked once. Subsequent calls are reduced to an atomic load
/// and indirect function call (for non-generic, non-`async` functions). Generic and `async` functions
/// cannot be stored in the atomic function pointer, which may result in additional branches.
/// * If called in multiple threads, there is no contention. It is possible for two threads to hit
/// the same function before function selection has completed, which results in each thread
/// invoking the function selector, but the atomic ensures that these are synchronized correctly.
///
/// [`target`]: attr.target.html
/// [`multiversion`]: attr.multiversion.html
/// [static dispatching]: index.html#static-dispatching
/// [conditional compilation]: index.html#conditional-compilation
#[proc_macro_attribute]
pub fn multiversion(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    parse_macro_input!(attr as Nothing);
    let func = parse_macro_input!(input as ItemFn);
    match multiversion::make_multiversioned_fn(func) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Provides a less verbose equivalent to the `target_arch` and `target_feature` attributes.
///
/// A function tagged with `#[target("[x86|x86_64]+avx+avx2")]`, for example, is equivalent to a
/// function tagged with each of:
/// * `#[target_arch(any(target_arch = "x86", target_arch = "x86_64"))]`
/// * `#[target_feature(enable = "avx")]`
/// * `#[target_feature(enable = "avx2")]`
///
/// The [`target`] attribute is intended to be used in tandem with the [`multiversion`] attribute
/// to produce hand-written multiversioned functions.
///
/// # Helper attributes
/// * `#[safe_inner]`
///   * Indicates that the inner contents of the function are safe and requires the use of `unsafe`
///     blocks to call `unsafe` functions.
/// * `#[static_dispatch]`
///   * Statically dispatches a multiversioned function, see [static dispatching].
///
/// # Static dispatching
/// The [`target`] attribute allows functions called inside the function to be statically dispatched.
/// See [static dispatching] for more information.
///
/// # Conditional compilation
/// The [`target`] attribute supports conditional compilation with the `#[target_cfg]` helper
/// attribute. See [conditional compilation] for more information.
///
/// [`target`]: attr.target.html
/// [`multiversion`]: attr.multiversion.html
/// [static dispatching]: index.html#static-dispatching
/// [conditional compilation]: index.html#conditional-compilation
#[proc_macro_attribute]
pub fn target(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let target = parse_macro_input!(attr as Option<syn::Lit>);
    let func = parse_macro_input!(input as ItemFn);
    match target::make_target_fn(target, func) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}
