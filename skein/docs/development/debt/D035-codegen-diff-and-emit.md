# D035: The schema diff is written twice, and codegen edits its own output as text

- **Kind:** duplication
- **Where:** `skein-codec/src/patch/plan.rs`, `skein/testing/skein-codegen/src/diff`, `skein/testing/skein-codegen/src/emit_functions.rs`, `skein/testing/skein-codegen/src/emit_limits.rs`, `skein/testing/skein-codegen/src/emit_bounds.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (A8; phase 3 item 14) and `reviews/2026-10-31-skein-areas.md` (A8, F4, F6, F7 and F17)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** skein-codegen's `diff` spells the change list that `skein_codec::patch::plan` computes, with no matching or comparison of its own (`skein/docs/design/codec.md`, 10.1); the generator builds its output as structure and emits it, with no edit of emitted text; `snake`, `pascal` and `rust_type` are defined once.

## What

- `skein-codec/src/patch/plan.rs` (1,736 lines) matches and compares
  two schemas for the patch machine; `skein-codegen/src/diff/`
  (`matching.rs`, `compare.rs`, `walk.rs`, 538 lines) does it again for
  the CLI.
- `snake` and `pascal` are defined in `emit_limits.rs`, in
  `patch/module.rs` and in `skein/tests/codec/tests/support/generic.rs`;
  `rust_type` in `emit_types.rs` and `emit_bounds.rs`.
- After emitting, the generator splits its own Rust text again:
  `split_bound_functions` and `split_record_functions`
  (`emit_functions.rs`) parse the emitted functions by `split('(')` and
  rewrite them with `replace`; each `emit_*` module opens with a
  module-wide expect.

## Why it matters

Two definitions of what a compatible change is can disagree, and the
text surgery is what blocks jig's request for generated "mirrors".

## What pays it back

Expose plan's matching and change list and have codegen's `diff` only
print and propose renames; one copy of the name helpers; emit statements
as structure. jig's mirrors request depends on the last.
