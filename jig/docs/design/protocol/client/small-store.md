# The client's protocol: the small store

## 1. The small store

- **The client's records** (`../../client/domain/client/small-store.md`, section 1):
  the request table's, each a whole request, jig's or an application's,
  with the key the engine knows it by, its party, the application's key
  for it, its owner and version, and when it was first sent; and the
  name of the client's stream (`wire.md`, section 2), under a key of its own, which
  no request's key can be (a request's is 16 bytes); nothing else of the
  client's, and never a credential, an answer, a request carrying a
  value, or one whose answer carries one (`README.md`, section 3). A top-level `Seen` has no stored record. Each
  kept record is encoded with jig's wire, `Kept { layout, record }`, its
  `layout` that of jig's wire it was written at (`versions.md`, section 1), and its
  head, `layout` included, the same at every layout. A request's entry,
  `TableEntry`, names beside its owner and version its owner's layout,
  the one the client declared it at.
- **A kept record holds the largest request.** A kept record's bytes
  (`Kept.record`) are bounded by the widest request-table entry
  (`TableEntry`), so that every request the client admits can be kept,
  and so sent: 1,114,368 bytes, the request's 1,114,240 and 128 for the
  entry's other fields and the request's length (88 at their widest).
  The client's own
  limits still bound what it writes (its `document_bytes`, `README.md`, section 10).
  A record past them is not kept (below, *Kept requests stored before
  sent*).
- **The stream's name** is saved at each hello, in a write of its own,
  and replaces the last; nothing waits for it, and a write of it not
  stored retires nothing. It is not a credential: ending a stream takes
  its sign-in's credential as well (`wire.md`, section 2). It is not one of the
  request table's records, and the records read at start are bounded by
  what the table holds and this one.
- **The application's own records are its own.** What its presentation
  keeps (where the person was, their drafts; the reference UI's places
  and drafts among them) it keeps itself, where it likes. The client's
  `Save` carries only the client's records.
- **Kept requests stored before sent.** The transport answers each `Save`
  and `Erase` `Stored` or `NotStored`. A kept request's exchange never
  leaves before its record is stored: the client sends it only once its
  `Save` is answered `Stored`. `NotStored` (a quota full, a disk failing)
  is each request
  whose record that write was the last to carry not sent, parked or not,
  answered *not kept* (`../../client/domain/client/small-store.md`, section 1).
  A record the link can't encode for the store is a write never made:
  its request is answered *not kept* by its key, unsent, as for
  `NotStored`, and never answered lost. A
  request carrying a value, or whose answer carries one, is never
  stored, and leaves at once, once (`README.md`, section 3). An account's move
  *written* is an ordinary request: kept, and sent again under its key.
- **Held exchanges wait for durability and leave in order.**
  `request_records_durable` holds keyed exchanges behind pending writes;
  `held_exchanges_fifo` releases the surviving attempts once, in arrival
  order, skipping cancelled attempts even when their credit is reused.
  The client protocol world holds this under load in
  `jig-client-protocol-world`,
  `orders::held_exchanges_wait_for_durability_and_leave_in_order_under_load`;
  it holds failure and late completions without a send in
  `orders::failed_persistence_never_releases_held_exchanges_under_load`.
- **`Seen` has no save of its own.** The client domain consumes its
  table save before the composed client or link registers a store write
  (`../../client/domain/client/requests.md`). Its exchange still waits
  behind writes already given, and is eligible only for the confirmed
  party on the current link. A failed mixed write reports not kept only
  for the ordinary requests whose records it carried; a held `Seen`
  exchange is lost and retried normally. Consuming the save creates no
  tracked write, and its store acknowledgements are any write's. Every terminal
  still emits `Erase`, including an absent `Seen` key, with the usual
  acknowledgement.
- **Read at start,** the records the application's shell loaded handed
  to `jig-client` as it starts, each request's to the request table, and
  the stream's name to the link (`wire.md`, section 2); one at a layout
  of jig's wire the client does not speak is dropped, and the
  application told, so a request kept across an update that changed its
  record's layout is lost, not sent wrong. A request whose owner,
  version and layout are not among those the application declares is
  one (`../../client/domain/client/small-store.md`, 1). Pre-release, nothing is
  translated.
- **An unanswered `Seen` is lost** on closing, reload or crash, since
  it is in memory alone; the next opening checks and sends the
  watermark again, with no notice from a record of its own
  (`../../client/domain/client/small-store.md`, section 1).
- **Where** is the shell's: a browser tab's session storage, as text
  skein's small store part encodes (skein's `ui/web/effects.md`, section 1); a
  file a native client writes whole and syncs; a service's file, synced
  before its requests leave; a world's memory.
