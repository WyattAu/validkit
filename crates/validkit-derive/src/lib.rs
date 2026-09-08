//! `#[derive(Validated)]` for `validkit` — declarative field validation.
//!
//! The derive generates an inherent `validate(&self) -> Result<(), ::validkit::ValidError>`
//! method (and an impl of `::validkit::Validate`) that checks every field
//! annotated with `#[validate(...)]`. Fields without the attribute are not
//! checked.
//!
//! # Supported rules
//!
//! | Attribute | Field types | Check |
//! |-----------|-------------|-------|
//! | `#[validate(email)]` | `String`, `Option<String>`, `EmailAddr`, `Option<EmailAddr>` | `validkit::is_valid_email` / `Validate::validate` |
//! | `#[validate(url)]` | `String`, `Option<String>`, `HttpsUrl`, `Option<HttpsUrl>` | `validkit::HttpsUrl::parse` / `Validate::validate` |
//! | `#[validate(postcode_uk)]` | `String`, `Option<String>` | structural UK postcode check (GIR 0AA included) |
//! | `#[validate(length(min = .., max = ..))]` | `String`, `Option<String>` | byte length within inclusive bounds |
//! | `#[validate(range(min = .., max = ..))]` | numeric primitives and `Option` thereof | inclusive range check |
//!
//! Both bounds in `length`/`range` are optional; `None` values on
//! `Option<_>` fields are considered valid. Checks run in field declaration
//! order and validation stops at the first failure.
//!
//! Generated code references the `validkit` crate by the exact name
//! `::validkit`, so the dependency must not be renamed.
//!
//! # Example
//!
//! ```
//! use validkit::{Validated, Validate};
//!
//! #[derive(Validated)]
//! struct Contact {
//!     #[validate(email)]
//!     email: String,
//!     #[validate(url)]
//!     homepage: Option<String>,
//!     #[validate(length(min = 1, max = 100))]
//!     name: String,
//! }
//!
//! let ok = Contact {
//!     email: "alice@example.com".to_string(),
//!     homepage: Some("https://example.com".to_string()),
//!     name: "Alice".to_string(),
//! };
//! assert!(ok.validate().is_ok());
//!
//! let bad = Contact {
//!     email: "not-an-email".to_string(),
//!     homepage: None,
//!     name: String::new(),
//! };
//! assert!(bad.validate().is_err());
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, Field, Fields, Meta, Token, Type};

/// One `#[validate(...)]` rule attached to a field.
enum Rule {
    Email,
    Url,
    PostcodeUk,
    Length {
        min: Option<Expr>,
        max: Option<Expr>,
    },
    Range {
        min: Option<Expr>,
        max: Option<Expr>,
    },
}

impl Rule {
    fn kind(&self) -> &'static str {
        match self {
            Rule::Email => "email",
            Rule::Url => "url",
            Rule::PostcodeUk => "postcode_uk",
            Rule::Length { .. } => "length",
            Rule::Range { .. } => "range",
        }
    }
}

/// Failure carrier so `?` composes inside the derive implementation.
struct DeriveError(syn::Error);

impl From<syn::Error> for DeriveError {
    fn from(e: syn::Error) -> Self {
        DeriveError(e)
    }
}

fn err<T>(span: proc_macro2::Span, message: &str) -> Result<T, DeriveError> {
    Err(syn::Error::new(span, format!("validkit: {message}")).into())
}

/// Early-return statement referencing `validkit::invalid_field` for a field.
fn bail(field_name: &str, reason: &str) -> proc_macro2::TokenStream {
    quote! {
        return ::core::result::Result::Err(
            ::validkit::invalid_field(#field_name, #reason),
        );
    }
}

/// Parse the rules contained in one `#[validate(...)]` attribute.
fn parse_validate_attr(attr: &syn::Attribute) -> Result<Vec<Rule>, DeriveError> {
    let metas: Punctuated<Meta, Token![,]> =
        attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut rules = Vec::new();
    for meta in &metas {
        let rule = match meta {
            Meta::Path(path) => {
                let name = path
                    .get_ident()
                    .map_or_else(|| String::from("?"), |i| i.to_string());
                match name.as_str() {
                    "email" => Rule::Email,
                    "url" => Rule::Url,
                    "postcode_uk" => Rule::PostcodeUk,
                    other => {
                        return err(
                            path.span(),
                            &format!(
                                "unknown validation rule `{other}` (supported: email, url, \
                                 postcode_uk, length, range)"
                            ),
                        );
                    }
                }
            }
            Meta::List(list) => {
                let name = list
                    .path
                    .get_ident()
                    .map_or_else(|| String::from("?"), |i| i.to_string());
                if name != "length" && name != "range" {
                    return err(
                        list.path.span(),
                        &format!("unknown validation rule `{name}` (it takes arguments)"),
                    );
                }
                let inner: Punctuated<Meta, Token![,]> =
                    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                let (min, max) = parse_bounds(&inner, &name)?;
                if name == "length" {
                    Rule::Length { min, max }
                } else {
                    Rule::Range { min, max }
                }
            }
            Meta::NameValue(nv) => {
                return err(
                    nv.path.span(),
                    "expected `rule` or `rule(min = .., max = ..)`",
                );
            }
        };
        rules.push(rule);
    }
    if rules.is_empty() {
        return err(attr.span(), "empty #[validate(...)] attribute");
    }
    Ok(rules)
}

/// Parse the `min = .., max = ..` argument list of `length`/`range`.
fn parse_bounds(
    inner: &Punctuated<Meta, Token![,]>,
    rule_name: &str,
) -> Result<(Option<Expr>, Option<Expr>), DeriveError> {
    let mut min = None;
    let mut max = None;
    for item in inner {
        let Meta::NameValue(nv) = item else {
            return err(
                item.span(),
                &format!("{rule_name} accepts `min = <expr>` and `max = <expr>`"),
            );
        };
        let key = nv
            .path
            .get_ident()
            .map_or_else(|| String::from("?"), |i| i.to_string());
        match key.as_str() {
            "min" if min.is_none() => min = Some(nv.value.clone()),
            "max" if max.is_none() => max = Some(nv.value.clone()),
            "min" | "max" => {
                return err(nv.path.span(), &format!("duplicate `{key}` bound"));
            }
            other => {
                return err(
                    nv.path.span(),
                    &format!("unknown bound `{other}` (expected `min` or `max`)"),
                );
            }
        }
    }
    Ok((min, max))
}

/// Rules for one field, rejecting duplicate rule kinds.
fn field_rules(field: &Field) -> Result<Vec<Rule>, DeriveError> {
    let mut rules: Vec<Rule> = Vec::new();
    for attr in &field.attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }
        for rule in parse_validate_attr(attr)? {
            if rules.iter().any(|r| r.kind() == rule.kind()) {
                return err(
                    field.span(),
                    &format!("duplicate `#[validate({})]` on field", rule.kind()),
                );
            }
            rules.push(rule);
        }
    }
    Ok(rules)
}

/// Final path segment identifier of a type, e.g. `String` for
/// `::std::string::String` or `Option<String>`.
fn type_ident(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        if tp.qself.is_none() {
            if let Some(seg) = tp.path.segments.last() {
                return Some(seg.ident.to_string());
            }
        }
    }
    None
}

/// Inner type of `Option<T>` (any qualified spelling), if applicable.
fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else { return None };
    if tp.qself.is_some() {
        return None;
    }
    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else {
        return None;
    };
    Some(inner)
}

const NUMERIC_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64",
];

/// Codegen for one rule on one field: statement(s) that early-return on
/// failure. `field_name` is the literal field name used in error messages.
fn field_check(
    field: &Field,
    rule: &Rule,
    field_name: &str,
) -> Result<proc_macro2::TokenStream, DeriveError> {
    let Some(ident) = field.ident.as_ref() else {
        return err(field.span(), "internal error: field without identifier");
    };
    let option_field = option_inner(&field.ty).is_some();
    let inner_ty = option_inner(&field.ty).unwrap_or(&field.ty);
    let inner_ident = type_ident(inner_ty);

    match rule {
        Rule::Email => {
            if inner_ident.as_deref() == Some("EmailAddr") {
                if option_field {
                    Ok(quote! {
                        if let ::core::option::Option::Some(__value) = &self.#ident {
                            ::validkit::Validate::validate(__value)?;
                        }
                    })
                } else {
                    Ok(quote! {
                        ::validkit::Validate::validate(&self.#ident)?;
                    })
                }
            } else if inner_ident.as_deref() == Some("String") {
                let bail = bail(field_name, "not a valid email address");
                if option_field {
                    Ok(quote! {
                        if let ::core::option::Option::Some(__value) = &self.#ident {
                            if !::validkit::is_valid_email(__value) {
                                #bail
                            }
                        }
                    })
                } else {
                    Ok(quote! {
                        if !::validkit::is_valid_email(&self.#ident) {
                            #bail
                        }
                    })
                }
            } else {
                err(
                    field.ty.span(),
                    "`#[validate(email)]` requires `String`, `Option<String>`, `EmailAddr`, or \
                     `Option<EmailAddr>`",
                )
            }
        }
        Rule::Url => {
            if inner_ident.as_deref() == Some("HttpsUrl") {
                if option_field {
                    Ok(quote! {
                        if let ::core::option::Option::Some(__value) = &self.#ident {
                            ::validkit::Validate::validate(__value)?;
                        }
                    })
                } else {
                    Ok(quote! {
                        ::validkit::Validate::validate(&self.#ident)?;
                    })
                }
            } else if inner_ident.as_deref() == Some("String") {
                let bail = bail(field_name, "not a valid HTTPS URL");
                if option_field {
                    Ok(quote! {
                        if let ::core::option::Option::Some(__value) = &self.#ident {
                            if ::validkit::HttpsUrl::parse(__value).is_err() {
                                #bail
                            }
                        }
                    })
                } else {
                    Ok(quote! {
                        if ::validkit::HttpsUrl::parse(&self.#ident).is_err() {
                            #bail
                        }
                    })
                }
            } else {
                err(
                    field.ty.span(),
                    "`#[validate(url)]` requires `String`, `Option<String>`, `HttpsUrl`, or \
                     `Option<HttpsUrl>`",
                )
            }
        }
        Rule::PostcodeUk => {
            if inner_ident.as_deref() != Some("String") {
                return err(
                    field.ty.span(),
                    "`#[validate(postcode_uk)]` requires `String` or `Option<String>`",
                );
            }
            let bail = bail(field_name, "not a valid UK postcode");
            if option_field {
                Ok(quote! {
                    if let ::core::option::Option::Some(__value) = &self.#ident {
                        if !::validkit::check_postcode_uk(__value) {
                            #bail
                        }
                    }
                })
            } else {
                Ok(quote! {
                    if !::validkit::check_postcode_uk(&self.#ident) {
                        #bail
                    }
                })
            }
        }
        Rule::Length { min, max } => {
            if inner_ident.as_deref() != Some("String") {
                return err(
                    field.ty.span(),
                    "`#[validate(length(..))]` requires `String` or `Option<String>`",
                );
            }
            let ((Some(_), Some(_)) | (Some(_), None) | (None, Some(_))) = (min, max) else {
                return err(
                    field.span(),
                    "`length` needs at least one bound (`min` or `max`)",
                );
            };
            let reason = match (min, max) {
                (Some(_), Some(_)) => "length out of range",
                (Some(_), None) => "shorter than the minimum length",
                _ => "longer than the maximum length",
            };
            let bail = bail(field_name, reason);
            // Two-sided bounds become `RangeInclusive::contains`; a single
            // bound stays a plain comparison.
            let conds = match (min, max) {
                (Some(min), Some(max)) => quote! { (#min..=#max).contains(&__len) },
                (Some(min), None) => quote! { __len >= (#min) },
                (None, Some(max)) => quote! { __len <= (#max) },
                (None, None) => unreachable!("handled above"),
            };
            if option_field {
                Ok(quote! {
                    if let ::core::option::Option::Some(__value) = &self.#ident {
                        let __len = __value.len();
                        if !(#conds) {
                            #bail
                        }
                    }
                })
            } else {
                Ok(quote! {
                    let __len = self.#ident.len();
                    if !(#conds) {
                        #bail
                    }
                })
            }
        }
        Rule::Range { min, max } => {
            let is_numeric = inner_ident
                .as_deref()
                .is_some_and(|id| NUMERIC_TYPES.contains(&id));
            if !is_numeric {
                return err(
                    field.ty.span(),
                    "`#[validate(range(..))]` requires a numeric primitive field",
                );
            }
            let ((Some(_), Some(_)) | (Some(_), None) | (None, Some(_))) = (min, max) else {
                return err(
                    field.span(),
                    "`range` needs at least one bound (`min` or `max`)",
                );
            };
            let reason = match (min, max) {
                (Some(_), Some(_)) => "value out of range",
                (Some(_), None) => "below the minimum",
                _ => "above the maximum",
            };
            let bail = bail(field_name, reason);
            let conds = match (min, max) {
                (Some(min), Some(max)) => quote! { (#min..=#max).contains(&__value) },
                (Some(min), None) => quote! { __value >= (#min) },
                (None, Some(max)) => quote! { __value <= (#max) },
                (None, None) => unreachable!("handled above"),
            };
            if option_field {
                // Numeric options are Copy, so bind by value for plain
                // comparisons.
                Ok(quote! {
                    if let ::core::option::Option::Some(__value) = self.#ident {
                        if !(#conds) {
                            #bail
                        }
                    }
                })
            } else {
                Ok(quote! {
                    let __value = self.#ident;
                    if !(#conds) {
                        #bail
                    }
                })
            }
        }
    }
}

/// Derive `Validated` — see the crate documentation for the attribute
/// grammar.
#[proc_macro_derive(Validated, attributes(validate))]
pub fn derive_validated(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(stream) => stream.into(),
        Err(DeriveError(e)) => e.to_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
    let Data::Struct(data) = &input.data else {
        return err(
            input.ident.span(),
            "`Validated` can only be derived for structs",
        );
    };
    let Fields::Named(named) = &data.fields else {
        return err(
            input.ident.span(),
            "`Validated` can only be derived for structs with named fields",
        );
    };

    let mut checks = proc_macro2::TokenStream::new();
    for field in &named.named {
        let Some(field_name) = field.ident.as_ref().map(syn::Ident::to_string) else {
            return err(field.span(), "internal error: field without identifier");
        };
        for rule in field_rules(field)? {
            checks.extend(field_check(field, &rule, &field_name)?);
        }
    }

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            /// Validates all fields annotated with `#[validate(...)]`.
            ///
            /// # Errors
            ///
            /// Returns the first validation failure, in field declaration order.
            pub fn validate(&self) -> ::core::result::Result<(), ::validkit::ValidError> {
                #checks
                ::core::result::Result::Ok(())
            }
        }

        impl #impl_generics ::validkit::Validate for #ident #ty_generics #where_clause {
            fn validate(&self) -> ::core::result::Result<(), ::validkit::ValidError> {
                #ident::validate(self)
            }
        }
    })
}
