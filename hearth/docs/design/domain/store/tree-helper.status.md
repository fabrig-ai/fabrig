# tree-helper: status

2026-11-10. Where the tree's code lags or departs from [tree-helper.md](tree-helper.md).

- **Not yet:** 1, 2, 3, 5, 6, 8: the helper process and
  service, namespace and id maps, pipe protocol with its message bounds,
  apply and empty, and the subvolume and sync operations. The typed
  requests exist, but no `hearth-service-tree` or
  `hearth-protocol-helper` runs them.
- **Not yet:** 3, 4: `hearth-protocol-xattr`'s capability and ACL
  validation, the scan rules and findings, and feeding the accounts
  decoder's entries to the helper for owner resolution and password
  checks. The walk keeps covered attribute values as bytes; the
  accounts decoder is separate from it.
- **Not yet:** 7: measuring helper request costs and filling a
  layer record's scan digest, placed downloads, tool versions and step
  costs. The record translation uses a zero scan digest for nonempty
  layers and empty lists for `placed`, `tools` and `costs`; packages
  metadata describes scratch alone.
