# red-87-reading: rules every task keeps

- **The request** is `temper/docs/plan/red-87-reading/inputs/request.md`:
  an after-landing red, the browser story
  `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place`,
  failing in two new ways since `red-82-chat-opening` (merge `cbfe498098`)
  made a chat's opening reads stop at a screen and a half. Read it whole;
  it is short. Its *For every task of this plan* binds every task.
- **The story and its fixture don't change here**
  (`temper/tests/client-browser/**`): they are temper/tiers'. A size or a
  step the story needs is said in the report and in `plan.md`'s *Found*,
  never committed. Temporary, uncommitted instrumentation of them is
  allowed while finding the cause (the task says how).
- **One browser story runs, by name, alone:** the failing story, as a
  red's repair may (`AGENTS.md`, 5), with the command the task gives.
  Never the browser tier unfiltered, another story, another slow tier, a
  tier's CPU figure, a walk, a render check or a throwaway driver.
  Everything else runs in the focused and fuzzy suites alone.
- **Tests stay cheap:** a new case of one behaviour is a row of an
  existing table test. Budgets and timeouts are the user's, never raised.
- **Every build and test through `heavy`,** with `TEMPER_BUNDLE=none` for
  the focused and fuzzy runs (the machines' cargo config sets it; the tier
  runner builds the browser's bundle itself). `cargo fmt` builds nothing
  and runs directly.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once alone on the pristine base. If the base passes and the branch
  fails, it's the branch's.
- **Decide, don't ask.** A small edit just outside the file list is made
  and named in the report. A `Question:` is only for what truly blocks the
  task.
