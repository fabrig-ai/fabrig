# first-goal-fixes: what every task holds to

- **No protocol or public interface change, and no schema edit.** `temper-protocol-store/schema/**`, `temper-client-protocol-wire/schema/**`, their generated code and goldens stay as they are in this plan: the ui drafts are changing them. A need for one is a `Question:`.
- **fabrig/ui has priority.** Leave alone what the ui drafts are changing: `temper-client-view`, `temper-client-domain`, `temper-client-shell-web/temper.css`, `temper/tools/gallery`, `temper/tools/journeys`, `temper/tests/client*`. A task's `Owns and may change` names the only exceptions, and how. Expect fab's catch-ups.
- **`brief-failure` owns** `temper-engine-domain-forge/src/brief.rs` and a retried review's words (`forge_route.rs`, `review_words`, `review_failure`). Leave them alone.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): over budget is red, and the report gives the measurements.
- **Nothing here reaches a real host in a test.**
- **fabrig names nothing of the deployment** (`AGENTS.md`, 1; `tests/boundary/tests/outward.rs`): no commit, machine or store of the run these came from, in code, tests or pages. The request's commit is not a fixture.
- **A status entry a task closes** is deleted by that task, where its spec says so. That departs from the implementer's text for that entry alone.
