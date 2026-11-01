+++
id = "fe-make-git"
part = "fs"
kind = "feature"
needs = ["fe-config-drop"]
files = ["temper/crates/temper-engine-domain-forge/**", "temper/crates/temper/tests/fake_backends/walk.rs"]
effort = "medium"
minutes = 30
+++

## Goal

A Forgejo elsewhere made by `MakeForge` takes the machine's git program
(`notes/contracts.md` T4, `application.machine.git_program`): serve puts it in
`Machine.github.git_program`; the connector's GitHub record already copies it (fe-make
copies the whole template), and a `ForgeChoice::Forgejo` record now takes it too, so
`temper box --fake-backends`' managed Forgejo pushes and fetches through the same program.

Acceptance:
- In `MakeForge`'s decision, the record made for `ForgeChoice::Forgejo { .. }` has
  `git_program: machine.github.git_program.clone()` (it was `None`); nothing else of the
  record changes. `Owned` and the playground's record keep their templates' own (`None`
  for owned; the fake's from its setup entry, as made today).
- The record's `Register` carries it.
- The step test below passes; every test of the crate passes.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T4
  (`application.machine.git_program`); `notes/fe-surface.md` section 1.
- `temper/crates/temper-engine-domain-forge/src/forges.rs` (`fe-make`'s `make`; find it by
  `ForgeChoice::Forgejo` if it lives elsewhere).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/`.
- May change: nothing else.

## Steps

1. In `make`, the Forgejo arm sets `git_program` from `d`'s machine template
   (`machine.github.git_program`).
2. Test, beside `fe-make`'s in `src/tests/make.rs`:
   `a_forgejo_elsewhere_takes_the_machines_git_program` (a machine whose GitHub template has
   `git_program: Some(b"data/bin/git")`: the made Forgejo record and its `Register` carry it;
   with `None`, the record has `None`; the owned record made from `machine.owned` keeps its
   own).
3. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/make::/)'` (check the count), then `--lib` whole; `cargo fmt --all`; `heavy cargo
   clippy -p temper-engine-domain-forge --all-targets -- -D warnings`; `python3 -I -B
   scripts/old_forms.py` from `temper/`.
4. Commit, push your branch.

## Report

- Nothing beyond the implementer's text.

## Answers

2026-11-01, from bx-walks' question (its fake-backends walk of a Forgejo elsewhere fails
for want of this change). Also: remove the
`#[ignore = "managed Forgejo's git program: fe-make-git"]` that bx-walks put on
`temper/crates/temper/tests/fake_backends/walk.rs`'s
`forgejo_elsewhere_and_a_key_reach_a_first_chat_on_the_fake_box` (the attribute alone;
`files` now names `walk.rs`), and run it alone:
`heavy cargo nextest run -p temper --profile process-temper --test process -E
'test(=fake_backends::walk::forgejo_elsewhere_and_a_key_reach_a_first_chat_on_the_fake_box)'`
(check the count: 1). It passes, or its failure, read, is reported as a question.
