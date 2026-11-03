+++
id = "merge-r1-paths"
part = "change-merge"
kind = "migration"
needs = ["change-merge-route", "merge-kind", "merge-reference"]
files = ["temper/crates/temper-worker-domain-tooling/src/**", "temper/crates/temper-engine-domain-forge/src/items.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-worker-domain/src/tests/jobs.rs", "temper/tests/worker/domain/**", "tools/merge/**"]
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

## Answers

2026-11-03. **May this task also change `temper-worker-domain/src/tests/jobs.rs` and
`temper-engine-domain-forge/src/domain.rs` for R1's fixture and the preflight's size?** Yes, both,
each a reader of R1's command form, which R1 gives this task (`notes/contracts.md`, R1:
`change-merge-route` builds to C2, this task moves its command to R1). `files` above now hold
them.

- `jobs.rs`: the fixture's command becomes `home/tools/check` (line 71), so its assertion's
  `<directory>/home/tools/check` holds; nothing else in the file.
- `domain.rs`, `merge_available` alone: the measured length counts the repository's name and its
  `/` before `command.len()`, as the root's encoder now writes it, so a spec the encoder refuses
  is never admitted; write the sum with checked arithmetic, which settles the lint at that line.
  Keep the hunk to that sum: `change-merge-route` owns the rest of the file and may be editing it
  at once. The other inherited lints (lines 2615, 2721, 2753) stay its: Left.
- `tools/merge/README.md`'s `home/` layout: correct that line too (`tools/merge/**` is yours).
- The inherited blockers stay Left, as reported: `tests/verification.rs`'s missing
  `StepInput.merge_job` and the merge job's workspace handoff (both `change-merge-route`'s, the
  latter by its own answer of today). The forge unit test and the four engine-world `merge_job`
  stories that they block are unobserved, not failed by this task; run them once each after
  merging `origin/reference-m7b/change-merge-route` if it has moved, and report what they show.
  With the worker unit suite green and the rest as reported, acceptance holds.

## Rebase (2026-11-03, step merge, foreman reference-m7b/merge-r1-paths/foreman-2)

resolve forge_route/tests.rs by retaining the tier-release helper and R1’s merge-command test.
The merge refused `temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`.
Integration commit `b406e75ee5` retained `release_held_tier` immediately before the merge-spec test; this branch changes that test’s adjacent documentation to R1.
Preserve `release_held_tier` and its callers, including main’s tier-reporting changes from `f012f783bf`.
Keep `a_merge_jobs_spec_is_its_command_and_its_two_commits` with its R1 documentation.
All three `super::merge::spec` calls in that test must pass `b"fabrig"` before `b".temper/merge"`; the decoded command must remain `b"fabrig/.temper/merge"`.
Preserve the request-byte assertions, exact-bound checks and following resolution-result test.
Keep the automatically merged repository-name argument in `forge_route.rs` and R1 encoder in `route/forge_route/merge.rs`.
Verify with `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/merge/)'`; confirm tests were selected.
Limit the conflict resolution to combining these existing changes; no budget or protocol change is needed.
