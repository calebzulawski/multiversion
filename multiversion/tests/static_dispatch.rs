mod foo {
    #[multiversion::multiversion]
    #[clone(target = "[x86|x86_64]+avx")]
    #[clone(target = "x86+sse")]
    pub(super) fn mul(x: f32, y: f32) -> f32 {
        x * y
    }
}

#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn square(x: &mut [f32]) {
    for v in x {
        *v = dispatch!(foo::mul(*v, *v));
    }
}

#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn square_indirect(x: &mut [f32]) {
    let mul = dispatch!(foo::mul);
    for v in x {
        *v = mul(*v, *v);
    }
}

struct Squarer;

impl Squarer {
    #[multiversion::multiversion]
    #[clone(target = "[x86|x86_64]+avx")]
    #[clone(target = "x86+sse")]
    fn mul(&self, x: f32, y: f32) -> f32 {
        x * y
    }

    #[multiversion::multiversion]
    #[clone(target = "[x86|x86_64]+avx")]
    #[clone(target = "x86+sse")]
    fn square(&self, x: &mut [f32]) {
        for v in x {
            *v = dispatch!(self.mul(*v, *v));
        }
    }

    #[multiversion::multiversion]
    #[clone(target = "[x86|x86_64]+avx")]
    #[clone(target = "x86+sse")]
    fn square_indirect(&self, x: &mut [f32]) {
        let mul = dispatch!(Self::mul);
        for v in x {
            *v = mul(self, *v, *v);
        }
    }
}

#[test]
fn static_dispatch() {
    let mut x = vec![0f32, 1f32, 2f32, 3f32];
    square(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 4f32, 9f32]);
}

#[test]
fn static_dispatch_indirect() {
    let mut x = vec![0f32, 1f32, 2f32, 3f32];
    square_indirect(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 4f32, 9f32]);
}

#[test]
fn static_dispatch_associated() {
    let mut x = vec![0f32, 1f32, 2f32, 3f32];
    let squarer = Squarer;
    squarer.square(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 4f32, 9f32]);
}

#[test]
fn static_dispatch_associated_indirect() {
    let mut x = vec![0f32, 1f32, 2f32, 3f32];
    let squarer = Squarer;
    squarer.square_indirect(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 4f32, 9f32]);
}
