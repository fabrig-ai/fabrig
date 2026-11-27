# Layers

Provisional, 2026-11-21. Which layer of the pyramid each test binary is in: `tools/tiers/layers.toml`, the boundary check that keeps it whole, and which layers run where: the gate's and the delivery check's runs the layers that replay, each tree's real pair the real loop and end to end after landing, and each tree's campaign pair its sweeps' fresh seeds after landing. Written from `temper/docs/design/draft/test-budgets.md`, 4 and 7, and, for the campaign pairs, `temper/docs/design/draft/fuzzy-campaigns.md`, 5.2.

## 1. In one page

- **A layer is a tier** of the strategy's section 2 (`skein/docs/foundation/testing-strategy.md`): step tests, domain worlds, system worlds, machine worlds, protocol worlds, io worlds, simulated worlds, the real loop, end to end. The repository's own checks are no layer of a pyramid, and a group of their own (section 4).
- **Every test binary is in one layer,** since CPU is measured per process and a process runs one binary's tests (`tools/tiers/design.md`, 11). A binary's tests are of its layer (the strategy's 8.1), and a binary of the real loop or end to end holds no test that replays (section 6.1).
- **By rule, with no line:** a library's own tests and a binary's own are step tests; every binary of a package outside the five trees is the repository's checks (section 4).
- **Every other binary has a line** in `tools/tiers/layers.toml`: its id and its layer, under its tree's table, as its tree's testing document places it (sections 5 and 6).
- **A boundary check keeps the table whole** (section 7): from `cargo metadata`, which builds nothing, every test binary has its layer, and every line names a binary that exists, in its own tree, at a layer the file names; and each tree's real pair names every real-loop and end-to-end line of the tree, and its campaign pair only binaries that replay.
- **The gate runs what replays** (section 9): `tools/tiers/one-process`, for the focused and fuzzy profiles, runs by default only the binaries of the layers that replay and the repository's checks, and judges only those layers; a caller asks for others with `--layers`.
- **The real loop and end to end after landing** (section 9.2): each tree's pair `real-TREE` runs its real-loop and end-to-end tests that the focused and fuzzy suites select, as the after-landing tier runs every slow pair.
- **Campaigns after landing** (section 9.3): a tree's pair `campaign-TREE` runs the fuzzy tests of the binaries, of layers that replay, whose sweeps take their cases from skein's seam, for `tools/tiers/campaign`.
- **Who reads it:** the boundary check, the suites' summaries by layer, and one-process's choice of what to run (section 8).

## 2. The layers

The layers are named in `tools/tiers/layers.toml`'s `layers` list, in the strategy's table order (section 2 there), bottom first. That list is their one owner: every reader takes the names and their order from it.

| Name | The strategy's tier |
|---|---|
| `step` | step tests (2.1) |
| `domain` | domain worlds (2.2) |
| `system` | system worlds (2.3) |
| `machine` | machine worlds (2.4) |
| `protocol` | protocol worlds (2.5) |
| `io` | io worlds (2.6) |
| `simulated` | simulated worlds (2.7) |
| `real-loop` | the real loop (2.8) |
| `end-to-end` | end to end (2.9), live tests included |

"Below" is the list's order: a layer is below every layer after it. The budgets' moves down read it (`temper/docs/design/draft/test-budgets.md`, 8).

## 3. Test binaries

- **A test binary** is a target of a workspace member that cargo builds for its tests: a target whose `test` is true in `cargo metadata --no-deps --format-version 1`. A target with `test = false` (a fixture's binary, a kit's library) is none.
- **Its id** is nextest's binary id, built from the package's name and the target's:

| Target's kind | Its id |
|---|---|
| a library (`lib`, `rlib`, `cdylib`, `dylib`, `staticlib`, `proc-macro`) | `PACKAGE` |
| `bin` | `PACKAGE::bin/TARGET` |
| `test` | `PACKAGE::TARGET` |
| `example` | `PACKAGE::example/TARGET` |
| `bench` | `PACKAGE::bench/TARGET` |

- **Its tree** is the first directory of its package's manifest path below the workspace root: `skein`, `smith`, `jig`, `temper` or `hearth`, or none of them for the repository's own packages (`tests/boundary/`).

## 4. By rule

These binaries have no line, and a line naming one is refused:

- **Step tests:** a library's own tests (an id with no `::`) and a binary's own (an id with `::bin/`) (`skein/docs/foundation/testing-strategy.md`, 2.1). What a crate's own tests check is one step function or one container (the strategy's 2.1); its worlds are integration targets of their own.
- **The repository's checks:** every test binary of a package outside the five trees, today `fabrig-boundary`'s. They check the repository, not a tree's code, and have a budget of their own (`temper/docs/design/draft/test-budgets.md`, 4).

## 5. The table

`tools/tiers/layers.toml`, its layers and then a table for each tree, in the trees' order, each tree's table present though it may be empty:

```toml
# Each test binary's layer (tools/tiers/layers.md). A library's own tests and a binary's own are
# step tests, and the repository's checks a group of their own, by rule, with no line.
layers = ["step", "domain", "system", "machine", "protocol", "io", "simulated", "real-loop", "end-to-end"]

[skein]
"skein-io-world::suite" = "io"

[smith]

[jig]

[temper]
"temper-engine-domain-world::suite" = "domain"

[hearth]
```

- **A line** is a binary's id, quoted, and its layer's name. Lines are sorted by id within their table.
- **A comment beside a line** says why, in a few words, where the tree's testing document doesn't place the binary outright: a binary its document lists beside the tiers, a browser binary, one whose tests span two layers (section 6).
- The header and the `layers` line are fixed as above; a change to the list is a change to the strategy's tiers first.

## 6. Placing a binary

- **What a test checks places it, never its cost** (the strategy's 8.1). Its tree's testing document places it: `skein/docs/design/testing/README.md`, 2; `smith/docs/design/testing-tiers.md`; `jig/docs/design/domain/testing.md`; `temper/docs/design/testing/tiers.md`, 1, where the client's world is a system world (2.1); `hearth/docs/design/testing/README.md`, 2. Where the document says nothing of it, the strategy's table (`layers.md`, section 2) does, by what the binary runs real and what it fakes, and the line carries a comment.
- **A binary whose tests span two layers** is split, or takes the higher one until it is, its comment naming both, where both replay. One whose tests include some that replay and some that don't is split (section 6.1).
- **The slow tiers' binaries** have layers too, by the same rule: the process and live tiers' are end to end; a browser binary, which runs a real browser on what ships, is end to end unless its tree's document places it lower.
- **A binary that calls the real kernel or real programs without the loop** (real sockets, locks or children, a real tool run synchronously and its output compared, hearth's conformance among them) is in the real loop: the lowest layer that runs what is real below, and does not replay (the strategy's 2.8, "real programs' output"). One that runs a program or script as it ships is end to end. Its comment names the real calls.
- **The change that adds a binary adds its line,** and its review reads the line against the tree's testing document.

### 6.1 What is real and what replays, apart

The gate runs the layers that replay, and the real loop and end to end run after landing (`temper/docs/design/draft/test-budgets.md`, 4). A binary is in one layer, so a test that replays in a real-loop or end-to-end binary would leave the gate with it: such a binary's tests are split first (draft, 4, "A binary that mixes them is split first").

- **A test replays** where it owns its clock, its seeds and its kernel (the strategy's table in its section 2, "Replays"): it writes no real file or directory, opens no socket, takes no lock, starts no child or thread, calls no real ring and reads no system clock. Reading the checkout's committed files, read-only, as fixtures, goldens or documents, is no real call: their bytes are the head's, as the repository's checks read the tree (draft, 4). Its layer is one below the real loop, as its tree's testing document places it (section 6).
- **A test that runs what is real** is of the real loop, or of end to end where it starts a program or script as it ships (section 6).
- **No real-loop or end-to-end binary holds a test that replays.** A binary holding both is split within its package: its tests that replay go to a binary of their own, with its line in the table at their layer, and the real ones keep the binary and its line, whose comment then names its real calls alone. Where the tests that replay span layers, their binary takes the highest, its comment naming each (section 6); a binary for each layer is as good where their modules part cleanly.
- **The new binary's name** is its layer's, in a target's form: `step`, `domain`, `system`, `machine`, `protocol`, `io` or `simulated` (`skein-kv-tests::simulated`), or a name saying what it holds where the package has a target of that name already. No profile of `.config/nextest.toml` selects it by name: never `live`, `process`, `live_github`, `proof_live`, `live_github_managed`, or a name starting `browser_`, `conformance_` or `recorded_`, so the focused and fuzzy suites select it by their ordinary rules.
- **Or the real ones move.** A test that moves is new to main's ledger, so it runs alone at its landing's gate (`tools/tiers/references.md`, 3). Where a binary's real tests are both fewer than its tests that replay and cheaper in the suite, they may move instead, to `real_loop`, or `end_to_end` for those that start a shipped program, and the binary keeping its name takes the layer of the tests that replay. A binary a slow tier's profile selects by name (`live`, `conformance_*`) keeps its real tests either way.
- **Each test keeps what it checks:** its body, its assertions, its seeds and its fixtures, and its module path within its binary, so its name and the fuzzy suite's `fuzzy::` selection hold; only its binary changes. What both binaries need goes to the package's library or a kit, never a file both compile (`tools/crates/design.md`, 3.2): `tests/boundary/tests/shared_test_code.rs` refuses one.
- **A test its tree's testing document keeps in the real loop** though it would replay stays, and its change's report says why.
- **Its room moves with it.** The CPU a split places below the real loop goes with a move down in `tools/tiers/budgets.toml`, for each suite and each pair of layers (`tools/tiers/budgets.md`, 10): `from` the split binary's layer, `to` the layer of the binary holding the tests that replay, and `cpu_s` that binary's CPU in the suite, summed over the tree's splits between the two layers, rounded down to a tenth, and at most what leaves the `from` layer's ceiling at 0. It is measured once, by `tools/tiers/one-process PROFILE --out DIR -p PACKAGE --test NAME`, as its processes' `cpu_s` in `results.json`. A suite that selects none of its tests, or a figure under a tenth, moves nothing, and so does a split of a binary neither suite selected (a `live` or `conformance_*` one): its tests that replay are new to the suites, and their CPU is their layer's growth. A move down is no raise (`tools/tiers/budgets.md`, 10.1), and no other figure changes. The change's report gives what each layer gains and loses, by suite, in CPU seconds.

## 7. The boundary check

A focused test of `fabrig-boundary`, `tests/boundary/tests/layers.rs` (`.config/gate/README.md`, the gate's 16th item), reads `cargo metadata --no-deps --format-version 1 --locked`, `tools/tiers/layers.toml`, `.config/nextest.toml` and `tools/tiers/packages.toml`, and lists, each naming the binary's id or the line and why:

- a test binary of a tree, placed by no rule, with no line;
- a line naming no test binary of the workspace, or one placed by rule (section 4);
- a line under a table other than its binary's tree, or a table that is no tree's;
- a line whose layer the `layers` list doesn't name, a `layers` list with a name twice or without `step`, or anything at the top but that list and the trees' tables;
- from `.config/nextest.toml`'s text, for a tree with a line at `real-loop` or `end-to-end`: no `[profile.real-TREE]`, or one whose `default-filter` doesn't name, by `binary_id(=ID)`, each such line of the tree, or names by `binary_id(=…)` an id that is no such line of the tree (section 9.2). A `real-TREE` profile for a tree with no such line is a finding too.
- from `tools/tiers/packages.toml`, for each such tree: no `real-TREE` list, a package of one of those lines missing from it, or a package on it that is no such line's. The lines of binaries the slow tiers own by name, `browser_*`, `conformance_*`, `recorded_*`, `live` and `live_github`, need no package there, since the pair's clause leaves them out (section 9.2): the runner builds only the packages its list names, so a missing one's tests run nowhere.
- from `.config/nextest.toml`'s text, for each `[profile.campaign-TREE]` (section 9.3): a TREE that is no tree; a `default-filter` that names no `binary_id(=…)`; an id it names that is no test binary of TREE's packages, or one placed at `real-loop` or `end-to-end`, by its line or unplaced. From `tools/tiers/packages.toml`, for each such profile: no `campaign-TREE` list, or one that doesn't hold exactly the packages of the ids its filter names; and no `campaign-TREE` list in `[binaries]`.
- from `tools/tiers/packages.toml`'s `[binaries]` table: a pair's list not sorted and unique by package then binary, which the runner refuses before any command (`tools/tiers/design.md`, 2.1).

The check passes with no finding. Its findings are computed from the metadata's, the table's and the configuration's text, so its own tests run it on small fixtures of each, and once on the workspace's.

## 8. Who reads it

- **The boundary check** (section 7).
- **The suites' summaries:** `tools/tiers/one-process` places each binary it runs by the file and section 4's rules, as the check does, and sums CPU by tree, suite and layer (`tools/tiers/budgets.md`, 3 and 4).
- **What a run runs:** one-process leaves out the binaries of the layers its run doesn't run (section 9), and a real or campaign pair's summary places its tests' binaries by the file (`tools/tiers/budgets.md`, 4.2).
- **The budgets:** `tools/tiers/budgets.toml`'s gate tables name their layers from the file's `layers` list (`tools/tiers/budgets.md`, 2).

## 9. The layers a run runs

The gate runs the layers that replay, and the real loop and end to end run after landing (`temper/docs/design/draft/test-budgets.md`, 4). The switch lives in the tree, in what `tools/tiers/one-process` runs by default, so the gate's runner, which runs one-process from the candidate's own checkout, changes nothing (the user, 11-21), and the delivery check, which runs it too (`tools/pre-pr/design.md`, 1), follows the same day.

### 9.1 What one-process runs

- **The layers that replay** are every name of the file's `layers` list but `real-loop` and `end-to-end`: today `step`, `domain`, `system`, `machine`, `protocol`, `io` and `simulated` (section 2). A layer a later change adds to the list replays unless this section says otherwise.
- **The default.** `tools/tiers/one-process [--layers LAYERS] …` (`tools/tiers/design.md`, 11.9). For the focused and fuzzy profiles, `default` and `fuzzy`, a run runs the binaries of the layers that replay and the repository's checks (`checks`, section 4), and leaves out every binary placed `real-loop` or `end-to-end`, whatever else its arguments say: with or without `--gate`, `--ledger` or `--walls`, a `-p` selection or a filter of the caller's. For any other profile it runs every layer, as before.
- **`--layers LAYERS`** asks for others: a comma-separated list of names of the file's `layers` list and `checks`, or `all` for every layer, as before this section. `--layers real-loop,end-to-end` runs those alone; `--layers all` runs everything the profile selects. A name the file's list doesn't hold, but `checks` and `all`, an empty list, or `--layers` beside `--again` or `--alone` is an invalid invocation (exit 2), refused before anything runs. Without the layers file, the default and `all` run every binary, as section 4's rules place them or unplaced, and record no layers (`tools/tiers/budgets.md`, 4.1); any other list is refused.
- **An unplaced binary runs** whatever the layers: its missing line is the boundary check's red (section 7), and a run never leaves out a binary it can't place.
- **After nextest's selection, never a filter.** The run places each binary its listing names (`tools/tiers/budgets.md`, 3) and leaves out those of a layer it doesn't run, after nextest has selected; it passes nextest no filter of its own. Nextest joins two `-E` filters by `or`, so a filter added to a caller's would widen it. A caller's filter thus selects within the layers the run runs: the gate's rerun of a named test, `-E 'binary_id(=B) & test(=T)'`, runs it where the default ran it, and selects nothing where the default left its binary out, as the default did.
- **What it builds.** Where the run narrows its listing to the targets its profiles can select (`TIERS_PRE_PR_TARGETS`, `TIERS_PROFILE_TARGETS`: `tools/tiers/design.md`, 11.9), it also leaves out the targets of the layers it doesn't run, so the delivery check builds no binary it won't run. `tools/tiers/build-profile-targets` builds as before, every layer, since a caller may run any.
- **It says so.** A run that leaves out a binary prints one line before its lanes start, naming the layers it left out and how many binaries they held, and `results.json` holds `selected_layers`: the names it ran, in the file's order, then `checks` (`tools/tiers/budgets.md`, 4.1). A selection left empty follows `--no-tests` as nextest's own empty selection does (`tools/tiers/design.md`, 11.2).
- **A process again and at the base.** `--again` and `--alone` (`tools/tiers/design.md`, 11.9) run what they name, whatever its layer: a run's own selection chose it.
- **Judged by what it ran.** A run judges only the layers it ran: no row, budget, warning or red for a layer it left out, and its `ledger.json` keeps no figure for one (`tools/tiers/budgets.md`, 4.1).
- **The gate's walls stay** 270 s focused and 215 s fuzzy until the user sets the new runs' figures (draft, 7).

### 9.2 The real pairs

The real loop and end to end don't stop running: each tree with a line at `real-loop` or `end-to-end` has a slow pair, `real-TREE` (`real-skein`, `real-smith`, `real-jig`, `real-temper`, `real-hearth`), run by `tools/tiers/run TREE real` as every pair is (`tools/tiers/design.md`, 1 and 6). Adding them to the after-landing tier's list, outside this repository, is the user's (`temper/docs/design/draft/test-budgets.execution.md`, 2).

- **What it runs:** the tests that the focused or the fuzzy profile selects in the tree's real-loop and end-to-end binaries, the ones one-process now leaves out of the gate. Its `[profile.real-TREE]` in `.config/nextest.toml` has the `default-filter` `(FOCUSED) | (FUZZY)` and'ed with `binary_id(=ID) | …` for every line of the tree at `real-loop` or `end-to-end`, FOCUSED and FUZZY copies of the tree's own clause of `[profile.default]`'s and `[profile.fuzzy]`'s `default-filter`. The clauses already leave out what the slow tiers own (`browser_*`, `live`, `process`, `conformance_*` and the rest), so the list names every such line, those included, and the boundary check holds it to the table (section 7). A change to a tree's clause in either profile changes its copy.
- **Its settings:** `junit = { path = "junit.xml" }`, as every pair writes its report, and no `global-timeout`, `slow-timeout` or `test-threads` of its own: it takes `[profile.default]`'s, 810 s and a 6 s period, and its overrides after its own, until the user sets the pair's figures. Its macOS overrides double its Linux slow periods, as every profile's do (`temper/docs/design/testing/macos.md`, 7).
- **Its packages** in `tools/tiers/packages.toml`: the packages of the binaries it selects tests from, sorted, as the map is generated (`tools/tiers/design.md`, 1 and 6), and the boundary check holds the list to the tree's lines (section 7). Its `[binaries]` list holds what its tree's `[test-binaries]` list holds, since the tests that run another package's program are its own now: `smith-fake-claude`'s `fake-claude` for `real-smith`, an empty list for the others.
- **Its budgets** are the tree's `real-loop` and `end-to-end` ceilings of both suites in `tools/tiers/budgets.toml`'s gate tables, reported by its runs as warnings, never red (draft, 11; `tools/tiers/budgets.md`, 4.2). It has no table of its own; a table the user adds later applies as any pair's.

### 9.3 The campaign pairs

A campaign runs a tree's fuzzy sweeps with fresh seeds after landing, in rounds of the tree's pair `campaign-TREE`, run by `tools/tiers/campaign` (`tools/tiers/campaign.md`) through `tools/tiers/run TREE campaign` as every pair is run (`temper/docs/design/draft/fuzzy-campaigns.md`, 5.2).

- **What it runs:** the tests that the fuzzy profile selects in the binaries it lists. Its `[profile.campaign-TREE]` in `.config/nextest.toml` has the `default-filter` `(FUZZY)` and'ed with `binary_id(=ID) | …`, FUZZY a copy of the tree's clause of `[profile.fuzzy]`'s `default-filter`, as a real pair copies it (section 9.2): a change to the clause changes its copy here too.
- **The binaries it lists** are of layers that replay (section 9.1), and their sweeps take their cases from skein's seam (`skein/docs/design/testing/sweeps.md`, 4.2). A binary of the real loop or end to end is never listed, since a case there doesn't replay, and the boundary check holds the list to the table (section 7). A world lists its binary in the change that adopts the seam, and a tree's pair comes with its first. A listed binary's other fuzzy tests run as the gate runs them, and a round names each with its time (`tools/tiers/campaign.md`, 3); a binary whose fixed tests outweigh its sweeps moves them to a binary of their own, or stays off the list.
- **temper's,** `campaign-temper`, lists `temper-engine-domain-world::suite`, whose walking sweep is the first to take its cases from the seam (`temper/tests/engine/domain/tests/suite/fuzzy/walking.rs`).
- **hearth's and skein's,** `campaign-hearth` and `campaign-skein`, list the binaries their testing documents name, and say which stay off and why (`hearth/docs/design/testing/README.md`, 4.5; `skein/docs/design/testing/README.md`, 7.1). Each filter names its ids one to a line, in the order of their binary ids. smith and jig have no pair yet.
- **Its settings:** `junit = { path = "junit.xml", store-success-output = true, store-failure-output = true }`, since a campaign reads the seam's lines in passing tests' output too, and no `global-timeout`, `slow-timeout` or `test-threads` of its own: it takes `[profile.default]`'s, 810 s and a 6 s period, until the user sets the campaign's wall (the draft, 5.3). Its macOS overrides double its Linux slow periods, as every profile's do (`temper/docs/design/testing/macos.md`, 7).
- **Its packages** in `tools/tiers/packages.toml`: the packages of the binaries it lists, sorted, generated and verified as every pair's are (`tools/tiers/design.md`, 1 and 6), and the boundary check holds the list to the filter's ids (section 7). Its `[binaries]` list holds the programs of other packages its tests run, an empty list for each of today's pairs.
- **Its budgets** are the tree's fuzzy ceilings for each layer, reported by its runs as warnings, never red (`tools/tiers/budgets.md`, 4.2). It has no table of its own.
- **Not in the after-landing tier's list:** campaigns run once a day, on a builder apart from the gate (the user, 11-26), and by hand until a runner runs them (`tools/tiers/campaign.md`, 8).

## 10. Later

- **Splitting binaries** that span two layers that replay, where the table's comments name them. Each takes the higher layer meanwhile, and the gate runs both (section 6.1).
