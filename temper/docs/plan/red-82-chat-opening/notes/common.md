# red-82-chat-opening: rules every task keeps

- **The request** is `temper/docs/plan/red-82-chat-opening/inputs/request.md`:
  an after-landing red, the browser story
  `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place`,
  failing since `1e6e2388ee` made a chat's opening read its whole window.
  Read it for the why; your spec and the design section it names are what
  you build.
- **The browser story and its fixture don't change**
  (`temper/tests/client-browser/**`). The story is the oracle: it runs
  after landing, never in this plan.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or
  live tier story, a tier's CPU figure, a walk, a render check or a
  throwaway driver.
- **Tests stay cheap:** a new case of one behaviour is a row of an
  existing table test, or of the table your spec makes from one test.
  Budgets and timeouts are the user's, never raised.
- **Every build and test through `heavy`,** with `TEMPER_BUNDLE=none` (the
  machines' cargo config sets it). `cargo fmt` builds nothing and runs
  directly.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once alone on the pristine base. If the base passes and the branch
  fails, it's the branch's.
- **Decide, don't ask.** A small edit just outside the file list is made
  and named in the report. A `Question:` is only for what truly blocks the
  task.
