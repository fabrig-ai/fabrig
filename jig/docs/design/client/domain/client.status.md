# client: status

2026-11-07. Where jig's code lags or departs from [client.md](client.md).

Unlisted sections have no known gap, including 8.2's `watches_full`
fact for `me` and application watches through retries and recovery.

- **Not yet:** 10: a committed turn's calls carry no objects or parts:
  `jig-protocol-transcript` decodes them empty.
- **Not yet:** 12: `not_signed_in` has no `link_used`, `link_expired` or `link_unknown`.
- **Not yet:** 12: no story relays a person's decision with their own credential.
  17, 20: the one-shot mode and its story, and a service in a host or world
  (the client takes `With::Service`).
- **Departs:** 19: `words` keys its phrases under skein-ui owner 65537 (`jig/crates/jig-client/src/words.rs`, `OWNER`), beside jig's 65536 (root.md, 15); no reason is written.
- **Departs:** 16: a turn answer that `translate::turn::inspect` cannot decode
  ends malformed before the link emits `Fact::UndecodableDocument`
  (`jig-client-protocol-link/src/component.rs`, `read_answer`). Other read
  answers that cannot decode emit the fact. The turn's early return remains
  separate from that shared decode arm; no reason for omitting the fact is
  recorded. Seen by reading.
