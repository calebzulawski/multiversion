use function_multiversioning::multiversion;

struct Foo(i64);

multiversion! {
    fn test(a: Foo, b: &Foo, c: &mut Foo, d: &mut [Foo]) -> (Foo, Foo)
    specialize x86 {
        ("avx", "sse2") => test_avx,
        ("popcnt") => test_popcnt,
    },
    default test_fallback
}

fn test_avx(_a: Foo, _b: &Foo, _c: &mut Foo, _d: &mut [Foo]) -> (Foo, Foo) {
    println!("avx");
    (Foo(0), Foo(0))
}
fn test_popcnt(_a: Foo, _b: &Foo, _c: &mut Foo, _d: &mut [Foo]) -> (Foo, Foo) {
    println!("popcnt");
    (Foo(0), Foo(0))
}
fn test_fallback(_a: Foo, _b: &Foo, _c: &mut Foo, _d: &mut [Foo]) -> (Foo, Foo) {
    println!("fallback");
    (Foo(0), Foo(0))
}

mod test {
    use super::*;

    #[test]
    fn call_test() {
        let a = Foo(1);
        let b = Foo(2);
        let mut c = Foo(3);
        let mut d = vec![Foo(4)];
        test(a, &b, &mut c, &mut d);
    }
}
