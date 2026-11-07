# skein-limits-rows: a margin under skein's focused limit

skein's focused suite has a limit of 30 CPU s and reads 28.6 to 30.0, in 7,730 tests, on its way up by about a second a day (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1). The limit now acts at the merge gate. A suite this close pays a second read on many batches, about 47 s each, and its signal flips. This plan makes **a margin of at least 2.5 CPU s,** by two causes the last repairs left.

## What's wanted

**1. The fake's access judge stops signing.** In `skein-forge-machine-world`'s `hand` tests, two RS256 signings cost 22,621,385 instructions, 73% of the largest sample (`skein/docs/development/skein-limits-heavy.md`, "Found and not fixed"). One is the product's and stays, as does parsing at each signing (`skein/docs/design/tls.md`, 4). The other is only the access judge checking a key: a judge that parses without signing removes it, and every test using that judge gains. Check what else pays for it, such as `skein-fake-forge-github` and `github-conformance`, and fix it once. A wrong key must still be judged wrong, shown by a test that fails first if the check is removed.

**2. Rows of one table test, for cases of one behaviour.** Section 8's rule: "cases of one behaviour that each cost less than that (a golden value per type, a refusal per limit, a check per source file) are rows of one table test, which checks every row and names each that fails". **What a fold saves is the binary's floor per test folded,** not the test's own work, so go where the floor is highest and the cases are most alike. Medians of three gate runs:

| Binary | Module | Tests | Median ms | Binary's floor, ms |
|---|---|---:|---:|---:|
| `skein-forge-machine-world::suite` | `hand::github::*`, `hand::forgejo::*` | 387 | 4.3 | 2.6 |
| `skein-forge-setup-world::suite` | all | 153 | 5.5 | 2.5 |
| `skein-fake-forge-github` | all | 198 | 4.0 | 1.9 |
| `skein-forge-setup` | all, `tests::host` 94, `host` 51, `tests::reachable` 38 | 552 | 2.1 | 1.9 |
| `skein-fake-forge-forgejo` | all | 227 | 2.2 | 1.8 |
| `skein-forge` | `forgejo` 247, `github` 231 | 510 | 1.9 | 1.7 |
| `skein-fake-forge-domain` | all, `tests` 69, `tests::promises` 48, `tests::actions` 34 | 246 | 1.8 | 1.6 |
| `skein-codec` | `patch` 130, `schema` 34 | 204 | 1.6 | 1.5 |
| `skein-oauth`, `skein-oauth-accounts` | `machine_tests` 74, `tests` 48 | | 1.7 | 1.6 |
| `skein-http`, `skein-forge-client` | `tests::server` 73, `tests` 61 | | 1.6 | 1.5 |

Folding about 1,000 tests here makes about 2 CPU s, by the floors above. Largest gain first, and stop when you reach it or the next group isn't one behaviour.

**A fold is honest or it isn't done.**
- **One behaviour only.** Rows share a name that says what they all show. Cases that show different behaviours stay apart, whatever they cost. Say which groups you left, and why, in one line each.
- **Every row runs, and a failure names its row.** The table checks them all rather than stopping at the first, and a row's name keeps what the old test's name said. Use a helper the tree already has, if there is one.
- **A row's setup stays a row's.** A fixture that a test built for itself is still built per row, unless sharing it can't change what's shown.
- **A table stays within section 8's guides.** If one would pass its guide, make it two tables.

**3. Report before and after,** in `plan.md`: tests per module, the instruction counts for what item 1 changes, and the groups left alone. Don't quote a suite total: the gate reads that after landing.

At most four tasks, one for item 1 and up to three for item 2, with one crate's folds in one task. `walk = false`. Effort medium.

**Stay out of** `skein-ui`, `skein-ui-web`, `skein-browser`, `skein-fake-person-browser` and their tests, which other plans are changing. No test changes suite in this plan. **Overlaps:** other plans add tests to these crates. Expect catch-ups.

## Sources

Read by section, too large to be sources: `skein/docs/foundation/testing-strategy.md`, 8 and 8.1; `skein/docs/development/skein-limits-heavy.md`, "Found and not fixed".

- `skein/tests/forge/machine/src/hand.rs`
- `skein/crates/skein-tls/src/rs256.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver.
- **Find a cost's cause with a profile before changing it** (section 8, "A slow test is understood, never hidden"). `valgrind --tool=callgrind` on one test by name gives instruction counts that a loaded machine doesn't move; quote them before and after. CPU seconds on these machines swing by a third or more between runs.
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized. Budgets are the user's.
- **Nothing is lost.** Every assertion stays, as a row or as a test. No assertion weakens.
- **Decide, don't ask.** A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
