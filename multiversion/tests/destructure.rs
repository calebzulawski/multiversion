struct Foo {
    bar: i64,
    baz: i64,
}

#[multiversion::target("[x86|x86_64]+avx")]
unsafe fn destructure_tuple_avx((x, y): (i64, i64)) -> (i64, i64) {
    (x, y)
}

#[multiversion::multiversion]
#[specialize(
    target = "[x86|x86_64]+avx",
    fn = "destructure_tuple_avx",
    unsafe = true
)]
fn destructure_tuple_multiversion((x, y): (i64, i64)) -> (i64, i64) {
    (x, y)
}

#[multiversion::target("[x86|x86_64]+avx")]
unsafe fn destructure_struct_avx(Foo { bar, baz }: Foo) -> (i64, i64) {
    (bar, baz)
}

#[multiversion::multiversion]
#[specialize(
    target = "[x86|x86_64]+avx",
    fn = "destructure_struct_avx",
    unsafe = true
)]
fn destructure_struct_multiversion(Foo { bar, baz }: Foo) -> (i64, i64) {
    (bar, baz)
}

#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "[arm|aarch64]+neon")]
fn destructure_tuple((x, y): (i64, i64)) -> (i64, i64) {
    (x, y)
}

#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "[arm|aarch64]+neon")]
fn destructure_struct(Foo { bar, baz }: Foo) -> (i64, i64) {
    (bar, baz)
}

#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "[arm|aarch64]+neon")]
fn destructure_tuple_generic<T>((x, y): (T, T)) -> (T, T) {
    (x, y)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn destructure() {
        assert_eq!(destructure_tuple((1, 2)), (1, 2));
        assert_eq!(destructure_tuple_multiversion((3, 4)), (3, 4));
        assert_eq!(destructure_struct(Foo { bar: 1, baz: 2 }), (1, 2));
        assert_eq!(
            destructure_struct_multiversion(Foo { bar: 3, baz: 4 }),
            (3, 4)
        );
        assert_eq!(destructure_tuple_generic((1i64, 2i64)), (1, 2));
    }
}
