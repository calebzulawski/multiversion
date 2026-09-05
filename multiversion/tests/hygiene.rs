use multiversion::multiversion;

#[multiversion(targets("x86_64+avx", "aarch64+sve"))]
fn r#type(value: u8) -> u8 {
    value + 1
}

#[multiversion(targets("x86_64+avx+avx2", "aarch64+sve", "x86_64+avx2+avx+avx", "aarch64+sve",))]
fn duplicate_targets(value: u8) -> u8 {
    value + 1
}

#[test]
fn names_and_duplicates() {
    assert_eq!(r#type(41), 42);
    assert_eq!(duplicate_targets(41), 42);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn duplicate_targets_preserve_priority() {
    #[multiversion(targets("x86_64+avx2", "x86_64+avx", "x86_64+avx2"))]
    fn selected_avx2() -> bool {
        multiversion::target::target_cfg_f!(target_feature = "avx2")
    }

    #[cfg(feature = "std")]
    let expected = std::arch::is_x86_feature_detected!("avx2");
    #[cfg(not(feature = "std"))]
    let expected = cfg!(target_feature = "avx2");
    assert_eq!(selected_avx2(), expected);
}
