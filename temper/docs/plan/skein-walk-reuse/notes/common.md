# skein-walk-reuse: rules every task keeps

- **The request** is `temper/docs/plan/skein-walk-reuse/inputs/request.md`:
  skein-codec's `nested_walks_grow_both_stacks_and_reuse_them` fails in
  one process depending on which tests ran before it. Read it for the
  captured failure and the why. Build what your spec and the design
  sections it names say.
- **The design is written:** `skein/docs/design/codec.md`, 11.8 (the
  *Memory* bullet: capacities and the span's count show reuse, never an
  address) and `skein/docs/design/testing.md`, 5 (*Reuse is counted,
  never read from an address*: `Span::made`). Their status files each
  hold one **Not yet** entry for it, and `skein/docs/design/STATUS.md`'s
  first paragraph names the codec's gap; your spec says how each goes.
  Those lines are your task's, though the implementer's text closes
  `docs/design/` to you; no other line of `docs/design/` is.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a process,
  live or browser tier, or `scripts/browser.sh`. No walk, render check
  or throwaway driver.
- **No budget, tier, filter or override changes.** No figure in
  `tools/tiers/budgets.toml`, nothing in `.config/nextest.toml`, and no
  test moved to a process of its own: the fix is at the cause.
- **Old forms:** `python3 -I -B skein/scripts/old_forms.py` once before
  pushing. A raised count is yours to fix.
- **Decide, don't ask.** A small edit just outside your file list is
  made, and named in the report. A `Question:` is only for what truly
  blocks your acceptance.
- **Before calling a focused or fuzzy failure "outside the change,"**
  run it once on the pristine base. If it passes there and fails on your
  branch, it is your branch's.
- **Citations:** after committing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from
  the repository root. It reads HEAD, not the working tree.
