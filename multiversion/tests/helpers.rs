#[multiversion::multiversion(targets = "simd")]
fn foo() {
    const WIDTH: Option<usize> = multiversion::target::selected!().suggested_simd_width::<f32>();

    #[multiversion::inherit_target]
    unsafe fn inherited() {}
}
