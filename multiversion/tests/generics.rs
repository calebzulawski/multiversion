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
        let mut x = [0u32, 2u32, 4u32];
        let mut y = [0u64, 2u64, 4u64];
        *super::double(&mut x) = 1;
        *super::double(&mut y) = 2;
        assert_eq!(x, [1u32, 4u32, 8u32]);
        assert_eq!(y, [2u64, 4u64, 8u64]);
    }

    #[rustversion::since(1.51)]
    #[test]
    fn associated_generics() {
        let do_it = true;
        let doubler = super::Doubler(&do_it);
        let mut x = [0u32, 2u32, 4u32];
        let mut y = [0u64, 2u64, 4u64];
        *doubler.double(&mut x) = 1;
        *doubler.double(&mut y) = 2;
        assert_eq!(x, [1u32, 4u32, 8u32]);
        assert_eq!(y, [2u64, 4u64, 8u64]);
    }
}
