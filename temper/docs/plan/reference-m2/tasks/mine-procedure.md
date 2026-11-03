+++
id = "mine-procedure"
part = "mine"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge-change/src/**"]
effort = "high"
minutes = 75
+++

# mine-procedure: a change on its requester's held branch takes a forward move as its new head

## Goal

The change procedure (`temper-engine-domain-forge-change`) holds a change `BranchMoved` when
its branch's fresh head differs from the head it is at (`opening`, `checking`, `gating`,
`queued`, `first`, `ready`). For a change on its requester's held branch (the goal's
landing, L, on g), a move with no drift and no writer in the slot is a move by its
requester's other delegates: its new head, checked again, never a hold
(`temper/docs/design/domain/forge.md`, 8.5). This task gives the procedure that rule behind a
new fact the forge will supply (contract C3); the forge's side is task `mine-forge`.

Acceptance:
- With `Facts.requester_branch` true, no drift, the writer slot free, and a fresh head other
  than the change's head, a change at opening, checking, gating, queued, first or ready
  takes the new head exactly as a repair's pushed head is taken (`delegated`'s
  `Status::Passed` path: clean heads cleared, `pushed(..)`'s state, `last_head`, `since`),
  its repairs, resolutions and updates counted on, and asks nothing else; it is not held.
- With `requester_branch` false, every phase holds `BranchMoved` as today.
- With `requester_branch` true and drift set, or the writer slot taken, the change does as
  today (holds for the drift; waits for the writer).
- A held change on its requester's branch (repairs spent, for instance) stays held while the
  branch moves; released, it takes the head as it then is and checks it (the existing
  release path; a test shows it).
- Landing (its merge in flight) is unchanged: the merge is never sent at a head the change
  has not checked.
- Every test of `temper-engine-domain-forge-change` and `temper-engine-domain-forge` passes.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 ("A change on its requester's held branch") and
  8.5 (its paragraph "On its requester's held branch").
- `temper/docs/plan/reference-m2/notes/contracts.md`, C3.
- `temper/docs/plan/reference-m2/notes/common.md`.

## Owns and may change

Owns `temper-engine-domain-forge-change/src/**`.

- `boundary.rs`: `Facts.requester_branch: bool`, after `writer_taken`, its doc comment C3's.
- `domain.rs`: the rule, in one place: in `advance`, after the writer-slot wait and the
  drift hold and before the phase's own step, or in a helper each listed phase calls before
  its `facts.branch != head` test: your choice; the report says which. Reuse the code of
  `delegated`'s `Status::Passed` path (factor it into a helper both call) rather than
  copying it.
- **Reader files, literals only:** every `Facts { .. }` literal outside this crate gets
  `requester_branch: false` and nothing else: `temper-engine-domain-forge/src/domain.rs`
  (`change_facts`), `temper-engine-domain-forge-landing/src/boundary.rs` and
  `src/tests/step.rs`, `temper/tests/client/tests/suite/change_panel.rs`, and any other the
  compiler names (`heavy cargo check -p <pkg> --tests` for each). The forge's real value is
  `mine-forge`'s.

## Steps

1. Tests first, in this crate's `src/tests.rs` beside
   `drift_writer_slots_and_stalls_do_not_write_over_a_branch` (use its fixtures):
   - `a_move_of_a_requesters_held_branch_is_its_new_head_in_each_phase`: for each listed
     phase, a change at head A, facts with `requester_branch: true` and branch B: the step's
     change has `last_head == Some(B)`, its state `pushed`'s for B, no hold, its repairs
     count unchanged;
   - `a_move_of_its_own_branch_still_holds_it`: the same with `requester_branch: false`:
     `BranchMoved`;
   - `a_requesters_branch_with_drift_or_a_writer_is_not_taken`: drift set holds for the
     drift; `writer_taken` waits;
   - `a_held_change_on_its_requesters_branch_takes_the_head_at_release`: held (repairs
     spent), the branch moves (no change while held), then `released`: the new head,
     checked.
2. The code, as above.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge-change --lib -E
   'test(/requesters|own_branch_still/)'` (check the count); then `heavy cargo nextest run
   -p temper-engine-domain-forge-change -p temper-engine-domain-forge-landing -p
   temper-engine-domain-forge`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge-change -p temper-engine-domain-forge-landing -p
   temper-engine-domain-forge --all-targets -- -D warnings`; `heavy cargo check -p
   temper-client-world --tests` (or the package `temper/tests/client` names). Commit
   ("forge-change: a move of a requester's held branch is the change's new head") and push.

## Report

As the implementer's text says, naming where the rule sits.
