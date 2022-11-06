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

// Since x86_64 always has sse, this should never result in runtime dispatch
#[multiversion(targets("x86_64+sse"), dispatcher = "default")]
fn skip_dispatch() {}

#[test]
fn dispatchers() {
    default_dispatch();
    static_dispatch();
    #[cfg(feature = "std")]
    direct_dispatch();
    #[cfg(feature = "std")]
    indirect_dispatch();
    skip_dispatch();
}
