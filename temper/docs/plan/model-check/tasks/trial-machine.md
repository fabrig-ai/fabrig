+++
id = "trial-machine"
part = "trial-machine"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-trials/**",
]
effort = "high"
minutes = 75
+++

# The trials' child, a step of its own

## Goal

`temper-engine-domain-trials`, the state of each account's trial, as a pure step: the root's part (`trial-root`) routes
to it and carries out what it asks. Nothing uses it yet.

Acceptance: the crate holds `notes/contracts.md`'s surface, and each row of `temper/docs/design/domain/trial.md`, 3.2,
is a test in `src/tests.rs`, every one failing before its code.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `temper/docs/plan/model-check/notes/contracts.md`, *The child*.
- `temper/docs/design/domain/trial.md`, 3.
- `temper/crates/temper-engine-domain-calendar/`: the shape to follow, a small step with `Limits`, an `Event`, a
  `Request` and a `step`.
- `skein/docs/foundation/programming-model.md`, 10: the strict subset.

## Owns and may change

Owns the crate. May also change:

- `Cargo.toml` (the root): its `members` and `[workspace.dependencies]` lines, one each, sorted as they stand;
  `Cargo.lock` by what cargo writes.
- The crate's `[lints]` and `clippy.toml` by `python3 -I -B tools/lints/lints.py --set step temper/crates/temper-engine-domain-trials`.
- `.config/` and gate lists a new crate must be named in, one line each, where a check says so. Say which.
- `temper/docs/design/domain/trial.status.md`: from **Not yet**, remove "no child", and only that.

Not `temper-engine-domain`: `trial-root` adds the dependency.

Options, each pre-approved:

- **A full table:** at `Try` with no room, the oldest ended row goes; with every row running, the `Try` is dropped with no
  request. Test both.
- **`rows` returning an iterator trips a lint:** return a `List` or a slice of what it needs, and say so.

## Steps

```rust
//! A model's trials, a row per account (temper/docs/design/domain/trial.md).
#![no_std]
#![forbid(unsafe_code)]
```

The surface is the note's, whole. `seconds` is `env.wall` less `began`, rounded up, at least 1. `step` only goes from
`Starts` to `Answers`, and `took` from none to some. `next_due` is the earliest running row's `began + WAIT`. `Due` fails
every running row at or past its minute, with `Why::Waited`, whatever its step.

`Event::AccountGone { account }` takes the rows whose `account` or `took` is that account. Each that runs with a task
asks `Cancel`. Each changes `Changed { person }` once.

`TRIALS_SUBJECT` is the note's constant.

Tests, one per row of 3.2, in `src/tests.rs`, with names that say the row:

- `a_first_try_makes_a_task`, `a_second_try_of_an_account_asks_nothing`, `again_on_an_ended_row_replaces_it`,
  `again_while_running_asks_nothing`.
- `made_sets_the_task`, `refused_fails_it`.
- `observed_moves_only_forward`.
- `a_turn_answers_and_cancels`, `seconds_round_up_and_are_at_least_one`, `finished_answers`.
- `a_failed_run_fails_it_and_cancels`, `a_minute_fails_it_and_cancels`, `next_due_is_the_earliest_runner`.
- `a_removed_account_cancels_what_it_started_and_what_took_it`.
- `an_event_for_no_row_is_nothing`.
- `each_person_reads_their_own_rows`.
- `every_change_asks_changed_once`.

The checks: fmt, `old_forms.py` for `temper`, `heavy cargo clippy -p temper-engine-domain-trials --all-targets -- -D warnings`,
`heavy cargo nextest run -p temper-engine-domain-trials`. Citations. Push.

## Report

As the implementer's text asks: each test's bite, the gate lists you touched.
