use function_multiversioning::multiclones;

#[multiclones(
    specialize ("x86", "x86_64") {
        ("avx"), ("sse")
    },
    specialize ("arm", "aarch64") {
        ("neon")
    },
)]
fn add(a: &mut [f32], b: &mut [f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a = *a + b);
}
