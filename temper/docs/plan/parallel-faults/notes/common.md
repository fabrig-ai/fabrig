# parallel-faults: what every task keeps

- **Reproduce first, in the lowest tier.** The request's causes are a driver's reading, not proved. Write the failing test before the fix and run it. If a cause is wrong, say so plainly in your report's first lines, and what you found instead.
- **No test reaches a real host.** Fakes and scripted worlds only; a real `git` in a temporary directory, or the repository's `git-transcripts` tool, is no host.
- **Stay out of `temper-client-*`:** another draft has priority there. A panel's words are not this plan's.
- **Budgets and timeouts of tests are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). No test or tier moves or rises to fit, and no stock limit changes here.
- **Formats change in place** before v0.1.0, and this plan needs none: a new stored field is a question.
- **`docs/design/` departs from your text in one way:** your task names the `Not yet` lines of a status file that it closes. Delete those lines, nothing else there. The design sections themselves are already written and on the trunk.
- **Nearby drafts:** `sign-in-new-tab/return-goes` also changes `temper/crates/temper-engine-domain/src/**` and `temper/tests/engine/domain/**`. Keep your hunks small and your stories in their own files, so a merge finds them apart.
