# git: status

2026-11-11. Where temper's code lags or departs from [git.md](git.md).

Section 4.7's delivery snapshot has no known implementation gap: it reads
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

Section 4.6's merge progress stdout is drained within the existing bound,
without treating a truncated progress message as a broken merge. The
component test `a_merges_progress_stdout_does_not_refuse_its_conflict_listing`
checks exit 0 and exit 1 past the cap; the latter still reads the unmerged
index. A merge's listing keeps its separate bounded prefix rule.

Section 7 has no known gap in the orderly worker-close ordering: serve
waits for its agent processes before aborting the worker's remaining
resources (README.md, section 6). Its second signal still aborts at once.

Sections 4.6 and 6 have no known gap in a merge's listing of its unmerged
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

- **Not yet:** 4.3: `refs/temper/landing` is written by a landing branch's fetch alone; no pin fetch writes the head a writable item naming no landing branch replaces.
- **Not yet:** 1, 2, 3, 4.1 to 4.3, 4.11, 5, 6 and 9: no store. `Clone` runs `git clone` of every branch and names an identity, `Fetch` fetches into the repository, `Make` names no store, a start sets aside no `stores`, and `Store` is neither in `temper-worker-domain-checkout`'s `git::Op` nor in `temper-worker-protocol-git` (`invocation.rs`, `arguments`), nor in the fake git or the transcripts.
- **Not yet:** 4.3 and 4.12: `Point` is not built, so a pin of a commit the prepare fetched is a second fetch.
- **Not yet:** 4.7: the checkout world's fake bounds a merge's listing alone: `perform_within` keeps `perform`'s commit, which takes every unmerged path whatever their count (`skein-fake-checkout`'s `git::commit_merging`), so that world's `PastTheBound` case (`temper/tests/worker/checkout/src/merges.rs`) delivers a merge whose `conflicts + 1` files were settled and never staged, where this layer's commit lists them past `conflicts` and ends `Broken` (section 10). The case holds the prepare past the bound and a push after it, not a delivery past it. 4.10: `Rewrite`, the lease-guarded move of a prepare job's candidate: neither `temper-worker-domain-checkout`'s `git::Op` nor `temper-worker-protocol-git` has it. 4.5: no test runs real git on a dirty workspace: `temper/tests/worker/checkout/tests/suite/reuse.rs` proves the worker's operations and, on the fake git, the two commands' state, where the fake's checkout replaces the whole tree whether or not `clean` ran, and `temper/crates/temper-git-transcripts-tool/src/cases.rs`, `locals`, records `checkout --force --detach` over no staged file or merge in progress. What 4.5 says of those rests on one run by hand, git 2.47.3, 2026-11-05 (section 10).
- **Design departs:** 4.7, from the request `parallel-faults` of 2026-11-06, item 3, decided by its plan's cut: a commit leaves a file out by its name alone, `.*.log`, where the request has a snapshot carry no file a change has no business with, so a stray of another name still lands unless the repository ignores it (section 10). A new file can't be told from a run's new work, which `a_saved_retry_delivers_the_file_written_before_its_run_ended` holds, and `*.log` would leave out a new fixture of a kind this repository tracks.
- **Design departs:** 4.5, from the request `first-goal-fixes` of 2026-11-05, item 3, decided by its plan's cut: the next hold sees the commit's tree with nothing staged, the index equal to that tree, where the request says "an empty index", which would contradict the commit's tree. What the run that request tells of saw stays unexplained: the world proves the worker's commands and the fake's state, not what that run's checkout held.
- **Departs:** 2, 4: with `TEMPER_INTERNAL_WORKSPACE_REUSE` set (`../domain/worker/checkouts.status.md`), because a workspace is to outlive a restart: a start sets `home` aside and leaves `workspaces` and `stores`, making each only if it is absent; and an operation section 4 does not list, `Look { at, remote }`, tells whether a repository's directory holds a clone of `remote`, by what `Clone` writes there (4.2). Run from the worker's root, so that no repository above the directory answers for it: `git --git-dir=workspaces/<token>/<repository>/.git config --local --get remote.origin.url`. Exit 0 with stdout exactly the remote and a newline ends `Looked { holds: true }`; other stdout, exit 1 (no such key) or exit 128 (no repository there) ends `Looked { holds: false }`; anything else is section 6's `Broken`. It names no identity: no credential is on disk (section 5), so the URL is the bare remote. Its transcripts are `origin-*` and `checkout-index-locked`.
