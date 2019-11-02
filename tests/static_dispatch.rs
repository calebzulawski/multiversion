use multiversion::target_clones;

#[target_clones("[x86|x86_64]+avx", "x86+sse")]
fn mul(x: f32, y: f32) -> f32 {
    x * y
}

#[target_clones("[x86|x86_64]+avx", "x86+sse")]
fn square(x: &mut [f32]) {
    #[static_dispatch]
    use mul;
    for v in x {
        *v = mul(*v, *v);
    }
}

#[test]
fn test_mul() {
    let mut x = vec![0f32, 1f32, 2f32, 3f32];
    square(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 4f32, 9f32]);
}
