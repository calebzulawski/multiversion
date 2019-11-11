use multiversion::target_clones;
use futures::executor::block_on;

#[target_clones("[x86|x86_64]+avx", "[x86|x86_64]+sse", "[arm|aarch64]+neon")]
async fn async_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

mod test {
    use super::*;

    #[test]
    fn async_fn() {
        let mut a = vec![0f32, 2f32, 4f32];
        let b = vec![1f32, 1f32, 1f32];
        block_on(async_add(&mut a, &b));
        assert_eq!(a, vec![1f32, 3f32, 5f32]);
    }
}
