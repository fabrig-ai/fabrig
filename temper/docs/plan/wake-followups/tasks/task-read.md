+++
id = "task-read"
part = "task-read"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-smith/src/answers.rs", "temper/crates/temper-engine-smith/src/tests.rs"]
effort = "medium"
minutes = 50
+++

# wake-followups: a task read refused as unknown says what to read instead

## Goal

A goal asked `read_task` for the `result` of its running delegates and each came back `Task
read refused: unknown`. The read worked as designed: a task has no result until it ends, and
jig refuses a result no ended task kept as unknown (jig's `jig/docs/design/domain/engine.md`,
7.3). The words left the run nothing to do. temper's words for that refusal now say what
`unknown` may mean and what to read instead (`temper/docs/design/protocol/llm.md`, section 4).
jig stays as it is.

Acceptance:

- `notes/contracts.md`, section 2, holds: `answers::task_read`'s refusal branch words
  `Refusal::Unknown` exactly as given, and every other refusal as today (through
  `task_problem`). Nothing else of `answers.rs` changes; `task_problem`'s `unknown` stays for
  every other answer.
- A row of `ANSWER_CASES` (`temper-engine-smith/src/tests.rs`, read by
  `each_of_jigs_new_call_answers_renders_its_task_and_value`):
  `engine::CallAnswer::TaskRead(Box::new(Err(jig_domain_tasks::Refusal::Unknown)))`, the
  contract's words, `true`; and a row that a task read refused `Reference` still reads `Task
  read refused: no reference`. The first fails before the words change and passes after.
- It bites: word `Unknown` as before and see the row fail; restore.
- Delete the bullet `**Not yet:** 4: a task read refused as unknown ...` from
  `temper/docs/design/protocol/llm.status.md`, a stated departure from the implementer's text
  (`notes/common.md`), with no other change there.

## Inputs

- `temper/docs/plan/wake-followups/notes/common.md`; `notes/contracts.md`, section 2.
- `temper/docs/design/protocol/llm.md`, section 4 (the bullet this plan adds); jig's
  `jig/docs/design/domain/engine.md`, 7.3 (`read_task`, what it reads, "An unavailable
  terminal report is unknown").

## Owns and may change

- Owns the files in the header. The task `size-refusal` adds one compiler-forced arm to this
  crate's `src/settings.rs` beside you; leave that file alone.
- Pre-approved: if the words pass a test's bound on a rendered refusal (a host answer limit,
  a `clipped_refusal`), keep them whole and say which bound they meet; if a generated help
  page renders this refusal, regenerate it by its tool and name it.
- Not yours: jig's refusal vocabulary and its read (`jig-domain-core/src/reads/run_task.rs`),
  the tool's declaration and description (`src/calls.rs`, whose bytes the brief's figures
  count), `docs/help/`.

## Steps

1. The rows; see the first fail.
2. The words; the rows pass.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-smith --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-smith`; `heavy cargo nextest run -p temper-engine-domain-world --test suite
   -E 'test(/host_calls/)'` (its `unrelated_task_read` checks a refused read leaks no words).
4. Commit (why: a run reading a running delegate's result heard only "unknown"); push your
   branch.

## Report

As the implementer's text says.
