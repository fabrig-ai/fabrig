# focused-pyramid: the focused suite's top back within its guides

Draft, 2026-11-15. The focused suite is meant to be quick feedback: most tests do no I/O and are bound by the CPU, and few expensive ones sit at the top (`skein/docs/foundation/testing-strategy.md`, 1 and 8). By count it is still that pyramid. By time it isn't: a few hundred tests at the top take most of it, and temper's worlds there are far past the guides `temper/docs/design/testing/checks.md`, 1 sets them. This draft takes them back within those guides in the strategy's repair order (`skein/docs/foundation/testing-strategy.md`, 8.1): each slow test placed by what it checks, then what stays made faster, every check kept.

## 1. In one page

- **What the suite costs** (section 2). The CI builder's focused run of 2026-11-15 10:04Z: 36,288 tests, 1,775 s of test time, 1,425 s of CPU, 224 s on 8 lanes, against 123 s on 2026-11-12. 88% of the tests take under 40 ms, together 8% of the time; the 213 over 1 s take 61%.
- **The trees' budgets mostly hold, temper's guides don't.** temper's tree spends 1,049 CPU s of its 1,250, jig's 189 of 215; smith's 38 of 30 is over (`tools/tiers/budgets.toml`). Within temper's budget, its guides: the one-process world spends 435 CPU s where the worlds below the domain together have 8 s, the real loop 126 CPU s where it has 15 s, and 79 of their stories pass a story's 2 s. The engine component's unit tests spend 28.7 CPU s against 20, 336 of its 591 tests past 40 ms.
- **Where it goes** (section 3). The one-process world's and the real loop's existing tests more than doubled since 2026-11-12: each start and each store open costs more, against checks.md's rule that a service builds each of its fixed tables once, and these tests start and open many times.
- **Each slow test placed first** (section 4), only by what it checks: a store mechanic is tested once, below the domains (`upgrades.md`, 13); a behaviour, in the lowest world that owns it (`temper/docs/design/testing/real-loop.md`). Cost never decides a tier.
- **Then the shared costs** (section 5): a store open checks once and refuses a wrong format before its registry; the wire's bridge is built only when a version needs it, its fixed tables once; fixtures prepare what doesn't change once per test; repository checks read the tree in one pass.
- **Every check stays.** Nothing is dropped, no count reduced to save time, no budget or guide raised.
- **The figure after landing** (section 6): the worlds' sums against their guides and each tree's CPU against its budget, from the CI builder's records.

## 2. What the suite costs

The CI builder's focused run of 2026-11-15 10:04Z (main about `a0ed35aa3c`), by each test's time:

| A test's time | Tests | Share | Their time | Share |
|---|---|---|---|---|
| under 40 ms (the strategy's unit-test guide) | 31,861 | 87.8% | 140 s | 7.9% |
| 40 to 200 ms | 3,533 | 9.7% | 304 s | 17.1% |
| 0.2 to 1 s (its focused world test's) | 681 | 1.9% | 243 s | 13.7% |
| 1 to 10 s | 186 | 0.5% | 577 s | 32.5% |
| over 10 s | 27 | 0.1% | 512 s | 28.8% |

- **Against `temper/docs/design/testing/checks.md`, 1** (its figures are for the idle development machine; the CI builder's are read under the gate's load, so they compare in size, not to the second):

| Guide | Its figure | On the CI builder |
|---|---|---|
| worlds below the domain but the real loop, focused | 8 s together | one-process world 435 CPU s, client world 45 |
| the real loop's stories, focused | 15 s together | 126 CPU s |
| a one-process or real-loop story | 2 s | 51 and 28 stories past it |
| the engine component's unit tests | 40 ms each, 20 CPU s | 336 of 591 past 40 ms; 28.7 CPU s |

- **The tests over 1 s** are in 47 binaries; most in `temper-one-process-world`'s suite (52 tests, 480 s), the real loop's (32, 158 s), the boundary checks' `pre_pr` (13, 72 s), `jig-store-world`'s suite (9, 69 s) and `temper-box`'s (16, 31 s). The boundary checks have no budget (99 CPU s).
- **The 8 lanes were 99.8% busy,** so the step's wall is its work divided among them: ordering can't win it back, only less work can.
- **What grew** since the run of 2026-11-12 22:36Z (35,048 tests, 888 s, 123 s on 8 lanes): the one-process world's longest process 71 to 170 s (its existing tests 312 s longer, four new ones 28 s), the real loop's 75 to 158 s (the same 34 tests), `jig-store-world`'s `upgrades::baseline::real_jig_set_versions_match_each_level` 2.9 to 52.5 s, the boundary checks 87 to 156 s.

## 3. Where it goes

- **Starts and opens cost more.** Redeploys' R4 (`c0082e8354`) builds the wire bridge and the spoken versions' metadata when a worker's components are made, even on a current-version link (`temper/crates/temper-protocol-worker-wire/src/application.rs`, and jig's `jig/crates/jig-protocol-worker-wire/src/application.rs`). The store's registry at open (`16a28b677c`, `d8a15fa5c2`, `2517c5e839`; `jig/docs/design/protocol/store/layouts.md`, 5) parses the families' schemas, computes their layouts and checks the bounds at every open, several times over: `plan::bounds` is reached from the registry's admission, from patch planning and from readiness, and a schema `Binary::new` has parsed is parsed again. Part of R4's first cost is already repaired (`da832f69c2`, `3b85d89fef`, `f1e11fb4c5`).
- **The tests at the top start and open many times.** The one-process world's upgrade stories each run a story, settle it, upgrade, cut the power at one write and start again; its power-loss and kill stories do the same at other steps. The real loop starts the service as it ships for each of its 34 stories.
- **One test opens 56 times.** The store's baseline test opens seven levels by eight families; `jig/crates/jig-protocol-store/src/open.rs` builds the registry before `format::check_versions`, so the 49 opens meant to be refused pay it too.
- **Reconnects rebuild what doesn't change.** `jig-engine-workers-world`'s `relinks::a_worker_lost_fifty_times_each_way_is_taken_back_each_time` makes a worker's statement for each of its 400 connections, and since R4 each makes the kept versions' layouts (`jig/tests/engine-workers/src/fixtures.rs`); `jig-worker-link-world`'s fixtures do the same.
- **Repository checks read the tree once per name.** `cli_names` searches every tracked file once for each declared name, across 97,000 tracked files (52,900 on 2026-11-11). `pre_pr` builds a scratch repository and runs the driver for each row of its tables.

## 4. Each slow test placed by what it checks

The first of the strategy's three questions (`skein/docs/foundation/testing-strategy.md`, 8.1), for every focused test past its guide, starting with section 2's binaries. A test moves only if what it checks belongs in another tier: cost never decides a tier (`temper/docs/design/testing/checks.md`, 1).

- **A store mechanic is tested once, below the domains** (`upgrades.md`, 13). Power lost at each step of the copy, each commit of the patching, the manifest and the switch is the store's guarantee, checked in `jig-store-world`'s suite on the simulated disk, where the store's generation and patch scenarios are, a few cuts focused and the sweep fuzzy (`jig/docs/design/domain/testing.md`, 9). The one-process world keeps what its design gives it: its story through an upgrade, and power lost at a drawn moment from the copy to the wires' commit, sweeps of drawn kills being fuzzy tests (`temper/docs/design/testing/README.md`, 2.5). Its tests today cut at four named writes, each a whole story.
- **A behaviour in the lowest world that owns it** (`skein/docs/foundation/testing-strategy.md`, 9). The real loop is smoke, one end-to-end path per feature, each behaviour and corner case tested in the lower world that owns it, never there (`temper/docs/design/testing/real-loop.md`): each of its stories that checks such a behaviour moves to that world, and the real loop keeps its path. The one-process world's stories meet the same rule against temper's system and domain worlds.
- **Where a test moves, the design that places it moves with it,** in the same change: `temper/docs/design/testing/README.md`, 2.5, for the one-process world, `temper/docs/design/testing/real-loop.md`, `upgrades.md`, 13, and `jig/docs/design/domain/testing.md`, 9.
- **A test that belongs where it is stays,** and is made faster (section 5); if neither brings it within its guide, its measure goes to the user (the third question).

## 5. The shared costs

The second question, for what section 4 leaves:

1. **A store open does its checks once.** One open carries its checked set and metadata through admission, patch planning and readiness, rather than checking the bounds again at each; `Binary::new` keeps the schema it parsed; the families' parsed texts and computed layouts are reused within the open and dropped when it settles. No cache outlives an open. Every check the open makes stays: stored text digests, persisted fingerprints, malformed declarations, held depth, each conversion before a write. Files: `jig/crates/jig-protocol-store/src/open.rs`, `jig/crates/jig-protocol-store/src/patching/plan.rs` and `types.rs`, `jig/crates/jig-protocol-store/src/registry.rs` and its `registry/` module's `texts.rs`, `rows.rs` and `check.rs`.
2. **A wrong format is refused before the registry.** After the set's checks and the format it needs, `format::check_versions` refuses before `registry::open` runs, keeping the refusals' order (`jig/docs/design/protocol/store/opening.md`, 1.8) and refusing before any write. The baseline test keeps all its levels and families.
3. **The wire's bridge when a version needs it, its fixed tables once.** The spoken range and fingerprints stay eager and are built once per service, as checks.md's rule asks; the application's bridge is made only when the agreed version needs a conversion, as jig's own bridges already are, with root and admission still checked. Fingerprints that are constants may come from the codec generator as a table, with a drift test against the parsed schemas. Files: `temper/crates/temper-protocol-worker-wire/src/application.rs`; in `jig/crates/jig-protocol-worker-wire/src/`, `application.rs`, `translation.rs`, `spoken.rs` and `kinds.rs`; the generator's `skein/testing/skein-codegen/src/patch/module.rs`.
4. **Fixtures prepare what doesn't change once per test.** A worker's statement bytes are made once per fixture and version and copied into each fresh peer; every connection, loss and assertion still runs (`jig/tests/engine-workers/src/fixtures.rs`, `jig/tests/worker-link/src/fixtures.rs`). A settled story the one-process world's cut cases start from may be a constant fixture, a golden image made once by its own drift test (`skein/docs/foundation/testing-strategy.md`, 8), each case still running its own upgrade, cut and recovery.
5. **Repository checks read the tree once.** `cli_names` searches all declared names in one pass, and the boundary checks that walk the same tracked files share one walk per binary (`tests/boundary/tests/cli_names.rs`, `skein_worlds.rs`, `shared_test_code.rs`). `pre_pr`'s rows that classify results move to the driver's Python unit tests (`tools/pre-pr/test_pre_pr.py`), its end-to-end tests (`tests/boundary/tests/pre_pr.rs`) keeping one per external path.

## 6. The figure after landing

- **The worlds' sums against their guides** (section 2's second table) and each tree's focused CPU against its budget, from the CI builder's records after each milestone lands, by whoever runs the builder. A change to these worlds gives the sums before and after it, as checks.md asks.
- **The fuzzy suite's** too, against its budgets: section 4 may move cuts into it.
- **Not forecast.** The survey behind section 3 ran six tests by name, at R4 and its parent; the rest is read from the run's records and the history. What section 5 wins back is bounded by section 2's growth, not promised by it.

## 7. Not in this plan

- **The gate holding the budgets.** The gate's runner prints "no budget set": no suite is red for being over. Holding them is the CI's configuration, outside this repository.
- **Raising a budget or a guide,** which is the user's alone, and dropping or shrinking any test.
- **The fuzzy suite's own repairs,** but for what section 4 moves into it.

**Tests stay cheap:** the suites are the check, and a milestone runs a test it moves or speeds by name alone, at most. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
