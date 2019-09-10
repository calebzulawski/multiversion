Multiversion
============
[![Build Status](https://api.travis-ci.org/calebzulawski/multiversion.svg?branch=master)](https://travis-ci.org/calebzulawski/multiversion)
[![Build Status](https://ci.appveyor.com/api/projects/status/vy6xkr2073dpf4xu/branch/master?svg=true)](https://ci.appveyor.com/project/calebzulawski/multiversion/branch/master)
[![Rustc Version 1.31+](https://img.shields.io/badge/rustc-1.31+-lightgray.svg)](https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html)
[![License](https://img.shields.io/crates/l/multiversion)](https://crates.io/crates/multiversion)
[![Crates.io](https://img.shields.io/crates/v/multiversion)](https://crates.io/crates/multiversion)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/multiversion/0.1.1/multiversion/)

Function multiversioning macro/attribute for Rust.

## Usage
Add the following to your dependencies in Cargo.toml:
```toml
[dependencies]
multiversion = "0.1"
```

## Example
Automatic function multiversioning with the `target_clones` attribute, similar to GCC's `target_clones` attribute:
```rust
use multiversion::target_clones;

#[target_clones("[x86|x86_64]+avx", "x86+sse")]
fn square(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}
```

Manual function multiversioning with the `multiversion!` macro:
```rust
use multiversion::multiversion;

multiversion!{
    fn square(x: &mut [f32])
    "[x86|x86_64]+avx" => square_avx,
    "x86+sse" => square_sse,
    default => square_generic,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx")]
unsafe fn square_avx(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}

#[cfg(target_arch = "x86")]
#[target_feature(enable = "sse")]
unsafe fn square_sse(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}

fn square_generic(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}
```

## License
Multiversion is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
