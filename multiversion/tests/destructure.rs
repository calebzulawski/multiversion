struct Foo {
    x: i64,
    y: i64,
}

#[multiversion::multiversion(targets("x86_64+avx", "aarch64+neon"))]
fn destructure_tuple_multiversion((x, y): (i64, i64)) -> (i64, i64) {
    (x, y)
}

#[multiversion::multiversion(targets("x86_64+avx", "aarch64+neon"))]
fn destructure_struct_multiversion(Foo { x, y }: Foo) -> (i64, i64) {
    (x, y)
}

#[multiversion::multiversion(targets("x86_64+avx", "aarch64+neon"))]
fn destructure_tuple((x, y): (i64, i64)) -> (i64, i64) {
    (x, y)
}

#[multiversion::multiversion(targets("x86_64+avx", "aarch64+neon"))]
fn destructure_struct(Foo { x, y }: Foo) -> (i64, i64) {
    (x, y)
}

#[multiversion::multiversion(targets("x86_64+avx", "aarch64+neon"))]
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
        assert_eq!(destructure_struct(Foo { x: 1, y: 2 }), (1, 2));
        assert_eq!(destructure_struct_multiversion(Foo { x: 3, y: 4 }), (3, 4));
        assert_eq!(destructure_tuple_generic((1i64, 2i64)), (1, 2));
    }
}
