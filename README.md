# Multiversion: [![Build Status](https://api.travis-ci.org/calebzulawski/multiversion.svg?branch=master)](https://travis-ci.org/calebzulawski/multiversion) [![Rustc Version 1.31+](https://img.shields.io/badge/rustc-1.31+-lightgray.svg)](https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html)

Multiversion provides function multiversioning for Rust.

## Usage
Add the following to your dependencies in Cargo.toml:
```toml
[dependencies]
multiversion = { git = "https://github.com/calebzulawski/multiversion" }
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
```
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
unsafe fn square_avx(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}

fn square_generic(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}

# fn main() {}
```

## License
Multiversion is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
