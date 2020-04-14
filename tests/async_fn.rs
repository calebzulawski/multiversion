#[rustversion::since(1.39)]
#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "[x86|x86_64]+sse")]
#[clone(target = "[arm|aarch64]+neon")]
async fn async_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

/*
mod test {
    #[rustversion::since(1.39)]
    #[test]
    fn async_fn() {
        let mut a = vec![0f32, 2f32, 4f32];
        let b = vec![1f32, 1f32, 1f32];
        futures::executor::block_on(super::async_add(&mut a, &b));
        assert_eq!(a, vec![1f32, 3f32, 5f32]);
    }
}
*/
