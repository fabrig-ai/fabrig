# Testing hearth: status

2026-11-06. Where hearth's code lags or departs from [testing.md](testing.md).

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
- **Not yet:** 9.1: worlds repeat per seed
  what a seed doesn't change, as `rendered_keys` does in
  `hearth/tests/builder/tests/suite/domain.rs`, because the request of 2026-11-06
  has its repairs stop once its aim is met, and a machine's own figure met that
  aim before this one began. What else they left stands in 11.
- **Departs:** 5.5, 9.2: the `conformance` profile is `conformance-hearth` and
  also runs `recorded_*`, and there is no `measure` profile, because the
  root's tier profiles are `<tier>-<tree>` (`tools/tiers/design.md`, 1).
- **Departs:** 8.4: a world's memory tests are modules of its suite, not a
  binary of their own, because a suite has one counting allocator
  (`skein/docs/design/testing.md`, 6).
- **Departs:** 9.1: no global timeout holds hearth's suites alone, because
  the root has one `default` and one `fuzzy` profile, whose 150 s and 600 s
  sum each tree's figures, hearth's 30 s and 120 s among them.
- **Departs:** 9.1: the budget reads the queue's figure, and for the suite as it
  stands none has been read. The last the request of 2026-11-06 gives, 30.5 CPU s
  over 2189 tests at 14:49 UTC that day, precedes its repairs, so neither the
  30 CPU s, nor the 27 it aims for on that figure, nor the record after landing it
  asks for, is shown, because nothing in a plan runs after it lands. Outside the
  queue, all 34 hearth workspace packages with
  `NEXTEST_DOUBLE_SPAWN=0` and unoptimized test builds measure each test's user
  plus system CPU, waited and reaped descendants included, excluding compilation,
  listing and nextest (`tools/tiers/design.md`, 7.3). At `3ed249efcd`, whose hearth
  code is main's, on a skein that has changed since, the focused suite reads
  23.767613 and 23.143157 CPU s over 2193 tests; the lower figure has 19.110414
  user s and 4.032743 system s. Its families are builder 4.938397 (106 tests),
  tree 6.606667 (55), real git 3.610547 (184), render 2.609483 (109), and the
  rest 5.378063 CPU s (1739). The same machine, in the same sitting, reads 31.472249
  and 30.349378 over 2189 tests at `39b490f138`, before the repairs: the lower
  figures stand at 76.26%, 7.206221 CPU s less, under the 85% of a machine's own
  figure that request aims for, and every one of the 2189 remains. Fuzzy reads
  68.963035 CPU s (68.569861 user, 0.393174 system), 19.271153 wall s over 60
  tests, the large render 4.114809 CPU s. All selected tests pass, without retries,
  starvation, unobserved tests, leaks or incomplete accounting. The suite's CPU by
  function has a sampled instruction profile, from before the repairs and none
  since (the `hearth-speed-2` plan's figures, at `668f7cd3e5`). The fuzzy budget's
  basis and the render world's selection on every change await the user's ruling
  (11). The causes of variation across machines and the large render's split
  between harness and production code remain unknown.
