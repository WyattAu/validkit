// Fallback-engine tests.
//
// The hand-rolled validators behind `#[cfg(not(feature = "regex"))]`,
// `#[cfg(not(feature = "url"))]`, and `#[cfg(not(feature = "idna"))]` are
// never compiled under `--all-features`, so the default coverage run cannot
// see them. Each module below runs the same valid/invalid cases through the
// fallback engine; CI executes this target with
// `cargo test --no-default-features --features serde,std`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

#[cfg(not(feature = "regex"))]
mod no_regex {
    use std::str::FromStr;
    use validkit::{CronExpr, FlagName, LocaleTag, PhoneE164, ValidError};

    #[test]
    fn phone_fallback_matches_e164_spec() {
        // Valid across the accepted range.
        for good in ["+12", "+14155552671", &format!("+{}", "1".repeat(15))] {
            let parsed = PhoneE164::parse(good).unwrap();
            assert_eq!(parsed.as_str(), good);
        }
        // Missing '+', leading zero, non-digit, too short, too long.
        for bad in [
            "14155552671",
            "+0123",
            "+12a4",
            "+1",
            &format!("+{}", "1".repeat(16)),
        ] {
            assert!(PhoneE164::parse(bad).is_err(), "case {bad}");
        }
        let err = PhoneE164::parse("+0123").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidPhone(msg) if msg.contains("must not start with 0")),
            "got: {err}"
        );
        let err = PhoneE164::parse("+1").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidPhone(msg) if msg.contains("2-15 digits")),
            "got: {err}"
        );
        let err = PhoneE164::parse("+12a4").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidPhone(msg) if msg.contains("only digits")),
            "got: {err}"
        );
        // Empty/whitespace pre-checks still apply before the fallback engine.
        assert!(PhoneE164::parse("").is_err());
        assert!(PhoneE164::parse("+1 415").is_err());
        // FromStr path through the same engine.
        assert!(PhoneE164::from_str("+1234").is_ok());
    }

    #[test]
    fn cron_fallback_field_rules() {
        for good in ["* * * * *", "0 0 * * 0", "*/5 * * * *", "0 0 1,15 * 1-5"] {
            assert!(CronExpr::parse(good).is_ok(), "case {good}");
        }
        // Wrong field counts, empty, whitespace-only, CR/LF.
        for bad in ["* * * *", "* * * * * *", "", "   ", "* * * *\n0"] {
            assert!(CronExpr::parse(bad).is_err(), "case {bad:?}");
        }
        // Invalid character in a field.
        let err = CronExpr::parse("a * * * *").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidCron(msg) if msg.contains("invalid character 'a'")),
            "got: {err}"
        );
        // Punctuation-only field (no digit, no '*').
        let err = CronExpr::parse("* * * * -,").unwrap_err();
        assert!(matches!(&err, ValidError::InvalidCron(msg) if msg.contains("invalid cron field")));
        // Leading/trailing punctuation.
        assert!(CronExpr::parse("/5 * * * *").is_err());
        assert!(CronExpr::parse("1- * * * *").is_err());
        assert!(CronExpr::parse("* * * * 1,").is_err());
        // Consecutive punctuation.
        for field in ["1//2", "1,,2", "1--2"] {
            assert!(
                CronExpr::parse(&format!("* * * * {field}")).is_err(),
                "case {field}"
            );
        }
    }

    #[test]
    fn locale_fallback_primary_and_subtag_rules() {
        for good in ["en", "en-US", "es-419", "zh-Hans-CN"] {
            assert!(LocaleTag::parse(good).is_ok(), "case {good}");
        }
        // Primary tag length bounds.
        let err = LocaleTag::parse("e").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidLocale(msg) if msg.contains("2-3 chars")),
            "got: {err}"
        );
        assert!(LocaleTag::parse("eng-Latn").is_ok()); // 3-char primary allowed
        assert!(LocaleTag::parse("engx").is_err());
        // Primary must be lowercase.
        let err = LocaleTag::parse("EN").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidLocale(msg) if msg.contains("lowercase a-z")),
            "got: {err}"
        );
        // Subtag rules: empty, >8 chars, non-alphanumeric.
        assert!(LocaleTag::parse("en-").is_err());
        let err = LocaleTag::parse("en-abcdefghi").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidLocale(msg) if msg.contains("exceeds 8")),
            "got: {err}"
        );
        let err = LocaleTag::parse("en-U!").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidLocale(msg) if msg.contains("alphanumeric")),
            "got: {err}"
        );
        // Empty, >35, CR/LF pre-checks.
        assert!(LocaleTag::parse("").is_err());
        assert!(LocaleTag::parse(&format!("en-{}", "a".repeat(40))).is_err());
        assert!(LocaleTag::parse("en\n-US").is_err());
    }

    #[test]
    fn flag_name_fallback_charclass() {
        for good in ["a", "flag_a", "ai_analyzers", "flag2"] {
            assert!(FlagName::parse(good).is_ok(), "case {good}");
        }
        for bad in ["", "Flag", "1flag", "flag-name", "flag name", "flag\nname"] {
            assert!(FlagName::parse(bad).is_err(), "case {bad:?}");
        }
        // First-char rule is distinct from the remainder charclass.
        assert!(FlagName::parse("_flag").is_err());
        assert!(FlagName::parse("f".repeat(64).as_str()).is_ok());
        // FromStr path.
        assert!(FlagName::from_str("ok_flag").is_ok());
        assert!(FlagName::from_str("not-ok").is_err());
    }
}

#[cfg(not(feature = "url"))]
mod no_url {
    use validkit::{HttpsUrl, ValidError};

    #[test]
    fn https_fallback_requires_scheme_host_no_credentials() {
        assert!(HttpsUrl::parse("https://example.com/path?q=1").is_ok());
        // Non-https scheme.
        let err = HttpsUrl::parse("http://example.com").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidUrl(msg) if msg.contains("scheme must be https")),
            "got: {err}"
        );
        // Missing host.
        let err = HttpsUrl::parse("https://").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidUrl(msg) if msg.contains("host is required")),
            "got: {err}"
        );
        // Credentials in host position.
        let err = HttpsUrl::parse("https://user:pass@example.com").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidUrl(msg) if msg.contains("credentials")),
            "got: {err}"
        );
        // Space in URL.
        let err = HttpsUrl::parse("https://example.com/a b").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidUrl(msg) if msg.contains("space")),
            "got: {err}"
        );
        // Empty input, CR/LF.
        assert!(HttpsUrl::parse("").is_err());
        assert!(HttpsUrl::parse("https://example.com\r\nX: y").is_err());
    }

    #[test]
    fn https_fallback_string_backed_accessors() {
        let u = HttpsUrl::parse("https://example.com/x").unwrap();
        assert_eq!(u.as_str(), "https://example.com/x");
        assert_eq!(u.to_string(), "https://example.com/x");
        let owned: String = u.into_inner();
        assert_eq!(owned, "https://example.com/x");
    }
}

#[cfg(not(feature = "idna"))]
mod no_idna {
    use validkit::{EmailAddr, ValidError};

    #[test]
    fn email_without_idna_accepts_ascii_and_lowercases_domain() {
        let e = EmailAddr::parse("User@EXAMPLE.COM").unwrap();
        assert_eq!(e.as_str(), "User@example.com");
    }

    #[test]
    fn email_without_idna_rejects_non_ascii_domains() {
        let err = EmailAddr::parse("a@ünicode.example").unwrap_err();
        assert!(
            matches!(&err, ValidError::InvalidEmail(msg) if msg.contains("non-ascii domain requires idna feature")),
            "got: {err}"
        );
    }
}
