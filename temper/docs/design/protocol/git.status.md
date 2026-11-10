# git: status

2026-11-10. Where temper's code lags or departs from [git.md](git.md).

Section 7 has no known gap in the orderly worker-close ordering: serve
waits for its agent processes before aborting the worker's remaining
resources (README.md, section 8). Its second signal still aborts at once.

Sections 4.6 and 6 have no known gap in a merge's listing of its unmerged
paths: past the checkout's `conflicts` it keeps the first `conflicts`, and
past `stdout_bytes` it is read as far as it was kept, a last cut path
dropped, while every other invocation's stdout past its bound, a merge's
commit's listing among them, stays `Broken`
(`temper-worker-protocol-git/src/output.rs`, `conflict_paths`;
`operation.rs`). The standard rows of
`each_operation_s_next_invocation_is_git_md_s`,
`nul_lists_are_complete_for_commits_and_bounded_prefixes_for_merges` and
`a_merges_unmerged_stdout_alone_keeps_a_prefix_past_its_cap` hold it, and
the checkout world's fake follows it for a merge
(`temper/tests/worker/checkout/src/translate.rs`, `perform_within`).

- **Not yet:** 4.7: the checkout world's fake bounds a merge's listing alone: `perform_within` keeps `perform`'s commit, which takes every unmerged path whatever their count (`skein-fake-checkout`'s `git::commit_merging`), so that world's `PastTheBound` case (`temper/tests/worker/checkout/src/merges.rs`) delivers a merge whose `conflicts + 1` files were settled and never staged, where this layer's commit lists them past `conflicts` and ends `Broken` (section 10). The case holds the prepare past the bound and a push after it, not a delivery past it. 4.10: `Rewrite`, the lease-guarded move of a prepare job's candidate: neither `temper-worker-domain-checkout`'s `git::Op` nor `temper-worker-protocol-git` has it. 4.5: no test runs real git on a dirty workspace: `temper/tests/worker/checkout/tests/suite/reuse.rs` proves the worker's operations and, on the fake git, the two commands' state, where the fake's checkout replaces the whole tree whether or not `clean` ran, and `temper/tools/git-transcripts/src/cases.rs`, `locals`, records `checkout --force --detach` over no staged file or merge in progress. What 4.5 says of those rests on one run by hand, git 2.47.3, 2026-11-05 (section 10).
- **Design departs:** 4.7, from the request `parallel-faults` of 2026-11-06, item 3, decided by its plan's cut: a commit leaves a file out by its name alone, `.*.log`, where the request has a snapshot carry no file a change has no business with, so a stray of another name still lands unless the repository ignores it (section 10). A new file can't be told from a run's new work, which `a_saved_retry_delivers_the_file_written_before_its_run_ended` holds, and `*.log` would leave out a new fixture of a kind this repository tracks.
- **Design departs:** 4.5, from the request `first-goal-fixes` of 2026-11-05, item 3, decided by its plan's cut: the next hold sees the commit's tree with nothing staged, the index equal to that tree, where the request says "an empty index", which would contradict the commit's tree. What the run that request tells of saw stays unexplained: the world proves the worker's commands and the fake's state, not what that run's checkout held.
- **Departs:** 2, 4: with `TEMPER_WORKSPACE_REUSE` set (`../domain/worker.status.md`), because a workspace is to outlive a restart: a start sets `home` aside and leaves `workspaces`, making it only if it is absent; and an operation section 4 does not list, `Look { at, remote }`, tells whether a repository's directory holds a clone of `remote`, by what `git clone` writes there. Run from the worker's root, so that no repository above the directory answers for it: `git --git-dir=workspaces/<token>/<repository>/.git config --local --get remote.origin.url`. Exit 0 with stdout exactly the remote and a newline ends `Looked { holds: true }`; other stdout, exit 1 (no such key) or exit 128 (no repository there) ends `Looked { holds: false }`; anything else is section 6's `Broken`. It names no identity: no credential is on disk (section 5), so the URL is the bare remote. Its transcripts are `origin-*` and `checkout-index-locked`.
