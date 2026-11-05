# Testing hearth: status

2026-11-05. Where hearth's code lags or departs from [testing.md](testing.md).

- **Not yet:** 2.3: the M2 system world joining the builder and the real tree
  helper on one fake machine. 2.5, 2.6: the tree helper's protocol and simulated
  worlds. 2.7: the tree helper's real loop and the process-group stopping test.
  The builder's domain world scripts the helper in
  `hearth/tests/builder/src/domain/layers.rs`; the tree world's walk runs
  separately in `hearth/tests/tree/`.
- **Not yet:** 3.2: the shared fake machine's full file metadata and kinds,
  snapshot sharing, subvolume inode rules and all path-resolution flags;
  the richer walk fixtures in
  `hearth/tests/tree/src/fs.rs` are separate. 3.3, 3.4, 3.5: btrfs, id maps,
  namespaces, cgroups and build sandboxes. 3.6: the universe's installed files,
  database and maintainer-script effects, beyond its resolution index.
  3.9: the power-cut crash model; the seam rejects `PowerLoss` as M2.
  3.10: filesystem capacities and the machine's M2 faults; its byte bound
  asserts a scenario limit rather than returning ENOSPC.
- **Not yet:** 3.11: the generic half's move into skein and use by temper;
  it remains in `hearth/testing/hearth-fake-machine/`.
- **Not yet:** 4.2, 4.3, 4.4, 4.5: M2's bootstrap, fetch, query and target-tool
  fakes and recordings. The installed tools in
  `hearth/testing/hearth-fake-machine/src/hearth.rs` are git and mmdebstrap;
  `hearth/testing/hearth-fake-mmdebstrap/` resolves without installing.
- **Not yet:** 5.2: M2's conformance scenarios, including resolution in both
  modes, bootstrap, extras, fetch, query and the target's tools. 5.4: the fake
  machine's btrfs and namespace rules held to the kernel. The conformance
  binaries cover git, resolution and render.
- **Not yet:** 5.5: nothing runs `conformance-hearth` when a fake or a
  recording changes, since neither the gate nor the after-landing tier names
  it (`.config/gate/README.md`): `recorded_*`, which needs no prerequisite,
  runs by hand alone, and nothing records `conformance_*`, which need the
  real tools and the mirror, passing on the skein hearth builds on.
- **Not yet:** 6.1, 6.2, 6.3, 6.4, 6.5: the real-build tier, its fetched .debs
  and extras, shared base layers, prerequisite checks and per-step cost checks.
  The mirror supports M1's indexes; `hearth/tests/builder/tests/real/main.rs`
  exercises render and diff with real git, rather than builds on btrfs.
  6.6: the built root booting in QEMU; the `real/boot.rs` module checks the
  rendered boot part, not a boot.
- **Not yet:** 8.1: the M2 store, build and system worlds on the complete fake
  machine. 8.3, 8.5: the build sandbox, sealed-layer and crash checks against
  that machine, including a cut between walk and record and
  `systemd-container`'s nested subvolume. 9.3: build, fetch and layer costs
  checked against that machine's counts and the real tier. 10: these M2
  additions; the builder's domain tests use scripted layer operations.
- **Departs:** 5.5, 9.2: the `conformance` profile is `conformance-hearth` and
  also runs `recorded_*`, and there is no `measure` profile, because the
  root's tier profiles are `<tier>-<tree>` (`tools/tiers/design.md`, 1).
- **Departs:** 8.4: a world's memory tests are modules of its suite, not a
  binary of their own, because a suite has one counting allocator
  (`skein/docs/design/testing.md`, 6).
- **Departs:** 9.1: no global timeout holds hearth's suites alone, because
  the root has one `default` and one `fuzzy` profile, whose 150 s and 600 s
  sum each tree's figures, hearth's 30 s and 120 s among them.
- **Departs:** 9.1: the focused suite is over its 30 CPU s, because the
  repairs tried stopped short of it: 44.886618 CPU s on 2026-11-05, by
  `tools/tiers/cpu.py`, 14.886618 over. That run came before the last
  repair, which took 0.803158 CPU s off the builder's domain tests, and no
  full run has followed it. The request expected the fixed scenarios to
  bring most of the excess; instead the suite went from 35 to 36 CPU s in
  the gate's records of 2026-11-04 to 48.292873 before the first repair,
  and what raised it, or whether those figures compare, is unknown. The
  gate's record gave 30.2 CPU s the same day, and which of the two
  measures the suite is unknown too; since both, the suite has gained
  the two tests of `hearth/tests/builder/tests/suite/orders.rs`,
  0.192197 CPU s, and the git and mmdebstrap worlds' order tests. The
  figure stands
  (`hearth/docs/design/domain/testing.md`, 11).
  Fuzzy, on 2026-11-04 on a shared machine: 68 CPU s and 16 to 21 s of
  wall, before the large render joined it at 12.238013 CPU s; it has not
  been measured since. Also of 2026-11-04: on a full gate hearth adds 27 s
  to the test build and 9 s to clippy; a change to `skein-lib` reaches 28
  of hearth's 30 packages and rebuilds them in 3 s of wall, with its
  compiler caches warm; and `hearth-render-world`, in
  `.config/gate/always.txt`, costs every hearth change 3.4 CPU s focused
  and 29 CPU s fuzzy.
