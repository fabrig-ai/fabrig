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
