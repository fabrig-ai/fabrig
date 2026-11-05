# git: status

2026-11-05. Where temper's code lags or departs from [git.md](git.md).

- **Not yet:** 4.10: `Rewrite`, the lease-guarded move of a prepare job's candidate: neither `temper-worker-domain-checkout`'s `git::Op` nor `temper-worker-protocol-git` has it.
- **Departs:** 2, 4: with `TEMPER_WORKSPACE_REUSE` set (`../domain/worker.status.md`), because a workspace is to outlive a restart: a start sets `home` aside and leaves `workspaces`, making it only if it is absent; and an operation section 4 does not list, `Look { at, remote }`, tells whether a repository's directory holds a clone of `remote`, by what `git clone` writes there. Run from the worker's root, so that no repository above the directory answers for it: `git --git-dir=workspaces/<token>/<repository>/.git config --local --get remote.origin.url`. Exit 0 with stdout exactly the remote and a newline ends `Looked { holds: true }`; other stdout, exit 1 (no such key) or exit 128 (no repository there) ends `Looked { holds: false }`; anything else is section 6's `Broken`. It names no identity: no credential is on disk (section 5), so the URL is the bare remote. Its transcripts are `origin-*` and `checkout-index-locked`.
