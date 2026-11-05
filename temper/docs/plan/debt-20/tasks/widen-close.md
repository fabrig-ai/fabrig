+++
id = "widen-close"
part = "skein-widening"
kind = "feature"
needs = ["widen-forge", "widen-oauth", "widen-llm", "widen-fakes", "widen-kit", "widen-ui", "widen-io"]
files = ["skein/scripts/old_forms.py", "skein/scripts/old-forms.txt", "skein/docs/development/debt/D002-index-and-length-widening.md", "jig/docs/development/debt/D043-length-widening.md"]
effort = "medium"
minutes = 15
+++

# skein: an old form holds the widening at its floor

## Goal

Pay `skein/docs/development/debt/D002-index-and-length-widening.md`. Its count stays at the floor because an old form holds it there, as the item asks, and the item goes.

```python
# skein/scripts/old_forms.py
FORMS['widening'] = re.compile(r'usize::try_from\(.*\)\.expect\(|u64::try_from\([^)]*\.len\(\)\)')
MARKERS['widening'] = ('try_from',)
# A `u32` widens with `usize_of` and a native length with `len_u64` (lib.md, 6): step code, not tests.
INCLUDE = {'widening': re.compile(r'^(crates|examples)/(?!.*(/tests/|/tests\.rs$|_tests\.rs$))')}
EXCLUDE['widening'] = ('crates/skein-lib/src/length.rs',)
```

Acceptance:

- `(cd skein && python3 -I -B scripts/old_forms.py)` passes, and `scripts/old-forms.txt` holds `widening <floor>`.
- The item's **Measure** reads the floor plus the generated sites (27) and `length.rs`'s one. Your report accounts for each site in the floor from the `widen-*` tasks' reports or by reading it: a `u64`'s conversion, or `skein-fake-checkout`'s two.
- By hand: add a `usize::try_from(n).expect("x")` to a crate's file; `old_forms.py` goes red. Do the same in a `tests.rs`, which stays green. Restore each.
- `python3 -I -B tools/old_forms/test_old_forms.py` passes, or the way that directory's tests say they run.
- The item is deleted. The citation in `jig/docs/development/debt/D043-length-widening.md`'s **Seen** line changes to words ("skein's item on widening said jig calls `len_u64` nowhere"), with nothing else in that file changed.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/widening.md`
- `skein/docs/development/debt/D002-index-and-length-widening.md`
- `tools/old_forms/old_forms.py`, and `hearth/scripts/old_forms.py` for a form with `INCLUDE`

## Owns and may change

- **Owns:** the files in `files`. D043's one line is the only edit in jig.
- **If a site is still a `u32` or a `usize` length,** a `widen-*` task missed it: move it here, and say so.
- **The forms' path test** is the tree-relative name the engine hands it. If `INCLUDE` wants another shape, follow the engine.

## Steps

1. The form; `(cd skein && python3 -I -B scripts/old_forms.py --write)`.
2. Break it by hand, as the acceptance says.
3. Delete the item; D043's line. The citations check; `python3 -I -B tools/debt/debt.py skein`. Commit and push.

## Report

The common note's items; the floor and each of its sites; what you broke and what failed.
