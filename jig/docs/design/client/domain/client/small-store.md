# The client: the small store

## 1. The small store

- **What the client keeps across its own restart:** the requests not yet
  answered, except top-level `Seen` and requests carrying a value
  (`requests.md`), as the request table's records, each with the key the
  engine knows it by, its party, the application's key, the request (jig's record, or a family's
  bytes under its owner's number and version), and the wall time of its first
  sending; and the name of its stream, which the client that follows
  sends gone before it opens its own
  (`../../../protocol/client/wire.md`, section 2, and `../../../protocol/client/small-store.md`, section 1). Nothing else of the
  client's: everything shown is read again. A presentation's own records
  (the reference UI's drafts and its place) are the application's, which
  it keeps beside the client's (../root.md, section 8; ../view-state.md, 8).
- **Kept requests before sent.** An ordinary request's record leaves in a
  `Save`, and the client gives its first `Exchange` only once the transport
  says `Stored`. `NotStored` (a quota full, a disk failing) retires the
  requests the write carried, unsent, each answered *not kept*, by its
  key (skein's `runtime/lib.md`, 12, not kept): parked, or with its `Exchange`
  waiting for the write, since a parked request's record leaves at its
  admission too. Its erase leaves as any retired request's, since an
  earlier write may hold its record; if that erase is not stored either,
  a restart tells the request `Kept` again, under its key. A record is
  whole in each write, so a request whose record a later write carries
  again is decided by that later write, and one whose record the write
  did not carry is sent once its own write is stored. So a restart never
  loses an ordinary kept request the engine may have made. A request
  carrying a value is sent once without a record, and its caller asks
  again (`requests.md`).
- **A record that can't be written is not kept.** A request whose record
  the link can't encode for the store (past a kept record's bounds,
  `../../../protocol/client/small-store.md`, section 1) is retired as one
  `NotStored` retires: unsent, answered *not kept* by its key, once, its
  erase leaving as any retired request's. It is never answered lost, so
  never sent again under its key, and the requests whose records were
  encoded go on as before.
- **A later write holds its requests.** `request_records_durable`
  requires pending writes to finish before held requests are sent; an
  earlier write's completion cannot release a later write's requests.
  A failed write ends its own obligations as not kept. The client
  world holds this under load in `jig-client-world`,
  `orders::a_later_write_holds_its_requests_under_load`.
- **The durability hold is uncapped.** Held exchanges are bounded by
  admitted attempts, never refused as not kept for a full hold. Once
  the write is durable, live attempts leave in FIFO order; stale
  attempts are skipped. Not kept reports a store failure, not internal
  queue room (skein/docs/foundation/programming-model.md, 7.2).
- **`Seen` has no save of its own.** Closing, reload or crash silently
  loses an unanswered in-memory mark, with no `Kept`, `Dropped` or not
  kept notice from its own store. The next opening checks and sends the
  watermark again; repeating a forward-only read position is safe. Its
  exchange still waits behind writes already given. A failed mixed
  write retires only the ordinary requests whose records it carried as
  not kept; a `Seen` exchange held behind it is lost and follows its
  normal retry. Consuming the table's save creates no tracked store
  write and changes no store acknowledgement protocol.
- **Every terminal erases.** The table's terminal `Erase` holds for
  `Seen` too: erasing an absent key is harmless, and its store
  acknowledgement completes normally. A `Seen` record the small store
  holds is not read: it is dropped, counted in `Dropped`, and its key
  erased at start.
- **Restored at `Start { kept }`,** before any other event: each
  request's record admitted to the table under its key, parked until
  the link is live, its party confirmed and the retention known; and
  each told to the application as `Kept { key, request }`, which the
  client root routes by the key's asker (../root.md, section 8).
- **It names its party.** Each record carries the party it is for, and
  its request is sent, under its original key, only once `me` confirms
  that party. While another party is signed in, it stays parked and
  unsent, until its party returns or the retention passes and it is
  answered unknown. Whether, and how, to warn of someone else's kept
  work is the presentation's (the reference UI's: ../pages.md, section 8).
  A `Send` before `me` has named the party is answered unconfirmed at
  once, and nothing of it is kept (`requests.md`).
- **Named by its layout,** that of jig's wire it was written at, so a
  client updated since reads what an older one kept at its own layout.
  A record at a layout it does not speak is dropped, and the application
  told (`../../../protocol/client/small-store.md`, section 1). A family's request
  is kept in its owner's version and layout: one whose owner the build
  does not declare, or whose version or layout is not the one declared
  (`watches.md`, 2), is such a record. So is one past the records the
  table holds (`README.md`, section 11). The client tells only how many, `Dropped {
  count }`, once at start, under no asker: nothing of a record it
  does not read is read, its key's asker included. The client root
  gives the count to the presentation that says so (../root.md, section
  8): the reference UI's notice (../view-state.md, 7).
- **Where it lives is the host's:** a browser tab's session storage, a
  file written whole and synced, a world's memory. A host whose store is
  copied (a browser tab duplicated) sends the same keys from both
  copies, whose ordinary deciding acts the engine makes once, while each
  copy draws new keys from a seed of its own; the copy sends the original's
  stream gone, once, which the original takes as a drop (`watches.md`, 2).
- **It is never the truth.** An ordinary deciding request kept is
  answered by the engine by its key, made once whatever the client did
  before.
