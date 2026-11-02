# release: status

2026-11-02. Where temper's code lags or departs from [release.md](release.md).

- **Not yet:** 2: the release's file, its symbol check and checksum. 3:
  `temper --version` and `temper version`, and the build script that sets
  the commit. 4: `temper box`'s check of git, `/usr/bin/rg` and `/bin/sh` at
  its start; only `temper init` warns of a missing git
  (`crates/temper/src/init.rs`). 5: `scripts/release.sh`. 6: the walk on a
  fresh machine.
