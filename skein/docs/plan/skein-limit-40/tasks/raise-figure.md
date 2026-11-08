+++
id = "raise-figure"
part = "skein-limit"
kind = "docs"
needs = []
files = ["tools/tiers/budgets.toml"]
effort = "medium"
minutes = 10
+++

## Goal

Skein's focused suite budget goes from 30 to 40 CPU seconds, by the
user's word of 11-08 (`skein/docs/plan/skein-limit-40/inputs/request.md`).
In `tools/tiers/budgets.toml`, `[gate.skein.default]` `cpu_suite_s = 30`
becomes `40`, and the header's skein line says so.

Acceptance:

- `[gate.skein.default]` reads `cpu_suite_s = 40`; every other table and
  value in the file is byte-for-byte as on the trunk.
- The header's skein line (today line 6, `# Skein's focused and fuzzy
  suites: 30 and 120 CPU seconds, the strategy's section 8, by the user's
  word of 11-07.`) becomes:

  ```toml
  # Skein's focused suite: 40 CPU seconds, by the user's word of 11-08; its fuzzy suite: 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
  ```

  No other comment line changes.
- `python3 -I -B tools/tiers/test_run.py` and
  `python3 -I -B tools/tiers/test_cpu.py` pass (they write their own
  budget files; the run proves the file's readers still parse the form).
- `git diff --stat` on the branch shows `tools/tiers/budgets.toml` alone.

## Inputs

- `skein/docs/plan/skein-limit-40/inputs/request.md`: the user's figure
  and the one change.
- `skein/docs/foundation/testing-strategy.md`, 8.1: CPU budgets are the
  user's; a role never raises one. This task carries the user's own
  figure, so it is not a raise by a role.
- `tools/tiers/design.md`, 7.3: the budget file's tables, the
  `[gate.<tree>.<profile>]` form, `cpu_suite_s` a finite positive number.

## Owns and may change

- Owns: `tools/tiers/budgets.toml`, the one value and the one comment
  line above.
- Changes nothing else, by the request: no other entry, no code, no test,
  no design or status file, no `.config/nextest.toml`. Design lines that
  still name skein's focused 30 (`skein/docs/design/testing.md`, 7;
  `skein/docs/design/testing.status.md`, entries under 7;
  `skein/docs/design/STATUS.md`) are not this task's; leave them as they
  are and do not list them under `## Left`, the plan's report names them.
- If the trunk's header line 6 no longer reads as quoted above (another
  draft changed it), keep its other words and change only the skein
  focused figure to 40 and its word to the user's of 11-08, and say so in
  the report.
- If `[gate.skein.default]` already reads `40` on the trunk, change
  nothing, push no commit and report `Done: already 40 on the trunk`.

## Steps

1. Edit the value and the header line as above.
2. Run `python3 -I -B tools/tiers/test_run.py` and
   `python3 -I -B tools/tiers/test_cpu.py`; neither builds or runs cargo,
   so neither goes through `heavy`. No cargo build, clippy, nextest or
   `old_forms.py` run is needed: no Rust and no counted form changes.
3. Commit with a message that says why (the user's figure of 11-08:
   skein's focused suite measured 28.5 CPU s against 30), and push.

## Report

The first line `Done:` or the case above; the diff's one value and one
comment line; the two Python suites' counts.
