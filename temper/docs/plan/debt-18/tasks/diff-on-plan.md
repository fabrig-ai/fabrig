+++
id = "diff-on-plan"
part = "codegen-once"
kind = "feature"
needs = []
files = ["skein/crates/skein-codec/src/patch.rs", "skein/crates/skein-codec/src/patch/plan.rs", "skein/crates/skein-codec/src/patch/plan/tests.rs", "skein/testing/skein-codegen/src/diff.rs", "skein/testing/skein-codegen/src/diff/**", "skein/docs/design/codec.status.md"]
effort = "high"
minutes = 120
+++

# skein: codegen's diff spells the plan's change list

## Goal

`skein/docs/development/debt/D035-codegen-diff-and-emit.md`'s first clause. `skein_codec::patch::plan` and
`skein_codegen::diff` each match and compare two schemas: two definitions of a compatible change, which can disagree. `plan`
exposes its matching and change list (`skein/docs/design/codec.md`, 11.6), and `diff` keeps only what's its own: spelling, placing
rename lines, and proposals.

Acceptance:

- `skein/testing/skein-codegen/src/diff/matching.rs` and `compare.rs` are gone, and nothing in `diff` matches or compares but through
  `skein_codec::patch`.
- The `diff` tests and `diff.txt` files pass unchanged, whole text included: `tests/suite/diff.rs`, `skein/tests/codec/tests/suite/{diff,fingerprint,patch}.rs`,
  `skein/tests/codec/diff/*/diff.txt`, the `patch_check` and `patch_run` suites, and the patch stories. `Op`'s `Display` writes stored
  `patches/NNNN.patch` lines: a byte moved there is a stored format, so stop and ask.
- `plan_bound` reads as it does: `Change` keeps its size and `changes` its capacity. `plan/tests.rs` passes unchanged, its error spans
  and precedence included.
- The `Not yet` entry in `skein/docs/design/codec.status.md` is gone, and it reads "No gap." again.
- The branch's net lines fall by at least 120 (`diff/` from 538 to about 285; `skein-codec` adds at most 60 of production).

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `skein/docs/development/debt/D035-codegen-diff-and-emit.md`
- `skein/docs/design/codec.md`, 10, 10.1, 10.2, 10.3 and 11.6
- `skein/docs/foundation/programming-model.md`, 10: `skein-codec` is under the strict subset, so no lifetime parameter on a type.

## Owns and may change

- **Owns:** the files in `files`. `plan` stays a private module: `patch.rs` re-exports the 11.6 names.
- **`Op` doesn't change.** `Change` lists no rename: the plan's rename lines are its input, not a change it must see covered, and a
  new `Op` variant would widen a public enum, add an arm no parser reaches, and grow `changes`' capacity. `diff` places renames itself.
- **The rename checks only check.** `declaration_rename` and `part_rename` return the pair or a `RenameRefused`; the caller
  pairs. So the plan's wrapper keeps its `Twice`, then its same-name `Mismatch`, then the pair, with the spans it has.
- **One disagreement is real, and stays:** `rename D D` and `rename D.a a`. The plan refuses them with `Mismatch` (pinned in `plan/tests.rs`).
  `diff` accepts them, pairing with no line, and refuses `[rename D D, rename D E]` at index 1 (`codec.md`, 10.1 doesn't list same-name
  as a refusal). Keep both, and if no test pins `diff`'s side, add a row and break it by hand.
- **The driver keeps its answer.** `diff`'s rename driver finds every error and returns the least index, not the first it meets:
  `[Part invalid, Declaration invalid]` returns 0.
- **`changes` is 98 lines:** a further arm needs a helper, for the length lint.
- **If you find a second disagreement,** a concrete input on which they answer differently: stop and ask, with that input.

## Steps

1. `skein_codec::patch`, as 11.6 says: `Matching` and `Comparison` are `pub`, `Change` gains `pub` on its fields, `RenameRefused` splits
   out of the plan's two rename functions, `Matching::new` is `empty_matching`. A test only where a public path has none; the
   plan's own tests cover what it does.

2. `diff.rs` and `diff/walk.rs`, ordinary Rust:

   - The rename driver, over `declaration_rename` and `part_rename`, then `pair_declarations`, `pair_parts`.
   - Spell each `Change` as today's `diff::Op`, in the list's order, with its `Kind`: `written.is_some()` is Written; adding or
     dropping a declaration is NoneNeeded; adding or dropping a field, `versioned`, `wrap` and `order` are Derived; adding a variant and
     widening follow `implicit`.
   - Merge renames as 10.2's Order says: a matched declaration's rename before its changes, and a part's rename before that part's
     change. An owner lookup from a `Target` to its declaration is yours; don't add a field to `Change`.
   - Keep `spell`, `Fill`, `version()`, `written()`, the family check, and proposals (`same_part`, `same_layout`) over `Matching::compare`
     and `Matching::payload`.

3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-codec -p skein-codegen --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-codec -p skein-codegen -p skein-codec-tests -p skein-codec-patches -p smith-codecs-world -p
   smith-transcript-patches -p jig-test-patches -p jig-test-patches-jig -p jig-test-patches-smith`; `git status --porcelain` shows only your edits;
   the citations check. Commit and push.

## Report

The common note's items; each public addition; any test you added and what you broke. Don't delete the item: `codegen-close` does.
