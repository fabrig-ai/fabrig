# state: status

2026-11-23. Where the tree's code lags or departs from [state.md](state.md).

- **Not yet:** 1.2: the subvolume enumeration, paths,
  identities, flags, lineage and nested-subvolume checks, and settling
  staging, trash, orphaned layers and tree files. The scan indexes layer
  records by their recomputed keys and leaves the other known layer
  names alone.
- **Departs:** 1.2: an unreadable layer record or one whose key does
  not recompute is left on disk, unindexed and reported among strays,
  rather than deleted with its layer. `hearth-domain-store`'s layer
  index and `hearth-protocol-builder`'s store adapter defer that
  settlement to the btrfs operations they cannot perform.
- **Departs:** 1.2: `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
  state enum per lifecycle, each state holding exactly its data, no lifecycle
  in flags or `Option`s beside it; `hearth-domain-store`'s
  `Scan` keeps `loading`, `layer`, `last`, `removing` and `after` beside its
  `Pass`, and `staging.rs`'s `Working` `parent`, `parent_uuid`, `walk`, `then`
  and `tmp` beside its `Next`, because the request of 2026-11-05 that gave
  `Store` its states paid a debt item that did not name these two.
- **Not yet:** 1.1: pins and ancestor
  retention, collection of layers, resolutions and cache files, the
  free-space floor and collection before declaring a store full;
  deletion of published layers by unrecord, retire, thaw, empty and
  rmdir, or whole-subvolume deletion with `user_subvol_rm_allowed`.
  Discard orders the emptying and removal of a staging layer and its
  temporary tree file, but the protocol refuses those operations.
- **Not yet:** 1.1: retaining each layer's counts in the in-memory
  index. Translation from the decoded record drops its counts, and the
  domain's indexed `Layer` has no counts field.
