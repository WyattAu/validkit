//! JSON Schema / OpenAPI support for validkit newtypes.
//!
//! Behind the non-default `openapi` feature, every newtype implements
//! [`schemars::JsonSchema`] so schemas generated for handlers using these
//! types carry the *real* validation constraints — `format: email`,
//! `format: uri`, `pattern`, `minLength`, and `maxLength` match what the
//! runtime validators enforce, so generated OpenAPI documents describe
//! the API that actually exists.
//!
//! Constraints that cannot be expressed as a portable ECMA-262 `pattern`
//! (for example `TenantIdSlug`'s ban on `..` and `.lock` suffixes, or
//! `BucketName`'s IP-address check) are documented in each schema's
//! `description`; the runtime validator remains the source of truth.
//!
//! # Example
//!
//! ```rust
//! # #[cfg(feature = "openapi")]
//! # fn main() {
//! use schemars::{schema_for, JsonSchema};
//!
//! let schema = schema_for!(validkit::EmailAddr);
//! let json = serde_json::to_value(&schema).expect("serializable");
//!
//! assert_eq!(json["format"], "email");
//! assert_eq!(json["maxLength"], 254);
//! # }
//! # #[cfg(not(feature = "openapi"))]
//! # fn main() {}
//! ```

#![allow(clippy::doc_markdown)]

use alloc::borrow::Cow;
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};

macro_rules! string_schema {
    ($name:ty, $desc:expr, {$($field:tt)*}) => {
        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "string",
                    "description": $desc,
                    $($field)*
                })
            }
        }
    };
}

string_schema!(
    crate::EmailAddr,
    "A validated email address (RFC 5321 local part, IDNA-normalised domain; \
     exactly one '@', no whitespace or control characters).",
    {
        "format": "email",
        "pattern": "^[^@\\s]+@[^@\\s]+$",
        "maxLength": 254
    }
);

string_schema!(
    crate::HttpsUrl,
    "A validated HTTPS URL (https scheme, host required, no embedded \
     credentials).",
    {
        "format": "uri",
        "pattern": "^https://"
    }
);

string_schema!(
    crate::PhoneE164,
    "A validated E.164 phone number, e.g. '+14155552671'.",
    {
        "pattern": "^\\+[1-9]\\d{1,14}$",
        "minLength": 3,
        "maxLength": 16
    }
);

string_schema!(
    crate::CronExpr,
    "A validated 5-field cron expression; each field matches \
     '[0-9*,/-]+' (e.g. '*/5 * * * *').",
    {
        "pattern": "^[\\d*,/-]+([\\s]+[\\d*,/-]+){4}$",
        "minLength": 9
    }
);

string_schema!(
    crate::TenantIdSlug,
    "A validated tenant ID slug: 3-63 chars of [a-z0-9-], not starting or \
     ending with '-', no '..' and no '.lock' suffix.",
    {
        "pattern": "^[a-z0-9]([a-z0-9-]{1,61}[a-z0-9])?$",
        "minLength": 3,
        "maxLength": 63
    }
);

string_schema!(
    crate::LocaleTag,
    "A validated BCP 47 locale tag, e.g. 'en', 'en-US', 'es-419' \
     (primary subtag 2-3 lowercase letters, max 35 characters).",
    {
        "pattern": "^[a-z]{2,3}(-[A-Za-z0-9]+)*$",
        "maxLength": 35
    }
);

string_schema!(
    crate::FlagName,
    "A validated flag name: snake-case starting with a lowercase letter, \
     only [a-z0-9_].",
    {
        "pattern": "^[a-z][a-z0-9_]*$"
    }
);

string_schema!(
    crate::BucketName,
    "A validated S3 bucket name: 3-63 chars of [a-z0-9.-], not starting or \
     ending with '-' or '.', no adjacent dots, never formatted as an IP \
     address.",
    {
        "pattern": "^[a-z0-9]([a-z0-9.-]{1,61}[a-z0-9])?$",
        "minLength": 3,
        "maxLength": 63
    }
);

string_schema!(
    crate::ObjectKey,
    "A validated S3 object key: non-empty, at most 1024 bytes, no leading \
     '/', no '..' traversal segments, no CR/LF or null bytes.",
    {
        "pattern": "^[^/].*$",
        "minLength": 1,
        "maxLength": 1024
    }
);
