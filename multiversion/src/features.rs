// Default features from build.rs
include!(concat!(env!("OUT_DIR"), "/default_features.rs"));

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TargetFeatures(&'static [&'static str]);

macro_rules! const_slice_loop {
    { for ($name:ident, $index:ident) in $slice:expr => $block:block } => {
        {
            let mut $index = 0;
            while $index < $slice.len() {
                let $name = $slice[$index];
                $block
                $index += 1;
            }
        }
    };
    { for $name:ident in $slice:expr => $block:block } => {
        const_slice_loop! {
            for ($name, __index) in $slice => $block
        }
    }
}

impl TargetFeatures {
    /// Assert that the provided features are supported.
    ///
    /// # Safety
    /// The provided features must be supported by the CPU.  This is usually indicated by feature
    /// detection.
    pub const unsafe fn with_features(features: &'static [&'static str]) -> Self {
        Self(features)
    }

    /// Create a new target instance, with only default features.
    pub const fn new() -> Self {
        Self(&[])
    }

    /// Check if the target supports a feature.
    pub const fn supports(&self, feature: &str) -> bool {
        // Check default features
        const_slice_loop! {
            for x in DEFAULT_FEATURES => {
                if string::eq(x, feature) {
                    return true;
                }
            }
        }

        // Check detected features
        const_slice_loop! {
            for x in self.0 => {
                if string::eq(x, feature) {
                    return true;
                }
            }
        }

        false
    }

    /// Returns a suggested number of elements for a SIMD vector of the provided type.
    ///
    /// The returned value is an approximation and not necessarily indicative of the optimal vector
    /// width.  A few caveats:
    /// * Every instruction set is different, and this function doesn't take into account any
    /// particular operations--it's just a guess, and should be accurate at least for basic arithmetic.
    /// * Variable length vector instruction sets (ARM SVE and RISC-V V) only use the minimum
    /// vector length.
    pub const fn suggested_simd_width<T: SimdType>(&self) -> Option<usize> {
        let is_f32 = string::eq(T::TYPE, "f32");
        let is_f64 = string::eq(T::TYPE, "f64");

        let v128 = 16 / core::mem::size_of::<T>();
        let v256 = 32 / core::mem::size_of::<T>();
        let v512 = 64 / core::mem::size_of::<T>();

        if cfg!(any(target_arch = "x86_64", target_arch = "x86")) {
            if self.supports("avx512f") {
                Some(v512)
            } else if self.supports("avx2") || (is_f32 || is_f64) && self.supports("avx") {
                // AVX supports f32 and f64
                Some(v256)
            } else if self.supports("sse2") || is_f32 && self.supports("sse") {
                // SSE supports f32
                Some(v128)
            } else {
                None
            }
        } else if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
            // Neon on armv7 doesn't support f64.
            if self.supports("neon") && !(is_f64 && cfg!(target_arch = "arm")) {
                Some(v128)
            } else {
                None
            }
        } else if cfg!(any(target_arch = "mips", target_arch = "mips64")) {
            if self.supports("msa") {
                Some(v128)
            } else {
                None
            }
        } else if cfg!(any(target_arch = "powerpc", target_arch = "powerpc64")) {
            // Altivec without VSX doesn't support f64.
            if self.supports("vsx") || (self.supports("altivec") && !is_f64) {
                Some(v128)
            } else {
                None
            }
        } else if cfg!(any(target_arch = "riscv32", target_arch = "riscv64")) {
            // V provides at least 128-bit vectors
            if self.supports("v") {
                Some(v128)
            } else {
                None
            }
        } else if cfg!(target_arch = "wasm32") && self.supports("simd128") {
            Some(v128)
        } else {
            None
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

pub trait SimdType: sealed::Sealed {
    #[doc(hidden)]
    const TYPE: &'static str;
}

macro_rules! impl_simd_type {
    { $($ty:ty),* } => {
        $(
        impl sealed::Sealed for $ty {}
        impl SimdType for $ty {
            const TYPE: &'static str = core::stringify!($ty);
        }
        )*
    }
}

impl_simd_type! { u8, u16, u32, u64, i8, i16, i32, i64, f32, f64 }

mod string {
    pub const fn eq(a: &str, b: &str) -> bool {
        let (a, b) = (a.as_bytes(), b.as_bytes());
        if a.len() != b.len() {
            false
        } else {
            const_slice_loop! {
                for (x, i) in a => {
                    if x != b[i] {
                        return false;
                    }
                }
            }
            true
        }
    }
}
