use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multiversion::multiversion;

#[cfg(feature = "std")]
#[multiversion(targets = "simd", dispatcher = "indirect")]
fn indirect_fn(values: &mut [f32]) {
    for v in values {
        *v *= *v;
    }
}

#[cfg(feature = "std")]
#[multiversion(targets = "simd", dispatcher = "direct")]
fn direct_fn(values: &mut [f32]) {
    for v in values {
        *v *= *v;
    }
}

#[multiversion(targets = "simd", dispatcher = "static")]
fn static_fn(values: &mut [f32]) {
    for v in values {
        *v *= *v;
    }
}

fn base_fn(values: &mut [f32]) {
    for v in values {
        *v *= *v;
    }
}

pub fn benchmark_dispatcher(c: &mut Criterion) {
    // Don't profile initial feature detection
    #[cfg(feature = "std")]
    {
        indirect_fn(&mut []);
        direct_fn(&mut []);
    }

    for len in &[0, 16, 1000] {
        let mut g = c.benchmark_group(&format!("{len} elements"));
        let mut i = vec![0f32; *len];

        #[cfg(feature = "std")]
        g.bench_function("indirect dispatcher", |b| {
            b.iter(|| indirect_fn(black_box(i.as_mut())))
        })
        .bench_function("direct dispatcher", |b| {
            b.iter(|| direct_fn(black_box(i.as_mut())))
        });

        g.bench_function("static dispatcher", |b| {
            b.iter(|| static_fn(black_box(i.as_mut())))
        })
        .bench_function("no multiversioning", |b| {
            b.iter(|| base_fn(black_box(i.as_mut())))
        });
        g.finish();
    }
}

criterion_group!(benches, benchmark_dispatcher);
criterion_main!(benches);
