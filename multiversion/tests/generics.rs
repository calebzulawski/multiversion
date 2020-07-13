#![allow(clippy::needless_lifetimes)]

#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx2+avx")]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn double<'a, T: Copy + std::ops::AddAssign>(x: &'a mut [T]) -> &'a mut T {
    assert!(!x.is_empty());
    for v in x.iter_mut() {
        *v += *v;
    }
    &mut x[0]
}

struct Doubler<'a>(&'a bool);
impl<'a> Doubler<'a> {
    #[multiversion::multiversion]
    #[clone(target = "[x86|x86_64]+avx2+avx")]
    #[clone(target = "[x86|x86_64]+avx")]
    #[clone(target = "x86+sse")]
    fn double<'b, T: Copy + std::ops::AddAssign>(&self, x: &'b mut [T]) -> &'b mut T {
        assert!(!x.is_empty());
        if *self.0 {
            for v in x.iter_mut() {
                *v += *v;
            }
        }
        &mut x[0]
    }
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

    #[test]
    fn associated_generics() {
        let do_it = true;
        let doubler = Doubler(&do_it);
        let mut x = vec![0f32, 2f32, 4f32];
        let mut y = vec![0f64, 2f64, 4f64];
        *doubler.double(&mut x) = 1.0;
        *doubler.double(&mut y) = 2.0;
        assert_eq!(x, vec![1f32, 4f32, 8f32]);
        assert_eq!(y, vec![2f64, 4f64, 8f64]);
    }
}
