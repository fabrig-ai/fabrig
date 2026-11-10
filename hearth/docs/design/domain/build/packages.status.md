# packages: status

2026-11-10. Where the tree's code lags or departs from [packages.md](packages.md).

- **Not yet:** 1, 2, 3, 4, 5, 6, 7: the digest-checked
  package fetch, sparse local repository, offline bootstrap arguments
  and hooks, seeds and preseeds, cleanup and installed-set verification.
  Resolution and its kept indices exist, but no build step runs the
  fetcher, mmdebstrap or the target's `dpkg-query`.
- **Departs:** 4, 5: unsettled: 4's mmdebstrap argument vector
  lists setup, essential and customize hooks, but omits the extract hook
  that 5 requires to seed fixed ids before packages are configured.
  `README.md`, section 13 keeps the question.
- **Not yet:** 8: computing the kernel, modules and initramfs-input
  digests and passing kernel inputs to the packages walk. The tree
  format carries boot digests, but the walk only copies its parent's
  list and has no kernel inputs for a packages layer.
