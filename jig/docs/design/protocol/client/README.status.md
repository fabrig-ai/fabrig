# client: status

2026-11-27. Where jig's code lags or departs from [README.md](README.md).
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
- **Departs:** 4: a tree's `next` is a cursor no read takes: `jig/crates/jig-protocol-engine-clients/src/translate/watches.rs` writes it with `cursor::encode_tree`, in the form `Tree`; `cursor::decode` and `cursor::decode_window` refuse that form, and a watch opens with no cursor at all, so a client that gives it back is refused. It stays because the request of 2026-11-05 held behaviour still while the cursor became a schema's record, and no debt item holds it. The mounted-call peer-end departure remains `default-http.status.md`, 4.
- **Not yet:** 4: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`../store/growth.md`, 1.1).
- **Not yet:** 4: a family window's continuation in the form `window`: the engine reads a window's `after` as a bare `DocumentCursor` naming no layout, and a family read whose continuation does not decode is answered not a record [in flight: change 21062]
