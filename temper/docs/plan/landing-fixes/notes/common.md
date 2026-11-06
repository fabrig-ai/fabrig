# landing-fixes: what every task keeps

- **Reproduce first, in the lowest tier.** The request's causes come from reading a run and its logs, not from a test. Write the failing test before the fix and run it. If a cause is wrong, say so in your report's first lines, and what you found instead.
- **No test reaches a real host.** Fakes and scripted worlds only; a real `git` in a temporary directory is no host.
- **Stay out of `temper-client-*`:** temper/ui owns it. A panel's or `ctl`'s words are not this plan's; report anything you find there under `## Left`.
- **A stored format or a protocol is a question first,** so its owner can be told. This plan needs neither: no task adds a stored field or a wire field. Formats change in place before v0.1.0, once the question is answered.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): no test or tier moves or rises to fit, and no stock limit changes here.
- **`docs/design/` departs from your text in one way:** your task names the `Not yet` lines of a status file that it closes. Delete those lines, nothing else there. The design sections themselves are already written and on the trunk.
- **`walk = false`:** temper/qa's next landing plan is the proof. Nothing here adds a walk or a QA rung.
- **Nearby drafts:** `sign-in-new-tab/return-goes` changes `temper/crates/temper-engine-domain/src/**` and `temper/tests/engine/domain/**`; `model-catalogue-1/stock-seven` and `model-check/trial-charter` change `temper/crates/temper-service/src/tests/stock.rs`. Keep your hunks small and new stories in their own files, so a merge finds them apart.
