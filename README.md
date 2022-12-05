Multiversion
============
[![Crates.io](https://img.shields.io/crates/v/multiversion)](https://crates.io/crates/multiversion)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/multiversion)
![Rustc Version 1.61+](https://img.shields.io/badge/rustc-1.61+-lightgray.svg)
[![License](https://img.shields.io/crates/l/multiversion)](https://crates.io/crates/multiversion)

Function multiversioning attribute macros for Rust.

## What is function multiversioning?
Many CPU architectures have a variety of instruction set extensions that provide additional functionality.
Common examples are single instruction, multiple data (SIMD) extensions such as SSE and AVX on x86/x86-64 and NEON on ARM/AArch64.
When available, these extended features can provide significant speed improvements to some functions.
These optional features cannot be haphazardly compiled into programs--executing an unsupported instruction will result in a crash.

**Function multiversioning** is the practice of compiling multiple versions of a function with various features enabled and safely detecting which version to use at runtime.

## Example
The `multiversion` macro compiles a function for multiple possible targets, and selects the optimal one at runtime:
```rust
use multiversion::multiversion;

#[multiversion(targets("x86_64+avx", "aarch64+neon"))]
fn square(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}
```

## License
Multiversion is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
