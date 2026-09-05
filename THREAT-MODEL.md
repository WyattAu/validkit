# Threat Model — validkit

Status: **v1.0** · Method: STRIDE over the public API surface: the nine
validated newtypes (`EmailAddr`, `HttpsUrl`, `PhoneE164`, `CronExpr`,
`TenantIdSlug`, `LocaleTag`, `FlagName`, `BucketName`, `ObjectKey`) and
their `is_valid_*` predicates.

Trust boundaries: (1) raw strings from users/HTTP entering `parse`, (2) the
two parallel validation engines (hand-rolled character loops vs. the
`regex`-crate paths selected by the `regex` feature), (3) downstream
consumers that trust a parsed value is safe to embed (headers, URLs, paths).

## Assets

| ID | Asset | Example |
|----|-------|---------|
| A1 | Downstream injection safety | CR/LF in an email → header injection when echoed into mail headers |
| A2 | Path/tenant containment | `..` in an `ObjectKey`/`TenantIdSlug` → traversal or lock-file collision |
| A3 | Caller availability | Catastrophic backtracking (ReDoS) during validation |
| A4 | Parse decisions | Inconsistent accept/reject between feature configurations |

## STRIDE Analysis

| # | Threat | Category | Surface | Mitigation | Verifying test |
|---|--------|----------|---------|------------|----------------|
| T1 | Email header injection (`\r`, `\n`) | Elevation | `EmailAddr::parse` | CR/LF rejected explicitly before any other check | `tests/integration.rs::email_invalid_cases` (CR/LF cases); `email_valid_cases`/`email_domain_lowercased` pin acceptance |
| T2 | ReDoS in validation | DoS | `CronExpr`, `LocaleTag`, `FlagName` under `regex` feature | All patterns are anchored fixed character classes over the linear-time `regex` crate (no backtracking engine); non-regex fallback is a hand-rolled loop with no backtracking | `tests/integration.rs::cron_valid`/`cron_invalid`, `locale_valid`/`locale_invalid`, `flag_valid`/`flag_invalid`; fuzzing via `fuzz/fuzz_targets/fuzz_parse.rs` |
| T3 | Path traversal via `..` | Elevation | `ObjectKey`, `TenantIdSlug` | `..` components rejected; object keys also cap length ≤ 1024 and forbid leading `/` | `tests/integration.rs::object_key_invalid`, `tenant_invalid`; `tests/proptest.rs::object_key_roundtrip`, `tenant_roundtrip` |
| T4 | Tenant lock-file collision (`foo.lock`) | Elevation | `TenantIdSlug` | `.lock` suffix rejected; `[a-z0-9-]` charset, 3–63 length | `tests/integration.rs::tenant_valid`/`tenant_invalid`; proptest `tenant_rejects_uppercase` |
| T5 | URL scheme downgrade / credential embedding | Spoofing | `HttpsUrl::parse` | `https` scheme enforced, host required, `username`/`password` in URL rejected, CR/LF rejected | `tests/integration.rs::https_url_valid`, `https_url_invalid`, `https_url_display_and_try_from` |
| T6 | Hostile input panics the parsers | DoS | all `parse` fns | `#![forbid(unsafe_code)]`; every rejection is a `ValidError`, length caps precede scanning (email ≤ 254, local ≤ 64) | `tests/proptest.rs::email_roundtrip` (adversarial strategy `arb_email`), `cron_roundtrip`, `phone_roundtrip`, `locale_roundtrip_via_strategy` |
| T7 | Phone/E.164 confusion (`+` missing, junk digits) | Spoofing | `PhoneE164` | `^\+[1-9]\d{1,14}$` | `tests/integration.rs::phone_valid`/`phone_invalid`; proptest `phone_rejects_missing_plus` |
| T8 | Bucket names that are IPs or uppercase | Spoofing | `BucketName` | S3 subset rules enforced | `tests/integration.rs::bucket_valid`/`bucket_invalid`; proptest `bucket_rejects_uppercase` |
| T9 | Injection via parsed values downstream | Elevation | all newtypes | Charset allow-lists strip `<>()[]\,;:"` in email local parts; cron fields restricted to `[\d*,/-]`; locale/flag/bucket allow-lists | `tests/integration.rs::email_invalid_cases` (problematic-character cases), `flag_invalid` |

## OPEN RISKS (missing mitigations — not fabricated)

- **OPEN-1 — two validation engines, no differential test.** The `regex`
  feature and the hand-rolled fallback implement the same rules twice
  (e.g. `src/cron.rs`, `src/locale.rs`, `src/flag_name.rs`). A drift between
  them silently changes what is accepted depending on feature flags; CI runs
  one engine at a time. A differential property test (same inputs, both
  engines, same verdict) is missing.
- **OPEN-2 — the no-`url`-crate `HttpsUrl` fallback is weaker.** Without the
  `url` feature, validation is a minimal prefix/host check (documented in
  `src/url.rs`); userinfo-with-`@` handling is cruder than the `url`-crate
  path. Same OPEN-1 class: feature-flag-dependent acceptance.
- **OPEN-3 — IDNA/punycode handling in email domains is not exercised.**
  README/docs claim "RFC 5321 + IDNA", but the domain checks are lowercase +
  charset tests; no test covers non-ASCII or punycode domains, so the actual
  guarantee is weaker than the claim.
- **OPEN-4 — percent-encoding/unicode confusion is out of scope by design.**
  Validated values are returned verbatim (`as_str`); callers embedding them
  into URLs or headers must still encode. Nothing in the API marks this.

## Out of Scope

- Deliverability of emails, reachability of URLs (syntax only).
- Phone number plans beyond E.164 shape (no region validation).
- Semantic cron validity (e.g. Feb 30) — field structure only.

## Residual Risks

- Validation is allow-list based but deliberately *not* RFC-complete for
  email (quoted strings, IP-literal domains rejected); stricter than RFC —
  false rejections, not false acceptances.
- Locale tags accept arbitrary subdomain-count chains up to the 35-char cap;
  BCP 47 canonicalization is not performed.
