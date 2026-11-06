+++
id = "dev-profiles"
part = "profiles"
kind = "feature"
needs = []
files = ["Cargo.toml"]
effort = "high"
minutes = 60
+++

# Optimize hearth's hot code in development builds

## Goal

The tree suite is pure computation, and the builder and render suites mostly are, but every hearth crate builds at `opt-level` 0, while `sha2` and `skein-lib` already have lines of their own. Give the hot crates lines too, with the fewest lines that meet the aim, each one kept because it measures well.

Acceptance:

- The root `Cargo.toml` gains `[profile.dev.package.<crate>]` tables, each setting `opt-level` and nothing else. `[profile.dev]`'s `overflow-checks = true` stays, and debug assertions stay on: what the tests check is the same.
- The method's figure is at most 85% of the same machine's at the base commit (`notes/common.md`), or as near as the allowed lines reach, with what's left stated.
- Each line kept saves at least 0.1 CPU s of the suite, measured. A line that doesn't is out.
- All of hearth's `default` and `fuzzy` tests pass with the lines in: a change of what a build does runs every reader's tests. `fuzzy` runs once, and its figure goes in the report.
- Each commit gives the saving and the build cost (below).

## Inputs

- `hearth/docs/plan/hearth-speed/notes/common.md`: the aim, the base commit, the method
- `hearth/docs/plan/hearth-speed/inputs/request.md`, "Debug builds of the hot code"
- The root `Cargo.toml`, `[profile.dev.package.*]`

## Owns and may change

- **Owns:** the `[profile.dev.package.*]` tables of the root `Cargo.toml`. No other part of it, and `Cargo.lock` doesn't change.
- **Departs from your text:** "the focused tests of the packages you changed and their direct dependents" here means every package under `hearth/`, which is what the lines change. Other trees' packages aren't rebuilt by a hearth line. Only `hearth-fake-git` under `hearth/` names `sha1`; `cargo tree --workspace -i sha1` says who else builds it, for the commit.
- **Candidates, pre-approved, in this order:**
  1. `sha1`, at 3 as `sha2` has it. The request names it.
  2. **The tree suite:** `hearth-lib`, `hearth-protocol-tree`, `hearth-domain-tree`, `hearth-tree-world`.
  3. **The builder and render suites:** `hearth-domain-builder`, `hearth-domain-render`, `hearth-domain-store`, `hearth-domain-build`, `hearth-protocol-builder`, `hearth-protocol-template`, `hearth-protocol-config`, `hearth-protocol-git`, `hearth-client-protocol-records`, `hearth-service-builder`, `hearth`, `hearth-fake-git`, `hearth-fake-machine`, `hearth-builder-world`, `hearth-test-render`, `hearth-render-world`.
  4. Another pure third-party dependency, if `figures.md` or your own profile shows it dominating a family.
- **The level:** try `1` and `3` on the tree suite's lines first. Use what saves most per second of build, the same level on every hearth line unless a measurement says otherwise. If `figures.md` isn't on the trunk yet, go by your own measurements: nothing here waits for it.
- **Not lines of this task:** a crate of skein, smith, jig or temper, whatever a measurement shows. Measure one if it looks large, leave it out, and put its saving and build cost under `## Left`.
- **A profile that won't hold:** if a line fails a test or a ceiling, take that line out and report the finding.

## Steps

1. **The base:** the method's two figures at the base commit. Then the build cost: `cargo clean -p` each hearth package, and time `heavy cargo nextest run --no-run` over them, with the CPU if `heavy` or `--timings` gives it. If the machines' compiler cache answers a build, it says nothing about cost: bypass it the same way on both sides with `RUSTC_WRAPPER=`, or say the figure is `unknown` and why.
2. **The search:** add a group, measure its family, keep what saves. Stop adding at the aim. Then take out, one at a time, any line whose absence costs under 0.1 CPU s.
3. **The result:** the method's two figures on the final lines, and the build cost measured as in step 1.
4. The full `default` and `fuzzy` runs of `hearth/`'s packages. The end of every task in `notes/common.md`.
5. A commit per group kept, each with what it saved, its build cost and the machine; push.

## Report

A table of each line: its saving, its build cost. The figures before and after, the machine, the level chosen and why, `fuzzy`'s figure, and `## Left`.
