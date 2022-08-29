use std::{
    collections::hash_set::HashSet,
    io::Write,
    process::{Command, Stdio},
};

fn triple(arch: &str) -> &'static str {
    match arch {
        "x86" => "x86_64-unknown-linux-gnu",
        "x86_64" => "x86_64-unknown-linux-gnu",
        "arm" => "arm-unknown-linux-gnueabi",
        "aarch64" => "aarch64-unknown-linux-gnu",
        "mips" => "mips-unknown-linux-gnu",
        "mips64" => "mips64-unknown-linux-gnuabi64",
        "powerpc" => "powerpc-unknown-linux-gnu",
        "powerpc64" => "powerpc64-unknown-linux-gnu",
        "riscv64" => "riscv64gc-unknown-linux-gnu",
        _ => unimplemented!(),
    }
}

fn possible_features(arch: &str) -> HashSet<String> {
    let output = Command::new("cross")
        .args([
            "+nightly",
            "run",
            "--target",
            triple(arch),
            "--bin",
            "list-features",
        ])
        .env("PATH", std::env::var("PATH").unwrap())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(ToString::to_string)
        .collect()
}

fn get_features(arch: &str, target_feature: &str) -> HashSet<String> {
    let output = Command::new("rustc")
        .args(["+nightly", "--print", "cfg", "--target", triple(arch)])
        .arg(format!("-Ctarget-feature={}", target_feature))
        .env("PATH", std::env::var("PATH").unwrap())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    assert!(output.status.success());

    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter_map(|s| {
            s.strip_prefix("target_feature=\"")
                .and_then(|s| s.strip_suffix('"'))
                .map(ToString::to_string)
        })
        .filter(|f| f != "llvm14-builtins-abi")
        .collect()
}

fn get_implied_features(arch: &str) -> Vec<(String, Vec<String>)> {
    let possible_features = possible_features(arch);

    let disable_default_features = get_features(arch, "")
        .iter()
        .map(|s| format!("-{}", s))
        .collect::<Vec<String>>()
        .join(",");

    // avoid LLVM issue
    let arch_features = if arch.starts_with("mips") {
        ",+fp64"
    } else {
        ""
    };

    let mut implied_features = possible_features
        .iter()
        .filter_map(|feature| {
            let mut implied_features = get_features(
                arch,
                &format!("{},+{}{}", disable_default_features, feature, arch_features),
            )
            .intersection(&possible_features)
            .cloned()
            .collect::<Vec<_>>();
            implied_features.sort();
            if implied_features.len() > 1 {
                Some((feature.to_string(), implied_features))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    implied_features.sort();
    implied_features
}

fn main() {
    let arches = [
        "x86",
        "x86_64",
        "arm",
        "aarch64",
        "mips",
        "mips64",
        "powerpc",
        "powerpc64",
        "riscv64",
    ];

    let mut file = std::fs::File::create(
        std::env::current_dir()
            .unwrap()
            .join("implied-features.txt"),
    )
    .unwrap();
    for (arch, features) in arches.iter().filter_map(|arch| {
        let features = get_implied_features(arch);
        if features.is_empty() {
            None
        } else {
            Some((arch, features))
        }
    }) {
        for (feature, implied) in features {
            writeln!(file, "{} {}: {}", arch, feature, implied.join(" ")).unwrap();
        }
    }
}
