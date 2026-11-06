+++
id = "hold-look"
part = "hold-look"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/render.rs",
  "temper/crates/temper-client-tools/src/jig/render/holds.rs",
  "temper/crates/temper-client-tools/src/root.rs",
  "temper/crates/temper-client-tools/src/change.rs",
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
- `temper/docs/design/client/tools.md`, 4, `jig's part apart from temper's`: the boundary test holds, and this task leaves it as it is.
- `temper/docs/design/domain/worker.md`, 12, `A hold naming its run's last failure`.
- `temper/docs/plan/ctl-words/notes/common.md`.

## Owns and may change

Owns the files in the header. In production, only these change: `recovery` and `way_out`'s signature in `render/holds.rs`, one method of the `Parts` trait in `render.rs`, its one implementation in `root.rs`, and the two `way_out` calls in `change.rs`. `root.rs` is `ctl-waits`' too: this is one method beside `command_prefix`, nothing else there.

Readers: `temper-client-tools/src/change.rs` prints `way_out`. `tests/printed.rs`'s `every_other_holds_offered_commands_parse` finds its commands from `task release `, so a prefix before it is safe, and it covers `Failures`. `temper-client-view` has its own words, and doesn't change.

Departs from the implementer's text: `temper/docs/help/hold.md` is open for the bullet in step 3, and `temper/docs/design/client/tools.status.md` for the one line in step 4.

Options, each pre-approved:

- If a golden prints this `way out:`, regenerate it with its own tool and say so. This cut found none.

## Steps

1. **Reproduce.** In `tests/jig/task_words.rs`, beside `a_held_tasks_text_names_its_reason_and_release_revision`, `a_hold_for_failed_runs_says_where_to_look`: for each of the six `c::HoldFailure` classes, the line above. See it fail.
2. **The words.** They name `temper`, so they come through `Parts`, and nothing under `src/jig/` holds them.
   - In `jig/render.rs`, beside `command_prefix`, add to `Parts` a required `fn failures_recovery(&self) -> &'static [u8];`, with a one-line doc saying it is what a hold for failed runs asks first. Neither its name nor its doc says `temper`.
   - In `root.rs`, `impl jig::render::Parts for Parts` returns `b"the engine's log names each failure (temper help hold), then "`.
   - In `render/holds.rs`, `way_out(task, vocabulary, out)` passes its `vocabulary` to `recovery(hold, vocabulary)`, and `recovery`'s `c::HoldCategory::Failures { .. }` has its own arm, `vocabulary.failures_recovery()`. `text` already has its `vocabulary`.
   - In `change.rs`, `Holds::task`'s two `way_out` calls pass `&crate::root::Parts`, as its `summary` call beside them does.
3. **Help.** In `temper/docs/help/hold.md`, after the bullet `**Two kinds of hold.**`, add:

   ```markdown
   - **Out of tries** names the class of its failures, not their text. The
     engine's log names each, `temper serve: task TASK run RUN attempt ATTEMPT
     failed CLASS: DETAIL`: under `temper box`, in `DATA/box/logs/serve.log`;
     under `temper serve`, in its journal (temper help server).
   ```
4. **Status.** In `temper/docs/design/client/tools.status.md`, delete the bullet beginning `**Not yet:** 6.1: a hold for failed runs`.
5. `cargo fmt`; from `temper/`, `python3 -I -B scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tools --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-tools`; `heavy cargo nextest run -p temper-docs`; `heavy cargo nextest run -p temper --lib -E 'test(help)'`. Commit and push.

## Report

As the implementer's text has it. Say whether `change show` needed anything.

## Answers

2026-11-06. Q: the required `(temper help hold)` in `src/jig/render/holds.rs` fails `tests/boundary.rs`: supply it through `Parts`, or have the boundary and its design permit this string? A: through `Parts`, as `command_prefix` and `release_again_refusal` supply temper's words. `way_out:` prints the same line, `tests/boundary.rs` and `temper/docs/design/client/tools.md`, 4, don't change, and no test's expectation moves. Steps 2 and 5, the header's `files` and `Owns` carry it. If the first run's commit is on your branch, keep its test, help bullet and status line, and move its literal out of `render/holds.rs`. Run the whole of step 5, since the first run's stopped at this failure: 853 tests of `temper-client-tools`, `temper-docs` and `temper --lib -E 'test(help)'` are unobserved.
