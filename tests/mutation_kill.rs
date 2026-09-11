// Accessor roundtrip & residual-branch tests (cargo-mutants triage,
// complementary to mutation_boundaries.rs).
//
// Survivors killed here:
// - `as_str`/`as_ref` fn-body replacements ("xyzzy" / "") — no legacy test
//   observed the accessor output, only the parse result.
// - `is_valid_flag_name` / `is_valid_locale` constant-`false` replacements.
// - bucket.rs `is_ip_like` short-part arm (`is_empty() || len() > 3`) —
//   needs a 4-digit zero-prefixed part (length > 3, value ≤ 255).
// - cron.rs leading-punctuation guard — needs a field starting with ','.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use validkit::{
    is_valid_flag_name, is_valid_locale, BucketName, CronExpr, EmailAddr, FlagName, HttpsUrl,
    LocaleTag, ObjectKey, TenantIdSlug,
};

// --- as_str / as_ref roundtrips: the stored string must survive ---

#[test]
fn bucket_accessors_roundtrip() {
    let b = BucketName::parse("my-bucket").unwrap();
    assert_eq!(b.as_str(), "my-bucket");
    assert_eq!(b.as_ref(), "my-bucket");
}

#[test]
fn cron_accessors_roundtrip() {
    let c = CronExpr::parse("*/5 * * * *").unwrap();
    assert_eq!(c.as_str(), "*/5 * * * *");
    assert_eq!(c.as_ref(), "*/5 * * * *");
}

#[test]
fn email_accessors_roundtrip() {
    let e = EmailAddr::parse("user@example.com").unwrap();
    assert_eq!(e.as_str(), "user@example.com");
    assert_eq!(e.as_ref(), "user@example.com");
}

#[test]
fn flag_accessors_roundtrip() {
    let f = FlagName::parse("flag_one").unwrap();
    assert_eq!(f.as_str(), "flag_one");
    assert_eq!(f.as_ref(), "flag_one");
}

#[test]
fn locale_accessors_roundtrip() {
    let l = LocaleTag::parse("en-US").unwrap();
    assert_eq!(l.as_str(), "en-US");
    assert_eq!(l.as_ref(), "en-US");
}

#[test]
fn object_key_accessors_roundtrip() {
    let k = ObjectKey::parse("path/to/file.txt").unwrap();
    assert_eq!(k.as_str(), "path/to/file.txt");
    assert_eq!(k.as_ref(), "path/to/file.txt");
}

#[test]
fn tenant_accessors_roundtrip() {
    let t = TenantIdSlug::parse("acme-corp").unwrap();
    assert_eq!(t.as_str(), "acme-corp");
    assert_eq!(t.as_ref(), "acme-corp");
}

#[test]
fn url_accessor_roundtrip() {
    let u = HttpsUrl::parse("https://example.com").unwrap();
    #[cfg(feature = "url")]
    {
        // url crate normalises an empty path to "/", so assert the canonical form.
        assert_eq!(u.as_ref(), "https://example.com/");
    }
    #[cfg(not(feature = "url"))]
    {
        // String-backed storage keeps the input verbatim (no normalization).
        assert_eq!(u.as_ref(), "https://example.com");
    }
}

// --- is_valid_* free functions (constant-false replacements) ---

#[test]
fn is_valid_flag_name_accepts_valid_input() {
    assert!(is_valid_flag_name("flag_one"));
    assert!(!is_valid_flag_name("Flag"));
}

#[test]
fn is_valid_locale_accepts_valid_input() {
    assert!(is_valid_locale("en-US"));
    assert!(!is_valid_locale("e"));
}

// --- bucket.rs is_ip_like: short-part arm and octet boundary ---

#[test]
fn bucket_zero_padded_long_part_is_not_ip_like() {
    // "0000" has length > 3 (so `len() > 3` fires in is_ip_like) but parses
    // to 0 (≤ 255). The short-part arm must reject the *part*, making the
    // whole name not-IP-like and therefore a valid bucket name.
    assert!(BucketName::parse("1.2.3.0000").is_ok());
}

// --- bucket.rs length boundary (deterministic kill: the proptest only
// generates 63-char names with low probability, making this kill flaky) ---

#[test]
fn bucket_length_boundary_63_ok_64_err() {
    let ok63 = "a".repeat(63);
    assert_eq!(ok63.len(), 63);
    assert!(BucketName::parse(&ok63).is_ok());

    let err = BucketName::parse(&format!("{ok63}b")).unwrap_err();
    assert!(
        matches!(&err, validkit::ValidError::InvalidBucketName(m) if m.contains("3-63")),
        "{err}"
    );
}

// --- cron.rs leading-punctuation guard ---

#[test]
fn cron_rejects_field_starting_with_comma() {
    let err = CronExpr::parse(",5 * * * *").unwrap_err();
    assert!(err.to_string().contains(",5"), "{err}");
}

// --- CR/LF guards whose rejection the downstream validator masks ---
// The mutant (`||` → `&&`) lets the input through to a later guard that
// still rejects it — but with a DIFFERENT message. Asserting the message
// pins the branch.

#[test]
fn locale_cr_lf_message_comes_from_the_cr_lf_guard() {
    let err = LocaleTag::parse("en\rus").unwrap_err();
    assert!(
        matches!(&err, validkit::ValidError::InvalidLocale(m) if m.contains("CR or LF")),
        "{err}"
    );
}

#[test]
fn email_cr_lf_message_comes_from_the_cr_lf_guard() {
    let err = EmailAddr::parse("a@b\rc").unwrap_err();
    assert!(
        matches!(&err, validkit::ValidError::InvalidEmail(m) if m.contains("CR or LF")),
        "{err}"
    );
}
