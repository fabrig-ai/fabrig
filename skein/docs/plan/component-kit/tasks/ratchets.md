+++
id = "ratchets"
part = "kit-contract"
kind = "feature"
repo = "fabrig"
needs = []
files = ["skein/scripts/old_forms.py", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 15
+++
# The draft's counts in skein's old-forms ratchet

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, section 4: each count is recorded on the trunk before the first
site moves, and may only fall. Add the seven forms of `decisions.md`'s
table to `skein/scripts/old_forms.py` and record their baselines in
`skein/scripts/old-forms.txt` with `--write`.

## What changes

- `FORMS` gains, in this order after the four it has: `io_resume`,
  `earliest`, `lifecycle`, `growth_floor`, `pop_loop`, `hex_decimal`,
  `host_dispatch`, each a pattern over the comment-free source:
  - `io_resume`: `\bio::resume\(` (both `io::resume(` and
    `skein_io::resume(`), outside `crates/skein-io/`;
  - `earliest`: `\bfn (?:earliest|earlier)\b`, outside `crates/skein-lib/`;
  - `lifecycle`: `\benum Lifecycle\b`, outside `crates/skein-lib/`;
  - `growth_floor`: `\bconst GROWTH_FLOOR\b`;
  - `pop_loop`: a `for _ in 0..` loop whose body is one discarded pop
    (`q.pop();`, `drop(q.pop());`, `let _ = q.pop();`) and nothing else;
  - `hex_decimal`: `\bfn (?:hex|decimal)\b`, outside `crates/skein-lib/`;
  - `host_dispatch`: `fn completions(&mut self)` whose body begins with
    `match self`.
- The per-file prefilter and the identifier prefilter serve the four
  identifier forms only; the new forms run on the whole comment-free
  source, as `takes` does. A form's path exclusion is a prefix of the
  path from `skein/`.
- The module docstring names the new forms and their exclusions in a
  line or two.

## Acceptance

- `python3 -I -B scripts/old_forms.py --write` from `skein/` records the
  baselines, and a second run without `--write` exits 0.
- The baselines match `decisions.md`'s survey column; a difference is
  reported with the sites that make it (`git grep` lines), not hidden.
- The four existing counts are unchanged (all 0).
- A throwaway file with one added `io::resume(` raises the count and the
  script exits 1; delete it before committing.

## Report

The seven baselines, each with the survey's number beside it, and the
time one run takes.
