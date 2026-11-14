# fake-machine: status

2026-11-14. Where the tree's code lags or departs from [fake-machine.md](fake-machine.md).

- **Not yet:** 1: the checks of a handle and of a process have no
  returning form, so their tests are `#[should_panic]`, and the ledger's
  test fills it one past its size; 6: the fake mmdebstrap's check
  panics on an invocation outside its model rather than returning its
  finding. [in flight: change 3255]
- **Not yet:** 2: the shared fake machine's full file metadata and kinds,
  snapshot sharing, subvolume inode rules and all path-resolution flags;
  the richer walk fixtures in
  `hearth/tests/tree/src/fs.rs` are separate. 3, 4, 5: btrfs, id maps,
  namespaces, cgroups and build sandboxes. 6: the universe's installed files,
  database and maintainer-script effects, beyond its resolution index.
  9: the power-cut crash model; the seam rejects `PowerLoss` as M2.
  10: filesystem capacities and the machine's M2 faults; its byte bound
  asserts a scenario limit rather than returning ENOSPC.
- **Not yet:** 11: the generic half's move into skein and use by temper;
  it remains in `hearth/testing/hearth-fake-machine/`.
