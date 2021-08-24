use criterion::{criterion_group, Bencher, Criterion};
use multiversion::multiversion;
use rand::distributions::Standard;
use rand::Rng;

#[multiversion(clones("[x86|x86_64]+avx"))]
fn square(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}

pub fn bench(c: &mut Criterion) {
    assert!(is_x86_feature_detected!("avx"));
    for i in [4usize, 20usize, 1000usize].iter() {
        let input: Vec<f32> = rand::thread_rng()
            .sample_iter(&Standard)
            .take(*i)
            .collect::<Vec<_>>();
        let mut group = c.benchmark_group(&format!("square {} values", i));
        group
            .bench_with_input("generic", &input, |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| square_default_version(i.as_slice(), o.as_mut_slice()))
            })
            .bench_with_input("AVX (direct)", &input, |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| unsafe { square_avx_version(i.as_slice(), o.as_mut_slice()) })
            })
            .bench_with_input("AVX (via FMV)", &input, |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| square(i.as_slice(), o.as_mut_slice()))
            });
        group.finish();
    }
}

criterion_group!(bench_square, bench);
