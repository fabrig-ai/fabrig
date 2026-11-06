# read-task-room: a run reads a task's words in its own room

Settled by recommendation, 2026-11-06, on `first-goal-fixes`' second question: a run's `read_task` reads in its own room, not in the clients' window.

`documents.text_bytes` bounds what a client decodes, and `first-goal-fixes` raised it to 512, as the web and terminal clients refuse longer. A run's `read_task` reads through the same window, so a task's spec takes a call per 512 bytes: each a turn at a coding model's prices. A first real goal made 59 such calls at 128. The `tools` client takes 32,768, and a run's tool has its own answer's room (`tool_payload`, `hub.*`).

**The change:** `read_task` as an agent's tool answers up to what the run's reply can hold, bounded by that room and not by `documents.text_bytes`; the clients' documents stay at 512. Read the first-goal-fixes question's words in fab's retired plan, or `temper/crates/temper-init/template/temper.json` and `temper/crates/temper/src/tests/init_clients.rs`, for what's there. Confirm the route first: if a run's reads already pass through a wider path, say so and stop.

A failing test first, in `temper/tests/engine` or lower: a spec larger than 512 bytes comes back in one call to a run, whole, up to the room; a longer one continues by offset, as it does now. The memory a read holds has a bound: say what it comes to. A protocol change is a `Question:`.

One task, temper and perhaps jig (hearth's manifests name no jig crate). fabrig/ui has priority. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). `walk = false`. fabrig/qa's next goal's `read_task` count is the proof.
