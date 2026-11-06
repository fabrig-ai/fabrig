+++
id = "inner-allow"
part = "inner-allow"
kind = "feature"
needs = []
files = ["jig/scripts/old_forms.py", "jig/scripts/old-forms.txt", "jig/scripts/test_old_forms.py", "jig/docs/development/debt/D020-inner-allow-guard.md"]
effort = "medium"
minutes = 15
+++

# An inner `#![allow` is an old form at 0 in jig

## Goal

Pay jig's D020. jig holds no inner `#![allow` today (`rg -l '#!\[allow' jig -g '*.rs'` finds none), and nothing stops one coming back: the workspace's `allow_attributes = "deny"` checks only outer attributes. The gate's old-forms step runs `(cd jig && python3 -I -B scripts/old_forms.py)` for every branch that touches jig (`.config/gate/README.md`, step 5), so a form at 0 there is the check. This is the item's second option, chosen over a `jig-boundary` test because it costs one regex and one row.

Acceptance:

- `jig/scripts/old_forms.py` has a form `inner_allow`; `jig/scripts/old-forms.txt` holds `inner_allow 0`, and the script exits 0 on the tree.
- An inner `#![allow(...)]` in any tracked `.rs` file under `jig` makes the script exit 1, naming `inner_allow: count 1, baseline 0`. An outer `#[allow(...)]`, or the words in a comment, doesn't count.
- `python3 -I -B jig/scripts/test_old_forms.py` passes, with a test for those three cases.
- `jig/docs/development/debt/D020-inner-allow-guard.md` is deleted.

This task adds a few lines, as a guard must: it's the plan's exception to "fewer lines".

## Inputs

- `jig/docs/development/debt/D020-inner-allow-guard.md`, whole.
- `tools/old_forms/README.md`.
- `AGENTS.md`, 2: a lint exception is a scoped `#[expect]`.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header, and no other. temper's `#![allow` uses stay: temper's own old forms are not this task's.

## Steps

In `jig/scripts/old_forms.py`:

- `FORMS` gains `'inner_allow': re.compile(r'#!\[allow\b')`, last, so the baseline's order keeps.
- `MARKERS` gains `'inner_allow': ('#![allow',)`.
- `RULE` gains its reason at its end, and the test's copy with it: `A lint exception is a scoped #[expect], never an inner allow (AGENTS.md, 2).` Say it in your own words if the line reads better.

`jig/scripts/old-forms.txt` gains `inner_allow 0` after `worst_case 0`: let `--write` place it if it does, otherwise by hand.

In `jig/scripts/test_old_forms.py`:

- `FORMS` gains `inner_allow`. The helper `repository` writes one baseline row per form with `zip(FORMS, counts)`, so a tuple of four would drop the new row: pad the counts with zeros inside `repository`, so the existing tests' tuples stay as they are.
- `test_a_count_above_its_baseline_fails_naming_the_form_and_the_rule`'s source gains a `#![allow(dead_code)]` line.
- One new test: `#![allow(dead_code)]` counts 1; `#[allow(dead_code)]` and `// #![allow(dead_code)]` count nothing.

Then watch the rule bite: add `#![allow(dead_code)]` to the top of one tracked jig `.rs` file, run `(cd jig && python3 -I -B scripts/old_forms.py)`, see exit 1 and the `inner_allow` line, and restore the file.

`python3 -I -B jig/scripts/test_old_forms.py`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. No Rust changes, so no build. Commit, push.

## Report

As your text says, with what you broke and what failed.
