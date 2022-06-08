#![allow(clippy::blacklisted_name)]

struct Foo {
    bar: i64,
    baz: i64,
}

#[multiversion::multiversion(versions(clone = "x86_64+avx", clone = "arm+neon"))]
fn destructure_tuple_multiversion((x, y): (i64, i64)) -> (i64, i64) {
    (x, y)
}

#[multiversion::multiversion(versions(clone = "x86_64+avx", clone = "arm+neon"))]
fn destructure_struct_multiversion(Foo { bar, baz }: Foo) -> (i64, i64) {
    (bar, baz)
}

#[multiversion::multiversion(versions(clone = "x86_64+avx", clone = "arm+neon"))]
fn destructure_tuple((x, y): (i64, i64)) -> (i64, i64) {
    (x, y)
}

#[multiversion::multiversion(versions(clone = "x86_64+avx", clone = "arm+neon"))]
fn destructure_struct(Foo { bar, baz }: Foo) -> (i64, i64) {
    (bar, baz)
}

#[multiversion::multiversion(versions(clone = "x86_64+avx", clone = "arm+neon"))]
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
