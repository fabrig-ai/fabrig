# Layers

Provisional, 2026-11-20. Which layer of the pyramid each test binary is in: `tools/tiers/layers.toml`, and the boundary check that keeps it whole. Written from `temper/docs/design/draft/test-budgets.md`, 4.

## 1. In one page

- **A layer is a tier** of the strategy's section 2 (`skein/docs/foundation/testing-strategy.md`): step tests, domain worlds, system worlds, machine worlds, protocol worlds, io worlds, simulated worlds, the real loop, end to end. The repository's own checks are no layer of a pyramid, and a group of their own (section 4).
- **Every test binary is in one layer,** since CPU is measured per process and a process runs one binary's tests (`tools/tiers/design.md`, 11). A binary's tests are of its layer (the strategy's 8.1), and a binary of the real loop or end to end holds no test that replays (section 6.1).
- **By rule, with no line:** a library's own tests and a binary's own are step tests; every binary of a package outside the five trees is the repository's checks (section 4).
- **Every other binary has a line** in `tools/tiers/layers.toml`: its id and its layer, under its tree's table, as its tree's testing document places it (sections 5 and 6).
- **A boundary check keeps the table whole** (section 7): from `cargo metadata`, which builds nothing, every test binary has its layer, and every line names a binary that exists, in its own tree, at a layer the file names.
- **Who reads it:** the boundary check, and the suites' summaries by layer once they are built (section 8).

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

- **What a test checks places it, never its cost** (the strategy's 8.1). Its tree's testing document places it: `skein/docs/design/testing/README.md`, 2; `smith/docs/design/testing.md`, 2; `jig/docs/design/domain/testing.md`; `temper/docs/design/testing/README.md`, 2, where the client's world is a system world (2.1); `hearth/docs/design/testing/README.md`, 2. Where the document says nothing of it, the strategy's table (section 2) does, by what the binary runs real and what it fakes, and the line carries a comment.
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

A focused test of `fabrig-boundary`, `tests/boundary/tests/layers.rs` (`.config/gate/README.md`, the gate's 16th item), reads `cargo metadata --no-deps --format-version 1 --locked` and `tools/tiers/layers.toml`, and lists, each naming the binary's id or the line and why:

- a test binary of a tree, placed by no rule, with no line;
- a line naming no test binary of the workspace, or one placed by rule (section 4);
- a line under a table other than its binary's tree, or a table that is no tree's;
- a line whose layer the `layers` list doesn't name, a `layers` list with a name twice or without `step`, or anything at the top but that list and the trees' tables.

The check passes with no finding. Its findings are computed from the metadata's and the table's text, so its own tests run it on small fixtures of both, and once on the workspace's.

## 8. Who reads it

- **The boundary check** (section 7).
- **The suites' summaries:** `tools/tiers/one-process` places each binary it runs by the file and section 4's rules, as the check does, and sums CPU by tree, suite and layer (`tools/tiers/budgets.md`, 3 and 4).
- **The budgets:** `tools/tiers/budgets.toml`'s gate tables name their layers from the file's `layers` list (`tools/tiers/budgets.md`, 2).

## 9. Later

- **Splitting binaries** that span two layers that replay, where the table's comments name them. Each takes the higher layer meanwhile, and the gate runs both (section 6.1).
