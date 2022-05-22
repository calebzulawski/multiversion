use multiversion::multiversion;

#[multiversion(
    versions(
        clone = "x86_64+avx",
        clone = "x86+avx",
        clone = "x86+sse",
        clone = "arm+neon",
    ),
    dispatcher = "default"
)]
fn default_dispatch() {}

#[multiversion(
    versions(
        clone = "x86_64+avx",
        clone = "x86+avx",
        clone = "x86+sse",
        clone = "arm+neon",
    ),
    dispatcher = "static"
)]
fn static_dispatch() {}

#[cfg(feature = "std")]
#[multiversion(
    versions(
        clone = "x86_64+avx",
        clone = "x86+avx",
        clone = "x86+sse",
        clone = "arm+neon",
    ),
    dispatcher = "direct"
)]
fn direct_dispatch() {}

#[cfg(feature = "std")]
#[multiversion(
    versions(
        clone = "x86_64+avx",
        clone = "x86+avx",
        clone = "x86+sse",
        clone = "arm+neon",
    ),
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
