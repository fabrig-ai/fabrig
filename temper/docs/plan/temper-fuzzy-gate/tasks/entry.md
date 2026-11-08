+++
id = "entry"
part = "entry"
kind = "feature"
needs = []
files = ["tools/tiers/budgets.toml"]
effort = "medium"
minutes = 10
+++

# Temper's fuzzy suite gets its gate entry

## Goal

Add temper's fuzzy gate entry to `tools/tiers/budgets.toml`, with `cpu_suite_s = 120` and nothing else, after `[gate.jig.fuzzy]`, plus one comment line after Jig's fuzzy comment. The user gave the figure on 11-07. It is the strategy's figure for a tree's fuzzy suite. The queue's record reads temper's fuzzy suite at 102 to 105 CPU seconds, so no code or test changes. Nothing else changes.

Acceptance:

- Line 8 of `tools/tiers/budgets.toml`, right after the line beginning `# Jig's fuzzy suite:` and before the line beginning `# Optional per-pair auto_reap`, reads exactly:

  ```toml
  # Temper's fuzzy suite: 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
  ```

- Right after the `[gate.jig.fuzzy]` table (its `cpu_suite_s = 120` line and the blank line after it), and before `[browser-skein]`, the file holds these two lines and then one blank line:

  ```toml
  [gate.temper.fuzzy]
  cpu_suite_s = 120
  ```

- `git diff origin/main -- tools/tiers/budgets.toml` shows only those four lines added (the comment, the table header, its key and one blank line), with nothing removed. `git diff --stat origin/main` names only `tools/tiers/budgets.toml`.
- `git grep -n 'gate.temper' -- tools/tiers/budgets.toml` prints `[gate.temper.fuzzy]` alone: there is no `[gate.temper.default]`.
- `python3 -I -B tools/tiers/test_cpu.py` passes (51 tests on the base).
- The real file still validates: `python3 -I -B -c "import runpy; runpy.run_path('tools/tiers/cpu.py')['budgets']('tools/tiers/budgets.toml')"` exits 0.

## Inputs

- The request: `temper/docs/plan/temper-fuzzy-gate/inputs/request.md`, read whole (it is short). Its `Limits` bind this task.
- `skein/docs/foundation/testing-strategy.md`, 8.1: the CPU budget a tree's suite is held to. This is background only.
- `tools/tiers/design.md`, 7.3: what `budgets.toml` holds. A `[gate.<tree>.<profile>]` table takes `cpu_suite_s`, `cpu_test_s` and `wall_hard_s`, and here it takes `cpu_suite_s` alone. `cpu.py`'s `budgets()` accepts any tree name there. This is background only.

Departures from the implementer's text, and why:

- **No `heavy`, no cargo:** nothing here builds. Run no `cargo fmt`, clippy or nextest, and no `old_forms.py`, because the file sits in `tools/` and belongs to no tree's code.
- **No test is added,** so you skip the break-by-hand step. The change is one entry and one comment, and the request names `test_cpu.py` as its check of the file's shape. Nothing is measured.
- **The tiers' design stays as it is.** `tools/tiers/design.md`, 7.3, lists Smith's, Skein's and Jig's gate entries and not temper's. The request limits this plan to `budgets.toml`, so leave that design alone. Don't list it under `## Left` either, because the cut has already recorded it in `plan.md`.

## Owns and may change

Owns: `tools/tiers/budgets.toml`. You add only the comment line and the two-line table above, plus the blank line that separates the new table from `[browser-skein]`.

Not to touch: every other line of `tools/tiers/budgets.toml`, including line 2 (`# Live pairs and other default/fuzzy suites have no entry yet ...`). That line stays true, because temper's focused suite and the live pairs still have no entry. Leave every other entry and figure, and every other file.

Reader and neighbour files: none. The lead's survey (`git grep` for `budgets.toml` and `gate.` across the repository, and `.config/gate/read-by-tests.txt`) found that no test reads the real file's entries or comments. `test_cpu.py` and `test_run.py` write their own fixture copies. `cpu.py`'s `budgets()` validates `[gate.*]` tables, and `tools/tiers/run` reads only pairs, not `gate`. No design under `temper/docs/design/` names a figure for temper's fuzzy suite.

Pre-approved options:

- **If the base already holds other entries or comment lines another plan landed** (another tree's gate entry, say), keep them all. Put the new table right after `[gate.jig.fuzzy]`, and the new comment right after the line beginning `# Jig's fuzzy suite:`, wherever those now sit. Give the line numbers in the report.
- **If `[gate.jig.fuzzy]` or Jig's comment line is missing from the base,** put the table after the last `[gate.*]` table and the comment after the last tree's comment line. Say so in the report.
- **If the base already holds `[gate.temper.fuzzy]`,** change nothing. Report `Done:` saying so.

## Steps

1. Add the comment line and the table to `tools/tiers/budgets.toml` where the acceptance places them, byte for byte.
2. Run `python3 -I -B tools/tiers/test_cpu.py`, the one-line validation of the real file from the acceptance, and `python3 -I -B tools/citations/citations.py --base origin/main`. Then check `git diff origin/main -- tools/tiers/budgets.toml`, `git diff --stat origin/main` and the `git grep` from the acceptance.
3. Commit and push. The commit message is `temper-fuzzy-gate: temper's fuzzy suite gets its gate entry`, and its body says that the user gave 120 CPU seconds on 11-07, the strategy's section 8 figure, and that the queue's record reads the suite at 102 to 105 CPU seconds.

## Report

Follow the implementer's text: the first line, the commit you pushed, the checks' outcomes (with the test count for `test_cpu.py`), and the minutes.
