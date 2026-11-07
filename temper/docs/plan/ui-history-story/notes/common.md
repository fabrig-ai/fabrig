# ui-history-story: what every task holds to

- **This plan is red-63's repair** (`temper/docs/plan/ui-history-story/inputs/request.md`).
  The one test it repairs, `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back`,
  may run by name, alone, as a red's repair may (`AGENTS.md`, 5); its task gives the
  command and how many times. No other browser, process or live story runs, nor
  `tools/tiers/run`, nor a whole tier: everything else runs in the focused and fuzzy
  profiles alone.
- **No product change if the client is right.** No new browser test, and no deadline
  or bound widened: a story's waits stay `WITHIN` and `ABSENT`, skein's poll and
  answer limits stay as they are.
- **Keep every check the story makes.** A step is replaced only by one that checks
  as much or more.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table
  test, not a new test (`skein/docs/foundation/testing-strategy.md`, 8).
- **No walk, render check, screenshot or throwaway driver.** A plan is complete when
  it lands.
- **Decide, don't ask.** A small edit just outside a task's files (a re-export, a
  `mod` line, a doc comment) is made and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on
  the pristine base. If the base passes and your branch fails, it's yours.
- **Citations:** before pushing, `python3 -I -B tools/citations/citations.py --base origin/main`
  from the repository root, and fix what it lists in your files.
