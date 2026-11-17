# test-budgets: the suites' cost held at the gate, layer by layer

Draft, 2026-11-16, with the user; section 3's delivery check, 11-17. The testing strategy says what the focused and fuzzy suites may cost (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1), and nothing holds it: the gate reads no budget, and a tree's one figure lets the top of its pyramid take the room of its bottom. This draft puts hard guardrails on both suites: CPU budgets by layer of the pyramid, held at the gate; a reference for each test; a wall limit at the gate that is red; and budgets that start at today's figures and only fall, towards an ideal estimated from a lower bound.

## 1. In one page

- **The gate holds the budgets** (section 2). Today it holds none: `tools/tiers/budgets.toml`'s figures for the gate are read by no run, each run's summary says "no budget set", and four trees' suites are over theirs unseen. The CI builder's figures are steady enough to hold: a layer's CPU moves by about 1% from one gate to the next.
- **CPU is the budget, and the gate is its judge** (section 3). User plus system time per process, summed by layer. Over a budget is red at the gate. A delivery check on a builder that keeps main's ledgers holds it too, and refuses the delivery, so a goal hears it before its pull request (the user, 11-17). Any other run (a builder's, anyone's own, a delivery check without the ledgers) prints the same comparison as a warning, since another machine's CPU isn't the gate's (the user, 11-16).
- **A budget for each layer** (section 4). Each tree's focused and fuzzy suites are split into the tiers of the strategy's section 2, with the repository's checks apart; every test binary belongs to one layer, by a table a boundary check keeps whole.
- **A partial run is judged whole** (section 5). The gate runs the packages a change reaches, and takes the others' figures at main from a ledger that every landed gate writes. A layer is red when the change takes it past its budget.
- **Each test has a reference** (section 6): its layer's per-test guide, from the strategy. A new test is measured alone and stays within it; an existing one stays within the larger of it and half again its own figure at main. A test past its reference is red unless a line beside the budgets gives its figure and its reason, which the change's review judges; it still counts in its layer.
- **Wall time: strict at the gate, generous elsewhere** (section 7). A gate run past its wall limit reruns once, and is red if it passes the limit again without being starved. Everywhere else a backstop three times higher stays a starved warning.
- **Start at today's figures, then only fall** (section 8). Each budget starts at the gate's figure of 11-16 plus a tenth, raises and cuts alike (the user, 11-16). After each landing a budget falls to its layer's figure at main plus a tenth, where that is lower. Budget moves down the pyramid with the tests a change moves, never up; a raise stays the user's.
- **An ideal beside each budget** (section 9), reported, never enforced. temper's focused suite: 100 CPU s, from a lower bound of 80 to 100 s for its 13,529 tests, against 1,009 s today; its fuzzy suite, the 180 s it has. jig's focused suite: the strategy's 30 s, which its own lower bound of 26 s confirms. The repository's checks: 30 s.

## 2. Where it stands

- **The gate holds no budget.** `tools/tiers/budgets.toml` has a figure for each tree's focused and fuzzy suites, under `[gate.<tree>.<suite>]`. `tools/tiers/cpu.py` checks their shape, and nothing else reads them: each run's summary says "no budget set", and the gate's runner, outside this repository, reads none (`focused-pyramid.md`, 7).
- **The only guard is the wall.** Each profile's `global-timeout` in `.config/nextest.toml`, 300 s focused (a band-aid by the user's word of 11-15, after `default-wall-230.md`'s 230 s) and 600 s fuzzy, which `tools/tiers/one-process` reads (`tools/tiers/design.md`, 11.4). Its expiry is starved, a warning (`skein/docs/foundation/testing-strategy.md`, 8.1), and the gate's runner reruns a starved run once.
- **Today's figures,** the CI builder's gates of 2026-11-16, nine focused runs and eleven fuzzy ones on 8 lanes, against each tree's budget:

| Tree | Focused CPU s (budget) | Fuzzy CPU s (budget) |
|---|---|---|
| temper | 1,009 (1,250) | 472 (180) |
| jig | 161 (215) | 127 (120) |
| smith | 40 (30) | 31 (120) |
| hearth | 26 (30) | 83 (120) |
| skein | 25 (40) | 126 (120) |
| the repository's checks (`fabrig-boundary`) | 66 (none) | none |
| all | 1,329, in 201 to 218 s of wall | 838, in 146 to 171 s |

- **Where it goes:** world tests take two thirds of the focused suite. temper's take 669 of its 1,009 CPU s, its one-process world's suite 416 alone (`focused-pyramid.md`, 2, has the detail).
- **Steady at the gate.** Split by tree and layer, a figure's standard deviation across those runs is about 1%, its largest spread 8%, though changes landed between them; the suites' wall moves by 4%. One test's wall inside a shared process is far noisier: of the tests whose median is 10 to 50 ms, a tenth took at least 2.7 times their median in one of the nine runs.
- **The CI builder** has 8 CPUs. While a gate runs, the builder's other jobs are held to one CPU's time and the gate has the rest; no CPU is reserved for it, and its disk is shared. The steadiness above is what that gives.

## 3. CPU, judged at the gate

- **The measure** stays the strategy's: user plus system time of each test process, its waited descendants included, from `wait4`'s rusage (`tools/tiers/design.md`, 11), summed by layer (section 4) for each tree and suite.
- **The gate is the judge.** A budget's figures are the CI builder's at the gate, and the gate's verdict on them is red: a change that takes a layer over its budget, or a test past its reference, doesn't land.
- **The delivery check catches it first** (the user, 11-17). On a builder that keeps main's ledgers, the delivery check of `tools/pre-pr/design.md` judges each suite's selection as the gate judges its own: with main's ledger for the suite and the gate's verdict (`tools/tiers/budgets.md`, 5). A layer the delivery takes over its budget, or a new or flagged test past its reference, refuses the delivery with the gate's lines: the layer, the binaries that grew and their tests, or the test and its limit (sections 5 and 6). The goal hears it while its agent still holds the change, not from the gate after its pull request.
  - **Why the gate's verdict holds there.** The builders that run goals are the CI builder's kind: 8 CPUs of the same model, one thread a core (checked 11-17). CPU time counts no waiting, so a busy builder stretches a run's wall rather than its CPU. What sharing the machine's caches adds is unmeasured; a delivery refused for a layer the gate then passes is the sign that it matters.
  - **Where the ledgers come from.** After each landing the gate's runner copies main's ledger for each suite (section 5) to each builder that runs goals, as `default.json` and `fuzzy.json` in a directory the builder's environment names in `TIERS_LEDGER`, as a deployment names its hold in `TIERS_HOLD` (`temper/docs/design/domain/worker/README.md`). A copy a landing or two old still judges a delivery, and the gate judges again with its own.
  - **Without them,** where the variable is unset, the suite's file is missing or `ledger()` refuses it, the check runs that suite as before, its comparison a warning, and says why in one line: a builder's missing ledger never refuses a delivery.
  - **What it leaves out.** The check builds nothing at its base (`tools/pre-pr/design.md`, 1), so a flagged test is judged by the ledger's figures alone where it has them, and is otherwise a warning (`tools/tiers/references.md`, 6). Main's red stays a warning, as at the gate (section 5). The wall stays the backstop's, a starved warning (section 7). The check's passed-tree cache counts the ledgers it read, so a delivery is judged again when they change.
- **Every other run warns.** A builder's own run, anyone's, `tools/tiers/run`, and a delivery check without the ledgers print the same comparison as a warning naming the layer, its figure and its budget (the user, 11-16). This narrows 8.1's "wherever they run" for the two suites.
- **The slow tiers keep theirs.** The browser, process and live tiers keep their budgets in `tools/tiers/budgets.toml` and run as they do: this draft is the focused and fuzzy suites'.

## 4. Layers

- **A layer is a tier of the strategy's section 2:** step tests, domain worlds, system worlds, machine worlds, protocol worlds, io worlds, simulated worlds, the real loop, and end to end against fakes. A tree has a budget for each layer it has, in each suite. The repository's own checks (`fabrig-boundary`) are no layer of a pyramid, and have a budget of their own.
- **Every test binary is in one layer,** since CPU is measured per process and a process runs one binary's tests (`tools/tiers/design.md`, 11). A new table, `tools/tiers/layers.toml`, places them:
  - a library's own tests (a binary id with no `::`, or a `::bin/` one) and a crate's generated codec tests (`skein/docs/foundation/testing-strategy.md`, 2.1) are step tests, by rule, with no line;
  - every other binary has a line: its id and its layer, as its tree's testing document places it (temper's: `temper/docs/design/testing/README.md`, 2, where the client's world is a system world, 2.1);
  - a binary whose tests span two layers is split, or takes the higher one until it is.
- **A boundary check keeps the table whole** (`tests/boundary/`): from `cargo metadata`, which builds nothing, every test binary in the workspace has its layer and every line names a binary that exists. The change that adds a binary adds its line, and the change's review reads it against the tree's testing document: what a test checks places it, never its cost (8.1).

## 5. A partial run, judged whole

- **The gate runs a selection:** the packages a change reaches and those `.config/gate/always.txt` adds (`.config/gate/README.md`, the gate's fourth item). A layer's figure needs all its binaries.
- **The ledger.** Every gate whose candidate lands records what it measured, by the commit it landed: each binary's CPU, and each test's wall and, where the gate ran it alone (section 6), its CPU. A binary's figure at main is its figure in the last landed gate that ran it; a test's, the median of its last three. The ledger is the gate's own record, kept beside its reports outside this repository, as its record of the base's failures is.
- **A layer's figure at the gate** is what the gate measured for the layer's binaries in its selection, plus the ledger's figures for the others. The same sum at main is the layer's figure there.
- **The red is the change's.** A layer over its budget at the head and within it at main is red. Over at both, the change is red only where it adds more than the noise (1% of the layer, at least half a second): otherwise the red is main's, as a failure the base has too is.
- **A red names its cause:** the layer, its figure and its budget, and the binaries that grew against main, largest first, each with its tests that grew by wall. The goal is told where to look.
- **A binary the ledger lacks** counts by what the gate measures, and its tests are new (section 6). An empty ledger is filled by a whole run of the gate's own before any budget is held.

## 6. Each test's reference

- **The references are the strategy's per-test guides** (`skein/docs/foundation/testing-strategy.md`, 8), placed by layer in `tools/tiers/budgets.toml`, a tree's testing document setting its own where it has one:

| Layer | A focused test | A fuzzy test |
|---|---|---|
| step tests | 40 ms | 10 s |
| domain, machine, protocol and io worlds | 0.2 s | 10 s |
| system worlds | 0.6 s | 10 s |
| simulated worlds, the real loop, end to end | 1 s; temper's 2 s (`temper/docs/design/testing/checks.md`, 1) | 10 s |
| the repository's checks | 1 s | none |

  temper's tests at their 90th percentile today sit at these figures below the top of the pyramid (step tests 45 ms, domain worlds 210 ms, system worlds 299 ms), and well past them at the top (the one-process world's median test 3.6 s, the real loop's 2.3 s).
- **A new test,** one the ledger lacks, is run alone after the suite, in a process of its own: its CPU is that process's, less the same binary's run selecting no test. It stays within its layer's reference, or is red.
- **An existing test** stays within the larger of its layer's reference and half again its own CPU at main: one already past the reference keeps its place, and doesn't grow. Its wall in the shared run flags it, when past twice its figure at main and at least 50 ms over it, above the shared run's noise (section 2). A flagged test is run alone at the head and at the base, as the gate already reruns a failure at the base, and judged by the two CPU figures.
- **Past it, with a reason.** A test may pass its reference by a line in `tools/tiers/budgets.toml`: its binary and name, the CPU it may take, and why, in a sentence. The change's review judges the reason by what the test checks; the gate checks only the figure. Such a test still counts in its layer, so a reason never raises a budget, and past its own line's figure it is red again.
- **What a reference catches that a layer can't:** an expensive test on the day it is written, by name, while its layer still has room; and one test that doubles inside its layer's tenth of headroom.

## 7. Wall time

- **At the gate, red.** Each suite has a wall limit at the gate, 270 s focused and 215 s fuzzy: the longest of section 2's runs, plus a quarter (the user, 11-16). The gate passes it to its run through a configuration of its own (`tools/tiers/design.md`, 11.2). A run past it is stopped and rerun once, as now. Past it again, it is red, unless the gate was starved: its own cgroup recorded full CPU or io stalls (the kernel's pressure accounting) for at least a quarter of the run's wall. The quarter is a first figure: the gate's report gives each run's stall, and the figure follows them. A starved run stays the warning it is today, its tests unobserved.
  - The limit catches what CPU can't see: a test that sleeps or waits on the real clock, tests a test group holds to one at a time, and a process so long that the other lanes wait on it.
  - It narrows 8.1, where a wall limit's expiry is always starved, at the gate only. A harness's own deadline keeps 2.10's rule.
- **Elsewhere, generous.** Each profile's `global-timeout`, which every other run reads, becomes three times the gate's limit, 810 s focused and 645 s fuzzy, a last resort for builders that may be busy (the user, 11-16). Its expiry stays starved, a warning. It replaces the band-aid of `default-wall-230.md`, which goes when it lands.
- **The wall limits aren't ratcheted.** A selection's wall says nothing of the whole suite's: the limits move with the ideals, by the user's word.

## 8. Starting, then only falling

- **The starting figures** are the gate's of 2026-11-16 (section 2's runs, their median) by layer, plus a tenth and at least a second, raises and cuts alike (the user, 11-16). For the trees' totals that comes to about:

| Tree | Focused: today's budget, its start | Fuzzy |
|---|---|---|
| temper | 1,250, 1,110 | 180, 520 |
| jig | 215, 180 | 120, 140 |
| smith | 30, 45 | 120, 35 |
| hearth | 30, 29 | 120, 92 |
| skein | 40, 28 | 120, 139 |
| the repository's checks | none, 73 | none |

  Each layer's figure is written by the milestone that writes the layers' budgets, from the same runs, under `tools/tiers/layers.toml`.
- **The ratchet.** After each landing, a layer's budget falls to its figure at main (section 5) plus a tenth, at least a second, where that is lower than the budget. What a repair wins is kept, and the next change can't spend it; nothing raises a budget by itself. The figure in `tools/tiers/budgets.toml` is the user's ceiling. The budget the gate holds is the lower of that ceiling and the layer's lowest figure at main since the ceiling was last set, plus the tenth; the gate's report prints both.
- **Budget moves down the pyramid.** A change that places tests lower (the strategy's section 9; `focused-pyramid.md`, 4) moves their room with them: a line in `tools/tiers/budgets.toml` moves CPU seconds from one layer's budget to that of a layer below it in the strategy's table, in any tree. The gate checks only that each move goes down and leaves the sum as it was, and applies it to the ceiling and the held budget alike. A move up is a raise, and a raise is the user's (8.1).

## 9. The ideal

- **Beside each budget, its ideal,** the user's figure, in `tools/tiers/budgets.toml`. The gate's report gives each layer's figure, budget and ideal; nothing is red for missing an ideal. Repairs bring the figures down (`focused-pyramid.md`, for one), the ratchet keeps what they win, and the gap to the ideal is what's left to do.
- **Without an estimate, the strategy's figures,** for a tree's totals only: 30 s focused and 120 s fuzzy, skein's focused 40 s, temper's fuzzy 180 s (`tools/tiers/budgets.toml`'s notes). A tree's split by layer comes with its estimate, as temper's and jig's below. skein and hearth meet theirs already.
- **temper's focused suite: 100 CPU s,** from a lower bound. The question, the user's: given all of temper's behaviour, its functions and its programming model, what must its tests cost at least? It has no exact answer. The estimate is, for each layer, its count of tests times what one test must cost at least, from the gate's focused run of 2026-11-16 12:04Z:

| Layer | Tests | CPU s today | Its median test | Count × median, s | Least a test costs | Count × least, s |
|---|---|---|---|---|---|---|
| step tests | 10,372 | 240 | 2 ms | 21 | 2 ms | 21 |
| domain worlds | 1,726 | 128 | 46 ms | 79 | 10 ms | 17 |
| system worlds | 525 | 112 | 90 ms | 47 | 40 ms | 21 |
| machine and protocol worlds | 125 | 13 | 21 ms | 3 | 10 ms | 1 |
| simulated: the one-process world | 66 | 416 | 3.6 s | 237 | 0.4 s | 8 to 26 |
| the real loop | 24 | 51 | 2.3 s | 56 | 0.4 s | 10 |
| not yet placed | 691 | 50 | 19 ms | 13 | 5 ms | 3 |
| all | 13,529 | 1,009 | | 455 | | 80 to 100 |

  Where each least figure comes from:
  - **step tests,** 2 ms, today's median: one step function fed its events in an unoptimized build, its fixtures included. The tenth of them past 45 ms take 82% of the layer's time: tests that build a whole service or open a store to check one function, which `temper/docs/design/testing/checks.md`, 1 already forbids.
  - **domain worlds,** 10 ms, what temper's best domain worlds take at their median already: the engine's forge world 8 ms, the worker's domain and checkout worlds 3 ms each, against the engine's domain world's 69 ms.
  - **system worlds,** 40 ms: some two thousand iterations of one loop over several domains at about 20 µs each. Estimated, not measured.
  - **machine and protocol worlds,** 10 ms, as a domain world.
  - **the one-process world,** 0.4 s a story: a service started with its fixed tables built once (`temper/docs/design/testing/checks.md`, 1) and one story through it. It keeps 20 to 66 stories, as many as stay once the store's own mechanics are placed below (`focused-pyramid.md`, 4).
  - **the real loop,** 0.4 s a story: one end-to-end path per feature (`temper/docs/design/testing/real-loop.md`), most of it real git's processes, a few milliseconds each.
  - **the binaries not yet placed,** 5 ms: integration tests of tools and shells, which `tools/tiers/layers.toml` places.

  Read across: 455 s is every test at its layer's median today, so the tails alone are more than half the suite. 80 to 100 s is the floor, ten to twelve times under today, and three times the strategy's 30 s, which temper's count of tests can't meet without fewer tests. The ideal is the floor's top, 100 s, by layer: step tests 25, domain worlds 20, system worlds 25, machine and protocol worlds 2, the one-process world 15, the real loop 10, end to end and the rest 3.
  - **Its figures are wall in a shared process,** since no test's CPU is measured there (`tools/tiers/design.md`, 11); at the gate temper's focused CPU is 0.81 of its tests' wall. Its layers are placed by hand from `temper/docs/design/testing/README.md`, 2, and `tools/tiers/layers.toml` replaces them.
- **temper's worlds' sums give way to the layers' ideals.** `temper/docs/design/testing/checks.md`, 1 held the worlds below the domain other than the real loop to 8 s together and the real loop to 15 s, before any estimate: the first is under this floor, since the one-process world alone needs 8 to 26 s. The split above replaces both, and checks.md's per-test guides are temper's references (section 6).
- **jig's focused suite: 30 s,** the strategy's, which the same estimate confirms. Its 9,444 tests' medians are mostly under temper's least figures already (its step tests' 0.11 ms, its domain and system worlds' 6.5 ms), so each layer's least is the lower of the two: step tests 1 s, domain and system worlds 16, machine and protocol worlds 7, its one-process world 2, a floor of 26 s against 161 today, nearly all of it in the tails. The ideal by layer: step tests 1, domain and system worlds 17, machine and protocol worlds 9, the one-process world 3. Its worlds are placed by hand here as temper's are.
- **The repository's checks: 30 s,** against 66 today. Each checks the tracked tree, whose size they grow with: one walk of it shared within a binary (`focused-pyramid.md`, 5), and the few that read every file's content at a few seconds each in an unoptimized build, put their floor near 20 s.
- **temper's fuzzy suite: 180 s,** as it has. A sweep's cost is its seeds times its world's, a choice of how much to cover rather than a floor; the repairs that make the worlds cheaper let more seeds fit.

## 10. What it changes

- **`skein/docs/foundation/testing-strategy.md`, 8 and 8.1:** budgets by layer, red at the gate and at a delivery check with main's ledgers, the gate's wall red unless starved, the references and their reasons, the ratchet and the moves down. Section 8's "they run by default (`cargo nextest run --workspace`)" names one-process (`tools/tiers/design.md`, 11), which runs both suites; nextest still selects and builds their tests and runs the slow tiers.
- **`tools/tiers/design.md`, 7.3 and 11:** summaries by layer, `tools/tiers/layers.toml`, the ledger's form in and out, the runs of a test alone, the references and their lines.
- **`temper/docs/design/testing/checks.md`, 1:** its budgets and guides become temper's layers' budgets, references and ideal.
- **`.config/gate/README.md`,** the gate's fourth item, and `tools/pre-pr/design.md`: the gate holds the budgets, and the delivery check holds them where its builder keeps main's ledgers, warning otherwise.
- **The gate's runner, outside this repository:** the ledger, the held budgets, the red cases, the wall's limits, the stall reading, and the ledgers' copies on the builders that run goals.

## 11. Not in this draft

- **The slow tiers' budgets,** which stay as they are.
- **The repairs towards the ideal:** plans of their own (`focused-pyramid.md`, for one).
- **Raising a budget, a reference or an ideal,** which is the user's alone, and dropping or shrinking a test to meet one.

**Tests stay cheap:** the suites are the check, and a milestone runs a test it adds or changes by name alone, at most. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
