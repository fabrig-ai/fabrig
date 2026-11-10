# client: status

2026-11-08. Where jig's code lags or departs from [README.md](README.md).

Unlisted sections have no known gap, including `watches.md`, 2's `watches_full`
fact for `me` and application watches through retries and recovery.

`watches.md`, section 2's `LinkView.expires` has no known gap: link and service
sign-ins and approval takes retain the answer's wall expiry, a held
credential starts with none, and dropping the sign-in clears it.
Keeping that expiry across an application's restart remains the
application's responsibility.

`conversations.md` and `README.md`, section 11 have no known gap in decoding a turn read alone:
`Read::Turn` uses the archive's byte bound, twice that bound for text
and smith's block ceiling and message ceiling plus one system message.
Opening turns include their supplied system text; version-6 archival reads
keep its absence. A call input past `turn_text`
is retained whole and unmarked, so tools can page through its last byte.
Turns carried by an activity page still use `turn_text`; a turn past
the archive's byte bound remains `README.md`, section 15's deferred read case.

Section 9 has no known gap in carrying `link_used`, `link_expired` and
`link_unknown` as `not_signed_in`, telling the application, stopping retries
and clearing the reason on the next sign-in. Falling back to a live
transport sign-in after such an answer remains deferred in section 15.

`conversations.md`'s decoded model and effort, their window weight, `latest_llm`
and `caught_up` are present. The decoder keeps the names outside the text
budget; the thread's table covers unread activity, older pages and passing
over undecodable or nameless turns. `README.md`, section 15 defers latest-run metadata
independent of readable turns.

- **Not yet:** 9: no story relays a person's decision with their own credential.
  14, 16: the one-shot mode and its story, and a service in a host or world
  (the client takes `With::Service`).
- **Departs:** 13: a turn answer that `translate::turn::inspect` cannot decode
  ends malformed before the link emits `Fact::UndecodableDocument`
  (`jig-client-protocol-link/src/component.rs`, `read_answer`). Other read
  answers that cannot decode emit the fact. The turn's early return remains
  separate from that shared decode arm; no reason for omitting the fact is
  recorded. Seen by reading.

Sections 4.1, 4.2 and 9 have the approval events and answers in linked
and unlinked modes, `With::Approval`, the take retried each second while
waiting, and its terminal signing outcomes. The secret stays in memory,
with secret-safe diagnostics, and never enters the small store.
Section 9's retrying approval take exposes not ready for a not-ready
answer and busy for a busy or lost answer, distinguishing a retry from
an unanswered take for the application's waiting line.
