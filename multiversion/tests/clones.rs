use multiversion::multiversion;

#[multiversion(versions(
    clone = "[x86|x86_64]+avx",
    clone = "[x86|x86_64]+sse",
    clone = "[arm|aarch64]+neon",
))]
pub fn pub_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

#[multiversion(versions(
    clone = "[x86|x86_64]+avx",
    clone = "[x86|x86_64]+sse",
    clone = "[arm|aarch64]+neon",
))]
fn priv_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

#[multiversion(versions(
    clone = "[x86|x86_64]+avx",
    clone = "[x86|x86_64]+sse",
    clone = "[arm|aarch64]+neon",
))]
#[inline]
pub unsafe fn pub_unsafe_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

#[multiversion(versions(
    clone = "[x86|x86_64]+avx",
    clone = "[x86|x86_64]+sse",
    clone = "[arm|aarch64]+neon",
))]
#[inline]
unsafe fn priv_unsafe_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

struct Adder(f32);

impl Adder {
    #[multiversion(versions(
        clone = "[x86|x86_64]+avx",
        clone = "[x86|x86_64]+sse",
        clone = "[arm|aarch64]+neon",
    ))]
    #[inline]
    pub fn pub_add(&self, x: &mut [f32]) {
        x.iter_mut().for_each(|x| *x += self.0);
    }

    #[multiversion(versions(
        clone = "[x86|x86_64]+avx",
        clone = "[x86|x86_64]+sse",
        clone = "[arm|aarch64]+neon",
    ))]
    #[inline]
    fn priv_add(&self, x: &mut [f32]) {
        x.iter_mut().for_each(|x| *x += self.0);
    }

    #[multiversion(versions(
        clone = "[x86|x86_64]+avx",
        clone = "[x86|x86_64]+sse",
        clone = "[arm|aarch64]+neon",
    ))]
    #[inline]
    pub unsafe fn pub_unsafe_add(&self, x: &mut [f32]) {
        x.iter_mut().for_each(|x| *x += self.0);
    }

    #[multiversion(versions(
        clone = "[x86|x86_64]+avx",
        clone = "[x86|x86_64]+sse",
        clone = "[arm|aarch64]+neon",
    ))]
    #[inline]
    unsafe fn priv_unsafe_add(&self, x: &mut [f32]) {
        x.iter_mut().for_each(|x| *x += self.0);
    }
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

    #[test]
    fn test_add_associated() {
        let adder = Adder(1.);
        let mut a = vec![0f32, 2f32, 4f32];
        adder.pub_add(&mut a);
        assert_eq!(a, vec![1f32, 3f32, 5f32]);
        adder.priv_add(&mut a);
        assert_eq!(a, vec![2f32, 4f32, 6f32]);
        unsafe {
            adder.pub_unsafe_add(&mut a);
        }
        assert_eq!(a, vec![3f32, 5f32, 7f32]);
        unsafe {
            adder.priv_unsafe_add(&mut a);
        }
        assert_eq!(a, vec![4f32, 6f32, 8f32]);
    }
}
