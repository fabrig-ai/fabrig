+++
id = "hold-look"
part = "hold-look"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/render/holds.rs",
  "temper/crates/temper-client-tools/src/tests/jig/task_words.rs",
  "temper/docs/help/hold.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "medium"
minutes = 20
+++

# A hold for failed runs says where to look

## Goal

A task held after failed runs says `out of tries` and its class, and the reason is in the engine's log, which `task show` doesn't mention. Say where to look, in `way out:` and in `temper help hold`. This stores nothing: keeping the failure's text with the hold is the plan's question `hold-failure-text`, and this task doesn't wait for it.

Acceptance:

- For every `HoldFailure` class, `task show`'s `way out:` is: `the engine's log names each failure (temper help hold), then task release 41 --hold 9; or cancel it: task cancel 41 REASON`.
- `change show`, which prints the same `way_out`, says the same.
- Every other hold's `way out:` is unchanged.
- `temper help hold` names the log under `temper box` and under `temper serve`.
- No stored field and no wire field changes.

## Inputs

- `temper/docs/design/client/tools.md`, 6.1, the bullet beginning `A hold for failed runs`.
- `temper/docs/design/domain/worker.md`, 12, `A hold naming its run's last failure`.
- `temper/docs/plan/ctl-words/notes/common.md`.

## Owns and may change

Owns the files in the header. Only `recovery` in `render/holds.rs` changes in production.

Readers: `temper-client-tools/src/change.rs` prints `way_out`. `tests/printed.rs`'s `every_other_holds_offered_commands_parse` finds its commands from `task release `, so a prefix before it is safe, and it covers `Failures`. `temper-client-view` has its own words, and doesn't change.

Departs from the implementer's text: `temper/docs/help/hold.md` is open for the bullet in step 3, and `temper/docs/design/client/tools.status.md` for the one line in step 4.

Options, each pre-approved:

- If a golden prints this `way out:`, regenerate it with its own tool and say so. This cut found none.

## Steps

1. **Reproduce.** In `tests/jig/task_words.rs`, beside `a_held_tasks_text_names_its_reason_and_release_revision`, `a_hold_for_failed_runs_says_where_to_look`: for each of the six `c::HoldFailure` classes, the line above. See it fail.
2. **The words.** In `recovery`, give `c::HoldCategory::Failures { .. }` its own arm: `b"the engine's log names each failure (temper help hold), then "`.
3. **Help.** In `temper/docs/help/hold.md`, after the bullet `**Two kinds of hold.**`, add:

   ```markdown
   - **Out of tries** names the class of its failures, not their text. The
     engine's log names each, `temper serve: task TASK run RUN attempt ATTEMPT
     failed CLASS: DETAIL`: under `temper box`, in `DATA/box/logs/serve.log`;
     under `temper serve`, in its journal (temper help server).
   ```
4. **Status.** In `temper/docs/design/client/tools.status.md`, delete the bullet beginning `**Not yet:** 6.1: a hold for failed runs`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tools --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-tools`; `heavy cargo nextest run -p temper-docs`; `heavy cargo nextest run -p temper --lib -E 'test(help)'`. Commit and push.

## Report

As the implementer's text has it. Say whether `change show` needed anything.
