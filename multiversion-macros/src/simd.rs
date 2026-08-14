use crate::target::Target;
use proc_macro2::Span;
use syn::LitStr;

const SINCE_1_89: bool = rustversion::cfg!(since(1.89));
const SINCE_1_93: bool = rustversion::cfg!(since(1.93));
const NIGHTLY: bool = cfg!(feature = "nightly");

const TARGETS: &[(&str, bool)] = &[
    // x86-64-v4; Rust does not provide runtime detection for `lahfsahf`.
    (
        "x86_64+avx+avx2+avx512bw+avx512cd+avx512dq+avx512f+avx512vl+bmi1+bmi2+cmpxchg16b+f16c+fma+lzcnt+movbe+popcnt+sse3+sse4.1+sse4.2+ssse3+xsave",
        SINCE_1_89,
    ),
    // x86-64-v3
    (
        "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+lzcnt+movbe+popcnt+sse3+sse4.1+sse4.2+ssse3+xsave",
        true,
    ),
    // x86-64-v2
    (
        "x86_64+cmpxchg16b+popcnt+sse3+sse4.1+sse4.2+ssse3",
        true,
    ),
    (
        "x86+avx+avx2+fma+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
        true,
    ),
    ("x86+sse+sse2+sse3+sse4.1+sse4.2+ssse3", true),
    ("x86+sse+sse2", true),
    ("aarch64+fp16+sve+sve2", true),
    ("aarch64+fp16+sve", true),
    ("arm+neon", NIGHTLY),
    ("loongarch64+lasx", SINCE_1_89),
    ("mips+msa", NIGHTLY),
    ("mips64+msa", NIGHTLY),
    ("powerpc+altivec+vsx", NIGHTLY),
    ("powerpc+altivec", NIGHTLY),
    ("powerpc64+altivec+vsx", NIGHTLY),
    ("powerpc64+altivec", NIGHTLY),
    ("riscv32+v", NIGHTLY),
    ("riscv64+v", NIGHTLY),
    (
        "s390x+vector+vector-enhancements-1+vector-enhancements-2+vector-enhancements-3",
        SINCE_1_93,
    ),
    (
        "s390x+vector+vector-enhancements-1+vector-enhancements-2",
        SINCE_1_93,
    ),
    ("s390x+vector+vector-enhancements-1", SINCE_1_93),
    ("s390x+vector", SINCE_1_93),
];

pub(crate) fn targets(span: Span) -> Vec<Target> {
    TARGETS
        .iter()
        .filter(|target| target.1)
        .map(|target| Target::parse(&LitStr::new(target.0, span)).unwrap())
        .collect()
}
