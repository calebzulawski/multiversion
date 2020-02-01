#![allow(clippy::needless_doctest_main)]
//! This crate provides the [`target`], [`target_clones`], and [`multiversion`] attributes for
//! implementing function multiversioning.
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
//! # Getting started
//!
//! If you are unsure where to start, the [`target_clones`] attribute requires no knowledge of SIMD
//! beyond understanding the available instruction set extensions for your architecture.  For more
//! advanced usage, hand-written SIMD code can be dispatched with [`target`] and [`multiversion`].
//!
//! # Cargo features
//! There is one cargo feature, `runtime_dispatch`, enabled by default.  When enabled,
//! [`multiversion`] and [`target_clones`] will use CPU feature detection at runtime to dispatch
//! the appropriate function, which requires the `std` crate.  Disabling this feature will only
//! allow compile-time function dispatch using `#[cfg(target_feature)]` and can be used in
//! `#[no_std]` crates.
//!
//! # Capabilities
//! Most functions can be multiversioned.  The following are notable exceptions that are
//! unsupported:
//! * Methods, associated functions, inner functions, or any other function not at module level.
//! In these cases, create a multiversioned function at module level and call it from the desired
//! location.
//! * Functions that take or return `impl Trait` (other than `async`, which is supported).
//!
//! If any other functions do not work, please file a bug report.
//!
//! # Target specification strings
//! Targets for the [`target`], [`target_clones`], and [`multiversion`] attributes are specified
//! as a combination of architecture (as specified in the `target_arch` attribute) and feature (as
//! specified in the `target_feature` attribute). A single architecture can be specified as:
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
//! that as well.  This is automatically implemented by the [`target_clones`] attribute.
//!
//! This works by compiling multiple *clones* of the function with various features enabled and
//! detecting which to use at runtime. If none of the targets match the current CPU (e.g. an older
//! x86-64 CPU, or another architecture such as ARM), a clone without any features enabled is used.
//! ```
//! use multiversion::target_clones;
//!
//! #[target_clones("[x86|x86_64]+avx", "x86+sse")]
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
//! optimizations. The multiversioned function is generated by the [`multiversion`] attribute.
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
//! #[multiversion(
//!     "[x86|x86_64]+avx" => unsafe square_avx,
//!     "x86+sse" => unsafe square_sse,
//! )]
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
//! The `#[static_dispatch]` attribute may be used on `use` statements to bring the implementation
//! with matching features to the current function into scope.  Functions created by [`target_clones`]
//! and [`multiversion`] are capable of being statically dispatched.  Functions tagged with [`target`]
//! may statically dispatch functions in their body, but cannot themselves be statically
//! dispatched.
//!
//! Caveats:
//! * The caller function must exactly match an available feature set in the called function.  A
//!   function compiled for `x86_64+avx+avx2` cannot statically dispatch a function compiled for
//!   `x86_64+avx`.  A function compiled for `x86_64+avx` may statically dispatch a function
//!   compiled for `[x86|x86_64]+avx`, since an exact feature match exists for that architecture.
//! * `use` groups are not supported (`use foo::{bar, baz}`).  Renames are supported, however (`use
//!   bar as baz`)
//! ```
//! # mod fix { // doctests do something weird with modules, this fixes it
//! use multiversion::target_clones;
//!
//! #[target_clones("[x86|x86_64]+avx", "x86+sse")]
//! fn square(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v
//!     }
//! }
//!
//! #[target_clones("[x86|x86_64]+avx", "x86+sse")]
//! fn square_plus_one(x: &mut [f32]) {
//!     #[static_dispatch]
//!     use self::square; // or just `use square` in with Rust 1.32.0+
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
//! conditional compilation in functions tagged with [`target_clones`] or [`target`].
//!
//! The `#[target_cfg]` attribute supports `all`, `any`, and `not` (just like `#[cfg]`) and
//! supports the following keys:
//! * `target`: takes a target specification string as a value and is true if the target matches
//! the function's target
//!
//! ```
//! #[multiversion::target_clones("[x86|x86_64]+avx", "[arm|aarch64]+neon")]
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
//! # Implementation details
//! The function version dispatcher consists of a function selector and an atomic function pointer.
//! Initially the function pointer will point to the function selector. On invocation, this selector
//! will then choose an implementation, store a pointer to it in the atomic function pointer for later
//! use and then pass on control to the chosen function. On subsequent calls, the chosen function
//! will be called without invoking the function selector.
//!
//! Some comments on the benefits of this implementation:
//! * The function selector is only invoked once. Subsequent calls are reduced to an atomic load
//! and indirect function call (for non-generic, non-`async` functions). Generic and `async` functions
//! cannot be stored in the atomic function pointer, which may result in additional branches.
//! * If called in multiple threads, there is no contention. It is possible for two threads to hit
//! the same function before function selection has completed, which results in each thread
//! invoking the function selector, but the atomic ensures that these are synchronized correctly.
//!
//! [`target`]: attr.target.html
//! [`target_clones`]: attr.target_clones.html
//! [`multiversion`]: attr.multiversion.html

extern crate proc_macro;

mod dispatcher;
mod multiversion;
mod target;
mod target_clones;
mod util;

use quote::ToTokens;
use syn::{parse_macro_input, ItemFn};

/// Provides function multiversioning by explicitly specifying function versions.
///
/// Functions are selected in order, calling the first matching target.  The function tagged by the
/// attribute is the generic implementation that does not require any specific architecture or
/// features.
///
/// This attribute is useful when writing SIMD or other optimized code by hand.  If you would like
/// to rely on the compiler to produce your optimized function versions, try the [`target_clones`]
/// attribute instead.
///
/// # Safety
/// Functions compiled with the `target_feature` attribute must be marked unsafe, since calling
/// them on an unsupported CPU results in a crash.  To dispatch an `unsafe` function version from a
/// safe function using the `multiversion` macro, the `unsafe` functions must be tagged as such. The
/// `multiversion` attribute will produce a safe function that calls `unsafe` function versions, and
/// the safety contract is fulfilled as long as your specified targets are correct.  If your
/// function versions are `unsafe` for any other reason, you must remember to mark your
/// multiversioned function `unsafe` as well.
///
/// # Examples
/// ## A simple feature-specific function
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
/// #[multiversion(
///     "[x86|x86_64]+avx" => where_am_i_avx,
///     "x86+sse" => where_am_i_sse,
///     "[arm|aarch64]+neon" => where_am_i_neon
/// )]
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
/// #[multiversion(
///     "[x86|x86_64]+avx" => unsafe where_am_i_avx,
///     "x86+sse" => unsafe where_am_i_sse,
///     "[arm|aarch64]+neon" => unsafe where_am_i_neon
/// )]
/// fn where_am_i() {
///     println!("generic");
/// }
///
/// # fn main() {}
/// ```
///
/// # Static dispatching
/// The [`multiversion`] attribute provides a function that can be statically dispatched.  See
/// [static dispatching] for more information.
///
/// [`target`]: attr.target.html
/// [`target_clones`]: attr.target_clones.html
/// [`multiversion`]: attr.multiversion.html
/// [static dispatching]: index.html#static-dispatching
#[proc_macro_attribute]
pub fn multiversion(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = parse_macro_input!(attr as multiversion::Config);
    let func = parse_macro_input!(input as ItemFn);
    match multiversion::make_multiversioned_fn(config, func) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Provides automatic function multiversioning by compiling *clones* of the function for each
/// target.
///
/// The proper function clone is invoked depending on runtime CPU feature detection.  Priority is
/// evaluated left-to-right, selecting the first matching target.  If no matching target is found,
/// a clone with no required features is called.
///
/// This attribute is useful when relying on the compiler to make optimizations, such as automatic
/// vectorization.  If you would like to dispatch hand-written optimized code, try the
/// [`multiversion`] attribute instead.
///
/// # Example
/// The function `square` runs with AVX or SSE compiler optimizations when detected on the CPU at
/// runtime.
/// ```
/// use multiversion::target_clones;
///
/// #[target_clones("[x86|x86_64]+avx", "x86+sse")]
/// fn square(x: &mut [f32]) {
///     for v in x {
///         *v *= *v;
///     }
/// }
/// ```
///
/// # Static dispatching
/// The [`target_clones`] attribute provides a function that can be statically dispatched.
/// Additionally, functions called inside the function may be statically dispatched. See
/// [static dispatching] for more information.
///
/// # Conditional compilation
/// The [`target_clones`] attribute supports the `#[target_cfg]` helper attribute to provide
/// conditional compilation for each function clone.  See [conditional compilation] for more
/// information.
///
/// [`target`]: attr.target.html
/// [`target_clones`]: attr.target_clones.html
/// [`multiversion`]: attr.multiversion.html
/// [static dispatching]: index.html#static-dispatching
/// [conditional compilation]: index.html#conditional-compilation
#[proc_macro_attribute]
pub fn target_clones(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = parse_macro_input!(attr as target_clones::Config);
    let func = parse_macro_input!(input as ItemFn);
    target_clones::make_target_clones_fn(config, func)
        .into_token_stream()
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
/// # Static dispatching
/// The [`target`] attribute allows functions called inside the function to be statically dispatched.
/// See [static dispatching] for more information.
///
/// # Conditional compilation
/// The [`target`] attribute supports conditional compilation with the `#[target_cfg]` helper
/// attribute. See [conditional compilation] for more information.
///
/// [`target`]: attr.target.html
/// [`target_clones`]: attr.target_clones.html
/// [`multiversion`]: attr.multiversion.html
/// [static dispatching]: index.html#static-dispatching
/// [conditional compilation]: index.html#conditional-compilation
#[proc_macro_attribute]
pub fn target(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = parse_macro_input!(attr as target::Config);
    let func = parse_macro_input!(input as ItemFn);
    match target::make_target_fn(config, func) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}
