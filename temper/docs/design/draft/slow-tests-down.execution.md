# slow-tests-down: execution

2026-11-23. How [slow-tests-down.md](slow-tests-down.md) gets built: one milestone, its goals side by side wherever their files overlap little enough that a conflict is a simple one, and in a chain where two goals would change the same seam or test file (the user, 2026-11-23: "execute in parallel what's possible"). It designs nothing: each goal writes its design from the draft and the surveys as it runs, checked at its own commit. Plans use the focused and fuzzy suites alone.

## 1. The milestone and its goals

One milestone, *slow tests at the gate*, of eighteen goals. Each lands into main on its own: a move is whole in one goal, its slow tests deleted with their holders in place.

| Goal | Draft parts | Trees | Needs | State |
|---|---|---|---|---|
| D1, the browser's deadlines: every browser step's expiry through the starvation rule, and a red story's wall bounded | 7 | skein, jig, temper | none | not started |
| D2, temper's real Forgejo paths and repeated command paths shortened to their boundary, in `temper-box::process` and `proof_live` | 5.1 | temper | none | landed |
| D3, temper's public board, history, reading, workers and journeys stories: their data to the client and domain worlds, their geometry left in Chromium | 5.1 | temper | none | not started |
| D4, skein's browser kit's bookkeeping, patch shapes and refused writes below the browser | 5.1 | skein | D1 | not started |
| D5, jig's browser duplicates, provider lifetimes and process controls | 5.1, 5.3 | jig | none | not started |
| D6, hearth's conformance duplicates and its builder world's render, diff, outcome and store matrix | 5.1, 5.2 | hearth | none | not started |
| D7, `temper-box::suite`'s admission, configuration, startup and support checks | 5.2 | temper | none | not started |
| D8, `temper-box::suite`'s fake backends, relay, forge setup, landing launcher and listeners | 5.2 | temper | D7 | not started |
| D9, `skein-codegen::suite`'s tables to its step tests, with an in-memory tree for the patch commands | 5.2 | skein | none | not started |
| D10, `github-conformance::suite` to step and protocol tests on the fake, with a supplied file tree for its journal | 5.2 | skein | none | not started |
| D11, temper's QA tool and CLI policy to a QA domain world and command-table steps | 5.2 | temper | none | not started |
| D12, `smith-bench::suite`, smith's store and native setup | 5.2, 5.3 | smith | none | not started |
| D13, smith's shipped CLI, with a file-operation seam around its finalizer | 5.3 | smith | none | not started |
| D14, temper's shells: forge setup, box, common, TUI, service, fetch and engine shells | 5.3 | temper | D8 | not started |
| D15, temper's journey tools and its end-to-end binaries' tables | 5.3 | temper | none | not started |
| D16, skein's shell and world harness, and skein's remaining small groups | 5.3, 5.4 | skein | none | not started |
| D17, the real-loop world's remaining stories and every single-row move left | 5.4 | temper | none | not started |
| D18, the cluster, restore, checkpoint and start-over stories' policy, once failure-worlds' restart faults have landed | 6 | temper | D2, failure-worlds' F8 | not started |

## 2. Order

- **Side by side,** fourteen goals at once: what they share is a suite's module list or a world's new test files, conflicts each goal settles as it lands.
- **Chained where a seam or a file is shared:** D4 after D1, both in skein's browser harness; D8 after D7, both adding seams to the box's shell and fixtures; D14 after D8, the box shell's again; D18 after D2, both in `temper-box::process`, and after F8 (`failure-worlds.execution.md`, 1) for the restart faults it moves onto.
- **The budget still holds** (the draft's section 8): a goal whose landing would take a layer past its accepted level stops before landing with its figures for the user.
- **After landing is held meanwhile** (the user, 2026-11-23) and resumes on the user's word once the moves have landed: the real pairs first, then the slow pairs as the goals clear them, `browser-skein` and `browser-jig` after D1, D4 and D5, `conformance-hearth` after D6, `browser-temper` and `process-temper` after D1 to D3, each turned on once it is green on main.
