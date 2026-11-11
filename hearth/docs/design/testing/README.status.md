# Testing hearth: status

2026-11-11. Where hearth's code lags or departs from [README.md](README.md).

- **Not yet:** 2.3: the M2 system world joining the builder and the real tree
  helper on one fake machine. 2.5, 2.6: the tree helper's protocol and simulated
  worlds. 2.7: the tree helper's real loop and the process-group stopping test.
  The builder's domain world scripts the helper in
  `hearth/tests/builder/src/domain/layers.rs`; the tree world's walk runs
  separately in `hearth/tests/tree/`.
- **Not yet:** 3.2: M2's conformance scenarios, including resolution in both
  modes, bootstrap, extras, fetch, query and the target's tools. 3.4: the fake
  machine's btrfs and namespace rules held to the kernel. The conformance
  binaries cover git, resolution and render.
- **Not yet:** 3.5: nothing runs `conformance-hearth` when a fake or a
  recording changes, since neither the gate nor the after-landing tier names
  it (`.config/gate/README.md`): `recorded_*`, which needs no prerequisite,
  runs by hand alone, and nothing records `conformance_*`, which need the
  real tools and the mirror, passing on the skein hearth builds on.
- **Not yet:** 4.1: worlds repeat per seed
  what a seed doesn't change, as `rendered_keys` does in
  `hearth/tests/builder/tests/suite/domain.rs`, because the request of 2026-11-06
  has its repairs stop once its aim is met, and a machine's own figure met that
  aim before this one began. What else they left stands in 11.
- **Departs:** 3.5, 4.2: the `conformance` profile is `conformance-hearth` and
  also runs `recorded_*`, and there is no `measure` profile, because the
  root's tier profiles are `<tier>-<tree>` (`tools/tiers/design.md`, 1).
- **Departs:** 4.1: no global wall timeout holds hearth's suites alone,
  because the root has one `default` and one `fuzzy` profile, whose 150 s
  and 600 s wall backstops cover all trees together.
- **Not yet:**  the complete-machine worlds remain `worlds.status.md`, 1.  The sandbox, sealed-layer and crash checks remain `worlds.status.md`, 3 and 5. 4.3: build, fetch and layer costs
  checked against that machine's counts and the real tier. 5: these M2
  additions; the builder's domain tests use scripted layer operations.
