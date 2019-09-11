use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use multiversion::target_clones;
use rand::distributions::Standard;
use rand::Rng;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn square_avx_impl(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}

#[cfg(target_arch = "x86_64")]
fn square_avx(x: &mut [f32]) {
    assert!(is_x86_feature_detected!("avx"));
    unsafe {
        square_avx_impl(x);
    }
}

fn square_generic(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}

#[target_clones("x86_64+avx")]
fn square_fmv(x: &mut [f32]) {
    for v in x {
        *v *= *v;
    }
}

fn bench_square(c: &mut Criterion) {
    let mut group = c.benchmark_group("In-place square");
    for i in [4usize, 1000usize, 1000000usize].iter() {
        let mut generic_input: Vec<f32> = rand::thread_rng()
            .sample_iter(&Standard)
            .take(*i)
            .collect::<Vec<_>>();
        let mut avx_input = generic_input.clone();
        let mut fmv_input = generic_input.clone();
        group.bench_function(BenchmarkId::new("Generic", i), |b| {
            b.iter(|| square_generic(&mut generic_input))
        });
        group.bench_function(BenchmarkId::new("AVX (direct)", i), |b| {
            b.iter(|| square_avx(&mut avx_input))
        });
        group.bench_function(BenchmarkId::new("AVX (via FMV)", i), |b| {
            b.iter(|| square_fmv(&mut fmv_input))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_square);
criterion_main!(benches);
