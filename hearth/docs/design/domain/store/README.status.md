# The store: status

2026-11-11. Where hearth's code lags or departs from [README.md](README.md).

- **Not yet:** 2.1, 2.2, 2.3, 4.1, 4.2: layers as read-only btrfs
  subvolumes, the empty layer's root ownership, and the on-disk snapshot,
  freeze, record and publish operations. The domain orders these
  operations, but `hearth-protocol-builder` refuses their requests; the
  builder world's helper copies directories and syncs without a
  read-only flag.
- **Not yet:** 4.4, 4.5, 6.2: the subvolume enumeration, paths,
  identities, flags, lineage and nested-subvolume checks, and settling
  staging, trash, orphaned layers and tree files. The scan indexes layer
  records by their recomputed keys and leaves the other known layer
  names alone.
- **Departs:** 4.4, 6.2: an unreadable layer record or one whose key does
  not recompute is left on disk, unindexed and reported among strays,
  rather than deleted with its layer. `hearth-domain-store`'s layer
  index and `hearth-protocol-builder`'s store adapter defer that
  settlement to the btrfs operations they cannot perform.
- **Departs:** 4.6: a second staging request is refused rather than
  waiting for a key in progress; different keys cannot stage together.
  `hearth-domain-store` holds one staged layer for a build that makes
  its layers in order; concurrent builds need a table of staged layers.
- **Departs:** 4, 6.2: `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
  state enum per lifecycle, each state holding exactly its data, no lifecycle
  in flags or `Option`s beside it; `hearth-domain-store`'s
  `Scan` keeps `loading`, `layer`, `last`, `removing` and `after` beside its
  `Pass`, and `staging.rs`'s `Working` `parent`, `parent_uuid`, `walk`, `then`
  and `tmp` beside its `Next`, because the request of 2026-11-05 that gave
  `Store` its states paid a debt item that did not name these two.
- **Not yet:** 5.1, 5.2, 5.3, 5.4, 5.5, 6.1: pins and ancestor
  retention, collection of layers, resolutions and cache files, the
  free-space floor and collection before declaring a store full;
  deletion of published layers by unrecord, retire, thaw, empty and
  rmdir, or whole-subvolume deletion with `user_subvol_rm_allowed`.
  Discard orders the emptying and removal of a staging layer and its
  temporary tree file, but the protocol refuses those operations.
- **Not yet:** 6.1: retaining each layer's counts in the in-memory
  index. Translation from the decoded record drops its counts, and the
  domain's indexed `Layer` has no counts field.
- **Not yet:** 9: the store world with btrfs ownership, read-only flags,
  subvolume enumeration and deletion, pins and collection; its full
  crash and fault matrix and real unprivileged btrfs checks. The builder
  world's fake helper uses stand-in digests; the separate tree world
  exercises the walk and its real codec.
- **Not yet:** 2.4, 7.1, 7.2, 7.3: checking the store's filesystem
  and subvolume, the subordinate uid and gid ranges and the namespace
  and cgroup prerequisites; recording the range before the first layer
  and refusing a different range. The store record's codec carries a
  range, but startup checks only the layout and uid.
- **Not yet:** 3.4: the tree-record digest gaps remain `records.status.md`, 3.
- **Departs:** 5.1: unsettled: `records.md`, 2 allows a reconciled packages
  layer's base to be collected while its descendants are kept; 5.1
  makes that base an ancestor held by a descendant's pin. Section 10
  keeps the question.
