#[multiversion::multiversion(targets = "simd")]
fn foo() {
    const WIDTH: Option<usize> =
        multiversion::target::selected_target!().suggested_simd_width::<f32>();
    println!("{WIDTH:?}");

    #[allow(unused)]
    #[multiversion::inherit_target]
    unsafe fn inherited() {}
}

#[test]
fn helpers() {
    foo()
}
