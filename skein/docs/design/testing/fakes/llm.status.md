# fake-llm: status

2026-11-12. Where skein's code lags or departs from [llm.md](llm.md).

- **Not yet:** 2.5: no strict switch; a turn no script answers takes
  "done" or the random answer [in flight: change 1019].

Section 3.2 has no known implementation gap in configured list bodies,
status replacement between reads, credential observations or keeping
lists apart from scripted calls.

- **Departs:** 4: `skein/docs/foundation/testing-strategy.md`, 4 asks
  for no shared types, but this section designs a shared codec:
  `skein-fake-llm-protocol/src/documents.rs` uses `skein-llm`'s
  `anthropic` and `openai` document types and codecs. No reason is
  recorded for departing from the foundation; the backfill recommends
  amending `skein/docs/foundation/testing-strategy.md`, 4, settled when
  `skein/docs/development/debt/D043-browser-kit-json-and-tables.md` is
  planned.
  3.1: a world may also have the peer keep each query's body as it read it,
  inflated if it came in gzip (`Peer::keeping_bodies`, `Peer::bodies`,
  `Server::keep_body`), because a test must compare a kept body with what
  the provider read (`../../llm/README.status.md`); it comes out with that hook. Unasked,
  nothing is kept, since the worlds count memory.
