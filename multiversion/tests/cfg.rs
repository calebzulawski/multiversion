use multiversion::{
    multiversion,
    target::{match_target, selected_target, target_cfg, target_cfg_attr, target_cfg_f},
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

#[test]
fn cfg_f() {
    #[multiversion(targets = "simd")]
    fn foo() {
        let cfg_avx = target_cfg_f!(all(target_arch = "x86_64", target_feature = "avx"));
        let has_avx =
            std::env::consts::ARCH == "x86_64" && selected_target!().supports_feature_str("avx");
        assert_eq!(cfg_avx, has_avx);
    }

    foo();
}

#[test]
fn match_target() {
    #[multiversion(targets = "simd")]
    fn foo() {
        let match_avx = match_target! {
            "x86_64+avx" => true,
            "aarch64+neon" | "x86_64+sse" => false,
            _ => false,
        };

        let has_avx =
            std::env::consts::ARCH == "x86_64" && selected_target!().supports_feature_str("avx");

        assert_eq!(match_avx, has_avx);
    }

    foo();
}
