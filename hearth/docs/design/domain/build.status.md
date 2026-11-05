# Building outputs: status

2026-11-05. Where hearth's code lags or departs from [build.md](build.md).

- **Not yet:** 2, 3, 4.1, 4.3: executing the rootfs recipe's steps,
  pinning the plan's chain and producing an output through the builder
  service. The build domain orders stage, run, seal and discard, but
  `hearth-protocol-builder` answers every step as unavailable and
  refuses the store's layer operations.
- **Not yet:** 4.2: reconcile-base selection, apt reconciliation,
  fallback to scratch and the depth and age bounds. The plan's `How`
  has only `Scratch`; it has no base candidates or reconcile runner.
- **Not yet:** 4.4: waiting on a key in progress, concurrent builds and
  steps, and per-source-host fetch admission. The builder runs one
  build, and its store holds one staged layer.
- **Not yet:** 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7: the digest-checked
  package fetch, sparse local repository, offline bootstrap arguments
  and hooks, seeds and preseeds, cleanup and installed-set verification.
  Resolution and its kept indices exist, but no build step runs the
  fetcher, mmdebstrap or the target's `dpkg-query`.
- **Departs:** 5.4, 5.5: unsettled: 5.4's mmdebstrap argument vector
  lists setup, essential and customize hooks, but omits the extract hook
  that 5.5 requires to seed fixed ids before packages are configured.
  Section 15 keeps the question.
- **Not yet:** 5.8: computing the kernel, modules and initramfs-input
  digests and passing kernel inputs to the packages walk. The tree
  format carries boot digests, but the walk only copies its parent's
  list and has no kernel inputs for a packages layer.
- **Not yet:** 6.1, 6.2, 6.3: fetching and placing software, recording
  the placed digests and running `software.sh` with the target's tools.
- **Not yet:** 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7: users and sysusers
  lines, applying the cached file tree with resolved owners and exact
  modes, tmpfiles, presets and `customize.sh`, using the target's
  binaries. The plan names these steps, but none has a runner.
- **Not yet:** 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8: the build
  sandboxes and tool root, namespaces, id maps, mounts and devices,
  offline environment, cgroup limits, deadlines and stopping with proof
  of an empty process tree. The service has no build-step process or
  sandbox adapter.
- **Not yet:** 9.1, 9.2, 9.3: scan-rule data, content checks and
  findings in the helper walk, refusing offenders and changes to fixed
  `/usr` and boot files, and recording the rules' digest. The accounts
  decoder exists, but the walk has no scan-rule input or findings.
- **Not yet:** 10.1, 10.2, 10.3, 10.4, 10.5: the build-mode
  mmdebstrap arguments and events, `hearth-protocol-dpkg`'s fetch and
  installed-set machines, `hearth-protocol-tail`, and the build tools'
  recordings and conformance checks. The mmdebstrap machine used for
  resolution does not run a packages build.
- **Not yet:** 11: caching fetched packages and software through their
  build steps and reusing published layers in real builds. The plan can
  reuse indexed keys, and render uses the resolution and blob caches.
- **Not yet:** 12: measuring and reporting step and build costs,
  including cgroup CPU and memory, fetch and write counts and cache
  hits; the layer record's tool and cost lists are empty.
- **Not yet:** 13: the build world with the recipe's real step
  behaviour, tools, sandbox views, scan and cost invariants, its full
  fault matrix and real builds. The builder domain world runs seeded
  steps and a fake helper with stand-in digests; it does not exercise
  the packages, software or config tools.
