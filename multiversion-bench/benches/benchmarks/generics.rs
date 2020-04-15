use criterion::{criterion_group, Bencher, Criterion, Fun};
use multiversion::multiversion;
use rand::distributions::Standard;
use rand::Rng;

#[multiversion]
#[clone(target = "[x86|x86_64]+avx2+avx")]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn direct(input: &[f32], output: &mut [f32], factor: f32) {
    for (i, o) in input.iter().zip(output.iter_mut()) {
        *o = *i * factor
    }
}

#[multiversion]
#[clone(target = "[x86|x86_64]+avx2+avx")]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn generic<T: Copy + std::ops::Mul<Output = T>>(input: &[T], output: &mut [T], factor: T) {
    for (i, o) in input.iter().zip(output.iter_mut()) {
        *o = *i * factor
    }
}

fn bench(c: &mut Criterion) {
    for i in [4usize, 1000usize, 1_000_000usize].iter() {
        let input: Vec<f32> = rand::thread_rng()
            .sample_iter(&Standard)
            .take(*i)
            .collect::<Vec<_>>();
        let functions = vec![
            Fun::new("direct", |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| direct(i.as_slice(), o.as_mut_slice(), 2.0))
            }),
            Fun::new("generic", |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| generic(i.as_slice(), o.as_mut_slice(), 2.0))
            }),
        ];
        c.bench_functions(&format!("{} values", i), functions, input);
    }
}

criterion_group!(bench_generics, bench);
