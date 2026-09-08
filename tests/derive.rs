//! Integration tests for the `Validated` derive (`validkit-derive`).
//!
//! Run with: `cargo test --feature derive` (covered by `--all-features`).

#![cfg(feature = "derive")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use validkit::{
    invalid_field, is_valid_email, EmailAddr, HttpsUrl, ValidError, Validate, Validated,
};

/// The spec's core case: email + url + length fields.
#[derive(Validated, Debug)]
struct Contact {
    #[validate(email)]
    email: String,
    #[validate(url)]
    homepage: Option<String>,
    #[validate(length(min = 1, max = 254))]
    display_name: String,
}

fn valid_contact() -> Contact {
    Contact {
        email: "alice@example.com".to_string(),
        homepage: Some("https://example.com".to_string()),
        display_name: "Alice".to_string(),
    }
}

#[test]
fn contact_valid() {
    assert!(valid_contact().validate().is_ok());
}

#[test]
fn contact_none_option_field_is_valid() {
    let c = Contact {
        homepage: None,
        ..valid_contact()
    };
    assert!(c.validate().is_ok());
}

#[test]
fn contact_invalid_email_reports_field() {
    let c = Contact {
        email: "not-an-email".to_string(),
        ..valid_contact()
    };
    let err = c.validate().unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidValue(msg) if msg.contains("email")),
        "unexpected error: {err}"
    );
    assert_eq!(
        err.to_string(),
        "invalid value: field `email`: not a valid email address"
    );
}

#[test]
fn contact_invalid_url_reports_field() {
    let c = Contact {
        homepage: Some("http://insecure.example.com".to_string()),
        ..valid_contact()
    };
    let err = c.validate().unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidValue(msg) if msg.contains("homepage")),
        "unexpected error: {err}"
    );
}

#[test]
fn contact_empty_display_name_fails_length() {
    let c = Contact {
        display_name: String::new(),
        ..valid_contact()
    };
    assert!(c.validate().is_err());
}

#[test]
fn contact_over_long_display_name_fails_length() {
    let c = Contact {
        display_name: "x".repeat(255),
        ..valid_contact()
    };
    assert!(c.validate().is_err());
}

#[test]
fn contact_length_boundary_values_pass() {
    for len in [1, 254] {
        let c = Contact {
            display_name: "x".repeat(len),
            ..valid_contact()
        };
        assert!(c.validate().is_ok(), "length {len} must pass");
    }
}

/// Newtype-typed fields act as leaves: validation delegates to the newtype.
#[derive(Validated)]
struct Profile {
    #[validate(email)]
    email: EmailAddr,
    #[validate(url)]
    site: HttpsUrl,
    #[validate(range(min = 0.0, max = 1.0))]
    trust: f64,
    #[validate(range(min = 0, max = 120))]
    age: u8,
    #[validate(postcode_uk)]
    postcode: String,
}

fn valid_profile() -> Profile {
    Profile {
        email: EmailAddr::parse("bob@example.co.uk").unwrap(),
        site: HttpsUrl::parse("https://example.co.uk").unwrap(),
        trust: 0.99,
        age: 30,
        postcode: "SW1A 1AA".to_string(),
    }
}

#[test]
fn profile_valid_with_newtype_leaves() {
    assert!(valid_profile().validate().is_ok());
}

#[test]
fn profile_postcode_cases() {
    for pc in ["M1 1AE", "B33 8TH", "gir0aa", "DN55 1PT"] {
        let p = Profile {
            postcode: pc.to_string(),
            ..valid_profile()
        };
        assert!(p.validate().is_ok(), "postcode {pc} must pass");
    }
    for pc in ["", "12345", "SW1A 1AAA", "ZZZ 1AA", "K1A 0B1"] {
        let p = Profile {
            postcode: pc.to_string(),
            ..valid_profile()
        };
        assert!(p.validate().is_err(), "postcode {pc} must fail");
    }
}

#[test]
fn profile_range_rejects_out_of_bounds_and_nan() {
    let p = Profile {
        trust: 1.5,
        ..valid_profile()
    };
    let err = p.validate().unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidValue(msg) if msg.contains("trust")),
        "unexpected error: {err}"
    );

    let p = Profile {
        trust: f64::NAN,
        ..valid_profile()
    };
    assert!(p.validate().is_err(), "NaN must fail the range check");

    let p = Profile {
        age: 121,
        ..valid_profile()
    };
    assert!(p.validate().is_err());
}

/// Checks run in declaration order and stop at the first failure.
#[derive(Validated)]
struct Ordered {
    #[validate(email)]
    first: String,
    #[validate(url)]
    second: String,
}

#[test]
fn checks_run_in_field_order() {
    let o = Ordered {
        first: "bad".to_string(),
        second: "also-bad".to_string(),
    };
    let err = o.validate().unwrap_err();
    assert!(
        matches!(&err, ValidError::InvalidValue(msg) if msg.contains("first")),
        "first field must be reported: {err}"
    );
}

/// The generated inherent method and the `Validate` trait impl agree, so
/// generic code over `Validate` works.
#[test]
fn validate_trait_impl_matches_inherent_method() {
    fn assert_valid<T: Validate>(value: &T) -> bool {
        value.validate().is_ok()
    }
    assert!(assert_valid(&valid_contact()));
    assert!(assert_valid(&valid_profile()));
    let bad = Contact {
        email: "bad".to_string(),
        ..valid_contact()
    };
    assert!(!assert_valid(&bad));
}

/// Fields without `#[validate]` are ignored, even when invalid.
#[derive(Validated)]
#[allow(dead_code)]
struct Partial {
    #[validate(length(min = 1))]
    checked: String,
    unchecked: String,
}

#[test]
fn unannotated_fields_are_ignored() {
    let p = Partial {
        checked: "ok".to_string(),
        unchecked: String::new(),
    };
    assert!(p.validate().is_ok());
}

/// Option<T> support for range rules.
#[derive(Validated)]
struct Ranged {
    #[validate(range(min = -1.0, max = 1.0))]
    delta: Option<f64>,
    #[validate(range(min = 0, max = 10))]
    count: Option<u32>,
}

#[test]
fn option_numeric_fields() {
    assert!(Ranged {
        delta: None,
        count: None
    }
    .validate()
    .is_ok());
    assert!(Ranged {
        delta: Some(0.5),
        count: Some(10)
    }
    .validate()
    .is_ok());
    assert!(Ranged {
        delta: Some(1.1),
        count: Some(3)
    }
    .validate()
    .is_err());
    assert!(Ranged {
        delta: Some(0.0),
        count: Some(11)
    }
    .validate()
    .is_err());
}

/// `invalid_field` formats errors uniformly (used by generated code).
#[test]
fn invalid_field_helper_formatting() {
    let err = invalid_field("email", "not a valid email address");
    assert_eq!(
        err.to_string(),
        "invalid value: field `email`: not a valid email address"
    );
}

/// Sanity: the underlying validators used by the derive are wired correctly.
#[test]
fn underlying_validators_reachable() {
    assert!(is_valid_email("a@b.co"));
    assert!(HttpsUrl::parse("https://example.com").is_ok());
}

/// Generics and where-clauses are supported (impl generics plumbing).
#[derive(Validated)]
struct Wrapper<T: Send + Sync> {
    #[validate(length(min = 1))]
    label: String,
    _marker: core::marker::PhantomData<T>,
}

#[test]
fn generic_structs_supported() {
    let w = Wrapper::<u8> {
        label: String::from("hi"),
        _marker: core::marker::PhantomData,
    };
    assert!(w.validate().is_ok());
    let w = Wrapper::<u8> {
        label: String::new(),
        _marker: core::marker::PhantomData,
    };
    assert!(w.validate().is_err());
}
