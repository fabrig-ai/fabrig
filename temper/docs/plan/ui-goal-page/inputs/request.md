# ui-goal-page: a goal's page, its acts and Start goal

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo look requests already made. There is no supported deployment: formats, protocols and public interfaces may change freely, with the fewest moving parts. Budgets are low priority. **This plan starts after `ui-goals` and `ui-needs` have landed:** it uses `goals::state`, `goals::ways`, `pages/report.rs` and the report note of the first; the parsed need card, its choices, its kind words and the toast of the second.

The mock is on main at `temper/docs/design/client/greenfield/` (its `README.md` says what's what). Open its pages from the checkout; nothing needs extracting.

## 1. What changes

`Address::Task` and `/tasks/N` stay. For `Shape::Tracked` alone, `pages/task.rs`'s `main` hands over to a new `pages/goal.rs`. `Shape::Agent` (`pages/agent.rs`) and `Shape::HeaderAndPlan` (a change's page, a person's) are not touched, and they are where a goal's task rows lead: the mock has no task page. On a goal the old "Task details" block, pagebar, "What was asked", "Result", "Plan" with its counts and its All/Live only filter, dependency lines, "Latest activity", "Budget and authority", the model choice and its old Stop and Cancel buttons are gone. `PlanFilter` goes from the address if nothing else draws it. `ctl task show` keeps those facts.

## 2. The engine's small part

`made: Wall` on the client's `Task` document, the record's `created_at`, through `jig-domain-core`'s document, the wire and the client's contract. Nothing else in jig.

## 3. The page

`.col.wide`, in order. Every word below is exact. `S` is `goals::state`.

**The header,** `header.goal-head`:

- `p.meta.mono`, items separated by `<span>·</span>`: `<b>TITLE</b>`, the state's word in its `state-*` class; "6 of 9 tasks" when there are tasks (true past one page, from `ui-goals`' counts); "31 of 40 spent", the budget as supplied, no `$`; "since " and `made` in the clock form.
- `<h1>`: the report's headline. With none: planning, "Planning from NAME: reading it and the designs it cites." with NAME the draft's file (section 5's `Draft:` line), or "Planning: reading what was asked." without one; stopped, "Stopped by you. What landed on the goal branch stays there."; landed, the result's first line, or "Landed."; failed, "Failed: " and the failure's reason; otherwise "N of M tasks landed."
- `p.byline`: "Written by its agent at " and the note's `changed` in the clock form. No byline with no report.
- `.acts`:
  - **"Draft"** `.btn.small`, when the goal's words name a draft and a chat (section 5): to `Address::Chat`. **"Chat T2"**, when the requester is a task: to that chat. Neither otherwise.
  - While live: **"Amend"** `.btn.small`, **"Pause"** or **"Resume"** `.btn.small`, **"Stop"** `.btn.quiet.small`, each drawn by its offer as 5.11 says: not offered, not drawn.
  - Ended: none of the three.

**Needs you bands,** one `section.band` named "Needs you" per open need of the goal, in the inbox's order, between header and report. `Inbox` from `pages/inbox.rs` runs inside the goal's page with `Filter { goal: Some(task), .. }`. Each is the mock's band: `<p><span class=tag>Needs you · KIND</span>QUESTION</p>`, KIND and QUESTION from `ui-needs`' parsed card ("a question", "a proposal", "held", "a person task", "funds"); then, as `choiceButtons(n, false)`, its choices in `.choices`, answering as Needs you does, with no Reply or Later. A card with no choices: `.btn.primary` **"Answer"** for a question or a person task, **"Open"** otherwise, to `Address::Inbox` filtered by this goal. A funds card has no "Add $10" or "Stop the goal": its acts stay `ui-needs`' Release and Leave held, or "Open".

**`.two`:**

`article.report`, heading "Report" (`.section-title`):
- each paragraph as `<p>`;
- planning: `.typing` (three `<i>`), named "Planning", under the words "Reading NAME and the designs it cites, to propose its tasks." or "Reading what was asked, to propose its tasks.", and no Left or Risk;
- `<p><strong>Left:</strong> …</p>`, `<p><strong>Risk:</strong> …</p>`, when the report has them;
- a report-less ended goal: the result's words, or a failure's reason, as paragraphs;
- `p.byline`: "By its agent at TIME". The mock's "earlier reports" is **not drawn**: its toast says what a future page would be, and one note is revised in place.

`aside.side`:
- **Tasks** (`.section-title`), the plan's direct delegates alone, from `Plan::of` (the tree's depth-1 rows and the ended page, merged by number as today; a delegate's own delegates aren't drawn; cancelled ones are left out):
  - `div.bar` `Role::Image` named "N landed of M", a `<span>` per task in order, class `landed` for an ended done, `running` for a running one, none otherwise;
  - each task not yet landed, `p.task-line` with class `running` when running: the title a link to `Address::Task`, then its state: "running"; "waits on TITLE" for one dependency; "waits on N tasks" for several; "waits on you" when its need is the reader's; "waits on its resources"; "held", in `state-needs`; "failed"; otherwise "waits";
  - `Role::Disclosure` "N landed" (`.link`, 13px, `margin-top: 6px`), folded, holding each landed task's `p.task-line`: its link, then its end in the clock form;
  - no tasks: `p.facts` "Proposing tasks from the draft…" planning, or "No tasks.";
  - past one page, "More of its tasks are not shown." as `.facts`.
- **Landing,** `p.facts`, the mock's own words: landed, "Landed at 01:12." with the clock form; stopped or failed, "Stopped before it landed." or "Failed before it landed."; live, holding a goal branch (a resource of the task named `temper/<root>/g<root>`), "Lands whole once all nine are checked." with the task count in words through nine, then a digit; "Lands whole once all its tasks are checked." with no tasks yet; live with no branch, "Each task lands on its own once checked." The mock's "The goal branch was last updated from main at 07:30, clean." is not drawn: no read says it.
- **How it runs,** `p.facts`: the way, then " · always asks you" or " · follows the away switch". The way: "Plan, tasks, land whole" with a goal branch, "Task by task" without. "follows the away switch" only if `ui-away` has landed, the requester has a switch, and the goal's words have no `Asks: always` line (section 5); `ui-away` changes this word itself if it lands later.
- **Decided by the goal,** only with a report's `Decided:` line: `p.facts`, its items as written, joined by " · ".

**`.sent`,** when the person has sent words: each of the goal's `Activity::Words` by the person in the latest activity page, `p.you`, oldest first, then the pending one "Being sent". No "Its agent:" line: the mock's reply is canned, and a goal's answer is its next report.

**The composer,** while live and Message offered, under a `max-width: 720px` box: the mock's `.composer`, as the chat's: placeholder and name "Tell TITLE something", Send disabled while empty, Enter sends, Shift+Enter a new line, grows to 220 px; `.composer-hint` "Its agent reads it at once. A change of plan becomes an amendment it shows you first." `Act::MessageWords` and `Act::Send` now route on `Shape::Tracked` too. While Message is refused as `NotWaiting`, the composer's place holds nothing. Ended, no composer.

## 4. The acts

- **Pause** sends `Ask::Stop` at once, no confirmation. A person's Stop is a placed hold over the goal's subtree. jig's Drain and a party's `Hold` are not built, and this plan doesn't build them. The toast is **"Paused: running tasks stop, nothing new starts."**: the mock's "finish" isn't true of Stop.
- **Resume**, on a goal in state paused: `Ask::Release` naming the hold's revision, at once, no confirmation. A subtree hold's root released lifts it whole. Toast "Resumed." A goal held for any other cause has no Resume: its band has the need.
- **Stop** opens the confirmation (`confirm.rs`, `Role::Dialog`): "Stop TITLE?", then "What landed on its branch stays there.", "Confirm", "Cancel". It sends `Ask::Cancel` with the reason "Stopped from its page". It replaces the `Effect::Cancel` Impact words on a goal.
- **Amend** opens today's editor (`amendment`, `Amending`, `amend::Draft`), between the header and the bands, unchanged in what it does: "Specification", "Tracked", "Priority", "Send amendment", "Discard". Restyle it in the page's look: `.field`, `.text`, `.btn`. The mock's toast is what it says isn't drawn; the editor is what exists.
- **The toast** is `ui-needs`' (one at a time, one optional act). It carries refusals too, in jig's words, with "Look again" after a moved one, as 5.11's outcomes.
- Pending, a double press joins the pending request, as 3.6 and 5.11.
- The goal page's extra reads and handles (its inbox, its cards, its report) may need the web and tools presets' `handles`, `watches` and `reads` raised (`temper-client-shell-web/src/limits.rs`, `temper-client-tools/src/limits.rs`).

## 5. Start goal

**The sheet,** `pages/start_goal.rs`, a `Role::Dialog` over the page (`dialog.sheet`, `::backdrop`), opened by an `Act::OpenStart { draft: Option<Box<[u8]>>, chat: Option<u64> }`, closed by Cancel or Escape. A domain struct holds its fields, on `Pages`, so that any page can open it. It is a form, in the mock's `startSheet` order:

- `h2` "Start a goal from FILE", FILE the draft's file name ("away-mode.md"); `.muted` 14px "The goal takes the draft as it is now. Later edits reach it through Amend." With no draft given: `h2` "Start a goal", and a `.field` "Draft", a text field, placeholder "docs/design/draft/name.md", which Start requires.
- `.field` "Name": `input.text`, the file's name without `.md`.
- `fieldset` legend "How it runs": a `ChoiceGroup` of `label.option` per way (the project's tracked templates in number order): `<b>WAY</b><small>TEXT</small>`, from `goals::ways`. The default adds " PROJECT’s default." to its text and is chosen: `ui-settings`' default way if that has landed, else the highest-numbered. After them `.link` 13px muted "The project’s ways of working", to `Settings { project }`.
- `fieldset` legend "When it needs you", **only if `ui-away` has landed:** `label.option` "Follow the away switch", small "Asks while you’re here; while you’re away, decides within the project’s limits and tells you.", chosen; `label.option` "Always ask", small "Waits for you on every question it can’t answer from the plan." "Always ask" adds the line `Asks: always` to the goal's words, and the goal's paragraph in `stock/chat.txt` gains: "If your words say Asks: always, never call decided." Without `ui-away`, no fieldset: a control with nothing behind it is worse than none.
- **Not drawn:** "Budget". Budgets are low priority.
- `.foot`: `.btn.quiet` "Cancel", `.btn.primary` "Start goal", disabled while Name or Draft is empty or a request is out.

**Start** calls `shapes::set_goal_from(me, project, number, GoalDraft { words, executor: Agent { charter: 1 } or the template's, budget: the first creating bound's budget, deadline: None, priority: None, llm: None }, reason "Started from its draft")`.

**The goal's words,** which the page reads back:

```
NAME

Build docs/design/draft/FILE.
Draft: docs/design/draft/FILE
Chat: T4
Asks: always
```

The first line is the task's title, as jig takes it. `Chat:` only when a chat opened the sheet; `Asks:` only as above. A pure function, `goal::draft_of(words) -> (Option<path>, Option<chat>)`, reads a line's start, and the Chat area's plan may use it for "started goal X".

**Its ends:** `Judged::Create`: send it, and on the answer go to the goal's page, which shows planning. `Judged::Propose`: send it, close the sheet, toast "Proposed to WHOM: it starts once accepted." `Judged::Refused` and `NotOffered`, or a refused answer: `.sheet-error` in the sheet, in jig's words, and it stays open.

**Where it opens:**
- on Goals, the `.link` "Start one from a draft" at the section head (`ui-goals` left it out), opening it with no draft: shown while the project has a tracked template. If the Chats area's Drafts filter has landed, the link goes to `Chats` under Drafts instead, as the mock's;
- the chat page's "Start goal…" (`.btn.primary.small`, `data-start-goal`) is the Chat area's plan: it needs a draft beside a chat, which isn't built here. If that plan has landed, wire its button to `Act::OpenStart` with its draft and chat; if not, make no button, and say so in the report.

## 6. The look

`temper.css`, under `#app:has(> .temper-rail)`, with `--gf-*` tokens, on new `look.rs` tokens (`temper-goal*`, `temper-sheet*`). The tokens `task-pages.css` has for a goal alone leave with it; `the_task_tokens_stay_on_their_roles` follows. Port these rules of `mock.css` exactly:

```css
.col.wide { max-width: 1000px; gap: 48px; }
.byline { font-size: 12px; color: var(--faint); }
.btn { display: inline-flex; align-items: center; gap: 8px; min-height: 40px; padding: 8px 16px; border: 1px solid var(--line-2); border-radius: 8px; background: var(--surface); color: var(--ink); font-size: 14.5px; text-decoration: none; text-align: left; }
.btn:hover { border-color: #b4b4ae; }
.btn.primary { border-color: var(--ink); background: var(--ink); color: #fff; font-weight: 500; }
.btn.primary:hover { background: #000; }
.btn.quiet { border-color: transparent; background: none; color: var(--muted); }
.btn.quiet:hover { background: var(--hover); color: var(--ink); }
.btn.small { min-height: 34px; padding: 5px 12px; font-size: 13.5px; }
.btn .rec { margin-left: auto; padding-left: 6px; font-size: 12px; font-weight: 400; opacity: .72; }
.acts { display: flex; flex-wrap: wrap; gap: 8px; }
.band { display: flex; flex-wrap: wrap; align-items: center; gap: 14px 20px; padding: 20px 24px; border-radius: 12px; background: var(--signal-pale); }
.band > p { flex: 1 1 280px; }
.band .tag { display: block; font-size: 12px; font-weight: 600; letter-spacing: .06em; text-transform: uppercase; color: var(--signal); }
.band .btn:not(.primary) { border-color: var(--signal-line); }
/* A goal. */
.goal-head { max-width: 720px; display: flex; flex-direction: column; gap: 14px; }
.goal-head .meta { display: flex; flex-wrap: wrap; gap: 4px 10px; }
.goal-head .meta b { color: var(--ink); font-weight: 500; }
.goal-head h1 { margin: 0; font-size: 30px; line-height: 1.3; font-weight: 500; letter-spacing: -.02em; transition: opacity .3s; }
.two { display: flex; flex-wrap: wrap; gap: 56px; }
.report { flex: 1 1 460px; min-width: 0; max-width: 640px; display: flex; flex-direction: column; gap: 16px; font-size: 16px; line-height: 1.75; }
.report strong { font-weight: 600; }
.side { flex: 1 1 280px; min-width: 0; display: flex; flex-direction: column; gap: 36px; }
.bar { display: flex; gap: 3px; margin-bottom: 10px; }
.bar span { flex: 1; height: 6px; border-radius: 3px; background: var(--line); }
.bar .landed { background: var(--ink); }
.bar .running { background: #a1a1aa; }
.task-line { display: flex; justify-content: space-between; gap: 12px; padding: 3px 0; font-size: 14px; }
.task-line span:last-child { flex: none; color: var(--faint); }
.task-line.running span:last-child { color: var(--good); }
.sent { display: flex; flex-direction: column; gap: 10px; max-width: 720px; }
.sent p { font-size: 14.5px; }
.sent .you { align-self: flex-end; max-width: 85%; padding: 8px 14px; border-radius: 14px 14px 4px 14px; background: var(--hover); }
.composer { display: flex; align-items: flex-end; gap: 10px; padding: 8px 8px 8px 16px; border: 1px solid var(--line-2); border-radius: 14px; background: var(--surface); }
.composer:focus-within { border-color: #a8a8a2; box-shadow: 0 0 0 3px rgba(24, 24, 27, .05); }
.composer textarea { flex: 1; min-height: 26px; max-height: 220px; margin: 6px 0; padding: 0; border: 0; outline: none; resize: none; background: none; line-height: 1.55; }
.composer textarea::placeholder { color: var(--hint); }
.composer button { flex: none; width: 36px; height: 36px; display: grid; place-items: center; border: 0; border-radius: 10px; background: var(--ink); color: #fff; }
.composer button:disabled { background: var(--line-2); cursor: default; }
.composer-hint { margin: 6px 6px 0; font-size: 12px; color: var(--faint); }
.typing { display: flex; gap: 4px; padding: 6px 0; }
.typing i { width: 6px; height: 6px; border-radius: 50%; background: var(--hint); animation: typing 1s infinite ease-in-out; }
.typing i:nth-child(2) { animation-delay: .15s; }
.typing i:nth-child(3) { animation-delay: .3s; }
/* Start a goal. */
dialog.sheet { width: min(620px, calc(100% - 32px)); max-height: calc(100vh - 32px); padding: 0; border: 0; border-radius: 14px; background: var(--surface); color: var(--ink); box-shadow: 0 24px 64px rgba(24, 24, 27, .22); }
dialog.sheet::backdrop { background: rgba(24, 24, 27, .32); }
.sheet form { display: flex; flex-direction: column; gap: 20px; padding: 24px 26px; }
.sheet h2 { margin: 0; font-size: 19px; font-weight: 500; letter-spacing: -.01em; }
.sheet fieldset { margin: 0; padding: 0; border: 0; display: flex; flex-direction: column; gap: 8px; }
.sheet legend { margin-bottom: 8px; padding: 0; font-size: 14px; font-weight: 600; }
.option { display: grid; grid-template-columns: 18px minmax(0, 1fr); gap: 2px 10px; padding: 12px 14px; border: 1px solid var(--line); border-radius: 10px; cursor: pointer; }
.option:has(input:checked) { border-color: var(--ink); background: #fafaf9; }
.option input { margin: 4px 0 0; accent-color: var(--ink); }
.option b { font-weight: 500; }
.option small { grid-column: 2; color: var(--muted); font-size: 13px; }
.field { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 14px; }
.field > label { min-width: 6.5em; font-size: 14px; font-weight: 600; }
.text { min-height: 38px; padding: 6px 12px; border: 1px solid var(--line-2); border-radius: 8px; background: var(--surface); }
.sheet .foot { display: flex; justify-content: flex-end; gap: 8px; padding-top: 16px; border-top: 1px solid var(--line); }
.btn:disabled { opacity: .45; cursor: default; }
.sheet-error { margin: -8px 0 0; color: #b42318; font-size: 14px; }
/* at 760px and under */
  .choices { flex-direction: column; align-items: stretch; }
  .choices .btn { min-height: 48px; font-size: 15.5px; }
  .choices .btn.quiet { min-height: 44px; }
  .goal-head h1 { font-size: 23px; }
  .two { gap: 32px; }
```

The `.acts` keep the `.btn.small` height at phone width, wrapping as drawn. The bands' buttons stack at 760 px (`.choices`). The side follows the report at 1000 px and under by `.two`'s wrap, not by a media rule: at 1000 wide the mock's `flex` basis decides.

## 7. Decisions already taken

- Task rows are links, the one departure: a held delegate's Release is on its own page.
- No "earlier reports"; no Add $10 or budget field; no drain; no reorder.
- "Stop" is `Ask::Cancel`, and no new word replaces it; `Ask::Stop` is "Pause". A goal branch nothing landed stays (`goal_branch.rs`), so the dialog's sentence holds.
- A goal with no report still reads whole, by the fallbacks in section 3.
- The chat's thread gets no "Started goal X" line: a person's Create leaves no turn. The Chat area may draw one from `draft_of`.

## 8. The terminal

The view is shared, and the page keeps its meaning. The regions print in order: header, needs, "Report", "Tasks", "Landing", "How it runs", "Decided by the goal", the composer. The bar prints by its name, "6 landed of 9"; the disclosure's rows print under it. `golden/task.txt` is re-recorded on a goal and says so in its commit; `each_fixture_page_prints_as_its_golden_file`, `agent_task.txt` and every change's page do not move.

## How the cut shapes it

- **One order, each `needs` the last:** the design; `made`; the page read-only, all states, with the look and its gallery samples; the acts; the composer; Start goal; the proof. `temper.css`, `temper/tools/gallery/src/tests.rs`, `names.txt` and the terminal's goldens conflict across plans: chain them.
- **Design documents first:** `client/README.md` 5.8 and 5.11 rewritten to this page, 3.5 (the `Task` row), 3.6, `browser.md` 2.5 and 7.2. Keep section numbers: the queue's citation gate rejects a broken citation (`tools/citations --all`). `client/README.md`'s tests list (section 10) follows the tests. Its section 11 loses what no longer applies.
- **Gallery samples,** beside `task-solo-T3`, `task-held`, `task-releasing`, `task-reference-T28`, which move: `goal-needs-funds`, `goal-needs-question`, `goal-running`, `goal-planning`, `goal-paused`, `goal-landed`, `goal-stopped`, `goal-failed`, `goal-no-report`, `goal-landed-open`, `goal-amending`, `goal-stop-dialog`, `goal-toast`, `goal-sending`, `start-goal`, `start-goal-draft`, `start-goal-refused`. Their words are `data.js`'s: first-run-projects, welcome-stall, forge-cards, pane-chat-line.
- **The proof, for every page and state above:** a render beside the mock at 1440, 1000 and 420 wide, plus each open state (the disclosure open, the dialog, the sheet, the amendment), screenshots in `~/src/fabrig/ui-goal-page/` on the builder, outside any worktree, the report saying which builder. Never committed. A box walk on `temper box --fake-backends`: start a goal from Goals' sheet, see planning, then its report and tasks; Pause, Resume; message it; Stop it. If the fake model's script cannot show a state, say which.
- **Tests:** `tests/task.rs` loses the goal's old page, keeps the rest; a new `tests/goal.rs`: every state's header, the fallbacks, Landing's words through nine and past, task-line states, the composer's offers, Pause and Resume sending at once, Stop's dialog; `draft_of`; the sheet's three ends. A browser tier test in the existing harness for Start goal to the goal's page.
- **Throwaway render scripts** stay untracked and are gone before the push.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). The Start goal sheet may be cut as its own task after the page.

## Render proofs

Chromium 147's new headless won't go under 500 px with `--window-size`, so a 420 px proof sets device metrics through the DevTools protocol (`Emulation.setDeviceMetricsOverride`). Any task may use a throwaway render script, untracked and gone before the push. Fonts: the mock's Google Fonts link needs the network; the client's own fonts are what's compared.

## Sources

- `temper/docs/design/client/browser.md`
- `temper/docs/design/draft/reference-code-production.md`
- `temper/docs/design/draft/template-places.md`
- `temper/docs/design/client/greenfield/mock.css`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
