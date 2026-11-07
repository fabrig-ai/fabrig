# smith-test-cost: what every task keeps

- **`inputs/request.md`, `For every task of this plan` and `Limits`, hold whole.** Only focused and fuzzy run. Decide, don't ask. A test's helper the change makes unused goes in the same task. Before calling a focused or fuzzy failure "outside the change", run it once on the pristine base.
- **What must not move:** no test leaves its tier to escape the budget, no budget or timeout changes, no build setting changes (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1). A test moves to fuzzy only where its spec says what it checks makes it a fuzzy test.
- **Nothing asserted today may be lost.** A check that moves or merges is still checked; say where in the report.
- **Not yours:** `skein/crates/skein-json` and `skein/testing/skein-heap`, which other plans have. Don't touch them or count on them. Which scenarios run in both placements stays open (`smith/docs/design/testing.md`, 8).
- **No figure is asked of a task.** Nothing here reads a tier's CPU figure, and acceptance names no number. A task may time its own tests to find a cause; the report gives what it saw as a reading, never as acceptance. The gate's records are read after landing.
- **`docs/design/` departs from your text in one way:** your task names the status lines it closes. Delete those lines, nothing else there. The design sections are already written and on the trunk.
- **Keep hunks small.** Other drafts change files beside yours, and each task names the ones it meets.
