use multiversion::target_clones;

#[target_clones("[x86|x86_64]+avx", "[x86|x86_64]+sse", "[arm|aarch64]+neon")]
fn add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a = *a + b);
}

mod test {
    use super::*;

    #[test]
    fn test_add() {
        let mut a = vec![0f32, 2f32, 4f32];
        let b = vec![1f32, 1f32, 1f32];
        add(&mut a, &b);
        assert_eq!(a, vec![1f32, 3f32, 5f32]);
    }
}
