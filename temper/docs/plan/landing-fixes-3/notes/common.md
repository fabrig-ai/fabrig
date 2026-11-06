# landing-fixes-3: the rules every task keeps

- **Failing test first, in the lowest tier.** The test goes in before the fix, and the report names how it failed. No test reaches a real host.
- **Focused and fuzzy suites alone** (`AGENTS.md`, 5). A process, browser or live story isn't run here.
- **Stay out of `temper-client-*`:** temper/ui owns them. Nothing here changes a stored format or a protocol: if it would, that's a question first, so that its owner can be told.
- **`landing-fixes-2` is beside this plan.** `saved-close`, a repair's brief and its words are its, and a catch-up's likely are too. Touch none of them, nor the `repair-cause` lines of `forge_route.rs`.
- **Budgets and timeouts are the user's.** No task raises one.
- **`walk = false`:** temper/qa's next real landing is the proof. No task claims a real run read these words.
- **Formats change in place before v0.1.0,** but none of these tasks changes one.
