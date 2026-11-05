# ui-goals: the Goals page, the rail's Goals, and a goal's report

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo look requests already made. There is no supported deployment: formats, protocols and public interfaces may change freely, with the fewest moving parts. Budgets are low priority. This plan starts after `look-round-4`. Plan `ui-goal-page` follows it and rebuilds a goal's own page; this plan draws no goal page.

The mock is on main at `temper/docs/design/client/greenfield/` (its `README.md` says what's what). Open its pages from the checkout; nothing needs extracting.

## 1. What exists

- `client/README.md` 3.5 lists `Board { project }`, but `Address` has no such variant and the rail no entry. `PublicBoard` (signed out) is another thing: leave it.
- A goal is a tracked agent task (`Shape::Tracked` in `temper-client-domain/src/pages/task.rs`). Its address stays `/tasks/N`.
- `Chats` (`pages/chats.rs`, domain and view) is the pattern: a live watch, one ended read, a filter in the address, a "more" read. `Inbox` (`pages/inbox.rs`) is the pattern for a read per row.

## 2. A goal's report (the engine's part, first)

The mock's goals carry agent-written words. They are a **note**, and need no new record, read or wire field.

- **The shape:** one note per goal, scope goal (`NoteScope::Task`; the tool's scope `goal`). Its description is `report: ` then the headline, one line. Its body is the report's paragraphs, separated by blank lines, then, each on its own line and each optional: `Left: …`, `Risk: …`, `Decided: first, 05:40 · second, 05:12`. Revised in place: the agent names the note and the revision from its brief's index, so one note stands. Limits: 256 bytes of description, 8 KB of body (standard profile).
- **The stock:** note scope goal on in the stock authority (`temper-service/src/stock.rs`, today `notes: Scopes(0)`), with its floors and tests. The reference's settings (`temper/docs/examples/reference/settings.json`) likewise if not already.
- **The charters:** a paragraph in the goal's part of `temper-service/src/stock/chat.txt` and in the reference's charter 5: after planning, and whenever what is true of the goal changes (a task landed, failed or split, a decision taken, a risk found): write the report note. Its headline says where the goal stands, in a sentence a person reads first. Its paragraphs say what happened and why, in the first person. `Left:` what remains; `Risk:` what could go wrong; `Decided:` what it decided itself, each with its time. Also: a message that changes the plan becomes an amendment the goal shows the person first. Model scripts and goldens of the stock's charters move with it. The scripted fake model writes a report in its goal script, so a box walk shows one.
- **Reading:** a pure helper in the client domain, `pages/report.rs`: `report::parse(&c::Note) -> Report { headline, paragraphs, left, risk, decided, at }`, and the read `Read::Notes { scope: NoteScope::Task { project, task }, description: Some("report: "), after: None }`, the newest by `changed` when several answer. Notes have no watch: read again when the goal's `last_activity` moves. No note yet is a normal state, never an error. Tests: every line optional, a body cut, words with "Left:" inside a paragraph (only a line's start counts).

## 3. The Goals page

**Address:** `Address::Goals { project: u32, filter: GoalsFilter }`, `GoalsFilter` being `All`, `Needs`, `Live`, `Ended`; path `/projects/<project>/goals`, with `?state=needs`, `?state=live`, `?state=ended`, parsed and printed as Chats' filter. Choosing one is a move. Added to every match in `root.rs`, `frame.rs` and `pages.rs`. It opens in the project's frame.

**Reads** (`pages/goals.rs` in the domain):
- the query: `TaskQuery { project, label: Some(the template's label), tracked: Some(true), order: Priority, .. }`, the label being template `shapes::GOAL`'s; a live watch of `Subject::Tasks`, and a read of the ended group by `QueryOrder::Ended`, as Chats. The project index serves it, since goals are `listed`.
- "More goals", a quiet button after the rows while the ended page has more, as "more closed chats".
- one read per row, one at a time, as the inbox's cards: its report (section 2), and its tasks, `Read::Tasks` by `parent` the goal.
- needs: `Read::Inbox { group: Needs, project: Some(project), goal: None, .. }`, again as the watch or `me.needs_total` moves. `InboxPage.selection` gives each row its goals.
- the web and tools presets' `handles`, `watches` and `reads` (`temper-client-shell-web/src/limits.rs`, `temper-client-tools/src/limits.rs`) grow if the page needs them.

**The page** (mock `goalsPage`), `.col`, in order:

1. `.intro`: `.kicker` "PROJECT · goals" ("fabrig · goals"), lower case as drawn; `<h1 class=headline>`: "5 goals, 3 under way." where the first number is every row and the second those live and those needing the person; **"1 goal, 1 under way."** in the singular, which the mock gets wrong; "No goals yet." when there are none; "More than N goals, M under way." while rows remain unread. `.lede`: "A goal turns a draft into landed work, its own way: W." with W the project's ways in order, lower case, joined by ", " (the mock: "plan, tasks, land whole, task by task, plan, then wait for me, one agent, one branch").
2. `.section` > `.section-head`: `.filters` with buttons "All", "Needs you", "Under way", "Ended", the chosen one `aria-pressed=true`; at the right `.link` "Start one from a draft". **This plan doesn't draw that link:** `ui-goal-page` adds it with the sheet.
3. `.goal-rows`, or `<p class=empty>None.</p>`. Under the filters: Needs: state needs. Under way: running, planning, paused. Ended: landed, stopped, failed.

**A row** (mock `goalRow`), the whole row one link to `Address::Task`, public as `goals::row` since Home's area will reuse it:

```
<a class="goal-row[ ended]">
  <span class="top mono"><span>TITLE</span><span>6 / 9 · <span class="state-needs">needs funds</span></span></span>
  <span class="line">HEADLINE</span>
</a>
```

`TITLE` is the chip's title, its spec's first line. `6 / 9`: done, then total, of the goal's direct delegates, not counting cancelled ones; left out, with its " · ", while there are none; the count is true past one page: a goal of a hundred tasks is this project's everyday. Add the two counts to the goal's document or the tree's read, whichever is the smaller change (formats may change, no compatibility); only if that proves costly, keep `6+ / 9+` and say so in `plan.md`. `TaskLine.phases` will not do: it counts the live subtree.

**A goal's state** is a pure function, `goals::state(...)`, public for the goal page, first match wins:

| State | When | Word | Class |
|---|---|---|---|
| needs | a needs row's selection names the goal | "needs funds" when the goal's own chip is held for `HoldCategory::Exceeded`, else "needs you" | `state-needs` |
| paused | live, held by a placed hold of mode Stop over its own subtree | "paused by you" | `state-paused` |
| planning | live, no report, no delegates | "planning" | `state-planning` |
| running | any other live | "running" | `state-running` |
| landed | ended, done | "landed " and the end in the clock form ("landed 01:12", "landed Mon 18:40") | `state-landed` |
| stopped | ended, cancelled | "stopped" | `state-stopped` |
| failed | ended, failed | "failed" | `state-stopped` |

`HEADLINE` is the report's headline. With no report: "Planning from the draft." planning; "N of M tasks landed." running or paused; "Landed." landed; "Stopped by you." stopped; "Failed." failed. An ended row carries `.ended`.

**Ways** are one public function, `goals::ways(me, project)`, in the view: the project's templates with `tracked` set, in number order, each with a name and a line. Template named `goal`: "Task by task", "The same plan; each task lands on its own once checked." Template named `goal for a draft`: "Plan, tasks, land whole", "A plan from the draft, its tasks on a goal branch, landed as one change once all are checked." Any other: its own name, no line. `ui-settings`' "Ways of working" and `ui-goal-page`'s sheet read this.

**The rail** (`frame.rs`, `entries`): "Goals" between "Needs you" and "Chats", to `Address::Goals { project, filter: All }`, current on Goals, on a goal (`Shape::Tracked`) and on a task whose document names a goal. Shown only when the project has a goal or the page is Goals, so that a fresh project keeps today's rail as the mock's Default project does. `Domain` keeps that fact, public as `Domain::has_goals(project)`, from a live watch and one ended read of the query above, with window 1, per open project: `ui-home-status` and `ui-away` read it too, so it stays whatever the rail does. If `ui-settings` has landed, the rail's test is its kind 7's `goals`, as its request says, and the fact stays. **The tab bar:** at most four, as the mock's `tabbar`: Home, Needs you, Goals, Chats with Goals shown; Home, Needs you, Chats, Settings without.

## 4. The look

`temper.css`, scoped under `#app:has(> .temper-rail)` with the `--gf-*` tokens, as Home's rules are, on new tokens in `look.rs` (`temper-goals*`). Port these rules of `mock.css` exactly, values and all:

```css
.col { max-width: 720px; margin: 0 auto; display: flex; flex-direction: column; gap: 60px; }
.kicker { font-size: 13px; color: var(--muted); }
.headline { margin: 0; font-size: 32px; line-height: 1.25; font-weight: 500; letter-spacing: -.02em; }
.lede { font-size: 17px; line-height: 1.65; color: var(--ink-2); }
.byline { font-size: 12px; color: var(--faint); }
.intro { display: flex; flex-direction: column; gap: 16px; }
.section { display: flex; flex-direction: column; gap: 14px; }
.section-head { display: flex; align-items: baseline; justify-content: space-between; gap: 12px; }
.section-title { margin: 0; font-size: 13px; font-weight: 600; letter-spacing: .06em; text-transform: uppercase; color: var(--muted); }
.section-title.signal { color: var(--signal); }
.section-head a { font-size: 13px; }
.empty { color: var(--muted); padding: 4px 0; }
.facts { font-size: 13.5px; color: var(--muted); }
.goal-rows { display: flex; flex-direction: column; }
.goal-row { display: flex; flex-direction: column; gap: 4px; padding: 18px 0; border-bottom: 1px solid var(--line); text-decoration: none; }
.goal-row .top { display: flex; justify-content: space-between; gap: 12px; }
.goal-row .line { font-size: 16px; }
.goal-row:hover .line { text-decoration: underline; text-decoration-color: var(--line-2); text-underline-offset: 3px; }
.goal-row.ended .line { color: var(--muted); }
.state-needs { color: var(--signal); }
.state-running, .state-planning { color: var(--good); }
.state-landed, .state-stopped, .state-paused { color: var(--faint); }
.filters { display: inline-flex; gap: 2px; padding: 3px; border-radius: 9px; background: var(--hover); }
.filters button { padding: 5px 12px; border: 0; border-radius: 7px; background: none; font-size: 13px; color: var(--muted); }
.filters button[aria-pressed=true] { background: var(--surface); color: var(--ink); font-weight: 500; box-shadow: 0 1px 2px rgba(0, 0, 0, .08); }
/* at 760px and under */
  .main { padding: 8px 20px 112px; }
  .col, .col.wide { gap: 32px; }
  .intro { gap: 10px; }
  .headline { font-size: 25px; line-height: 1.3; letter-spacing: -.015em; }
  .lede { font-size: 15.5px; }
  .goal-row { padding: 12px 0; }
  .goal-row .line { font-size: 15px; }
```

`.main`'s padding is Home's. Rows' `.mono` is `font: 12px var(--gf-mono)`, colour faint. The mono face doesn't reach these pages yet: `#app:has(> .temper-rail)` sets `--temper-mono` to the sans, and `files/fonts` holds IBM Plex Sans alone. Add `--gf-mono: 'IBM Plex Mono', 'JetBrains Mono', ui-monospace, monospace`, and bundle IBM Plex Mono 400 and 500 as the sans is, unless another plan already has.

## 5. The terminal

The view is shared. Goals prints as a page of its own, heading, filters as numbered buttons, a row a numbered link with its state and headline. A golden `goals.txt` beside `golden/task.txt`. The rail's entry prints in the frame only where it shows, so that no other golden moves but those with goals.

## 6. Decisions already taken

- `ui-home-status` and `ui-settings` build on `goals::row`, `Domain::has_goals` and `goals::ways`: keep those names.

- `Board` and its "Set a goal" and "reorder goals" are dropped from `client/README.md` 3.5; `Goals` replaces the row. Amend's Priority still orders goals.
- The Home page's "Goals" section is the Home area's plan.
- Parts (a project's switch for goals) are `ui-settings`' kind 7. Don't build one here.
- The filter lives in the address, not in stored state.
- Don't draw spend on this page.

## How the cut shapes it

- **One order, each `needs` the last:** the design; the report (section 2); the Goals page and its look, with its gallery samples; the rail and tab bar; the proof. `temper.css`, `temper/tools/gallery/src/tests.rs`, `names.txt` and the terminal's goldens conflict across plans: chain them.
- **Design documents first:** `client/README.md` 3.5 (the table, a *Goals* paragraph), 5.3 (rail, tab bar), `browser.md` 2.3 (the path) and 2.5, `domain/agent.md` (the report note), `protocol/configuration.md` 13 (the stock's note scope; the charters). Keep section numbers: the queue's citation gate rejects a broken citation (`tools/citations --all`).
- **Gallery samples:** `goals-solo` (rows in each state), `goals-reference`, `goals-none`, `goals-needs`, `goals-under-way`, `goals-ended`, `goals-filter-empty`, `goals-more`, and a rail with Goals. Solo and Reference worlds gain goals, notes and an inbox selection; the worlds' `data.js` is the model for words.
- **The proof, for every page and state above:** a render beside the mock's `goals.html` at 1440, 1000 and 420 wide, and its open states (each filter, the switcher open), screenshots in `~/src/fabrig/ui-goals/` on the builder, outside any worktree, the report saying which builder. Never committed. The hello project's empty list against the mock's with the project switched. A box walk on `temper box --fake-backends`: a chat's goal appears in the rail and the list, with its report's headline.
- **Tests:** `temper-client-domain` address round trips; `goals::state` for every row of its table; `report::parse`; view tests beside `tests/chats.rs`; `tests/rail.rs`.
- **Throwaway render scripts** stay untracked and are gone before the push.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

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
