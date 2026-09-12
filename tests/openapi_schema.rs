//! OpenAPI feature: every newtype must expose a JSON Schema that reflects
//! its real validation constraints.
//!
//! Snapshot-style assertions per type, plus a pattern-agreement check proving
//! the published `pattern` accepts exactly what the validator's
//! pattern-expressible rules accept.
#![cfg(feature = "openapi")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use schemars::{schema_for, JsonSchema};
use serde_json::{json, Value};

fn schema_json<T: JsonSchema>() -> Value {
    serde_json::to_value(schema_for!(T)).expect("schema serializes")
}

fn assert_string_schema(schema: &Value, expected: Value) {
    // Every key in `expected` must appear with an equal value.
    let obj = expected.as_object().expect("expected is an object");
    for (key, want) in obj {
        assert_eq!(
            &schema[key], want,
            "schema key `{key}` mismatch; full schema:\n{schema}"
        );
    }
    assert_eq!(schema["type"], "string", "all newtypes are strings");
}

#[test]
fn email_schema_carries_format_and_max_length() {
    let schema = schema_json::<validkit::EmailAddr>();
    assert_string_schema(
        &schema,
        json!({
            "format": "email",
            "pattern": "^[^@\\s]+@[^@\\s]+$",
            "maxLength": 254
        }),
    );
}

#[test]
fn https_url_schema_carries_uri_format_and_scheme_pattern() {
    let schema = schema_json::<validkit::HttpsUrl>();
    assert_string_schema(
        &schema,
        json!({
            "format": "uri",
            "pattern": "^https://"
        }),
    );
}

#[test]
fn phone_schema_carries_e164_pattern_and_bounds() {
    let schema = schema_json::<validkit::PhoneE164>();
    assert_string_schema(
        &schema,
        json!({
            "pattern": "^\\+[1-9]\\d{1,14}$",
            "minLength": 3,
            "maxLength": 16
        }),
    );
}

#[test]
fn cron_schema_carries_five_field_pattern() {
    let schema = schema_json::<validkit::CronExpr>();
    assert_string_schema(
        &schema,
        json!({
            "pattern": "^[\\d*,/-]+([\\s]+[\\d*,/-]+){4}$",
            "minLength": 9
        }),
    );
}

#[test]
fn tenant_schema_carries_slug_constraints() {
    let schema = schema_json::<validkit::TenantIdSlug>();
    assert_string_schema(
        &schema,
        json!({
            "pattern": "^[a-z0-9]([a-z0-9-]{1,61}[a-z0-9])?$",
            "minLength": 3,
            "maxLength": 63
        }),
    );
}

#[test]
fn locale_schema_carries_bcp47_pattern() {
    let schema = schema_json::<validkit::LocaleTag>();
    assert_string_schema(
        &schema,
        json!({
            "pattern": "^[a-z]{2,3}(-[A-Za-z0-9]+)*$",
            "maxLength": 35
        }),
    );
}

#[test]
fn flag_name_schema_carries_snake_case_pattern() {
    let schema = schema_json::<validkit::FlagName>();
    assert_string_schema(
        &schema,
        json!({
            "pattern": "^[a-z][a-z0-9_]*$"
        }),
    );
}

#[test]
fn bucket_schema_carries_s3_constraints() {
    let schema = schema_json::<validkit::BucketName>();
    assert_string_schema(
        &schema,
        json!({
            "pattern": "^[a-z0-9]([a-z0-9.-]{1,61}[a-z0-9])?$",
            "minLength": 3,
            "maxLength": 63
        }),
    );
}

#[test]
fn object_key_schema_carries_s3_key_constraints() {
    let schema = schema_json::<validkit::ObjectKey>();
    assert_string_schema(
        &schema,
        json!({
            "pattern": "^[^/].*$",
            "minLength": 1,
            "maxLength": 1024
        }),
    );
}

/// The pattern we publish must agree with the validator on every input whose
/// rejection reason the pattern *can* express. Inputs the validator rejects
/// for non-pattern reasons (consecutive dots, 64-char local part, …) are the
/// documented gap covered by `format: email` and the schema description.
#[test]
fn email_pattern_agrees_with_validator() {
    let schema = schema_json::<validkit::EmailAddr>();
    let pattern = schema["pattern"].as_str().expect("pattern is a string");
    let re = regex::Regex::new(pattern).expect("published pattern compiles");

    // Validator accepts => pattern must accept.
    let accepted = [
        "a@b.co",
        "alice@example.com",
        "foo.bar+tag@example.org",
        "User@EXAMPLE.COM",
    ];
    for s in accepted {
        assert!(
            validkit::is_valid_email(s),
            "test bug: `{s}` should be validator-accepted"
        );
        assert!(re.is_match(s), "pattern must accept validator-valid `{s}`");
    }

    // Rejected for pattern-expressible reasons => pattern must reject.
    let pattern_rejects = [
        "no-at.example.com", // no '@'
        "a b@c.com",         // whitespace in local part
        "a@b c.com",         // whitespace in domain
        "a@@b.com",          // two '@'
        "@b.com",            // empty local part
        "a@",                // empty domain
    ];
    for s in pattern_rejects {
        assert!(!re.is_match(s), "pattern must reject `{s}`");
        assert!(!validkit::is_valid_email(s), "`{s}` must be rejected");
    }
}
