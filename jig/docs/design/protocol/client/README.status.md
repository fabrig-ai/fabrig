# client: status

2026-11-11. Where jig's code lags or departs from [README.md](README.md).
Unlisted sections have no known gap.

Sections 10 and 11's transport accounting is present: emitted exchanges
keep credit through `Reply` or `Lost`, including cancelled watch calls.
The stream table holds authorization behind a full fetch table, then
releases it after successful or failed HTTP terminals, with no duplicate
watch ending.

- **Design departs:** 8, from the request `ui-small-1` of 2026-11-06,
  item 1's unconditional used-link explanation: after a restart or eviction
  from the last 8 remembered links, a used link answers `link_unknown`.
  The plan chose bounded memory rather than a stored history; its direction
  to use the current terminal's link still holds.

- **Not yet:** 4: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`../store/README.md`, 6.1). 11: the protocol world's stories over an in-process transport: `jig/tests/client-protocol` runs them through native HTTP bytes alone.
- **Not yet:** 6, *A version is its schema's `family` line* and *The layouts' list holds versions to layouts*: jig's client schema's `family` line says 19 while `VERSION` is 20; no wire keeps a layouts' list and nothing checks one [in flight: change 1977].
- **Not yet:** 6, *A record the client can't read*: such a record drops the stream, the link goes offline and opens it again after its backoffs [in flight: change 1978].
- **Departs:** 8, *Never taken, nothing made*: the approval-expiry reservation remains `sign-in.status.md` (Departs, the whole document).
- **Departs:** 4: a tree's `next` is a cursor no read takes: `jig/crates/jig-protocol-engine-clients/src/translate/watches.rs` writes it with `cursor::encode_tree`, in the form `Tree`; `cursor::decode` refuses that form, `cursor::decode_window`, which a family's window reads, takes a bare `DocumentCursor` (`jig/crates/jig-protocol-engine-clients/src/tests/reads.rs`), and a watch opens with no cursor at all, so a client that gives it back is refused. It stays because the request of 2026-11-05 held behaviour still while the cursor became a schema's record, and no debt item holds it.  the mounted-call peer-end departure remains `default-http.status.md`, 4.
