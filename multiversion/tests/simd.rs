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

#[multiversion::multiversion(targets = "simd")]
#[allow(dead_code)]
fn simd() {}
