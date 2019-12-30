# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- `multiversion` macro now requires `unsafe` to dispatch unsafe functions from safe functions
### Fixed
- Fixed incorrect argument forwarding for destructured function arguments
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

[Unreleased]: https://github.com/calebzulawski/multiversion/compare/0.2.0...HEAD
[0.2.0]: https://github.com/calebzulawski/multiversion/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/calebzulawski/multiversion/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/calebzulawski/multiversion/releases/tag/0.1.0
