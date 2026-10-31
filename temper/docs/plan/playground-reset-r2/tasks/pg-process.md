+++
id = "pg-process"
part = "playground"
kind = "feature"
needs = ["pg-gone", "pg-script", "rm-process"]
files = [
  "temper/crates/temper/tests/process.rs",
  "temper/crates/temper/tests/support/playground.rs",
  "temper/crates/temper/tests/qa_all/qa_first_run.rs",
  "temper/crates/temper/tests/support/restart.rs",
]
effort = "high"
minutes = 90
+++

## Goal

The process tier walks a first run into the playground and out by its
reset (draft 8): under `temper box` on `temper init`'s document, `temper
ctl playground start` answers running, a chat in the playground's
project runs on the scripted model and delegates into
`playground/widget`; `playground end` returns once there is no
playground; the store then reads as empty (no forge, no account of the
scripted provider, no playground project, no playground), and
`playground start` is offered again and makes a forge with the next
number, whose scripted chat still delegates (`pg-script`). And a box
killed during a reset ends it whole at the next start; a box restarted
with the playground running resets it (draft 9's first default). The
process and QA tests that walked the demo's end are rewritten to the
reset.

Acceptance (process tier, `process-temper`; QA tier, `qa-temper`):

- `a_first_run_goes_into_the_playground_and_out_by_its_reset`
  (replacing `a_second_forge_ends_the_playground_for_good_closing_its_chat_and_account`):
  as the goal says; after the reset, `forge list --json` is empty,
  `account list --json` holds no account of the scripted provider,
  the project list holds no project named as
  `application.playground.project.name`, the box's
  `box/groups/N` and the fake forge's directory are gone; a second
  `playground start` answers running on forge `N + 1` and its chat's
  delegation lands.
- `a_box_killed_during_a_reset_ends_it_at_the_next_start`: the box
  killed right after `playground end` prints resetting; the next start's
  `playground end` (or a read) finds no playground and the empty store
  as above.
- `a_box_restarted_in_the_playground_resets_it`: a second run of the
  box after a running playground: the playground reads none and the
  store is empty, without any request (replacing the "second run finds
  the running demo" test near `process.rs:2100`).
- The other process and QA tests that named the demo's end
  (`support/playground.rs`'s `retired_reload`, `support/restart.rs`
  near lines 223 and 329, `qa_all/qa_first_run.rs` near lines 119 and
  212, `qa_all/qa_delegation.rs`'s playground steps): rewritten so that
  a real forge's set up starts from the reset (the banner's way), or
  deleted where they only proved the end on another forge's connect;
  each named in the report.
- Each test's wall time within its tier's budget, the figures in the
  report; over budget is a question.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2, 3 and 5.
- `temper/docs/design/draft/playground-reset.md`, 6.1, 6.2, 6.3, 8 and 9.
- `skein/docs/foundation/testing-strategy.md`, section 1 and 8.1.
- `AGENTS.md`, sections 5 and 6.
- The support code: `temper/crates/temper/tests/support/boxed.rs`,
  `support/ctl.rs`, `support/playground.rs`, rm-process's
  `support/start_over.rs` (its kill-at-a-line step), and
  `tests/process.rs`'s playground tests (`a_first_run_answers_on_the_demo`,
  `playground_end_steps`, `assert_playground_chat_and_account_open`).
- The ctl: cl-tools' `playground start`, `playground end`, `forge list`,
  `account list`, the project list.

## Owns and may change

Owns the files in the header. May change any other file under
`temper/crates/temper/tests/` (process, QA and suite tests and their
support) that the demo's end or the playground's numbering breaks, and
`temper/tests/real-loop/src/green_light.rs` where it uses playground
fixtures the reset changes. `notes/scope.md` applies. Production code:
none; a fault found is reported with its log as `Question:` first.

## Steps

1. Rework `support/playground.rs`: steps for start, chat, end, the
   empty-store checks; drop `retired_reload` if nothing needs it.
2. The three tests and the rewrites.
3. Then, one at a time:
   - `heavy cargo nextest run -p temper --test process --profile process-temper -E 'test(/playground/)'`
     (or through `tools/tiers/run`, as `AGENTS.md` says for the tier);
   - `heavy cargo nextest run -p temper --profile qa-temper -E 'binary(/^qa_/) & test(/first_run|delegation/)'`;
   - `heavy cargo nextest run -p temper` (focused);
   - `heavy cargo nextest run -p temper-real-loop-world` if you changed it;
   - `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
4. Commit and push.

## Report

The standard report, plus: each test rewritten or deleted and why;
each test's wall time and its tier's budget.
