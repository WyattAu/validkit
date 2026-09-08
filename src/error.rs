//! Error types for `validkit`.

extern crate alloc;

use alloc::string::String;
use thiserror::Error;

/// Validation error covering all newtypes in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidError {
    /// Invalid email address.
    #[error("invalid email: {0}")]
    InvalidEmail(String),

    /// Invalid URL.
    #[error("invalid url: {0}")]
    InvalidUrl(String),

    /// Invalid cron expression.
    #[error("invalid cron: {0}")]
    InvalidCron(String),

    /// Invalid tenant id slug.
    #[error("invalid tenant id: {0}")]
    InvalidTenantId(String),

    /// Invalid locale tag.
    #[error("invalid locale: {0}")]
    InvalidLocale(String),

    /// Invalid flag name.
    #[error("invalid flag name")]
    InvalidFlagName,

    /// Invalid bucket name.
    #[error("invalid bucket name: {0}")]
    InvalidBucketName(String),

    /// Invalid object key.
    #[error("invalid object key: {0}")]
    InvalidObjectKey(String),

    /// Invalid phone number.
    #[error("invalid phone: {0}")]
    InvalidPhone(String),

    /// Generic invalid value.
    #[error("invalid value: {0}")]
    InvalidValue(String),
}

/// Builds a [`ValidError::InvalidValue`] naming a field and a short reason.
///
/// Used by the [`Validated`](https://docs.rs/validkit/latest/validkit/derive.Validated)
/// derive for checks that have no dedicated `ValidError` variant (length,
/// range, and postcode checks); also handy for hand-written validators that
/// want uniform error formatting.
#[must_use]
pub fn invalid_field(field: &str, reason: &str) -> ValidError {
    ValidError::InvalidValue(alloc::format!("field `{field}`: {reason}"))
}
