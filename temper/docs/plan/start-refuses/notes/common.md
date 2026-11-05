# start-refuses: what every task holds to

- **No stored format, protocol or public interface changes.** Any of them is a `Question:`. Formats change in place before v0.1.0, and a store of truly another format stays refused as it is today (`jig/crates/jig-protocol-store/src/open.rs`): leave that alone.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): over budget is red, and your report gives the measurements.
- **fabrig/ui has priority.** Its drafts are changing `temper-client-*`, `temper/tools/gallery`, `temper/tools/journeys`, `temper/tests/client*` and `temper/tests/one-process`. Leave those files alone.
- **The store is ops's.** ops keeps the store that crashed and re-runs the fixed build on a copy of it as the real proof. No task reads, copies or asks for it; a task's proof is its own world's.
- **fabrig names nothing of the deployment** (`AGENTS.md`, 1; `tests/boundary/tests/outward.rs`): no tool, machine, store path or commit of whoever runs it, in code, tests or pages.
