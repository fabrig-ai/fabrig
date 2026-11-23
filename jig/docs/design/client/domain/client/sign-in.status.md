# sign-in: status

2026-11-23. Where the tree's code lags or departs from [sign-in.md](sign-in.md).

Section 1 has no known gap in carrying `link_used`, `link_expired` and
`link_unknown` as `not_signed_in`, telling the application, stopping retries
and clearing the reason on the next sign-in. Falling back to a live
transport sign-in after such an answer remains deferred in `README.md`, section 15.

`vocabulary.md`, sections 1.1 and 1.2, and `sign-in.md`, section 1 have the approval events and answers in linked
and unlinked modes, `With::Approval`, the take retried each second while
waiting, and its terminal signing outcomes. The secret stays in memory,
with secret-safe diagnostics, and never enters the small store.
Section 1's retrying approval take exposes not ready for a not-ready
answer and busy for a busy or lost answer, distinguishing a retry from
an unanswered take for the application's waiting line.

- **Not yet:** 1: no story relays a person's decision with their own credential.
