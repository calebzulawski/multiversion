// Implicit unsafe operations are intentional: clones must preserve unsafe fn bodies.
#![allow(unsafe_op_in_unsafe_fn)]

use multiversion::{inherit_target, multiversion};

macro_rules! test_safety {
    ($name:ident, $dispatcher:literal) => {
        #[test]
        fn $name() {
            #[multiversion(targets("x86_64+avx", "aarch64+sve"), dispatcher = $dispatcher)]
            fn increment(value: u8) -> u8 {
                #[inherit_target]
                fn inner(value: u8) -> u8 {
                    value + 1
                }

                inner(value)
            }

            #[multiversion(targets("x86_64+avx", "aarch64+sve"), dispatcher = $dispatcher)]
            unsafe fn read(value: *const u8) -> u8 {
                *value
            }

            let value = 41;
            assert_eq!(increment(value), 42);
            // SAFETY: value points to a live, initialized u8.
            assert_eq!(unsafe { read(&value) }, value);
        }
    };
}

test_safety!(default_dispatch, "default");
test_safety!(static_dispatch, "static");
#[cfg(feature = "std")]
test_safety!(direct_dispatch, "direct");
#[cfg(feature = "std")]
test_safety!(indirect_dispatch, "indirect");
