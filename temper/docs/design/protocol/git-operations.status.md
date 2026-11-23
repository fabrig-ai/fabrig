# git-operations: status

2026-11-23. Where the tree's code lags or departs from [git-operations.md](git-operations.md).

- **Not yet:** 1.7: an empty listing still refuses with its words rather
  than ending `ParentUnreachable { head, merging, parent }` [in flight:
  change 6015].
- **Not yet:** 1.3, *Main's counts*: the component has no `Counts`; 1.13: the view
  copies no `refs/temper/counts`; 1.14: so no counts fetch takes the store's
  lock [in flight: change 13129].

Section 1.7's delivery snapshot has no other known implementation gap: it reads
HEAD and MERGE_HEAD, checks a second parent against freshly fetched remote
branches and tags, preserves a local commit whose tree is delivered, and
moves HEAD after a new delivery commit. The protocol step tests in
`src/tests/snapshot.rs` hold the staged merge, committed merge, local commit
and unreachable second parent on scripted git output. The checkout world's
`suite::delivery` also checks the resulting fake object graph and preserved
commit ids, and refuses an unpushed second parent in staged and committed
merges.

Unchecked saves use the same snapshot. The checkout world's delivery rows
cover both checked and unchecked pushes. A checked delivery after an
unchecked save keeps the fresh-commit requirement even for an equal tree;
the protocol's `a_checked_delivery_after_an_unchecked_save_makes_a_fresh_equal_tree_commit`
checks that replacement and its HEAD update.

Section 1.6's merge progress stdout is drained within the existing bound,
without treating a truncated progress message as a broken merge. The
component test `a_merges_progress_stdout_does_not_refuse_its_conflict_listing`
checks exit 0 and exit 1 past the cap; the latter still reads the unmerged
index. A merge's listing keeps its separate bounded prefix rule.

`git.md`, section 7 has no known gap in the orderly worker-close ordering: serve
waits for its agent processes before aborting the worker's remaining
resources (service-process.md, section 1). Its second signal still aborts at once.

`git-operations.md`, section 1.6, and `git.md`, section 6 have no known gap in a merge's listing of its unmerged
paths: past the checkout's `conflicts` it keeps the first `conflicts`, and
past `stdout_bytes` it is read as far as it was kept, a last cut path
dropped, while every other parsed invocation's stdout past its bound, a merge's
commit's listing among them, stays `Broken`
(`temper-worker-protocol-git/src/output.rs`, `conflict_paths`;
`operation.rs`). The standard rows of
`each_operation_s_next_invocation_is_git_md_s`,
`nul_lists_are_complete_for_commits_and_bounded_prefixes_for_merges` and
`a_merges_unmerged_stdout_alone_keeps_a_prefix_past_its_cap` hold it, and
the checkout world's fake follows it for a merge
(`temper/tests/worker/checkout/src/translate.rs`, `perform_within`).

- **Not yet:** 1.3: `refs/temper/landing` is written by a landing branch's fetch alone; no pin fetch writes the head a writable item naming no landing branch replaces.
- **Not yet:** 1.10: `Rewrite`, the lease-guarded move of a prepare job's candidate: neither `temper-worker-domain-checkout`'s `git::Op` nor `temper-worker-protocol-git` has it. 1.5: no test runs real git on a dirty workspace: `temper/tests/worker/checkout/tests/fakes/reuse.rs` proves the worker's operations and, on the fake git, the two commands' state, where the fake's checkout replaces the whole tree whether or not `clean` ran, and `temper/crates/temper-git-transcripts-tool/src/cases.rs`, `locals`, records `checkout --force --detach` over no staged file or merge in progress. What 1.5 says of those rests on one run by hand, git 2.47.3, 2026-11-05 (`git-later.md`, section 1).
- **Departs:** 1: with `TEMPER_INTERNAL_WORKSPACE_REUSE` set (`../domain/worker/checkouts.status.md`), because a workspace is to outlive a restart: a start sets `making` and `home` aside and leaves `workspaces` and `gitdirs`, making each only if it is absent; and an operation section 1 does not list, `Look { at, remote, store }`, tells whether a repository's directory holds a clone of `remote`, by what `Clone` writes in the worker's directory (1.2), run at the worker's root as every invocation is: `git --git-dir=gitdirs/<token>/<repository> config --local --get remote.origin.url`. Exit 0 with stdout exactly the remote and a newline ends `Looked { holds: true }`; other stdout or exit 1 (no such key) ends `Looked { holds: false }`; exit 128 (no repository there) ends it false too, but where `workspaces/<token>/<repository>` is a directory: a checkout made before the worker kept two git directories, which `Look` takes on, making the worker's directory as `Clone`'s steps 1 to 4 do, from `store`, and leaving the work tree and its `.git`, now the run's and never read, as they are, before it ends `Looked { holds: true }`. The prepare's check out then writes every file of its commit, the worker's index being new, and the view (1.13). Anything else is `git.md`, section 6's `Broken`. It names no identity: no credential is on disk (`git.md`, section 5), so the URL is the bare remote. Its transcripts are `origin-*` and `checkout-index-locked`.
- **Design departs:** 1.7, from the request `parallel-faults` of 2026-11-06, item 3, decided by its plan's cut: a commit leaves a file out by its name alone, `.*.log`, where the request has a snapshot carry no file a change has no business with, so a stray of another name still lands unless the repository ignores it (`git-later.md`, section 1). A new file can't be told from a run's new work, which `a_saved_retry_delivers_the_file_written_before_its_run_ended` holds, and `*.log` would leave out a new fixture of a kind this repository tracks.
- **Design departs:** 1.5, from the request `first-goal-fixes` of 2026-11-05, item 3, decided by its plan's cut: the next hold sees the commit's tree with nothing staged, the index equal to that tree, where the request says "an empty index", which would contradict the commit's tree. What the run that request tells of saw stays unexplained: the world proves the worker's commands and the fake's state, not what that run's checkout held.
