# lib: status

2026-11-06. Where skein's code lags or departs from [lib.md](lib.md).

- **Not yet:** 3: `Slab::get2_mut`. 6: `length::Measure`, while skein-http's three writers each keep their own. 6: a carrier's lengths, beyond
  skein-http's heads, skein-channel's frames and the oauth crates' bodies
  and URLs (http.md, 2; channel.md, 3; oauth.md, 4). These still derive `Debug` over what they carry, as the request of
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
  none has a known credential path. jig's `translate::Up::Visited` and
  `jig_domain_accounts::Event::Visited` print the visit address, with the sign-in's `state`,
  challenge and nonce that oauth.md, 4 has hidden, because jig's
  `protocol/llm.md`, 5.2 keeps the address as an answer, spent with its sign-in.
- **Departs:** 6: a step widens with `usize_of` and `len_u64`, but `try_from`
  still stands for them in three places. What the schema compiler prints
  holds `usize::try_from(limit).expect("u32 fits usize")`, 13 sites in
  `skein-oauth` and `skein/examples/ui/view`, which the debt item the request
  of 2026-11-05 paid counts and names no exception for, because changing it
  regenerates every tree's codecs (codec.md, 13). `skein-fake-checkout`
  measures two lengths with `u64::try_from`, against that item too, because
  it has no `skein-lib` dependency; `skein/scripts/old-forms.txt`'s
  `widening` holds them at 23, the other 21 a
  `u64`'s or a signed count's, which can fail and are outside the rule. And
  136 lines of step code write `usize::try_from(..)` with no `.expect(..)` on
  the line, over a `u32` or a `u64`, `skein-kv`'s frames and snapshots among
  them, which that item does not name and that form does not count. 12:
  `skein/docs/foundation/programming-model.md`, 5.4
  asks for one explicit state enum per lifecycle, each state holding
  exactly its data, no lifecycle in flags or `Option`s beside it,
  exhaustive matches and one handler per cell;
  `skein-lib/src/requests.rs`'s `RequestTable` keeps `Entry`'s
  `restored`, `save` and `changed` flags and `progress` and `anchor`
  options beside `state`, rather than in lifecycle states; no reason is
  recorded (the code's side is
  `skein/docs/development/debt/D025-request-table-in-lib.md`).
