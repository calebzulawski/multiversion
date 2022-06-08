mod foo {
    #[multiversion::multiversion(versions(clone = "x86_64+avx", clone = "x86+sse"))]
    pub(super) fn mul(x: f32, y: f32) -> f32 {
        x * y
    }
}

#[multiversion::multiversion(versions(clone = "x86_64+avx", clone = "x86+sse"))]
fn square(x: &mut [f32]) {
    for v in x {
        *v = dispatch!(foo::mul(*v, *v));
    }
}

#[multiversion::multiversion(versions(clone = "x86_64+avx", clone = "x86+sse"))]
fn square_indirect(x: &mut [f32]) {
    let mul = dispatch!(foo::mul);
    for v in x {
        *v = mul(*v, *v);
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
fn static_dispatch_target() {
    #[multiversion::target("x86_64+avx")]
    unsafe fn square_avx(x: &mut [f32]) {
        dispatch!(square(x));
    }

    if multiversion::are_cpu_features_detected!("avx") {
        let mut x = vec![0f32, 1f32, 2f32, 3f32];
        unsafe {
            square_avx(x.as_mut_slice());
        }
        assert_eq!(x, vec![0f32, 1f32, 4f32, 9f32]);
    }
}
