+++
id = "change-decides-again"
part = "machine"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge-change/src/domain.rs", "temper/crates/temper-engine-domain-forge-change/src/tests.rs", "temper/crates/temper-engine-domain-forge-change/src/tests/turn.rs"]
effort = "high"
minutes = 60
+++

# A change with no update outstanding decides again

## Goal

A change `Updating` or `Preparing` with no result and none outstanding waits for `stall` and then holds `Stalled`, deciding nothing. It decides again on a fresh read of its head and base, by the table in `notes/contracts.md`, *Deciding again*. Nothing reaches this path yet but a release: `update-let-go` sends the lost updates here. The machine's own tests hold it.

Acceptance, each test seen failing first, for its rule's reason, except the last two, which pin what stays:

1. In `temper/crates/temper-engine-domain-forge-change/src/tests.rs`, for `State::Updating`, with `effect: EffectResult::None`:
   - `an_update_let_go_updates_again_where_its_head_is_behind`: `Decision::Effect(Effect::Update { head: HEAD, base: <facts.base_tip> })`, `updates` one more, `State::Updating` at the current base tip.
   - `an_update_let_go_holds_updates_at_its_bound`: `updates` at `LIMITS.updates`: `Decision::Hold(Hold::Updates)`.
   - `an_update_let_go_goes_on_where_its_head_holds_the_base`: head where it was, `contains_base` `Passed`: `State::Checking { head: HEAD }`, `clean` unchanged, `Decision::None`.
   - `an_update_let_go_takes_a_moved_head_holding_the_base`: as `EffectResult::Made` does, `from` in `clean`, `last_head` the new head.
   - `an_update_let_go_holds_a_moved_head_not_holding_the_base`: `Decision::Hold(Hold::BranchMoved)`.
   - `an_update_let_go_waits_on_what_it_cannot_read`: no branch, or `contains_base` `Unknown` or `Pending`: `Decision::Wait`, and `Hold::Stalled` once `stall` has passed.
   - `a_released_failed_update_decides_again`: `released` of `State::Held { was: Updating, why: Hold::Failed }`, stepped as the top steps it with `EffectResult::None`, head behind: `Effect::Update`, `updates` 1, the release having reset it.
2. In `temper/crates/temper-engine-domain-forge-change/src/tests/turn.rs`, for `State::Preparing`, the same six stories named `a_prepare_let_go_...`, the first giving `Effect::Prepare { head: HEAD, main: BASE }`, `head` holding the base going through the `Made` arm to `State::Gating`, and `a_released_failed_prepare_decides_again`.
3. `a_prepare_let_go_at_a_main_that_moved_asks_its_own_main_again`: `facts.base_tip` not `main`, head behind: `Effect::Prepare { head: HEAD, main }`, at the main it was asked for. Pins the design's words in `landing.md`, 5.2.
4. `an_update_outstanding_still_waits` and `a_prepare_outstanding_still_waits`: `EffectResult::Pending`, head behind: `Decision::Wait`, no effect, `updates` unchanged.
5. `EffectResult::Failed` in both states still holds `Hold::Failed`, and `Conflict` still goes to a resolution: the existing tests stay as they are.
6. The bullet naming `change-decides-again` is gone from `temper/docs/design/domain/forge.status.md`.

## Inputs

- `temper/docs/plan/update-recovers/notes/common.md`, `temper/docs/plan/update-recovers/notes/contracts.md`, *Deciding again*
- `temper/docs/design/domain/forge.md`, 8.5, 10
- `temper/docs/design/domain/landing.md`, 5.2

## Owns and may change

- Owns the crate's `domain.rs`, `updating` and `preparing` and what they call; `ready`, only to move its body from the `updates` check on into `prepare_due`; `step`'s arm that calls `updating`, to pass `ready_since`. Owns its tests.
- **Existing tests where `heard()`'s `EffectResult::None` stood for a write outstanding** now decide again. Give them `EffectResult::Pending`, which is what the top shows: `a_prepare_ask_makes_a_rebase_update`'s last assertion in `tests/turn.rs` is one, and `tests.rs` has more around each `State::Updating`. Change only that field. A test whose meaning changes otherwise goes in the report.
- **Dependents' tests, the same edit and no other:** where a test in `temper-engine-domain-forge/src/tests/**`, `temper-engine-domain/src/route/forge_route/tests.rs` or `temper/tests/engine/{forge,domain}/**` puts a change in `Updating` or `Preparing` by hand with no `pending` and steps it, give the row its `pending`, or its heard `Pending`.

**Options:**

- **A step test table** (`tests.rs`, the all-states loop) just gains these paths: leave it.
- **If `updating` needs `clean`'s bound for the moved-head arm,** it is the `Made` arm's and comes with it: `Hold::Updates` at `clean_heads`. Share the arm, don't copy it.
- **If stepping in the top's loop, `Decision::None` then again, would send an effect twice:** it can't, as `Decision::Effect` returns from that loop and `row.pending` is then set; a test showing otherwise is a `Question:`.

## Steps

1. Write the tests; run `heavy cargo nextest run -p temper-engine-domain-forge-change --lib`; note the failures' reasons.
2. Split `EffectResult::None` from `Pending` in `updating` and `preparing`; factor `prepare_due`; pass `ready_since`. A module comment cites `temper/docs/design/domain/forge.md`, 10.
3. Repair the tests the second bullet of *Owns* names. Break each rule once by hand, for the report.
4. Close the status entry. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-forge-change --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-forge-change -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-forge-world -p temper-engine-domain-world`, one package a command if heavy prefers. Commit, small and saying why; push.

## Report

As your text says, and each existing test you gave `Pending`, by file and name.
