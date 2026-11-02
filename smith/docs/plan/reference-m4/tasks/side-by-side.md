+++
id = "side-by-side"
part = "side-by-side"
kind = "feature"
needs = ["program-children"]
files = ["smith/tests/agent-process/tests/suite/program_children.rs", "smith/tests/agent-process/src/program.rs", "smith/tests/agent-process/tests/live/program.rs", "smith/crates/smith-protocol-program/src/**", "smith/crates/smith-agent-service/src/service/program.rs", "smith/crates/smith-agent-service/src/service/program/**"]
effort = "high"
minutes = 120
+++
# Inspect-only children run side by side on a program; their share and their stop

## Goal

With one program child working (task `program-children`), show the rest of
`smith/docs/design/protocol/programs.md`, 5.10 in the agent-process world: two inspect-only
children asked in one message run side by side, each a program of its own, each answering with
its report; a child's share ends in its final turn, given as main's wind-down is; a run's cancel
stops its children's programs; and a live probe checks that two `claude` processes on one
configuration directory answer at once. Fix in production what these stories find, in the
program backend or the service.

Acceptance:

- The stories below pass in `smith-agent-process-world`'s `suite`.
- The live probe compiles and passes clippy; it is never run here.
- Each production change has a unit test in its crate, red before and green after.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/reference-m4/notes/common.md` (read it first) and
  `smith/docs/plan/reference-m4/notes/contracts.md`, C3 and C4.
- `smith/docs/design/protocol/programs.md`, 5.10 (whole), 5.7 (the wind-down and a cancel), 5.9
  (an interrupt is a control request, and says so), 8 (the fake's and the live suite's
  children).
- `smith/docs/design/domain/run.md`, 5.3 (a final turn; the result's usage line; inspect-only
  children side by side) and 9.1 (a child closed by the wind-down returns unanswered).

## Owns and may change

- `smith/tests/agent-process/tests/suite/program_children.rs`: new tests beside
  `program-children`'s, which you may extend with shared helpers but not weaken.
- `smith/tests/agent-process/src/program.rs`: harness helpers and `instances` only.
- `smith/tests/agent-process/tests/live/program.rs`: one new probe and what it needs from the
  file's `Live` helper (a way to start two at once), nothing else of its behaviour.
- `smith/crates/smith-protocol-program/src/**` and the service's `program.rs` and `program/**`:
  only what a story here shows wrong, each with a unit test.
- Pre-approved:
  - The final turn: if the backend gives a child's final turn otherwise than 5.10 says (an
    interrupt, the note as the next user message opening with 5.9's line, every later call
    refused at the prompt as not run), make it so in `smith-protocol-program`, reusing main's
    wind-down path, and say what you changed.
  - The fake: if a story needs the fake `claude` to do what it cannot (a cued script for the
    user message after an interrupt, say), first use an `asked` script keyed on the note's
    words (`smith-domain-run/src/prompt.rs`, `CHILD_FINAL`); if that cannot, stop and ask (the
    fake is not this task's).
  - If the harness cannot cancel a run at a chosen moment, end it by its wall time
    (`Case` and `cancel` in `src/program.rs`) and say so.

## Steps

1. Read the inputs and `program_children.rs` as `program-children` left it.
2. Stories, each run red first where it can be
   (`heavy cargo nextest run -p smith-agent-process-world --test suite -E 'test(/program_children/)'`):
   - `two_program_children_run_side_by_side_and_each_answers`: main's turn 1 is
     `Step::Together` of two `Step::CallChecked { tool: "sub_agent", .. }`, briefs cued
     `CUE-ALPHA` and `CUE-BETA`, each `"tools":["inspect"]`, each `contains` its child's word;
     each child's script makes one allowed `Read` of a workspace file (`Step::Act`) and then says
     its word; main's turn 2 finishes with a report. Assert: two child `-p` invocations, each with
     its `--model`, `--effort` and `--tools Read,Glob,Grep`; both children's `conversation.opened`
     come before either's `conversation.closed`; each child's `Read` reached the prompt and was
     allowed; main's two told `sub_agent` results hold the two words; the run is accepted.
   - `a_program_childs_final_turn_answers_for_its_budget`: main asks one child with
     `"max_turns":2`; the child's first turn calls `Read` and would call again; the agent ends
     its turn with an interrupt and sends the final-turn note; the child answers with text.
     Assert: one interrupt control request reached the child (the fake's `Interrupted`), main's
     told result holds the answer and `stopped for budget: yes`, and the child's
     `conversation.closed` has end `budget`.
   - `a_cancelled_program_run_stops_its_children`: main asks a child whose script hangs
     (`Step::Hang`); the run is cancelled; the child's and main's programs are stopped (the fake's
     `Interrupted` then, past the grace, `Terminated`), the child's call returns unanswered, and
     the run answers cancelled.
3. The live probe in `tests/live/program.rs`: `two_live_programs_on_one_configuration_answer_at_once`:
   two `Live` processes started before either is read, on the same configuration directory,
   each asked to answer with a distinct word and nothing else; both end with a successful
   `result` whose text holds its word. Its failure message names which process failed and its
   `result`'s subtype, never the text. `heavy cargo check -p smith-agent-process-world --tests`.
4. Production fixes the stories show, each with its unit test.
5. `cargo fmt`; `heavy cargo clippy -p smith-agent-process-world` (and every production package
   you changed) `--all-targets -- -D warnings`.
6. Focused tests, one command at a time: `heavy cargo nextest run -p smith-agent-process-world`,
   and for each production package you changed, its own tests and
   `heavy cargo nextest run -p smith --test end_to_end -E 'test(/program/)'`.
7. Commit, push.

## Report

Each story's red run (or why it was green at once), every production change and its unit test,
the pre-approved options you took, the live probe as written, the test runs (selected, passed),
and implementation and check minutes apart.
