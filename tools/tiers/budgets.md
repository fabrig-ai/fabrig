# Budgets by layer

Provisional, 2026-11-16. What the focused and fuzzy suites may cost, by layer of the pyramid: the gate's tables in `tools/tiers/budgets.toml`, how `tools/tiers/one-process` sums a run by layer and judges it, and where the starting figures come from. Written from `temper/docs/design/draft/test-budgets.md`, 3, 4, 6, 8 and 9. The layers and the binaries in each are `tools/tiers/layers.md`'s.

## 1. In one page

- **A budget for each layer** (section 2): each tree's focused and fuzzy suites have a CPU budget for each layer the tree has (`tools/tiers/layers.md`, 2), and the repository's checks one of their own. Beside each layer's budget are its per-test reference and, where the draft gives one, its share of the ideal. Each tree has an ideal for each suite.
- **Each process placed** (section 3): `tools/tiers/one-process` gives each binary it runs a tree and a layer, by `tools/tiers/layers.md`'s rules and table, and records them with the binary's processes.
- **The summary by layer** (section 4): a one-process run's CPU summed by tree and layer, beside each layer's budget, ideal and reference, and each tree's total beside its ideal.
- **Red at the gate alone** (section 5): a layer over its budget is red where the caller asks for the gate's verdict (`--gate`), and a warning in every other run (the user, 11-16; `skein/docs/foundation/testing-strategy.md`, 8.1).
- **Starting figures** (section 6): the CI builder's gate runs of 11-16, their median by tree, suite and layer, plus a tenth and at least a second.
- **What follows** (section 7): the ledger that judges a partial run whole, the held budget and the moves down (B3), and each test held to its reference (B4), all in this document as they are designed.

## 2. The budgets file's gate tables

`tools/tiers/budgets.toml`'s `gate` table holds the focused and fuzzy suites' budgets. It sits beside the slow tiers' pair tables and `[tools]` (`tools/tiers/design.md`, 7.3), which keep their form. The table has one entry per profile: `default` (focused) and `fuzzy`.

```toml
[gate.default]
references = { step = 0.04, domain = 0.2, system = 0.6, machine = 0.2, protocol = 0.2, io = 0.2, simulated = 1, real-loop = 1, end-to-end = 1, checks = 1 }

[gate.default.temper]
ideal_s = 100

[gate.default.temper.step]
cpu_s = 253
ideal_s = 25

[gate.default.temper.protocol]
cpu_s = 9
ideal_s = 1
reference_s = 1

[gate.default.repository]
ideal_s = 30

[gate.default.repository.checks]
cpu_s = 72
```

- **A profile's table** holds `references` and one table for each **group**. A group is one of the five trees by name (`skein`, `smith`, `jig`, `temper`, `hearth`), or `repository` for the repository's checks (`tools/tiers/layers.md`, 4).
- **`references`** is an inline table that maps a layer's name to the per-test reference of a test in that layer, in CPU seconds (`temper/docs/design/draft/test-budgets.md`, 6). It names layers from `tools/tiers/layers.toml`'s `layers` list, and `checks` for the repository's checks.
- **A group's table** holds `ideal_s`, the group's ideal for the suite, and a table for each layer it has: for a tree, layers named in `layers`; for `repository`, `checks` alone.
- **A layer's table** holds `cpu_s`, the layer's budget for the suite. That figure is the user's ceiling: raising it is the user's (`skein/docs/foundation/testing-strategy.md`, 8.1). The table may also hold `ideal_s`, the layer's share of the group's ideal, where the draft gives one, and `reference_s`, the tree's own per-test reference for the layer, which replaces the profile's.
- **Every value** is a finite positive number, never a boolean. Nothing else is allowed in these tables.
- **`budgets(path, pairs=None, layers=Path('tools/tiers/layers.toml'))`** (`tools/tiers/design.md`, 10.2) reads the layers' names from the `layers` file's `layers` list. It reads that file only when the budgets file has a `gate` table. It raises `ValueError` on:
  - a key or value outside the form above;
  - a group that isn't one of the six;
  - a layer the list doesn't name, `checks` under a tree, or any other layer under `repository`;
  - a group without `ideal_s`, or a layer without `cpu_s`;
  - a layer with no reference: neither its own `reference_s` nor an entry in its profile's `references`.

  Its result's `gate` is the table as read: by profile, then `references` and the groups, then each group's `ideal_s` and layer tables. The form `[gate.<tree>.<profile>]` with `cpu_suite_s`, a tree's one figure for a suite, is gone and is refused.

## 3. A process's tree and layer

- **`run`** (`tools/tiers/design.md`, 11.1) places each binary of its selection. It reads the binary's package from `cargo metadata`, which it already reads (`tools/tiers/design.md`, 11.5), and the table from its new `layers` argument, `Path('tools/tiers/layers.toml')` by default, relative to the checkout root like `config`.
  - **Its tree:** the first directory of its package's `manifest_path` below the metadata's `workspace_root`, when that directory is `skein`, `smith`, `jig`, `temper` or `hearth` (`tools/tiers/layers.md`, 3). Otherwise it is `repository`.
  - **Its layer,** in this order:
    1. `checks` for a binary of `repository`, whatever its kind: the repository's checks are every test binary of a package outside the trees (`tools/tiers/layers.md`, 4);
    2. `step` for a library's own tests (an id with no `::`), a binary's own (an id with `::bin/`), and a crate's generated codec tests (an id ending `::generated`);
    3. otherwise the layer of its line under its tree's table;
    4. otherwise none: an unplaced binary, which the boundary check refuses (`tools/tiers/layers.md`, 7).
  - **When the layers file is absent,** rules 1 and 2 still place what they place, and every other binary is unplaced. The run goes on either way: a missing line is the boundary check's to refuse, never a run's.
- **What it records:**
  - each of `plan.json`'s binaries holds `tree` and `layer`, a string or null;
  - each of `results.json`'s processes holds the same two keys, copied from its binary;
  - `results.json` holds `layers`, the layers file's `layers` list, or an empty list when the file is absent.

  `--again` (`tools/tiers/design.md`, 11.9) copies its binary's `tree` and `layer` from the plan, and writes `layers` empty.

## 4. The summary by layer

`summary(directory, limits=None, pair=None, expected=None, gate=False)` (`tools/tiers/design.md`, 10.2) reads a one-process run's processes by tree and layer. It adds the following keys to its result (empty lists for a nextest run and for tools). A run directory whose processes lack `tree` and `layer` reads them as null.

- **`layers`:** one row for each tree and layer that has a process in the run, with these keys:
  - `tree` and `layer`, the layer null for an unplaced binary;
  - `cpu_s`, the sum of its processes' CPU, of every kind (`tests`, `fresh`, `alone`), as the suite's sum counts them;
  - `budget_s`, its layer table's `cpu_s` for the run's profile, or null;
  - `ideal_s`, its layer table's, or null;
  - `reference_s`, its own `reference_s` or else its profile's `references` entry, or null;
  - `over`, a boolean.

  Rows are ordered by tree (`skein`, `smith`, `jig`, `temper`, `hearth`, `repository`), then by `results.json`'s `layers` list. `checks` and the unplaced row come last.
- **`trees`:** one row for each tree that has a process in the run, with these keys:
  - `tree`;
  - `cpu_s`, its rows' sum;
  - `budget_s`, the sum of every layer table's `cpu_s` for the tree under the run's profile, or null when the tree has no table there;
  - `ideal_s`, the tree's `ideal_s`, or null.
- **The budgets** are `limits['gate'][profile]`, the run's profile in `budgets()`'s result. With none for the profile, every `budget_s` is null and nothing is over.
- **A row is over** in either case:
  - its `cpu_s` is more than its `budget_s`;
  - it has no `budget_s`, its layer isn't null, and its tree has a table for the profile. A tree has a budget for each layer it has (`temper/docs/design/draft/test-budgets.md`, 4). Room for a layer new to a tree comes from the user's raise, or from a move down once moves exist (section 7).

  An unplaced row is never over. It gives the warning `unplaced: BINARY_ID has no layer (tools/tiers/layers.md)`, one line for each such binary.
- **The line for a layer over its budget** reads `layer over budget: TREE PROFILE LAYER: FIGURE CPU s, budget BUDGET s`, or ends `no budget` where the row has none. `FIGURE` has two decimals and `BUDGET` is the file's figure as Python's `str` gives it.
- **With `gate`,** each such line is in `over_budget`. The verdict keeps its order (`tools/tiers/design.md`, 10.2): `over budget` when `over_budget` holds any line, a layer's or the pair's own; else `starved` for a starved run; else `within budget` when the profile has gate tables or a pair table; else `no budget set`.
- **Without `gate`,** each such line is in `warnings`, and the verdict is what it would be with no gate tables: a layer over its budget is a warning there, never red (the user, 11-16).
- **A selection** (a run with `-p`) sums only what it ran. A layer's figure in it is a lower bound of the layer's whole figure, and is judged as it stands. A partial run is judged whole once the ledger exists (section 7).
- **`print_summary`** prints, after the suite's totals, a line for each `layers` row and each `trees` row: the figure, the budget, the ideal and, for a layer, the reference, with `none` where null. Each warning or over-budget line follows, as today. The exact text of the rows is the code's; the over line is the form above.

## 5. Who asks for the gate's verdict

- **`tools/tiers/one-process [--out DIR] [--budgets PATH] [--walls PATH] [--gate] PROFILE [NEXTEST_ARGS...]`** (`tools/tiers/design.md`, 11.9).
  - **`--budgets`:** defaults to `tools/tiers/budgets.toml` where that file exists, so every run compares its layers with the repository's budgets. Without the file, no budgets apply.
  - **`--gate`:** asks for the gate's verdict. It is passed to `execute_one_process` as `gate`, and a layer over its budget then exits 1, as a CPU budget passed does.
  - **With `--again`,** `--gate` is an invalid invocation, as `--budgets` and `--walls` are.
- **`execute_one_process(command, env, directory, profile, limits, config=Path('.config/nextest.toml'), walls=None, gate=False)`:** gives `gate` to `summary`.
- **`tools/tiers/cpu-summary DIR [--budgets PATH] [--pair PAIR] [--expect ID ...] [--gate] [--json]`:** gives `gate` to `summary`. Its exit status keeps its rule: 1 when `over_budget` holds a line.
- **The gate** passes `--gate` in both suites' commands (`.config/gate/README.md`, item 4). The gate's runner, outside this repository, takes it up when it follows (`temper/docs/design/draft/test-budgets.execution.md`, 2). Until then the comparison prints and holds nothing.
- **Every other run** passes no `--gate`: the delivery check (`tools/pre-pr/design.md`, 4), a builder's run and anyone's own. Each prints the same rows, and a layer over its budget appears as a warning in its output.

## 6. The starting figures

- **Their source:** the CI builder's gate runs of 2026-11-16, nine focused and eleven fuzzy, each on 8 lanes, the runs `temper/docs/design/draft/test-budgets.md`, 2 reads. The user computed the figures from those runs' `results.json`, kept outside this repository, at 18:47Z (goal 5567's question, answered by the user, 11-16).
  - Each run's CPU, user plus system, is summed by tree and layer, the layers placed by main's `tools/tiers/layers.toml` as section 3 places them.
  - A figure is the median over the runs. A run without the cell counts 0.
- **A layer's budget** is its figure plus the larger of a tenth and a second, rounded up to a whole second (`temper/docs/design/draft/test-budgets.md`, 8; the user, 11-16). A layer the tree has in `tools/tiers/layers.toml` (its lines' layers, and `step`) with no figure in a suite starts at 1 s, the second added to nothing. The repository's checks have a focused budget alone, as the draft has.
- **Two cells left out:** temper's focused 4.8 s on `temper-protocol-store::patches`, and 2.0 s of step tests in packages outside the trees. Both are binaries main no longer has: the patches tests left with the store's patch history, and `jig-test-patches` and `temper-protocol-store-patches` were removed on 11-16. A test of theirs that moved elsewhere counts in that binary's layer, inside its tenth.
- **The references** are the draft's (`temper/docs/design/draft/test-budgets.md`, 6):
  - focused: 40 ms for a step test; 0.2 s for a domain, machine, protocol or io world test; 0.6 s for a system world test; 1 s for a simulated, real-loop or end-to-end test, and for the repository's checks;
  - fuzzy: 10 s in every layer.

  temper sets its own where its testing document does (`temper/docs/design/testing/checks.md`, 1): 1 s for a focused protocol world test, and 2 s for a focused simulated, real-loop or end-to-end test. No other tree's testing document sets a per-test figure.
- **The ideals** are the draft's (`temper/docs/design/draft/test-budgets.md`, 9):
  - each tree's for each suite: 30 s focused and 120 s fuzzy; skein's focused 40 s; temper's 100 s focused and 180 s fuzzy; the repository's checks 30 s;
  - by layer, temper's focused: step 25, domain 20, system 25, machine 1, protocol 1, simulated 15, real-loop 10, end-to-end 3;
  - by layer, jig's focused: step 1, domain 17, machine 1, protocol 8, simulated 3.

  Where the draft's figure covers two layers ("machine and protocol worlds"), it is split by those layers' starting figures, in whole seconds. A layer the draft names by its one binary ("the one-process world") takes it under that binary's layer. A world the draft placed by hand now has the layer `tools/tiers/layers.toml` gives it, as the draft says it will. So a layer's ideal is the draft's figure for the layer of that name, and some of jig's are far from their layers' figures (section 8).
- **`tools/tiers/budgets.toml`'s comments** cite this section and the user's 11-16 decisions, and replace the comments on the per-tree figures they replace.

## 7. Later

- **A partial run judged whole** (`temper/docs/design/draft/test-budgets.md`, 5; B3): the ledger of main's figures in and the run's own figures out; a layer's figure as what was measured plus the ledger's; the held budget, the lower of the ceiling and the lowest figure at main plus a tenth; the moves down and their check; a red that names its layer, its binaries and their tests.
- **Each test's reference held** (`temper/docs/design/draft/test-budgets.md`, 6; B4): new and flagged tests run alone, and the reasons' lines with their figures.
- **The suites run through nextest** (`tools/tiers/run`, section 7's collector) are not summed by layer. The focused and fuzzy suites run through one-process (`tools/tiers/design.md`, 11).

## 8. Open questions

- **jig's layer ideals.** The draft estimated jig's ideal with its worlds placed by hand. `tools/tiers/layers.toml` places most of them in `simulated` and `protocol`. Under it, jig's `domain` ideal of 17 s sits beside a figure of 2.4 s, and its `simulated` ideal of 3 s beside 99.5 s. Ideals are reported and never enforced. A split re-estimated under the layers' placement is the user's figure.
