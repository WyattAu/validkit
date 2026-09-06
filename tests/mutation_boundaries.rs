// Boundary & hostile-input tests (cargo-mutants triage).
//
// Each case pins a specific validation branch (message fragment asserted)
// whose mutants — operator swaps, constant replacement, body deletion —
// survived the legacy suite because no input reached that branch.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use validkit::{
    is_valid_bucket_name, is_valid_cron, is_valid_object_key, is_valid_tenant_id, BucketName,
    CronExpr, EmailAddr, HttpsUrl, LocaleTag, PhoneE164, TenantIdSlug, ValidError,
};

// --- BucketName: IP-like detection (is_ip_like) ---

#[test]
fn bucket_rejects_dotted_quad_ip_addresses() {
    let err = BucketName::parse("1.2.3.4").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidBucketName(m) if m.contains("IP address")),
        "{err}"
    );
}

#[test]
fn bucket_allows_dotted_numbers_that_are_not_ips() {
    // 999 > 255 → not an IPv4 address → allowed as a plain bucket name.
    assert!(BucketName::parse("1.2.3.999").is_ok());
    // 255 is a valid octet → still IP-shaped → rejected.
    let err = BucketName::parse("1.2.3.255").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidBucketName(m) if m.contains("IP address")),
        "{err}"
    );
}

#[test]
fn bucket_rejects_cr_lf_with_specific_message() {
    let err = BucketName::parse("ab\rc").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidBucketName(m) if m.contains("CR or LF")),
        "{err}"
    );
}

#[test]
fn is_valid_bucket_name_matches_parse() {
    assert!(is_valid_bucket_name("my-bucket"));
    assert!(!is_valid_bucket_name("ab"));
    assert!(!is_valid_bucket_name("1.2.3.4"));
}

// --- EmailAddr: RFC 5321 length boundaries ---

#[test]
fn email_rejects_total_length_over_254() {
    let local = "a".repeat(60);
    let domain = format!("{}.{}.com", "b".repeat(63), "c".repeat(63));
    let email = format!("{local}@{domain}"); // 60 + 1 + 130 = 191 — fine
    assert!(EmailAddr::parse(&email).is_ok());

    let email255 = format!("{}@{}", "a".repeat(64), "b".repeat(62 + 1 + 63 + 1 + 63));
    assert_eq!(email255.len(), 255);
    let err = EmailAddr::parse(&email255).unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidEmail(m) if m.contains("exceeds 254")),
        "{err}"
    );
}

#[test]
fn email_rejects_local_part_over_64_octets() {
    let err = EmailAddr::parse(&format!("{}@example.com", "a".repeat(65))).unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidEmail(m) if m.contains("local part exceeds 64")),
        "{err}"
    );
    // Boundary: exactly 64 is fine.
    assert!(EmailAddr::parse(&format!("{}@example.com", "a".repeat(64))).is_ok());
}

#[test]
fn email_rejects_domain_label_over_63() {
    let err = EmailAddr::parse(&format!("u@{}.com", "d".repeat(64))).unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidEmail(m) if m.contains("label exceeds 63")),
        "{err}"
    );
    // Boundary: exactly 63 is fine.
    assert!(EmailAddr::parse(&format!("u@{}.com", "d".repeat(63))).is_ok());
}

// --- LocaleTag: total length boundary ---

#[test]
fn locale_rejects_total_length_over_35() {
    let ok35 = "en-abcdefgh-ijklmnop-qrstuvwx-abcde"; // 35 chars, BCP47-shaped
    assert_eq!(ok35.len(), 35);
    assert!(LocaleTag::parse(ok35).is_ok());

    let bad36 = "en-abcdefgh-ijklmnop-qrstuvwx-abcdef"; // 36 chars
    assert_eq!(bad36.len(), 36);
    let err = LocaleTag::parse(bad36).unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidLocale(m) if m.contains("exceeds 35")),
        "{err}"
    );
}

// --- PhoneE164: whitespace/control rejection ---

#[test]
fn phone_rejects_embedded_space_with_specific_message() {
    let err = PhoneE164::parse("+1 415").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidPhone(m) if m.contains("must not contain whitespace")),
        "{err}"
    );
}

// --- TenantIdSlug: CR/LF rejection ---

#[test]
fn tenant_rejects_cr_lf_with_specific_message() {
    let err = TenantIdSlug::parse("abc\rd").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidTenantId(m) if m.contains("CR or LF")),
        "{err}"
    );
}

#[test]
fn is_valid_tenant_id_matches_parse() {
    assert!(is_valid_tenant_id("acme-corp"));
    assert!(!is_valid_tenant_id("ab"));
    assert!(!is_valid_tenant_id("-acme-"));
}

// --- HttpsUrl: control characters and credentials ---

#[test]
fn url_rejects_cr_lf_even_though_url_crate_strips_them() {
    let err = HttpsUrl::parse("https://example.com/\r\nx").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidUrl(m) if m.contains("CR or LF")),
        "{err}"
    );
}

#[test]
fn url_rejects_userinfo_credentials() {
    // Username only.
    let err = HttpsUrl::parse("https://user@example.com").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidUrl(m) if m.contains("credentials")),
        "{err}"
    );
    // Password only (empty username) — exercises the password arm.
    let err = HttpsUrl::parse("https://:secret@example.com").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidUrl(m) if m.contains("credentials")),
        "{err}"
    );
}

#[test]
fn is_valid_object_key_matches_parse() {
    assert!(is_valid_object_key("path/to/file.txt"));
    assert!(!is_valid_object_key("//double"));
}

// --- CronExpr: is_valid helper ---

#[test]
fn is_valid_cron_matches_parse() {
    assert!(is_valid_cron("* * * * *"));
    assert!(is_valid_cron("*/5 * * * *"));
    assert!(!is_valid_cron("nonsense"));
    assert!(!is_valid_cron("* * * *"));
}

#[test]
fn cron_rejects_field_starting_with_slash() {
    // The regex charset allows '/', so only the punctuation-position checks
    // reject a field that starts with it.
    let err = CronExpr::parse("/1 * * * *").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidCron(m) if m.contains("invalid cron field")),
        "{err}"
    );
}

// --- EmailAddr: exact total-length boundary and CR/LF arm ---

#[test]
fn email_allows_total_length_exactly_254() {
    // 3-octet local + '@' + 250-char domain (labels 63.63.63.57 + 3 dots = 252).
    let email = format!("abc@{}.{}.{}.{}.com", "b".repeat(63), "c".repeat(63), "d".repeat(63), "e".repeat(54));
    assert_eq!(email.len(), 254);
    assert!(EmailAddr::parse(&email).is_ok());
}

#[test]
fn email_rejects_lone_cr_with_specific_message() {
    let err = EmailAddr::parse("a@b\rc").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidEmail(m) if m.contains("CR or LF")),
        "{err}"
    );
}

// --- LocaleTag: CR/LF arm ---

#[test]
fn locale_rejects_lone_cr_with_specific_message() {
    let err = LocaleTag::parse("en\r").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidLocale(m) if m.contains("CR or LF")),
        "{err}"
    );
}

// --- PhoneE164: CR/LF arms (space arm covered above) ---

#[test]
fn phone_rejects_lone_cr_with_specific_message() {
    let err = PhoneE164::parse("+1\r41").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidPhone(m) if m.contains("must not contain whitespace")),
        "{err}"
    );
}

// --- HttpsUrl: CR without LF (the url crate strips CRLF pairs silently) ---

#[test]
fn url_rejects_lone_cr_with_specific_message() {
    let err = HttpsUrl::parse("https://example.com/\rx").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidUrl(m) if m.contains("CR or LF")),
        "{err}"
    );
}
