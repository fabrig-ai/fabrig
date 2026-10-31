+++
id = "counts"
part = "kit-contract"
kind = "feature"
needs = []
files = ["skein/scripts/old_forms.py", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 30
+++

## Goal

Add the run's seven counts to skein's old-forms ratchet, so that each copy
a later task removes lowers a count and none can rise
(`skein/docs/design/draft/component-kit.md, 4`). The baseline is today's
tree, counted by the script and written by `--write`.

Acceptance:

- `python3 -I -B scripts/old_forms.py` (from `skein/`) knows the four
  existing forms and the seven below, in that order; the four existing
  counts are unchanged (all `0`).
- `old-forms.txt` holds the seven new lines with today's counts, written
  by `--write`. On this base the figures are expected near: `io_resume
  39`, `earliest 3`, `lifecycle 3`, `growth_floor 5`, `pop_loop 20`,
  `hex_decimal 26`, `forwarding_host 9` (counted without comment
  stripping; yours, with it, win).
- Adding one more matching line anywhere counted makes the script exit 1;
  removing one lowers the figure it reports.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md`.
- `skein/docs/design/draft/component-kit.md, 4`.
- `skein/scripts/old_forms.py` whole (167 lines): its comment stripping,
  its prefilter, its `EXEMPT`.

## Owns and may change

- Owns `skein/scripts/old_forms.py` and `skein/scripts/old-forms.txt`.
- Nothing else. No Rust changes.

## Steps

1. Add the forms to `FORMS`, after the existing four, each matched on the
   comment-stripped source (as `takes` is, not on the identifier
   prefilter), with a path exclusion where given (a path prefix relative
   to `skein/`, as `git ls-files` prints it):

   | form | pattern (Python `re`) | not counted under |
   |---|---|---|
   | `io_resume` | `\b(?:skein_)?io::resume\b` | `crates/skein-io/` |
   | `earliest` | `\bfn (?:earliest\|earlier)\s*\([^)]*\bTime\b` | `crates/skein-lib/` |
   | `lifecycle` | `\benum Lifecycle\b` | `crates/skein-lib/` |
   | `growth_floor` | `\bconst GROWTH_FLOOR\b` | |
   | `pop_loop` | `for _ in 0\.\.[\w.]+\.len\(\)\s*\{\s*(?:drop\()?(?:let _ = )?[\w.]+\.pop\(\)\)?;\s*\}` | |
   | `hex_decimal` | `\bfn (?:hex\|decimal)\b` | `crates/skein-lib/` |
   | `forwarding_host` | `fn iterate\(&mut self, now: Time, wall: Wall\)\s*\{\s*(?:self\.host_mut\(\)\.iterate\|match self\b)` | |

   (In the table `\|` is a plain `|`.)
2. The file prefilter (`any(marker in source ...)`) uses the form names as
   markers; give each new form a cheap literal marker instead (`resume`,
   `fn earl`, `enum Lifecycle`, `GROWTH_FLOOR`, `pop()`, `fn hex` /
   `fn decimal`, `fn iterate`), and keep the identifier prefilter for the
   identifier forms only. The script must stay fast on the whole tree
   (time it before and after; report both).
3. Update the module docstring, `HEADER` and `RULE` to name both sources:
   dynamic-memory.md, 7 and component-kit.md, 4, and what each new form
   counts, in one line each.
4. `python3 -I -B scripts/old_forms.py --write` from `skein/`; check the
   figures against step 1's expectations and look at every match of
   `pop_loop` and `forwarding_host` (they are the multi-line patterns):
   each must be a loop that only pops, or a `Host::iterate` that only
   forwards. Fix the pattern, not the code, if one is not.
5. Probe the acceptance by hand on a scratch copy (a matching line added
   to an untracked file counts only if tracked, so use a temporary edit of
   a tracked file, reverted before committing).
6. Commit (`old_forms.py` and `old-forms.txt` together) and push.

## Report

The figures per form, and per form the files that hold its matches (one
line per file with its count): the later tasks are cut from it. The
script's time before and after.
