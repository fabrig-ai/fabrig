# ctl-waits: what every task holds to

From the request (`temper/docs/plan/ctl-waits/inputs/request.md`), which binds:

- **Failing tests first, in the lowest tier.** No test reaches a real host.
- **Focused and fuzzy suites alone** (`AGENTS.md`, 5). A slow-tier story is written and compiled (`--no-run`), never run.
- **The 30 s stays.** Budgets and timeouts are the user's (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **A stored format or a protocol change is a question first,** so its owner can be told. Formats change in place before v0.1.0.
- **Boundaries:** a generic mechanism goes in skein first, and skein names no temper; jig never names temper. temper/ui owns `temper-client-*` except `temper-client-tools` and `temper-client-shell-cli`: changing another of those is a question.
- **Nothing private and no payload in a line:** no request's words, no secret, no path of a machine. A line names states, stages, durations and counts.
- `walk = false`: temper/qa's next walk is the proof, so no task runs one.
