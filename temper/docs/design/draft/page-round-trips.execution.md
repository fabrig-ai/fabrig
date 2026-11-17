# page-round-trips: execution

2026-11-17. How [page-round-trips.md](page-round-trips.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole, with its own tests and fixtures. Each is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

Each moves the client's wire, so at dev.fabrig.ai's redeploy after it lands an open page is refused by name and reloads (`page-round-trips.md`, section 8). None needs an operator's step.

| Milestone | Draft parts | Needs | State |
|---|---|---|---|
| R1, one exchange, many messages: jig's HTTP mapping's batch route, its replies streamed as made, each message's answer its own, a batch admitted as one call and one exchange per sign-in; the client's link sending one whenever more than one read, watch or unwatch is ready, in the page's order; a stream opened naming the one it replaces, with no `stream-gone` before it; the versions moved | 3, 7, 8 | — | not started |
| R2, parts on rows: jig's task query naming an application's part, made per row by the engine's root through its connector, carried as the family's bytes, delivered again with its row, cut to its kind's bytes; temper's goal, chat and milestone parts; the `changes` subject's rows naming their goal, chat and milestone; the trials' rows their running trial's line | 4, 7, 8 | — | not started |
| R3, one fact, one read: the stream's hello carrying `me` and every stream watching it; the party's own and the deployment's reads and watches sent without waiting for `me`; a narrower watch served by a wider one of the same subject; no reread of an ended group on a list's first delivery; a setting read with the settings' watch at one state | 5, 7, 8 | — | not started |
| R4, Work on its budget: Work's, Goals' and Changes' reads through R1 to R3, with no read per row and nothing opened that the page doesn't draw; its sections drawn top down as their lists come, rows whole when first drawn; the budget's focused test on a small project and on one at the client's bounds, its constant beside the page | 2, 6, 7 | R1, R2, R3 | not started |

## 2. Order

- **Eager.** R1, R2 and R3 start at once, each its own goal, in parallel. R4 starts once all three have landed.
- **No priority mark.** v0.1.0's critical pieces are the migration and UI with looks; these plans start when places are free and marked work isn't about to arrive.
- **Home after,** by a draft of its own (`page-round-trips.md`, section 10): R2 and R3 already cut its goals', chats' and milestones' reads.
