+++
id = "gate-entries"
part = "gate-entries"
kind = "feature"
needs = []
files = ["tools/tiers/budgets.toml"]
effort = "medium"
minutes = 5
+++

# Skein's suites get their gate entries

## Goal

`tools/tiers/budgets.toml` holds hearth's and smith's gate entries. Add skein's two beside them, at 30 and 120, with one comment line, and change nothing else.

Acceptance:

- `git diff --stat origin/skein-gate-entries/integration...HEAD` names `tools/tiers/budgets.toml` alone, with 7 lines added and none removed. The trunk holds this plan's own files, which main doesn't, so compare with the trunk and not with main.
- The two additions in step 1 are there, byte for byte.
- The command in step 2 prints exactly `{'hearth': {'default': {'cpu_suite_s': 30}, 'fuzzy': {'cpu_suite_s': 120}}, 'smith': {'default': {'cpu_suite_s': 30}, 'fuzzy': {'cpu_suite_s': 120}}, 'skein': {'default': {'cpu_suite_s': 30}, 'fuzzy': {'cpu_suite_s': 120}}}`.
- `python3 -I -B tools/tiers/test_cpu.py` passes.

## Inputs

- `tools/tiers/design.md`, 7.3, the paragraph on `tools/tiers/budgets.toml`, and 7.4's first sentence.
- `skein/docs/plan/skein-gate-entries/notes/common.md`.

## Owns and may change

Owns `tools/tiers/budgets.toml`, for the two additions in step 1. No other file changes, and no other entry, tree or figure. The design files that name smith's entries (`tools/tiers/design.md`, `tools/tiers/design.status.md`) stay as they are.

No reader needs an edit. In this repository `budgets()` in `tools/tiers/cpu.py` validates a `[gate.<tree>.<profile>]` table and nothing reads it beyond that (`tools/tiers/design.md`, 7.3). The tests in `tools/tiers/` write their own fixture files, and no test reads the real file.

Departs from the implementer's text:

- No Rust and no tree's code changes, so no `cargo` step, `heavy` command or `old_forms.py` run applies. `test_cpu.py` runs directly, as `tools/tiers/design.md`, 7.4 says.
- No test is added, so there is nothing to break by hand.

Options, each pre-approved:

- `test_cpu.py` fails. Read the failure and run it once on the pristine base. If the base fails too, report `Done:` and list it under `## Left`. If only your branch fails, it's yours.
- Another tree's entries or comment have reached `tools/tiers/budgets.toml` before your branch starts. Keep them as they are and add the two additions where step 1 places them. Step 2 then prints those entries too, with hearth's, smith's and skein's as the acceptance gives them; the line count stays 7 added, none removed.

## Steps

1. In `tools/tiers/budgets.toml`, directly under the comment line that begins `# Smith's focused and fuzzy suites:`, add this line:

   ```toml
   # Skein's focused and fuzzy suites: 30 s and 120 s of the strategy's section 8, on as CPU seconds; whether those are their figures stays open.
   ```

   Directly after the `[gate.smith.fuzzy]` table, add a blank line and these two tables, keeping one blank line before the table that follows:

   ```toml
   [gate.skein.default]
   cpu_suite_s = 30

   [gate.skein.fuzzy]
   cpu_suite_s = 120
   ```

   `cpu_suite_s` alone, as the other trees' have it. The validator would also accept `cpu_test_s` and `wall_hard_s` (`tools/tiers/design.md`, 7.3); add neither.

2. `test_cpu.py` checks the validator on its own fixtures. Validate the real file with the same function, which builds nothing:

   ```sh
   python3 -I -B -c "import runpy; print(runpy.run_path('tools/tiers/cpu.py')['budgets']('tools/tiers/budgets.toml')['gate'])"
   ```

3. `python3 -I -B tools/tiers/test_cpu.py`.
4. `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commit and push.

## Report

As the implementer's text has it. Give the line step 2 printed, the number of tests `test_cpu.py` ran, and what the citation check said.
