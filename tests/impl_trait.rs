#[multiversion::multiversion]
fn add_one(mut input: impl AsMut<[i64]> + AsRef<[i64]>) -> impl AsRef<[i64]> {
    for i in input.as_mut().iter_mut() {
        *i += 1;
    }
    input
}

#[test]
fn impl_trait() {
    let mut i = [0, 1, 2, 3];
    let o = add_one(&mut i);
    assert_eq!(o.as_ref(), [1, 2, 3, 4]);
}
