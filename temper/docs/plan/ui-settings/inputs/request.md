# ui-settings: Settings' frame, How work runs and Repositories, as the greenfield mock draws them

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo the look requests already landed. There is no supported deployment: formats, protocols and public interfaces may change freely, with the fewest moving parts. This plan starts once `look-round-5` (the name "Configure temper") and `ui-changes` have landed; `needs` keeps it after them. `ui-settings-tabs` follows it.

## The mock

The mock is on main at `temper/docs/design/client/greenfield/` (its `README.md` says what's what). Open its pages from the checkout; nothing needs extracting.

Render with `chromium --headless=new --no-sandbox --disable-gpu --allow-file-access-from-files --screenshot=OUT.png --window-size=W,H --virtual-time-budget=3000 file://...`. `settings.html?project=hello` is the project with everything hidden. The mock's state is in localStorage: to see a state, use a throwaway copy of the page.

## 1. The frame of every tab

`Settings { project }` stops being the stub (`pages/home.rs::settings_controls`). Every tab is a project page in the rail's frame with "Settings" current, a `.col` as Home's (720 wide, gap 60; 32 on a phone), holding:

- **Kicker:** `.kicker`, "PROJECT · settings".
- **Heading and lede,** by tab, `.headline` level 1 and `.lede`:

| Tab | Heading | Lede |
|---|---|---|
| How work runs | How work runs in PROJECT. | Everything the other pages do for this project comes from here, so each project, and each person, can work its own way. |
| Repositories | What PROJECT works on. | The repositories chats and goals in PROJECT read and change. |
| Forges | Where your code lives. | Each forge temper is connected to, or setting up. Add one to bring its repositories to temper. |
| Accounts | What agents run on. | The LLM accounts temper’s agents run on for you. A task keeps its account: while the account is limited, the task waits for it. |
| Builders | Where agents run. | Builders are a separate app; temper only names which to use. |

- **Tabs:** a navigation named "Settings", `.tabs`, links, the current `aria-current`:
  - `.tabs { display: flex; flex-wrap: wrap; gap: 20px; border-bottom: 1px solid var(--line); font-size: 14px; }`
  - `.tabs a { padding: 8px 0; color: var(--muted); text-decoration: none; }`
  - `.tabs a[aria-current] { color: var(--ink); font-weight: 500; box-shadow: inset 0 -2px var(--ink); }`
  - In this order: "How work runs", to `Settings { project }`; "Repositories", to the new `Repositories { project }` at `/projects/<project>/settings/repositories`; "Forges", to `Forges`; "Accounts", to `Accounts`. "Builders" comes with `ui-settings-tabs`.
  - They wrap on a phone, as the mock's do.
  - "Forges" and "Accounts" carry the rail's dots, `forge_mark` and `account_mark`.

**`Forges` and `Accounts` keep their addresses and paths,** so sign-in returns, To begin's links and the tests hold. Whenever the page has a project (`rail_project`), they draw inside this frame, their old level-1 heading gone and their `pagebar` with it; today's rows stay under the tabs, restyled in `ui-settings-tabs`. With no project, as today. "Sign out" moves to the foot of the `Accounts` tab, in this plan: a `.setting` row, "Signed in as NAME", its login in `small`, and a `.btn.small` "Sign out". "Sign out did not reach the engine." moves with it.

"Settings" is current on every tab, `ConnectForge`, `ConnectModel`, `Adopt`, `Configure` and a configuring chat. Choosing a project keeps the tab (5.3); on `Forges` and `Accounts` nothing changes but the words naming the project.

## 2. How work runs

Under the tabs, `.settings`, sections 44 apart:

```
.settings { display: flex; flex-direction: column; gap: 44px; }
.setting { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 280px); align-items: center; gap: 6px 28px; padding: 14px 0; border-bottom: 1px solid var(--line); }
.setting > div > small { display: block; margin-top: 2px; font-size: 13.5px; color: var(--muted); }
.setting select, .setting .text { width: 100%; min-height: 38px; padding: 6px 10px; border: 1px solid var(--line-2); border-radius: 8px; background: var(--surface); font-size: 14px; }
.checks { display: flex; flex-direction: column; gap: 6px; font-size: 14px; }
.checks label { display: flex; align-items: baseline; gap: 10px; }
.checks input { accent-color: var(--ink); }
.checks .locked { color: var(--faint); }
.switch { appearance: none; position: relative; flex: none; width: 34px; height: 20px; margin: 0; border-radius: 10px; background: var(--line-2); cursor: pointer; transition: background .15s; }
.switch::after { content: ''; position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: #fff; transition: left .15s; }
.switch:checked { background: var(--ink); }
.switch:checked::after { left: 16px; }
.toggle-row { display: flex; justify-content: flex-end; align-items: center; gap: 10px; font-size: 14px; color: var(--muted); }
/* 760 px and under */
.setting { grid-template-columns: minmax(0, 1fr); }
```

Each section is `.section` with gap 0, its `h2.section-title` 6 px above its first row.

**First, no title, "Configure temper":** one `.setting`: "Configure temper"; `small`: "Tell it how you work, in your own words or by pointing at a document. It proposes the settings; nothing changes until you accept."; at the right, justified to its end, a `.btn.small`, primary: "Start", to `Configure { project }`; or, with a configuring chat of the person's open in the project, "Go on", to the newest such `Chat { task }`. These are the entry `configure-temper-name` asks for; To begin's step keeps its words. The configuring page stays as it is, reached from here, its path PROJECT / Settings / Configure temper.

**What the project shows,** live. A row each: the label, `small`, and at the right `.toggle-row`, the state's word then the switch:

| Label | Small | Words | Stored |
|---|---|---|---|
| Chats can write drafts | A chat keeps a document beside it, in docs/design/draft/, that a goal can start from. | On / Off | `drafts` |
| Goals | Bigger work from a draft: a plan, tasks, landing. | Shown / Hidden | `goals` |
| Changes | The landing queue, as a page. | Shown / Hidden | `changes` |
| Away switch | Lets agents decide within limits while you’re away. | Shown / Hidden | `away` |

**Ways of working,** only while Goals is shown. A row per way: its name; `small`, its line, then " Used by N." or " Used by none yet."; at the right `.toggle-row`, a radio, " The default". The ways are the project's templates with `tracked` set, in number order, named by `ui-goals`' word table and worded by `ui-goal-page`'s table of each way's text: reuse those functions, and add a line for a way they lack. Where a goal's template is not recorded, leave the "Used by" sentence out. Choosing sends the change at once, and a goal's start, once `ui-goal-page` has landed, preselects the default.

**Status,** disabled (below). Each shows what temper does at the time this lands; read the trunk, and pick the first that is true:

| Label | Small | Shows, disabled |
|---|---|---|
| Status lines and reports | Written words, beside the counts temper keeps itself. | "Written by each goal’s agent" once `ui-goals`' reports have landed; else "None: counts only" |
| Rewritten | | "When someone opens it" |
| The brief on Home | What changed since you last looked. | "Written by the status agent" once `ui-home-status` has landed, whose agent always runs; else "Off" |

**What needs you:**

| Label | Options | |
|---|---|---|
| Questions a goal can answer from its plan | "Answered by the goal, kept in its decisions" (`foreman`); "Asked of me too" (`up`); "Not asked: each agent decides alone" (`none`) | live: temper's kind 6, a project's questions (`protocol/configuration.md`, 13.2) |
| Funds | shows "Ask when a budget is used up" | disabled |

**While you’re away, agents may decide** is `ui-away`'s, its controls writing through its `SetAway`. If it has landed, move its section here, unchanged, after What needs you, shown only while the Away switch is. If not, leave a place for it there: its request takes this page.

**Telling you,** disabled: "Notifications", `small` "Only when something needs you; never for status.", showing "Off"; "Quiet hours", showing "None".

**Disabled sections** say so once, as their last line: `.facts` (13.5 px, muted), "Not built yet: this is what temper does today." Funds' `small` says the same. `.setting select:disabled { color: var(--faint); background: transparent; }`. No control does nothing: whoever builds a mechanism later turns its row on.

**Acts:** each live control sends at once, one request, its control disabled while it is out, as the switcher's forms (3.6). A refusal is said under its row in jig's words, the control back at its value; `full` too. A `stale` refusal is sent once more against the newest revision.

**The terminal:** the switches as checks, the selects as selects, disabled ones marked so; it all prints.

## 3. Repositories

- Under the tabs, a `.section`, `.section-head` holding the title "Repositories · N" and a `.btn.small` "Adopt a repository", to `Adopt { project }`.
- `.lines`: each repository of the project, from the `forges` watch's `Row.repositories` where `project` matches: `owner/name`, and in `.when` its forge's name. Its row is no link. The playground's project shows its repository from the playground's read.
- None: `.band.quiet-band`: the tag "No repository yet", then "Chats in PROJECT talk and plan, but change no code until a repository is in.", then the `.btn.small`. As the mock's chats page draws it.

```
.band { display: flex; flex-wrap: wrap; align-items: center; gap: 14px 20px; padding: 20px 24px; border-radius: 12px; background: var(--signal-pale); }
.band > p { flex: 1 1 280px; }
.band .tag { display: block; font-size: 12px; font-weight: 600; letter-spacing: .06em; text-transform: uppercase; color: var(--signal); }
.quiet-band { background: #f1f1ee; }
.quiet-band .tag { color: var(--muted); }
```

## 4. What the settings change

- **The rail:** "Changes", "Goals" and the Away switch, each only while shown, as the mock's `navItems()` and `rail()`. Kind 7 replaces the tests `ui-goals` and `ui-away` left for this, a project having a goal: both requests say so. On Goals, on a goal, and while Away is on, the entry stays shown, so that nothing goes from under the person.
- **The phone:** the tab bar is `tabbar()`'s: Home, Needs you, then Goals and Chats with Goals shown, Chats and Settings with it hidden; with Goals shown, Settings is reached by the switcher's "Settings for PROJECT". The Away chip in the top bar as the rail's switch.
- **Chats:** `ui-chats`' one function `drafts_offered(me, project)` reads `drafts`, and nothing else of chats changes.
- **A hidden page keeps its address.** Hiding removes the way in, not the page.
- **Home's link** "Turn on drafts and goals" and "See the settings", where Home's plan draws them, open this tab. `settings.html#away` and "Limits" open this tab too: `Address` has no anchors.

## 5. A proposal of settings, in its card

`ui-chats` restyles a proposal's card as `.proposal`, but a proposal of settings has no facts to show: `cards/proposal.rs` draws nothing for `ProposalAction::Settings(_)`. So the configuring chat cannot show what it proposes. Fill that branch, as the mock's `proposalCard()`:

- `.proposal-title`: "How temper will run PROJECT".
- `ul`: an `li` per line of the proposal's reason, `span`, ink; then an `li` per item, its setting in `small`, `12px` mono, faint: kind 7's as "Drafts: on", "Goals: shown", "Changes page: shown", "Away switch: shown", "Way: NAME"; kind 6's "Questions: answered by the goal"; any other, "charter 2", "template 4", "landing rule 1", as `temper ctl` names it.
- Acts: "Accept", primary, small, jig's `Decide` with running work staying; "Change something", quiet, small, focusing the composer.
- `.proposal-state`: replaced, "Replaced by the proposal below", with `.proposal.superseded`, `opacity: .5`; accepted, `.ok`, "✓ Accepted · new work follows it".
- The mock's `.proposal-split` ("temper enforces", "Agents follow") is left out: it needs each item's grouping, which is 5.13's review page.

Needs you's card is `ui-needs`'; it shows the same lines. If `ui-chats` is not in by the time this task runs, chain after it.

## 6. The engine

**Kind 7, "a project's ways":** a seventh of temper's kinds, in the table of `protocol/configuration.md`, 13.2: scope each project, number 1, decided by that project's owners, owned and judged by the forge's connector, kept in `Setting`, erased with its project, a record in temper's codec: `drafts`, `goals`, `changes`, `away`, each a bool, and `way`, a template's number, 0 for the first that starts a goal. Stock (13.4): all false, `way` 0, the mock's `firstRun.project.parts`, so a fresh project shows Home, Needs you and Chats. Judged: decodes; `way` 0 or a template the project offers. **The reference's settings,** `temper/docs/examples/reference/settings.json`, set all four true, so that a reference project shows its goals. "Seventh" stands while no other plan has taken the number: take the next free one.

**Proposable:** as an item of a proposal of settings, like every kind, so that an accepted proposal changes the rail as the mock's `acceptProposal()` does. The configuring agent's stock instructions (`configuring.txt`, in `temper-service`'s `stock` module) and the documentation built into the binary (`protocol/documentation.md`) say what kind 7 is and when to set it: goals, drafts and the rest "follow from how the person works". `temper ctl settings show` prints it.

**Direct:** the tab sends jig's `ChangeSettings` with one item, against the newest revision the person has. 5.13's "the pages never send `ChangeSettings`" gets this exception: rows of this tab that are one setting. A person who is no owner is refused, in jig's words.

**Reading it:**
- `Read::Setting` today asks for a deployment role (`temper-engine-domain/src/route/kinds.rs::read_refusal`). A read of a project-scoped kind, asked in that project's scope, is allowed to a party with a role there. Kinds 6 and 7 need it.
- The client holds jig's `settings` watch for the session, as `dismissed` and `forges` are held, Home's own watch merging into it. It reads kind 7 for the open project as it opens and whenever the `settings` document says kind 7 of that project changed, and kind 6 on the tab. Until read, or refused, the optional entries are hidden.

## 7. Decisions already taken

Settled; do not ask.

- **"Configure temper" stays a page,** reached from How work runs. It is not folded into the tab, and no docking.
- **The tab replaces `HowWorkRuns` in 5.13.** Roles, ways to start work, rules, running work and history stay in the design as later; `Proposal { proposal }` too. Rewrite 5.13 so that what is built reads first and the rest is marked later. `client/README.md` section 11's "How work runs' place and look" is closed by this.
- **Status, Funds and Telling you are drawn and disabled,** showing what temper does today: `ui-home-status`' agent always runs and has no setting, and nothing notifies. While you're away is `ui-away`'s.
- **Sign out** is on the Accounts tab; the Owner row stays plain text.
- A stored format, a protocol or a public interface may change: decide, and note it in `plan.md` so that it reaches the user.

## How the cut shapes it

- Three to five tasks, chained with `needs`: the design first (below); kind 7 and the read; the session's facts and the rail; the frame and How work runs; Repositories; the proposal's card. `temper.css`, `temper/tools/gallery/src/tests.rs` and `names.txt`, and the terminal's golden pages conflict across plans: `needs` holds them in one line.
- **Design documents updated:** `temper/docs/design/client/README.md` (grep, never read whole: 3.5's table and its *Settings*; 5.3; 5.13; section 11); `protocol/configuration.md`, 13.2 and 13.4 (grep); `protocol/store.md`; `draft/configuring.md`, 5.1; `client/browser.md`, 2.3 and 2.5; their `.status.md`.
- **Tests:** step tests of each control's request, refusal and stale; the rail and tab bar under each value; a world test that accepts a proposal of kind 7 and sees `me`'s project show it. Gallery samples: `settings`, `settings-all-hidden`, `settings-goals`, `settings-chat-open`, `settings-refused`, `repositories`, `repositories-none`, `forges-in-settings`, `accounts-in-settings`.
- **The proof:** the page against the mock's at 1440, 1000 and 420 wide: `settings.html` scrolled whole (use a window 2400 tall), `settings.html?project=hello`; open states: each switch off, the rail changing with it, the switcher open on the page. Repositories at the three widths, with rows and with none. Screenshots of both, by name, in `~/src/fabrig/ui-settings/` on the builder, outside any worktree, the report saying which builder. Never committed. Throwaway render scripts stay untracked.
- **The terminal's prints** keep working and keep their meaning: `each_fixture_page_prints_as_its_golden_file`.
- No `Look-request:` trailer: this lands no look request.

**Beside it:** `ui-goals`, `ui-goal-page`, `ui-needs`, `ui-home-status` and `ui-away` work on the rail, Home and goals. `ui-away` chains after this plan if this one is queued first. Expect catch-ups.

## Size

One cut is at most 90 minutes. Release early. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Render proofs

Chromium 147's new headless won't go under 500 px with `--window-size`, so a 420 px proof sets device metrics through the DevTools protocol (`Emulation.setDeviceMetricsOverride`). Any task may use a throwaway render script, untracked and gone before the push. Fonts: the mock's Google Fonts link needs the network; the client's own fonts are what's compared.

## Sources

- `temper/docs/design/draft/configuring.md`
- `temper/docs/design/draft/template-places.md`
- `temper/docs/design/client/browser.md`
- `temper/docs/design/client/journeys.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
