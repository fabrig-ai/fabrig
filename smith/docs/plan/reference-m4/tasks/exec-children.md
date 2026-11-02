+++
id = "exec-children"
part = "exec-children"
kind = "feature"
needs = ["program-children"]
files = ["smith/tests/end-to-end/program_children.rs", "smith/tests/end-to-end/end_to_end.rs", "smith/tests/fake-claude/src/print.rs", "smith/tests/fake-claude/src/story.rs", "smith/tests/fake-claude/tests/suite/turns.rs"]
effort = "high"
minutes = 75
+++
# `smith exec` on Claude Code asks inspect-only sub-agents side by side

## Goal

The walk's story through the shipped product, on the fake program: `smith exec` on a `program`
account with `sub_agents` on runs main on Claude Code, which asks two inspect-only sub-agents in
one message; each is a `claude` of its own at the model the call names, they run side by side,
and each answers with its report, which main's own report carries. This is the story fabrig/qa
walks on the real program once M4 lands (`smith/docs/plan/reference-m4/inputs/request.md`,
Walk). Test code only: a production fault it finds is a `Question:` (task `side-by-side` owns
production code in this wave).

Acceptance:

- The `smith` package's `end_to_end` binary has a module `program_children` with the stories
  below, passing.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/reference-m4/notes/common.md` (read it first) and
  `smith/docs/plan/reference-m4/notes/contracts.md`, C4.
- `smith/docs/design/protocol/programs.md`, 5.10; `smith/docs/design/shell.md`, 5.2 ("The run's
  models and accounts": with `sub_agents` on, main's endpoint's models are the run's, sharing
  main's account).
- `smith/docs/design/testing.md`, 2.3 (the end-to-end tier on the fake program).

## Owns and may change

- `smith/tests/end-to-end/program_children.rs` (new): its own setup helper, modelled on
  `program_runs.rs`' `signed_in` and `invocations`, writing a whole `Story` (main's positional
  turns and the children's `cued` scripts) and `sub_agents=true` into the user layer.
- `smith/tests/end-to-end/end_to_end.rs`: one `mod program_children;` line beside
  `mod program_runs;`, nothing else.
- Not yours: `program_runs.rs`, `support/**`, `live.rs` (another draft's tasks are changing them).
- Pre-approved: if `sub_agents=true` needs another setting for main's endpoint's models to be
  the run's (a model declared, a price), add it in your helper's user layer and say which.

## Steps

1. Read the inputs, `program_runs.rs` (its helpers and `exec_spawns_one_print_run_on_its_program`)
   and `smith/tests/fake-claude/src/story.rs` (`Story`, `cued`, `Step::Together`,
   `Step::CallChecked`).
2. Stories:
   - `exec_on_claude_asks_two_sub_agents_side_by_side`: main `anthropic/claude-opus-5-5` on the
     `work` program account, `sub_agents=true`; main's turn 1 is `Step::Together` of two
     `Step::CallChecked { tool: "sub_agent", .. }` with briefs cued `CUE-ALPHA` and `CUE-BETA`,
     `"tools":["inspect"]` and `"llm"` a model of main's endpoint other than main's
     (`claude-haiku-5-5` if the run lists it), each `contains` its child's word; each child says
     its word and ends; main's turn 2 gives the report outcome holding both words. Assert: exit
     0 and the report on standard output; the fake's observations hold main's `-p` and two
     children's `-p`, each child's with `--model` the named model, `--tools Read,Glob,Grep` and no
     `--json-schema`, all three found in the account directory's observation file, which the
     fake opens under its `CLAUDE_CONFIG_DIR` (as
     `a_program_run_never_sees_the_person_s_credential_variables` reasons).
   - `exec_on_claude_refuses_a_writing_sub_agent`: one call asking `"tools":["inspect","shell"]`,
     checked with `contains: "tools"`; no child `-p` is observed; main still finishes.
   Run them: `heavy cargo nextest run -p smith --test end_to_end -E 'test(/program_children/)'`.
3. `cargo fmt`; `heavy cargo clippy -p smith --all-targets -- -D warnings`.
4. `heavy cargo nextest run -p smith --test end_to_end`.
5. Commit, push.

## Report

The stories as written, any setting the helper needed, the test runs (selected, passed), and
implementation and check minutes apart.

## Answers

2026-11-02. **The fake rejects `Step::CallChecked` inside `Step::Together`: who adds it?**
This task does; its acceptance cannot hold without it, and `side-by-side` no longer needs it (it
uses `Step::Call` and reads main's told results from the transcript). Widened to own, test code
only:

- `smith/tests/fake-claude/src/print.rs`: `together` accepts `Step::CallChecked` beside `Call`,
  `Builtin` and `Act`; each queued call carries its own expected reply (the `together` queue's
  entries gain the `contains`, or `None`), and `tool_result`'s and `together`'s `pop_front` set
  `expected_reply` from the entry they start, so each call's reply is checked against its own
  text in the order they run. `interrupt_result` already clears both. Nothing else of the fake
  changes.
- `smith/tests/fake-claude/src/story.rs`: `Step::Together`'s doc names `CallChecked` among its
  steps.
- `smith/tests/fake-claude/tests/suite/turns.rs`: one test beside the file's `Together` tests: a
  `Together` of two `CallChecked` whose first reply holds its text and whose second does not
  observes `Called { error: false }` then `Called { error: true }`. Run it with
  `heavy cargo nextest run -p smith-fake-claude --test suite -E 'test(/together/)'`, then
  `heavy cargo nextest run -p smith-fake-claude` and
  `heavy cargo clippy -p smith-fake-claude --all-targets -- -D warnings`.

Keep the pushed stories as they are; they should pass once the fake does. Merge
`origin/reference-m4/exec-children` first.

