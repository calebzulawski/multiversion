#[multiversion::multiversion(targets = "simd")]
fn foo() {
    const WIDTH: Option<usize> = multiversion::selected_target!().suggested_simd_width::<f32>();
}
