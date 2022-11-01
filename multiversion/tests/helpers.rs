#[multiversion::multiversion(targets = "simd")]
fn foo() {
    const WIDTH: Option<usize> =
        multiversion::target::selected_target!().suggested_simd_width::<f32>();

    #[multiversion::inherit_target]
    unsafe fn inherited() {}
}
