# records: status

2026-11-11. Where the tree's code lags or departs from [records.md](records.md).

- **Not yet:** 6: the store prerequisites remain `README.status.md`, 2.4, 7.1, 7.2 and 7.3.
- **Not yet:** 3: subtree digests and the packages layer's boot
  digests. The tree codec carries `boot`; the walk copies its parent's
  list and gives an empty list with no parent.
- **Not yet:** 2: the helper cost and record-population gaps remain `tree-helper.status.md`, 7.
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
