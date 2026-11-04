+++
id = "gate-readme"
part = "landing-contract"
kind = "docs"
needs = []
files = [
  ".config/gate/README.md",
  ".config/gate/wasm32.txt",
  ".config/nextest.toml",
  "tests/boundary/outward.txt",
]
effort = "high"
minutes = 50
+++

# What a branch passes to land, on one page

## Goal

fabrig is to name nothing of whoever runs its development. Today the checks it asks of a landing
are described as another system's steps. This task writes them down as fabrig's own contract, in
a new `.config/gate/README.md`: each check, what it is, its command and where its rule lives,
cited, never copied. It rewords the two `.config/` lines that name a runner, and lowers the
outward inventory for every file this plan cleaned. Nothing changes what it does.

Acceptance:

- `.config/gate/README.md` holds the page of `## Steps`, 1; every section it cites exists (each
  checked by `grep -n` on its heading); it names no runner (`notes/common.md`).
- `.config/gate/wasm32.txt`'s first line and `.config/nextest.toml`'s comment above
  `[profile.live-temper]` read as `## Steps`, 2 and 3 give them, and nothing else in either file
  changes: the check of `## Steps`, 4 prints nothing.
- `tests/boundary/outward.txt` is your base's with the six entries of `## Steps`, 5 lowered or
  dropped, and nothing else; `heavy cargo nextest run -p fabrig-boundary` passes (13 selected).
- `python3 -I -B tools/citations/citations.py --all` reports 0 broken.
- No other file changes.

## Inputs

- `temper/docs/plan/landing-contract/notes/common.md`: the standing rules.
- `temper/docs/plan/landing-contract/inputs/request.md`, sections 1 and 3 and Bounds: the request.
  This spec settles what it leaves open.
- The design this plan's lead wrote, on your base (read these; cite them):
  - `tools/tiers/design.md`, 10 (a caller's interface: the commands, `cpu.py`, the files a
    caller reads), with 7.1, 7.3 and 8;
  - `smith/docs/design/testing.md`, 2.3 (`live-smith` after every landing, its two sign-ins);
  - `temper/docs/design/domain/testing.md`, 2.7 (`live-temper`'s owned-forge landing pair after
    every landing, its inputs: "What the stories need of the runner").
- The rules the page cites (read the sections, cite them by path and section):
  - `skein/docs/design/testing.md`, 6 ("The gate runs formatting, the lints as errors, clippy's
    lints as errors for `wasm32-unknown-unknown` over the portable crates ... then the focused
    suite and the fuzzy suite, with nextest ..., and nothing slower") and 10 (the portable target,
    "Checked in the gate");
  - `skein/docs/foundation/testing-strategy.md`, 8 (two suites; "The merge gate runs both suites
    in full"), 8.1 (budgets, starved, unobserved, over budget is red) and 9 (where a failure is
    fixed);
  - `AGENTS.md`, 1 (old-form counts only fall) and 2 (lint exceptions; "Some Markdown is read by
    tests ...: a docs change there needs its tests");
  - `tools/old_forms/README.md`; `tools/citations/README.md` and `DESIGN.md`, 4 (citations);
  - `tests/boundary/tests/outward.rs` (its module doc) and `tests/boundary/outward.txt` (its
    header);
  - `tools/tiers/design.md`, 1, 4, 7.3 and 9 (fixture readers);
  - `temper/docs/design/protocol/tooling.md`, 4.1 (`.temper/merge`),
    `temper/docs/design/protocol/configuration.md`, 13.9, and `smith/docs/design/domain/run.md`,
    8.1 (`.temper/pre-pr`).
- The files the page describes: `.config/gate/always.txt`, `.config/gate/read-by-tests.txt`,
  `.config/gate/wasm32.txt`, `.config/nextest.toml`, `.config/fixture-readers.toml`,
  `.temper/merge`, `.temper/pre-pr`, `tools/tiers/run`, `tools/tiers/budgets.toml`, `rustfmt.toml`,
  `tests/clippy.toml` and the trees' `clippy.toml`.

## Owns and may change

- **`.config/gate/README.md`** (new), whole.
- **`.config/gate/wasm32.txt`:** line 1 only, as `## Steps`, 2.
- **`.config/nextest.toml`:** the three comment lines above `[profile.live-temper]` only, as
  `## Steps`, 3.
- **`tests/boundary/outward.txt`:** the six entries of `## Steps`, 5 only.
- **If your base differs** from what this spec expects (a cited section moved, a count in
  `## Steps`, 5 not what the cross-check prints for that file): the cross-check decides the
  count; a section that moved is cited where it now is; the report says which. An entry of a file
  this plan did not change stays as it is, whatever it counts today.
- **If a fact the page needs is in no source above** (a command's exact flags, say): write what
  the sources say and no more, and list the gap in the report under `## Left`. Never fill it from
  memory of another system.

## Steps

1. **`.config/gate/README.md`.** One page, in fabrig's words: "a landing check", "the gate", "the
   after-landing tier", "a caller". Commands are written as they run from the checkout root, with
   plain `cargo` and `python3 -I -B`: a caller may wrap each in its own build command, and the
   page says so once. Each item gives what the check is, its command, and where its rule lives,
   as a citation (`path`, section); it never restates the rule. Its sections, in this order:

   - **`# What a branch passes to land`**, then a short paragraph: the checks fabrig asks of a
     branch before it lands on main (the gate) and of main after each landing (the after-landing
     tier); a red is a red whoever runs the check.
   - **`## The gate`**: what `skein/docs/design/testing.md`, 6 and
     `skein/docs/foundation/testing-strategy.md`, 8 say the gate is (the branch merged onto main;
     every check below; nothing slower than the fuzzy suite). Then one item per check:
     1. **Format:** `cargo fmt --all --check`; the rule is `rustfmt.toml` at the root.
     2. **Clippy, warnings denied:** `cargo clippy --workspace --all-targets -- -D warnings`, or
        over a selection, a `-p` for each package the change reaches and each in `always.txt`
        in place of `--workspace`; the rules are each tree's
        `clippy.toml`, `tests/clippy.toml` and `AGENTS.md`, 2 (a lint exception is a scoped
        `#[expect]`).
     3. **The wasm32 clippy:** `cargo clippy --target wasm32-unknown-unknown -p PKG ... -- -D
        warnings`, with a `-p` for each package in `wasm32.txt`; the rule is
        `skein/docs/design/testing.md`, 10 ("Checked in the gate").
     4. **The gate's nextest profiles:** `cargo nextest run --profile default` (the focused
        suite) and `cargo nextest run --profile fuzzy`, as `.config/nextest.toml` defines them;
        the rules are `skein/docs/foundation/testing-strategy.md`, 8 and 8.1 (each tree's CPU
        budget; over budget is red; starved is a warning). The slow tiers' profiles never run in
        the gate.
     5. **The old-forms counts:** `python3 -I -B scripts/old_forms.py` from each tree the branch
        touches, `--write` to lower a count for what the branch removed; counts only fall. The
        rule is `AGENTS.md`, 1 and `tools/old_forms/README.md`.
     6. **Citations:** `python3 -I -B tools/citations/citations.py --all` at 0 broken; the rule is
        `DESIGN.md`, 4 and `tools/citations/README.md`.
     7. **The outward check:** `tests/boundary/tests/outward.rs`, in package `fabrig-boundary`,
        which the `default` profile runs; its inventory `tests/boundary/outward.txt`, whose counts
        only fall, and a raised count needs the `# Raised: <who, why>` line its header asks for.
        The rule is its module doc and `AGENTS.md`, 2.
   - **`## The gate's lists`**: one item per file of `.config/gate/`, each with its format (one
     entry per line; `#` starts a comment line), who edits it, and what adding a line does:
     - `always.txt`: package names whose tests read other packages' files by path, so a change
       to a package they do not depend on can break them. A gate that runs only the packages a
       change reaches runs these too. The branch that adds such a test adds its package; adding
       a line makes that package's tests run for every change.
     - `read-by-tests.txt`: globs, from the checkout root, of Markdown that tests compile in or
       read (`AGENTS.md`, 2). A change to a matching file is a change to those tests' inputs:
       the gate runs the tests for it, as for code, where Markdown matching no glob runs none.
       The branch that makes a test read a Markdown file adds its glob; adding a line makes a
       change to the files it matches run the tests.
     - `wasm32.txt`: the packages the wasm32 clippy (item 3) lints. A package joins once its
       wasm32 clippy passes with warnings denied, by the branch that makes it pass; the trees'
       status files name the portable packages still left out. Adding a line adds the package to
       item 3's command.
   - **`## After landing`**:
     - **The slow tiers:** `tools/tiers/run TREE TIER [-- NEXTEST_ARGS...]` runs a tree's slow
       tier with its prerequisites, with `--cpu DIR` to account it against
       `tools/tiers/budgets.toml`; the tier profiles are `.config/nextest.toml`'s hyphenated
       ones (list them as the file has them). Never in the gate. The rule is
       `tools/tiers/design.md`, 1, 4 and 7, and `skein/docs/foundation/testing-strategy.md`, 8.1.
     - **The after-landing tier:** after each landing on main, `live-smith`
       (`smith/docs/design/testing.md`, 2.3) and `live-temper` under `--run-ignored only`
       (`temper/docs/design/domain/testing.md`, 2.7), each through `tools/tiers/run`. Name only
       these two: no design places another profile after each landing. Its inputs, by name,
       each a sign-in or a place the tier is given, never someone's store of them:
       for `live-smith`, `SMITH_LIVE`, the directory holding smith's ChatGPT record and Claude
       Code's login (smith's 2.3); for `live-temper`, `TEMPER_LIVE_ACCOUNT` (a held ChatGPT
       record), `TEMPER_LIVE_CLAUDE` (Claude Code's login) with `claude` on `PATH`,
       `TEMPER_LIVE_STORE` and a private `TMPDIR` (temper's 2.7). No API key is among them.
     - **What red means:** a failure observed in the tier after a landing is red, and its repair
       starts in the lowest tier that shows it (`skein/docs/foundation/testing-strategy.md`, 9);
       over a CPU budget is red; past the wall hard limit is starved, a warning; a test the run
       did not reach is unobserved, never passed (8.1). A caller reads the verdict and its
       `unobserved` attempts through `tools/tiers/design.md`, 10.
   - **`## The repository's hooks and fixture readers`**:
     - `.temper/pre-pr`: the format check and each tree's old-forms counts at delivery
       (`temper/docs/design/protocol/configuration.md`, 13.9; `smith/docs/design/domain/run.md`,
       8.1).
     - `.temper/merge`: the reference's merge job, `tools/merge/merge.py`
       (`temper/docs/design/protocol/tooling.md`, 4.1).
     - `.config/fixture-readers.toml`: the readers and generators of every tracked
       `**/fixtures/**` file (`tools/tiers/design.md`, 9;
       `skein/docs/foundation/testing-strategy.md`, 8).

   Check each cited section with `grep -n` on its heading before you commit. Keep the page about
   a page and a half: a line per fact, no copied rule.

2. **`.config/gate/wasm32.txt`, line 1** becomes exactly:

   ```
   # Packages the gate's wasm32 clippy lints for wasm32-unknown-unknown (README.md beside this file).
   ```

3. **`.config/nextest.toml`**, the three comment lines above `[profile.live-temper]` become
   exactly:

   ```toml
   # The after-landing tier, under --run-ignored only: the owned-forge landing pair on the
   # sign-ins the tier takes as inputs (temper/docs/design/domain/testing.md, 2.7). The owner's
   # scripts ignore this filter and name their own story.
   ```

4. **Nothing changes what it does.** From the repository root, with `BASE` the commit your
   branch started from, this prints nothing (bash):

   ```sh
   python3 -I -c 'import subprocess, sys, tomllib
   base = subprocess.run(["git", "show", sys.argv[1] + ":.config/nextest.toml"], capture_output=True, check=True).stdout
   sys.exit(0 if tomllib.loads(base.decode()) == tomllib.load(open(".config/nextest.toml", "rb")) else "nextest.toml parses differently")' "$BASE"
   diff <(git show "$BASE:.config/gate/wasm32.txt" | grep -v '^#') <(grep -v '^#' .config/gate/wasm32.txt)
   ```

5. **The outward inventory.** This plan's lead cleaned five design files on your base and steps 2
   and 1 clean or add the rest. In `tests/boundary/outward.txt`, with nothing else changed:

   | Entry at the cut | After |
   |---|---|
   | `.config/gate/wasm32.txt 1` | dropped |
   | `skein/docs/design/testing.status.md 1` | dropped |
   | `smith/docs/design/shell.status.md 1` | dropped |
   | `temper/docs/design/domain/testing.md 2` | dropped |
   | `temper/docs/design/domain/testing.status.md 2` | dropped |
   | `tools/tiers/design.md 8` | `tools/tiers/design.md 3` |

   Confirm each with the cross-check of `temper/docs/plan/outward-rule/tasks/outward-check.md`,
   `## Steps`, 4 (its `git grep` prints each file's count; a file it does not print holds none),
   run on your committed branch. `.config/gate/README.md` must print nothing there. Then
   `heavy cargo nextest run -p fabrig-boundary` (13 selected; check the count).

6. **Commit and push.** One commit for the page and the two reworded lines, one for the
   inventory, each saying why (fabrig states its landing contract in its own words; the counts
   fall for the files this plan cleaned). Then step 4's check and `python3 -I -B
   tools/citations/citations.py --all` (0 broken). Push your branch.

## Report

As the implementer's text says, and: the page's sections and every citation it holds; step 4's
output; the inventory lines you changed and the cross-check's counts for them; the nextest run's
selected count; anything under `## Left`.
