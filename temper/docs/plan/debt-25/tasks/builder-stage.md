+++
id = "builder-stage"
part = "spot-review"
kind = "feature"
needs = ["builder-shape"]
files = ["hearth/crates/hearth-protocol-builder/src/git.rs", "hearth/crates/hearth-protocol-builder/src/events.rs", "hearth/crates/hearth-protocol-builder/src/lib.rs", "hearth/docs/development/debt/**"]
effort = "high"
minutes = 40
+++

# hearth-protocol-builder: a git child's intakes live in its live stage

## Goal

Finding 2's `git.rs` part in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `Stage { Live, Closing }` is `closing: bool` under another name, and it passes hearth's paid item on lifecycle flags. `output_state` and `error_state` have no meaning once closing, so they belong in `Live`. Either they move there, or the flag is recorded as open.

Acceptance:

- `Stage` is `Live { output: Intake, error: Intake }` and `Closing`; `Child`'s `output_state` and `error_state` go. Every read of them is in a `Live` arm, or there is no such read in `Closing`.
- `lib.rs`'s drain arm (`(child.failed || ready) && child.output_state != Intake::Ended && ...`) is guarded on the stage first, as its own commit, with its tests run before the move. It's the one reader with no stage check, so that commit is where a change would show.
- No test's visible events or assertions change.
- Production's net lines don't grow by more than about 10.

**If the guard or the move changes a test's visible events,** or the move needs `hearth/crates/hearth-service-builder/src/lib.rs`: don't ask, and don't rewrite the test. Revert the move, and add `hearth/docs/development/debt/D014-git-child-stage-flag.md` in the item form of `hearth/docs/development/debt/README.md`, saying that `Stage` is a renamed flag, which test or file stopped the move, and what pays it. Raise the README's highest ID to D014. Then report `Done:`, with that as the outcome.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 2
- `hearth/docs/design/domain/render.md`, 2.2
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header. `hearth-protocol-builder/src/lib.rs` is yours: the file the request says fabrig/hearth holds is `hearth-service-builder/src/lib.rs`, which names none of this (`Child` is `pub(crate)` here), and it stays off.
- **`Intake` stays in `git.rs`;** `refused` while closing does nothing, since closing has no intakes.
- **Tests** may change in spelling only. None reads these fields at this plan's base.

## Steps

1. The guard, on its own: `heavy cargo nextest run -p hearth-protocol-builder --lib`, `-p hearth-render-world`, `-p hearth-builder-world`. Commit.
2. The move, through `git.rs`, `events.rs` and `lib.rs`. `resume`'s git part and `ready`'s git part read one `let Stage::Live { .. }` or a `match`, not a stage check beside a field.
3. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`; the tests above, then `-p hearth-service-builder` and `-p hearth-test-render`. Commit, run the citations tool and push.

## Report

The common note's items; whether the move landed or the item was added, and why; whether any read in `Closing` turned up.
