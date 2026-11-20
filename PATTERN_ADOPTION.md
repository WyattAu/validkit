# PATTERN_ADOPTION: validkit

Relationship to the WyattAu pattern canon. Statuses follow the closed
vocabulary: adopted, partial, planned, refused, n/a. Evidence paths are
relative to this repo.

| Pattern | Status | Evidence |
|---|---|---|
| T1 | adopted | Parse-once validated newtypes: src/lib.rs:9-10 (crate statement: parse-once, use-everywhere), src/email.rs:38 and src/phone.rs:27 (parse constructors that reject invalid input), src/lib.rs:39-41 (serde deserialization runs full validation, so invalid values cannot enter through JSON). The invariant lives in the type; callers never re-validate |
