use multiversion::{multiversion, target};

#[target("x86_64+avx2")]
unsafe fn test_fn_unsafe(a: i64) -> i64 {
    println!("avx2");
    a
}

fn test_fn_safe(a: i64) -> i64 {
    println!("avx");
    a
}

#[multiversion(
    "x86_64+avx2" => unsafe test_fn_unsafe,
    "x86_64" => test_fn_safe
)]
pub fn pub_test_fn(a: i64) -> i64 {
    println!("fallback");
    a
}

#[multiversion(
    "x86_64+avx2" => unsafe test_fn_unsafe,
    "x86_64" => test_fn_safe
)]
fn priv_test_fn(a: i64) -> i64 {
    println!("fallback");
    a
}

#[multiversion(
    "x86_64+avx2" => test_fn_unsafe,
    "x86_64" => test_fn_safe
)]
pub unsafe fn pub_test_unsafe_fn(a: i64) -> i64 {
    println!("fallback");
    a
}

#[multiversion(
    "x86_64+avx2" => test_fn_unsafe,
    "x86_64" => test_fn_safe
)]
unsafe fn priv_test_unsafe_fn(a: i64) -> i64 {
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
