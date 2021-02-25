use criterion::{criterion_group, Bencher, Criterion, Fun};
use multiversion::multiversion;
use rand::distributions::Standard;
use rand::Rng;

#[multiversion]
#[clone(target = "[x86|x86_64]+avx")]
fn square(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}

pub fn bench(c: &mut Criterion) {
    for i in [4usize, 20usize, 1000usize].iter() {
        let input: Vec<f32> = rand::thread_rng()
            .sample_iter(&Standard)
            .take(*i)
            .collect::<Vec<_>>();
        assert!(is_x86_feature_detected!("avx"));
        let functions = vec![
            Fun::new("generic", |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| square_default_version(i.as_slice(), o.as_mut_slice()))
            }),
            Fun::new("AVX (direct)", |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| unsafe { square_avx_version(i.as_slice(), o.as_mut_slice()) })
            }),
            Fun::new("AVX (via FMV)", |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| square(i.as_slice(), o.as_mut_slice()))
            }),
        ];
        c.bench_functions(&format!("square {} values", i), functions, input);
    }
}

criterion_group!(bench_square, bench);
