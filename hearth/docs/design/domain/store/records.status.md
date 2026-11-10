# records: status

2026-11-10. Where the tree's code lags or departs from [records.md](records.md).

- **Not yet:** 6: checking the store's filesystem
  and subvolume, the subordinate uid and gid ranges and the namespace
  and cgroup prerequisites; recording the range before the first layer
  and refusing a different range. The store record's codec carries a
  range, but startup checks only the layout and uid.
- **Not yet:** 3: subtree digests and the packages layer's boot
  digests. The tree codec carries `boot`; the walk copies its parent's
  list and gives an empty list with no parent.
- **Not yet:** 2: measuring helper request costs and filling a
  layer record's scan digest, placed downloads, tool versions and step
  costs. The record translation uses a zero scan digest for nonempty
  layers and empty lists for `placed`, `tools` and `costs`; packages
  metadata describes scratch alone.
- **Not yet:** 1, 6: target records, their codec and bounds, and
  recording the last N builds per target.
- **Departs:** 2: unsettled: 2 allows a reconciled packages
  layer's base to be collected while its descendants are kept; `README.md`, 5.1
  makes that base an ancestor held by a descendant's pin. `README.md`, section 10
  keeps the question.
- **Not yet:** 4: recording apt's and mmdebstrap's versions in a
  resolution's `made` fields in M2. The resolution adapter supplies
  empty version strings.
- **Not yet:** 8: writing and ordering durable layer tree files and
  records through the helper and builder adapters, and treating all
  layers as unrecorded when the store record is unreadable or its
  layout is unknown. The startup path rewrites the store record and
  then indexes layer records without that distinction.
