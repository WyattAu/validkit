# validkit

Typed newtypes for validated domain primitives — replaces hand-rolled `is_valid_*` checks with parse-once, use-everywhere strong types.

[![Crates.io](https://img.shields.io/crates/v/validkit.svg)](https://crates.io/crates/validkit)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE-MIT)

## Purpose

`validkit` is an L0 leaf crate providing validated newtypes for emails, URLs, cron expressions, tenant slugs, locale tags, flag names, S3 bucket names, object keys, and E.164 phones. Each type guarantees its invariants at construction time.

## Types

| Type | Validation |
|------|------------|
| `EmailAddr` | RFC 5321 + IDNA, injection-safe, single `@`, 254 chars |
| `HttpsUrl` | `https` only, no credentials, host required |
| `PhoneE164` | E.164 `^\+[1-9]\d{1,14}$` |
| `CronExpr` | 5-field cron (`*` or `[\d,/\-]+`) |
| `TenantIdSlug` | `3–63` chars, `[a-z0-9-]`, no `..`, no `.lock` |
| `LocaleTag` | BCP 47 `^[a-z]{2,3}(-[A-Za-z0-9]+)*$`, max 35 |
| `FlagName` | `^[a-z][a-z0-9_]*$` |
| `BucketName` | S3 bucket rules, not IP, 3–63 chars |
| `ObjectKey` | S3 object key, no `..` traversal, ≤1024, no leading `/` |

## Features

- `std` (default) — enables `std` support
- `serde` — `Serialize`/`Deserialize` transparent
- `url` — use `url` crate for `HttpsUrl`
- `regex` — regex-backed validation (more precise)
- `chrono` — chrono integration (placeholder for timestamp types)
- `idna` — IDNA domain validation for emails
- `full` — enables all above
- `no_std` — `no_std` compatible (`extern crate alloc`)

No `unsafe` code (`#![forbid(unsafe_code)]`), `#![deny(missing_docs)]`.

## Usage

```rust
use validkit::{EmailAddr, HttpsUrl, TenantIdSlug};

let email = EmailAddr::parse("alice@example.com").expect("valid");
assert_eq!(email.as_str(), "alice@example.com");

let url = HttpsUrl::parse("https://example.com").expect("valid");
assert!(url.as_str().starts_with("https://"));

let tenant = TenantIdSlug::parse("my-tenant").expect("valid");
assert_eq!(tenant.as_str(), "my-tenant");

// All types impl TryFrom<String>, FromStr, Display, Deref<Target=str>, AsRef<str>
let flag: validkit::FlagName = "my_flag".parse().expect("valid");
```

## No_Std

```toml
validkit = { version = "0.1", default-features = false }
```

## License

MIT OR Apache-2.0

## Security

Threat model: [THREAT-MODEL.md](THREAT-MODEL.md).
