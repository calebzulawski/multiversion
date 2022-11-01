fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("default_features.rs");
    let features = std::env::var("CARGO_CFG_TARGET_FEATURE")
        .map(|x| x.split(',').map(|f| format!("\"{}\"", f)).collect())
        .unwrap_or_else(|_| Vec::new());
    std::fs::write(
        dest_path,
        format!(
            "const DEFAULT_FEATURES: &[&str] = &[{}];",
            features.join(", ")
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
