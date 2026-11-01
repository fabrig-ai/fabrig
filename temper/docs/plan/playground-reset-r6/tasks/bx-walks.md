+++
id = "bx-walks"
part = "fs"
kind = "migration"
needs = ["bx-fixtures"]
files = ["temper/crates/temper/tests/fake_backends/**"]
effort = "high"
minutes = 90
+++

## Goal

The fake backends' walks (`temper/crates/temper/tests/fake_backends/`, binary `process`)
make their forges at runtime, as a person does on a fresh sandbox: GitHub by `forge make
github` (forge 1 on a fresh store, the number the fake GitHub's seeded App names), the owned
Forgejo by `forge make here`, the fake Forgejo by `forge make forgejo API GIT` at its peer's
held address, each set up or connected by the number its make answered, never forge 2, 3
or 4 of the old document.

Acceptance:
- Every walk and qa test under `tests/fake_backends/` that named forge 2 (GitHub), 3 (the
  owned Forgejo) or 4 (the managed fake Forgejo) makes it first through
  `support::forges::{make_github, make_here, make_forgejo}` and takes its number from
  `support::forges::made`; `walk.rs`'s `forge_number(forges, N)` helpers read the made
  number.
- `the_serve_child_reads_retained_effective_documents_across_restart` and
  `qa_restart_preserves_effective_edits_data_and_secrets` check the effective document's
  T4 members (`application.machine.github`) instead of forge 2.
- Run, each alone, with the process profile:
  `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=fake_backends::walk::github_one_click_and_a_key_reach_a_first_chat_on_the_fake_box)'`,
  `...githubs_own_app_and_a_key_reach_a_first_chat_on_the_fake_box`,
  `...chatgpt_sign_in_and_github_one_click_reach_a_first_chat_on_the_fake_box`,
  `fake_backends::walk::program_here::a_program_and_imported_account_survive_a_box_restart`,
  `fake_backends::the_serve_child_reads_retained_effective_documents_across_restart`, and
  each walk of `walk/elsewhere.rs`, `walk/adoption.rs`, `walk/here_key.rs` (check each
  selected count: the module paths are the file's; run `--list` once to read them).

## Inputs

- `temper/docs/design/draft/fake-backends.md` (the sandbox and its walks).
- Draft `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.4.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/tasks/cl-ctl.md`; bx-init's report (the rewrite,
  the fakes' git program); bx-harness's report (`support::forges`).

## Owns and may change

- Owns: `temper/crates/temper/tests/fake_backends/**`.
- May change: `temper/crates/temper/tests/support/forges.rs`, adding a helper the walks
  share (no change to an existing one's behaviour).
- Options, pre-approved:
  - If bx-init's report says the fakes' git program is not set (bx's contract request 2
    unanswered), a walk that pushes through the fake GitHub or Forgejo fails on that push:
    leave it failing, mark it `#[ignore = "fake git program: bx contract request 2"]`, and
    list it.
  - A walk whose number is not 1 on a fresh store because another forge was made first
    (the playground's) makes GitHub first, or reads the number made; never hard-codes it.

## Steps

1. The walks; `heavy cargo check -p temper --tests`.
2. The runs above, one at a time; read a failure's log before anything reruns.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets`. Old forms never raised.
   Commit and push.

## Report

Each walk run, its time, and each one ignored or left failing with the reason.

## Answers

2026-11-01. Wait for fe-make-git and the owned-forge admission fix, or make those production
repairs here? Neither. Waiting cannot happen: `sv-startup-refuse` needs this task, and
`fe-make-git` needs `fe-config-drop`, which needs `sv-startup-refuse`. The repairs stay
with their owners (both specs now say so), and this task lands the migration as pushed
(`0ef0f77df`), with the four walks the repairs block ignored:
- `walk.rs`'s `forgejo_elsewhere_and_a_key_reach_a_first_chat_on_the_fake_box`:
  `#[ignore = "managed Forgejo's git program: fe-make-git"]`.
- `walk/adoption.rs`'s `context_preserves_the_writer_and_a_refused_setup_preserves_the_first_key`,
  `walk/here_key.rs`'s `forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box`,
  `walk/program_here.rs`'s `a_program_and_forgejo_here_reach_ready_on_the_fake_box`:
  `#[ignore = "a made owned forge is not offered: sv-startup-refuse"]`.
Keep each ignored walk's migration whole: it is to pass unchanged once its owner's fix is in.
Acceptance is then the other runs of the list passing, and each of the four, run alone with
`--run-ignored all`, failing only for its reason (the elsewhere walk: no committed reply; the
owned three: `not_offered`). Leave `temper/crates/temper/src/fake/check/tests.rs`'s `verb`
failure under `## Left` (not this task's). Change nothing outside `tests/fake_backends/**`.
