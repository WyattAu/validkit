# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [Unreleased]

## [0.1.0] - 2026-09-03

### Added

- Typed newtypes for validated domain primitives (email, URL, and more) —
  invalid values cannot be constructed.
- Feature-gated precision: `serde`, `url`, `regex`, `chrono`, `idna`,
  `full`.
- `no_std` compatibility (`extern crate alloc`) behind the `no_std`
  feature; `std` on by default.
- Criterion benches and proptest suites.
