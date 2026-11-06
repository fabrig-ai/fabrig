# resume-told: what every task keeps

- **Reproduce first, in the lowest tier.** The request's causes come from reading a run, not from a test. Write the failing test before the fix and run it. If a cause is wrong, say so in your report's first lines, and what you found instead.
- **No test reaches a real host.** Fakes and scripted worlds only.
- **Stay out of `temper-client-*`:** temper/ui owns it. Report anything you find there under `## Left`.
- **A stored format or a protocol is a question first,** so its owner can be told. This plan needs neither: no task adds a stored field or a wire field. `from_saved` keeps its type and its name, and says what it always said.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): no test or tier moves or rises to fit, and no stock limit changes here.
- **`docs/design/` departs from your text in one way:** your task names the `Not yet` lines of a status file that it closes. Delete those lines, nothing else there. The design sections themselves are already written and on the trunk.
- **`walk = false`:** temper/qa's next replay is the proof. Nothing here adds a walk or a QA rung.
- **Nearby drafts:** `model-catalogue-1/stock-seven` and `model-catalogue-1/configuring-told` change `temper/crates/temper-service/src/tests/stock.rs`, `stock_brief.rs` and `temper/tests/engine/domain/**`. Keep your hunks small and new stories in their own files, so a merge finds them apart.
