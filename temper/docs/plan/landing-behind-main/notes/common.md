# landing-behind-main: rules every task keeps

- **The request** is `temper/docs/plan/landing-behind-main/inputs/request.md`:
  two faults a real Forgejo deployment met. A change far behind main waited
  at its turn until its stall held it (part A), and a resolution's every
  start was refused at the worker for its charter's size (part B). Read it
  for the why; your spec and the design sections it names are what you
  build.
- **Failing tests first.** Write your spec's rows, run them, and see each
  new row fail for the reason the spec gives before you change production
  code. The report names each row's failure before the fix.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing
  table test, or of the table your spec makes from one test, on the fake
  forge and the in-crate worker route. No test reaches a real host.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or
  live tier story, a walk, a render check or a throwaway driver.
- **No figure changes.** Every limit stays as configured or as its profile
  sets it: `hub.charter_bytes`, smith's `run_bytes`, the forge client's rows
  and bytes, the brief budgets. A test may set its own figures; production
  defaults don't move.
- **No stored format or protocol changes:** no store record, no wire or link
  field, no settings document member. A spec that seems to need one is a
  `Question:`.
- **Every build and test through `heavy`,** with `TEMPER_BUNDLE=none` (the
  machines' cargo config sets it). `cargo fmt` builds nothing and runs
  directly.
- **Before calling a focused or fuzzy failure "outside the change",** run it
  once alone on the pristine base. If the base passes and the branch fails,
  it's the branch's.
- **Your status entry.** Each spec names one **Not yet** entry this cut
  wrote for it. Delete that entry's lines alone, nothing around them, when
  your acceptance holds. This departs from the implementer's text, which
  closes `docs/design/` to you: the entry is your task's. Write no new
  status text.
- **Decide, don't ask.** A small edit just outside the file list, of the
  kinds your spec pre-approves, is made and named in the report. A
  `Question:` is only for what truly blocks the task.
