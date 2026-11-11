# release: status

2026-11-11. Where temper's code lags or departs from [release.md](release.md).

5.1's workflow, `.github/workflows/release-macos.yml`, is built, its
strip and ad hoc signature with it, and has not yet run: no dispatch
has shown that a cold build with the bundle embedded fits its job's 120
minutes, that the file Xcode's `strip` leaves verifies under the
signature `codesign` remakes, or that its checks pass on that file on
the runner. Its first dispatch follows landing.

Section 3's `temper-version` is present: its build script reads the
commit and whether tracked files differ, and reruns on `HEAD`, the
branch it names, the packed refs and the index, a worktree's included;
its library holds `VERSION`, `COMMIT` and `line`, and its tests hold
the commit's word and the watched paths. Every binary's `--version`
prints from it: the box's, the engine's, the worker's and the agent's
through their tables, the client's through `line`.

- **Not yet:** 3: the four tables' binaries hand their identity in: today `temper-box-shell`, `temper-engine-shell`, `temper-worker-shell` and `temper-agent-shell` link `temper-version` and their tables read `VERSION`, `COMMIT` and `INTERNAL_COMMIT` from it, so a commit rebuilds them, the worlds that link them and every test binary above them.
- **Not yet:** 3: the client's binary hands its line in: today the library `temper-client` links `temper-version` and calls `line` itself, its binary in the same package.
- **Not yet:** 3: the direction rule's row that fails a package outside `temper/bin/` depending on `temper-version`.
- **Not yet:** 2: the release's file, its symbol check and checksum. 4: `temper box`'s check of git, `/usr/bin/rg` and `/bin/sh` at
  its start; only `temper-box init` warns of a missing git
  (`crates/temper-box-shell/src/init.rs`). 5: `scripts/release.sh`. 6: the walk on a
  fresh machine.
