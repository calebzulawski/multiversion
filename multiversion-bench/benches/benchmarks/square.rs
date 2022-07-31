use criterion::{criterion_group, Bencher, Criterion};
use multiversion_bench::*;
use rand::distributions::Standard;
use rand::Rng;

pub fn bench(c: &mut Criterion) {
    assert!(is_x86_feature_detected!("avx"));
    for i in [4usize, 20usize, 1000usize].iter() {
        let input: Vec<f32> = rand::thread_rng()
            .sample_iter(&Standard)
            .take(*i)
            .collect::<Vec<_>>();
        let mut group = c.benchmark_group(&format!("square {} values", i));
        group
            .bench_with_input(
                "default features",
                &input,
                |b: &mut Bencher, i: &Vec<f32>| {
                    let mut o = vec![0f32; i.len()];
                    b.iter(|| square(i.as_slice(), o.as_mut_slice()))
                },
            )
            .bench_with_input(
                "AVX (no multiversioning)",
                &input,
                |b: &mut Bencher, i: &Vec<f32>| {
                    let mut o = vec![0f32; i.len()];
                    b.iter(|| unsafe { square_avx(i.as_slice(), o.as_mut_slice()) })
                },
            )
            .bench_with_input(
                "AVX (indirect dispatch)",
                &input,
                |b: &mut Bencher, i: &Vec<f32>| {
                    let mut o = vec![0f32; i.len()];
                    b.iter(|| square_indirect(i.as_slice(), o.as_mut_slice()))
                },
            )
            .bench_with_input(
                "AVX (direct dispatch)",
                &input,
                |b: &mut Bencher, i: &Vec<f32>| {
                    let mut o = vec![0f32; i.len()];
                    b.iter(|| square_direct(i.as_slice(), o.as_mut_slice()))
                },
            );
        group.finish();
    }
}

criterion_group!(bench_square, bench);
