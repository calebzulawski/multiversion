use multiversion::multiversion;

#[multiversion(
    versions(
        clone = "[x86|x86_64]+avx",
        clone = "[x86|x86_64]+sse",
        clone = "[arm|aarch64]+neon",
    ),
    dispatcher = "default"
)]
fn default_dispatch() {}

#[multiversion(
    versions(
        clone = "[x86|x86_64]+avx",
        clone = "[x86|x86_64]+sse",
        clone = "[arm|aarch64]+neon",
    ),
    dispatcher = "static"
)]
fn static_dispatch() {}

#[cfg(feature = "std")]
#[multiversion(
    versions(
        clone = "[x86|x86_64]+avx",
        clone = "[x86|x86_64]+sse",
        clone = "[arm|aarch64]+neon",
    ),
    dispatcher = "direct"
)]
fn direct_dispatch() {}

#[cfg(feature = "std")]
#[multiversion(
    versions(
        clone = "[x86|x86_64]+avx",
        clone = "[x86|x86_64]+sse",
        clone = "[arm|aarch64]+neon",
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
