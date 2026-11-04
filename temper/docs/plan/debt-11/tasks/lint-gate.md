+++
id = "lint-gate"
part = "gate"
kind = "docs"
needs = ["hearth-profiles", "jig-profiles", "skein-clippy", "temper-clippy", "map-busy"]
files = [".config/gate/README.md", "tests/boundary/tests/lints.rs", "AGENTS.md", "tools/lints/design.status.md", "jig/docs/development/debt/**", "skein/docs/development/debt/**", "temper/docs/development/debt/**", "hearth/docs/development/debt/**"]
effort = "high"
minutes = 45
+++

# The check goes on, and the four items go

## Goal

Every package and every `clippy.toml` of the four trees is on a definition by now, so turn the check on: it joins the
gate's list and runs in a test the gate selects on every change. Delete the four items and their rows in one branch,
since each cites the others. Build the table the request asks for: each lint whose level changed for any crate.

Acceptance:

- `python3 -I -B tools/lints/lints.py`, with no path given, reads 0 findings.
- `python3 -I -B tools/lints/test_lints.py` passes.
- `.config/gate/README.md` has the check as a gate item.
- `heavy cargo nextest run -p fabrig-boundary` passes, and its new test can fail: see **Bite**.
- D011's measure reads 0. `rg -n '^disallowed_\w+ = "(allow|warn)"' -g Cargo.toml skein jig temper hearth` finds nothing.
- The four item files and their README rows are gone, and `python3 -I -B tools/citations/citations.py --all` reads 0.
- `tools/lints/design.status.md` says there is no gap.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 3 and 4
- `tools/lints/design.md`, 5
- `jig/docs/development/debt/D019-lint-config-copied.md`, `skein/docs/development/debt/D036-lint-blocks-by-hand.md`,
  `temper/docs/development/debt/D011-manual-map-expectations.md`, `hearth/docs/development/debt/D010-lint-config-copied.md`,
  whole, with each tree's `docs/development/debt/README.md`
- `tests/boundary/tests/profiles.rs` for the form of a boundary test
- `DESIGN.md`, 2, for the status file

## Owns and may change

This task spans the trees: the items are one job (the request). That departs from `notes/common.md`'s first line.

- **Owns:** `.config/gate/README.md`, `tests/boundary/tests/lints.rs`, `tools/lints/design.status.md`.
- **May change:** the four item files (deleted) and one row in each tree's debt README (deleted). The root
  `AGENTS.md`, section 2: one bullet, after the step crates' line, as below. Nothing else.
- **Options:**
  - If another draft landed a package or a `clippy.toml` that is no definition's copy, move it as a tree's task would:
    `--set`, or copy the variant, and meet what turns red by `notes/common.md`'s list. Say so in the report.
  - If `lints.py` finds what a merge broke in a package with real fallout, that is yours too, up to a dozen sites.
    Past that, report it under `## Left`.
  - D019's, D036's and D010's measures count manifests with a block of their own, and the request has each manifest
    keep its block, so they stay above 0: `lints.py` at 0 findings is their target. Report what each reads.

## Steps

1. Add to `.config/gate/README.md`'s list, after item 7, the outward check, in the form of its neighbours:

   ```markdown
   8. **Lint profiles:** `python3 -I -B tools/lints/lints.py`, at 0 findings, and
      `python3 -I -B tools/lints/test_lints.py`. Rule: `tools/lints/design.md`, 5.
   ```

   Renumber nothing.
2. `tests/boundary/tests/lints.rs`, a thin test: it runs the check from the workspace root and asserts on its exit.

   ```rust
   //! Every package and `clippy.toml` of the four trees is a definition's copy
   //! (tools/lints/design.md, 5). The test runs the check, so that it runs on every change.

   #[test]
   fn every_package_takes_a_lint_profile() { ... }
   ```

   Spawn `python3 -I -B tools/lints/lints.py` with `Command`, current directory `skein_scratch::paths::workspace()`.
   A check that can't start is a failure, never a skip. The assertion's message carries the check's stdout. Add a
   `[[test]]` to `tests/boundary/Cargo.toml` if its neighbours need one; they don't at the base.
3. **Bite.** Change one lint's level in one manifest, see the new test fail and the message name that manifest, and
   restore. Do the same with one byte of one `clippy.toml`. Neither break is committed.
4. Root `AGENTS.md`, section 2, after the "Step crates follow the strict subset" bullet:

   ```markdown
   - A package's `[lints]` block and its `clippy.toml` are copies of a definition (`tools/lints/design.md`): a new crate takes a profile with `tools/lints/lints.py --set`, and the gate holds it there.
   ```
5. Delete the four items and their rows. `tools/lints/design.status.md`, in `DESIGN.md`, 2's form, says the
   repository has no gap, with today's date.
6. Citations, fmt, the old forms of each tree you touched. Commit and push.

## Report

The report of your implementer text, and:

- **The level table.** From `git show BASE:PATH` of every manifest against its block now, where BASE is this plan's
  first commit on its trunk: one row per lint and move, with the crates, or their count where many, in the form of the
  contracts' section 3. A throwaway script, untracked, deleted before the push. Say which rows the contracts' section 3
  lacks, and which it has that did not happen. Likewise the bans of section 4, by crate.
- **The bite table:** what you broke, which test failed, what it read.
- What each of D019's, D036's and D010's measures reads, and D011's.
- What `lints.py` printed.
