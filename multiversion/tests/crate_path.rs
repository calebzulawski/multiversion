// export multiversion somewhere else
pub mod multiversion_export {
    pub mod nested {
        pub use multiversion::*;
    }
}

#[allow(unused_imports)]
use core as multiversion; // override the multiversion name

#[multiversion_export::nested::multiversion(
    versions(clone = "[x86|x86_64]+avx"),
    crate_path = "multiversion_export::nested"
)]
fn foo() {}

#[test]
fn crate_path() {
    foo()
}
