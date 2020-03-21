use criterion::{criterion_group, Bencher, Criterion, Fun};
use multiversion::target_clones;
use rand::distributions::Standard;
use rand::Rng;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn square_avx(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}

fn square_generic(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}

#[target_clones("x86_64+avx")]
fn square_fmv(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}

pub fn bench(c: &mut Criterion) {
    for i in [4usize, 1000usize, 1_000_000usize].iter() {
        let input: Vec<f32> = rand::thread_rng()
            .sample_iter(&Standard)
            .take(*i)
            .collect::<Vec<_>>();
        let functions = vec![
            Fun::new("generic", |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| square_generic(i.as_slice(), o.as_mut_slice()))
            }),
            Fun::new("AVX (direct)", |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                assert!(is_x86_feature_detected!("avx"));
                b.iter(|| unsafe { square_avx(i.as_slice(), o.as_mut_slice()) })
            }),
            Fun::new("AVX (via FMV)", |b: &mut Bencher, i: &Vec<f32>| {
                let mut o = vec![0f32; i.len()];
                b.iter(|| square_fmv(i.as_slice(), o.as_mut_slice()))
            }),
        ];
        c.bench_functions(&format!("square {} values", i), functions, input);
    }
}

criterion_group!(bench_square, bench);
