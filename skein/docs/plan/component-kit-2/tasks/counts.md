+++
id = "counts"
part = "kit-contract"
kind = "feature"
needs = []
files = ["skein/scripts/old_forms.py", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 20
+++
# The draft's seven counts in skein's old-forms ratchet

## Goal

Draft, section 4: each copy the run removes is counted on the trunk
before any site moves, and the count may only fall. Add seven forms to
`skein/scripts/old_forms.py` and record their baselines in
`skein/scripts/old-forms.txt`.

Acceptance:

- From `skein/`, `python3 -I -B scripts/old_forms.py --write` records the
  seven baselines, and a run without `--write` then exits 0.
- The baselines are the survey's (table below). A difference is
  reported with the `git grep` lines that make it, never fitted away.
- The four existing counts stay 0.
- An added `io::resume(` in a throwaway `.rs` file outside
  `crates/skein-io/`, staged with `git add -N`, makes the script exit 1
  naming `io_resume`; an added `// io::resume(` comment does not. Undo
  both before committing.
- One run takes under 5 seconds on the slot (report the time).

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md` (scope, commits,
  report).
- The script as it is: `FORMS`, `EXEMPT`, `without_comments`, `counts`.

## Owns and may change

- `skein/scripts/old_forms.py`, `skein/scripts/old-forms.txt`. Nothing else.

## Steps

1. `FORMS` gains, in this order after the four it has, seven forms, each
   matched on the comment-free source of each file:

   | Form | Pattern | Excluded paths (prefix from `skein/`) | Survey |
   |---|---|---|---|
   | `io_resume` | `\b(?:skein_)?io::resume\(` | `crates/skein-io/` | 39 |
   | `earliest` | `\bfn (?:earliest\|earlier)\b` | `crates/skein-lib/` | 4 |
   | `lifecycle` | `\benum Lifecycle\b` | `crates/skein-lib/` | 3 |
   | `growth_floor` | `\bconst GROWTH_FLOOR\b` | none | 5 |
   | `pop_loop` | `for _ in 0\.\.[^{\n]*\{\s*(?:drop\()?(?:let _ = )?[\w.()&]+\.pop\(\)\)?;\s*\}` | none | 21 |
   | `hex_decimal` | `\bfn (?:hex\|decimal)\b` | `crates/skein-lib/` | 26 |
   | `host_dispatch` | `fn completions\(&mut self\)[^{]*\{\s*(?:match self\b\|self\.host_mut\(\))` | none | 9 |

   (`\|` in the table is the regex's `|`.) `io::resume(` written over a
   line break (`io::resume(\n`) matches as written; `skein_io::resume`
   counts too.
2. The per-file marker prefilter and the identifier prefilter stay for the
   four identifier forms. The new forms, like `takes`, run on the whole
   comment-free source; add their markers (`resume`, `fn `, `Lifecycle`,
   `GROWTH_FLOOR`, `pop()`, `completions`) to the per-file prefilter so a
   file holding only a new form is read. Keep one pass per file.
3. A form's exclusions are a small table beside `EXEMPT` (`EXCLUDED`,
   form → path prefixes), checked against the path `git ls-files` gives.
4. The module docstring names the seven forms and their exclusions in two
   or three lines, citing `skein/docs/design/draft/component-kit.md, 4`.
   `HEADER` and `RULE` stay as they are.
5. `python3 -I -B scripts/old_forms.py --write` from `skein/`; check the
   table's numbers; run the acceptance checks; commit both files; push.

No Rust changes, so no cargo command runs.

## Report

The seven baselines, each beside the survey's number; the run's time;
the acceptance checks and what each printed.
