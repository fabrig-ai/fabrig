# repair-green: what every task holds to

- **The rule is `temper/docs/design/domain/forge.md`, 8.4, its bullet "A green head ends a CI repair".** Read it first; it binds where a spec's sketch is silent.
- **No stored or wire format changes.** `State`, `Hold`, `ChangeRow` and `temper-protocol-store/schema/**`, `temper-client-protocol-wire/schema/**`, their generated code and goldens stay as they are. `Heard` and `Decision` live in memory. If a task finds it needs a stored field, that is a `Question:`.
- **fabrig/ui has priority.** Leave alone `temper-client-view`, `temper-client-domain`, `temper-client-shell-web`, `temper/tools/gallery`, `temper/tools/journeys`, and the panel's words: the change simply leaves `repairing`, and no word changes.
- **A gate's, a landing's and a resolution's repairs are not touched.** A test that shows one of them changing is a red.
- **Failing test first:** write each story, see it fail for the rule's reason, then build. The report names what each failed on.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): over budget is red, and the report gives the measurements.
- **fabrig names nothing of the deployment** (`AGENTS.md`, 1; `tests/boundary/tests/outward.rs`): no commit, machine or store of the run these came from, in code, tests or pages.
- **A status entry a task closes** is deleted by that task, where its spec says so. That departs from the implementer's text for that entry alone; no other `docs/design/` file is yours.
