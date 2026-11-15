# kept-change-answers: status

2026-11-15. Where the design lags or departs from [kept-change-answers.md](kept-change-answers.md).

The design carries the draft: section 1 in `../domain/forge/change/questions.md`, 1 (*Relayed at once, kept or not* and *Held only for message room*), section 2 in jig's `domain/engine/views.md`, 3 (*its question*), `client/domain/contract/documents.md` and, for temper's pages and `ctl`, `../client/pages/goal.md` and `../client/ctl/results.md`.

- **Design lags:** none.
- **Design departs:** 4: the relay's pure function (`temper-engine-domain-forge-change`'s `relay`) takes no keptness at all, so its unit tests pin an answer delivered with no input that could hold it, and the zero-room hold and retry; delivery while a dependency is pending is pinned by the root's focused story in `forge_routes/questions.rs`, where dependencies exist. Keeping a keptness input the relay ignores would only invite the old guard back. The goal's designer, 2026-11-15, keeping the draft's intent.
- **Open:** 1: an answer that needs a dependency the requester adds with it now reaches the producer before that dependency lands, so the stock coordinator charter's prerequisite line (`../../charters/stock/coordinator.md`: make it, add it to the change's `after`, "and answer that it waits for it") no longer makes the producer wait. Asked of the user, 2026-11-15, recommending that the coordinator subscribe to the prerequisite and answer once it has landed.
