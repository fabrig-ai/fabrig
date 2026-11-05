# lib: status

2026-11-05. Where skein's code lags or departs from [lib.md](lib.md).

- **Not yet:** 3: `Slab::get2_mut`. 6: a carrier's lengths, beyond
  skein-http's heads and skein-channel's frames (http.md, 2; channel.md,
  3). These still derive `Debug` over what they carry, as the request of
  2026-11-05 left them: skein-io's stream, pipe and kernel bytes, with
  lib's `stream::Down`, `stream::Up`, `Intake` and `Writer`; skein-kv's
  values; skein-oauth's `Json`; jig's bodies; skein-channel's refusal text
  and `FrameWriter`; skein-http's `sse::Dispatch` and `sse::Outgoing`, and
  what its event-stream reader and writer hold. That request has the first
  four with no known credential path, and the code has one through each.
  The stream's records and `Intake` hold an HTTP head, an HTTP body or a
  channel frame on its way through the stream below. kv's values hold
  jig's secrets' records, unsealed, an account's key and a source's saved
  tokens among them (`jig-protocol-store/src/secrets.rs`). A `Json` holds a
  token response, though only as a local of its reader, in no record that
  prints (`skein-oauth/src/machine.rs`). jig's bodies hold the encoded
  `LinkSignIn` in `Message::SignInLink`, on each side of the transport, and
  in the engine's HTTP `Call`, so the printed link's secret prints there,
  and a bearer's token in `Reply::Record`'s encoded `SignedIn`. Of the rest,
  a `FrameWriter` holds a body as it is written, a local in the same way, and
  none has a known credential path.
- **Departs:** 12: `skein/docs/foundation/programming-model.md`, 5.4
  asks for one explicit state enum per lifecycle, each state holding
  exactly its data, no lifecycle in flags or `Option`s beside it,
  exhaustive matches and one handler per cell;
  `skein-lib/src/requests.rs`'s `RequestTable` keeps `Entry`'s
  `restored`, `save` and `changed` flags and `progress` and `anchor`
  options beside `state`, rather than in lifecycle states; no reason is
  recorded (the code's side is
  `skein/docs/development/debt/D025-request-table-in-lib.md`).
