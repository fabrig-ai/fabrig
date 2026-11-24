# slow-tests-down: execution

2026-11-23. How [slow-tests-down.md](slow-tests-down.md) gets built: one milestone, its goals side by side wherever their files overlap little enough that a conflict is a simple one, and in a chain where two goals would change the same seam or test file (the user, 2026-11-23: "execute in parallel what's possible"). It designs nothing: each goal writes its design from the draft and the surveys as it runs, checked at its own commit. Plans use the focused and fuzzy suites alone.

## 1. The milestone and its goals

Eighteen goals, started on 2026-11-23 on the user's approval, each on its own rather than in a milestone, which the party starting them may not make; their order is in their dependencies. Each lands into main on its own: a move is whole in one goal, its slow tests deleted with their holders in place.

| Goal | Draft parts | Trees | Needs | State |
|---|---|---|---|---|
| D1, the browser's deadlines: every browser step's expiry through the starvation rule, and a red story's wall bounded | 7 | skein, jig, temper | none | landed, task 13857 |
| D2, temper's real Forgejo paths and repeated command paths shortened to their boundary, in `temper-box::process` and `proof_live` | 5.1 | temper | none | landed, task 13858 |
| D3, temper's public board, history, reading, workers and journeys stories: their data to the client and domain worlds, their geometry left in Chromium | 5.1 | temper | none | landed, task 13859 |
| D4, skein's browser kit's bookkeeping, patch shapes and refused writes below the browser | 5.1 | skein | D1 | started, task 13870 |
| D5, jig's browser duplicates, provider lifetimes and process controls | 5.1, 5.3 | jig | none | landed, goal 13860: its design in jig's `protocol/browser/testing.md`, `protocol/client/openid.md` and `protocol/service-testing.md`, 1, with smith's `shell/README.md`, 2.6 for the seam it adds; each moved test's holder in `jig/docs/development/slow-tests-holders-2026-11-23.md` |
| D6, hearth's conformance duplicates and its builder world's render, diff, outcome and store matrix | 5.1, 5.2 | hearth | none | landed, task 13861 (`hearth/docs/design/testing/README.md`, 2.5, 2.7 and 3.2) |
| D7, `temper-box::suite`'s admission, configuration, startup and support checks | 5.2 | temper | none | started, task 13862 |
| D8, `temper-box::suite`'s fake backends, relay, forge setup, landing launcher and listeners | 5.2 | temper | D7 | started, task 13871 |
| D9, `skein-codegen::suite`'s tables to its step tests, with an in-memory tree for the patch commands | 5.2 | skein | none | landed, task 13856 |
| D10, `github-conformance::suite` to step and protocol tests on the fake, with a supplied file tree for its journal | 5.2 | skein | none | started, task 13863 |
| D11, temper's QA tool and CLI policy to a QA domain world and command-table steps | 5.2 | temper | none | landed, task 13864 |
| D12, `smith-bench::suite`, smith's store and native setup | 5.2, 5.3 | smith | none | started, task 13865 |
| D13, smith's shipped CLI, with a file-operation seam around its finalizer | 5.3 | smith | none | started, task 13866 |
| D14, temper's shells: forge setup, box, common, TUI, service, fetch and engine shells | 5.3 | temper | D8 | started, task 13872 |
| D15, temper's journey tools and its end-to-end binaries' tables | 5.3 | temper | none | started, task 13867 |
| D16, skein's shell and world harness, and skein's remaining small groups | 5.3, 5.4 | skein | none | landed, task 13868: `drive`'s loop over a supplied kernel and clock, startup's decisions and root preparation over supplied facts (`skein/docs/design/runtime/shell.md`, 8 and 11), the hosted kill report without a kernel (`skein/docs/design/testing/service-tiers.md`, 1), the live suite's login admission offline (`skein/docs/design/llm/README.md`, 5), the KV writer's golden bytes, the bundle's routes and the throwaway's controls below the real loop (`skein/docs/design/data/kv.md`, 9; `skein/docs/design/ui/web/testing.md`; `skein/docs/design/testing/fakes/throwaway.md`, 1), the fail-stop tail at its step, and the world harness's and io's duplicates retired; the real witnesses that the draft keeps stay (`slow-tests-down.md`, 4) |
| D17, the real-loop world's remaining stories and every single-row move left | 5.4 | temper | none | started, task 13869 |
| D18, the cluster, restore, checkpoint and start-over stories' policy, once failure-worlds' restart faults have landed | 6 | temper | D2, failure-worlds' F8 | started, task 13873 |

## 2. Order

- **Side by side,** fourteen goals at once: what they share is a suite's module list or a world's new test files, conflicts each goal settles as it lands.
- **Chained where a seam or a file is shared:** D4 after D1, both in skein's browser harness; D8 after D7, both adding seams to the box's shell and fixtures; D14 after D8, the box shell's again; D18 after D2, both in `temper-box::process`, and after F8 (`failure-worlds.execution.md`, 1) for the restart faults it moves onto.
- **The budget still holds** (the draft's section 8): a goal whose landing would take a layer past its accepted level stops before landing with its figures for the user.
- **After landing is held meanwhile** (the user, 2026-11-23) and resumes on the user's word once the moves have landed: the real pairs first, then the slow pairs as the goals clear them, `browser-skein` and `browser-jig` after D1, D4 and D5, `conformance-hearth` after D6, `browser-temper` and `process-temper` after D1 to D3, each turned on once it is green on main.
