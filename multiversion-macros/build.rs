fn main() {
    // retpolines are not yet recognized by rust as a regular target feature.
    // We can't detect them with `cfg(target_feature = "retpoline")`, but we can detect them in
    // rustflags, since they shouldn't be the default for any target.
    let rustflags = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap();
    let retpolines_enabled = rustflags.split('\x1f').any(|flag| {
        let features = flag
            .strip_prefix("target-feature=")
            .or(flag.strip_prefix("-Ctarget-feature="));
        if let Some(features) = features {
            features
                .split(',')
                .any(|feature| feature.starts_with("+retpoline"))
        } else {
            false
        }
    });

    if retpolines_enabled {
        println!("cargo::rustc-cfg=retpoline")
    }
    println!("cargo::rustc-check-cfg=cfg(retpoline)");
    println!("cargo::rerun-if-changed=build.rs");
}
