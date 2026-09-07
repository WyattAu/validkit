# Coverage Notes — validkit

## Measurement methodology: feature-cfg'd fallback validators

```
cargo llvm-cov --summary-only --all-features   # primary gate measurement
cargo test --locked --no-default-features --features serde,std   # fallback-validators suite (CI job)
```

The hand-rolled fallback validators in `src/` are behind
`#[cfg(not(feature = "regex"))]`, `#[cfg(not(feature = "url"))]`, and
`#[cfg(not(feature = "idna"))]` (`idna` is an implicit feature from the optional
`idna` dependency). Consequence for measurement:

- Under `--all-features` (the primary gate command) every fallback module is
  **compiled out**, so it is *excluded from the coverage denominator entirely*
  — the `--all-features` number cannot "see" fallback code at all, covered or
  not. No single-command sweep can attribute coverage to it.
- The fallback code is exercised by the dedicated no-default-features CI job
  (`ci.yml`, "Test (fallback validators, no-default-features + serde,std)")
  running `tests/fallback_paths.rs`, which exists only in that cfg.

**Therefore: feature-cfg'd code needs per-config runs.** Any coverage audit of
this crate must pair the `--all-features` gate number with a green
no-default-features fallback job; a single-sweep number is not evidence about
the fallback validators in either direction. (The fallback job was re-verified
green during the 2026-09-07 gap-closing pass.)

## Known exception: unreachable defensive arms under `--all-features`

All 28 lines uncovered under `--all-features` (96.99% line) are provably
unreachable defensive code, verified by invariant:

| Location | Lines | Why unreachable |
|---|---|---|
| `email.rs` | 95 | `input.find('@')`'s `None` arm — the `at_count != 1` guard above guarantees `Some`. |
| `email.rs` | 146–148 | `domain.len() > 253` — total ≤ 254 and local ≥ 1 cap the domain at 252 chars. |
| `email.rs` | 175–177 | empty domain label — the `contains("..")` / starts/ends-with-`.` guards already reject every way a label could be empty. |
| `email.rs` | 204–206 | `idna::domain_to_ascii` returning `Ok("")` for a non-empty domain. |
| `bucket.rs` | 79 | `u16` parse failure — the `p.len() > 3` + all-digits guards make every part 1–3 digits, always parseable. |
| `url.rs` | 97 | `host_str().is_none()` — `https` is a WHATWG special scheme: `Url::parse` errors (`EmptyHost`) instead of yielding a host-less URL, and the scheme check above rejects everything else. |
| `cron.rs`, `flag_name.rs`, `locale.rs`, `phone.rs` | 94+101, 71+76, 80–82+89–91, 78–80+87–89 | `OnceLock<Regex>` init fallbacks — `Regex::new` on a constant pattern cannot fail, and `get()` after a successful `set()` is always `Some`. |

These arms are kept intentionally: they preserve fail-closed behavior if the
guards above them are ever weakened, and `unwrap`/`expect` are denied by the
workspace clippy lints. Writing tests to "cover" them would require changing
library behavior; deleting them would trade robustness for a percentage.
