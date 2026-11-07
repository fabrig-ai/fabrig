+++
id = "comment"
part = "comment"
kind = "docs"
needs = []
files = ["tools/tiers/budgets.toml"]
effort = "medium"
minutes = 10
+++

# Smith's gate entries carry the user's word

## Goal

`tools/tiers/budgets.toml`'s comment on `[gate.smith.default]` (30) and `[gate.smith.fuzzy]` (120) says whether those are their figures "stays open". The user confirmed on 11-07 that they are, as CPU seconds. Replace that one comment line with the request's line, and change nothing else.

Acceptance:

- `tools/tiers/budgets.toml` line 5 reads exactly:

  ```toml
  # Smith's focused and fuzzy suites: 30 and 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
  ```

- `git diff origin/main -- tools/tiers/budgets.toml` shows that one line removed and that one line added, nothing else; `git diff --stat origin/main` names `tools/tiers/budgets.toml` alone.
- `python3 -I -B tools/tiers/test_cpu.py` passes.
- The real file still validates: `python3 -I -B -c "import runpy; runpy.run_path('tools/tiers/cpu.py')['budgets']('tools/tiers/budgets.toml')"` exits 0.

## Inputs

- The request: `smith/docs/plan/smith-gate-comment/inputs/request.md`, whole (it is short). Its `Limits` bind this task.
- `tools/tiers/design.md`, 7.3 (what `budgets.toml` holds and that its comments tell the user's approved limits from provisional ones), for context only.

Departures from the implementer's text, and why:

- **No `heavy`, no cargo:** nothing here builds. No `cargo fmt`, clippy or nextest, and no `old_forms.py`: the file is in `tools/`, no tree's code.
- **No test is added,** so no break-by-hand step: the change is one comment line, and `test_cpu.py` is the request's named check of the file's shape.
- **The design stays as it is:** `tools/tiers/design.md`, 7.3 and 8 and `tools/tiers/design.status.md` still say Smith's figures remain open. The request forbids any doc change in this plan, so leave them; do not mention them under `## Left` either, the cut has recorded it.

## Owns and may change

Owns: `tools/tiers/budgets.toml`, line 5 alone (the `# Smith's focused and fuzzy suites: ...` comment), replaced whole.

Not to touch: every other line of `tools/tiers/budgets.toml`, including the Skein, Hearth and Jig comments that still say "stays open" (the user's word was given for Smith's entries alone), every entry and figure, and every other file.

Reader and neighbour files: none. The lead's survey (`git grep` for the comment's text and for `budgets.toml`): no test reads the real file's comments; `test_cpu.py` and `test_run.py` read fixture copies they write themselves; `cpu.py`'s `budgets()` and `tools/tiers/run` read the values, which do not change.

Pre-approved options:

- **If the line on the base is not the request's old line word for word,** replace whichever line begins `# Smith's focused and fuzzy suites:` with the new line, and say so in the report.

## Steps

1. Replace line 5 of `tools/tiers/budgets.toml` with the new line above, byte for byte.
2. Run `python3 -I -B tools/tiers/test_cpu.py` and the one-line validation of the real file from the acceptance; check `git diff --stat origin/main`.
3. Commit (`smith-gate-comment: Smith's gate entries carry the user's word`, the body saying the user confirmed 30 and 120 as CPU seconds on 11-07) and push.

## Report

As the implementer's text says: the first line, the commit pushed, the two checks' outcomes (test count for `test_cpu.py`), and the minutes.
