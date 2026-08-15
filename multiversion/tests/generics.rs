#![cfg_attr(
    all(feature = "nightly", target_arch = "arm"),
    feature(arm_target_feature, stdarch_arm_feature_detection)
)]
#![cfg_attr(
    all(feature = "nightly", any(target_arch = "mips", target_arch = "mips64")),
    feature(mips_target_feature, stdarch_mips_feature_detection)
)]
#![cfg_attr(
    all(
        feature = "nightly",
        any(target_arch = "powerpc", target_arch = "powerpc64")
    ),
    feature(powerpc_target_feature, stdarch_powerpc_feature_detection)
)]
#![cfg_attr(
    all(
        feature = "nightly",
        any(target_arch = "riscv32", target_arch = "riscv64")
    ),
    feature(riscv_target_feature, stdarch_riscv_feature_detection)
)]
#![allow(clippy::needless_lifetimes)]

#[multiversion::multiversion(targets = "simd")]
fn pass<'a>(x: &'a i32) -> &'a i32 {
    x
}

#[multiversion::multiversion(targets = "simd")]
fn static_lifetime() -> &'static [u8] {
    "hello".as_bytes()
}

#[multiversion::multiversion(targets = "simd")]
fn placeholder_lifetime(value: &'_ [u8]) -> usize {
    value.len()
}

#[multiversion::multiversion(targets = "simd")]
fn double<'a, T: Copy + std::ops::AddAssign, const N: usize>(x: &'a mut [T; N]) -> &'a mut T {
    assert!(!x.is_empty());
    for v in x.iter_mut() {
        *v += *v;
    }
    &mut x[0]
}

mod test {
    #[test]
    fn generics() {
        let mut x = [0u32, 2u32, 4u32];
        let mut y = [0u64, 2u64, 4u64];
        *super::double(&mut x) = 1;
        *super::double(&mut y) = 2;
        assert_eq!(x, [1u32, 4u32, 8u32]);
        assert_eq!(y, [2u64, 4u64, 8u64]);
    }

    #[test]
    fn lifetimes() {
        let a = 42;
        assert_eq!(super::pass(&a), &a);
        assert_eq!(super::static_lifetime(), b"hello");
        assert_eq!(super::placeholder_lifetime(b"hello"), 5);
    }
}
