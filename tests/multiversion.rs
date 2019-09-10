use multiversion::multiversion;

multiversion! {
    pub fn pub_test_fn(x: i64) -> i64
    "x86_64+avx2" => test_fn_unsafe,
    "x86_64+avx" => test_fn_safe,
    default => test_fallback
}

multiversion! {
    fn priv_test_fn(x: i64) -> i64
    "x86_64+avx2" => test_fn_unsafe,
    "x86_64+avx" => test_fn_safe,
    default => test_fallback
}

#[inline]
multiversion! {
    pub unsafe fn pub_test_unsafe_fn(x: i64) -> i64
    "x86_64+avx2" => test_fn_unsafe,
    "x86_64+avx" => test_fn_safe,
    default => test_fallback
}

#[inline]
multiversion! {
    unsafe fn priv_test_unsafe_fn(x: i64) -> i64
    "x86_64+avx2" => test_fn_unsafe,
    "x86_64+avx" => test_fn_safe,
    default => test_fallback
}

#[cfg(target_arch = "x86_64")]
unsafe fn test_fn_unsafe(a: i64) -> i64 {
    println!("avx512");
    a
}

#[cfg(target_arch = "x86_64")]
fn test_fn_safe(a: i64) -> i64 {
    println!("avx");
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
        assert_eq!(pub_test_fn(123), 123);
        assert_eq!(priv_test_fn(123), 123);
        assert_eq!(unsafe { pub_test_unsafe_fn(123) }, 123);
        assert_eq!(unsafe { priv_test_unsafe_fn(123) }, 123);
    }
}
