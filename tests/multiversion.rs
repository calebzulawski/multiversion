use multiversion::multiversion;

multiversion! {
    fn test_fn(x: i64) -> i64
    "[x86|x86_64]+avx512f" => test_avx512,
    "[x86|x86_64]+avx+xsave" => test_avx,
    "[arm|aarch64]+neon" => test_neon,
    "[arm|aarch64]" => test_arm,
    "mips" => test_mips,
    default => test_fallback
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn test_avx512(a: i64) -> i64 {
    println!("avx512");
    a
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn test_avx(a: i64) -> i64 {
    println!("avx");
    a
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
fn test_neon(a: i64) -> i64 {
    println!("neon");
    a
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
fn test_arm(a: i64) -> i64 {
    println!("arm");
    a
}

#[cfg(target_arch = "mips")]
fn test_mips(a: i64) -> i64 {
    println!("mips");
    a
}

fn test_fallback(a: i64) -> i64 {
    println!("fallback");
    a
}

mod test {
    use super::*;

    #[test]
    fn call_test() {
        assert_eq!(test_fn(123), 123);
    }
}
