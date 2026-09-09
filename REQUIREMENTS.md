# Requirements — validkit

Numbered, testable requirements. Every requirement maps to at least one named
test; every security-relevant test cites at least one requirement. Doc
comments on the implementing public item carry `REQ-VK-NNN` tags.

## Functional

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-VK-001 | `EmailAddr::parse` accepts RFC 5321-valid addresses and normalises the domain (IDNA, lowercased) | MUST |
| REQ-VK-002 | `EmailAddr` rejects total length > 254, local part > 64 octets, oversized domain labels, edge dots, and malformed domain shapes | MUST |
| REQ-VK-003 | `HttpsUrl::parse` accepts only `https` scheme URLs with a host, and exposes the parsed `url::Url` via `as_url` | MUST |
| REQ-VK-004 | `HttpsUrl` rejects userinfo credentials (`user:pass@host`) and non-https schemes | MUST |
| REQ-VK-005 | `PhoneE164::parse` accepts `^\+[1-9]\d{1,14}$`, including min (12 → `+` + 1 digit + …) and max (15 digits) boundary lengths | MUST |
| REQ-VK-006 | `PhoneE164` rejects missing `+`, letter digits, embedded spaces, and empty/whitespace-only input | MUST |
| REQ-VK-007 | `CronExpr::parse` accepts common 5-field forms (`*`, lists, ranges, steps) | MUST |
| REQ-VK-008 | `CronExpr` rejects wrong field counts, empty fields, fields starting with `,` or `/`, punctuation-only fields, and consecutive/leading/trailing punctuation | MUST |
| REQ-VK-009 | `TenantIdSlug::parse` enforces 3–63 chars, `[a-z0-9-]` charset, and rejects uppercase and edge characters | MUST |
| REQ-VK-010 | `LocaleTag::parse` accepts BCP 47 compound tags and rejects bad primary/subtag shapes, empty input, and total length > 35 | MUST |
| REQ-VK-011 | `FlagName::parse` accepts `^[a-z][a-z0-9_]*$` and rejects hyphens, empty, CRLF, and bad characters | MUST |
| REQ-VK-012 | `BucketName::parse` enforces S3 rules: 3–63 chars, lowercase/digits/hyphen/dot, hyphen/dot edge rules, and rejects uppercase | MUST |
| REQ-VK-013 | `ObjectKey::parse` enforces ≤ 1024 bytes, no leading `/`, no `..` traversal, no null bytes, and rejects empty/CRLF input | MUST |
| REQ-VK-014 | Free functions `is_valid_email` / `is_valid_cron` / `is_valid_tenant_id` / `is_valid_bucket_name` / `is_valid_object_key` / `is_valid_flag_name` / `is_valid_locale` agree exactly with the corresponding `parse` | MUST |
| REQ-VK-015 | `TryFrom<String>` and `FromStr` validate identically to `parse` for every newtype | MUST |
| REQ-VK-016 | `Display` output re-parses to an equal value (parse → display → parse is idempotent) | MUST |
| REQ-VK-017 | `as_str` / `Deref<Target=str>` / `AsRef<str>` accessors return exactly the stored, validated value | MUST |
| REQ-VK-018 | `new` (unchecked constructor) and `into_inner` round-trip the inner value without alteration | SHOULD |
| REQ-VK-019 | Serde serialization is transparent (string form) and deserialization round-trips for all 9 newtypes | MUST |
| REQ-VK-020 | The inherent `Validate::validate` and the `#[derive(Validated)]`-generated method agree on accept/reject for the same struct value | MUST |
| REQ-VK-021 | `#[derive(Validated)]` checks annotated fields in declaration order and reports the failing field name | MUST |
| REQ-VK-022 | `#[derive(Validated)]` supports `Option<T>` fields (`None` skips validation), numeric ranges (incl. NaN rejection), UK postcodes, newtype leaves, generic structs, and ignores unannotated fields | MUST |
| REQ-VK-023 | `invalid_field` helper produces a stable, field-attributed error message | SHOULD |
| REQ-VK-024 | `check_postcode_uk` accepts standard UK formats and rejects malformed ones | SHOULD |

## Security

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-VK-100 | No parser panics on arbitrary/malformed input — every invalid input yields `Err(ValidError)`, never a panic (fuzz + property-backed) | MUST |
| REQ-VK-101 | CR and LF bytes are rejected by every string-validated type with a specific, attributable error message (header/CRLF injection guard) | MUST |
| REQ-VK-102 | `ObjectKey` rejects `..` path traversal segments and null bytes | MUST |
| REQ-VK-103 | `HttpsUrl` rejects embedded credentials so secrets cannot be smuggled into stored URLs | MUST |
| REQ-VK-104 | Serde deserialization runs full validation — invalid values cannot enter through JSON regardless of construction path | MUST |
| REQ-VK-105 | `BucketName` rejects dotted-quad IP-address-like names (AWS rule) while allowing dotted numbers that are not IP-like | MUST |
| REQ-VK-106 | Email IDNA normalisation is budget-bounded: punycode expansion that would exceed the 254-octet limit is rejected, not truncated | MUST |
| REQ-VK-107 | `TenantIdSlug` rejects `..` and CRLF so tenant strings cannot traverse paths or forge headers | MUST |

## Robustness

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-VK-200 | Property-based round-trip: for every generated-valid input, `parse` → `Display` → `parse` succeeds and preserves the value | MUST |
| REQ-VK-201 | Error messages for rejection classes are deterministic and specific (e.g. CR/LF guard messages are distinguishable from generic charset errors) | SHOULD |
| REQ-VK-202 | Accessor contracts hold uniformly across all 9 newtypes (single generic harness) | MUST |
| REQ-VK-203 | Fallback (no-`url`/no-`idna` feature) validators enforce the same core contracts as the full implementations | SHOULD |
| REQ-VK-204 | Crate compiles `no_std` + `alloc` (minus `url`-dependent type) with `#![forbid(unsafe_code)]` | MUST |

## Traceability Matrix

| Requirement | Test (fn, file) | Property class |
|-------------|-----------------|----------------|
| REQ-VK-001 | `email_idna_normalises_unicode_domain` (`tests/validation_paths.rs`), `email_domain_lowercased` (`tests/integration.rs`) | unit |
| REQ-VK-002 | `email_rejects_total_length_over_254`, `email_rejects_local_part_over_64_octets` (`tests/mutation_boundaries.rs`), `email_rejects_domain_shapes` (`tests/validation_paths.rs`) | unit/mutation |
| REQ-VK-003 | `https_url_valid` (`tests/integration.rs`), `url_as_url_exposes_parsed_url` (`tests/validation_paths.rs`) | unit |
| REQ-VK-004 | `url_rejects_userinfo_credentials` (`tests/mutation_boundaries.rs`), `url_rejects_crlf_and_non_https` (`tests/validation_paths.rs`) | unit |
| REQ-VK-005 | `phone_valid` (`tests/integration.rs`), `phone_boundary_lengths` (`tests/validation_paths.rs`) | unit |
| REQ-VK-006 | `phone_rejects_missing_plus`, `phone_rejects_letter_digits`, `phone_rejects_empty_and_whitespace` (`tests/validation_paths.rs`) | unit |
| REQ-VK-007 | `cron_accepts_common_forms` (`tests/validation_paths.rs`), `cron_valid` (`tests/integration.rs`) | unit |
| REQ-VK-008 | `cron_rejects_crlf_and_bad_field_counts`, `cron_rejects_punctuation_only_field`, `cron_rejects_consecutive_punctuation` (`tests/validation_paths.rs`), `cron_rejects_field_starting_with_comma` (`tests/mutation_kill.rs`) | unit |
| REQ-VK-009 | `tenant_enforces_length_bounds` (`tests/validation_paths.rs`), `tenant_rejects_uppercase` (`tests/proptest.rs`) | unit/property |
| REQ-VK-010 | `locale_allows_valid_compound_tags`, `locale_rejects_bad_primary_and_subtags`, `locale_rejects_total_length_over_35` (`tests/validation_paths.rs`, `tests/mutation_boundaries.rs`) | unit |
| REQ-VK-011 | `flag_valid` (`tests/integration.rs`), `flag_rejects_empty_crlf_and_bad_chars` (`tests/validation_paths.rs`) | unit |
| REQ-VK-012 | `bucket_enforces_length_bounds`, `bucket_rejects_crlf_dots_and_hyphen_edges` (`tests/validation_paths.rs`), `bucket_length_boundary_63_ok_64_err` (`tests/mutation_kill.rs`) | unit/mutation |
| REQ-VK-013 | `object_key_rejects_empty_leading_slash_crlf`, `object_key_enforces_1024_byte_limit` (`tests/validation_paths.rs`) | unit |
| REQ-VK-014 | `is_valid_tenant_id_matches_parse`, `is_valid_cron_matches_parse`, `is_valid_bucket_name_matches_parse`, `is_valid_object_key_matches_parse` (`tests/mutation_boundaries.rs`), `is_valid_flag_name_accepts_valid_input` (`tests/mutation_kill.rs`) | unit |
| REQ-VK-015 | `try_from_impls_validate`, `from_str_impls_match_parse` (`tests/validation_paths.rs`), `email_try_from_and_from_str` (`tests/integration.rs`) | unit |
| REQ-VK-016 | `display_parse_idempotent` (`tests/integration.rs`), `display_and_deref_across_types` (`tests/validation_paths.rs`) | unit |
| REQ-VK-017 | `email_addr_accessors_return_stored_value` et al. (`tests/accessor_contracts.rs`) | unit |
| REQ-VK-018 | `email_new_and_into_inner_roundtrip`, `tenant_new_and_into_inner_roundtrip` (`tests/validation_paths.rs`) | unit |
| REQ-VK-019 | `serde_transparent_roundtrips`, `serde_rejects_invalid_values_for_all_types` (`tests/validation_paths.rs`), `serde_roundtrip_preserves_*` (`tests/accessor_contracts.rs`) | unit |
| REQ-VK-020 | `validate_trait_impl_matches_inherent_method` (`tests/derive.rs`), `email_validate_trait_revalidates_inner_value` (`tests/validation_paths.rs`) | unit |
| REQ-VK-021 | `checks_run_in_field_order`, `contact_invalid_email_reports_field` (`tests/derive.rs`) | unit |
| REQ-VK-022 | `option_numeric_fields`, `profile_range_rejects_out_of_bounds_and_nan`, `profile_valid_with_newtype_leaves`, `generic_structs_supported`, `unannotated_fields_are_ignored` (`tests/derive.rs`) | unit |
| REQ-VK-023 | `invalid_field_helper_formatting` (`tests/derive.rs`) | unit |
| REQ-VK-024 | `profile_postcode_cases`, `valid_profile` (`tests/derive.rs`) | unit |
| REQ-VK-100 | `tests/proptest.rs` (all `*_roundtrip` properties), `tests/fuzz/` targets, `smoke::reexports_work` (`src/lib.rs`) | fuzz/property |
| REQ-VK-101 | `email_rejects_lone_cr_with_specific_message`, `url_rejects_cr_lf_even_though_url_crate_strips_them`, `tenant_rejects_cr_lf_with_specific_message`, `locale_rejects_lone_cr_with_specific_message`, `phone_rejects_lone_cr_with_specific_message`, `bucket_rejects_cr_lf_with_specific_message` (`tests/mutation_boundaries.rs`) | unit/mutation |
| REQ-VK-102 | `object_key_rejects_traversal_and_null_byte` (`tests/validation_paths.rs`) | unit |
| REQ-VK-103 | `url_rejects_userinfo_credentials` (`tests/mutation_boundaries.rs`), `https_fallback_requires_scheme_host_no_credentials` (`tests/fallback_paths.rs`) | unit |
| REQ-VK-104 | `serde_rejects_invalid_values_for_all_types`, `email_serde_rejects_invalid` (`tests/validation_paths.rs`) | unit |
| REQ-VK-105 | `bucket_rejects_dotted_quad_ip_addresses`, `bucket_allows_dotted_numbers_that_are_not_ips`, `bucket_zero_padded_long_part_is_not_ip_like`, `bucket_ip_like_rules_match_aws` (`tests/mutation_boundaries.rs`, `tests/mutation_kill.rs`, `tests/validation_paths.rs`) | unit/mutation |
| REQ-VK-106 | `email_rejects_when_punycode_expansion_blows_length_budget`, `email_idna_rejects_malformed_punycode` (`tests/validation_paths.rs`) | unit |
| REQ-VK-107 | `tenant_rejects_crlf_and_edge_chars` (`tests/validation_paths.rs`), `tenant_rejects_cr_lf_with_specific_message` (`tests/mutation_boundaries.rs`) | unit |
| REQ-VK-200 | `email_roundtrip`, `cron_roundtrip`, `flag_roundtrip`, `bucket_roundtrip`, `locale_roundtrip_via_strategy` (`tests/proptest.rs`) | property |
| REQ-VK-201 | `email_cr_lf_message_comes_from_the_cr_lf_guard`, `locale_cr_lf_message_comes_from_the_cr_lf_guard` (`tests/mutation_kill.rs`) | mutation |
| REQ-VK-202 | generic harness in `tests/accessor_contracts.rs` (`roundtrip<T>`, `as_ref_str<T>`) | unit |
| REQ-VK-203 | `phone_fallback_matches_e164_spec`, `cron_fallback_field_rules`, `locale_fallback_primary_and_subtag_rules`, `flag_name_fallback_charclass`, `https_fallback_string_backed_accessors` (`tests/fallback_paths.rs`) | unit |
| REQ-VK-204 | `#![forbid(unsafe_code)]` + `no_std` cfg in `src/lib.rs`; `smoke::reexports_work` (`src/lib.rs`) | build/unit |
