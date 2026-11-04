+++
id = "jig-scripted-phase"
part = "jig-scripted-rules"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client-domain/src/contract/common.rs",
  "jig/crates/jig-client-domain/src/tests.rs",
  "jig/crates/jig-client-view/src/state/page/conversations.rs",
  "jig/testing/jig-client-scripted-engine/src/queries.rs",
  "jig/testing/jig-client-scripted-engine/src/watches.rs",
  "jig/docs/design/client/domain/testing.status.md",
]
effort = "high"
minutes = 45
+++

# jig D028, phase classification: the contract's `Phase::class`

## Goal

The scripted engine classifies phases itself, three times, and the view holds its own `live`. A
task's class becomes the contract's, `Phase::class`, called by the view and the fake, and the fake's
two phase-count matches become one. The first of four tasks on
`jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`; the item stays open.

Acceptance:

- `Phase::class` exists as sketched, with a unit test naming every phase.
- `rg -n 'PhaseClass::' jig/testing/jig-client-scripted-engine/src/queries.rs` reads only the
  `phase != PhaseClass::Ended` comparison in `selects`; `fn ended` is gone from `watches.rs`; and
  `line` and `fold_tree` share one count function.
- `jig-client-view`'s `live` is `Phase::class`'s, and means what it meant.
- The note's `What a task runs` passes, adding `-p jig-client-domain -p jig-client-view`, with no
  test changed.
- `jig/tests/boundary` needs no change: the fake's dependencies stand.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/jig-scripted.md`, `AGENTS.md`,
  `jig/AGENTS.md`.
- `jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`, whole.
- `jig/docs/design/client/domain/testing.md`, 4 and 5, which this cut extended.
- `jig/crates/jig-domain-core/src/documents/task_query.rs`, `phase`: the rule, to compare by hand.

## Owns and may change

- Owns: the files above. `Phase` is in `contract/common.rs`; `PhaseClass` in `contract/queries.rs`
  is already in scope there through `contract`.
- Status: in `testing.status.md`'s **Not yet**, the entry becomes "4: inbox rows served as seeded:
  the scripted engine's `reads.rs`, `inbox_order`, decides them.", with the date line today's.

## Steps

1. In `contract/common.rs`:

   ```rust
   impl Phase {
       /// The class a task query selects by.
       #[must_use]
       pub const fn class(self) -> PhaseClass {
           match self {
               Self::Waiting => PhaseClass::Waiting,
               Self::Held => PhaseClass::Held,
               Self::Closing => PhaseClass::Closing,
               Self::Ended | Self::Done | Self::Failed | Self::Cancelled => PhaseClass::Ended,
               Self::Idle | Self::Due | Self::Preparing | Self::Claimed | Self::Running | Self::BackingOff
               | Self::Active => PhaseClass::Working,
           }
       }
   }
   ```

2. `conversations.rs`'s `live` matches on `phase.class()`: `Ended` is false, the other four true.
3. In the fake: `selects` calls `task.chip.phase.class()`. `line`'s `ended_at` and `watches.rs`'s
   `subject_end` compare `class()` with `PhaseClass::Ended`; `ended` goes.
4. One `pub(crate) fn count(counts: &mut PhaseCounts, phase: Phase)` in `queries.rs`, from `fold`'s
   match, as it stands: `Ended` and `Cancelled` count `cancelled`. `line` counts a member whose
   `class()` is not `Ended`, so its counts stay as they are, starts from `PhaseCounts::ZERO`, and
   `fold_tree` does too.
5. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

The tests that ran, and whether `class` and the core's `phase` agree on every arm, which you read by hand.
