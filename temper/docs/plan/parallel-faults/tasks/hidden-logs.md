+++
id = "hidden-logs"
part = "snapshots"
kind = "feature"
needs = ["merged-start"]
files = [
  "temper/crates/temper-worker-protocol-git/src/invocation.rs",
  "temper/crates/temper-worker-protocol-git/src/tests/invocation.rs",
  "temper/crates/temper-fake-git/src/check.rs",
  "temper/crates/temper-fake-git/src/answer.rs",
  "temper/crates/temper-fake-git/src/tests.rs",
  "temper/crates/temper-fake-git-hosted/tests/suite/transcripts.rs",
  "temper/tools/git-transcripts/src/cases.rs",
  "temper/tools/git-transcripts/transcripts/**",
  "temper/docs/design/protocol/git.status.md",
]
effort = "high"
minutes = 60
+++

# A commit stages no hidden log

## Goal

A failure's save put `.cursor-focused.log`, 8,373 lines, on a goal's branch. A save and a delivery are one operation, and `git add --all` takes an untracked file unless the repository ignores it, as `git.md` 4.7 said to. So the request's inference is right that it stages untracked files, but the repository's ignore rules are already honoured. Neither a save nor a delivery can tell a run's new file from a stray one, and a test pins that new work survives (`a_saved_retry_delivers_the_file_written_before_its_run_ended`). So the rule this plan took is by name: a file named `.*.log`, at any depth, is never staged.

Acceptance:

- `Command::Add` is `git add --all -- . ':(exclude,glob)**/.*.log'`, for a save and a delivery, with and without a second parent.
- An untracked `.a.log` and `sub/.b.log` reach no commit. A new `keep.log` and `sub/work.txt` do.
- A tracked `sub/.tracked.log` that was changed keeps the parent's contents in the commit.
- A tree whose only difference is a hidden log commits nothing (`Unchanged`).

These are all as real git behaves (run by hand with 2.47 on 2026-11-06).

## Inputs

- `temper/docs/design/protocol/git.md`, 4.7, and section 10's **A stray file of another name**.
- `temper/docs/plan/parallel-faults/notes/common.md`.

## Owns and may change

Owns the files in the header.

Reader and neighbour files, the argv only: `temper/crates/temper-fake-git-hosted/tests/suite/check.rs`, `temper/tests/worker/protocol-git/tests/suite/transcripts.rs`, `temper/crates/temper/tests/suite/fake.rs`, `temper/tests/worker/checkout/src/hosted.rs`, `temper/crates/temper-worker-protocol-git/src/operation.rs`, `sequence.rs` and `files.rs`. `temper/tests/worker/domain/**` is a read-only reader: its fake works below the argv and doesn't see this rule, so no story there can bite.

Departs from the implementer's text: delete from `temper/docs/design/protocol/git.status.md` the `Not yet` entry beginning `4.7: the exclusion of a hidden log`, leaving `4.10` and the rest.

Options, each pre-approved:

- **Transcripts.** The `commit-add` case records `git add --all` and `temper/tools/git-transcripts/src/cases.rs` runs it twice. Change both and regenerate with the tool (its `README.md`, which needs the pinned Forgejo). If the tool can't run on your builder, then the transcript test goes red, so stop and report `Question:` with that, your branch pushed. Never hand-edit a fixture.
- If `temper-fake-git`'s `stage` can't express the exclusion simply, give it a plain name check for this one pattern, not a general pathspec parser. The argv is checked exactly in `check.rs`, and real git's reading of the pattern is what the transcript records.
- The fake's `stage` keeps a tracked file in `prior`: keep the parent's contents for a tracked hidden log rather than the working tree's.

## Steps

1. **Reproduce.** `hosted_git_stages_no_hidden_log` in `temper/crates/temper-fake-git-hosted/tests/suite/transcripts.rs`, beside `hosted_git_stages_files_and_commits_the_requested_parents_without_moving_head`, driving `Disk::run` with `Command::Add` over the acceptance's files, then write-tree and commit-tree. It fails today on `.a.log`.
2. **Fix.** In `invocation.rs`: `Command::Add => words(&mut out, &[b"add", b"--all", b"--", b".", b":(exclude,glob)**/.*.log"])`, with the pattern as one named `const` citing `protocol/git.md`, 4.7. Then `check.rs`'s accepted argv, the invocation table in `src/tests/invocation.rs`, and the fake's `stage`.
3. Break the rule once (the old argv in `invocation.rs`, or the exclusion dropped from `stage`) and see the story fail.
4. fmt, `old_forms.py`, clippy and the focused tests of `temper-worker-protocol-git`, `temper-fake-git`, `temper-fake-git-hosted`, `temper-worker-domain-checkout`, the `temper/tests/worker/*` worlds and `git-transcripts`. Commit and push.

## Report

The implementer's report, plus: whether the transcript was regenerated against real git, and what it recorded for this argv.
