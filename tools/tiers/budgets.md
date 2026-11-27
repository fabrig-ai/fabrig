# Budgets by layer

Provisional, 2026-11-16. What the focused and fuzzy suites may cost, by layer of the pyramid: the gate's tables in `tools/tiers/budgets.toml`, how `tools/tiers/one-process` sums a run by layer and judges it, where the starting figures come from, the ledger that judges a partial run whole, the budget the gate holds and the moves down. Written from `temper/docs/design/draft/test-budgets.md`, 3, 4, 5, 6, 8, 9 and 11. The layers and the binaries in each are `tools/tiers/layers.md`'s.

## 1. In one page

- **A budget for each layer** (section 2). Each tree's focused and fuzzy suites have a budget for each layer the tree has (`tools/tiers/layers.md`, 2), and the repository's checks have their own. An instruction ceiling in the layer's table makes instructions its measure. A layer with only a CPU ceiling is judged in seconds. Each layer has a per-test reference in CPU seconds and may have an ideal in either measure.
- **Each process placed** (section 3): `tools/tiers/one-process` gives each binary it runs a tree and a layer, by `tools/tiers/layers.md`'s rules and table, and records them with the binary's processes.
- **The summary by layer** (section 4). The report gives each layer's CPU, its user-space instruction count, and its budgets and ideals. The sums include `tests` and `fresh` processes. Diagnostic retries keep their own records. The report judges only the layers the run selected (section 4.1). The real pairs report their real-loop and end-to-end CPU figures as warnings after landing, and the campaign pairs each layer's against its fuzzy ceiling (section 4.2).
- **The gate's verdict** (section 5). Callers use `--gate` for red verdicts on layer budgets and per-test references. The gate's runner reports per-test reference overruns as warnings while its per-test check is off. A layer budgeted in instructions uses CPU for warnings when counts are unavailable. Drift gives warnings.
- **Starting figures** (section 6): the CI builder's gate runs of 11-16, their median by tree, suite and layer, plus a tenth and at least a second.
- **The ledger** (section 7). Main's ledger keeps each binary's last CPU figures, its counts for the last 100 counted landings, each test's last walls and the lowest layer figures in both measures. The gate's runner keeps it outside this repository. Every run writes a copy with its own figures folded in.
- **A partial run judged whole** (section 8). The report combines the measured binaries with main's figures for the other binaries. For an instruction layer, a change is red when it takes the layer over budget or its growth, after credit for new moves into the layer, exceeds both 2% of main's count and one billion instructions, unless the head stays within the layer's accepted level, where the user set one. This growth cap applies even within budget. Over at both with growth within the allowance warns. Main at least 8% above its lowest while within budget warns of drift. A CPU layer keeps its CPU comparison.
- **The held budget** (section 9). The gate holds the lower of the ceiling and the lowest figure at main plus a tenth, with a minimum allowance of a second or ten billion instructions, according to the measure. An instruction budget is held at least at the layer's accepted level, where set. The lowest figure only falls while the ceiling stands.
- **Moves down** (section 10). A move transfers CPU seconds and can also transfer instructions between layers. The gate checks that it goes down.
- **Every figure the user's** (section 10.1): the gate refuses a change that raises a figure of `tools/tiers/budgets.toml` or adds a reason's line, checked against its base by `tools/tiers/figures.py`, unless the change touches that file alone; and each budget red and refusal ends with a line sending its repair to the strategy.
- **Each test compared with its reference** is `tools/tiers/references.md`'s. New and flagged tests run alone, outside the layer sums. Their CPU figures go in the ledger beside their walls (section 7). References and the user's reason lines use CPU seconds.

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
instructions = 2300000000000
ideal_instructions = 225000000000

[gate.default.temper.protocol]
cpu_s = 9
ideal_s = 1
reference_s = 1

[gate.default.repository]
ideal_s = 30

[gate.default.repository.checks]
cpu_s = 72
```

- **A profile's table** holds `references`, `moves` where it has any (section 10), `reasons` where it has any (`tools/tiers/references.md`, 2), and one table for each **group**. A group is one of the five trees by name (`skein`, `smith`, `jig`, `temper`, `hearth`), or `repository` for the repository's checks (`tools/tiers/layers.md`, 4).
- **`references`** is an inline table that maps a layer's name to the per-test reference of a test in that layer, in CPU seconds (`temper/docs/design/draft/test-budgets.md`, 6). It names layers from `tools/tiers/layers.toml`'s `layers` list, and `checks` for the repository's checks.
- **A group's table** holds `ideal_s`, the group's ideal for the suite, and a table for each layer it has: for a tree, layers named in `layers`; for `repository`, `checks` alone.
- **A layer's table** holds `cpu_s`, the user's CPU ceiling for the suite, and may hold `instructions`, the user's instruction ceiling, and with it `accepted_instructions`, the user's accepted level. The layer may grow to that count past the growth cap (section 8), and its held instruction budget is at least that count (section 9). Reports give `ideal_s` and `ideal_instructions` where present. `reference_s` overrides the profile's per-test reference for this layer. Raising a ceiling is the user's (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **Every CPU value** is a finite positive number. `instructions`, `ideal_instructions` and `accepted_instructions` must be positive integers. Booleans are refused. Moves and reasons have their own forms (section 10 and `tools/tiers/references.md`, 2).
- **`budgets(path, pairs=None, layers=Path('tools/tiers/layers.toml'))`** (`tools/tiers/design.md`, 10.2) reads the layers' names from the `layers` file's `layers` list. It reads that file only when the budgets file has a `gate` table. It raises `ValueError` on:
  - a key or value outside the form above;
  - a group that isn't one of the six;
  - a layer the list doesn't name, `checks` under a tree, or any other layer under `repository`;
  - a group without `ideal_s`, or a layer without `cpu_s` or `instructions`;
  - a layer with `accepted_instructions` and no `instructions`;
  - a layer with no reference: neither its own `reference_s` nor an entry in its profile's `references`;
  - a reasons' line outside its form, or two naming the same binary and test (`tools/tiers/references.md`, 2).

  Its result's `gate` is the table as read: by profile, then `references` and the groups, then each group's `ideal_s` and layer tables. The form `[gate.<tree>.<profile>]` with `cpu_suite_s`, a tree's one figure for a suite, is gone and is refused.

## 3. A process's tree and layer

- **`run`** (`tools/tiers/design.md`, 11.1) places each binary of its selection. It reads the binary's package from `cargo metadata`, which it already reads (`tools/tiers/design.md`, 11.5), and the table from its new `layers` argument, `Path('tools/tiers/layers.toml')` by default, relative to the checkout root like `config`.
  - **Its tree:** the first directory of its package's `manifest_path` below the metadata's `workspace_root`, when that directory is `skein`, `smith`, `jig`, `temper` or `hearth` (`tools/tiers/layers.md`, 3). Otherwise it is `repository`.
  - **Its layer,** in this order:
    1. `checks` for a binary of `repository`, whatever its kind: the repository's checks are every test binary of a package outside the trees (`tools/tiers/layers.md`, 4);
    2. `step` for a library's own tests (an id with no `::`) and a binary's own (an id with `::bin/`);
    3. otherwise the layer of its line under its tree's table;
    4. otherwise none: an unplaced binary, which the boundary check refuses (`tools/tiers/layers.md`, 7).
  - **When the layers file is absent,** rules 1 and 2 still place what they place, and every other binary is unplaced. The run goes on either way: a missing line is the boundary check's to refuse, never a run's.
- **What it records:**
  - each of `plan.json`'s binaries holds `tree` and `layer`, a string or null;
  - each of `results.json`'s processes holds the same two keys, copied from its binary;
  - `results.json` holds `layers`, the layers file's `layers` list, or an empty list when the file is absent;
  - `results.json` holds `workspace`, a list with one entry for each test binary of the workspace (`tools/tiers/layers.md`, 3: a target whose `test` is true in the `cargo metadata` `run` reads, its id by that section's table), sorted by `binary_id`. Each entry has `binary_id`, `tree` and `layer` as this section places it, and `listed`, true where the run's listing names the binary with the status `listed`, whether or not any of its tests was selected. Section 8 reads it to count, with a ledger, the binaries the run didn't run.

  `--again` (`tools/tiers/design.md`, 11.9) copies its binary's `tree` and `layer` from the plan, and writes `layers` and `workspace` empty.

## 4. The summary by layer

`summary(directory, limits=None, pair=None, expected=None, gate=False, ledger=None)` (`tools/tiers/design.md`, 10.2) reads a one-process run's processes by tree and layer. It adds the following keys to its result (empty lists for a nextest run and for tools). A run directory whose processes lack `tree` and `layer` reads them as null. This section is a run without a ledger; section 8 is what main's ledger adds.

- **`layers`:** one row for each tree and layer that has a suite process in the run, with these keys:
  - `tree` and `layer`, the layer null for an unplaced binary;
  - `cpu_s`, the sum of its `tests` and `fresh` processes' CPU. The same kinds make up the suite, binary, and ledger sums in both measures (sections 7 and 8). Each diagnostic `alone`, `reference`, and `empty` process keeps its own record outside those sums (`tools/tiers/design.md`, 11.4 and 11.6; `tools/tiers/references.md`, 4);
  - `ceiling_s`, the layer's ceiling for the run's profile: its table's `cpu_s` with the moves applied (section 10), or null where it has none;
  - `budget_s`, the budget the run holds it to: its ceiling, or with a ledger its held budget (section 9); null where it has no ceiling;
  - `ideal_s`, its layer table's, or null;
  - `reference_s`, its own `reference_s` or else its profile's `references` entry, or null;
  - `over`, a boolean.

  Rows are ordered by tree (`skein`, `smith`, `jig`, `temper`, `hearth`, `repository`), then by `results.json`'s `layers` list. `checks` and the unplaced row come last.
- **`trees`:** one row for each tree that has a suite process in the run, with these keys:
  - `tree`;
  - `cpu_s`, its rows' sum;
  - `ceiling_s` and `budget_s`, the sums of the ceilings and of the budgets of every layer of the tree that has a ceiling under the run's profile, or null when the tree has no table there;
  - `ideal_s`, the tree's `ideal_s`, or null.
- **Counts beside the CPU** (`tools/tiers/design.md`, 11.4). Each process's `instructions` comes from `results.json`, null where there is no count. Each layer's `instructions` is its binaries' count at the head (section 8), null where any binary has no count. Each tree's `instructions` is its layers' sum, null where any is null. The result's `counting` is the run's counting report, null where absent, for a nextest run and for tools.
- **The budgets** are `limits['gate'][profile]`, the run's profile in `budgets()`'s result. With none for the profile, every `ceiling_s` and `budget_s` is null and nothing is over.
- **A layer's instruction figures** are `instructions_ceiling` after moves, `instructions_budget` at the head, `instructions_lowest` in the folded ledger, `main_instructions_budget` and `main_instructions_lowest` at main (section 9), `ideal_instructions` from the table, and `instructions_growth` against main in percent (section 8). A figure unavailable or inapplicable is null. The CPU figures, including `main_over`, keep their CPU meanings.
- **A layer with `instructions` in its table** follows section 8's count comparison. Without a ledger it can compare a counted selection with its ceiling. Where counting is unavailable, its CPU comparison follows the rules below and gives warnings with or without `gate`.
- **A row judged in CPU is over** in either case:
  - its `cpu_s` is more than its `budget_s`;
  - it has no `budget_s`, its layer isn't null, and its tree has a table for the profile. A tree has a budget for each layer it has (`temper/docs/design/draft/test-budgets.md`, 4). Room for a layer new to a tree comes from the user's raise, or from a move down (section 10).

  With a ledger, section 8 narrows `over` to the change's red.

  An unplaced row is never over. It gives the warning `unplaced: BINARY_ID has no layer (tools/tiers/layers.md)`, one line for each such binary.
- **The line for a layer over its budget** reads `layer over budget: TREE PROFILE LAYER: FIGURE CPU s, budget BUDGET s`, or ends `no budget` where the row has none. `FIGURE` has two decimals and `BUDGET` is the file's figure as Python's `str` gives it.
- **With `gate`,** layer lines eligible for a red go in `over_budget`. The verdict is `over budget` when that list holds any line, else `starved` for a starved run, else `within budget` when the profile has gate tables or a pair table, else `no budget set` (`tools/tiers/design.md`, 10.2).
- **Without `gate`,** each such line is in `warnings`, and the verdict is what it would be with no gate tables: a layer over its budget is a warning there, never red (the user, 11-16).
- **Each budget red ends with section 10.1's repair line,** on its own line after any binaries and tests (section 8). Check-loss reds follow it with the reason instructions (section 13). Layer warnings and per-test reference warnings carry no repair line.
- **A selection** (a run with `-p`) sums only what it ran. Without a ledger, a layer's figure in it is a lower bound of the layer's whole figure, and is judged as it stands. With main's ledger it is judged whole (section 8).
- **`print_summary`** prints each layer and tree after the suite totals, giving its CPU figure, budget, ceiling and ideal, plus the reference for a layer and the figure at main when a ledger is present. It prints the head and main instruction counts and, for layers with an instruction ceiling or ideal, the instruction ceilings, budgets, lowest counts, ideals and growth. It prints `none` for a null figure. Each process's line ends with its count. Warnings and reds follow the rows.

### 4.1 Only the layers a run ran

A one-process run of the focused or fuzzy profile runs by default the layers that replay and the repository's checks, and leaves out the real loop and end to end (`tools/tiers/layers.md`, 9.1). It judges only what it ran (`temper/docs/design/draft/test-budgets.md`, 4).

- **`selected_layers`** in `results.json` is the list of the layers the run ran: names of the layers file's `layers` list in its order, then `checks` where the run ran the repository's checks; every name and `checks` for a run of every layer. An unplaced binary runs whatever the list. Without the layers file, the default and `all` run every binary, those its rules place (step and checks, section 3) and the unplaced, and `selected_layers` is null; any other list is refused (`tools/tiers/layers.md`, 9.1). A `results.json` without it, or with it null, as a run's before this section or `--again`'s, reads as every layer.
- **The rows:** `layers` has a row only for a layer in `selected_layers`, and the unplaced row; a layer the run left out has none, at the head or at main, so it is never over, never warned of and never red. A tree's row sums its rows', and its `ceiling_s` and `budget_s` sum the ceilings and budgets of its layers in `selected_layers` alone. Its `ideal_s` stays the tree's, reported as before.
- **The ledger's binaries:** with a ledger, a binary placed in a layer the run left out, by the run's `workspace` entry for it or, where `workspace` lacks it, by the ledger's, counts nowhere: neither at the head (section 8) nor at main in any row. In the run's `ledger.json` (section 7) it leaves, listed or not, as a binary the listing named and the run selected none of leaves: the ledger holds what the gate runs, so the replay after landing compares the gate's binaries alone (draft, 4). A binary the ledger places in a layer the run ran and the run's `workspace` in one it left out counts at main in the ledger's layer, as a binary placed elsewhere does, and leaves.
- **The held budgets.** The fold (section 9) writes a `budgets` entry only for a layer in `selected_layers`. A layer the run left out has no entry in the run's `ledger.json` and neither lowest figure can be lowered by an unselected layer's empty sum. A later run can seed an instruction budget from main's count when it has one (section 9).
- **The moves** apply to the ceilings as before (section 10): a move from the real loop into a layer below it still gives that layer its room.

The first gate after this lands passes main's ledger as the gate before it wrote it, real-loop and end-to-end binaries and budgets included; its run leaves them out of its rows and its `ledger.json`, and main's ledger holds the layers that replay from its landing on.

### 4.2 The real and campaign pairs, after landing

Each tree's pair `real-TREE` runs its real-loop and end-to-end tests after landing, through nextest, a process per test (`tools/tiers/layers.md`, 9.2). The real loop's and end to end's layers keep their CPU figures in the gate tables, reported by those runs as warnings, until a draft of their own (`temper/docs/design/draft/test-budgets.md`, 11). Each tree's pair `campaign-TREE` runs its adopted sweeps' fuzzy binaries with fresh seeds, the same way (`tools/tiers/layers.md`, 9.3), and its rounds are reported against the tree's fuzzy ceilings, as warnings (`temper/docs/design/draft/fuzzy-campaigns.md`, 5.3).

- **`summary(..., pair='real-TREE')`** of a nextest run (`tools/tiers/design.md`, 7.3), for a pair whose tier, the name's part before its last hyphen, is `real`, places each attempt's binary by its `binary_id` under the pair's tree in the layers file, `tools/tiers/layers.toml` from the checkout root, as section 3 places one; an id with no line there is unplaced.
- **`layers`** then holds a row for each layer with an attempt: `tree`, the pair's; `layer`; `cpu_s`, the sum of its attempts' recorded CPU, as the suite's CPU sums them; `ceiling_s`, the sum of the tree's ceilings for the layer under `gate.default` and `gate.fuzzy`, each with its profile's moves applied (section 10), null where neither has one, since the pair runs both suites' tests; `budget_s`, the same; `ideal_s` and `reference_s`, null; `instructions`, null; `over`, where `budget_s` isn't null and `cpu_s` is more than it. `trees` stays empty.
- **A row over** gives `layer over budget: TREE real LAYER: FIGURE CPU s, budget BUDGET s`, section 4's form with `real` for the profile, in `warnings` whatever `gate` says, never in `over_budget`; an unplaced binary gives section 4's warning. Neither moves the verdict or the exit status. The pair's own table, should the user add one, holds it as any pair's is held (`tools/tiers/design.md`, 7.3).
- **Why it reads high.** The gate measured those figures a process per binary, and the pair runs a process per test, so a layer here carries every test's process start beside its work. The warning is the report the draft asks for, never a verdict, and the user's figures for these runs come with their own draft.
- **`summary(..., pair='campaign-TREE')`,** for a pair whose tier is `campaign`, gives the same rows and warnings, but for two things: `ceiling_s` and `budget_s` are the tree's ceiling for the layer under `gate.fuzzy` alone, with its moves applied, since the pair runs only fuzzy tests; and its line reads `layer over budget: TREE campaign LAYER: FIGURE CPU s, budget BUDGET s`. A campaign's cases draw other work than the gate's seeds, and its round runs a process per test, so the comparison is no forecast (the draft, 5.3). `tools/tiers/campaign` keeps each round's rows and warnings in its report (`tools/tiers/campaign.md`, 7).
- **Any other pair** summarises as before, with `layers` empty (section 11).

## 5. Who asks for the gate's verdict

- **`tools/tiers/one-process [--out DIR] [--budgets PATH] [--walls PATH] [--ledger PATH] [--gate] PROFILE [NEXTEST_ARGS...]`** (`tools/tiers/design.md`, 11.9).
  - **`--budgets`:** defaults to `tools/tiers/budgets.toml` where that file exists, so every run compares its layers with the repository's budgets. Without the file, no budgets apply.
  - **`--ledger`:** main's ledger for the profile (section 7), read with `ledger()` before anything runs; a ledger `ledger()` refuses, or one for another profile, is an invalid invocation (exit 2). It is passed to `execute_one_process` as `ledger`. None by default.
  - **`--gate`:** asks for the gate's verdict and is passed to `execute_one_process` as `gate`. A red layer verdict or a test past its reference then makes the command exit 1.
  - **With `--again`,** `--gate` is an invalid invocation, as `--budgets`, `--walls` and `--ledger` are.
- **`execute_one_process(command, env, directory, profile, limits, config=Path('.config/nextest.toml'), walls=None, gate=False, ledger=None)`:** gives `gate` and `ledger`, the ledger as `ledger()` read it or none, to `summary`, and writes the run's `ledger.json` (section 7) once the summary is made.
- **`tools/tiers/cpu-summary DIR [--budgets PATH] [--pair PAIR] [--expect ID ...] [--ledger PATH] [--base DIR] [--gate] [--json]`:** gives `gate`, the ledger `--ledger` names as `ledger()` reads it, and the base's run directory `--base` names (`tools/tiers/references.md`, 5 and 6), to `summary`. It writes no ledger. Its exit status keeps its rule: 1 when `over_budget` holds a line; 2 for a ledger refused.
- **Main Gate** compares both suites with the available ledgers. It uses the instruction verdict for layers with instruction figures and a CPU verdict for other layers. The runner selects the verdict policy by passing `--gate`. It reports per-test reference overruns as warnings while its per-test check is off and keeps them red when it is on.
- **The full goal check** passes `--ledger` where its builder keeps a readable ledger for the profile. It reports layer budget and per-test reference warnings (`tools/pre-pr/design.md`, 4). Its runner selects the verdict policy. Without a usable ledger it reports why.
- **Every other run** passes no `--gate` and no `--ledger`: a builder's own run and anyone's, and a delivery check without a ledger. Each prints the same rows, and a layer over its budget appears as a warning in its output.

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

## 7. The ledger

- **What it is:** main's figures for one suite, from the gate runs whose candidates landed (`temper/docs/design/draft/test-budgets.md`, 5): each binary's last CPU figures and its counts by landing, each test's last walls and, where a gate ran it alone, its last CPU figures alone (`tools/tiers/references.md`, 4), each layer's held state (section 9) and the moves its held budgets have taken (section 10). The gate's runner, outside this repository, keeps one for each profile beside its reports (`temper/docs/design/draft/test-budgets.execution.md`, 2). The repository holds none.
- **Its form:** one JSON object, written as `cpu.py`'s `save` writes a file:

```json
{"ledger": 7, "profile": "default",
 "binaries": {"temper-engine::domain": {"tree": "temper", "layer": "domain", "cpu": [60.4, 63.0, 61.2],
                                        "tests": {"forge::opens": [0.011, 0.012, 0.010]},
                                        "alone": {"forge::opens": [0.008]},
                                        "counts": [{"commit": "1f3a...", "instructions": 482103771120},
                                                   {"commit": "9b0c...", "instructions": 482109310442}],
                                        "checks": {"store-check": 16, "seed": 32}}},
 "budgets": {"temper": {"domain": {"ceiling_s": 141, "lowest_s": 118.4,
                                    "instructions_ceiling": 900000000000,
                                    "instructions_lowest": 482109310442}}},
 "moves": [{"from": "temper.simulated", "to": "temper.domain", "cpu_s": 12, "why": "..."}],
 "check_reasons": []}
```

  - `ledger`, the form's version, 7;
  - `profile`, `default` or `fuzzy`;
  - `binaries`, by binary id: `tree`, one of section 2's six groups; `layer`, a name of the `layers` list, `checks`, or null, as section 3 placed it in the run that wrote the ledger; `cpu`, a list of one to three CPU figures, each the CPU of its processes, of the suite's kinds (section 4), in one of the last runs that measured it whole (below), oldest first, whose median is its figure at main (section 8); `tests`, by test name, a list of one to three walls, that test's in the last runs that measured it, oldest first; `alone`, by test name, a list of one to three CPU figures, that test's alone (`tools/tiers/references.md`, 4) in the last runs that ran it alone, oldest first, for tests some landed gate ran alone: every name in `alone` is in `tests`; `counts`, a list of none to 100 entries, oldest first, one for each of the last runs that measured it whole and counted it (below): `instructions`, its count there (`tools/tiers/design.md`, 11.4), the sum of its processes' of the suite's kinds, and `commit`, that run's `commit` from its `results.json` (`tools/tiers/design.md`, 11.8), or null. Its last entry's count is its count at main (section 8): a count is no sample, so no median is taken (draft, 5). Two entries in a row say what the landings between them added to the binary, and since a gate runs only the packages a change reaches, a landing that didn't run the binary added nothing to it;
  - `budgets`, by group and layer, holds `ceiling_s` and `lowest_s` for each layer with a CPU ceiling. An instruction ceiling adds `instructions_ceiling` and `instructions_lowest`, both nonnegative integers, with null allowed for the lowest until a complete count is available (section 9).
  - `moves`, the moves of main's budgets file for the profile, each as section 10's form has it;
  - `check_reasons`, the reasons in main's budgets file for the profile, each as section 13's form has it;
  - `spent_check_reasons`, the budget check reasons that have covered a loss on a landed run. The fold keeps them even when their lines leave the budgets file (section 13).

  A binary's `checks` holds its totals by kind from the last run that measured it whole (section 13).

  Every figure is a finite number at least 0, never a boolean; a count is an integer at least 0, never a boolean, and a commit is 40 or 64 lowercase hex digits. Nothing else is allowed.
- **The last 100 landings** (draft-10, 11-20). The draft keeps each landing's count beside the binary's count at main (draft, 5). Kept for every landing, the ledger would grow without end, and every gate run and delivery check reads and rewrites it whole: the week to 11-20 had 470 landings on main and the workspace about 500 test binaries, near 0.75 MB a day of counts. So a binary keeps its counts for its last 100 counted landings, close to the walls' size in the ledger, and says what each of them added. The runner's kept ledgers hold the older counts. Naming drift with the landings that grew a layer since its lowest (draft, 8) may need a longer look-back, and widens the bound with a new version then.
- **`ledger(path)`** (`tools/tiers/design.md`, 10.2) reads one and returns it as a dictionary. It raises `ValueError` on anything outside the form: another key, a version other than 1 to 7, a profile other than the two, a group other than section 2's six, a layer that is neither a nonempty string nor null (`budgets` takes no null), a figure outside the above, a binary whose `cpu` is no list or has no figure or more than three, a test with no wall or more than three, an `alone` entry with no figure or more than three, or for a name not in `tests`, a binary whose `counts` is no list or has more than 100 entries, an entry of it with keys other than `commit` and `instructions`, a count outside the above or a commit that is neither null nor one, an empty id or name, a move without section 10's keys and their types. It reads neither the layers file nor a budgets file: a layer or a move main no longer has is no error, and counts as section 8 and 9 say. A new version's reader reads the forms before it too, since the runner's ledger outlives the landing that changes it.
  - **Versions 1 to 3.** An older ledger reads as one without counts. Version 3, B7's, is version 4 without `counts`: each binary has exactly the keys `tree`, `layer`, `cpu`, `tests` and `alone`, and `ledger()` returns it with `counts` empty. Version 2, B4's, is version 3 with `cpu_s`, one figure, in place of `cpu`: each binary has exactly the keys `tree`, `layer`, `cpu_s`, `tests` and `alone`, and `ledger()` returns it with `cpu` the list of that one figure, no `cpu_s` and `counts` empty. Version 1, B3's, is version 2 without `alone`: each binary has exactly the four other keys, and `ledger()` returns it with `alone` and `counts` empty too. A binary of any is checked against its own version's keys, so a version 2 binary with `cpu`, a version 3 one with `cpu_s` or `counts`, or a version 4 one without `counts`, is refused. The reader preserves the version in `ledger`.
  - **Versions 1 to 4** have CPU-only `budgets` entries and moves. When a layer gains an instruction ceiling, the fold seeds its lowest from the layer's count at main when available (section 9). Every run writes version 7.
  - **Versions 1 to 5** read with an empty `checks` object on each binary.
  - **Version 7** takes `check_reasons` and `spent_check_reasons`. An absent list is empty. Version 6 reads as version 7, moving its `coverage_reasons` and `spent_coverage_reasons` into those lists and dropping entries for test names.
  - **A reader before version 7** refuses a version 7 ledger. Its delivery check runs without main's ledger and says why until the branch takes main (`tools/pre-pr/design.md`, 4).
- **Who passes it:** the gate, as `--ledger PATH` (section 5): main's ledger for the suite, at the candidate's base; and the delivery check, a copy the gate's runner left on its builder, a landing or two old at most. Every other run passes none, and its selection is judged as it stands (section 4).
- **Every run writes its own.** A one-process run that reaches its summary writes `ledger.json` in its directory (`tools/tiers/design.md`, 11.8): main's ledger as it will stand once the run's candidate lands. It is the given ledger, or an empty one, with the run's figures folded in:
  - **a binary the run measured whole,** every one of its selected tests `passed`, `failed` or `crashed` and none `starved`, `cancelled` or unobserved: its `tree` and `layer` as the run placed it, its `cpu` the ledger's last two figures for it, then the CPU the run measured for it (`[64.0, 61.2, 63.0]` from `[60.0, 64.0, 61.2]` and 63.0; a binary new to the ledger has the run's figure only, and one the run places elsewhere keeps its figures, as a test's walls are kept), and its `tests` as the run ran them, each with the ledger's last two walls for it, then the run's own. Its `alone`, for each test the run ran: the ledger's last two figures for it, then the run's own, where the run's `references` (`tools/tiers/references.md`, 4) give it a `cpu_s` that isn't null, the lowest of the test's runs alone there; otherwise the ledger's figures as they were, where it has any. A test the ledger had and the run didn't run leaves, from both. Its `counts`, where the run counted it (its count at the head isn't null, section 4): the ledger's last 99 entries for it, then `{"commit": COMMIT, "instructions": COUNT}`, COMMIT the run's `commit` and COUNT its count at the head; where the run didn't count it, the ledger's entries as they were, none for a binary new to the ledger. A binary the run places elsewhere keeps its counts, as its walls are kept;
  - **a binary the run cut short:** its ledger figures and histories, with the run's `tree` and `layer`, or none where the ledger lacks it, since its CPU is partial. When the selector took whole packages, retain only its selected names in `tests` and `alone` (section 13);
  - **a ledger binary the run didn't run:** kept, with its `workspace` entry's `tree` and `layer` (section 3), where `workspace` has it unlisted. An unlisted binary in a selected package has empty `tests`, `alone` and `checks` when the selector took whole packages. It leaves where `workspace` lacks it (its target is gone), where the listing named it and the run selected none of its tests, and where it is placed in a layer the run left out, listed or not (section 4.1);
  - **`budgets`** as section 9 folds them. **`moves`** and **`check_reasons`** are the run's budgets file's lists for the profile, each empty where the file gives none.
- **The runner keeps** the `ledger.json` of the gate run whose candidate landed as main's ledger for that suite, and passes it to the next gate on top of that landing. Its first ledger is the `ledger.json` of a whole run of its own on main, with no `-p` and no `--ledger`: until then no selection is judged whole and no budget is held below its ceiling (draft, 5). The gate selects by package alone; a run that also filters tests writes a ledger that isn't main's.

### 7.1 Main's published counts

After each landing the gate's runner publishes one commit at
`refs/temper/counts` with two files, `counts.json` and `allowances.tsv`. A worker fetches the
ref and its blob at prepare (`temper/docs/design/draft/test-budgets.md`,
3). Its form is:

```json
{"version": 1,
 "commits": ["<oldest sha>", "...", "<newest sha>"],
 "profiles": {"default": {"binaries": {"jig-world::domain": {"tree": "jig",
                    "layer": "domain", "cpu_s": 9.7,
                    "counts": {"<sha>": 123456789}}},
                          "moves": []},
              "fuzzy": {"binaries": {}, "moves": []}}}
```

`commits` holds the last twenty commits main landed, oldest first.
At commit C, a binary has the count of the newest entry in its
`counts` at or before C in that window. It has no count before its first
entry. Its tree, layer and CPU figure are those at the newest commit.
Each profile's `moves` are those at the newest commit, as kept in main's
ledger in section 7.

`allowances.tsv` is the same data at the newest commit, for `tools/tiers/test`'s
lines (`tools/tiers/design.md`, 11.9), tab-separated, a line each: `version 1`;
`main SHA`, the newest commit; `binary PROFILE BINARY_ID TREE LAYER` for each
binary; and `layer PROFILE TREE LAYER COUNT ALLOWANCE` for each layer, its count
at the newest commit and `allowance` of it, with the layer's accepted level
in the newest commit's `tools/tiers/budgets.toml`. Small and flat, so that an
implementer's run reads it in a few milliseconds.

`python3 -I -B tools/tiers/counts.py export --ledger default=PATH [--ledger fuzzy=PATH] [--head REV] --out DIR`,
from the checkout root, writes both files into DIR, REV `HEAD` by default. The gate's
runner runs it after each landing, with main's ledgers as main's tip has them
(section 7) and REV main's tip. `commits` are REV's last twenty first-parent commits.
A binary keeps the ledger's counts at the window's commits, and the newest count it
had before them stands at the window's first commit where it has none there, so each
commit's sums are main's there. Its `cpu_s` is its figure at main (section 8). A binary with no count is left
out, and a profile without a ledger has no binaries and no moves. The allowances take
the accepted levels of REV's `tools/tiers/budgets.toml` (section 2). It exits 2 on a
ledger `ledger()` refuses, a ledger of the other profile, or a git failure.

`tools/tiers/counts.py` reads the window. `read(root, env)` uses
`git cat-file blob refs/temper/counts:counts.json` with
`GIT_NO_LAZY_FETCH=1` and returns the parsed version 1 object or none
when the local file is unavailable. `base(data, root, env, head='HEAD')`
checks the window newest first with `git merge-base --is-ancestor`
under the same environment and returns the newest commit head contains,
or none. `binaries(data, profile, commit)` maps binary IDs to each
counted binary's `tree`, `layer` and `instructions`.
`layers(data, profile, commit)` sums them by `(tree, layer)`.
`allowance(count, credit=0, accepted=None)` returns the growth allowance in section 8,
`max(count // 50, 1_000_000_000) + credit`, or `accepted - count` where that is larger.

## 8. A partial run, judged whole

`summary`'s `ledger` (section 4) is main's ledger as `ledger()` read it, or none. With one, `summary` refuses (`ValueError`) a nextest run, tools, a ledger whose `profile` isn't the run's, and a `results.json` without `workspace`.

- **A binary's figure at main** is the median of its ledger entry's `cpu` figures, as Python's `statistics.median` takes it: the middle of three, the mean of two, the one of one. One run's figure is a sample, and a landing keeps the samples that passed, so the last alone sits low; a test's figure at main is the median of its walls the same way (draft, 5). Wherever this section takes the ledger's figure for a binary, it takes this one: at main, at the head for a binary the run didn't run, and in its growth. A ledger whose `cpu` lists hold `[60.0, 64.0, 61.2]` and `[50.0, 54.0]` gives those binaries 61.2 s and 52.0 s at main.
- **A binary's figure at the head** is the CPU of its processes of the suite's kinds (section 4) where the run ran it; its ledger figure where the run's `workspace` has it with `listed` false; nothing where the listing named it and the run ran none of its tests, or where `workspace` lacks it. It counts in the tree and layer the run places it in: its processes', or its `workspace` entry's.
- **A binary's count at main** is the last entry of its ledger `counts`, null when there is none. At the head a whole measurement sums its `tests` and `fresh` processes' counts. A binary carried over by the workspace rules above, or one whose run was cut short, uses its last whole count where available. An incomplete binary with no last count is uncounted. A binary absent on one side adds nothing there. Each layer's `main_instructions` is the sum of the counts at main of the binaries the ledger places there. A tree's `main_instructions` sums those of its layers. Either is null without a ledger or if any count in the sum is unavailable.
- **A layer's figure at the head** is the sum of its binaries' figures there. **Its figure at main** is the sum of the figures at main of the binaries the ledger places there.
- **The rows** (section 4's `layers`) are one for each tree and layer the run ran (section 4.1) with a binary at the head or at main, in section 4's order, and gain these keys:
  - `cpu_s`, its figure at the head;
  - `measured_s`, what the run measured there, the sum of its processes' CPU;
  - `main_s`, its figure at main, null without a ledger;
  - `budget_s`, its held budget at the head (section 9);
  - `main_budget_s`, its held budget at main (section 9), null without a ledger or where the ledger's `budgets` has no entry for it;
  - `lowest_s`, the `lowest_s` the run's `ledger.json` records for it (section 9), null where it has no ceiling;
  - `main_over`, true where its figure at main is more than its held budget at main;
  - `grew`, the binaries its over line names (below), empty where it isn't over.

  Without a ledger, `measured_s` is `cpu_s`, `main_s`, `main_budget_s` are null, `main_over` is false and `grew` empty. A tree's row (section 4's `trees`) gains `main_s`, the sum of its rows', null without a ledger, and its `cpu_s`, `ceiling_s` and `budget_s` are section 4's sums of these figures.
- **The CPU red is the change's** (draft, 5). A CPU row is over in section 4's cases if main is within budget, or if the head grows by more than the larger of 5% of main's CPU figure and one second. The line is a red with `gate` for a CPU-only layer, or a warning for an instruction layer without counts.
- **Main's CPU overrun** warns when both figures are over their budgets and the head is within that tolerance. The line reads `layer over budget at main: TREE PROFILE LAYER: MAIN CPU s at main, budget MAIN_BUDGET s; FIGURE CPU s at the head`. The figures have two decimals, and the budget prints as in section 4.
- **The instruction verdict** is red when the head exceeds its held budget and main is within its budget, or when the head's growth exceeds its growth allowance, even within budget. The allowance is the larger of 2% of main's count and 1,000,000,000 instructions, plus credit for the change's new moves into the layer. Where the layer's table at the head holds `accepted_instructions` (section 2), the user's accepted level, the allowance is that level less main's count where that is larger: a head count up to the level passes the growth check. The level also floors the held budgets, which may exceed their ceilings (section 9). The comparison uses whole instructions, with 2% rounded down. If both counts are over budget and growth is within the allowance, the report gives an over-at-main warning with both counts and main's budget. With no ledger, compare the head's count with its held budget alone. Growth prints as a percentage where main's count is nonzero and available.
- **Growth credit** comes from moves into the layer that the budgets file has and main's ledger lacks, compared as a multiset of whole move lines. A move giving instructions credits that count. For moves giving CPU seconds alone, multiply their total seconds by the receiving layer's count at main and divide by its CPU seconds at main, rounding down to whole instructions. A missing or zero count or CPU figure at main gives no credit for CPU seconds. Outgoing moves give no growth credit to the source. The red line or a warning names the floor or credit where it changes the growth allowance used to judge growth past 2%, or the user's accepted level alone where it sets the allowance.
- **Drift** warns when main's count is at least 8% above its lowest and within its held budget. The warning names the layer, main's count, its lowest and its held budget.
- **When counts are unavailable**, the CPU comparison runs as a warning for each layer budgeted in instructions. A run that cannot count gives one line saying why instruction enforcement is off. In a counted run, a layer with any missing binary count at the head or at main gives one line naming the layer and the missing side or sides. Layers with CPU budgets alone retain their CPU verdict.
- **A red names its cause.** An instruction red gives the layer, its head count, held budget, ceiling, main count and growth percent where available. With a ledger, a CPU red continues `, ceiling CEILING s` where the row has a ceiling and then `, at main MAIN CPU s`, the ceiling printed as the budget is and the figure with two decimals. In the same entry of `over_budget` or `warnings`, one per line, follow:
  - the binaries that grew in the layer against main, largest growth first, at most ten. A binary's growth is its figure in the layer at the head less the figure in the layer at main in the verdict's measure. The figure at main is absent where the ledger lacks it or places it elsewhere. Each line names the binary, its growth, and its figures at the head and at main, or says it is new;
  - under each binary, at most five of its tests that grew by wall, largest growth first. A test's growth is its wall in the run less the median of the ledger's walls for it, or its whole wall where the ledger lacks it. Each line names the test, its wall and the ledger's median, or says it is new;
  - a line saying how many more there are, where either list was cut.

  The row's `grew` holds the same, uncut: for each binary, `binary_id`, `cpu_s`, `main_s` (null where new) and, for an instruction finding, `instructions` and `main_instructions`. Each entry also has `tests`, each with `test_name`, `wall_s` and `main_wall_s` (null where new). A binary the run didn't run grows only by its placement, and lists no test. The text of these lines past the first line's form is the code's.
- **An example,** a layer held to 131 s at the head and at main (section 9's example), its run's figure 132.00 s against 120.00 s at main. The layer is within at main, so the red is the change's: `layer over budget: temper default domain: 132.00 CPU s, budget 131 s, ceiling 141 s, at main 120.00 CPU s`, and below it the binaries that grew. At 132.00 s against 131.50 s at main, both over and 0.50 s apart, within the noise of 6.575 s: no red of the change's, and the warning `layer over budget at main: temper default domain: 131.50 CPU s at main, budget 131 s; 132.00 CPU s at the head`. At 138.00 s, 6.50 s apart, still within it, with the same warning; at 139.00 s, 7.50 s apart, past it: the change's red. A layer of 12.00 s at main, held to 11 s, has the noise's floor, a second: 12.90 s at the head is within it, 13.10 s past it.

## 9. The held budget

- **The ceiling** is the user's: a layer's `cpu_s` in `tools/tiers/budgets.toml` with the moves applied (section 10). Only the user raises it (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **The budget the gate holds** is lower where main's figures have fallen: the lower of the ceiling and the layer's lowest figure at main since the ceiling was last set, plus the larger of a tenth and a second, rounded up to a whole second, as the starting figures were (section 6; draft, 8). The sum is taken to the millisecond before it is rounded up. With `ceiling_s` and `lowest_s` the ledger's entry for the layer:
  - **at main:** `min(ceiling_s, ceil(lowest_s + max(lowest_s / 10, 1)))`;
  - **at the head,** with `C` the layer's ceiling there and `net` the seconds of the new moves into the layer less those of the new moves out of it, a new move being one the run's budgets file has and the ledger's `moves` lacks, counted as a multiset of whole moves: where the ledger has an entry for the layer and `C - net` equals its `ceiling_s` to the millisecond, `min(C, held at main + net)`; otherwise `C`. "Otherwise" is a run with no ledger, a layer the ledger has no entry for, and a ceiling the change sets: no figure at main has been seen since it was set.
- **The fold** (section 7's `budgets`): for each layer the run ran (section 4.1) with a ceiling at the head, `ceiling_s` is `C`, and `lowest_s` is its figure in the run's `ledger.json` where its held budget was `C` by "otherwise", and otherwise the lower of the ledger's `lowest_s` plus `net` and that figure, at least 0. A layer with no ceiling at the head has no entry.
  - **Its figure in the run's `ledger.json`** is the sum of the figures at main (section 8) of the binaries the run's `ledger.json` places in the layer, by their `cpu` there: the layer's figure at main should the candidate land, each binary's median of its last figures, never the run's one sample (draft, 5). A layer with none of them counts 0.
  - **Section 8's row `lowest_s`** is the same figure the fold records: `summary` folds the run's binaries as section 7 does, with the ledger it is given, so a `cpu-summary` run, which writes no ledger, prints what the fold would record.
- **So it only falls.** After each landing, a layer's budget falls to its figure at main plus the tenth, where that is lower: what a repair wins is kept, and the next change can't spend it (draft, 8). Nothing raises it but the ceiling's change. Each row prints both the ceiling and the held budget (section 4).
- **An example.** A layer with ceiling 141 s and `lowest_s` 118.4 is held at main to `min(141, ceil(118.4 + 11.84))`, 131 s, and at the head to the same where the change moves nothing and sets no ceiling. A landing whose `ledger.json` gives the layer 125.00 s leaves `lowest_s` at 118.4; one that gives it 112.00 s leaves 112.0, so the next gate holds the layer to 124 s. A layer of one binary whose `cpu` is `[120.0]` at main, measured whole at 112.00 s, has `[120.0, 112.0]` in the run's `ledger.json`: its landing leaves `lowest_s` at 116.0, not 112.0. A change that sets the ceiling to 150 s is held to 150 s at its gate, and its landing records the layer's figure in its `ledger.json` as `lowest_s`. A change adding a 12 s move into the layer is held to `min(153, 131 + 12)`, 143 s, and its landing records `lowest_s` as the lower of 130.4 and that figure.

- **The instruction budget at main, before the accepted level,** is the lower of its instruction ceiling and its lowest count plus the larger of a tenth of that count rounded up and 10,000,000,000 instructions. With no known lowest it is the ceiling. When the ledger lacks an instruction entry, use the current ceiling before any new instruction moves and seed the lowest from main's complete layer count, where available.
- **At the head, before the accepted level,** where the ceiling matches main's after new instruction moves, use main's budget from the preceding calculation plus the net moved into the layer, bounded by zero and the new ceiling. Otherwise use the head's ceiling. With no ledger use the head's ceiling. The growth floor and credit in section 8 apply only to the growth check.
- **The accepted level floors both held instruction budgets.** Where the layer's table at the head has `accepted_instructions`, take the larger of each calculated budget and that level. Apply the floor after the moves and ceiling calculations. The held budget may exceed its ceiling, by the user's acceptance. CPU budgets have no accepted-level floor.
- **The instruction fold** writes the head's ceiling and the lower of its carried lowest and the layer's complete count in the folded ledger. The carried lowest includes the net of new instruction moves, with a floor of zero. A changed ceiling outside those moves starts a new lowest. A run or layer without a complete head count cannot lower it. If neither main nor the fold supplied a lowest, the ledger records null. The accepted level does not change the recorded lowest. With a fixed ceiling and accepted level and no moves, the lowest and the resulting budget only fall.

## 10. Moves down, and every figure the user's

- **A move** gives CPU seconds and optionally instructions of one layer's budget to a layer below it, in any tree, with the tests a change places lower (`skein/docs/foundation/testing-strategy.md`, 9; draft, 8). It is a table of the profile's `moves` array, in `tools/tiers/budgets.toml`:

```toml
[[gate.default.moves]]
from = "temper.simulated"
to = "temper.domain"
cpu_s = 12
why = "the store's own mechanics placed below the one-process world"
```

  - `from` and `to`, each `TREE.LAYER`: one of the five trees, a dot, and a name of `tools/tiers/layers.toml`'s `layers` list. The repository's checks are no layer of a pyramid, and move nothing;
  - `cpu_s`, the seconds it moves, a finite positive number, never a boolean;
  - `instructions`, the number moved if present, a positive integer. A move without it leaves instruction ceilings and held budgets alone.
  - `why`, a nonempty string, a sentence for the change's review.
- **`budgets()` refuses** (`ValueError`) a move with any other key or a value outside the form; a `to` whose layer isn't below `from`'s in the `layers` list, "below" as `tools/tiers/layers.md`, 2 has it; a `to` tree with no table for the profile; and moves that take any layer's CPU or instruction ceiling below 0. So each move goes down and leaves the sum of the ceilings as it was, which is all the gate checks of it. A move up, sideways or into another tree's same layer is a raise, the user's (8.1).
- **A layer's ceiling** for a profile is its table's `cpu_s`, or 0 where it has no table, plus the profile's moves into it less its moves out of it. A layer with neither a table nor a move into it has no ceiling. A layer a move gives its first seconds has a ceiling, so it isn't over for having no table (section 4), and its reference is its profile's `references` entry. Instruction ceilings use the same transfers, starting at the row's `instructions` or at zero for a layer receiving its first instructions. The layer's table selects the verdict's measure (section 4).
- **Applied once to the held budget.** A move stays in the file, and the ceiling has it at every run. The ledger's `moves` are those main's held budgets have taken: a move the ledger lacks changes the held budget once, in each measure it carries (section 9). Moves are matched separately in each measure by endpoints, amount and reason.
- **Folding a move into the figures,** a later change that writes its seconds into both layers' `cpu_s` and removes the line, leaves each ceiling, and so each held budget, as it was. A move removed alone raises its `from` layer's ceiling and lowers its `to` layer's, a change to the ceilings that is the user's, as any change to a `cpu_s` is; each held budget it changes restarts at its ceiling (section 9).

### 10.1 Every figure the user's

- **The gate alone refuses a raise** (`temper/docs/design/draft/test-budgets.md`, 8; the user, 11-18 and 11-22). It compares the candidate merged onto main with main's tip and names each raise or addition listed below. Lowering a figure, dropping a reason's line and a CPU move down pass.
- **A change of that file alone passes the gate's figures check,** whatever it does there (the user, 11-18 12:4xZ): it is how the user's own figures land, by a drafter's mechanical change on their word. So does a member of a batched landing whose own change is that file alone, wherever the batch places it (the user, 11-24): its neighbours' paths are not its own. The refusal is for a change that touches any other path too, as every goal's landing does: a raise a goal's change makes alone still meets it at the goal's landing.
- **A budget check reason** (section 13) added against BASE in a change that also touches another file gives `budget check reason added: PROFILE budget check reason BINARY: REASON`. REASON is the whole line as JSON. A merged tip does not excuse this finding against BASE.
- **The check:** `python3 -I -B tools/tiers/figures.py --base BASE [--head HEAD] [--merged REV]…`, from the checkout root, BASE, HEAD and each REV any commit or tree git names, HEAD `HEAD` by default. It reads committed trees through git, never the working tree, as `tools/citations/citations.py` does. It exits 0 where it passes, 1 where it refuses, and 2 for a usage error, a git failure, a file that isn't TOML, a CPU or wall figure that isn't a finite number, or an instruction figure that isn't a positive integer.
  - **What changed:** `git diff --no-renames --name-only BASE HEAD`. Without `tools/tiers/budgets.toml` among the paths it passes, printing `figures: tools/tiers/budgets.toml unchanged since BASE`; with that path alone it passes, printing `figures: tools/tiers/budgets.toml alone changed since BASE: the user's figures`. Otherwise it judges a batch's members one by one where HEAD is a batch (below), and else compares the file's figures.
  - **What it reads:** `tools/tiers/budgets.toml` at BASE and at HEAD (`git show REV:tools/tiers/budgets.toml`), each as TOML, a side without the file read as an empty table; and HEAD's `tools/tiers/layers.toml`, its `layers` list alone, for the moves' direction. It reads the figures at the places below and leaves the file's form to `budgets()` (section 2), which every suite's run applies: a key it doesn't name is no figure of its.
- **A merge's figures are its own** only where raised or added against BASE and every `--merged REV`: what a merged tip brings already landed where it came from, and what the branch brings was checked when it reached the branch. Each REV is compared with HEAD by the same rules, including the changed-path short cuts, with HEAD's layers list. A short cut names no places and prints its own line with REV in place of BASE. The check keeps only BASE's findings whose places every REV also names, in BASE's order. Its count line is `figures: N raised or added since BASE and REV1[ and REV2…]`, then the repair line on a refusal. With no `--merged`, its output is unchanged. A tree HEAD supplies no history: the caller names the merged tips.
- **A batch's members are judged one by one.** HEAD is a batch where, with no `--merged`, its first-parent line reaches BASE through two or more commits and each of them is a merge: temper's merge train commits one merge per member onto the prefix below, the member's head its second parent (`temper/docs/design/domain/landing-prepare.md`, 1.2). Each member, lowest first, is checked as a single change is, with the commit below it as BASE and its own commit as HEAD, short cuts included. So its figures are compared only where its own change touches another path too. Its findings and lines are the single check's, the repair line left to the end.
  - **Its output:** first `figures: HEAD stacks M members on BASE: each judged by its own change`, M their number; then each member's lines: its short cut's line, or its findings and `figures: K raised or added since BELOW`; then `figures: N raised or added since BASE`, N the sum of the members' findings, and the repair line on a refusal. BELOW is the commit below the member, BASE as given for the lowest and the full commit id for the others. The exit is 1 where N is above 0. A batch whose second member raises temper's focused domain ceiling with other paths, beside a lowest member that changes the file alone, prints the following, with the ids and the last line cut here.

```
figures: HEAD stacks 2 members on 1a2b3c4d5e: each judged by its own change
figures: tools/tiers/budgets.toml alone changed since 1a2b3c4d5e: the user's figures
figure raised: default temper domain ceiling: 137.5 to 141.5
figures: 1 raised or added since 6f7a8b9c0d
figures: 1 raised or added since 1a2b3c4d5e
repair: read skein/docs/foundation/testing-strategy.md in full first, ...
```

  - **Anything else is one change:** a single merge, a commit on the line that isn't a merge, a line that doesn't reach BASE, a tree for either side, or a checkout without those commits (a shallow fetch) is compared whole, as above. A goal's landing is one merge, so a raise inside a goal still meets the refusal, and a train that squashes into main (`temper/docs/design/domain/landing-settings.md`, 1.1, `combine`) has no merges to tell apart.
- **The figures** are compared by place. CPU and wall values are rounded to three decimals before comparison. Instruction values are compared as integers. For CPU and wall limits, dropping a bound raises it to none, except that a missing layer CPU ceiling counts as 0 (section 4) and a dropped reason's line leaves the test its reference.
  - **a layer's CPU ceiling,** for each profile, group and layer, as section 10's third bullet gives it (`gate_ceilings` in `tools/tiers/cpu.py`), at HEAD less the seconds of HEAD's new moves into it that go down. A new move is one HEAD's moves for the profile have and BASE's lack, counted as a multiset by endpoints, seconds and reason. It goes down where its `from` and `to` are each `TREE.LAYER` of one of the five trees and a name of the layers list, `to`'s layer below `from`'s. A move down or a raise of its seconds passes this comparison. Removing a move alone or lowering its seconds raises its `from` layer's CPU ceiling. Folding its seconds into the figures leaves each CPU ceiling as it was.
  - **a profile's ceilings' sum,** the sum of its layer tables' `cpu_s`, which moves leave as they find it, named only where no layer's ceiling of the profile is: so a raise of a layer's `cpu_s` can't pass as a new move out of it;
  - **a reference:** each entry of a profile's `references`, by layer; and, for each layer with a table at HEAD, its reference as section 4 takes it, its `reference_s` or else its profile's entry;
  - **an ideal:** each group's `ideal_s`, and each layer table's `ideal_s` for a layer with a table at HEAD;
  - **a layer's instruction ceiling and ideal.** Each layer table's `instructions` and `ideal_instructions` are checked directly, even when a move changes its effective instruction ceiling. Adding, raising or removing either field is refused. Removing an instruction ceiling can return the layer to CPU judgment.
  - **a layer's accepted level,** its table's `accepted_instructions` (section 8): adding or raising it is refused. Lowering or removing it passes, since without it the layer's growth has its usual allowance alone.
  - **a move's instruction amount.** Adding or raising its `instructions` is refused, even for a move down. Lowering or dropping the amount passes this comparison. Within a profile, match moves by endpoints and reason. Match equal amounts first, then compare the remaining amounts in descending order.
  - **a reason's line,** by profile, binary and test: added, or its `cpu_s` raised. A side holding two lines for one test, which `budgets()` refuses, counts its larger;
  - **a suite's limit:** each pair table's (every top-level table but `gate` and `tools`) and each `tools` suite's `cpu_suite_s`, `cpu_test_s` and `wall_hard_s` (`tools/tiers/design.md`, 7.3 and 7.5). A pair's `auto_reap` is no figure.

  Dropping a whole layer table lowers its CPU ceiling and removes any instruction figures it held. Dropping a whole group's table drops its `ideal_s`, a raise. A tree with no table for a profile is never over (section 4).
- **Its refusal lines** use `figure raised: PLACE: BASE_FIGURE to HEAD_FIGURE` for raises and `figure added: PLACE: HEAD_FIGURE` for additions, in the order of the list above. Each figure prints as Python's `str` gives it, or `none` where absent. For a CPU ceiling, HEAD_FIGURE is its ceiling at HEAD less what new moves excuse. The count line is `figures: N raised or added since BASE`, followed by the repair line on a refusal. Where it compares and refuses nothing, it prints `figures: 0 raised or added since BASE` alone. PLACE is one of `PROFILE GROUP LAYER ceiling`, `PROFILE ceilings' sum`, `PROFILE references LAYER`, `PROFILE GROUP LAYER reference`, `PROFILE GROUP ideal`, `PROFILE GROUP LAYER ideal`, `PROFILE GROUP LAYER instruction ceiling`, `PROFILE GROUP LAYER instruction ideal`, `PROFILE GROUP LAYER instruction accepted level`, `PROFILE move FROM to TO instructions (REASON)`, `PROFILE reason BINARY TEST`, `PAIR KEY` and `tools SUITE KEY`. A move's reason prints as Python's `repr` gives it, with ` #2`, ` #3` and so on after the parentheses for later ranks with the same endpoints and reason. A change raising temper's focused domain `cpu_s` from 136 to 140, so its ceiling, with the 1.5 s moved into it, from 137.5 s to 141.5 s, raising temper's focused simulated `reference_s` from 2 to 3 and adding a line for a test prints the following, with the last line cut here.

```
figure raised: default temper domain ceiling: 137.5 to 141.5
figure raised: default temper simulated reference: 2 to 3
figure added: default reason temper-system::suite suite::chat::a_whole_chat_lands: 3
figures: 3 raised or added since 1a2b3c4d5e
repair: read skein/docs/foundation/testing-strategy.md in full first, ...
```

- **At the gate.** The gate runs `python3 -I -B tools/tiers/figures.py --base BASE` on the merged candidate, with main's tip as BASE (`.config/gate/README.md`, item 17). It tells a batch's members apart where its checkout holds the candidate's commits down to BASE. Its runner makes a red case of each refusal. The gate's runner, outside this repository, takes it up when it follows (`temper/docs/design/draft/test-budgets.execution.md`, 2).
- **On delivery.** Where the delivery's changed paths include `tools/tiers/budgets.toml`, the delivery check reads that file and `tools/tiers/layers.toml` from the delivered tree alone. A file `budgets()` accepts passes. A refusal is reported as `budgets` with its error and the repair line (`tools/pre-pr/design.md`, 4, check 7; the user, 11-22).
- **The repair line.** Each budget red, reference red, figure refusal at the gate and budgets validation failure on delivery ends with one line sending its repair to the strategy (draft, 8):

```
repair: read skein/docs/foundation/testing-strategy.md in full first, then ask 8.1's three questions in order: does the test belong to its tier; if it does, can it be made faster; if neither, take the measurements to the user.
```

  Its text is one constant of `tools/tiers/cpu.py`, which `figures.py` reads. Every entry of `summary`'s `over_budget` (section 4) has it on a line of its own. Check-loss reds follow it with the reason line instructions (section 13). This covers layer budgets, per-test references (`tools/tiers/references.md`, 6), and the budgets for pairs and tools suites. A warning carries none, nor does a red that is no budget's, such as a failed test. The delivery check's failures keep each entry whole as their logs (`tools/pre-pr/design.md`, 4), so it reaches a goal there too.

## 11. Later

- **The suites run through nextest** (`tools/tiers/run`, section 7's collector in `tools/tiers/design.md`) are not summed by layer, but the real pairs (section 4.2). The focused and fuzzy suites run through one-process (`tools/tiers/design.md`, 11).

## 12. Open questions

- **jig's layer ideals.** The draft estimated jig's ideal with its worlds placed by hand. `tools/tiers/layers.toml` places most of them in `simulated` and `protocol`. Under it, jig's `domain` ideal of 17 s sits beside a figure of 2.4 s, and its `simulated` ideal of 3 s beside 99.5 s. Ideals are reported and never enforced. A split re-estimated under the layers' placement is the user's figure.
- **Counting's cost in CPU.** A counted process's system time carries about 23 µs at each of its context switches (`tools/tiers/design.md`, 11.4, *Its cost*). The budgets and the ledger's lowest figures were measured uncounted, so a layer whose tests switch often, the real loop's above all, which has left the gate for its pairs (section 4.2), may read higher in CPU from the first counted gate on, with no change of its own. Its budget stays the user's figure (section 9), and nothing here moves one. Whether a layer that shows this needs a word from the user is open until the first counted gates show it.

## 13. Checks done at main

- **Faster means doing the same checks for less.** A reduction in checks needs a
  reason, even where the layer uses fewer instructions or less CPU. A
  budget-driven loss goes to the user, the third question in
  `skein/docs/foundation/testing-strategy.md`, 8.1.
- **The figures.** The summary sums each process's `checks`
  (`tools/tiers/design.md`, 11.8) by binary and kind, across the suite's
  `tests` and `fresh` processes. The ledger keeps the last whole run's totals
  for each binary (section 7), with the counts as they were where it keeps
  an unlisted or cut-short binary. A results file without process `checks`
  reads those records as empty. A check kind matches
  `[a-z][a-z0-9-]{0,31}` and its recorded total is a positive integer.
- **The comparison.** With main's ledger, compare its binaries in the
  run's selected layers (section 4.1). A caller's test filter skips the
  comparison of checks done with one warning. An unlisted binary is skipped
  where its package was left out or the record predates package scope.
  A listed binary with no tests, an unlisted binary in a selected package,
  or an absent binary is judged with zero tests and checks. A binary moved
  into an unselected layer is judged with zero tests and checks in its
  layer at main, even if it is unlisted. Only a `moved` reason covers
  its lost checks. Compare each binary's names with the names it ran. For a
  cut-short binary, compare against all its selected names, including
  `unobserved`, and skip its check counts. A newly ignored test can lose
  its name from the run.
- **A move across modules.** When a name at main is absent from the run,
  it can pair with a new test in the run that has the same final `::`
  segment within its binary or in a binary new to main's ledger. A test is
  new where main's ledger lacks its name in the binary that ran it. Pair
  one to one within each binary first, then into new binaries, sorting by
  test name and then binary ID.
  The summary lists both ends in `check_moves` and prints up to ten
  pairs per source binary, with the number beyond them.
- **Missing names.** The summary puts one note per binary in `warnings`
  as `checks note: BINARY: N tests main ran didn't run at the head: NAME, ...`.
  It lists up to ten names and the number beyond them. Missing names
  do not affect the verdict.
- **Check counts.** Compare each kind's total at the head with main, with
  an absent kind counting zero. Sum over each connected group of whole
  binaries joined by automatic moves. If a group has a cut-short member,
  compare each whole member's counts against its own at main. Fixed seeds make the totals
  comparable without a tolerance.
- **A reason** is a table of the profile's `check_reasons` array in
  `tools/tiers/budgets.toml`.

```toml
[[gate.default.check_reasons]]
binary = "temper-engine::domain"
checks = { seed = 32 }
kind = "removed"
why = "The code that required this sweep was removed."
by = "The change's reviewer"
```

  `binary`, `why` and `by` are nonempty strings. `checks` is a nonempty
  table from kind to the exact number of checks lost. `kind` is `removed`
  for code that went away, `duplicate` for checks another test supplies,
  `moved` for a move to a layer the run leaves out, or `budget`. A budget
  reason's `by` must start with `the user`. `budgets()` refuses other keys and values outside
  the form.
- **When a reason applies.** A budget reason must already be in main's
  ledger and still be in the budgets file. It lands in a budgets-only
  change (10.1). The run that uses it folds it into
  `spent_check_reasons`; the ledger of a landing makes that use final.
  An unused approval can wait for its change without warning. Every
  other kind applies only while main's ledger lacks the whole line.
  Anyone can prune a spent budget reason or a landed reason of another kind.
- **The verdict.** Each binary or group that lost checks without a reason
  has a red in `over_budget`, with or without `--gate`, beginning
  `layer checks lost: TREE PROFILE LAYER: BINARY`. It gives the main and
  head totals for each kind of check lost. The repair line (10.1) and
  the line naming the reason table follow. Every applied reason is named in
  the summary and its printed verdict, including a red-free verdict. A
  new non-budget reason applying to no loss gives a `stale check reason: …` warning.
