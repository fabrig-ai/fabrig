# page-round-trips: execution

2026-11-17. How [page-round-trips.md](page-round-trips.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole, with its own tests and fixtures. Each is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

Each moves the client's wire, so at dev.fabrig.ai's redeploy after it lands an open page is refused by name and reloads (`page-round-trips.md`, section 8). None needs an operator's step.

| Milestone | Draft parts | Needs | State |
|---|---|---|---|
| R1, one exchange, many messages: jig's HTTP mapping's batch route, its member and byte bounds checked by the engine, its replies streamed in the order released, each member's answer its own, a batch cut short losing only its unanswered members; a batch admitted as one call and one exchange per sign-in, each member reserved as alone against per-sign-in bounds of reads and watches; skein's browser fetch and jig's native connections giving an exchange's body in pieces; the client root settling an iteration before its link sends, the link sending one batch while it holds more than one message, in the order asked; a stream opened naming the one it replaces, with no `stream-gone` before it; the versions moved | 3, 7, 8 | — | done, by goal 6234 (jig's `protocol/client/wire.md`, 2; `protocol/client/default-http.md`, 6; `client/domain/root.md`, 3; skein's `ui/web/README.md`, 8), at jig's client wire 32, temper's family unmoved, none of its records changing |
| R2, parts on rows: jig's task query naming an application's part by owner, kind and version, made per row by the engine's root through its connector as reads are made, carried in its owner's envelope, failed or not served typed; each kind's inputs marking its rows, a note made, edited or deleted among them; a list's ended group's mark, given with its ended read and every delivery, and the client reading the ended group again only when it moves; temper's goal, chat and milestone parts, bounded as `page-round-trips.md`, section 4, bounds them; the `changes` subject's rows naming their goal, chat, milestone and a landing's signers; the trials' rows their running trial's run wait | 4, 5, 7, 8 | — | landed |
| R3, `me` with the stream, and one fact, one read: the hello holding the served ranges, then `me` in its envelope, and every stream watching it; the party's own and the deployment's reads and watches sent without waiting for `me`; a narrower watch served by a wider one of the same subject; the needs' selection read again only as its marks move; a setting's read answering with its revision, read again only for a newer one | 5, 7, 8 | — | landed, goal 6236 |
| R4, Work on its budget: Work's, Goals' and Changes' reads through R1 to R3, with no read per row and nothing opened that the page doesn't draw; its sections drawn top down as their lists come, rows whole when first drawn; the budget's focused tests on a small project and on one at the client's bounds, into each filter, and on a reload with kept requests, its constant beside the page | 2, 6, 7 | R1, R2, R3 | not started |

## 2. Order

- **Eager.** R1, R2 and R3 start at once, each its own goal, in parallel. R4 starts once all three have landed.
- **No priority mark.** v0.1.0's critical pieces are the migration and UI with looks; these plans start when places are free and marked work isn't about to arrive.
- **Home after,** by a draft of its own (`page-round-trips.md`, section 10): R2 and R3 already cut its goals', chats' and milestones' reads.
