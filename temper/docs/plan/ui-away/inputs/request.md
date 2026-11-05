# ui-away: the Away switch, decided for you, and overturn

At the user's request (2026-11-05, through fabrig/ui): the greenfield mock's Away is to work for real. The third of three plans for the area; it `needs` `ui-home-status`. If a plan of the settings area is in the queue, chain after it too, and take its Settings page.

## The mock

The mock is on main at `temper/docs/design/client/greenfield/` (its `README.md` says what's what). Open its pages from the checkout; nothing needs extracting.

## 1. The switch

- **A family `Away`,** the store's next free byte, as `Dismissed` (`domain/forge.md`, 5.5), keyed by party and project: `since`, `until` (a wall, or none), `kinds` (bits, default implementation, retry, wording on; dependency off), `landed` (`ProjectFacts.landed` at `since`), `on`.
- **A FORGE request `SetAway { project, on, until, kinds }`,** and the subject `away`, of the party's own. The client works out `until`: the next 08:00 in the browser's zone. Past `until` agents ask again, and the client draws off, with no write.
- **The control:** a button, `State::Pressed` when on, in the rail's foot above Settings, `.away`, 8 below it: full width, 8 10 padding, 1px `--gf-line`, radius 8, `--gf-surface`, 13.5px muted, a 30 by 18 track `i` with a 14px knob; on, ink ground, white words, knob at the right. Words: "Away", or "Away until 08:00" (`TimeForm::Clock`), "Away" alone when `until` is none. Phone: in the banner at the right, `.away-chip`: "Away: off", "Away until 08:00", min-height 36, radius 18.
- **Shown** once the project has a goal (the `Domain` fact `ui-goals` keeps for its rail), or `on`, or by the Settings part when one exists: a fresh project has no switch, as the mock's first run.
- **Toasts** (`ui-needs`'): on, "Away until 08:00: agents decide within fabrig’s limits." with the act "Limits", to Settings; off, "Away is off: questions wait for you again."
- Terminal: the same button in its `navigation`.

## 2. The agent decides

No engine auto-answer. A **host tool `decided`** (`temper-engine-smith/src/calls.rs`, as `goal`; `engine::Tool::Decided`, routed in `temper-engine-domain/src/route/tools.rs`), charter 1's:

`{ question, options: [2 to 6 words], took: index, why, kind: "implementation" | "retry" | "wording" | "dependency" | "other" }`

The root finds the caller's requester party (`domain.core.tasks.delegation`); the forge judges in the decision: that party's `Away` row for the project is on, before `until`, and `kind` is among its `kinds` (never `other`). Then a **`Decided` row** is saved, and the tool answers "Decided: carry on with TOOK." Otherwise it answers "Ask: your person is here or this kind waits for them. Ask with message, as a question." Calls are idempotent by the call's key, as `change`'s are.

`stock/chat.txt`, the goal's paragraph: "Before you ask your requester anything, call decided with the question, its options, the one you'd take, why, and its kind; on Decided carry on with it and ask no one; on Ask, ask them with message, as a question. Funds, sign-ins, machines, a plan's go-ahead and anything hard to reverse are kind other. If they tell you a decision was overturned, ask it with message, as a question, and wait." Update the host-part figures.

Only goals call it. A question skipping the tool waits as ever. Held tasks and proposals always wait, away or not.

**`Decided`,** the next byte, keyed by project and number: `task`, `at`, `question`, `options`, `took`, `why`, `overturned`. At most 64 a project, oldest dropped, each text bounded. Subject `decided`, per project. Erased with the project.

## 3. Home, while any decision shows

A row shows for 24 hours from `at`. With any showing, Home takes the mock's away form, in `pages/home.rs`'s one intro:

- **Kicker:** "While you were away · 23:10 to now", or "· 23:10 to 08:03" once off; `since` and the time it went off.
- **Headline:** "57 changes landed." (`landed` since `Away.landed`, none for 0) then "Four things were decided for you." or "One thing was decided for you." in place of the needs sentence. Lede and byline as `ui-home-status` gives them.
- **"Decided for you · 4",** a region, first after the intro: `section-head` with "Limits" at the right, to Settings. `.decided`: a top rule, a `.decision` per row, oldest first: mono "TITLE · 01:05", `.muted` the question, `.took` 500, `.why` 14px muted; at the right:
  - live task, not overturned: "Overturn" (`.btn.small`);
  - overturned: `.late` "Overturned: it asks you again", took struck through, `--gf-faint`;
  - task ended: `.late` "landed 03:12 · a new goal can undo it" (its ended time, "ended" for one that didn't end done).
  Task and its state: the client reads each row's task (`Read::Task`), as the inbox reads cards; at most 7 rows shown, newest kept.
- **Needs' head:** "Still needs you · 2" with any row showing.

Empty rows, the switch off and old: Home is as `ui-home-status` left it. The mock draws this on Home alone: nothing in Needs you.

## 4. Overturn

"Overturn" sends a FORGE request `Overturn { project, number }`, which marks the row (refused if overturned already), then, on its answer, the client's existing message to the task, in words: "I overturn your decision. QUESTION You took TOOK. Ask me again with message, as a question, and wait for my answer." Toast: "Overturned: its goal asks you again, under Still needs you." The goal's question comes as any other. A refused message (the task ended between) leaves the row marked and a toast "It has ended; a new goal can undo it."

## 5. Settings

Under Settings, after the existing list unless the settings area's page has landed, a section `#away`, `section-title` "While you’re away, agents may decide": `.setting` "Kinds of decision", small "Each shows afterwards as decided for you, with Overturn.", `.checks`: "Implementation choices within a task", "Retries and splitting a failing task", "A design document’s wording", "Adding a dependency", and locked and unticked "Budgets, sign-ins, machines, what is hard to reverse: always wait"; a select "Away until", small "When it isn’t turned off by hand.", options "08:00, the end of quiet hours", "I turn it off". Each writes through `SetAway`. It works while off: the row keeps them, `on` false. "Limits" lands here.

## Decisions already taken

- Agents decide, through `decided`, not the engine. The tool's answer is live, so a running goal needs no brief or wake.
- Overturn while the task lives; no time limit beyond that (the draft's open question).
- Decided rows last 24 hours on Home.
- A "while away" retry of a failing task, the mock's `d-split`, needs a goal to call `decided` before splitting; the tool's `retry` kind covers it, nothing more is built.
- A goal's page: once this lands, its "How it runs" (`ui-goal-page`) says "follows the away switch" where its requester has a switch; change that one word here. The start sheet's choice and "Decided by the goal" stay the goals area's.
- Notifications and quiet hours aren't here.
- A change to a stored format, a protocol or a public interface needs no `Question:` in this plan: decide, and note it in `plan.md`, so that it reaches the user. Family bytes may move if another plan lands first: catch up.

## How the cut shapes it

Tasks, chained by `needs`:

1. Design: a section "Away" in `client/README.md`, 3.5, `domain/forge.md`, `protocol/store.md`, `domain/agent.md` (the tool, 4.3), `protocol/configuration.md` 13.4.
2. `Away`, `SetAway`, the subject; the control, its toasts.
3. `decided`, `Decided`, the charter's words and figures.
4. Home's away form and the section.
5. Overturn.
6. Settings' section.

**Gallery states:** `home-away`, `home-away-overturned`, `home-away-one`, `rail-away-on`, `settings-away`.

**The proof:** `home.html` in the away state, against the mock's, at 1440, 1000 and 420 wide, full page; the switch on and off at each width; toasts open; screenshots saved on the builder under `~/src/fabrig/ui-away/`, outside any worktree, the report saying which builder. A walk with `temper-fake`: Away on; a scripted goal calls `decided` and carries on; Home shows it; Overturn; the goal asks; answer it on Home. And Away off: `decided` says Ask, and the question waits.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Render proofs

Chromium 147's new headless won't go under 500 px with `--window-size`, so a 420 px proof sets device metrics through the DevTools protocol (`Emulation.setDeviceMetricsOverride`). Any task may use a throwaway render script, untracked and gone before the push. Fonts: the mock's Google Fonts link needs the network; the client's own fonts are what's compared.

## Sources

- temper/docs/design/draft/to-begin.md
- temper/crates/temper-client-view/src/pages/home.rs
- temper/crates/temper-client-view/src/frame.rs
- temper/crates/temper-client-domain/src/pages/home.rs
- temper/crates/temper-client-domain/src/forge/dismissed.rs
- temper/crates/temper-engine-domain-forge/src/dismissed.rs
- temper/crates/temper-engine-domain/src/route/tools.rs
- temper/crates/temper-service/src/stock/chat.txt

## Rulings (fabrig/ui, 2026-11-05)

- This plan builds "When it needs you" on the one Start goal sheet (ui-goal-page's start-sheet), with `Asks: always` and its charter sentence, and changes the goal page's "How it runs" word to "follows the away switch". See `coord/ui/report/rulings.md`'s words: Goals show in the rail when the setting is on or the project has a goal.

## Beside it

`ui-settings-tabs` is on main, carrying most of `ui-settings`; `ui-home-status` lands within the hour, and this plan builds on its Home: take it by catch-up before any Home task. `ui-goal-page` adds the Start goal sheet this plan extends.

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
