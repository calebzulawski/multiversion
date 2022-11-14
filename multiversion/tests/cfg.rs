use multiversion::{
    multiversion,
    target::{selected_target, target_cfg, target_cfg_attr},
};

#[test]
fn cfg() {
    #[multiversion(targets = "simd")]
    fn foo() {
        #[target_cfg(all(target_arch = "x86_64", target_feature = "avx"))]
        fn test_avx(has_avx: bool) {
            assert!(has_avx);
        }

        #[target_cfg(not(all(target_arch = "x86_64", target_feature = "avx")))]
        fn test_avx(has_avx: bool) {
            assert!(!has_avx);
        }

        let has_avx =
            std::env::consts::ARCH == "x86_64" && selected_target!().supports_feature_str("avx");
        test_avx(has_avx);
    }

    foo();
}

#[test]
fn cfg_attr() {
    #[multiversion(targets = "simd")]
    fn foo() {
        #[target_cfg_attr(all(target_arch = "x86_64", target_feature = "avx"), cfg(all()))]
        #[target_cfg_attr(not(all(target_arch = "x86_64", target_feature = "avx")), cfg(any()))]
        fn test_avx(has_avx: bool) {
            assert!(has_avx);
        }

        #[target_cfg_attr(all(target_arch = "x86_64", target_feature = "avx"), cfg(any()))]
        #[target_cfg_attr(not(all(target_arch = "x86_64", target_feature = "avx")), cfg(all()))]
        fn test_avx(has_avx: bool) {
            assert!(!has_avx);
        }

        let has_avx =
            std::env::consts::ARCH == "x86_64" && selected_target!().supports_feature_str("avx");
        test_avx(has_avx);
    }

    foo();
}
