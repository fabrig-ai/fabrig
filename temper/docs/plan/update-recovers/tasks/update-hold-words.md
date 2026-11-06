+++
id = "update-hold-words"
part = "machine"
kind = "feature"
needs = ["update-class", "update-let-go"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/tests/engine/domain/tests/suite/forge_routes/update_lost.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "high"
minutes = 60
+++

# A person told why, and only then

## Goal

When a change's `updates` are spent on lost writes, `Hold::Updates` names no cause. It says the last write's account. And the engine's whole world shows a lost update holding no one, through the jig core and the root: a task held only at the bound.

Acceptance, in a new `temper/tests/engine/domain/tests/suite/forge_routes/update_lost.rs`, on `update_first.rs`'s fixtures, with `world.lose_writes`:

1. `a_lost_update_is_made_again_and_holds_no_one`: `lose_writes = 1`: the change reaches `Checking` at the updated head, `updates` 1, and the task is never held. Expected to pass once `update-class` is on the trunk: bite it by putting kind 11's class back.
2. `an_update_lost_for_good_holds_at_its_count_told_why`: every write lost: `State::Held { why: Hold::Updates, .. }`, writes equal to `updates * write_attempts`, and the task held with `tasks::Hold::Procedure`, its source's words holding `update pull request: forge timed out`. Fails first: the hold names no account.
3. `a_released_update_holds_no_longer`: then `lose_writes = 0` and a release: the update is made and the change lands, as `update_first.rs`'s `assert_landed`.
4. `an_update_whose_base_went_on_while_lost_takes_the_new_tip`: `lose_writes = 1`, the base moved before the check of its second send: `updates` 2, the second entry's `Condition::Update` at the new tip, no hold, landed holding both.
5. `an_updates_hold_with_no_lost_write_says_as_it_did`: `updates` spent on updates that were made: no account in the hold, as today.
6. `hold_change_decision` in `temper/crates/temper-engine-domain/src/route/forge_route.rs` gives `Hold::Updates` with a row's `effect_account` a `StepEnd::HoldFrom`, `tasks::Hold::Procedure`, with `holds::source(domain, Some(account.entry), account.words.clone())`.
7. The bullet naming `update-hold-words` is gone from `temper/docs/design/domain/forge.status.md`.

## Inputs

- `temper/docs/plan/update-recovers/notes/common.md`, `temper/docs/plan/update-recovers/notes/contracts.md`, *The hold's words*
- `temper/docs/design/domain/forge.md`, 10
- `temper/tests/engine/domain/tests/suite/forge_routes/update_first.rs`, `temper/tests/engine/domain/tests/suite/forge_routes/flights.rs`

## Owns and may change

- In `forge_route.rs`, `hold_change_decision` only: `landing-fixes-2/saved-close` and `repair-cause` write elsewhere in this file. In `forge_routes.rs`, the `mod update_lost;` line.
- `update_first.rs`: make `goal_world`, `finish`, `status`, `move_base` and `assert_landed` `pub(super)` if you use them; no other edit there.

**Options:**

- **`why` is shadowed by the category in `hold_change_decision`:** test the change's `Hold` before the shadowing, in a local, as the code already does for its other arms.
- **If a world's `STEPS` is too few to pass the lifetimes,** use `update_first.rs`'s loop shape with a bound of your own in the new file, and report its wall time. No fixture's `lifetime` or `write_attempts` moves. If the file's time is over its budget, take the first of the brief's questions: does it belong here; can it be faster.
- **If `lose_writes` can't lose only the update** (another write comes first), set it just before the update's send, as `flights.rs` does.

## Steps

1. Write the stories; run them: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/update_lost/)'`. Check the binary's name, and the count. Note each failure's reason.
2. Change `hold_change_decision`. Break each rule once by hand, for the report. Close the status entry.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain` and `-p temper-engine-domain-world`, one command at a time. Commit, small and saying why; push.

## Report

As your text says, and the stories' wall time.
