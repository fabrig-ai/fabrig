+++
id = "shell-reason"
part = "shell-reason"
kind = "feature"
needs = []
files = ["smith/crates/smith-local-shell/src/settings/resolve.rs", "smith/crates/smith-local-shell/src/settings/tests.rs"]
effort = "medium"
minutes = 20
+++
# `smith check` says why another program account's models stay out

## Goal

With `sub_agents` on, the local shell leaves out of a run's models those of a program account
that is not main's, and `smith check` names the reason "a program runs no sub-agent". A
sub-agent now runs on a program, on main's account (`smith/docs/design/shell.md`, 5.2;
`smith/docs/design/protocol/programs.md`, 5.10), so the reason is wrong. Change it to
`a sub-agent runs on a program only on main's account`, and pin it with a test. Which models are
left out does not change.

Acceptance:

- `smith-local-shell/src/settings/resolve.rs` (about line 65) gives that reason, and nothing
  else of `build` changes.
- A test in `settings/tests.rs` asserts it.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/reference-m4/notes/common.md` (read it first).
- `smith/docs/design/shell.md`, 5.2 ("The run's models and accounts").

## Owns and may change

- `smith/crates/smith-local-shell/src/settings/resolve.rs`: the reason string.
- `smith/crates/smith-local-shell/src/settings/tests.rs`: one new test.
- Pre-approved:
  - If a test elsewhere asserts the old string (none did when this was written:
    `rg 'program runs no sub' smith`), update it to the new one and list it.
  - If composing those settings needs more (a `[programs]` path, say), add what
    `settings/sources_tests.rs`' `sub_agents_leave_out_the_models_of_a_program_account` and the
    fixture's other program tests give.

## Steps

1. Add `composition_reports_another_program_accounts_models_outside_the_run` beside
   `composition_reports_an_account_without_a_source_outside_the_run`: settings
   `sub_agents=true`, an `anthropic` account whose source is `program`, and
   `endpoints.anthropic.program='claude_code'`, main on the fixture's default `codex` model;
   `outside_models["anthropic/claude-opus-5-5"]` is the new reason. Run it red:
   `heavy cargo nextest run -p smith-local-shell --lib -E 'test(/outside_the_run/)'`.
2. Change the string; rerun green.
3. `cargo fmt`; `heavy cargo clippy -p smith-local-shell --all-targets -- -D warnings`;
   `heavy cargo nextest run -p smith-local-shell`.
4. Commit, push.

## Report

The red run, the change, the test runs (selected, passed), and implementation and check minutes
apart.
