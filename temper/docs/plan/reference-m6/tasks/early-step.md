+++
id = "early-step"
part = "early"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge-change/src/**", "temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/generated/**", "temper/crates/temper-protocol-store/golden/**", "temper/crates/temper-protocol-store/src/records/change.rs", "temper/crates/temper-protocol-store/src/tests/**"]
effort = "high"
minutes = 100
+++

# early-step: the change procedure's early start, as its pure step

## Goal

The change procedure's step decides a producer's early start from the facts its top gives
it: when to start early, from which dependencies' heads, once, what a conflict and an early
failure do, and that the next producer of a change that started early merges its landing
tip first (contract C5). It is the pure half: the top fills `Facts.pending` and
`Heard.start_conflict`, and reads `Change.early`, in task `early-route`; until then the top
gives no pending dependency and no conflict, so every change steps as today.

Acceptance:

- C5's `Pending`, `PendingKind`, `StartedFrom` and `Early`, `Facts.pending`, `Change.early`,
  `Heard.start_conflict` and `Decision::CancelProducer`, exported from the crate as C5 has
  them.
- `Change.early` is stored with the change: `temper-protocol-store`'s `record Change` gains
  `early: Early` (an `enum Early { none, started: list 64 StartedFrom, spent: list 64
  StartedFrom }`, `record StartedFrom { task: u64 head: fixed 32 }`), its translation in
  `src/records/change.rs` both ways, its generated code and goldens as the store's tests
  require; a stored change round-trips each arm.
- The step follows C5's rules, each shown by a pure test in the crate's `src/tests.rs`:
  - `a_change_with_a_pending_change_without_a_head_waits_with_no_stall`: `Wait { until:
    None }`, no producer, and stepping again past the stall still waits;
  - `a_change_whose_pending_changes_have_heads_starts_early_from_them`: `Delegate(Produce)`,
    `early` `Started` with the heads in `pending`'s order, `Elsewhere` left out;
  - `a_pending_dependency_kept_by_its_own_dependency_blocks_the_early_start`, and the same
    for an `Other`;
  - `only_elsewhere_dependencies_start_the_producer_with_no_heads`: produces as today, with
    `early` `Started` of no heads (nothing to merge, nothing to mark);
  - `a_conflicting_early_start_cancels_its_producer_and_waits`: `CancelProducer`, `early`
    `Spent`, then no hold when the cancelled delegate ends, and waiting while `pending` is
    not empty;
  - `a_failed_early_producer_holds_nothing_and_the_next_waits_for_the_dependencies`:
    `Spent`, `Producing { requested: false }`, waiting; once `pending` is empty, one
    `Delegate(Produce)`; that producer's failure holds as `Hold::Failed` does today;
  - `a_change_never_starts_early_twice`: with `Spent`, pending changes with heads do not
    start a producer.
- A change made today (`Early::None`, empty `pending`, no conflict) steps as before, save
  that its producing stall counts from the producer's request (`since = now` there): every
  existing test of the crate passes unchanged, except the one Wait deadline in
  `a_change_is_produced_opened_checked_queued_and_landed` that this moves (`at(30)` to
  `at(31)`).
- The forge connector and the root still build and pass their tests, with `pending` empty,
  `early` `Early::None`, `start_conflict` `None` at their literal sites, and the top's arm
  for `CancelProducer` doing what it does for `Decision::None`, its comment saying the top
  sets no `start_conflict` yet.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11 (every bullet), 8.2 step 1 and 8.4 (a producer
  that fails for good; a release at producing).
- `temper/docs/design/domain/forge.md`, 8.4's stalls bullet (what a stall counts).
- `temper/docs/design/protocol/store.md`, section 4 (the store's version and an in-place
  schema change before the first release).
- `temper/docs/plan/reference-m6/notes/common.md`; `notes/contracts.md`, C5.

## Owns and may change

Owns the files in the header.

- Where it goes: `producing` in `src/domain.rs` decides the producer today (`facts.branch`
  absent and not `requested` makes `Delegate::Produce`; a failed delegate holds). Put the
  rules there or in a function it calls; a stall counts from `change.since`, so set it as
  the early wait ends and the producer is asked.
- Reader and neighbour files: every `Change { .. }`, `Facts { .. }` and `Heard { .. }`
  literal and every exhaustive `match` on `Decision` outside this crate
  (`temper-engine-domain-forge/src/**`, `temper-engine-domain/src/route/**`, their tests,
  `temper/tests/engine/forge/**`, `temper/tests/engine/domain/**`), the new field's default
  or the new arm only, as Acceptance says.
- Options: if a stored `Change` is also translated elsewhere (a restore test, a golden of
  the forge's rows), update it for `early` and say so. If the store's tests pin a record's
  largest size, update the pinned figure for the new field and name it.

## Steps

1. The types and the store field.
2. The step's rules, then the pure tests above.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge-change -p temper-protocol-store -p temper-engine-domain-forge
   -p temper-engine-domain --all-targets -- -D warnings`; the focused tests of those four
   packages. Commit, push.

## Report

As the implementer's text says; say where the early rules sit in the step and how the wait
keeps the stall from counting.

## Answers

2026-11-03. Q: may the store schema wrap the early heads in a record, since skein-codegen
rejects a list as an enum variant's payload? A: yes. Use `record EarlyHeads { heads: list
64 StartedFrom }` and `enum Early { none, started: EarlyHeads, spent: EarlyHeads }`, the
schema's usual shape (as `Segments` wraps a branch's segments); `record Change` gains
`early: Early`, store version 11 unchanged, the domain's C5 types unchanged. skein is not
widened. Run the old-forms check from `temper/` (`python3 -I -B scripts/old_forms.py`),
where its `scripts/old-forms.txt` is.

2026-11-03. Q: may the ordinary producer's stall count from its request, moving
`a_change_is_produced_opened_checked_queued_and_landed`'s producing deadline from `at(30)` to
`at(31)`, or must a durable dependency-wait state keep it exact? A: count from the request;
update that one assertion, and no other existing test. `forge.md`, 8.11 says a change at
producing with no producer does not count its stall, so the producing stall rightly starts
when the producer is asked, for every change; an ordinary change is asked its producer in
its first step, so the move is the gap between its making and that step. Add no stored
state beyond C5, and say in the report that the assertion moved and why.
