# D009: The oauth crates hold credentials as raw bytes, not `Secret`

- **Kind:** primitives
- **Where:** `skein-oauth`, `skein-oauth-accounts`, `skein-fake-oauth`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the credentials row; A10) and `reviews/2026-10-31-skein-areas.md` (A10)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** every credential in the three crates (tokens, codes, verifiers, records) is a `skein_lib::Secret`, and none of their types carries a `missing_debug_implementations` expect.
- **Measure:** `rg -c 'missing_debug_implementations' skein/crates/skein-oauth skein/crates/skein-oauth-accounts skein/crates/skein-fake-oauth | awk -F: '{s+=$2} END{print s}'`: 25 at base

## What

The three crates keep credentials in plain byte boxes, so their types
cannot derive `Debug` safely and opt out of the lint instead: 25 of
those expects, in `skein-oauth/src/machine.rs` (7),
`skein-fake-oauth/src/lib.rs` (6), `skein-oauth-accounts/src/boundary.rs`
(3) and others. The issuer, by contrast, holds all its credentials as
`Secret` and derives `Debug` everywhere.

## Why it matters

lib's `Secret` exists for this: it redacts in `Debug` and keeps the
bytes out of traces. Without it each type needs a hand guarantee that
no trace prints a token.

## What pays it back

Move the fields to `Secret`, derive `Debug`, and delete the expects.
The issuer is the model.
