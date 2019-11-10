#[multiversion::target_clones("[x86|x86_64]+avx2+avx", "[x86|x86_64]+avx", "x86+sse")]
fn double<'a, T: Copy + std::ops::AddAssign>(x: &'a mut [T]) -> &'a mut T {
    assert!(x.len() > 0);
    for v in x.iter_mut() {
        *v += *v;
    }
    &mut x[0]
}

mod test {
    use super::*;

    #[test]
    fn generics() {
        let mut x = vec![0f32, 2f32, 4f32];
        let mut y = vec![0f64, 2f64, 4f64];
        *double(&mut x) = 1.0;
        *double(&mut y) = 2.0;
        assert_eq!(x, vec![1f32, 4f32, 8f32]);
        assert_eq!(y, vec![2f64, 4f64, 8f64]);
    }
}
