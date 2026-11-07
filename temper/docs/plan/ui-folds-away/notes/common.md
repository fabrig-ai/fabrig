# ui-folds-away: what every task holds to

Read this note, then your spec. Where this note and the implementer's text differ, this note wins. The request is `temper/docs/plan/ui-folds-away/inputs/request.md`.

- **This plan is red-65's repair.** The three stories it repairs, `skein-browser-world`'s `browser_real`, `folds_away::*`, may run by name, alone, as a red's repair may (`AGENTS.md`, 5); the spec gives the commands and how many times. That is the one departure from "only focused and fuzzy run": no other browser, process or live story runs, nor `tools/tiers/run`, nor `scripts/browser.sh`, nor a whole tier or a tier's CPU figure.
- **The runs before the edit are the reproduction the request asks for,** not reruns: read each failure's output before the next run, and stop early only as the spec says.
- **Keep every check each story makes,** and widen no deadline: a story's waits stay as they are (`Duration::from_secs(2)`), and skein's poll and answer limits (`skein_browser::Limits`) stay as they are. A step is added only to wait for what a person would see; a step is replaced only by one that checks as much or more.
- **No new browser test.** A new case of one behaviour is a row of an existing table test (`skein/docs/foundation/testing-strategy.md`, 8).
- **No walk, render check, screenshot or throwaway driver.** The harness's own failure capture (its snapshot and screenshot) is read, not written.
- **Decide, don't ask (the user, 2026-11-06, asleep).** A small edit just outside your files (a `use` line, a doc comment, a status line your fix makes stale) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes and your branch fails, it's yours.
- **Citations:** before pushing, `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root, and fix what it lists in your files.
- **Neighbours:** `look-round-16/field-options` (not yet merged) is changing `skein/crates/skein-ui-web/src/page.rs`, `src/page/web.rs`, `src/tests/page/events.rs` and `skein/docs/design/ui-web.status.md`. This plan touches them only if the runs show a skein-ui-web fault (the spec's option D); keep those hunks small, and whichever lands second catches up.
