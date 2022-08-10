#[derive(Copy, Clone, PartialEq, Eq)]
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
    pub const unsafe fn with_features(features: &'static [&'static str]) -> Self {
        Self(features)
    }

    pub const fn supported(&self, feature: &str) -> bool {
        const_slice_loop! {
            for x in self.0 => {
                if string::eq(x, feature) {
                    return true;
                }
            }
        }
        false
    }

    const fn any_feature_starts_with(&self, needle: &str, except: Option<&str>) -> bool {
        const_slice_loop! {
            for feature in self.0 => {
                if string::starts_with(feature, needle) {
                    if let Some(except) = except {
                        if !string::eq(feature, except) {
                            return true;
                        }
                    } else {
                        return true;
                    }
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
    /// * Variable length vector instruction sets, like ARM SVE or RISC-V V, are not taken into
    /// account.
    pub const fn suggested_simd_width<T: SimdType>(&self) -> Option<usize> {
        macro_rules! supported {
            { $features:expr, $feature:literal } => {
                { cfg!(target_feature = $feature) || $features.supported($feature) }
            }
        }

        let is_f32 = string::eq(T::TYPE, "f32");
        let is_f64 = string::eq(T::TYPE, "f64");

        let v128 = 16 / core::mem::size_of::<T>();
        let v256 = 32 / core::mem::size_of::<T>();
        let v512 = 64 / core::mem::size_of::<T>();

        if cfg!(any(target_arch = "x86_64", target_arch = "x86")) {
            // Assume AVX-512 if any feature starting with "avx512" is present
            let avx512 =
                cfg!(target_feature = "avx512f") || self.any_feature_starts_with("avx512", None);

            // Assume AVX if any of "avx", "avx2", or "fma" are present.
            let avx =
                supported!(self, "avx") || supported!(self, "avx2") || supported!(self, "fma");

            // Assume SSE2 if any feature starting with "sse" is present, except "sse" itself.
            let sse2 =
                cfg!(target_feature = "sse2") || self.any_feature_starts_with("sse", Some("sse"));

            // Assume SSE if any feature starting with "sse" is present.
            let sse = cfg!(target_feature = "sse") || self.any_feature_starts_with("sse", None);

            if avx512 {
                Some(v512)
            } else if supported!(self, "avx2") {
                Some(v256)
            } else if (is_f32 || is_f64) && avx {
                // AVX supports f32 and f64
                Some(v256)
            } else if sse2 {
                Some(v128)
            } else if is_f32 && sse {
                // SSE supports f32
                Some(v128)
            } else {
                None
            }
        } else if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
            // Neon on armv7 doesn't support f64.
            if supported!(self, "neon") && !(is_f64 && cfg!(target_arch = "arm")) {
                Some(v128)
            } else {
                None
            }
        } else if cfg!(any(target_arch = "mips", target_arch = "mips64")) {
            if supported!(self, "msa") {
                Some(v128)
            } else {
                None
            }
        } else if cfg!(any(target_arch = "powerpc", target_arch = "powerpc64")) {
            // Altivec without VSX doesn't support f64.
            if supported!(self, "vsx") || (supported!(self, "altivec") && !is_f64) {
                Some(v128)
            } else {
                None
            }
        } else if cfg!(target_arch = "wasm32") && cfg!(target_feature = "simd128") {
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

    pub const fn starts_with(val: &str, needle: &str) -> bool {
        let (val, needle) = (val.as_bytes(), needle.as_bytes());
        if val.len() < needle.len() {
            false
        } else {
            const_slice_loop! {
                for (x, i) in needle => {
                    if val[i] != x {
                        return false;
                    }
                }
            }
            true
        }
    }
}
