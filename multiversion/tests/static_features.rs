use multiversion::{inherit_target, multiversion, target::target_cfg};

// Adds lanes with SSE2 on x86-64 and NEON on AArch64, which every build enables, so `target_cfg`
// selects that code in every function, the default included. Each function must have those target
// features for the intrinsics, which require them in their context, to be called without `unsafe`.
macro_rules! add {
    ($name:ident, $($target:literal),*) => {
        #[multiversion(targets($($target),*))]
        fn $name(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
            #[target_cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
            #[inherit_target]
            fn add_lanes(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
                use core::arch::x86_64::*;
                let load = |w: [u32; 4]| {
                    _mm_set_epi32(w[3] as i32, w[2] as i32, w[1] as i32, w[0] as i32)
                };
                let sum = _mm_add_epi32(load(a), load(b));
                [
                    _mm_cvtsi128_si32(sum) as u32,
                    _mm_cvtsi128_si32(_mm_shuffle_epi32::<1>(sum)) as u32,
                    _mm_cvtsi128_si32(_mm_shuffle_epi32::<2>(sum)) as u32,
                    _mm_cvtsi128_si32(_mm_shuffle_epi32::<3>(sum)) as u32,
                ]
            }

            #[target_cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            #[inherit_target]
            fn add_lanes(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
                use core::arch::aarch64::*;
                let load = |w: [u32; 4]| {
                    let v = vdupq_n_u32(w[0]);
                    let v = vsetq_lane_u32::<1>(w[1], v);
                    let v = vsetq_lane_u32::<2>(w[2], v);
                    vsetq_lane_u32::<3>(w[3], v)
                };
                let sum = vaddq_u32(load(a), load(b));
                [
                    vgetq_lane_u32::<0>(sum),
                    vgetq_lane_u32::<1>(sum),
                    vgetq_lane_u32::<2>(sum),
                    vgetq_lane_u32::<3>(sum),
                ]
            }

            #[target_cfg(not(any(
                all(target_arch = "x86_64", target_feature = "sse2"),
                all(target_arch = "aarch64", target_feature = "neon")
            )))]
            fn add_lanes(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
                core::array::from_fn(|i| a[i] + b[i])
            }

            add_lanes(a, b)
        }
    };
}

// Dispatch is elided: the build enables the best targets' features.
add!(add_static, "x86_64+sse2", "aarch64+neon");

// Dispatch is not elided: the build enables only some of the best targets' features.
add!(add_dispatched, "x86_64+sse2+avx2", "aarch64+neon+sve2");

#[test]
fn statically_enabled_features() {
    for add in [add_static, add_dispatched] {
        assert_eq!(add([1, 2, 3, 4], [10, 20, 30, 40]), [11, 22, 33, 44]);
    }
}
