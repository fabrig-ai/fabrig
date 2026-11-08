+++
id = "drafter-run"
part = "drafter-stock"
kind = "feature"
needs = ["delegate-held", "drafter-stock"]
files = [
  "temper/crates/temper-service/src/tests/ask_bound.rs",
]
effort = "medium"
minutes = 35
+++

# A design session's first run, on the stock

## Goal

One story, on the stock's own settings, that a person's design session runs the
drafter: template 6's chat is admitted under the stock policy (its delegation names
charter 12), its run carries the drafter's instructions, is given no repository,
and is declared no `goal`, `change` or `delegate`, while it keeps the forge's reads
and its document's `note` and `recall` (`temper/docs/design/domain/agent.md`, 8.5;
`temper/docs/design/protocol/configuration.md`, 13.4). It is the meeting point of
`drafter-stock`'s charter and `delegate-held`'s declarations, both merged.

Acceptance:

1. `first_assignment()` in `temper/crates/temper-service/src/tests/ask_bound.rs`
   takes the charter and template it starts, `first_assignment(charter: u32,
   template: u32) -> engine::Assignment`, and
   `asks_first_request_fits_both_shapes` calls it with Ask temper's, 11 and 5,
   unchanged in what it checks.
2. A test beside it, `a_design_session_runs_the_drafter_with_no_change_or_goal`,
   over `first_assignment(12, 6)`:
   - the run's policy instructions are `include_bytes!("../stock/drafter.txt")`;
   - `assignment.workspace.repositories` is empty;
   - `temper_engine_smith::charter(&assignment).grants.host_tools` names none of
     `goal`, `change` and `delegate`, and names `note`, `recall`, `read_branch`
     and `read_file`.
3. Nothing else in the file moves. If the start is refused or proposed rather than
   assigned, the stock's delegation lacks charter 12 somewhere: that is
   `drafter-stock`'s red, a `Question:` naming the refusal, not this task's fix.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.2, 4.3 and 8.5.
- `temper/docs/design/protocol/configuration.md`, 13.4, the rows *charter 12* and
  *template 6*.
- `temper/docs/plan/ui-drafter/notes/contracts.md`.
- `temper/docs/plan/ui-drafter/notes/common.md`.

## Owns and may change

- Owns: `temper/crates/temper-service/src/tests/ask_bound.rs`, its fixture's
  parameters and the new test.
- Its constants `ASKING_CHARTER` and `ASKING_TEMPLATE` may become the first
  call's arguments; add `DRAFTER_CHARTER` and `DRAFTING_TEMPLATE` beside them if
  the test reads better so.

## Steps

1. Read the inputs and `ask_bound.rs` (`first_assignment`, about line 69).
2. Write the test, then the fixture's parameters. Break each assertion once (the
   charter 1 in place of 12 for the declarations; the instructions' file) and see
   it fail for it; restore.
3. Narrow tests: `heavy cargo nextest run -p temper-service --lib -E
   'test(=tests::ask_bound::a_design_session_runs_the_drafter_with_no_change_or_goal)
   | test(=tests::ask_bound::asks_first_request_fits_both_shapes)'`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-service`.
5. Commit (why, in the message) and push.

## Report

As the implementer's text says, and: each assertion broken and the failure seen;
the test's CPU time as nextest reports it.
