# Budgets by layer

Provisional, 2026-11-16. What the focused and fuzzy suites may cost, by layer of the pyramid: the gate's tables in `tools/tiers/budgets.toml`, how `tools/tiers/one-process` sums a run by layer and judges it, where the starting figures come from, the ledger that judges a partial run whole, the budget the gate holds and the moves down. Written from `temper/docs/design/draft/test-budgets.md`, 3, 4, 5, 6, 8 and 9. The layers and the binaries in each are `tools/tiers/layers.md`'s.

## 1. In one page

- **A budget for each layer** (section 2): each tree's focused and fuzzy suites have a CPU budget for each layer the tree has (`tools/tiers/layers.md`, 2), and the repository's checks one of their own. Beside each layer's budget are its per-test reference and, where the draft gives one, its share of the ideal. Each tree has an ideal for each suite.
- **Each process placed** (section 3): `tools/tiers/one-process` gives each binary it runs a tree and a layer, by `tools/tiers/layers.md`'s rules and table, and records them with the binary's processes.
- **The summary by layer** (section 4): a one-process run's CPU summed by tree and layer, beside each layer's budget, ideal and reference, and each tree's total beside its ideal.
- **Red at the gate, and at a delivery check with main's ledgers** (section 5): a layer over its budget is red where the caller asks for the gate's verdict (`--gate`), which the gate does and a delivery check does on a builder that keeps main's ledgers (the user, 11-17), and a warning in every other run (the user, 11-16; `skein/docs/foundation/testing-strategy.md`, 8.1).
- **Starting figures** (section 6): the CI builder's gate runs of 11-16, their median by tree, suite and layer, plus a tenth and at least a second.
- **The ledger** (section 7): main's figures for a suite, each binary's last CPU figures and each test's last walls, which the gate passes to its run and which every run writes again with its own figures folded in. The gate's runner, outside this repository, keeps it.
- **A partial run judged whole** (section 8): a layer's figure is what the run measured plus main's for the binaries it didn't run, a binary's figure at main the median of its last three; a layer over its budget is the change's red where main is within it or the change adds more than the noise, a twentieth of the layer at main, and its line names the binaries and tests that grew.
- **The held budget** (section 9): the lower of the user's ceiling and the layer's lowest figure at main since the ceiling was set, plus a tenth, so a budget only falls.
- **Moves down** (section 10): a line beside the budgets moves CPU seconds from one layer to a layer below it, with the tests a change moves; the gate checks that it goes down.
- **Each test held to its reference** is `tools/tiers/references.md`'s: new and flagged tests run alone, outside every sum here, their CPU in the ledger beside their walls (section 7), and the reasons' lines beside the budgets.

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

- **A profile's table** holds `references`, `moves` where it has any (section 10), `reasons` where it has any (`tools/tiers/references.md`, 2), and one table for each **group**. A group is one of the five trees by name (`skein`, `smith`, `jig`, `temper`, `hearth`), or `repository` for the repository's checks (`tools/tiers/layers.md`, 4).
- **`references`** is an inline table that maps a layer's name to the per-test reference of a test in that layer, in CPU seconds (`temper/docs/design/draft/test-budgets.md`, 6). It names layers from `tools/tiers/layers.toml`'s `layers` list, and `checks` for the repository's checks.
- **A group's table** holds `ideal_s`, the group's ideal for the suite, and a table for each layer it has: for a tree, layers named in `layers`; for `repository`, `checks` alone.
- **A layer's table** holds `cpu_s`, the layer's budget for the suite. That figure is the user's ceiling: raising it is the user's (`skein/docs/foundation/testing-strategy.md`, 8.1). The table may also hold `ideal_s`, the layer's share of the group's ideal, where the draft gives one, and `reference_s`, the tree's own per-test reference for the layer, which replaces the profile's.
- **Every value** is a finite positive number, never a boolean. Nothing else is allowed in these tables. The moves have their own form (section 10), and the reasons' lines theirs (`tools/tiers/references.md`, 2).
- **`budgets(path, pairs=None, layers=Path('tools/tiers/layers.toml'))`** (`tools/tiers/design.md`, 10.2) reads the layers' names from the `layers` file's `layers` list. It reads that file only when the budgets file has a `gate` table. It raises `ValueError` on:
  - a key or value outside the form above;
  - a group that isn't one of the six;
  - a layer the list doesn't name, `checks` under a tree, or any other layer under `repository`;
  - a group without `ideal_s`, or a layer without `cpu_s`;
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

- **`layers`:** one row for each tree and layer that has a process in the run, with these keys:
  - `tree` and `layer`, the layer null for an unplaced binary;
  - `cpu_s`, the sum of its processes' CPU, of the suite's kinds (`tests`, `fresh`, `alone`), as the suite's sum counts them. A `reference` or `empty` process, which measures one test alone after the suite (`tools/tiers/references.md`, 4), counts in no sum: neither a layer's, nor the suite's, nor a binary's figure at the head (section 8), nor the ledger's (section 7);
  - `ceiling_s`, the layer's ceiling for the run's profile: its table's `cpu_s` with the moves applied (section 10), or null where it has none;
  - `budget_s`, the budget the run holds it to: its ceiling, or with a ledger its held budget (section 9); null where it has no ceiling;
  - `ideal_s`, its layer table's, or null;
  - `reference_s`, its own `reference_s` or else its profile's `references` entry, or null;
  - `over`, a boolean.

  Rows are ordered by tree (`skein`, `smith`, `jig`, `temper`, `hearth`, `repository`), then by `results.json`'s `layers` list. `checks` and the unplaced row come last.
- **`trees`:** one row for each tree that has a process in the run, with these keys:
  - `tree`;
  - `cpu_s`, its rows' sum;
  - `ceiling_s` and `budget_s`, the sums of the ceilings and of the budgets of every layer of the tree that has a ceiling under the run's profile, or null when the tree has no table there;
  - `ideal_s`, the tree's `ideal_s`, or null.
- **The budgets** are `limits['gate'][profile]`, the run's profile in `budgets()`'s result. With none for the profile, every `ceiling_s` and `budget_s` is null and nothing is over.
- **A row is over** in either case:
  - its `cpu_s` is more than its `budget_s`;
  - it has no `budget_s`, its layer isn't null, and its tree has a table for the profile. A tree has a budget for each layer it has (`temper/docs/design/draft/test-budgets.md`, 4). Room for a layer new to a tree comes from the user's raise, or from a move down (section 10).

  With a ledger, section 8 narrows `over` to the change's red.

  An unplaced row is never over. It gives the warning `unplaced: BINARY_ID has no layer (tools/tiers/layers.md)`, one line for each such binary.
- **The line for a layer over its budget** reads `layer over budget: TREE PROFILE LAYER: FIGURE CPU s, budget BUDGET s`, or ends `no budget` where the row has none. `FIGURE` has two decimals and `BUDGET` is the file's figure as Python's `str` gives it.
- **With `gate`,** each such line is in `over_budget`. The verdict keeps its order (`tools/tiers/design.md`, 10.2): `over budget` when `over_budget` holds any line, a layer's or the pair's own; else `starved` for a starved run; else `within budget` when the profile has gate tables or a pair table; else `no budget set`.
- **Without `gate`,** each such line is in `warnings`, and the verdict is what it would be with no gate tables: a layer over its budget is a warning there, never red (the user, 11-16).
- **A selection** (a run with `-p`) sums only what it ran. Without a ledger, a layer's figure in it is a lower bound of the layer's whole figure, and is judged as it stands. With main's ledger it is judged whole (section 8).
- **`print_summary`** prints, after the suite's totals, a line for each `layers` row and each `trees` row: the figure, the budget, the ceiling, the ideal and, for a layer, the reference, and with a ledger the figure at main, with `none` where null. Each warning or over-budget line follows, as today. The exact text of the rows is the code's; the over line is the form above, and section 8's with a ledger.

## 5. Who asks for the gate's verdict

- **`tools/tiers/one-process [--out DIR] [--budgets PATH] [--walls PATH] [--ledger PATH] [--gate] PROFILE [NEXTEST_ARGS...]`** (`tools/tiers/design.md`, 11.9).
  - **`--budgets`:** defaults to `tools/tiers/budgets.toml` where that file exists, so every run compares its layers with the repository's budgets. Without the file, no budgets apply.
  - **`--ledger`:** main's ledger for the profile (section 7), read with `ledger()` before anything runs; a ledger `ledger()` refuses, or one for another profile, is an invalid invocation (exit 2). It is passed to `execute_one_process` as `ledger`. None by default.
  - **`--gate`:** asks for the gate's verdict. It is passed to `execute_one_process` as `gate`, and a layer over its budget then exits 1, as a CPU budget passed does.
  - **With `--again`,** `--gate` is an invalid invocation, as `--budgets`, `--walls` and `--ledger` are.
- **`execute_one_process(command, env, directory, profile, limits, config=Path('.config/nextest.toml'), walls=None, gate=False, ledger=None)`:** gives `gate` and `ledger`, the ledger as `ledger()` read it or none, to `summary`, and writes the run's `ledger.json` (section 7) once the summary is made.
- **`tools/tiers/cpu-summary DIR [--budgets PATH] [--pair PAIR] [--expect ID ...] [--ledger PATH] [--base DIR] [--gate] [--json]`:** gives `gate`, the ledger `--ledger` names as `ledger()` reads it, and the base's run directory `--base` names (`tools/tiers/references.md`, 5 and 6), to `summary`. It writes no ledger. Its exit status keeps its rule: 1 when `over_budget` holds a line; 2 for a ledger refused.
- **The gate** passes `--gate` in both suites' commands, and `--ledger` with main's ledger for the suite where its runner keeps one (`.config/gate/README.md`, item 4; section 7). Where its run flagged a test, the runner runs it alone at the base and judges again with `--base` (`tools/tiers/references.md`, 5). The gate's runner, outside this repository, takes them up when it follows (`temper/docs/design/draft/test-budgets.execution.md`, 2). Until then the comparison prints and holds nothing.
- **The delivery check** passes both where its builder keeps main's ledger for the suite, in the directory `TIERS_LEDGER` names, and `ledger()` reads it: a layer the delivery takes over its budget, or a test past its reference, refuses the delivery, which hears it before its pull request, and the gate judges again with its own ledger (`tools/pre-pr/design.md`, 4; `temper/docs/design/draft/test-budgets.md`, 3). Without one it passes neither, and says why. The gate's runner copies main's ledgers to the builders that run goals once it follows (`temper/docs/design/draft/test-budgets.execution.md`, 2).
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

- **What it is:** main's figures for one suite, from the gate runs whose candidates landed (`temper/docs/design/draft/test-budgets.md`, 5): each binary's last CPU figures, each test's last walls and, where a gate ran it alone, its last CPU figures alone (`tools/tiers/references.md`, 4), each layer's held state (section 9) and the moves its held budgets have taken (section 10). The gate's runner, outside this repository, keeps one for each profile beside its reports (`temper/docs/design/draft/test-budgets.execution.md`, 2). The repository holds none.
- **Its form:** one JSON object, written as `cpu.py`'s `save` writes a file:

```json
{"ledger": 3, "profile": "default",
 "binaries": {"temper-engine::domain": {"tree": "temper", "layer": "domain", "cpu": [60.4, 63.0, 61.2],
                                        "tests": {"forge::opens": [0.011, 0.012, 0.010]},
                                        "alone": {"forge::opens": [0.008]}}},
 "budgets": {"temper": {"domain": {"ceiling_s": 141, "lowest_s": 118.4}}},
 "moves": [{"from": "temper.simulated", "to": "temper.domain", "cpu_s": 12, "why": "..."}]}
```

  - `ledger`, the form's version, 3;
  - `profile`, `default` or `fuzzy`;
  - `binaries`, by binary id: `tree`, one of section 2's six groups; `layer`, a name of the `layers` list, `checks`, or null, as section 3 placed it in the run that wrote the ledger; `cpu`, a list of one to three CPU figures, each the CPU of its processes, of the suite's kinds (section 4), in one of the last runs that measured it whole (below), oldest first, whose median is its figure at main (section 8); `tests`, by test name, a list of one to three walls, that test's in the last runs that measured it, oldest first; `alone`, by test name, a list of one to three CPU figures, that test's alone (`tools/tiers/references.md`, 4) in the last runs that ran it alone, oldest first, for tests some landed gate ran alone: every name in `alone` is in `tests`;
  - `budgets`, by group, then by layer, for each layer with a ceiling at main: `ceiling_s`, that ceiling (section 10), and `lowest_s`, the layer's lowest figure at main since the ceiling was set (section 9);
  - `moves`, the moves of main's budgets file for the profile, each as section 10's form has it.

  Every figure is a finite number at least 0, never a boolean. Nothing else is allowed.
- **`ledger(path)`** (`tools/tiers/design.md`, 10.2) reads one and returns it as a dictionary. It raises `ValueError` on anything outside the form: another key, a version other than 1, 2 or 3, a profile other than the two, a group other than section 2's six, a layer that is neither a nonempty string nor null (`budgets` takes no null), a figure outside the above, a binary whose `cpu` is no list or has no figure or more than three, a test with no wall or more than three, an `alone` entry with no figure or more than three, or for a name not in `tests`, an empty id or name, a move without section 10's keys and their types. It reads neither the layers file nor a budgets file: a layer or a move main no longer has is no error, and counts as section 8 and 9 say. A form that changes moves `ledger` by one, and its reader reads the form before it too, since the runner's ledger outlives the landing that changes it.
  - **Versions 1 and 2.** Version 2, B4's, is version 3 with `cpu_s`, one figure, in place of `cpu`: each binary has exactly the keys `tree`, `layer`, `cpu_s`, `tests` and `alone`, and `ledger()` returns it with `cpu` the list of that one figure and no `cpu_s`. Version 1, B3's, is version 2 without `alone`: each binary has exactly the four other keys, and `ledger()` returns it with `alone` empty too. A binary of either is checked against its own version's keys, so a version 2 binary with `cpu`, or a version 3 one with `cpu_s`, is refused. So its callers read one form, version 3's, with the version read in `ledger`. Every run writes version 3.
- **Who passes it:** the gate, as `--ledger PATH` (section 5): main's ledger for the suite, at the candidate's base; and the delivery check, a copy the gate's runner left on its builder, a landing or two old at most. Every other run passes none, and its selection is judged as it stands (section 4).
- **Every run writes its own.** A one-process run that reaches its summary writes `ledger.json` in its directory (`tools/tiers/design.md`, 11.8): main's ledger as it will stand once the run's candidate lands. It is the given ledger, or an empty one, with the run's figures folded in:
  - **a binary the run measured whole,** every one of its selected tests `passed`, `failed` or `crashed` and none `starved`, `cancelled` or unobserved: its `tree` and `layer` as the run placed it, its `cpu` the ledger's last two figures for it, then the CPU the run measured for it (`[64.0, 61.2, 63.0]` from `[60.0, 64.0, 61.2]` and 63.0; a binary new to the ledger has the run's figure only, and one the run places elsewhere keeps its figures, as a test's walls are kept), and its `tests` as the run ran them, each with the ledger's last two walls for it, then the run's own. Its `alone`, for each test the run ran: the ledger's last two figures for it, then the run's own, where the run's `references` (`tools/tiers/references.md`, 4) give it a `cpu_s` that isn't null; otherwise the ledger's figures as they were, where it has any. A test the ledger had and the run didn't run leaves, from both;
  - **a binary the run cut short:** its ledger entry as it was, `alone` included, with the run's `tree` and `layer`, or none where the ledger lacks it, since its CPU is partial;
  - **a ledger binary the run didn't run:** kept, with its `workspace` entry's `tree` and `layer` (section 3), where `workspace` has it unlisted. It leaves where `workspace` lacks it (its target is gone), and where the listing named it and the run selected none of its tests;
  - **`budgets`** as section 9 folds them, and **`moves`** the run's budgets file's for the profile, an empty list without one.
- **The runner keeps** the `ledger.json` of the gate run whose candidate landed as main's ledger for that suite, and passes it to the next gate on top of that landing. Its first ledger is the `ledger.json` of a whole run of its own on main, with no `-p` and no `--ledger`: until then no selection is judged whole and no budget is held below its ceiling (draft, 5). The gate selects by package alone; a run that also filters tests writes a ledger that isn't main's.

## 8. A partial run, judged whole

`summary`'s `ledger` (section 4) is main's ledger as `ledger()` read it, or none. With one, `summary` refuses (`ValueError`) a nextest run, tools, a ledger whose `profile` isn't the run's, and a `results.json` without `workspace`.

- **A binary's figure at main** is the median of its ledger entry's `cpu` figures, as Python's `statistics.median` takes it: the middle of three, the mean of two, the one of one. One run's figure is a sample, and a landing keeps the samples that passed, so the last alone sits low; a test's figure at main is the median of its walls the same way (draft, 5). Wherever this section takes the ledger's figure for a binary, it takes this one: at main, at the head for a binary the run didn't run, and in its growth. A ledger whose `cpu` lists hold `[60.0, 64.0, 61.2]` and `[50.0, 54.0]` gives those binaries 61.2 s and 52.0 s at main.
- **A binary's figure at the head** is the CPU of its processes of the suite's kinds (section 4) where the run ran it; its figure at main where the run's `workspace` has it with `listed` false; nothing where the listing named it and the run ran none of its tests, or where `workspace` lacks it. It counts in the tree and layer the run places it in: its processes', or its `workspace` entry's.
- **A layer's figure at the head** is the sum of its binaries' figures there. **Its figure at main** is the sum of the figures at main of the binaries the ledger places there.
- **The rows** (section 4's `layers`) are one for each tree and layer with a binary at the head or at main, in section 4's order, and gain these keys:
  - `cpu_s`, its figure at the head;
  - `measured_s`, what the run measured there, the sum of its processes' CPU;
  - `main_s`, its figure at main, null without a ledger;
  - `budget_s`, its held budget at the head (section 9);
  - `main_budget_s`, its held budget at main (section 9), null without a ledger or where the ledger's `budgets` has no entry for it;
  - `lowest_s`, the `lowest_s` the run's `ledger.json` records for it (section 9), null where it has no ceiling;
  - `main_over`, true where its figure at main is more than its held budget at main;
  - `grew`, the binaries its over line names (below), empty where it isn't over.

  Without a ledger, `measured_s` is `cpu_s`, `main_s`, `main_budget_s` are null, `main_over` is false and `grew` empty. A tree's row (section 4's `trees`) gains `main_s`, the sum of its rows', null without a ledger, and its `cpu_s`, `ceiling_s` and `budget_s` are section 4's sums of these figures.
- **The red is the change's** (draft, 5). A row is `over` where it is over by section 4's two cases, and either it isn't `main_over`, or its figure at the head exceeds its figure at main by more than the noise: the larger of a twentieth of its figure at main and a second (draft, 5). The gate's whole runs measure the same tree's layers up to 4% apart, so a hundredth held nearly every change red where main was over; a layer over at main may so grow by less than the noise at a landing, and its line names it at every run. Its line goes where section 4 puts it: in `over_budget` with `gate`, in `warnings` without.
- **Main's red.** A row over by section 4's cases that is `main_over` and within the noise isn't `over`. It gives a warning in every run, `layer over budget at main: TREE PROFILE LAYER: MAIN CPU s at main, budget MAIN_BUDGET s; FIGURE CPU s at the head`, the figures with two decimals and the budget as section 4 prints one. It is main's red, as a failure the base has too is; the gate's runner reads it from `main_over`.
- **A red names its cause.** With a ledger, section 4's over line continues `, ceiling CEILING s` where the row has a ceiling and then `, at main MAIN CPU s`, the ceiling printed as the budget is and the figure with two decimals. In the same entry of `over_budget` or `warnings`, one per line, follow:
  - the binaries that grew in the layer against main, largest growth first, at most ten. A binary's growth is its figure in the layer at the head less its figure in the layer at main, its median above, which is nothing where the ledger lacks it or places it elsewhere. Each line names the binary, its growth, and its figures at the head and at main, or says it is new;
  - under each binary, at most five of its tests that grew by wall, largest growth first. A test's growth is its wall in the run less the median of the ledger's walls for it, or its whole wall where the ledger lacks it. Each line names the test, its wall and the ledger's median, or says it is new;
  - a line saying how many more there are, where either list was cut.

  The row's `grew` holds the same, uncut: for each binary, `binary_id`, `cpu_s`, `main_s` (null where new) and `tests`, each with `test_name`, `wall_s` and `main_wall_s` (null where new). A binary the run didn't run grows only by its placement, and lists no test. The text of these lines past the first line's form is the code's.
- **An example,** a layer held to 131 s at the head and at main (section 9's example), its run's figure 132.00 s against 120.00 s at main. The layer is within at main, so the red is the change's: `layer over budget: temper default domain: 132.00 CPU s, budget 131 s, ceiling 141 s, at main 120.00 CPU s`, and below it the binaries that grew. At 132.00 s against 131.50 s at main, both over and 0.50 s apart, within the noise of 6.575 s: no red of the change's, and the warning `layer over budget at main: temper default domain: 131.50 CPU s at main, budget 131 s; 132.00 CPU s at the head`. At 138.00 s, 6.50 s apart, still within it, with the same warning; at 139.00 s, 7.50 s apart, past it: the change's red. A layer of 12.00 s at main, held to 11 s, has the noise's floor, a second: 12.90 s at the head is within it, 13.10 s past it.

## 9. The held budget

- **The ceiling** is the user's: a layer's `cpu_s` in `tools/tiers/budgets.toml` with the moves applied (section 10). Only the user raises it (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **The budget the gate holds** is lower where main's figures have fallen: the lower of the ceiling and the layer's lowest figure at main since the ceiling was last set, plus the larger of a tenth and a second, rounded up to a whole second, as the starting figures were (section 6; draft, 8). The sum is taken to the millisecond before it is rounded up. With `ceiling_s` and `lowest_s` the ledger's entry for the layer:
  - **at main:** `min(ceiling_s, ceil(lowest_s + max(lowest_s / 10, 1)))`;
  - **at the head,** with `C` the layer's ceiling there and `net` the seconds of the new moves into the layer less those of the new moves out of it, a new move being one the run's budgets file has and the ledger's `moves` lacks, counted as a multiset of whole moves: where the ledger has an entry for the layer and `C - net` equals its `ceiling_s` to the millisecond, `min(C, held at main + net)`; otherwise `C`. "Otherwise" is a run with no ledger, a layer the ledger has no entry for, and a ceiling the change sets: no figure at main has been seen since it was set.
- **The fold** (section 7's `budgets`): for each layer with a ceiling at the head, `ceiling_s` is `C`, and `lowest_s` is its figure in the run's `ledger.json` where its held budget was `C` by "otherwise", and otherwise the lower of the ledger's `lowest_s` plus `net` and that figure, at least 0. A layer with no ceiling at the head has no entry.
  - **Its figure in the run's `ledger.json`** is the sum of the figures at main (section 8) of the binaries the run's `ledger.json` places in the layer, by their `cpu` there: the layer's figure at main should the candidate land, each binary's median of its last figures, never the run's one sample (draft, 5). A layer with none of them counts 0.
  - **Section 8's row `lowest_s`** is the same figure the fold records: `summary` folds the run's binaries as section 7 does, with the ledger it is given, so a `cpu-summary` run, which writes no ledger, prints what the fold would record.
- **So it only falls.** After each landing, a layer's budget falls to its figure at main plus the tenth, where that is lower: what a repair wins is kept, and the next change can't spend it (draft, 8). Nothing raises it but the ceiling's change. Each row prints both the ceiling and the held budget (section 4).
- **An example.** A layer with ceiling 141 s and `lowest_s` 118.4 is held at main to `min(141, ceil(118.4 + 11.84))`, 131 s, and at the head to the same where the change moves nothing and sets no ceiling. A landing whose `ledger.json` gives the layer 125.00 s leaves `lowest_s` at 118.4; one that gives it 112.00 s leaves 112.0, so the next gate holds the layer to 124 s. A layer of one binary whose `cpu` is `[120.0]` at main, measured whole at 112.00 s, has `[120.0, 112.0]` in the run's `ledger.json`: its landing leaves `lowest_s` at 116.0, not 112.0. A change that sets the ceiling to 150 s is held to 150 s at its gate, and its landing records the layer's figure in its `ledger.json` as `lowest_s`. A change adding a 12 s move into the layer is held to `min(153, 131 + 12)`, 143 s, and its landing records `lowest_s` as the lower of 130.4 and that figure.

## 10. Moves down

- **A move** gives CPU seconds of one layer's budget to a layer below it, in any tree, with the tests a change places lower (`skein/docs/foundation/testing-strategy.md`, 9; draft, 8). It is a table of the profile's `moves` array, in `tools/tiers/budgets.toml`:

```toml
[[gate.default.moves]]
from = "temper.simulated"
to = "temper.domain"
cpu_s = 12
why = "the store's own mechanics placed below the one-process world"
```

  - `from` and `to`, each `TREE.LAYER`: one of the five trees, a dot, and a name of `tools/tiers/layers.toml`'s `layers` list. The repository's checks are no layer of a pyramid, and move nothing;
  - `cpu_s`, the seconds it moves, a finite positive number, never a boolean;
  - `why`, a nonempty string, a sentence for the change's review.
- **`budgets()` refuses** (`ValueError`) a move with any other key or a value outside the form; a `to` whose layer isn't below `from`'s in the `layers` list, "below" as `tools/tiers/layers.md`, 2 has it; a `to` tree with no table for the profile; and moves that take any layer's ceiling below 0. So each move goes down and leaves the sum of the ceilings as it was, which is all the gate checks of it. A move up, sideways or into another tree's same layer is a raise, the user's (8.1).
- **A layer's ceiling** for a profile is its table's `cpu_s`, or 0 where it has no table, plus the profile's moves into it less its moves out of it. A layer with neither a table nor a move into it has no ceiling. A layer a move gives its first seconds has a ceiling, so it isn't over for having no table (section 4), and its reference is its profile's `references` entry.
- **Applied once to the held budget.** A move stays in the file, and the ceiling has it at every run. The ledger's `moves` are those main's held budgets have taken: a move the ledger lacks moves its seconds of held budget at the gate of the change that adds it, and once only (section 9).
- **Folding a move into the figures,** a later change that writes its seconds into both layers' `cpu_s` and removes the line, leaves each ceiling, and so each held budget, as it was. A move removed alone raises its `from` layer's ceiling and lowers its `to` layer's, a change to the ceilings that is the user's, as any change to a `cpu_s` is; each held budget it changes restarts at its ceiling (section 9).

## 11. Later

- **The suites run through nextest** (`tools/tiers/run`, section 7's collector in `tools/tiers/design.md`) are not summed by layer. The focused and fuzzy suites run through one-process (`tools/tiers/design.md`, 11).

## 12. Open questions

- **jig's layer ideals.** The draft estimated jig's ideal with its worlds placed by hand. `tools/tiers/layers.toml` places most of them in `simulated` and `protocol`. Under it, jig's `domain` ideal of 17 s sits beside a figure of 2.4 s, and its `simulated` ideal of 3 s beside 99.5 s. Ideals are reported and never enforced. A split re-estimated under the layers' placement is the user's figure.
