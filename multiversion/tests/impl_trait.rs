#[multiversion::multiversion(targets("x86_64+avx2"))]
fn sum(input: impl AsRef<[i64]>) -> i64 {
    input.as_ref().iter().sum()
}

#[multiversion::multiversion(targets("x86_64+avx2"))]
fn sum_ref(input: &impl AsRef<[i64]>) -> i64 {
    input.as_ref().iter().sum()
}

#[test]
fn impl_trait() {
    assert_eq!(sum([0, 1, 2, 3]), 6);
    assert_eq!(sum_ref(&[0, 1, 2, 3]), 6);
}
