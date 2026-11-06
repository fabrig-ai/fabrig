# client: status

2026-11-06. Where jig's code lags or departs from [client.md](client.md).

Unlisted sections have no known gap.

- **Not yet:** 10: a committed turn's calls carry no objects or parts: `jig-protocol-transcript` decodes them empty. 12: no story relays a person's decision with their own credential. 17, 20: the one-shot mode and its story, and a service in a host or world (the client takes `With::Service`).
- **Departs:** 19: `words` keys its phrases under skein-ui owner 65537 (`jig/crates/jig-client/src/words.rs`, `OWNER`), beside jig's 65536 (root.md, 15); no reason is written.
- **Departs:** 16: a turn answer that `translate::turn::inspect` cannot decode
  ends malformed before the link emits `Fact::UndecodableDocument`
  (`jig-client-protocol-link/src/component.rs`, `read_answer`). Other read
  answers that cannot decode emit the fact. The turn's early return remains
  separate from that shared decode arm; no reason for omitting the fact is
  recorded. Seen by reading.
