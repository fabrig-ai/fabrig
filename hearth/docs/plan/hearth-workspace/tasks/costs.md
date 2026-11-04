+++
id = "costs"
part = "costs"
kind = "docs"
needs = ["test-layout", "conformance-tier", "fixture-readers", "direction"]
files = ["hearth/docs/plan/hearth-workspace/status.md"]
effort = "medium"
minutes = 100
+++

# What hearth costs the gate: the measurements for the user, in the plan's final status

## Goal

Budgets are the user's, so this plan sets none. Measure what hearth costs, on your base, the trunk
with everything else of this plan merged, and write it in the plan's final status,
`hearth/docs/plan/hearth-workspace/status.md`, with the figures awaiting the user's ruling. No code
changes.

Acceptance: `status.md` holds every row of step 5, each a measured figure or `unknown` with why, with the
commit, the machine, `date -u` and whether anything else ran on that machine.

## Inputs

- `hearth/docs/plan/hearth-workspace/inputs/request.md`, section 4.
- `notes/common.md`, in this plan's directory.
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1; `tools/tiers/design.md`, 7.1 and 10.
- `.config/gate/README.md`, the gate's item 4 and `The gate's lists`.
- This cut wrote no design: the request asks for measurements.

**Departures from the implementer's text,** each because the request asks it:

- Whole-workspace builds, the ones in step 3, and `cargo clean` of your own slot's `target/`.
- A throwaway caller of `tools/tiers/cpu.py` lives untracked, and goes before you push.
- No fmt, clippy or tests of a changed package: nothing changes but one Markdown file.

## Owns and may change

**Owns:** `status.md`. **May change:** nothing else.

Pre-approved:

- A figure a tool can't give: `unknown`, with why; never a guess.
- hearth over 30 s focused or 120 s fuzzy: not a red of yours. Report the measurements, and the slowest
  tests, and the user rules.
- Touch only your own `target/`, and `touch` (step 4) changes no content.

## Steps

1. **CPU and wall, hearth alone:** use `tools/tiers/cpu.py` as a caller, as `tools/tiers/design.md`, 7.1 and
   10.2, describe, through `heavy`, for `heavy cargo nextest run -p 'hearth*' --profile default` and
   `--profile fuzzy`. `LIMITS` is empty: `{}`.
2. **Cold and warm:** after `cargo clean`, build, then run each profile: that run's cold. Run each again
   twice straight after: warm, and report both.
3. **Added build time on a full gate:** from a clean `target/`, timed by `date -u` around each:
   `heavy cargo nextest run --workspace --exclude 'hearth*' --no-run`, then
   `heavy cargo nextest run --workspace --no-run`; and the same pair with `heavy cargo clippy ...
   --all-targets -- -D warnings`. The difference of each pair is what hearth adds over a warm rest.
4. **A skein-only change:**
   - From `cargo metadata`, which of hearth's 30 packages each of `skein-lib`, `skein-io`, `skein-shell`,
     `skein-json`, `skein-sim`, `skein-world`, `skein-heap` and `skein-scratch` reaches, and how many packages
     of the workspace each reached at this plan's base and reaches now.
   - `touch skein/crates/skein-lib/src/lib.rs`, then `heavy cargo nextest run -p 'hearth*' --no-run`, timed:
     what hearth rebuilds for that change. Its tests' cost is step 1's.
   - `.config/gate/always.txt` holds `hearth-render-world`: every change's selection runs it. Measure
     `heavy cargo nextest run -p hearth-render-world` in `default` and in `fuzzy`, and its build from a touch.
5. **`status.md`:**

   ```markdown
   # hearth-workspace: status

   <date -u>, at <commit>, on <machine>. What hearth costs the gate, for the user's ruling.

   ## Measurements

   | What | CPU | Wall |
   |---|---|---|
   | `default`, cold / warm | | |
   | `fuzzy`, cold / warm | | |
   | Full gate's test build: without hearth / added | | |
   | Full gate's clippy: without hearth / added | | |
   | A skein-lib change: hearth's rebuild | | |
   | `hearth-render-world` on every change: `default` / `fuzzy` / rebuild | | |

   ## What a skein-only change now costs

   <the reach of each skein crate, and the sum: rebuild plus both suites>

   ## Figures awaiting the user

   - **The workspace's `global-timeout`:** `default` 150 s and `fuzzy` 600 s in `.config/nextest.toml`, the sum
     of each tree's, with hearth's 30 s and 120 s, doubled provisionally on 10-22: set by the file's own rule
     until the user rules.
   - **`tools/tiers/budgets.toml`:** no entry for `conformance-hearth`, and none set for `default` or `fuzzy`.
   - **`.config/gate/always.txt`:** `hearth-render-world` runs on every change, by that file's rule, at the
     cost above.
   - **`hearth/scripts/fail_stop.py`:** kept to hearth's packages, and no gate runs it. Over the whole workspace
     it names `temper-stage-one` and `temper-client-browser-world`'s `playground-serve`.
   ```

   Keep those four, with any figure a merged task's report changed: check them against the files.
6. Commit and push.

## Report

The implementer's report, the table, and each figure's source command.
