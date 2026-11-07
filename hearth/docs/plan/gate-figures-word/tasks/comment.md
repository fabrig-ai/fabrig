+++
id = "comment"
part = "comment"
kind = "docs"
needs = []
files = ["tools/tiers/budgets.toml"]
effort = "medium"
minutes = 10
+++

# Hearth's, Skein's and Jig's gate entries carry the user's word

## Goal

Three comment lines in `tools/tiers/budgets.toml` still say whether the gate figures beside them "stays open": Hearth's fuzzy (line 4), Skein's focused and fuzzy (line 6) and Jig's fuzzy (line 7). The user confirmed on 11-07 that the figures, 30 for focused and 120 for fuzzy, are CPU seconds per tree. Bring the three lines to the form Smith's line 5 already has. Nothing else changes.

Acceptance:

- Lines 4, 6 and 7 of `tools/tiers/budgets.toml` read exactly, in this order around the unchanged line 5:

  ```toml
  # Hearth's fuzzy suite: 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
  # Smith's focused and fuzzy suites: 30 and 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
  # Skein's focused and fuzzy suites: 30 and 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
  # Jig's fuzzy suite: 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
  ```

- `git diff origin/main -- tools/tiers/budgets.toml` shows those three lines removed and their three replacements added, nothing else; `git diff --stat origin/main` names `tools/tiers/budgets.toml` alone.
- `git grep -n 'stays open' -- tools/tiers/budgets.toml` prints nothing.
- `python3 -I -B tools/tiers/test_cpu.py` passes.
- The real file still validates: `python3 -I -B -c "import runpy; runpy.run_path('tools/tiers/cpu.py')['budgets']('tools/tiers/budgets.toml')"` exits 0.

## Inputs

- The request: `hearth/docs/plan/gate-figures-word/inputs/request.md`, whole (it is short). Its `Limits` bind this task.
- `hearth/docs/design/domain/testing.md`, 9.1, as this plan's cut wrote it: Hearth's fuzzy suite takes at most 120 CPU seconds, by the user's word of 11-07, kept in `tools/tiers/budgets.toml`. Section 11's item "The suites' figures" no longer asks whether it is CPU seconds. Line 4's old citation of section 11 therefore goes with the old text.
- `tools/tiers/design.md`, 7.3 (what `budgets.toml` holds, and that its comments tell the user's approved limits from provisional ones), for context only.

Departures from the implementer's text, and why:

- **No `heavy`, no cargo:** nothing here builds. No `cargo fmt`, clippy or nextest, and no `old_forms.py`, since the file is in `tools/` and belongs to no tree's code.
- **No test is added,** so there is no break-by-hand step: the change is three comment lines, and `test_cpu.py` is the request's named check of the file's shape.
- **The tiers' design stays as it is:** `tools/tiers/design.md`, 7.3 and 8 and `tools/tiers/design.status.md` still say Skein's and Jig's figures remain open. The request limits this plan to `budgets.toml` and hearth's testing design, so leave them. Don't list them under `## Left` either: the cut has recorded it.

## Owns and may change

Owns: `tools/tiers/budgets.toml`, lines 4, 6 and 7 alone (the comments beginning `# Hearth's fuzzy suite:`, `# Skein's focused and fuzzy suites:` and `# Jig's fuzzy suite:`), each replaced whole.

Not to touch: every other line of `tools/tiers/budgets.toml`, including line 3 (`# Hearth's focused suite: the user's 11-04 ruling ...`, which stays as it is), line 5 (Smith's, already in the new form) and line 2 (`# Live pairs and other default/fuzzy suites ...`); every entry and figure; and every other file, `hearth/docs/design/` included (the cut has already changed it).

Reader and neighbour files: none. The lead's survey (`git grep` for the three comments' text, for `stays open` and for `budgets.toml`) found that no test reads the real file's comments. `test_cpu.py` and `test_run.py` read fixture copies they write themselves. `cpu.py`'s `budgets()` and `tools/tiers/run` read the values, which do not change. The citation checker reads line 4's citation of `hearth/docs/design/domain/testing.md, 11`; removing it breaks nothing.

Pre-approved options:

- **If a line on the base is not the request's old line word for word,** replace whichever line begins with that tree's prefix above with its new line, and say so in the report.
- **If another tree's comment line on the base says "stays open"** about its focused or fuzzy gate figure, bring it to the same form, naming its own suites and figures (`30` for `default`, `120` for `fuzzy`, as its `[gate.<tree>.<profile>]` entries hold), and say so in the report.

## Steps

1. Replace lines 4, 6 and 7 of `tools/tiers/budgets.toml` with the new lines above, byte for byte.
2. Run `python3 -I -B tools/tiers/test_cpu.py`, the one-line validation of the real file from the acceptance, and `python3 -I -B tools/citations/citations.py --base origin/main`; check `git diff --stat origin/main` and the `git grep` from the acceptance.
3. Commit (`gate-figures-word: every tree's gate figures carry the user's word`, the body saying that the user confirmed on 11-07 that 30 and 120 are CPU seconds per tree) and push.

## Report

As the implementer's text says: the first line, the commit pushed, the checks' outcomes (the test count for `test_cpu.py`), and the minutes.
