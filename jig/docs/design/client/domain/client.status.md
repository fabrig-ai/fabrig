# client: status

2026-11-07. Where jig's code lags or departs from [client.md](client.md).

Unlisted sections have no known gap, including 8.2's `watches_full`
fact for `me` and application watches through retries and recovery.

Sections 10 and 14 have no known gap in decoding a turn read alone:
`Read::Turn` uses the archive's byte bound, twice that bound for text
and smith's message and block ceilings. A call input past `turn_text`
is retained whole and unmarked, so tools can page through its last byte.
Turns carried by an activity page still use `turn_text`; a turn past
the archive's byte bound remains section 18's deferred read case.

Section 12 has no known gap in carrying `link_used`, `link_expired` and
`link_unknown` as `not_signed_in`, telling the application, stopping retries
and clearing the reason on the next sign-in. Falling back to a live
transport sign-in after such an answer remains deferred in section 18.

Section 10's decoded model and effort, their window weight, `latest_llm`
and `caught_up` are present. The decoder keeps the names outside the text
budget; the thread's table covers unread activity, older pages and passing
over undecodable or nameless turns. Section 18 defers latest-run metadata
independent of readable turns.

- **Not yet:** 10: a committed turn's calls carry no objects or parts:
  `jig-protocol-transcript` decodes them empty.
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

Sections 4.1, 4.2 and 12 have the approval events and answers in linked
and unlinked modes, `With::Approval`, the take retried each second while
waiting, and its terminal signing outcomes. The secret stays in memory,
with secret-safe diagnostics, and never enters the small store.
Section 12's retrying approval take exposes not ready for a not-ready
answer and busy for a busy or lost answer, distinguishing a retry from
an unanswered take for the application's waiting line.

- **Not yet:** 8.2: the link's `expires`, the expiry of the sign-in it
  holds as the last sign-in's answer said it, is not built: the link
  layer drops `SignedIn`'s expiry, and `LinkView` names none.
