# red-81-drain: rules every task keeps

- **The request** is `temper/docs/plan/red-81-drain/inputs/request.md`:
  temper's service never finishes draining when the terminal leaves with
  a family request admitted and unanswered. Read it for the captured
  state and the why. Build what your spec and the design sections it
  names say.
- **The design is written:** `jig/docs/design/protocol/client.md`, 4.2
  (`ExchangeGone`, and the bullet *A caller gone lets its exchange go*),
  14 (the component's world and the mapping's ends) and 17.4 (*A peer's
  end is the call's end*). `jig/docs/design/protocol/client.status.md`
  holds one **Not yet** entry for it, which the task removes whole. That
  entry is your task's, though the implementer's text closes
  `docs/design/` to you; no other line of `docs/design/` is.
- **Only focused and fuzzy run,** with one exception: the red's own
  story, `temper_open_adopts_a_repository_by_commands`, by name and
  alone, as the spec says. Never `tools/tiers/run`, another process,
  live or browser story, or `scripts/browser.sh`. No walk, render check
  or throwaway driver.
- **No budget, timeout or tier changes.** No figure in
  `tools/tiers/budgets.toml`, no timeout, slow period or thread count in
  `.config/nextest.toml`, and the story's 60 s deadline
  (`temper/crates/temper/tests/support/`) stays as it is: a longer
  patience can't supply the missing answer.
- **No sleeps and no timing luck** in a test: the worlds step their
  peers, servers and components by hand.
- **Old forms:** `python3 -I -B jig/scripts/old_forms.py` once before
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
