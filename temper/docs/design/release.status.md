# release: status

2026-11-08. Where temper's code lags or departs from [release.md](release.md).

5.1's workflow, `.github/workflows/release-macos.yml`, is built, its
strip and ad hoc signature with it, and has not yet run: no dispatch
has shown that a cold build with the bundle embedded fits its job's 120
minutes, that the file Xcode's `strip` leaves verifies under the
signature `codesign` remakes, or that its checks pass on that file on
the runner. Its first dispatch follows landing.

- **Not yet:** 2: the release's file, its symbol check and checksum. 3:
  `temper --version` and `temper version`, and the build script that sets
  the commit. 4: `temper box`'s check of git, `/usr/bin/rg` and `/bin/sh` at
  its start; only `temper init` warns of a missing git
  (`crates/temper/src/init.rs`). 5: `scripts/release.sh`. 6: the walk on a
  fresh machine.
