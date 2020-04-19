# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/calebzulawski/multiversion/compare/0.5.0...HEAD
[0.5.0]: https://github.com/calebzulawski/multiversion/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/calebzulawski/multiversion/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/calebzulawski/multiversion/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/calebzulawski/multiversion/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/calebzulawski/multiversion/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/calebzulawski/multiversion/releases/tag/0.1.0
