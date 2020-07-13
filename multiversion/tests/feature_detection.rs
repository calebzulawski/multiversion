use multiversion::are_cpu_features_detected;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[test]
fn detect_x86() {
    let _: bool = are_cpu_features_detected!("sse", "avx");
}
