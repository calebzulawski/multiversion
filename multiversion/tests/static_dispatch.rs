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
#[static_dispatch(fn = "self::foo::mul")]
#[cpu_features_token(name = "TOKEN")]
fn square(x: &mut [f32]) {
    for v in x {
        *v = mul(*v, *v);
    }
}

#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
#[static_dispatch(fn = "self::foo::mul", rename = "muls")]
#[cpu_features_token(name = "TOKEN")]
fn square2(x: &mut [f32]) {
    for v in x {
        *v = muls(*v, *v);
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
    #[static_dispatch(fn = "Self::mul")]
    #[cpu_features_token(name = "TOKEN")]
    fn square(&self, x: &mut [f32]) {
        for v in x {
            *v = mul(self, *v, *v);
        }
    }

    #[multiversion::multiversion]
    #[clone(target = "[x86|x86_64]+avx")]
    #[clone(target = "x86+sse")]
    #[static_dispatch(fn = "self::foo::mul", rename = "muls")]
    #[cpu_features_token(name = "TOKEN")]
    fn square2(&self, x: &mut [f32]) {
        for v in x {
            *v = muls(*v, *v);
        }
    }
}

#[test]
fn static_dispatch() {
    let mut x = vec![0f32, 1f32, 2f32, 3f32];

    square(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 4f32, 9f32]);
    square2(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 16f32, 81f32]);
}

#[test]
fn static_dispatch_associated() {
    let mut x = vec![0f32, 1f32, 2f32, 3f32];
    let squarer = Squarer;

    squarer.square(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 4f32, 9f32]);
    squarer.square2(x.as_mut_slice());
    assert_eq!(x, vec![0f32, 1f32, 16f32, 81f32]);
}
