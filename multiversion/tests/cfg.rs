use multiversion::{
    multiversion,
    target::{cfg_attr_selected, cfg_selected, selected},
};

#[test]
fn cfg() {
    #[multiversion(targets = "simd")]
    fn foo() {
        #[cfg_selected(all(target_arch = "x86_64", target_feature = "avx"))]
        fn test_avx(has_avx: bool) {
            assert!(has_avx);
        }

        #[cfg_selected(not(all(target_arch = "x86_64", target_feature = "avx")))]
        fn test_avx(has_avx: bool) {
            assert!(!has_avx);
        }

        let has_avx = std::env::consts::ARCH == "x86_64" && selected!().supports("avx");
        test_avx(has_avx);
    }

    foo();
}

#[test]
fn cfg_attr() {
    #[multiversion(targets = "simd")]
    fn foo() {
        #[cfg_attr_selected(all(target_arch = "x86_64", target_feature = "avx"), cfg(all()))]
        #[cfg_attr_selected(not(all(target_arch = "x86_64", target_feature = "avx")), cfg(any()))]
        fn test_avx(has_avx: bool) {
            assert!(has_avx);
        }

        #[cfg_attr_selected(all(target_arch = "x86_64", target_feature = "avx"), cfg(any()))]
        #[cfg_attr_selected(not(all(target_arch = "x86_64", target_feature = "avx")), cfg(all()))]
        fn test_avx(has_avx: bool) {
            assert!(!has_avx);
        }

        let has_avx = std::env::consts::ARCH == "x86_64" && selected!().supports("avx");
        test_avx(has_avx);
    }

    foo();
}
