# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [1.1.0] - 2026-09-09

### Added

- `validkit-derive` 0.1.0: a new workspace proc-macro crate providing
  `#[derive(Validated)]`, which generates `validate(&self) ->
  Result<(), ValidError>` from field annotations:
  - `#[validate(email)]` — `String` / `Option<String>` via
    `is_valid_email`, or `EmailAddr` / `Option<EmailAddr>` leaves via the
    `Validate` trait;
  - `#[validate(url)]` — `String` / `Option<String>` via `HttpsUrl::parse`,
    or `HttpsUrl` / `Option<HttpsUrl>` leaves;
  - `#[validate(postcode_uk)]` — `String` / `Option<String>` structural UK
    postcode check (GIR 0AA included), implemented in-crate so generated
    code stays `no_std`-safe;
  - `#[validate(length(min = .., max = ..))]` — inclusive byte-length
    bounds on `String` / `Option<String>`;
  - `#[validate(range(min = .., max = ..))]` — inclusive bounds on numeric
    primitives and `Option` thereof.
- `derive` feature on `validkit` (non-default, excluded from `full`) that
  re-exports the macro; the `no_std` core and default build are untouched.
- `Validate` impl for `HttpsUrl` (parity with `EmailAddr`).
- `validkit::invalid_field(field, reason)` helper producing a
  `ValidError::InvalidValue` with uniform `field `x`: reason` formatting;
  used by generated code.
- `tests/derive.rs` covering all five rules, `Option` handling, newtype
  leaves, field-order reporting, `Validate`-trait interop, and generics.

## [1.0.0] - 2026-09-05

First stable release. The public API is now covered by the project's
semver guarantees: breaking changes require a major version bump.

### Fixed

- `serde::Deserialize` no longer bypasses validation. The newtypes derived
  `serde(transparent)` for both directions, so deserializing e.g.
  `"no-at.example.com"` into `EmailAddr` silently produced an invalid value.
  Serialization remains transparent; deserialization now runs the same
  validation as `parse` and rejects invalid input.

### Added

- `tests/fallback_paths.rs`: exercises the hand-rolled validators behind
  `#[cfg(not(feature = "regex"))]`, `#[cfg(not(feature = "url"))]`, and
  `#[cfg(not(feature = "idna"))]`, which never run under `--all-features`.
  CI now runs them via the `fallback-validators` job
  (`cargo test --no-default-features --features serde,std`).

## [0.1.0] - 2026-09-03

### Added

- Typed newtypes for validated domain primitives (email, URL, and more) —
  invalid values cannot be constructed.
- Feature-gated precision: `serde`, `url`, `regex`, `chrono`, `idna`,
  `full`.
- `no_std` compatibility (`extern crate alloc`) behind the `no_std`
  feature; `std` on by default.
- Criterion benches and proptest suites.
