+++
id = "change-merge-steps"
part = "change-merge"
kind = "feature"
needs = ["change-merge-surface"]
files = ["temper/crates/temper-engine-domain-forge-change/src/**"]
effort = "high"
minutes = 120
+++

## Goal

The change procedure's step decides the merge job (`temper/docs/design/domain/forge.md, 8.4`,
8.2 steps 1 and 6, section 10): a conflict goes to the merge job where `facts.merge_job`, else
to a resolution; under rules asking no update it is resolved too, no longer held; the job's
result moves the change on from a pushed merge, or to a resolution of the same base; a
catch-up's producing is the merge job. Pure step code in the forge-change crate, with its unit
tests; the root makes and reads the job in `change-merge-route`.

Acceptance (each a unit test in the crate):

- **a conflict, rules asking an update:** `First` with `mergeable: Failed`, and `Updating` whose
  effect is `Conflict`, decide `Delegate(Merge { base, from: Some(head) })` and state
  `Resolving { requested: true, base }` where `facts.merge_job`, `resolutions` plus one; without
  `merge_job`, `Delegate(Resolve { base })` as today;
- **a conflict, rules asking none** (`facts.updates == false`): the same two outcomes, no hold;
  a merge whose effect answers `Conflict` (the `landing` step) decides the same at once, from
  `facts.base_tip`, rather than going back to `First`; a merge effect that `Failed` goes back
  to `First` as today;
- **the job's result:** in `Resolving { requested: true, base }` with `facts.delegate ==
  Some(Merge { base, from })` and the delegate `Passed`: `facts.branch` other than `from` goes on
  as a resolution's push does (`pushed`, `last_head`, `clean` emptied, the turn given up);
  `facts.branch == from` decides `Delegate(Resolve { base })`, state `Resolving { requested:
  true, base }`, `resolutions` unchanged; `Failed` holds as a failed resolution does;
- **the bound:** a merge job past `limits.resolutions` holds `Hold::Resolutions` as
  `resolving_due` does; a conflict settled by job then resolution counts one;
- **a catch-up** (`facts.catch_up == Some(commit)`): `Producing { requested: false }` with no
  branch and `contains_catch_up == Passed` ends `NoChange { words }`, the words saying the
  landing branch already contains the commit, naming it in lowercase hexadecimal; `Failed`
  decides `Delegate(Merge { base: commit, from: None })` and `Producing { requested: true }`,
  counting no resolution; `Pending` waits. Its job's passed result with a branch read goes to
  `Opening` as a producer's push does; with no branch, `Delegate(Resolve { base: commit })`, one
  resolution counted; a failed job holds as a failed producer does;
- every other existing test passes, but `a_conflict_under_rules_asking_no_update_holds_the_change`,
  which this task rewrites to the new rule (named
  `a_conflict_under_rules_asking_no_update_is_resolved`).

## Inputs

- `temper/docs/design/domain/forge.md, 8.2` (steps 1 and 6), `8.4` ("The merge job", "A
  resolution", the bounds), section 10 ("A conflict", "A catch-up").
- `temper/docs/plan/reference-m7b/notes/contracts.md`, C3 and C3.1; `notes/common.md`.

## Owns and may change

- `temper/crates/temper-engine-domain-forge-change/src/**`: `boundary.rs` (C3.1's two `Facts`
  fields, documented as C3.1 has them), `domain.rs` (`first`, `updating`, `landing`, `resolving`
  / `delegated`, `producing`, a `conflict_due` beside `resolving_due`), and its tests
  (`src/tests.rs`, `src/tests/turn.rs`: their `facts()` literals gain the fields, `None`).
- Readers: the forge's facts builder (`temper-engine-domain-forge/src/domain.rs`, where `Facts`
  is built) gains `delegate: None, catch_up: None`, literal only: `change-merge-route` fills
  them. Any other `Facts` literal in temper, the same.
- The turn's prepare conflict (`preparing`) keeps going to a resolution: the merge job is the
  queue's (`temper/docs/design/domain/landing.md, 5.3` is unchanged). If sharing `conflict_due`
  would send it to the job, keep `preparing` on `resolving_due`.
- Not this task's: the root, the forge crate's behaviour, the store.

## Steps

1. C3.1's fields; `conflict_due(change, base, from, facts, now, limits)`, the one place that
   picks job or resolution and counts.
2. The steps above, keeping each function's existing outcomes for every other case.
3. Unit tests, one per acceptance bullet, in `src/tests.rs` (or a new `src/tests/merge.rs`
   registered beside `tests/turn.rs`), named for what they show
   (`a_conflict_with_a_merge_job_goes_to_the_job_before_a_resolution`,
   `a_merge_job_that_pushed_moves_the_change_on_from_its_merge`,
   `a_merge_job_that_left_conflicts_makes_a_resolution_of_the_same_base`,
   `a_refused_merge_goes_to_the_merge_job_at_once`,
   `a_catch_up_already_contained_ends_with_no_change`,
   `a_catch_ups_producer_is_the_merge_job`, ...), each citing
   `temper/docs/design/domain/forge.md, 8.4`.
4. `heavy cargo nextest run -p temper-engine-domain-forge-change --lib`, then
   `-p temper-engine-domain-forge --lib` and `-p temper-engine-domain --lib` (readers of the
   step); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy on the three,
   `--all-targets -- -D warnings`.
5. This changes what a conflict under rules asking no update means: run the engine domain's
   world too, `heavy cargo nextest run -p <temper/tests/engine/domain's package> -E
   'test(/light_rules|conflict/)'`. Its `a_conflict_under_rules_asking_neither_says_why_it_holds`
   may now fail, since the change resolves rather than holds: that story is
   `change-merge-route`'s to rewrite; list it under `## Left`, do not change it.
6. Commit and push.

## Report

As the implementer's text says; list each existing test whose expectation changed and why.
