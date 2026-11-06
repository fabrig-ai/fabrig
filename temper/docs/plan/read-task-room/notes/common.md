# read-task-room: standing rules

- **What stays:** the clients' documents stay at 512. `documents.text_bytes` in `temper/crates/temper-init/template/temper.json` and `bare.json`, `temper/crates/temper/src/tests/init_clients.rs`, and jig's `reads/task_text.rs` (what a client reads) are not this plan's.
- **No protocol change:** nothing here touches a wire schema, a stored format or a configuration name. `NamedAction` is in-process and never stored: a task read enters before durable-call replay (`temper/docs/design/domain/root.md`, 5.2). If the work seems to need one, that is a `Question:`.
- **The tool's words stay:** `read_task`'s name, schema and description, and `docs/help/`, do not change: `next:` and `from` go on working as they do.
- **fabrig/ui has priority:** this plan touches no client view, shell or look file, and takes nothing from a UI draft's way.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1).
