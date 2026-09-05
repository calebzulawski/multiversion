use multiversion::multiversion;

#[cfg(feature = "std")]
#[test]
fn dispatcher_variable_names_in_parameters() {
    #[multiversion(targets("x86_64+avx", "aarch64+sve"), dispatcher = "indirect")]
    fn add(current_fn: u8, current_ptr: u8) -> u8 {
        current_fn + current_ptr
    }

    assert_eq!(add(10, 20), 30);
    assert_eq!(add(11, 21), 32);
}

#[test]
fn synthesized_parameter_names() {
    #[multiversion(targets("x86_64+avx", "aarch64+sve"))]
    fn add((value,): (u8,), arg_0: u8) -> u8 {
        value + arg_0
    }

    assert_eq!(add((10,), 20), 30);
    assert_eq!(add((11,), 21), 32);
}

#[cfg(feature = "std")]
#[test]
fn atomic_type_names_in_signature() {
    struct Ordering(u8);
    struct AtomicPtr(Ordering);

    #[multiversion(targets("x86_64+avx", "aarch64+sve"), dispatcher = "indirect")]
    fn wrap(value: Ordering) -> AtomicPtr {
        AtomicPtr(value)
    }

    let AtomicPtr(Ordering(value)) = wrap(Ordering(42));
    assert_eq!(value, 42);
}

#[cfg(feature = "std")]
#[test]
fn shadowed_standard_library_paths() {
    mod core {}
    mod std {}

    #[multiversion(targets("x86_64+avx", "aarch64+sve"), dispatcher = "direct")]
    fn direct() -> bool {
        multiversion::target::target_cfg_f!(target_pointer_width = "64")
    }

    #[multiversion(targets("x86_64+avx", "aarch64+sve"), dispatcher = "indirect")]
    fn indirect() -> bool {
        multiversion::target::target_cfg_f!(target_pointer_width = "64")
    }

    assert_eq!(direct(), ::core::cfg!(target_pointer_width = "64"));
    assert_eq!(indirect(), ::core::cfg!(target_pointer_width = "64"));
}

#[test]
fn shadowed_cfg_macro() {
    mod core {}
    #[allow(unused_macros)]
    macro_rules! cfg {
        ($($tokens:tt)*) => {
            compile_error!("resolved the caller's cfg macro")
        };
    }

    #[multiversion(targets("x86_64+avx", "aarch64+sve"), dispatcher = "static")]
    fn selected() -> bool {
        multiversion::target::target_cfg_f!(target_pointer_width = "64")
    }

    assert_eq!(selected(), ::core::cfg!(target_pointer_width = "64"));
}

#[multiversion(targets("x86_64+avx", "aarch64+sve"))]
fn r#type(value: u8) -> u8 {
    value + 1
}

#[multiversion(targets("x86_64+avx+avx2", "aarch64+sve", "x86_64+avx2+avx+avx", "aarch64+sve",))]
fn duplicate_targets(value: u8) -> u8 {
    value + 1
}

#[test]
fn names_and_duplicates() {
    assert_eq!(r#type(41), 42);
    assert_eq!(duplicate_targets(41), 42);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn duplicate_targets_preserve_priority() {
    #[multiversion(targets("x86_64+avx2", "x86_64+avx", "x86_64+avx2"))]
    fn selected_avx2() -> bool {
        multiversion::target::target_cfg_f!(target_feature = "avx2")
    }

    #[cfg(feature = "std")]
    let expected = std::arch::is_x86_feature_detected!("avx2");
    #[cfg(not(feature = "std"))]
    let expected = cfg!(target_feature = "avx2");
    assert_eq!(selected_avx2(), expected);
}
