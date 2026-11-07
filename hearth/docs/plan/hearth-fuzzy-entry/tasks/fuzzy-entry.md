+++
id = "fuzzy-entry"
part = "fuzzy-entry"
kind = "feature"
needs = []
files = ["tools/tiers/budgets.toml"]
effort = "medium"
minutes = 5
+++

# Hearth's fuzzy suite gets its gate entry

## Goal

`tools/tiers/budgets.toml` holds `[gate.hearth.default]` at 30. Add the fuzzy suite's entry beside it at 120, with one comment line, and change nothing else.

Acceptance:

- `git diff --stat origin/main...HEAD` names `tools/tiers/budgets.toml` alone, with 4 lines added and none removed.
- The two additions in step 1 are there, byte for byte.
- The command in step 2 prints exactly `{'hearth': {'default': {'cpu_suite_s': 30}, 'fuzzy': {'cpu_suite_s': 120}}}`.
- `python3 -I -B tools/tiers/test_cpu.py` passes.

## Inputs

- `tools/tiers/design.md`, 7.3, the paragraph on `tools/tiers/budgets.toml`, and 7.4's first sentence.
- `hearth/docs/plan/hearth-fuzzy-entry/notes/common.md`.

## Owns and may change

Owns `tools/tiers/budgets.toml`, for the two additions in step 1. No other file changes, and no other entry, tree or figure.

No reader needs an edit. In this repository `budgets()` in `tools/tiers/cpu.py` validates a `[gate.<tree>.<profile>]` table and nothing reads it beyond that (`tools/tiers/design.md`, 7.3). The tests in `tools/tiers/` write their own fixture files, and no test reads the real file.

Departs from the implementer's text:

- No Rust and no tree's code changes, so no `cargo` step, `heavy` command or `old_forms.py` run applies. `test_cpu.py` runs directly, as `tools/tiers/design.md`, 7.4 says.
- No test is added, so there is nothing to break by hand.

Options, each pre-approved:

- `test_cpu.py` fails. Read the failure and run it once on the pristine base. If the base fails too, report `Done:` and list it under `## Left`. If only your branch fails, it's yours.
- Another draft has merged a change to `tools/tiers/budgets.toml` before your branch starts. Keep their lines and add the two additions where step 1 places them.

## Steps

1. In `tools/tiers/budgets.toml`, directly under the comment line that begins `# Hearth's focused suite:`, add this line:

   ```toml
   # Hearth's fuzzy suite: 120 s of the strategy's section 8, on as CPU seconds; whether that is its figure stays open (hearth/docs/design/domain/testing.md, 11).
   ```

   Directly after the `[gate.hearth.default]` table and before `[browser-skein]`, add a blank line and this table, keeping one blank line before `[browser-skein]`:

   ```toml
   [gate.hearth.fuzzy]
   cpu_suite_s = 120
   ```

2. `test_cpu.py` checks the validator on its own fixtures. Validate the real file with the same function, which builds nothing:

   ```sh
   python3 -I -B -c "import runpy; print(runpy.run_path('tools/tiers/cpu.py')['budgets']('tools/tiers/budgets.toml')['gate'])"
   ```

3. `python3 -I -B tools/tiers/test_cpu.py`.
4. `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commit and push.

## Report

As the implementer's text has it. Give the line step 2 printed, the number of tests `test_cpu.py` ran, and what the citation check said.
