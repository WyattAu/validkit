#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str::FromStr;
use validkit::{
    BucketName, CronExpr, EmailAddr, FlagName, HttpsUrl, LocaleTag, ObjectKey, PhoneE164,
    TenantIdSlug,
};

fuzz_target!(|data: &[u8]| {
    // Bound input so parse attempts stay fast.
    let s = String::from_utf8_lossy(&data[..data.len().min(4096)]);

    // Malformed or adversarial strings must return Err, never panic.
    let _ = EmailAddr::parse(&s);
    let _ = HttpsUrl::parse(&s);
    let _ = PhoneE164::parse(&s);
    let _ = CronExpr::parse(&s);
    let _ = TenantIdSlug::parse(&s);
    let _ = LocaleTag::parse(&s);
    let _ = FlagName::parse(&s);
    let _ = BucketName::parse(&s);
    let _ = ObjectKey::parse(&s);

    // FromStr must agree with the inherent `parse` constructor.
    let _ = EmailAddr::from_str(&s);
    let _ = HttpsUrl::from_str(&s);
    let _ = PhoneE164::from_str(&s);
    let _ = CronExpr::from_str(&s);
    let _ = TenantIdSlug::from_str(&s);
    let _ = LocaleTag::from_str(&s);
    let _ = FlagName::from_str(&s);
    let _ = BucketName::from_str(&s);
    let _ = ObjectKey::from_str(&s);

    // `is_valid_*` must never panic and must agree with the parse result.
    assert_eq!(validkit::is_valid_email(&s), EmailAddr::parse(&s).is_ok());
    assert_eq!(validkit::is_valid_cron(&s), CronExpr::parse(&s).is_ok());
    assert_eq!(
        validkit::is_valid_tenant_id(&s),
        TenantIdSlug::parse(&s).is_ok()
    );
    assert_eq!(validkit::is_valid_locale(&s), LocaleTag::parse(&s).is_ok());
    assert_eq!(
        validkit::is_valid_flag_name(&s),
        FlagName::parse(&s).is_ok()
    );
    assert_eq!(
        validkit::is_valid_bucket_name(&s),
        BucketName::parse(&s).is_ok()
    );
    assert_eq!(
        validkit::is_valid_object_key(&s),
        ObjectKey::parse(&s).is_ok()
    );

    // Accepted values must round-trip through Display losslessly.
    if let Ok(v) = ObjectKey::parse(&s) {
        assert_eq!(ObjectKey::parse(v.as_str()).as_ref(), Ok(&v));
    }
    if let Ok(v) = TenantIdSlug::parse(&s) {
        assert_eq!(TenantIdSlug::parse(v.as_str()).as_ref(), Ok(&v));
    }
});
