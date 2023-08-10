use multiversion::multiversion;

#[multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "aarch64+neon"),
    dispatcher = "default"
)]
fn default_dispatch() {}

#[multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "aarch64+neon"),
    dispatcher = "static"
)]
fn static_dispatch() {}

#[cfg(feature = "std")]
#[multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "aarch64+neon"),
    dispatcher = "direct"
)]
fn direct_dispatch() {}

#[cfg(feature = "std")]
#[multiversion(
    targets("x86_64+avx", "x86+avx", "x86+sse", "aarch64+neon"),
    dispatcher = "indirect"
)]
fn indirect_dispatch() {}

// Since x86_64 always has sse, this should never result in runtime dispatch
#[multiversion(targets("x86_64+sse"), dispatcher = "default")]
fn skip_dispatch() {}

// Since aarch64 always has neon, this should never result in runtime dispatch
#[multiversion(targets("aarch64+neon"), dispatcher = "default")]
fn skip_dispatch_2() {}

#[test]
fn dispatchers() {
    default_dispatch();
    static_dispatch();
    #[cfg(feature = "std")]
    direct_dispatch();
    #[cfg(feature = "std")]
    indirect_dispatch();
    skip_dispatch();
    skip_dispatch_2();
}
