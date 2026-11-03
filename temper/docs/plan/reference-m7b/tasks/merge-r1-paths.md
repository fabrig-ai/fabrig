+++
id = "merge-r1-paths"
part = "change-merge"
kind = "migration"
needs = ["change-merge-route", "merge-kind", "merge-reference"]
files = ["temper/crates/temper-worker-domain-tooling/src/**", "temper/crates/temper-engine-domain-forge/src/items.rs", "temper/tests/worker/domain/**", "tools/merge/**"]
effort = "medium"
minutes = 75
+++

## Goal

Revision 1 (`notes/contracts.md`, R1): the merge job's repository keeps its own name, so its
remote is right; the job capability resolves a command against the job's directory; the root
gives the merge job's command as `<repository name>/<merge_job>`. This moves the work of
`change-merge-surface`, `change-merge-route`, `merge-kind` and `merge-reference` from C2's
`home` to R1.

Acceptance:

- the forge's item for `Delegate::Merge` is named as the repository is (no `home`), and the
  surface's item test says so;
- `temper-worker-domain-tooling` runs `<directory>/<command>`: its tests' commands gain the
  `home/` they meant (`home/bin/tool`), and a test shows a command under another directory
  (`fabrig/.temper/merge`) runs from there; a command with `..`, a leading `/` or a NUL is still
  refused its start;
- the root's merge spec's command is `<repository name>/<merge_job>`, its unit test
  (`a_merge_jobs_spec_is_its_command_and_its_two_commits`) checking that form;
- the worker's world stories (M7a's job stories and `merge-kind`'s merge stories) name their
  commands under the job's directory as R1 has it, and pass;
- `tools/merge/merge.py` finds its repository from its own path (the directory above
  `.temper/`), never by the name `home`; its tests run it from a repository directory not named
  `home` and pass.

## Inputs

- `temper/docs/design/domain/worker.md, 4.4` ("One spec and one result", "The merge kind");
  `temper/docs/design/protocol/tooling.md, 2` and `4.1`.
- `temper/docs/plan/reference-m7b/notes/contracts.md`, C2 and R1; `notes/common.md`.

## Owns and may change

- `temper/crates/temper-worker-domain-tooling/src/**`: the path (`domain.rs`, where it builds
  `<directory>/home/<command>`), `boundary.rs`'s doc of `Spec::command`, its tests.
- `temper/crates/temper-engine-domain-forge/src/items.rs` (the item's name) and the forge's test
  of it.
- The root's merge encoder, wherever `change-merge-route` put it (`forge_route.rs` or
  `route/forge_route/merge.rs`), the command's form alone, and its unit test. If
  `catch-up-call` is editing the same function at once, keep the hunk to the command's bytes.
- `temper/tests/worker/domain/**`: the stories' commands and directories.
- `tools/merge/merge.py` and `tools/merge/test_merge.py`, only where they assume `home`.
- Readers: `temper-service`'s job tests (`src/tests/worker_jobs.rs`) where they pin a
  `home/` program path, that expectation only.

## Steps

1. The capability's path; its tests.
2. The item's name; the root's command; their tests.
3. The worker world's stories; `tools/merge`, if needed.
4. `heavy cargo nextest run -p temper-worker-domain-tooling`, `-p temper-worker-domain --lib`,
   `-p temper-worker-domain-world`, `-p temper-engine-domain-forge --lib`,
   `-p temper-engine-domain --lib -E 'test(/merge/)'`, `-p temper-service --lib -E
   'test(/job/)'`; `heavy python3 -I -B tools/merge/test_merge.py`; `cargo fmt`;
   `python3 -I -B temper/scripts/old_forms.py`; clippy on every package changed,
   `--all-targets -- -D warnings`.
5. Commit and push.

## Report

As the implementer's text says; name the revision (R1) in the commit's message.
