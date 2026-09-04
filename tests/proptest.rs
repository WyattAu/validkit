use proptest::prelude::*;
use std::str::FromStr;
use validkit::{
    BucketName, CronExpr, EmailAddr, FlagName, LocaleTag, ObjectKey, PhoneE164, TenantIdSlug,
};

// Strategy for tenant slug: 3-63 chars, [a-z0-9-], not start/end hyphen, not .., not .lock
fn arb_tenant() -> impl Strategy<Value = String> {
    "[a-z0-9][a-z0-9\\-]{1,61}[a-z0-9]".prop_filter("no .. or .lock", |s: &String| {
        !s.contains("..") && !s.ends_with(".lock") && s.len() >= 3 && s.len() <= 63
    })
}

fn arb_flag() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}"
}

fn arb_bucket() -> impl Strategy<Value = String> {
    "[a-z0-9][a-z0-9\\-\\.]{1,61}[a-z0-9]".prop_filter("not ip, no ..", |s: &String| {
        if s.contains("..") || s.contains(".-") || s.contains("-.") {
            return false;
        }
        // Not IP
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() == 4 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
            return false;
        }
        s.len() >= 3 && s.len() <= 63
    })
}

fn arb_object_key() -> impl Strategy<Value = String> {
    // Avoid .. segment and leading /
    "[a-zA-Z0-9_\\-\\.]+(/[a-zA-Z0-9_\\-\\.]+)*".prop_filter("valid object key", |s: &String| {
        s.len() <= 1024
            && !s.is_empty()
            && !s.starts_with('/')
            && !s.split('/').any(|seg| seg == "..")
    })
}

fn arb_email() -> impl Strategy<Value = String> {
    // Simple: local + @ + domain
    ("[a-z]{1,8}", "[a-z]{1,8}\\.[a-z]{2,3}")
        .prop_map(|(local, domain)| format!("{}@{}", local, domain))
}

fn arb_cron() -> impl Strategy<Value = String> {
    // 5 fields, each * or digits/groups
    let field = prop_oneof![
        Just("*".to_string()),
        "[0-9]{1,2}",
        "[0-9]{1,2},[0-9]{1,2}",
        r"\*/[0-9]{1,2}",
        "[0-9]{1,2}-[0-9]{1,2}",
    ];
    (
        field.clone(),
        field.clone(),
        field.clone(),
        field.clone(),
        field,
    )
        .prop_map(|(a, b, c, d, e)| format!("{} {} {} {} {}", a, b, c, d, e))
}

fn arb_phone() -> impl Strategy<Value = String> {
    // +[1-9] followed by 1-14 digits => total 2-15 digits after '+'
    (2..16usize).prop_flat_map(|len| {
        let digits = prop::collection::vec(0..10u8, len);
        digits.prop_map(move |ds| {
            let mut s = String::from("+");
            // first digit 1-9
            let first = (ds[0] % 9) + 1;
            s.push(char::from(b'0' + first));
            for d in ds.iter().skip(1) {
                s.push(char::from(b'0' + *d));
            }
            s
        })
    })
}

proptest! {
    #[test]
    fn tenant_roundtrip(s in arb_tenant()) {
        let parsed = TenantIdSlug::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = TenantIdSlug::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }

    #[test]
    fn flag_roundtrip(s in arb_flag()) {
        let parsed = FlagName::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = FlagName::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }

    #[test]
    fn bucket_roundtrip(s in arb_bucket()) {
        let parsed = BucketName::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = BucketName::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }

    #[test]
    fn object_key_roundtrip(s in arb_object_key()) {
        let parsed = ObjectKey::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = ObjectKey::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }

    #[test]
    fn email_roundtrip(s in arb_email()) {
        let parsed = EmailAddr::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = EmailAddr::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }

    #[test]
    fn cron_roundtrip(s in arb_cron()) {
        let parsed = CronExpr::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = CronExpr::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }

    #[test]
    fn phone_roundtrip(s in arb_phone()) {
        let parsed = PhoneE164::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = PhoneE164::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }

    #[test]
    fn locale_roundtrip_via_strategy(s in "[a-z]{2,3}(-[A-Za-z0-9]{1,4}){0,2}") {
        // Filter to valid
        let re = regex::Regex::new(r"^[a-z]{2,3}(-[A-Za-z0-9]+)*$").unwrap();
        prop_assume!(re.is_match(&s) && s.len() <= 35);
        let parsed = LocaleTag::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = LocaleTag::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }
}

// Additional property: invalid inputs are rejected
proptest! {
    #[test]
    fn tenant_rejects_uppercase(s in "[A-Z]{3,10}") {
        prop_assert!(TenantIdSlug::parse(&s).is_err());
    }

    #[test]
    fn flag_rejects_hyphen(s in "[a-z]{1,5}-[a-z]{1,5}") {
        prop_assert!(FlagName::parse(&s).is_err());
    }

    #[test]
    fn phone_rejects_missing_plus(s in "[1-9][0-9]{1,10}") {
        prop_assert!(PhoneE164::parse(&s).is_err());
    }

    #[test]
    fn bucket_rejects_uppercase(s in "[A-Z]{3,10}") {
        prop_assert!(BucketName::parse(&s).is_err());
    }
}
