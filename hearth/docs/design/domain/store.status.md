# The store: status

2026-11-05. Where hearth's code lags or departs from [store.md](store.md).

- **Not yet:** 2.1, 2.2, 2.3, 4.1, 4.2: layers as read-only btrfs
  subvolumes, the empty layer's root ownership, and the on-disk snapshot,
  freeze, record and publish operations. The domain orders these
  operations, but `hearth-protocol-builder` refuses their requests; the
  builder world's helper copies directories and syncs without a
  read-only flag.
- **Not yet:** 2.4, 6.6, 9.1, 9.2, 9.3: checking the store's filesystem
  and subvolume, the subordinate uid and gid ranges and the namespace
  and cgroup prerequisites; recording the range before the first layer
  and refusing a different range. The store record's codec carries a
  range, but startup checks only the layout and uid.
- **Not yet:** 3.4, 6.3: subtree digests and the packages layer's boot
  digests. The tree codec carries `boot`; the walk copies its parent's
  list and gives an empty list with no parent.
- **Not yet:** 4.4, 4.5, 8.2: the subvolume enumeration, paths,
  identities, flags, lineage and nested-subvolume checks, and settling
  staging, trash, orphaned layers and tree files. The scan indexes layer
  records by their recomputed keys and leaves the other known layer
  names alone.
- **Departs:** 4.4, 8.2: an unreadable layer record or one whose key does
  not recompute is left on disk, unindexed and reported among strays,
  rather than deleted with its layer. `hearth-domain-store`'s layer
  index and `hearth-protocol-builder`'s store adapter defer that
  settlement to the btrfs operations they cannot perform.
- **Departs:** 4.6: a second staging request is refused rather than
  waiting for a key in progress; different keys cannot stage together.
  `hearth-domain-store` holds one staged layer for a build that makes
  its layers in order; concurrent builds need a table of staged layers.
- **Departs:** 4, 8.2: `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
  state enum per lifecycle, each state holding exactly its data, no lifecycle
  in flags or `Option`s beside it; `hearth-domain-store`'s
  `Scan` keeps `loading`, `layer`, `last`, `removing` and `after` beside its
  `Pass`, and `staging.rs`'s `Working` `parent`, `parent_uuid`, `walk`, `then`
  and `tmp` beside its `Next`, because the request of 2026-11-05 that gave
  `Store` its states paid a debt item that did not name these two.
- **Not yet:** 5.1, 5.2, 5.3, 5.5, 5.6, 5.8: the helper process and
  service, namespace and id maps, pipe protocol with its message bounds,
  apply and empty, and the subvolume and sync operations. The typed
  requests exist, but no `hearth-service-tree` or
  `hearth-protocol-helper` runs them.
- **Not yet:** 5.3, 5.4: `hearth-protocol-xattr`'s capability and ACL
  validation, the scan rules and findings, and feeding the accounts
  decoder's entries to the helper for owner resolution and password
  checks. The walk keeps covered attribute values as bytes; the
  accounts decoder is separate from it.
- **Not yet:** 5.7, 6.2: measuring helper request costs and filling a
  layer record's scan digest, placed downloads, tool versions and step
  costs. The record translation uses a zero scan digest for nonempty
  layers and empty lists for `placed`, `tools` and `costs`; packages
  metadata describes scratch alone.
- **Not yet:** 6.1, 6.6: target records, their codec and bounds, and
  recording the last N builds per target.
- **Departs:** 6.2, 7.1: unsettled: 6.2 allows a reconciled packages
  layer's base to be collected while its descendants are kept; 7.1
  makes that base an ancestor held by a descendant's pin. Section 12
  keeps the question.
- **Not yet:** 6.4: recording apt's and mmdebstrap's versions in a
  resolution's `made` fields in M2. The resolution adapter supplies
  empty version strings.
- **Not yet:** 6.8: writing and ordering durable layer tree files and
  records through the helper and builder adapters, and treating all
  layers as unrecorded when the store record is unreadable or its
  layout is unknown. The startup path rewrites the store record and
  then indexes layer records without that distinction.
- **Not yet:** 7.1, 7.2, 7.3, 7.4, 7.5, 8.1: pins and ancestor
  retention, collection of layers, resolutions and cache files, the
  free-space floor and collection before declaring a store full;
  deletion of published layers by unrecord, retire, thaw, empty and
  rmdir, or whole-subvolume deletion with `user_subvol_rm_allowed`.
  Discard orders the emptying and removal of a staging layer and its
  temporary tree file, but the protocol refuses those operations.
- **Not yet:** 8.1: retaining each layer's counts in the in-memory
  index. Translation from the decoded record drops its counts, and the
  domain's indexed `Layer` has no counts field.
- **Not yet:** 11: the store world with btrfs ownership, read-only flags,
  subvolume enumeration and deletion, pins and collection; its full
  crash and fault matrix and real unprivileged btrfs checks. The builder
  world's fake helper uses stand-in digests; the separate tree world
  exercises the walk and its real codec.
