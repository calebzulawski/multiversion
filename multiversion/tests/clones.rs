use multiversion::multiversion;

#[multiversion(targets("x86_64+avx", "x86+avx", "x86+sse", "aarch64+neon",))]
pub fn pub_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

#[multiversion(targets("x86_64+avx", "x86+avx", "x86+sse", "aarch64+neon",))]
fn priv_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

#[multiversion(targets("x86_64+avx", "x86+avx", "x86+sse", "aarch64+neon",))]
pub unsafe fn pub_unsafe_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

#[multiversion(targets("x86_64+avx", "x86+avx", "x86+sse", "aarch64+neon",))]
unsafe fn priv_unsafe_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

mod test {
    use super::*;

    #[test]
    fn test_add() {
        let mut a = vec![0f32, 2f32, 4f32];
        let b = vec![1f32, 1f32, 1f32];
        pub_add(&mut a, &b);
        assert_eq!(a, vec![1f32, 3f32, 5f32]);
        priv_add(&mut a, &b);
        assert_eq!(a, vec![2f32, 4f32, 6f32]);
        unsafe {
            pub_unsafe_add(&mut a, &b);
        }
        assert_eq!(a, vec![3f32, 5f32, 7f32]);
        unsafe {
            priv_unsafe_add(&mut a, &b);
        }
        assert_eq!(a, vec![4f32, 6f32, 8f32]);
    }
}
