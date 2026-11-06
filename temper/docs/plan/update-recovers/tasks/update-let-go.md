+++
id = "update-let-go"
part = "machine"
kind = "feature"
needs = ["change-decides-again"]
files = ["temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/tests/change.rs"]
effort = "high"
minutes = 45
+++

# The top lets go of a lost update, and a release decides again

## Goal

An update settled `Failed` with `Timeout`, `Unavailable` or `Stale` says nothing of the change, yet holds it `Failed`. The top reports no result and keeps the account, so the change decides again (`change-decides-again`). And a release of a failed update keeps `row.effect`, so it holds again at once: it drops the result.

Acceptance, in `temper/crates/temper-engine-domain-forge/src/tests/change.rs`, each test seen failing first but the two that pin what stays:

1. `an_update_lost_or_overtaken_is_let_go_with_its_account`: for a change `Updating` with a `Write::Update { style: Merge }` entry and one `Preparing` with `Rebase`, each of `Failed(Timeout)`, `Failed(Unavailable)`, `Failed(Stale)`: `row.effect` `EffectResult::None`, `row.pending` `None`, `row.effect_account` the entry's, its words starting `update pull request: `.
2. `a_let_go_update_is_decided_again`: after that outcome, with fresh reads behind the base, the row steps to a new `Effect::Update` under a new entry, `updates` one more. Check how `change_outcome`'s caller wakes the step, and that this path takes it.
   `a_let_go_update_restored_is_decided_again`: the same with the row saved at the let-go and restored (`Event::Restore`, as `temper/tests/engine/forge/tests/suite/green.rs` does) before the step: a row `Updating` with neither `pending` nor a result restores without complaint, and decides again.
3. `an_update_refused_for_good_still_fails`, pinning: `Failed(Protected)`, `Forbidden`, `Closed`, `Refused`: `EffectResult::Failed`; `Failed(Conflict)`: `EffectResult::Conflict`.
4. `only_an_update_is_let_go`, pinning: a merge's `Failed(Stale)` and `Failed(Timeout)`, and an open's `Failed(Timeout)`: `EffectResult::Failed`, the hold's account as today.
5. `a_release_of_a_failed_update_drops_its_result`: a change held `Hold::Failed` from `Updating`, then from `Preparing`, `row.effect` `Failed`: `Event::ReleaseChange` leaves `row.effect` `None`, and the next step decides again, not `Hold::Failed` again. Held from another state, or with `Conflict`, `row.effect` stays.
6. `a_stored_held_update_keeps_its_hold_and_its_words`: `Outcome::Held` on an update entry, as a store may hold from before: `EffectResult::Failed`, words `update pull request: uncertain: it may or may not have been made; ...`. Released, it decides again. This is the plan's item 2, nothing to clear.
7. The bullet naming `update-let-go` is gone from `temper/docs/design/domain/forge.status.md`.

## Inputs

- `temper/docs/plan/update-recovers/notes/common.md`, `temper/docs/plan/update-recovers/notes/contracts.md`, *Let go, in the top*
- `temper/docs/design/domain/forge.md`, 10
- `temper/crates/temper-engine-domain-forge/src/effect_words.rs`, which holds the words and doesn't change

## Owns and may change

- In `temper-engine-domain-forge/src/domain.rs`, `change_outcome` and `release_change`, and no other function: `landing-fixes-2/saved-close` writes elsewhere in this file. In `tests/change.rs`, your tests at the end of the file.
- `row.effect = match outcome` needs the write: `settled_account` reads `d.entries` in that same function, so the entry is there. Every `match` lists every variant, as the strict subset asks.

**Options:**

- **If `pull_closed`, `Made::Updated` or the wake after the match reads `row.effect`,** keep their meaning for every outcome but those let go, and say so.
- **If a test in `temper-engine-domain-forge`, or in `temper/tests/engine/{forge,domain}`, asserted `Hold::Failed` after an update's `Stale`, `Timeout` or `Unavailable`,** change it to what the table says and name it in the report. None is known; `simulation.rs`'s `Hold::Failed` assertions are a delegate's.
- **If a story needs `update-class`** (a lost update reaching `Failed(Timeout)` by the client's own road): it isn't in your `needs`. Feed the outcome directly, as `tests/change.rs` does.

## Steps

1. Write the tests; run `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/update|release/)'`; note the failures' reasons.
2. Change `change_outcome` and `release_change`, citing `temper/docs/design/domain/forge.md`, 10.
3. Break each rule once by hand, for the report. Close the status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-forge --all-targets -- -D warnings`. **A change of meaning runs its readers' tests,** as `row.effect` now holds `None` after a failure: `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-forge-world -p temper-engine-domain-world`, one command at a time. Commit, small and saying why; push.

## Report

As your text says, and how the step is woken after a let-go.
