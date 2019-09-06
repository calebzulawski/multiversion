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

fn test_avx(a: Foo, b: &Foo, c: &mut Foo, d: &mut [Foo]) -> (Foo, Foo) {
    (Foo(0), Foo(0))
}
fn test_popcnt(a: Foo, b: &Foo, c: &mut Foo, d: &mut [Foo]) -> (Foo, Foo) {
    (Foo(0), Foo(0))
}
fn test_fallback(a: Foo, b: &Foo, c: &mut Foo, d: &mut [Foo]) -> (Foo, Foo) {
    (Foo(0), Foo(0))
}
