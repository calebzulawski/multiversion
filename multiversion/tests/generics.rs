#![allow(clippy::needless_lifetimes)]

#[rustversion::since(1.51)]
#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx2+avx")]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn double<'a, T: Copy + std::ops::AddAssign, const N: usize>(x: &'a mut [T; N]) -> &'a mut T {
    assert!(!x.is_empty());
    for v in x.iter_mut() {
        *v += *v;
    }
    &mut x[0]
}

struct Doubler<'a>(&'a bool);

impl<'a> Doubler<'a> {
    #[rustversion::since(1.51)]
    #[multiversion::multiversion]
    #[clone(target = "[x86|x86_64]+avx2+avx")]
    #[clone(target = "[x86|x86_64]+avx")]
    #[clone(target = "x86+sse")]
    fn double<'b, T: Copy + std::ops::AddAssign, const N: usize>(
        &self,
        x: &'b mut [T; N],
    ) -> &'b mut T {
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
    #[rustversion::since(1.51)]
    #[test]
    fn generics() {
        let mut x = [0f32, 2f32, 4f32];
        let mut y = [0f64, 2f64, 4f64];
        *super::double(&mut x) = 1.0;
        *super::double(&mut y) = 2.0;
        assert_eq!(x, [1f32, 4f32, 8f32]);
        assert_eq!(y, [2f64, 4f64, 8f64]);
    }

    #[rustversion::since(1.51)]
    #[test]
    fn associated_generics() {
        let do_it = true;
        let doubler = super::Doubler(&do_it);
        let mut x = [0f32, 2f32, 4f32];
        let mut y = [0f64, 2f64, 4f64];
        *doubler.double(&mut x) = 1.0;
        *doubler.double(&mut y) = 2.0;
        assert_eq!(x, [1f32, 4f32, 8f32]);
        assert_eq!(y, [2f64, 4f64, 8f64]);
    }
}
