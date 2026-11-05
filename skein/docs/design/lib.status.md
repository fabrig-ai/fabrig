# lib: status

2026-11-05. Where skein's code lags or departs from [lib.md](lib.md).

- **Not yet:** 3: `Slab::get2_mut`. 6: a carrier's lengths, beyond
  skein-http's heads and skein-channel's frames (http.md, 2; channel.md,
  3). These still derive `Debug` over what they carry: skein-io's stream,
  pipe and kernel bytes, with lib's `stream::Down`, `stream::Up`, `Intake`
  and `Writer`; skein-kv's values; skein-oauth's `Json`; jig's bodies;
  skein-channel's refusal text and `FrameWriter`; skein-http's
  `sse::Dispatch`. Of these, the stream's records and `Intake` hold an HTTP
  head or a channel frame on its way through the stream below, a credential
  among its bytes; none of the others has a known credential path.
- **Departs:** 12: `skein/docs/foundation/programming-model.md`, 5.4
  asks for one explicit state enum per lifecycle, each state holding
  exactly its data, no lifecycle in flags or `Option`s beside it,
  exhaustive matches and one handler per cell;
  `skein-lib/src/requests.rs`'s `RequestTable` keeps `Entry`'s
  `restored`, `save` and `changed` flags and `progress` and `anchor`
  options beside `state`, rather than in lifecycle states; no reason is
  recorded (the code's side is
  `skein/docs/development/debt/D025-request-table-in-lib.md`).
