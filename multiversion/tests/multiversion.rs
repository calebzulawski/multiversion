#![allow(clippy::blacklisted_name)]

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

#[multiversion]
#[specialize(target = "x86_64+avx2", fn = "test_fn_unsafe", unsafe = true)]
#[specialize(target = "x86_64", fn = "test_fn_safe")]
pub fn pub_test_fn(a: i64) -> i64 {
    println!("fallback");
    a
}

#[multiversion]
#[specialize(target = "x86_64+avx2", fn = "test_fn_unsafe", unsafe = true)]
#[specialize(target = "x86_64", fn = "test_fn_safe")]
fn priv_test_fn(a: i64) -> i64 {
    println!("fallback");
    a
}

#[multiversion]
#[specialize(target = "x86_64+avx2", fn = "test_fn_unsafe")]
#[specialize(target = "x86_64", fn = "test_fn_safe")]
pub unsafe fn pub_test_unsafe_fn(a: i64) -> i64 {
    println!("fallback");
    a
}

#[multiversion]
#[specialize(target = "x86_64+avx2", fn = "test_fn_unsafe")]
#[specialize(target = "x86_64", fn = "test_fn_safe")]
unsafe fn priv_test_unsafe_fn(a: i64) -> i64 {
    println!("fallback");
    a
}

struct Foo;
impl Foo {
    #[target("x86_64+avx2")]
    unsafe fn test_fn_unsafe(&self, a: i64) -> i64 {
        println!("avx2");
        a
    }

    fn test_fn_safe(&self, a: i64) -> i64 {
        println!("avx");
        a
    }

    #[multiversion]
    #[specialize(target = "x86_64+avx2", fn = "test_fn_unsafe", unsafe = true)]
    #[specialize(target = "x86_64", fn = "test_fn_safe")]
    pub fn pub_test_fn(&self, a: i64) -> i64 {
        println!("fallback");
        a
    }

    #[multiversion]
    #[specialize(target = "x86_64+avx2", fn = "test_fn_unsafe", unsafe = true)]
    #[specialize(target = "x86_64", fn = "test_fn_safe")]
    fn priv_test_fn(&self, a: i64) -> i64 {
        println!("fallback");
        a
    }

    #[multiversion]
    #[specialize(target = "x86_64+avx2", fn = "test_fn_unsafe")]
    #[specialize(target = "x86_64", fn = "test_fn_safe")]
    pub unsafe fn pub_test_unsafe_fn(&self, a: i64) -> i64 {
        println!("fallback");
        a
    }

    #[multiversion]
    #[specialize(target = "x86_64+avx2", fn = "test_fn_unsafe")]
    #[specialize(target = "x86_64", fn = "test_fn_safe")]
    unsafe fn priv_test_unsafe_fn(&self, a: i64) -> i64 {
        println!("fallback");
        a
    }
}

mod test {
    use super::*;

    #[test]
    fn specialize() {
        assert_eq!(pub_test_fn(123), 123);
        assert_eq!(priv_test_fn(123), 123);
        assert_eq!(unsafe { pub_test_unsafe_fn(123) }, 123);
        assert_eq!(unsafe { priv_test_unsafe_fn(123) }, 123);
    }

    #[test]
    fn associated_specialize() {
        let foo = Foo;
        assert_eq!(foo.pub_test_fn(123), 123);
        assert_eq!(foo.priv_test_fn(123), 123);
        assert_eq!(unsafe { foo.pub_test_unsafe_fn(123) }, 123);
        assert_eq!(unsafe { foo.priv_test_unsafe_fn(123) }, 123);
    }
}
