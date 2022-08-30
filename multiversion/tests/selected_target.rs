#[multiversion::multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "arm+neon"),
    selected_target = "TARGET"
)]
pub fn selected_target() {
    println!("{:#?}", TARGET);
}
