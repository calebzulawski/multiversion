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

#[cfg(feature = "target-features")]
use multiversion::target::selected_target;
use multiversion::{
    multiversion,
    target::{match_target, target_cfg, target_cfg_attr, target_cfg_f},
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

        let has_avx = target_cfg_f!(all(target_arch = "x86_64", target_feature = "avx"));
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

        let has_avx = target_cfg_f!(all(target_arch = "x86_64", target_feature = "avx"));
        test_avx(has_avx);
    }

    foo();
}

#[test]
fn cfg_f() {
    #[multiversion(targets = "simd")]
    fn foo() {
        let cfg_avx = target_cfg_f!(all(target_arch = "x86_64", target_feature = "avx"));
        let match_avx = match_target! {
            "x86_64+avx" => true,
            _ => false,
        };
        assert_eq!(cfg_avx, match_avx);
        #[cfg(feature = "target-features")]
        assert!(!cfg_avx || selected_target!().supports_feature_str("avx"));
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

        let has_avx = target_cfg_f!(all(target_arch = "x86_64", target_feature = "avx"));

        assert_eq!(match_avx, has_avx);
    }

    foo();
}
