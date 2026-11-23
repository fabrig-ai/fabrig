# What a branch passes to land

fabrig asks these checks of a branch before it lands on main (the gate), and of
main after each landing (the after-landing tier). A red is a red whoever runs the
check. Commands start at the checkout root; a caller may wrap each in its own
build command.

## The gate

A landing check checks the branch merged onto main: every check below, with
nothing slower than the fuzzy suite (`skein/docs/design/testing/layout.md`, 1;
`skein/docs/foundation/testing-strategy.md`, 8).

The dispatched Gate reads its selector, reporter and replay tests from the
landing base. An edit confined to Gate labels, YAML comments or the reporter
runs workflow syntax and reporting checks. A routing edit replays the selector
with a stand-in for the runner's Gate, checking the calls and exit status.
The replay runs the landed tests against the proposed scripts and also runs
any changed tests. Executable workflow changes and changes that mix Gate code
with product or build inputs also run the regular Gate. A failed replay is red.

1. **Format:** `cargo fmt --all --check`. Rule: the root `rustfmt.toml`.
2. **Clippy, warnings denied:** `cargo clippy --workspace --all-targets -- -D warnings`.
   For a selection, replace `--workspace` with a `-p` for each package the change
   reaches and each in `.config/gate/always.txt`. Rules: `skein/clippy.toml`,
   `smith/clippy.toml`, `jig/clippy.toml`, `temper/clippy.toml`, `hearth/clippy.toml`,
   `tests/clippy.toml` and `AGENTS.md`, 2 (a lint exception is a scoped `#[expect]`).
3. **The wasm32 clippy:** `cargo clippy --target wasm32-unknown-unknown -p PKG ... -- -D warnings`,
   with a `-p` for each package in `.config/gate/wasm32.txt`.
   Rule: `skein/docs/design/testing/README.md`, 10, “Checked in the gate”.
4. **The focused and fuzzy suites:** `tools/tiers/one-process --gate --walls
   .config/gate/walls.toml --ledger LEDGER default` (focused) and `tools/tiers/one-process
   --gate --walls .config/gate/walls.toml --ledger LEDGER fuzzy`, the profiles `.config/nextest.toml`
   defines: each test binary's selected tests in one process, one at a time, and
   binaries in parallel (`tools/tiers/design.md`, 11), with nextest as their front
   end, selecting and building. One-process runs by default the layers that
   replay, step tests and domain, system, machine, protocol, io and simulated
   worlds, with the repository's checks, by `tools/tiers/layers.toml`, and leaves
   out the real-loop and end-to-end binaries, which run after landing; it judges
   only the layers it ran, with or without `--gate` (`tools/tiers/layers.md`, 9.1;
   `tools/tiers/budgets.md`, 4.1). A rerun of named tests, `-E 'binary_id(=B) &
   test(=T)'`, selects within those layers too. For a selection, add a `-p` for each package, as
   in item 2; with none, each runs the whole workspace. The walls file holds
   the gate's wall limits, 270 s focused and 215 s fuzzy, in place of the
   profiles' own (`tools/tiers/design.md`, 11.7), until the user sets the
   figures of the runs without the real loop and end to end
   (`temper/docs/design/draft/test-budgets.md`, 7). `--gate` asks for the
   gate's verdict on each tree's layers against `tools/tiers/budgets.toml`
   (`tools/tiers/budgets.md`, 5). `LEDGER` is main's ledger for that suite,
   where the gate's runner keeps one; omit `--ledger LEDGER` until it does.
   The run writes `ledger.json` with its figures folded in, which the runner
   keeps for the next gate when the candidate lands (`tools/tiers/budgets.md`, 7).
   With a ledger, the run also runs each new and flagged test alone after the
   suite and holds it to its reference. Where its `results.json` has a
   flagged test, the runner runs `tools/tiers/one-process --alone RUN_DIR`
   from a checkout of the candidate's base, then `tools/tiers/cpu-summary
   RUN_DIR --ledger LEDGER --base BASE_RUN_DIR --gate`, whose exit status is
   the verdict with the base's figures (`tools/tiers/references.md`, 5 and 6).
   Rules:
   `skein/docs/foundation/testing-strategy.md`, 8 and 8.1: a CPU budget for
   each layer of each tree the gate runs, and the repository's checks', over budget is
   red, starved is a warning; a run its wall stopped
   runs once more, and stopped again it is red unless the gate's own cgroup
   was starved. Slow-tier profiles never run in the gate.
5. **The old-forms counts:** `(cd TREE && python3 -I -B scripts/old_forms.py)` for
   each tree the branch touches; add `--write` to lower counts for what it removed.
   Counts only fall (`tools/old_forms/README.md`).
6. **Citations:** `python3 -I -B tools/citations/citations.py --all`, at 0 broken.
   Rules: `DESIGN.md`, 4; `tools/citations/README.md`.
7. **The outward check:** `tools/tiers/one-process default -p fabrig-boundary`,
   part of item 4's focused suite. Rule: `tests/boundary/tests/outward.rs`'s module doc and `AGENTS.md`, 1.
   The rule is absolute: any match fails, and the check has no inventory.
8. **Lint profiles and the count of catches:** `python3 -I -B tools/lints/lints.py` and
   `python3 -I -B tools/lints/panics.py`, each at 0 findings, and their tests,
   `python3 -I -B tools/lints/test_lints.py` and `python3 -I -B tools/lints/test_panics.py`.
   Rules: `tools/lints/design.md`, 5 and 7.
9. **The fail-stop check:** `tools/tiers/one-process default -p fabrig-boundary`,
   part of item 4's focused suite. Rule: `skein/docs/foundation/programming-model.md`, 2, and
   `tests/boundary/tests/fail_stop.rs`'s module doc: every binary's `main` installs
   `skein_fail_stop`'s hook first.
10. **The command line's checks:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `skein/docs/foundation/cli.md`, 9, and
    `tests/boundary/tests/cli.rs` and `tests/boundary/tests/cli_names.rs`'s
    module docs: the layout, `main.rs`'s size, no features, and declared
    environment names (check 5), with their exceptions.
11. **The patch check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `skein/docs/design/data/codec/command.md`, 2,
    and `tests/boundary/tests/patches.rs`: wires and formats outside the registry
    pass `check` against git's landed files and `current`, and the owner list
    covers every crate with `schema/kept/`. Registered map families keep no history.
12. **The size check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `tools/crates/design.md`, 3.1, and
    `tests/boundary/tests/sizes.rs`'s module doc: library and shared-family lines,
    with their shrinking exceptions.
13. **The client layouts' check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `jig/docs/design/protocol/client/versions.md`, 1,
    and `tests/boundary/tests/layouts.rs`: each listed wire's versions rise by one,
    the last line is its schema's version and family fingerprint, and git's landed
    list stands unchanged at the head. The base is the branch's merge base with
    `origin/main`, or `HEAD` when that reference is absent, as in the patch check.
14. **The features check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `tools/crates/design.md`, 3.3, and
    `tests/boundary/tests/features.rs`'s module doc: external entries carry every
    context's feature union, platform entries stay on their platforms, and
    `.cargo/config.toml` enables Cargo's workspace feature unification.
15. **The generated-tests check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `tools/crates/design.md`, 3.4, and
    `tests/boundary/tests/generated_tests.rs`'s module doc: generated records hold
    no test code, and their libraries build no test target, with shrinking
    exceptions.
16. **The layers check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `tools/tiers/layers.md`, 7, and
    `tests/boundary/tests/layers.rs`: every test binary of a tree has its layer
    in `tools/tiers/layers.toml`, by rule or by its line, and every line names a
    test binary of its tree, placed by no rule, at a layer the file names.
17. **The figures:** `python3 -I -B tools/tiers/figures.py --base BASE`, on
    the candidate, BASE the commit of main it merges onto, as item 6 runs
    on the candidate. Exit 1 refuses a change that raises a figure of
    `tools/tiers/budgets.toml` or adds a reason's line, one line naming
    each, the output ending with the repair line; a change of that file
    alone passes, whatever it does there. The runner makes a red case of
    each `figure raised:` and `figure added:` line. Rule:
    `tools/tiers/budgets.md`, 10.1; every figure there is the user's
    (`skein/docs/foundation/testing-strategy.md`, 8.1).
18. **The merge job's tests:** `python3 -I -B tools/merge/test_merge.py`,
    its one test with the real generator building `skein-codegen`. Rules:
    `tools/merge/README.md` and `temper/docs/design/protocol/tooling.md`, 4.1.

## The gate's lists

Each list has one entry per line; `#` starts a comment line.

- **`.config/gate/always.txt`:** package names whose tests read other packages'
  files by path, so changes outside their dependencies can break them. A gate
  selecting the packages a change reaches runs these too. The branch adding such
  a test adds its package; adding a line runs that package's tests for every change.
  A line `PACKAGE PATH-PREFIX` runs them only for a change to a path that starts with
  the prefix, as written: for a test whose reads all lie under it. The full CI
  check reads the list too (`tools/pre-pr/design.md`, 3): a line without a prefix
  is a boundary package, whose focused and fuzzy tests run, with
  `fabrig-boundary`'s, for every full check that selects packages, a docs-only one
  included; a prefixed line adds its package where a changed code input starts
  with the prefix.
- **`.config/gate/read-by-tests.txt`:** checkout-root globs of the inputs, under
  `docs/` (any format) or elsewhere, that tests compile in or read (`AGENTS.md`, 2).
  A matching change changes test inputs and runs their tests as for code; Markdown,
  and any file under a `docs/` directory, matching no glob is docs and runs none,
  so every non-Markdown input under `docs/` is listed. The branch making a test
  read such a file adds its glob; adding a line makes changes to matching files
  run the tests.
- **`.config/gate/wasm32.txt`:** package names for item 3. The branch making a
  package's wasm32 clippy pass with warnings denied adds it; the trees' status
  files name portable packages still left out. Adding a line adds that package to
  item 3's command (`skein/docs/design/testing/README.md`, 10).

## After landing

- **The slow tiers:** `tools/tiers/run TREE TIER [-- NEXTEST_ARGS...]` runs a tree's
  tier with its prerequisites; `tools/tiers/run --cpu DIR TREE TIER [-- NEXTEST_ARGS...]`
  accounts it against `tools/tiers/budgets.toml`. The hyphenated profiles in
  `.config/nextest.toml` are `browser-skein`, `live-skein`, `real-skein`, `live-smith`,
  `real-smith`, `live-jig`, `browser-jig`, `real-jig`, `process-temper`, `live-temper`,
  `browser-temper`, `real-temper`, `conformance-hearth` and `real-hearth`. Never in
  the gate. Rules: `tools/tiers/design.md`, 1, 4 and 7;
  `skein/docs/foundation/testing-strategy.md`, 8.1.
- **The real loop and end to end:** `tools/tiers/run TREE real`, for each of
  skein, smith, jig, temper and hearth, runs the pair `real-TREE`: the tree's
  real-loop and end-to-end tests that the focused and fuzzy suites select,
  which the gate leaves out (`tools/tiers/layers.md`, 9.2). They run at each
  landing on main. With `--cpu`, each layer's CPU is reported beside the
  tree's figures for it in both suites' gate tables as a warning, never red
  (`tools/tiers/budgets.md`, 4.2); a failure is red, repaired from the
  lowest tier that shows it.
- **The after-landing tier:** after each landing on main, the five real
  pairs above, once its runner lists them;
  `tools/tiers/run smith live` runs `live-smith` (`smith/docs/design/testing.md`, 2.3),
  and `tools/tiers/run temper live -- --run-ignored only` runs `live-temper`'s
  owned-forge landing pair (`temper/docs/design/testing/live-tier.md`).
  The inputs for `live-smith` are `SMITH_LIVE`, the directory holding smith's
  ChatGPT record and Claude Code's login (smith's section 2.3); for `live-temper`,
  `TEMPER_TEST_LIVE_ANTHROPIC_KEY` (the key's file), `TEMPER_TEST_LIVE_CLAUDE` (Claude Code's
  login), `claude` on `PATH`, `TEMPER_TEST_LIVE_STORE` and a private `TMPDIR` (temper's
  section 2.7).
  On fabrig's own Forgejo, temper dispatches `.forgejo/workflows/tier-main.yml`
  at each move of main: it runs the runner's `$CI_TIER` at `temper_commit`, with
  `temper_tests` as `CI_TIER_TESTS`, and uploads `junit.xml` as the artifact `junit`.
- **The macOS tiers:** `.forgejo/workflows/tier-macos-e2e.yml` and
  `tier-macos-suites.yml`, on the runner label `relay`, run the runner's
  `$CI_TIER --tier macos-e2e` or `macos-suites` as tier main's runs `$CI_TIER`;
  it hands the run to GitHub's `.github/workflows/macos.yml` at the same commit
  and brings back its report (`temper/docs/design/testing/macos.md`, 9). A macOS
  red blocks nothing.
- **What red means:** a failure observed after landing is red; repair starts in
  the lowest tier that shows it (`skein/docs/foundation/testing-strategy.md`, 9).
  Over a CPU budget is red; past the wall hard limit is starved, a warning; tests
  the run did not reach are unobserved, never passed
  (`skein/docs/foundation/testing-strategy.md`, 8.1). A caller reads the verdict
  and its `unobserved` attempts through `tools/tiers/design.md`, 10, with summary
  semantics in 7.3.

## The repository's hooks and fixture readers

- **`.temper/pre-pr`** checks the delivery in the worker workspace using the
  changed packages, their immediate consumers and named integration targets.
  A checker-only Gate runs the checker's Python suite. Explicit diagnostic
  dispatch can check an accumulated goal using `.temper/goal-pre-pr HEAD DESTINATION`
  (`tools/pre-pr/design.md`, 1).

- **`.temper/merge`:** the reference's merge job, `tools/merge/merge.py`
  (`temper/docs/design/protocol/tooling.md`, 4.1).
- **`.temper/close`:** fabrig's close command, `tools/close/close.py`
  (`temper/docs/design/protocol/tooling.md`, 4.2).
- **`.config/fixture-readers.toml`:** the readers and generators of every tracked
  `**/fixtures/**` file (`tools/tiers/design.md`, 9;
  `skein/docs/foundation/testing-strategy.md`, 8).
