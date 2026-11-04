# D036: Lint configuration is copied by hand into 48 manifests

- **Kind:** lints
- **Where:** the `[lints]` blocks of skein's test and binary crates; skein's `clippy.toml` copies
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T7) and `reviews/2026-10-31-skein-areas.md` (A0)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 6 and decision D2)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** every skein package takes its lints from one of a few named profiles (step, harness, binary), held by a gate check, and skein's `clippy.toml` files come in as few variants as profiles, each with a true header.
- **Measure:** `rg -l '^\[lints\.(rust|clippy)\]' skein -g Cargo.toml | wc -l`: 48 at base

## What

- 43 skein manifests take `[lints] workspace = true`; 48 write their own
  `[lints.rust]`/`[lints.clippy]` blocks, in 14 variants. The review
  found some lacking `allow_attributes`, `wildcard_enum_match_arm` or
  `unwrap_used`.
- 27 `clippy.toml` files under `skein/` come in 14 variants; the review
  found several carrying another crate's header.
- 638 `#[expect]`s carry over 500 distinct reasons; one ban's reason
  appears in many wordings.

## Why it matters

A gap in one copy goes unseen, and a change of policy is 48 edits.

## What pays it back

Named profiles with a gate check that each package matches one; the
closure-ban lints set once at workspace level; one `testing/clippy.toml`.
Whether test crates keep `wildcard_enum_match_arm` is the user's call.
jig and temper have the same debt
(`jig/docs/development/debt/D019-lint-config-copied.md`,
`temper/docs/development/debt/D011-manual-map-expectations.md`): the
profiles are proposed as one plan across the trees when the first of the
three is paid.
