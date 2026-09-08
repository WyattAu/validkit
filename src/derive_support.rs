//! Internal support code for the `Validated` derive macro.
//!
//! These items are `#[doc(hidden)]`: they exist so generated code can call
//! them through the `::validkit` path without pulling `alloc`/`std` paths
//! into user crates. They are stable within the 1.x series but are not part
//! of the documented API surface.

extern crate alloc;

use alloc::string::String;

/// Structural UK postcode check used by `#[derive(Validated)]`
/// `#[validate(postcode_uk)]`.
///
/// Accepts the `GIR 0AA` special case; otherwise requires the standard
/// `A(A)D(A) DAA` pattern (1–2 leading letters, then a digit, then an
/// optional alphanumeric in the outward code; `digit + 2 letters` in the
/// inward code). Input is trimmed, uppercased, and whitespace-compacted
/// before checking, mirroring `geo-kit`'s hand-rolled validation path.
///
/// Exposed for the derive macro only; prefer `geo-kit`'s `UkPostcode` when
/// you need the full validator as a type.
#[must_use]
pub fn check_postcode_uk(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    if input.contains('\r') || input.contains('\n') || input.contains('\t') {
        return false;
    }
    let compact: String = input
        .trim()
        .chars()
        .filter(|&c| c != ' ')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if compact.len() <= 3 || !compact.is_char_boundary(compact.len() - 3) {
        return false;
    }
    let (outward, inward) = compact.split_at(compact.len() - 3);
    if outward == "GIR" && inward == "0AA" {
        return true;
    }
    // inward: exactly `digit + 2 uppercase letters` (3 bytes by construction).
    let mut in_ok = false;
    for (i, b) in inward.bytes().enumerate() {
        match i {
            0 => in_ok = b.is_ascii_digit(),
            1 | 2 => in_ok &= b.is_ascii_uppercase(),
            _ => {
                in_ok = false;
                break;
            }
        }
    }
    if !in_ok {
        return false;
    }
    // outward: 1-2 uppercase letters, then a digit, then at most one
    // alphanumeric.
    let mut letters = 0;
    for &b in outward.as_bytes() {
        if b.is_ascii_uppercase() {
            letters += 1;
        } else {
            break;
        }
    }
    if !matches!(letters, 1 | 2) {
        return false;
    }
    let Some(rest) = outward.as_bytes().get(letters..) else {
        return false;
    };
    let Some((&digit, tail)) = rest.split_first() else {
        return false;
    };
    if !digit.is_ascii_digit() {
        return false;
    }
    tail.len() <= 1 && tail.iter().all(|b| b.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_uk_patterns() {
        for s in [
            "SW1A 1AA", "M1 1AE", "B33 8TH", "DN55 1PT", "GIR 0AA", "sw1a1aa",
        ] {
            assert!(check_postcode_uk(s), "must accept {s}");
        }
    }

    #[test]
    fn rejects_uk_patterns() {
        for s in [
            "", "SW1A1A", "12345", "SW1A 1A", "ZZZ 1AA", "ééé", "K1A 0B1", "123 AB",
        ] {
            assert!(!check_postcode_uk(s), "must reject {s}");
        }
    }
}
