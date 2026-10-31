+++
id = "counts"
part = "counts"
kind = "feature"
needs = []
files = ["skein/scripts/old_forms.py", "skein/scripts/old-forms.txt", "skein/scripts/test_old_forms.py"]
effort = "medium"
minutes = 30
+++

## Goal

skein's old-forms ratchet counts the seven forms of
`skein/docs/plan/component-kit-3/notes/counts.md`, beside its four
existing forms, and `skein/scripts/old-forms.txt` records their baseline.
Done when `python3 -I -B scripts/old_forms.py` (from `skein/`) passes on
your branch, the file lists all eleven forms, and each new form's count is
the table's baseline or the report says why it differs.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`.
- `skein/docs/plan/component-kit-3/notes/counts.md`: the forms, their
  regexes, their excluded paths and their baselines.
- `skein/scripts/old_forms.py` as it stands; `temper/scripts/test_old_forms.py`
  shows the shape of a test of it.

## Owns and may change

- Owns `skein/scripts/old_forms.py`, `skein/scripts/old-forms.txt`, and a new
  `skein/scripts/test_old_forms.py`.
- Nothing else. No Rust file changes in this task.
- Options, pre-approved:
  - The new forms match over comment-stripped source, not over the
    identifier prefilter the max_out, JournalRoom and worst_case forms use
    (as `takes` does). The file prefilter (`any(marker in source ...)`)
    needs a literal marker per form, since the form names do not occur in
    the source: give each form one (`resume`, `earl`, `Lifecycle`,
    `GROWTH_FLOOR`, `pop()`, `fn `, `Host`).
  - A per-form excluded path prefix (relative to `skein/`): add it as a
    table beside `EXEMPT`, applied to the path before reading the file.
  - Keep the existing four forms, their order and their behaviour; the new
    seven follow them in the table's order. The header and rule texts may
    name `skein/docs/design/draft/component-kit.md, 4` beside
    dynamic-memory.md's section 7.
  - If a regex of the table matches something plainly not its form (count
    above the baseline), tighten the regex, say how in the report, and keep
    the count's meaning. If it is below, record the script's count.

## Steps

1. Add the forms, the per-form exclusions and the markers to `old_forms.py`.
2. A test `skein/scripts/test_old_forms.py` (standard library `unittest`,
   run as `python3 -I -B -m unittest scripts/test_old_forms.py` from `skein/`),
   in a temporary git repository like temper's: for each new form, one
   source that counts and one near miss that does not (a comment, an excluded
   path, `fn hex` inside `crates/skein-lib/`, a `for _ in 0..n` loop whose
   body does more than discard a pop).
3. Initialize the new lines: from `skein/`, `python3 -I -B scripts/old_forms.py --write`
   adds nothing for missing forms, since the file must list each form once;
   append the seven lines in order with their counts, then run the script
   without `--write` to check it passes.
4. Run the new test and the script. Commit and push.

## Report

The eleven counts, each new form's against the table, and any regex you
tightened, with the match that made you.
