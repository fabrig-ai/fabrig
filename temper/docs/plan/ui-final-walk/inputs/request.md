# ui-final-walk: the whole client, walked on a live box, against the mock

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work like the greenfield mock (`temper/docs/design/client/greenfield/`, its `README.md`), with the changes look requests made since. Every plan so far proved its own pages, mostly as gallery states. Nobody has walked the whole thing as a person, and when `ui-chat-draft` finally did, it found two real bugs. This plan is that walk. Decisions already taken are in `/home/free/src/fabrig/coord/ui/report/decisions.md` and `rulings.md` on hel8: the cut reads them and carries what tasks need into `notes/`. A departure listed there is not a finding.

## The walk

A fresh `temper box --fake-backends`, headless Chromium through CDP, a person's path with no shortcuts, at 1440 wide and again at 420 (`Emulation.setDeviceMetricsOverride`). Handle `Page.javascriptDialogOpening`.

1. **First run:** the welcome, Forge (Forgejo on this machine, empty `work/hello`), Model (each of ChatGPT and Claude offered; sign in with ChatGPT), Ready, Home with To begin.
2. **Everyday:** Settings, How work runs: turn on drafts, Goals, Changes and Away. The rail changes. Configure temper opens and reads as its look request says.
3. **A chat that changes the code:** new chat, tool lines, a change, Changes' queue and landed list, Home's brief.
4. **A draft to a goal:** + Draft, two document turns, Split, Draft, Start goal…, the sheet, the goal's page (report, tasks, Landing, How it runs), Pause, Resume, Stop, Draft back to the chat. The rail's menu.
5. **Needs you:** a question with choices, Later, Reply, Undo, one held task, a proposal; the rail's count, Home's card.
6. **Away:** turn it on, a decision the goal takes, "decided for you" on Home, overturn.
7. **Settings' other tabs:** Repositories, Forges, Accounts, Builders; adding a project; the project switcher; Sign out, reload, back in.

Where the fake model can't reach a state, say so and show it some other way (gallery, ctl) rather than skipping it silently.

## Two real checks

- **The real Claude Code fetch:** on a machine with no `claude` on `PATH`, a real, non-fake fresh `temper box`: the Model step offers Claude; `temper program-login`'s fetch really downloads the pinned 2.1.311 and it runs as far as asking to sign in. Don't sign in, and use no credentials. It has never been run.
- **A real browser's Sign out** on a non-fake `temper serve`.

## The result

- `report.md`: a table, page by page and state by state, at both widths: matches, differs (with why: a decision named, or a finding), or unreachable.
- **Findings:** fix what's small, each with a test red first, in this plan's tasks: a look rule, a word, a missing link, a stale read. List what's large as its own line, with its reproduction, for fabrig/ui to plan. Don't redesign, and don't reverse a listed decision.
- **Screenshots:** mock beside client for each state, under `~/src/fabrig/ui-final-walk/` on the builder, outside any worktree; the report names the builder. A contact page, `index.html`, links them.

## How the cut shapes it

Three to five tasks: the walk first, as one task producing findings; then small fixes chained; then the walk again on the fixed tip. Decide, don't ask. Nothing here changes a stored format unless a finding needs it.

## Size

One cut is at most 30 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/client/journeys.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
