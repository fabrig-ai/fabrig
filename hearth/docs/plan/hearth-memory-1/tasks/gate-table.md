+++
id = "gate-table"
part = "budget"
kind = "feature"
needs = []
files = ["tools/tiers/budgets.toml", "tools/tiers/cpu.py", "tools/tiers/test_cpu.py", "tools/tiers/test_run.py"]
effort = "medium"
minutes = 25
+++

# The gate's table in the budgets file

## Goal

`tools/tiers/budgets.toml` takes the user's `[gate.hearth.default]`, `cpu_suite_s = 30`. Added alone, that table breaks the file's reader:
`budgets()` in `tools/tiers/cpu.py` refuses a top-level `gate` as an unknown pair, or as invalid keys when no `pairs` are given, so
`tools/tiers/run --verify-all` and `--cpu` would exit 2. The reader learns the table first.

Acceptance:

- The table is in the file, with the figure as the request gives it.
- `budgets()` returns it and refuses a malformed one, with or without `pairs`.
- `tools/tiers/run --verify-all` validates the real file with exit 0.
- `python3 -I -B tools/tiers/test_cpu.py` and `python3 -I -B tools/tiers/test_run.py` pass.

## Inputs

- `tools/tiers/design.md`, 7.3 and 10.2, which this cut already changed.
- `hearth/docs/plan/hearth-memory-1/inputs/request.md`, 4.
- `hearth/docs/plan/hearth-memory-1/notes/common.md`, "What doesn't move".

## Owns and may change

- **Owns:** the files under `files`.
- **Departure from the implementer's text:** there's no Rust here, so no `heavy`, `cargo fmt` or clippy. `old_forms.py` doesn't apply.

## Steps

1. In `budgets()`, beside `tools`, take `gate`: a table of trees, each a table of profiles, each table validated by `validate` with no
   `auto_reap`. Refuse an empty name, and a level that isn't a table. Don't check names against a list of trees or profiles.
2. `summary` reads one pair's table, so `gate` needs nothing there. Check that, and leave it.
3. Add the table to `tools/tiers/budgets.toml`:

   ```toml
   [gate.hearth.default]
   cpu_suite_s = 30
   ```

   Its header says default and fuzzy have no entry yet. Make that line true: hearth's focused suite has one now, by the user's ruling of 11-04
   in `hearth/docs/plan/hearth-memory-1/inputs/request.md`, 4. Change no other figure.
4. Test, as rows of `test_budget_validation`'s shape in `tools/tiers/test_cpu.py`:
   - accepted, with and without `pairs`: the table above, returned as `{'gate': {'hearth': {'default': {'cpu_suite_s': 30}}}}`;
   - refused: `gate=1`, `[gate]\ncpu_suite_s=1`, `[gate.hearth]\ncpu_suite_s=1`, `[gate.hearth.default]\nunknown=1`,
     `[gate.hearth.default]\ncpu_suite_s=nan`, `[gate.hearth.default]\nauto_reap=[]`.
5. In `tools/tiers/test_run.py`, a test that `--verify-all` accepts a `[gate.hearth.default]` table, by the shape of
   `test_verify_all_validates_budgets_before_any_command`.
6. Commit and push.

## Report

The first line, the tests run and how many, and what broke each new test when you removed the rule.
