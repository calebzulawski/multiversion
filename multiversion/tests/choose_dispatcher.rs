use multiversion::multiversion;

#[multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "arm+neon"),
    dispatcher = "default"
)]
fn default_dispatch() {}

#[multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "arm+neon"),
    dispatcher = "static"
)]
fn static_dispatch() {}

#[cfg(feature = "std")]
#[multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "arm+neon"),
    dispatcher = "direct"
)]
fn direct_dispatch() {}

#[cfg(feature = "std")]
#[multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "arm+neon"),
    dispatcher = "indirect"
)]
fn indirect_dispatch() {}

#[test]
fn dispatchers() {
    default_dispatch();
    static_dispatch();
    #[cfg(feature = "std")]
    direct_dispatch();
    #[cfg(feature = "std")]
    indirect_dispatch();
}
