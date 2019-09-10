//! This crate provides macro the [`target_clones`] attribute and [`multiversion!`] macro for
//! implementing function multiversioning.
//!
//! [`target_clones`]: attr.target_clones.html
//! [`multiversion!`]: macro.multiversion.html
//!
//! Many CPU architectures have a variety of instruction set extensions that provide additional
//! functionality. Common examples are single instruction, multiple data (SIMD) extensions such as
//! SSE and AVX on x86/x86-64 and NEON on ARM/AArch64. When available, these extended features can
//! provide significant speed improvements to some functions. These optional features cannot be
//! haphazardly compiled into programs–executing an unsupported instruction will result in a
//! crash.  Function multiversioning is the practice of compiling multiple versions of a function
//! with various features enabled and detecting which version to use at runtime.
//!
//! # Example
//! The following example is a good candidate for optimization with SIMD.  The function `square`
//! optionally uses the AVX instruction set extension on x86 or x86-64.  The SSE instruciton set
//! extension is part of x86-64, but is optional on x86 so the square function optionally detects
//! that as well.
//!
//! This is works by compiling multiple *clones* of the function with various features enabled and
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
//! implementations are manually specified.  This is typically more useful when the implementations
//! aren't identical, such as when using explicit SIMD instructions instead of relying on compiler
//! optimizations.
//! ```
//! use multiversion::multiversion;
//!
//! multiversion!{
//!     fn square(x: &mut [f32])
//!     "[x86|x86_64]+avx" => square_avx,
//!     "x86+sse" => square_sse,
//!     default => square_generic,
//! }
//!
//! #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
//! #[target_feature(enable = "avx")]
//! unsafe fn square_avx(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v;
//!     }
//! }
//!
//! #[cfg(target_arch = "x86")]
//! #[target_feature(enable = "sse")]
//! unsafe fn square_avx(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v;
//!     }
//! }
//!
//! fn square_generic(x: &mut [f32]) {
//!     for v in x {
//!         *v *= *v;
//!     }
//! }
//!
//! # fn main() {}
//! ```

extern crate proc_macro;

mod dispatcher;
mod multiversion;
mod target;
mod target_clones;

use quote::ToTokens;
use syn::{parse_macro_input, ItemFn};

#[proc_macro]
pub fn multiversion(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as multiversion::MultiVersion)
        .into_token_stream()
        .into()
}

#[proc_macro_attribute]
pub fn target_clones(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = parse_macro_input!(attr as target_clones::Config);
    let func = parse_macro_input!(input as ItemFn);
    target_clones::TargetClones::new(config, &func)
        .into_token_stream()
        .into()
}
