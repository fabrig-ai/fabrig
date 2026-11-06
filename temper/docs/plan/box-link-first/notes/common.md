# box-link-first: what every task holds to

Each task names this note. The draft is `temper/docs/design/draft/box-link-first.md`; the design it became is cited in each task.

- **Leave fabrig/ui's files alone:** `temper/crates/temper-client-*/**`, `temper/crates/temper-fake/src/**`, `temper/tools/gallery/**`, `temper/docs/plan/ui-final-walk/**`, `temper/docs/plan/ui-chat-draft/**`. That draft has priority and is changing them. A task here that truly needs one of them says so as a `Question:`; its own trunk catches up by merge.
- **A secret stays out of argv, a child's environment, a log and a test's output** (`AGENTS.md`, 3). A test shows a secret absent by looking at the `Spawn` the box asks for, or by comparing inside the test: it never reads `/proc/*/environ` or `ps e`, and it prints no secret, even in a failure's message.
- **Not yours, unless the task says so:** any `docs/design/`, drafts, `docs/help/`. A task that changes `docs/help/` says so in `Owns and may change`.
- **No skein change.** The plan is jig's one surface change and temper. A mechanism skein lacks is a `Question:`.
- **No wire or store format changes** beyond what the task names. A protocol change beyond the draft is a `Question:`.
- **Budgets and timeouts are the user's.** A test that needs more time is a question to ask with measurements, never a raised bound.
