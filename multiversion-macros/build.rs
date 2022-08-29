use std::{env, error::Error, fs::File, io::Write, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let features = include_str!("implied-features.txt");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let mut module = File::create(Path::new(&out_dir).join("implied_features.rs"))?;

    writeln!(
        module,
        "fn implied_features<'f>(arch: &str, feature: &'f str) -> Vec<&'f str> {{"
    )
    .unwrap();

    for line in features.lines() {
        let mut split = line.split(' ');
        let arch = split.next().unwrap();
        let feature = split.next().unwrap().strip_suffix(':').unwrap();
        let implied = split
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(",");

        writeln!(
            module,
            "    if arch == \"{arch}\" && feature == \"{feature}\" {{",
        )?;
        writeln!(module, "        return vec![{implied}];")?;
        writeln!(module, "    }}")?;
    }
    writeln!(module, "    vec![feature]")?;
    writeln!(module, "}}")?;

    println!("cargo:rerun-if-changed=implied-features.txt");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
