// Accessor & serde contract tests (cargo-mutants triage).
//
// Every mutant that replaced an accessor body (`as_str` / `as_ref` /
// `into_inner` / serde `Serialize`) with a constant survived the legacy suite:
// the values were produced but never observed. These tests pin the observable
// contracts for all nine value types.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use validkit::{
    BucketName, CronExpr, EmailAddr, FlagName, HttpsUrl, LocaleTag, ObjectKey, PhoneE164,
    TenantIdSlug,
};

fn as_ref_str<T: AsRef<str>>(t: &T) -> &str {
    t.as_ref()
}

macro_rules! accessor_contract {
    ($ty:ty, $sample:literal) => {{
        let v = <$ty>::parse($sample).unwrap();
        assert_eq!(v.as_str(), $sample);
        assert_eq!(as_ref_str(&v), $sample);
        assert_eq!(v.into_inner(), $sample);
    }};
}

#[test]
fn bucket_name_accessors_return_stored_value() {
    accessor_contract!(BucketName, "my-bucket");
}

#[test]
fn cron_expr_accessors_return_stored_value() {
    accessor_contract!(CronExpr, "* * * * *");
}

#[test]
fn email_addr_accessors_return_stored_value() {
    accessor_contract!(EmailAddr, "user@example.com");
}

#[test]
fn flag_name_accessors_return_stored_value() {
    accessor_contract!(FlagName, "my_flag_1");
}

#[test]
fn locale_tag_accessors_return_stored_value() {
    accessor_contract!(LocaleTag, "en-US");
}

#[test]
fn object_key_accessors_return_stored_value() {
    accessor_contract!(ObjectKey, "path/to/file.txt");
}

#[test]
fn phone_e164_accessors_return_stored_value() {
    accessor_contract!(PhoneE164, "+14155552671");
}

#[test]
fn tenant_id_slug_accessors_return_stored_value() {
    accessor_contract!(TenantIdSlug, "acme-corp");
}

#[test]
fn https_url_accessors_return_stored_value() {
    let u = HttpsUrl::parse("https://example.com/x").unwrap();
    assert_eq!(u.as_str(), "https://example.com/x");
    assert_eq!(as_ref_str(&u), "https://example.com/x");
}

// --- serde contracts (Serialize body mutants) ---

#[cfg(feature = "serde")]
mod serde_contracts {
    use super::*;

    macro_rules! serde_roundtrip {
        ($ty:ty, $sample:literal) => {{
            let v = <$ty>::parse($sample).unwrap();
            let json = serde_json::to_string(&v).expect("serialize");
            assert_eq!(json, format!("\"{}\"", $sample));
            let back: $ty = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back.as_str(), $sample);
        }};
    }

    #[test]
    fn serde_roundtrip_preserves_bucket_name() {
        serde_roundtrip!(BucketName, "my-bucket");
    }

    #[test]
    fn serde_roundtrip_preserves_cron_expr() {
        serde_roundtrip!(CronExpr, "* * * * *");
    }

    #[test]
    fn serde_roundtrip_preserves_flag_name() {
        serde_roundtrip!(FlagName, "my_flag_1");
    }

    #[test]
    fn serde_roundtrip_preserves_locale_tag() {
        serde_roundtrip!(LocaleTag, "en-US");
    }

    #[test]
    fn serde_roundtrip_preserves_object_key() {
        serde_roundtrip!(ObjectKey, "path/to/file.txt");
    }

    #[test]
    fn serde_roundtrip_preserves_phone_e164() {
        serde_roundtrip!(PhoneE164, "+14155552671");
    }

    #[test]
    fn serde_roundtrip_preserves_https_url() {
        serde_roundtrip!(HttpsUrl, "https://example.com/x");
    }
}
