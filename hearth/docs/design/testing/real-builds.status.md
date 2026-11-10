# real-builds: status

2026-11-10. Where the tree's code lags or departs from [real-builds.md](real-builds.md).

- **Not yet:** 1, 2, 3, 4, 5: the real-build tier, its fetched .debs
  and extras, shared base layers, prerequisite checks and per-step cost checks.
  The mirror supports M1's indexes; `hearth/tests/builder/tests/real/main.rs`
  exercises render and diff with real git, rather than builds on btrfs.
  6: the built root booting in QEMU; the `real/boot.rs` module checks the
  rendered boot part, not a boot.
