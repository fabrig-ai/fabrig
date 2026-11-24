# client: status

2026-11-18. Where jig's code lags or departs from [README.md](README.md).
Unlisted sections have no known gap.



`README.md`, section 10, and `testing.md`, section 1's transport accounting is present: emitted exchanges
keep credit through `Reply` or `Lost`, including cancelled watch calls.
The stream table holds authorization behind a full fetch table, then
releases it after successful or failed HTTP terminals, with no duplicate
watch ending.

- **Design departs:** 8, from the request `ui-small-1` of 2026-11-06,
  item 1's unconditional used-link explanation: after a restart or eviction
  from the last 8 remembered links, a used link answers `link_unknown`.
  The plan chose bounded memory rather than a stored history; its direction
  to use the current terminal's link still holds.

- **Departs:** 8, *Never taken, nothing made*: the approval-expiry reservation remains `sign-in.status.md` (Departs, the whole document).
- **Departs:** 4: a tree's `next` is a cursor no read takes: `jig/crates/jig-protocol-engine-clients/src/translate/watches.rs` writes it with `cursor::encode_tree`, in the form `Tree`; `cursor::decode` refuses that form, `cursor::decode_window`, which a family's window reads, takes a bare `DocumentCursor` (`jig/crates/jig-protocol-engine-clients/src/tests/reads.rs`), and a watch opens with no cursor at all, so a client that gives it back is refused. It stays because the request of 2026-11-05 held behaviour still while the cursor became a schema's record, and no debt item holds it.  the mounted-call peer-end departure remains `default-http.status.md`, 4.
- **Not yet:** 4: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`../store/growth.md`, 1.1).
