# What a branch passes to land

fabrig asks these checks of a branch before it lands on main (the gate), and of
main after each landing (the after-landing tier). A red is a red whoever runs the
check. Commands start at the checkout root; a caller may wrap each in its own
build command.

## The gate

A landing check checks the branch merged onto main: every check below, with
nothing slower than the fuzzy suite (`skein/docs/design/testing/README.md`, 6;
`skein/docs/foundation/testing-strategy.md`, 8).

1. **Format:** `cargo fmt --all --check`. Rule: the root `rustfmt.toml`.
2. **Clippy, warnings denied:** `cargo clippy --workspace --all-targets -- -D warnings`.
   For a selection, replace `--workspace` with a `-p` for each package the change
   reaches and each in `.config/gate/always.txt`. Rules: `skein/clippy.toml`,
   `smith/clippy.toml`, `jig/clippy.toml`, `temper/clippy.toml`, `hearth/clippy.toml`,
   `tests/clippy.toml` and `AGENTS.md`, 2 (a lint exception is a scoped `#[expect]`).
3. **The wasm32 clippy:** `cargo clippy --target wasm32-unknown-unknown -p PKG ... -- -D warnings`,
   with a `-p` for each package in `.config/gate/wasm32.txt`.
   Rule: `skein/docs/design/testing/README.md`, 10, “Checked in the gate”.
4. **The focused and fuzzy suites:** `tools/tiers/one-process default` (focused)
   and `tools/tiers/one-process fuzzy`, the profiles `.config/nextest.toml`
   defines: each test binary's selected tests in one process, one at a time, and
   binaries in parallel (`tools/tiers/design.md`, 11), with nextest as their front
   end, selecting and building. For a selection, add a `-p` for each package, as
   in item 2; with none, each runs the whole workspace. Rules:
   `skein/docs/foundation/testing-strategy.md`, 8 and 8.1: each tree's CPU
   budget, over budget is red, starved is a warning. Slow-tier profiles never run
   in the gate.
5. **The old-forms counts:** `(cd TREE && python3 -I -B scripts/old_forms.py)` for
   each tree the branch touches; add `--write` to lower counts for what it removed.
   Counts only fall (`tools/old_forms/README.md`).
6. **Citations:** `python3 -I -B tools/citations/citations.py --all`, at 0 broken.
   Rules: `DESIGN.md`, 4; `tools/citations/README.md`.
7. **The outward check:** `tools/tiers/one-process default -p fabrig-boundary`,
   part of item 4's focused suite. Rule: `tests/boundary/tests/outward.rs`'s module doc and `AGENTS.md`, 1.
   The rule is absolute: any match fails, and the check has no inventory.
8. **Lint profiles:** `python3 -I -B tools/lints/lints.py`, at 0 findings, and
   `python3 -I -B tools/lints/test_lints.py`. Rule: `tools/lints/design.md`, 5.
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
    part of item 4's focused suite. Rule: `temper/docs/design/protocol/baseline.md`, 7,
    and `tests/boundary/tests/patches.rs`: every stored owner passes `check` against
    git's landed files and `current`, and the owner list covers every kept crate.
12. **The size check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `tools/crates/design.md`, 3.1, and
    `tests/boundary/tests/sizes.rs`'s module doc: library and shared-family lines,
    with their shrinking exceptions.
13. **The client layouts' check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `jig/docs/design/protocol/client/README.md`, 6,
    and `tests/boundary/tests/layouts.rs`: each listed wire's versions rise by one,
    the last line is its schema's version and family fingerprint, and git's landed
    list stands unchanged at the head. The base is the branch's merge base with
    `origin/main`, or `HEAD` when that reference is absent, as in the patch check.
14. **The features check:** `tools/tiers/one-process default -p fabrig-boundary`,
    part of item 4's focused suite. Rule: `tools/crates/design.md`, 3.3, and
    `tests/boundary/tests/features.rs`'s module doc: external entries carry every
    context's feature union, platform entries stay on their platforms, and
    `.cargo/config.toml` enables Cargo's workspace feature unification.

## The gate's lists

Each list has one entry per line; `#` starts a comment line.

- **`.config/gate/always.txt`:** package names whose tests read other packages'
  files by path, so changes outside their dependencies can break them. A gate
  selecting the packages a change reaches runs these too. The branch adding such
  a test adds its package; adding a line runs that package's tests for every change.
  A line `PACKAGE PATH-PREFIX` runs them only for a change to a path that starts with
  the prefix, as written: for a test whose reads all lie under it.
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
  `.config/nextest.toml` are `browser-skein`, `live-skein`, `live-smith`, `live-jig`,
  `browser-jig`, `process-temper`, `live-temper`, `browser-temper` and
  `conformance-hearth`. Never in the gate. Rules: `tools/tiers/design.md`, 1, 4 and 7;
  `skein/docs/foundation/testing-strategy.md`, 8.1.
- **The after-landing tier:** after each landing on main,
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
- **What red means:** a failure observed after landing is red; repair starts in
  the lowest tier that shows it (`skein/docs/foundation/testing-strategy.md`, 9).
  Over a CPU budget is red; past the wall hard limit is starved, a warning; tests
  the run did not reach are unobserved, never passed
  (`skein/docs/foundation/testing-strategy.md`, 8.1). A caller reads the verdict
  and its `unobserved` attempts through `tools/tiers/design.md`, 10, with summary
  semantics in 7.3.

## The repository's hooks and fixture readers

- **`.temper/pre-pr`:** the format check and each tree's old-forms counts at
  delivery, then clippy on changed packages and focused tests, one process per binary
  (`tools/tiers/one-process default`), on them, their direct dependents and
  prefixed path readers, against `refs/temper/landing`'s
  merge base. A lint or test failure `.temper/known-reds.md` lists, and an
  exceeded count or citation the base has too, does not refuse; without a
  base, the first two checks alone (`tools/pre-pr/design.md`, 1–9).
- **`.temper/merge`:** the reference's merge job, `tools/merge/merge.py`
  (`temper/docs/design/protocol/tooling.md`, 4.1).
- **`.temper/close`:** fabrig's close command, `tools/close/close.py`
  (`temper/docs/design/protocol/tooling.md`, 4.2).
- **`.config/fixture-readers.toml`:** the readers and generators of every tracked
  `**/fixtures/**` file (`tools/tiers/design.md`, 9;
  `skein/docs/foundation/testing-strategy.md`, 8).
