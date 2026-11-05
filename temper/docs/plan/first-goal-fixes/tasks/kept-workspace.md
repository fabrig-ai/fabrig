+++
id = "kept-workspace"
part = "kept-workspace"
kind = "feature"
needs = []
files = [
  "temper/tests/worker/checkout/tests/suite/reuse.rs",
  "temper/tests/worker/checkout/src/world.rs",
  "temper/tests/worker/checkout/src/hosted.rs",
]
effort = "high"
minutes = 60
+++

# Prove what a kept workspace's next hold sees

## Goal

A resumed run said its checkout held another task's staged files; the pull request it made shipped only its own, and all we have is the run's own words. `TEMPER_WORKSPACE_REUSE` was set on that worker. Prove in the checkout world, either way, that a next hold sees exactly the commit's tree with nothing staged. If it doesn't hold, fix it.

From reading, it holds: `hold.rs` asks `Op::CheckOut { clean: !start.is_saved() }`, `sequence.rs`, `checked_out`, then runs `Command::Clean`, and a saved start never keeps a workspace (`cache.rs`, `starts_saved`). The tests have to show it.

**What this world can and can't show.** The fake is kinder than git: skein's `git::check_out` replaces the whole tree, so a tree test passes whether or not `clean` ran. So each story also asserts the operation (`clean: true`), and the commands are asserted at the program's level. Real git's part was run by hand once, git 2.47.3, 2026-11-05: `checkout --force --detach` then `clean -ffdxq` on a repository with staged new files, an unstaged edit, a conflicted merge in progress, ignored files, an untracked file and a nested repository left `status --porcelain --ignored` empty, `diff --cached` empty and no `MERGE_HEAD`, `MERGE_MSG` or `AUTO_MERGE`. `temper/docs/design/protocol/git.md`, 4.5 says so. No test here runs real git.

"Nothing staged" is what the request's "an empty index" means: the index equals the commit's tree, as `git diff --cached` empty does. A literally empty index would contradict the commit's tree.

Acceptance, each story with its dirty state: a staged new file, an unstaged edit of a tracked file, a merge in progress (`.git/MERGE_HEAD`, `.git/temper-conflicts`), and an ignored file.

- **Kept from another workstream, reuse set:** `Cached::Kept`, no make and no clone, `Op::CheckOut { clean: true }`, then exactly the commit's tree and no merge state.
- **The same workstream's next hold,** reuse set and unset: `Cached::Reused`, the same.
- **After a restart, reuse set:** `Cached::Looking`, the clone kept, the same.
- **A saved start on that dirty workspace, reuse set** (a resumed run's start): it is made again, and nothing of the dirty state survives.
- **At the program's level:** `Command::CheckOut` then `Command::Clean`, on the fake git over that dirty state, each exit 0, leave the tree and `.git/temper-index` equal to the commit's tree and no merge state.
- Break the rule once: `clean: false` in `hold.rs` turns the first three stories red. The report names what they showed. Restore it.
- If a story fails on the worker's own code, fix it in `temper-worker-domain-checkout` or `temper-worker-protocol-git` and say what was wrong.

## Inputs

- `temper/docs/plan/first-goal-fixes/notes/common.md`.
- `temper/docs/design/protocol/git.md`, 4.5.
- `temper/docs/design/domain/worker.status.md`: the **Departs** entry on `TEMPER_WORKSPACE_REUSE`.
- `temper/docs/design/domain/worker.md`, 5.

## Owns and may change

Owns `temper/tests/worker/checkout/tests/suite/reuse.rs`; `temper/tests/worker/checkout/src/world.rs`, for read accessors; `temper/tests/worker/checkout/src/hosted.rs`, where `apply` may become `pub`.

May change only if a story fails on them: `temper-worker-domain-checkout/src/hold.rs`, `cache.rs`; `temper-worker-protocol-git/src/sequence.rs`, `invocation.rs`, each with its step test.

Options, each pre-approved:

- `World` gains `disk_tree(&self, path) -> Tree` and `checkouts(&self, client) -> Vec<(Vec<u8>, bool)>` (repository, `clean`), recorded where `Op::CheckOut` is taken in `world.rs`. Other names of the same shape are yours.
- A fake that disagrees with the 2.47.3 run above is a red in `temper-fake-git`: fix it there and say so. A change to `skein-fake-checkout` is a `Question:`.
- `tools/git-transcripts` stays as it is: recapturing needs a pinned Forgejo and rewrites every case.

## Steps

```rust
// suite/reuse.rs, on its `settings`, `plan`, `ready`, `ITERATIONS`
fn dirty(world: &mut World, at: &[u8]) { /* write_disk: NEW and .git/temper-index/NEW; README edited;
                                            .git/MERGE_HEAD, .git/temper-conflicts; .gitignore'd target/o */ }
fn clean_at(world: &World, client: u64, at: &[u8]) { /* checkouts: clean true; disk_tree less git dirs
                                                        == forge tree of the commit; no merge state */ }

#[test] fn a_kept_workspace_from_another_workstream_starts_from_exactly_its_commit() { /* settings(true, 1), plans 0 then 1 */ }
#[test] fn the_same_workstreams_next_hold_starts_from_exactly_its_commit() { /* for reuse in [true, false] */ }
#[test] fn a_restart_with_a_dirty_clone_starts_from_exactly_its_commit() { /* settings(true, 1), restart */ }
#[test] fn a_saved_start_on_a_dirty_workspace_is_made_again() { /* told.makes rises */ }
#[test] fn the_two_commands_leave_the_commits_tree_and_nothing_staged() { /* hosted::scenario, apply, apply */ }
```

Then `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-worker-checkout-world --all-targets -- -D warnings` (with any crate you fixed), and `heavy cargo nextest run -p temper-worker-checkout-world -p temper-worker-domain-checkout -p temper-worker-protocol-git`. Commit and push.

## Report

The implementer's text's report, and its first line says which: **it holds** or **it did not hold**, with the fix. Say plainly that this proves the worker's commands and the fake's state, and that it does not say what that run saw; the run's own words stay unexplained if it holds.
