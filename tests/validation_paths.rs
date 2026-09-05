// Tests exercise hostile inputs and boundary rules directly; unwrap/expect,
// slicing, and panicking asserts are the test signal here.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use std::str::FromStr;
use validkit::{
    BucketName, CronExpr, EmailAddr, FlagName, HttpsUrl, LocaleTag, ObjectKey, PhoneE164,
    TenantIdSlug, ValidError,
};

// --- EmailAddr ---

#[test]
fn email_new_and_into_inner_roundtrip() {
    let e = EmailAddr::new("user@example.com".to_string()).unwrap();
    assert_eq!(e.as_str(), "user@example.com");
    assert_eq!(e.into_inner(), "user@example.com");
}

#[test]
fn email_rejects_overall_length_over_254() {
    // 250-char domain + local part pushes the total past 254.
    let local = "a".repeat(10);
    let domain = "b".repeat(250);
    let email = format!("{local}@{domain}.com");
    assert!(email.len() > 254);
    let err = EmailAddr::parse(&email).unwrap_err();
    assert!(matches!(&err, ValidError::InvalidEmail(msg) if msg.contains("254")));
}

#[test]
fn email_rejects_local_part_over_64() {
    let email = format!("{}@example.com", "a".repeat(65));
    let err = EmailAddr::parse(&email).unwrap_err();
    assert!(matches!(&err, ValidError::InvalidEmail(msg) if msg.contains("local part exceeds 64")));
}

#[test]
fn email_rejects_local_part_edge_dots() {
    for bad in [".a@example.com", "a.@example.com"] {
        let err = EmailAddr::parse(bad).unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidEmail(msg) if msg.contains("must not start or end with '.'")),
            "case {bad}"
        );
    }
}

#[test]
fn email_rejects_domain_shapes() {
    for bad in ["a@.b.com", "a@b.com.", "a@-b.com", "a@b.com-"] {
        let err = EmailAddr::parse(bad).unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidEmail(msg) if msg.contains("must not start or end with '.' or '-'")),
            "case {bad}: {err}"
        );
    }
    assert!(
        matches!(EmailAddr::parse("a@b .com").unwrap_err(), ValidError::InvalidEmail(msg) if msg.contains("whitespace"))
    );
}

#[test]
fn email_rejects_bad_domain_labels() {
    let long_label = format!("a@{}.com", "b".repeat(64));
    assert!(
        matches!(EmailAddr::parse(&long_label).unwrap_err(), ValidError::InvalidEmail(msg) if msg.contains("label exceeds 63")),
        "got: {:?}",
        EmailAddr::parse(&long_label)
    );
    // Trailing-hyphen labels reach the label loop; leading-hyphen domains
    // are caught earlier by the domain-edge check.
    let err = EmailAddr::parse("a@b-.co").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidEmail(msg) if msg.contains("label must not start or end with '-'")),
        "got: {err}"
    );
}

#[test]
#[cfg(feature = "idna")]
fn email_idna_normalises_unicode_domain() {
    let e = EmailAddr::parse("user@ÜNICODE.example").unwrap();
    assert_eq!(e.as_str(), "user@xn--nicode-2ya.example");
}

#[test]
#[cfg(feature = "idna")]
fn email_idna_rejects_malformed_punycode() {
    let err = EmailAddr::parse("a@xn--aaaaaaaaa.com").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidEmail(msg) if msg.contains("IDNA")),
        "got: {err}"
    );
}

#[test]
#[cfg(feature = "idna")]
fn email_rejects_when_punycode_expansion_blows_length_budget() {
    // Each single-"ü" label becomes "xn--tda" (7 chars), so a short
    // pre-normalisation address exceeds the 254-byte budget afterwards.
    let domain = "ü.".repeat(40) + "ü";
    let email = format!("a@{domain}");
    assert!(email.len() <= 254);
    let err = EmailAddr::parse(&email).unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidEmail(msg) if msg.contains("normalised email exceeds 254")),
        "got: {err}"
    );
}

#[test]
fn email_validate_trait_revalidates_inner_value() {
    let good = EmailAddr::parse("a@b.co").unwrap();
    assert!(validkit::Validate::validate(&good).is_ok());
}

#[test]
#[cfg(feature = "serde")]
fn email_serde_transparent_roundtrip() {
    let e = EmailAddr::parse("user@example.com").unwrap();
    let json = serde_json::to_string(&e).unwrap();
    assert_eq!(json, "\"user@example.com\"");
    let back: EmailAddr = serde_json::from_str(&json).unwrap();
    assert_eq!(back, e);
}

#[test]
#[cfg(feature = "serde")]
fn email_serde_rejects_invalid() {
    let err = serde_json::from_str::<EmailAddr>("\"no-at.example.com\"").unwrap_err();
    assert!(err.to_string().contains("invalid email"), "got: {err}");
}

// --- PhoneE164 ---

#[test]
fn phone_new_and_into_inner_roundtrip() {
    let p = PhoneE164::new("+14155552671".to_string()).unwrap();
    assert_eq!(p.as_str(), "+14155552671");
    assert_eq!(p.into_inner(), "+14155552671");
}

#[test]
fn phone_rejects_empty_and_whitespace() {
    assert!(matches!(
        PhoneE164::parse("").unwrap_err(),
        ValidError::InvalidPhone(msg) if msg.contains("empty")
    ));
    assert!(
        matches!(PhoneE164::parse("+1 415").unwrap_err(), ValidError::InvalidPhone(msg) if msg.contains("whitespace"))
    );
}

#[test]
fn phone_rejects_letter_digits() {
    let err = PhoneE164::parse("+1415E555").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidPhone(msg) if msg.contains("E.164") || msg.contains("only digits")),
        "got: {err}"
    );
}

#[test]
fn phone_boundary_lengths() {
    // "+1" + 1 digit minimum per ^\+[1-9]\d{1,14}$ → "+12" is the shortest.
    assert!(PhoneE164::parse("+12").is_ok());
    assert!(PhoneE164::parse("+1").is_err());
    // Maximum: + followed by 15 digits.
    let max = format!("+{}", "1".repeat(15));
    assert!(PhoneE164::parse(&max).is_ok());
    let too_long = format!("+{}", "1".repeat(16));
    assert!(PhoneE164::parse(&too_long).is_err());
}

// --- CronExpr ---

#[test]
fn cron_new_and_into_inner_roundtrip() {
    let c = CronExpr::new("*/5 * * * *".to_string()).unwrap();
    assert_eq!(c.as_str(), "*/5 * * * *");
    assert_eq!(c.into_inner(), "*/5 * * * *");
}

#[test]
fn cron_rejects_empty_and_whitespace_only() {
    assert!(
        matches!(CronExpr::parse("").unwrap_err(), ValidError::InvalidCron(msg) if msg.contains("empty"))
    );
    assert!(
        matches!(CronExpr::parse("   ").unwrap_err(), ValidError::InvalidCron(msg) if msg.contains("empty"))
    );
}

#[test]
fn cron_rejects_crlf_and_bad_field_counts() {
    assert!(CronExpr::parse("* * * *\n0").is_err());
    assert!(
        matches!(CronExpr::parse("* * * *").unwrap_err(), ValidError::InvalidCron(msg) if msg.contains("5 fields, got 4"))
    );
    assert!(
        matches!(CronExpr::parse("* * * * * *").unwrap_err(), ValidError::InvalidCron(msg) if msg.contains("5 fields, got 6"))
    );
}

#[test]
fn cron_rejects_punctuation_only_field() {
    // Punctuation present but no digit or '*'.
    let err = CronExpr::parse("* * * * -,").unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidCron(msg) if msg.contains("invalid cron field")),
        "got: {err}"
    );
}

#[test]
fn cron_rejects_leading_trailing_punctuation() {
    // Leading '/' (step must follow a value or '*').
    assert!(
        matches!(CronExpr::parse("/5 * * * *").unwrap_err(), ValidError::InvalidCron(msg) if msg.contains("invalid cron field"))
    );
    // Trailing '-', trailing ','.
    assert!(CronExpr::parse("1- * * * *").is_err());
    assert!(CronExpr::parse("* * * * 1,").is_err());
}

#[test]
fn cron_rejects_consecutive_punctuation() {
    for field in ["1//2", "1,,2", "1--2"] {
        let input = format!("* * * * {field}");
        let err = CronExpr::parse(&input).unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidCron(msg) if msg.contains("invalid cron field")),
            "case {field}: {err}"
        );
    }
}

#[test]
fn cron_accepts_common_forms() {
    for good in ["0 0 * * 0", "1-5 * * * *", "0 0 1,15 * *", "5-10/2 * * * *"] {
        assert!(CronExpr::parse(good).is_ok(), "case {good}");
    }
}

// --- TenantIdSlug ---

#[test]
fn tenant_new_and_into_inner_roundtrip() {
    let t = TenantIdSlug::new("acme-prod".to_string()).unwrap();
    assert_eq!(t.into_inner(), "acme-prod");
}

#[test]
fn tenant_enforces_length_bounds() {
    assert!(
        matches!(TenantIdSlug::parse("ab").unwrap_err(), ValidError::InvalidTenantId(msg) if msg.contains("3-63"))
    );
    let too_long = "a".repeat(64);
    assert!(
        matches!(TenantIdSlug::parse(&too_long).unwrap_err(), ValidError::InvalidTenantId(msg) if msg.contains("3-63"))
    );
    assert!(TenantIdSlug::parse(&"a".repeat(63)).is_ok());
}

#[test]
fn tenant_rejects_crlf_and_edge_chars() {
    assert!(TenantIdSlug::parse("abc\ndef").is_err());
    for bad in ["-abc", "abc-", "a..b", "abc.lock", "AB_C"] {
        assert!(TenantIdSlug::parse(bad).is_err(), "case {bad}");
    }
}

// --- LocaleTag ---

#[test]
fn locale_new_and_into_inner_roundtrip() {
    let l = LocaleTag::new("en-US".to_string()).unwrap();
    assert_eq!(l.into_inner(), "en-US");
}

#[test]
fn locale_rejects_empty_length_and_crlf() {
    assert!(matches!(
        LocaleTag::parse("").unwrap_err(),
        ValidError::InvalidLocale(msg) if msg.contains("empty")
    ));
    let long = format!("en-{}", "a".repeat(40));
    assert!(
        matches!(LocaleTag::parse(&long).unwrap_err(), ValidError::InvalidLocale(msg) if msg.contains("35"))
    );
    assert!(LocaleTag::parse("en\n-US").is_err());
}

#[test]
fn locale_rejects_bad_primary_and_subtags() {
    // Uppercase primary fails the BCP47 pattern.
    assert!(LocaleTag::parse("EN-us").is_err());
    // Digit-first primary.
    assert!(LocaleTag::parse("419-en").is_err());
    // Single-char primary.
    assert!(LocaleTag::parse("e").is_err());
    // Empty subtag.
    assert!(LocaleTag::parse("en-").is_err());
    // Non-alphanumeric subtag.
    assert!(LocaleTag::parse("en-U!").is_err());
}

#[test]
fn locale_allows_valid_compound_tags() {
    assert!(LocaleTag::parse("zh-Hans-CN").is_ok());
    assert!(LocaleTag::parse("es-419").is_ok());
}

// --- FlagName ---

#[test]
fn flag_new_and_into_inner_roundtrip() {
    let f = FlagName::new("new_checkout_v2".to_string()).unwrap();
    assert_eq!(f.into_inner(), "new_checkout_v2");
}

#[test]
fn flag_rejects_empty_crlf_and_bad_chars() {
    assert!(FlagName::parse("").is_err());
    assert!(FlagName::parse("flag\nname").is_err());
    for bad in ["Flag", "1flag", "flag-name", "flag name", ""] {
        assert!(FlagName::parse(bad).is_err(), "case {bad:?}");
    }
}

// --- BucketName ---

#[test]
fn bucket_new_and_into_inner_roundtrip() {
    let b = BucketName::new("logs-prod.eu-west-1".to_string()).unwrap();
    assert_eq!(b.into_inner(), "logs-prod.eu-west-1");
}

#[test]
fn bucket_enforces_length_bounds() {
    assert!(
        matches!(BucketName::parse("ab").unwrap_err(), ValidError::InvalidBucketName(msg) if msg.contains("3-63"))
    );
    let too_long = "a".repeat(64);
    assert!(
        matches!(BucketName::parse(&too_long).unwrap_err(), ValidError::InvalidBucketName(msg) if msg.contains("3-63"))
    );
}

#[test]
fn bucket_rejects_crlf_dots_and_hyphen_edges() {
    assert!(BucketName::parse("bucket\nname").is_err());
    assert!(BucketName::parse("my..bucket").is_err());
    assert!(BucketName::parse("my.-bucket").is_err());
    assert!(BucketName::parse("my-.bucket").is_err());
    assert!(BucketName::parse("bucket.").is_err());
    assert!(BucketName::parse(".bucket").is_err());
}

#[test]
fn bucket_ip_like_rules_match_aws() {
    // Real IPv4 shapes are rejected.
    assert!(BucketName::parse("192.168.1.1").is_err());
    assert!(BucketName::parse("1.2.3.4").is_err());
    // Out-of-range octets are NOT IPv4, so the name is acceptable.
    assert!(BucketName::parse("999.1.1.1").is_ok());
    assert!(BucketName::parse("256.256.256.256").is_ok());
    // Five dot-separated groups are not an IP shape.
    assert!(BucketName::parse("1.2.3.4.5").is_ok());
}

// --- ObjectKey ---

#[test]
fn object_key_new_and_into_inner_roundtrip() {
    let k = ObjectKey::new("photos/2026/cat.jpg".to_string()).unwrap();
    assert_eq!(k.into_inner(), "photos/2026/cat.jpg");
}

#[test]
fn object_key_rejects_empty_leading_slash_crlf() {
    assert!(matches!(
        ObjectKey::parse("").unwrap_err(),
        ValidError::InvalidObjectKey(msg) if msg.contains("empty")
    ));
    assert!(
        matches!(ObjectKey::parse("/etc/passwd").unwrap_err(), ValidError::InvalidObjectKey(msg) if msg.contains("leading") || msg.contains("start with '/'")),
        "got: {:?}",
        ObjectKey::parse("/etc/passwd")
    );
    assert!(ObjectKey::parse("a\nb").is_err());
}

#[test]
fn object_key_rejects_traversal_and_null_byte() {
    assert!(
        matches!(ObjectKey::parse("a/../b").unwrap_err(), ValidError::InvalidObjectKey(msg) if msg.contains("traversal"))
    );
    assert!(
        matches!(ObjectKey::parse("..").unwrap_err(), ValidError::InvalidObjectKey(msg) if msg.contains("traversal"))
    );
    // Interior '..' inside a segment is not traversal.
    assert!(ObjectKey::parse("file..txt").is_ok());
    // Null byte rejected.
    assert!(matches!(
        ObjectKey::parse("a\0b").unwrap_err(),
        ValidError::InvalidObjectKey(msg) if msg.contains("null byte")
    ));
}

#[test]
fn object_key_enforces_1024_byte_limit() {
    let too_long = "a".repeat(1025);
    assert!(
        matches!(ObjectKey::parse(&too_long).unwrap_err(), ValidError::InvalidObjectKey(msg) if msg.contains("1024"))
    );
    assert!(ObjectKey::parse(&"a".repeat(1024)).is_ok());
}

// --- HttpsUrl ---

#[test]
fn url_new_into_inner_and_accessors() {
    let u = HttpsUrl::new("https://example.com/x?y=1".to_string()).unwrap();
    assert_eq!(u.as_str(), "https://example.com/x?y=1");
    assert_eq!(u.to_string(), "https://example.com/x?y=1");
    // Deref to str.
    let s: &str = &u;
    assert!(s.starts_with("https://"));
    assert_eq!(u.into_inner(), "https://example.com/x?y=1");
}

#[test]
#[cfg(feature = "url")]
fn url_as_url_exposes_parsed_url() {
    let u = HttpsUrl::parse("https://example.com/deep/path").unwrap();
    assert_eq!(u.as_url().host_str(), Some("example.com"));
    assert_eq!(u.as_url().path(), "/deep/path");
}

#[test]
fn url_rejects_crlf_and_non_https() {
    assert!(matches!(
        HttpsUrl::parse("https://example.com\r\nX: y").unwrap_err(),
        ValidError::InvalidUrl(msg) if msg.contains("CR or LF")
    ));
    assert!(matches!(
        HttpsUrl::parse("http://example.com").unwrap_err(),
        ValidError::InvalidUrl(msg) if msg.contains("https")
    ));
    assert!(matches!(
        HttpsUrl::parse("ftp://example.com").unwrap_err(),
        ValidError::InvalidUrl(msg) if msg.contains("https")
    ));
    assert!(HttpsUrl::parse("").is_err());
}

// --- Serde roundtrips for the remaining newtypes ---

#[test]
#[cfg(feature = "serde")]
fn serde_transparent_roundtrips() {
    let tenant = TenantIdSlug::parse("acme").unwrap();
    assert_eq!(serde_json::to_string(&tenant).unwrap(), "\"acme\"");
    assert_eq!(
        serde_json::from_str::<TenantIdSlug>("\"acme\"").unwrap(),
        tenant
    );

    let flag = FlagName::parse("dark_mode").unwrap();
    assert_eq!(
        serde_json::from_str::<FlagName>("\"dark_mode\"").unwrap(),
        flag
    );

    let phone = PhoneE164::parse("+1234567890").unwrap();
    assert_eq!(serde_json::to_string(&phone).unwrap(), "\"+1234567890\"");
}

#[test]
#[cfg(feature = "serde")]
fn serde_rejects_invalid_values_for_all_types() {
    assert!(serde_json::from_str::<TenantIdSlug>("\"ab\"").is_err());
    assert!(serde_json::from_str::<FlagName>("\"BAD\"").is_err());
    assert!(serde_json::from_str::<PhoneE164>("\"1234\"").is_err());
    assert!(serde_json::from_str::<CronExpr>("\"* * * *\"").is_err());
    assert!(serde_json::from_str::<LocaleTag>("\"EN\"").is_err());
    assert!(serde_json::from_str::<BucketName>("\"ab\"").is_err());
    assert!(serde_json::from_str::<ObjectKey>("\"/abs\"").is_err());
    assert!(serde_json::from_str::<HttpsUrl>("\"http://example.com\"").is_err());
}

// --- TryFrom / FromStr across types ---

#[test]
fn try_from_impls_validate() {
    assert!(EmailAddr::try_from(String::from("a@b.co")).is_ok());
    assert!(EmailAddr::try_from("not-an-email").is_err());
    assert!(PhoneE164::try_from(String::from("+12")).is_ok());
    assert!(PhoneE164::try_from("12").is_err());
    assert!(CronExpr::try_from(String::from("* * * * *")).is_ok());
    assert!(CronExpr::try_from("* * *").is_err());
    assert!(TenantIdSlug::try_from(String::from("acme")).is_ok());
    assert!(TenantIdSlug::try_from("ab").is_err());
    assert!(LocaleTag::try_from(String::from("en-US")).is_ok());
    assert!(LocaleTag::try_from("EN-US").is_err());
    assert!(FlagName::try_from(String::from("ok_flag")).is_ok());
    assert!(FlagName::try_from("not-ok").is_err());
    assert!(BucketName::try_from(String::from("my-bucket")).is_ok());
    assert!(BucketName::try_from("BAD").is_err());
    assert!(ObjectKey::try_from(String::from("a/b")).is_ok());
    assert!(ObjectKey::try_from("/abs").is_err());
    assert!(HttpsUrl::try_from(String::from("https://example.com")).is_ok());
    assert!(HttpsUrl::try_from("http://example.com").is_err());
}

#[test]
fn from_str_impls_match_parse() {
    assert_eq!(
        EmailAddr::from_str("a@b.co").unwrap(),
        EmailAddr::parse("a@b.co").unwrap()
    );
    assert!(FlagName::from_str("Bad").is_err());
    assert!(BucketName::from_str("192.168.1.1").is_err());
    assert!(LocaleTag::from_str("en-US").is_ok());
    assert!(CronExpr::from_str("* * * * *").is_ok());
    assert!(PhoneE164::from_str("+1234").is_ok());
    assert!(TenantIdSlug::from_str("tenant").is_ok());
    assert!(HttpsUrl::from_str("https://example.com").is_ok());
}

// --- Display / Deref across types ---

#[test]
fn display_and_deref_across_types() {
    let email = EmailAddr::parse("a@b.co").unwrap();
    assert_eq!(email.to_string(), "a@b.co");
    let deref_email: &str = &email;
    assert_eq!(deref_email, "a@b.co");

    let cron = CronExpr::parse("* * * * *").unwrap();
    let deref_cron: &str = &cron;
    assert_eq!(deref_cron, "* * * * *");

    let tenant = TenantIdSlug::parse("acme").unwrap();
    let deref_tenant: &str = &tenant;
    assert_eq!(deref_tenant, "acme");

    let locale = LocaleTag::parse("en-US").unwrap();
    let deref_locale: &str = &locale;
    assert_eq!(deref_locale, "en-US");

    let flag = FlagName::parse("a_flag").unwrap();
    let deref_flag: &str = &flag;
    assert_eq!(deref_flag, "a_flag");

    let bucket = BucketName::parse("my-bucket").unwrap();
    let deref_bucket: &str = &bucket;
    assert_eq!(deref_bucket, "my-bucket");

    let key = ObjectKey::parse("a/b").unwrap();
    let deref_key: &str = &key;
    assert_eq!(deref_key, "a/b");

    let phone = PhoneE164::parse("+12").unwrap();
    let deref_phone: &str = &phone;
    assert_eq!(deref_phone, "+12");
}
