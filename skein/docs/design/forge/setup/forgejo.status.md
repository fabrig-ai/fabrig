# forgejo: status

2026-11-11. Where the tree's code lags or departs from [forgejo.md](forgejo.md).

- **Not yet:** 2: `skein/tests/forge/conformance/src/migrate.rs`,
  `ordinary`, judges a missing source's refusal under its earlier name, by
  leaving nothing, and asks no private source [in flight: change 524].
- **Not yet:** 2: `setup-migrate-of-a-missing-source-says-why-and-leaves-an-empty-repository`
  and `setup-migrate-of-a-private-source-says-why-and-leaves-an-empty-repository`
  are in no `facts.txt` line: the file is a run's output, and no `--migrate`
  run has recorded them, so it still holds the missing source's earlier
  name, `fails`, and nothing of a private source, whose refusal the fake
  Forgejo answers as a missing one's on no recording.
- **Not yet:** 2: `setup-actions-reads-forgejo-workflows-alone-when-both-are-there`
  stands `unchecked` in `skein/tests/forge/conformance/facts.txt`: the
  recording ran `--migrate` without `--runner`, so no run shows Actions
  reading `.forgejo/workflows` alone, which a copy's starter workflow
  stands on. 8, and `skein/docs/design/forge/README.md`, 4.3: the facts
  `--migrate` writes (`setup-migrate-*`, `setup-repository-delete-*`, the
  `setup-scope-*` of `Migrate` and `DeleteRepository` and
  `setup-org-of-a-users-name-reads-404-and-making-it-is-refused`) and
  their transcripts were recorded at Forgejo 17.0.0 and stand below
  `facts.txt`'s 17.0.1 header, unchecked at the pin, as no run of the
  mode since has been committed.
   the crate gives the checks and an `Add` one
  refuses is ignored, the owner having checked first, but
  `skein-forge-setup/src/host/limits.rs`'s `check_forge` is `pub(super)`:
  an owner reaches it only by building a `Host`, and
  `temper/crates/temper-service/src/setup/runtime.rs` records an `Add` as
  served whether the host took it or not.
  2: 1.5's step 2 given a path stands on a site admin reading, by its
  path, a private repository in an org it is not in, and no fact records
  that read: `setup-repository-by-id-reads-its-path-for-an-admin` is the
  id's, and `skein/tests/forge/conformance/src/repo_setup.rs` has none for
  the path, so temper's process story alone shows it on a real Forgejo
  (`temper/crates/temper/tests/support/forgejo_elsewhere.rs`).
