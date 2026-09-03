use validkit::{
    BucketName, CronExpr, EmailAddr, FlagName, HttpsUrl, LocaleTag, ObjectKey, PhoneE164,
    TenantIdSlug,
};
use std::str::FromStr;

// Helpers
fn roundtrip<T>(s: &str, parse: impl Fn(&str) -> Result<T, validkit::ValidError>) -> bool
where
    T: ToString + FromStr<Err = validkit::ValidError>,
    T: AsRef<str>,
{
    let parsed = match parse(s) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let displayed = parsed.to_string();
    match T::from_str(&displayed) {
        Ok(reparsed) => reparsed.as_ref() == parsed.as_ref(),
        Err(_) => false,
    }
}

// Email -------------------------------------------------------------

#[test]
fn email_valid_cases() {
    let valid = [
        "a@b.co",
        "foo.bar+tag@example.org",
        "user@sub.domain.com",
        "test123@example.co.uk",
        "a@b.c", // minimal
    ];
    for case in valid {
        assert!(EmailAddr::parse(case).is_ok(), "should be valid: {}", case);
        assert!(
            roundtrip(case, EmailAddr::parse),
            "roundtrip failed: {}",
            case
        );
    }
}

#[test]
fn email_domain_lowercased() {
    let e = EmailAddr::parse("User@EXAMPLE.COM").expect("valid");
    assert_eq!(e.as_str(), "User@example.com");
    // roundtrip preserves lowercased domain
    let reparsed = EmailAddr::parse(e.as_str()).expect("reparse");
    assert_eq!(reparsed.as_str(), "User@example.com");
}

#[test]
fn email_invalid_cases() {
    let invalid = [
        "",
        "no-at.example.com",
        "a@@b.com",
        "@example.com",
        "a@",
        "a@b.com\r\nBcc: x@y.com",
        "a..b@example.com",
        ".a@example.com",
        "a.@example.com",
        "a@.example.com",
        "a@example.com.",
        "a@exam..ple.com",
        "a @example.com",
        "a@exam ple.com",
        "a<b>@example.com",
    ];
    for case in invalid {
        assert!(
            EmailAddr::parse(case).is_err(),
            "should be invalid: {:?}",
            case
        );
        assert!(!validkit::is_valid_email(case));
    }
}

#[test]
fn email_try_from_and_from_str() {
    let e: EmailAddr = "a@b.co".parse().expect("from_str");
    assert_eq!(e.as_str(), "a@b.co");
    let e2 = EmailAddr::try_from("a@b.co".to_string()).expect("try_from String");
    assert_eq!(e2.as_str(), "a@b.co");
    let e3 = EmailAddr::try_from("a@b.co").expect("try_from &str");
    assert_eq!(e3.as_str(), "a@b.co");
}

// HttpsUrl ----------------------------------------------------------

#[test]
#[cfg(feature = "url")]
fn https_url_valid() {
    let valid = [
        "https://example.com",
        "https://example.com/path?q=1#frag",
        "https://sub.domain.co.uk/a/b",
        "https://example.com:443/",
    ];
    for case in valid {
        let parsed = HttpsUrl::parse(case);
        assert!(parsed.is_ok(), "should be valid: {} {:?}", case, parsed);
        assert!(roundtrip(case, HttpsUrl::parse), "roundtrip failed: {}", case);
    }
}

#[test]
fn https_url_invalid() {
    let invalid = [
        "",
        "http://example.com",
        "https://",
        "https://user:pass@example.com",
        "ftp://example.com",
        "example.com",
        "https://exa mple.com",
    ];
    for case in invalid {
        assert!(
            HttpsUrl::parse(case).is_err(),
            "should be invalid: {:?}",
            case
        );
    }
}

#[test]
fn https_url_display_and_try_from() {
    let u: HttpsUrl = "https://example.com".parse().expect("from_str");
    let s = u.to_string();
    assert!(s.starts_with("https://"));
    let u2 = HttpsUrl::try_from("https://example.com".to_string()).expect("try_from");
    assert_eq!(u2.as_str(), u.as_str());
}

// Phone -------------------------------------------------------------

#[test]
fn phone_valid() {
    let valid = ["+12", "+14155552671", "+442071838750", "+123456789012345"];
    for case in valid {
        assert!(PhoneE164::parse(case).is_ok(), "should be valid: {}", case);
        assert!(roundtrip(case, PhoneE164::parse), "roundtrip failed: {}", case);
    }
}

#[test]
fn phone_invalid() {
    let invalid = [
        "",
        "14155552671",
        "+0123",
        "+1",
        "+1234567890123456",
        "+1 415 555 2671",
        "+abc",
        "+1\n23",
    ];
    for case in invalid {
        assert!(
            PhoneE164::parse(case).is_err(),
            "should be invalid: {:?}",
            case
        );
    }
}

// Cron --------------------------------------------------------------

#[test]
fn cron_valid() {
    let valid = [
        "* * * * *",
        "0 0 * * 0",
        "*/5 * * * *",
        "0 0 1,15 * 1-5",
        "0 0-23/2 * * *",
        "5 4 * * *",
    ];
    for case in valid {
        assert!(CronExpr::parse(case).is_ok(), "should be valid: {}", case);
        assert!(roundtrip(case, CronExpr::parse), "roundtrip failed: {}", case);
    }
}

#[test]
fn cron_invalid() {
    let invalid = [
        "",
        "* * * *",
        "* * * * * *",
        "a * * * *",
        "* * * * *\n",
        "*/ * * * *",
        "0 0 * *",
    ];
    for case in invalid {
        assert!(
            CronExpr::parse(case).is_err(),
            "should be invalid: {:?}",
            case
        );
        assert!(!validkit::is_valid_cron(case));
    }
}

// Tenant ------------------------------------------------------------

#[test]
fn tenant_valid() {
    let valid = ["abc", "my-tenant-123", "a-b-c", "tenant1", "foo-bar-baz"];
    for case in valid {
        assert!(
            TenantIdSlug::parse(case).is_ok(),
            "should be valid: {}",
            case
        );
        assert!(
            roundtrip(case, TenantIdSlug::parse),
            "roundtrip failed: {}",
            case
        );
    }
}

#[test]
fn tenant_invalid() {
    // Explicit checks
    assert!(TenantIdSlug::parse("ab").is_err());
    assert!(TenantIdSlug::parse(&"a".repeat(64)).is_err());
    assert!(TenantIdSlug::parse("-abc").is_err());
    assert!(TenantIdSlug::parse("abc-").is_err());
    assert!(TenantIdSlug::parse("Abc").is_err());
    assert!(TenantIdSlug::parse("a..b").is_err());
    assert!(TenantIdSlug::parse("foo.lock").is_err());
    assert!(TenantIdSlug::parse("my_tenant").is_err());
    assert!(TenantIdSlug::parse("a b").is_err());
    assert!(!validkit::is_valid_tenant_id("ab"));
}

// Locale ------------------------------------------------------------

#[test]
fn locale_valid() {
    let valid = ["en", "en-US", "es-419", "zh-Hans-CN", "fr-CA", "abc", "ab-12"];
    for case in valid {
        assert!(LocaleTag::parse(case).is_ok(), "should be valid: {}", case);
        assert!(
            roundtrip(case, LocaleTag::parse),
            "roundtrip failed: {}",
            case
        );
    }
}

#[test]
fn locale_invalid() {
    assert!(LocaleTag::parse("").is_err());
    assert!(LocaleTag::parse("EN").is_err());
    assert!(LocaleTag::parse("e").is_err());
    assert!(LocaleTag::parse("en-").is_err());
    assert!(LocaleTag::parse("en--US").is_err());
    assert!(LocaleTag::parse("en-US-").is_err());
    assert!(LocaleTag::parse("en-@US").is_err());
    assert!(LocaleTag::parse(&"a".repeat(36)).is_err());
    assert!(!validkit::is_valid_locale("EN"));
}

// FlagName ----------------------------------------------------------

#[test]
fn flag_valid() {
    let valid = ["a", "flag_a", "ai_analyzers", "flag2", "my_flag_123"];
    for case in valid {
        assert!(FlagName::parse(case).is_ok(), "should be valid: {}", case);
        assert!(roundtrip(case, FlagName::parse), "roundtrip failed: {}", case);
    }
}

#[test]
fn flag_invalid() {
    assert!(FlagName::parse("").is_err());
    assert!(FlagName::parse("Flag").is_err());
    assert!(FlagName::parse("1flag").is_err());
    assert!(FlagName::parse("flag-name").is_err());
    assert!(FlagName::parse("flag name").is_err());
    assert!(FlagName::parse("_flag").is_err());
    assert!(!validkit::is_valid_flag_name("Flag"));
}

// Bucket ------------------------------------------------------------

#[test]
fn bucket_valid() {
    let valid = ["my-bucket", "my.bucket123", "abc", "a-b.c", "my-bucket-123"];
    for case in valid {
        assert!(BucketName::parse(case).is_ok(), "should be valid: {}", case);
        assert!(
            roundtrip(case, BucketName::parse),
            "roundtrip failed: {}",
            case
        );
    }
}

#[test]
fn bucket_invalid() {
    assert!(BucketName::parse("ab").is_err());
    assert!(BucketName::parse(&"a".repeat(64)).is_err());
    assert!(BucketName::parse("192.168.1.1").is_err());
    assert!(BucketName::parse("-bucket").is_err());
    assert!(BucketName::parse("bucket-").is_err());
    assert!(BucketName::parse(".bucket").is_err());
    assert!(BucketName::parse("Bucket").is_err());
    assert!(BucketName::parse("my_bucket").is_err());
    assert!(BucketName::parse("my..bucket").is_err());
    assert!(!validkit::is_valid_bucket_name("ab"));
}

// ObjectKey ---------------------------------------------------------

#[test]
fn object_key_valid() {
    let valid = [
        "foo/bar/baz.txt",
        "file..txt",
        "a",
        "foo/bar//baz",
        "my-object/key-123",
        "a/b/c/d",
    ];
    for case in valid {
        assert!(
            ObjectKey::parse(case).is_ok(),
            "should be valid: {}",
            case
        );
        assert!(
            roundtrip(case, ObjectKey::parse),
            "roundtrip failed: {}",
            case
        );
    }
}

#[test]
fn object_key_invalid() {
    assert!(ObjectKey::parse("").is_err());
    assert!(ObjectKey::parse("/foo/bar").is_err());
    assert!(ObjectKey::parse("../etc/passwd").is_err());
    assert!(ObjectKey::parse("foo/../bar").is_err());
    assert!(ObjectKey::parse("foo/..").is_err());
    assert!(ObjectKey::parse(&"a".repeat(1025)).is_err());
    assert!(ObjectKey::parse("foo\r\nbar").is_err());
    assert!(!validkit::is_valid_object_key("/foo"));
}

// Serde roundtrip (with serde feature)
#[test]
#[cfg(feature = "serde")]
fn serde_roundtrip() {
    let email = EmailAddr::parse("a@b.co").expect("valid");
    let json = serde_json::to_string(&email).expect("serialize");
    let de: EmailAddr = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(email, de);

    let tenant = TenantIdSlug::parse("my-tenant").expect("valid");
    let json = serde_json::to_string(&tenant).expect("serialize");
    let de: TenantIdSlug = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(tenant, de);
}

// Display -> parse idempotence for all types
#[test]
fn display_parse_idempotent() {
    let email = EmailAddr::parse("test@example.com").expect("valid");
    assert_eq!(email.to_string(), "test@example.com");
    let url = HttpsUrl::parse("https://example.com/foo").expect("valid");
    assert!(url.to_string().starts_with("https://"));
    let tenant = TenantIdSlug::parse("my-tenant").expect("valid");
    assert_eq!(tenant.to_string(), "my-tenant");
}
