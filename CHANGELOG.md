# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.3] - 2023-08-10
### Fixed
- Don't include unstable target features in `targets = "simd"`.

## [0.7.2] - 2023-05-10
### Fixed
- Added workaround for documentation bug (https://github.com/rust-lang/rust/issues/111415)

## [0.7.1] - 2022-12-23
### Fixed
- Fixed handling patterns in `match_target`.

## [0.7.0] - 2022-12-09
### Changed
- The `multiversion` macro has been overhauled. Now uses a single attribute macro, rather than helper attributes.
- Increased minimum required Rust version to 1.61.0.
### Added
- The function dispatch method is now selectable, between direct or indirect dispatch, as well as compile-time static dispatch.
- Added a variety of macros in the `multiversion::target` module for querying the selected target features.
- Targets can now be specified by CPU (e.g. `x86-64-v2` or `skylake`).
- Added option to pass attributes to clones.
- Added special `targets = "simd"` option to automatically target all SIMD instruction sets.
### Removed
- Removed the `specialize` mode. All targets now specify clones. Specialization should be implemented by querying the selected targets.
- Removed support for functions that reference `self` or `Self`. Previous support was inconsistent and difficult to use correctly.
### Fixed
- Fixed broken `impl Trait` support.  Using `impl Trait` in return position now results in an error.
- Dispatch is now bypassed in scenarios where no targets are specified for the target architecture, or if the first matching target features are known to exist at compile time.
- Improved performance of direct dispatch.
- Avoid indirect dispatch in some situations, such as when using retpolines

## [0.6.1] - 2020-08-18
### Fixed
- Fixed disallowing some valid architectures, such as "wasm32"

## [0.6.0] - 2020-07-13
### Added
- Added `are_cpu_features_detected` macro.
- Added `#[crate_path]` helper attribute for renaming/reimporting the crate.
### Changed
- Changed `runtime_dispatch` Cargo feature to `std`
- Changed static dispatching from the `#[static_dispatch]` helper attribute to `dispatch!` helper macro.

## [0.5.1] - 2020-05-29
### Changed
- Removed dependency on `regex` and `once_cell`
### Fixed
- Fixed bug where `#[multiversion]` failed to compile when crate was re-exported or renamed.

## [0.5.0] - 2020-04-19
### Added
- Support for associated functions (including methods).
- Support for `impl Trait`.
- Specification for name mangling.
- Documentation for `#[safe_inner]` helper attribute for `#[target]`.
### Removed
- Removed `#[target_clones]` attribute (functionality is now included in `#[multiversion]` attribute).
### Changed
- `#[multiversion]` interface now uses helper attributes, providing both target specialization and function cloning.
- Increased minimum required Rust version to 1.34.0.
### Fixed
- Vague error spans now point to a more informative source location.
- All errors now produce `compile_error!` instead of macro panics.

## [0.4.0] - 2020-03-20
### Added
- Cargo feature `runtime_dispatch` (enabled by default) which adds runtime CPU feature detection.
- `#[no_std]` support when the `runtime_dispatch` feature is disabled.
### Fixed
- Fixed disallowing features with dots, such as `sse4.2`.

## [0.3.0] - 2019-12-30
### Added
- Conditional compilation with `#[target_cfg]` helper macro
- Support for const generics
### Changed
- `multiversion` macro now requires `unsafe` to dispatch unsafe functions from safe functions
### Fixed
- Fixed incorrect argument forwarding for destructured function arguments
- Static dispatch across modules now uses proper visibility
- Specifying architectures without features no longer results in compilation errors

## [0.2.0] - 2019-11-10
### Changed
- Improved ergonomics of `multiversion` macro.  It has been changed from a function-like macro to a macro attribute.
### Added
- `target` macro attribute
- Static dispatching with `#[static_dispatch]` helper macro
- Support for `async`, lifetimes, and generic functions
- This changelog

## [0.1.1] - 2019-09-10
### Fixed
- Removed an extra dependency

## [0.1.0] - 2019-09-10
### Added
- Initial multiversion implementation

[Unreleased]: https://github.com/calebzulawski/multiversion/compare/0.7.3...HEAD
[0.7.2]: https://github.com/calebzulawski/multiversion/compare/0.7.2...0.7.3
[0.7.2]: https://github.com/calebzulawski/multiversion/compare/0.7.1...0.7.2
[0.7.1]: https://github.com/calebzulawski/multiversion/compare/0.7.0...0.7.1
[0.7.0]: https://github.com/calebzulawski/multiversion/compare/0.6.1...0.7.0
[0.6.1]: https://github.com/calebzulawski/multiversion/compare/0.6.0...0.6.1
[0.6.0]: https://github.com/calebzulawski/multiversion/compare/0.5.1...0.6.0
[0.5.1]: https://github.com/calebzulawski/multiversion/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/calebzulawski/multiversion/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/calebzulawski/multiversion/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/calebzulawski/multiversion/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/calebzulawski/multiversion/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/calebzulawski/multiversion/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/calebzulawski/multiversion/releases/tag/0.1.0
