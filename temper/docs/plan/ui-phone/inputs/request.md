# ui-phone: the last pass at a phone's width

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo look requests already made, on a laptop and on a phone. This plan runs **last, after every other `ui-*` plan has landed** (`ui-welcome`, `ui-needs`, `ui-goals`, `ui-goal-page`, `ui-away`, `ui-home-status`, `ui-chats`, `ui-chat-draft`, `ui-settings`, `ui-settings-tabs`, `ui-changes`). Each of those owns its own page's phone rules. This one builds the little nobody owns (section 2) and proves the whole at a phone's width.

The mock is on main at `temper/docs/design/client/greenfield/`. Open its pages from the checkout.

Landed already and measured equal, don't touch: the top bar (padding `14px 16px 10px 20px`, the star, 17px name, menu at least 220 wide), four tabs at 12.5px and 48 tall, `.main`'s padding `8px 20px 112px`, headlines 25px, ledes 15.5px, and the welcome's whole narrow set. All 111 gallery states have no sideways scroll at 420 as of `look-round-4`.

## 1. A chat on a phone: check it landed

`ui-chats` owns this: every chat takes `look::SESSION`, and at 760 and under the rail and tab bar go, as `temper-session` does for the configuring chat. Check against the mock's `chatPage()`, and fix only a miss:

- no top bar and no tab bar; the page `100dvh`; the thread scrolls and the composer keeps the foot;
- `.session-top { padding: 10px 12px; gap: 8px }`; `.crumbs .back`, "‹" to Chats, 36 by 36, `font-size: 20px`; `.trail` hidden; the title strong on one line; `.right .byline` hidden, "Change model" in its place;
- `.thread { padding: 20px 16px }`, `.convo-foot { padding: 8px 12px 14px }`;
- with a draft (`ui-chat-draft`): Chat and Draft alone, `.seg .split` hidden, `.doc { padding: 24px 20px 60px; border-left: 0 }`;
- every other page keeps its frame, a task's and a goal's included.

## 2. The phone's own bits

- **`viewport-fit=cover`:** the mock's meta is `width=device-width, initial-scale=1, viewport-fit=cover`; the client's lacks it, so the tab bar's `env(safe-area-inset-bottom)` is always 0 and a notched phone's home bar covers it. Add it wherever the page's head is printed.
- **The closed menu on a page of no project's** (`signed-out`, `google-sign-in`, `out-of-date`): `.temper-corner`'s link state measures 36 to 44px past the right edge at 420 with the menu closed. If it shows, keep it within the page; if it's hidden, leave it and say so.
- **A toast,** if any plan landed one: `bottom: 88px`, clear of the tabs (`.toast` in `mock.css`).

## 3. The sweep

For every page the mock draws, at 390 and 420, that its phone rules landed. Any missing is fixed here, in `temper.css`, in that page's own place. Any page that doesn't exist is reported, not built.

| Mock's rule | Where to check |
|---|---|
| `.tabbar`: Goals + Chats with goals on, else Chats + Settings; a Needs count in signal colour | every framed page |
| `.phonebar`: `position: sticky`, `z-index: 20`, on scroll; `.away-chip`, `min-height: 36px`, on ink when on | Home, a long page |
| `.needs-full` hidden, `.needs-cta`, `.go` 48 tall, "Answer, one at a time" or "Answer"; `.hide-narrow` | Home with one need, with several |
| `.ask { padding: 18px }`, `.choices` stacked, `.btn` 48 tall, `.quiet` 44 | Home, Needs |
| `.flow-q { font-size: 23px }`; `.flow .choices .btn` 54 tall; `.pair` | Needs |
| `.goal-row { padding: 12px 0 }`, `.line` 15px; `.goal-head h1` 23px | Goals, a goal |
| `.two { gap: 32px }`; `.col, .col.wide { gap: 32px }`; `.intro { gap: 10px }` | Home, a goal |
| `.decision { padding: 12px 0 }`, `.why` hidden, `.late` 8em | Home, decided for you |
| `.setting` one column | Settings |
| `.choices-grid, .fields` one column; `.first-project-row` stacked, button 48 tall; `.choice, .panel, .away-note { padding: 18px }` | the welcome |

**Beyond the mock's pages,** at 360, 390 and 420, every gallery state:
- no element past the viewport's right edge unless inside a scrolling parent;
- the last control reachable above the tab bar, by `scrollIntoView` and a hit test;
- every tab and primary button at least 44 tall;
- an open switcher and an open sheet fully on screen.

Keep this sweep as a script for the plan's life, not in the tree.

## Decided

- **`phone.html`, `mock-toggle` and `mock-panel` are not built.**
- **Notifications** ("To this phone’s browser", in Settings): `ui-settings`'; no push here.
- **The tab bar's rule** is `ui-goals`' and `ui-settings`', the Away chip `ui-away`'s, the CTA and the project's count `ui-needs`', the chat's frame `ui-chats`': this plan checks them and fixes only a miss.
- **No new breakpoint:** 760, as the mock has it. Existing 600, 430 and 850 rules stay unless they fight the mock.

## How the cut shapes it

- **One order, each `needs` the last:** the design; the phone's bits (2); the checks and their fixes (1, 3). `temper.css` conflicts across plans: it comes last for that reason.
- **Design documents first:** `client/browser.md` 2.5 (the phone's frame), for what section 2 changes. Keep section numbers: the queue's citation gate rejects a broken citation.
- **Golden pages:** the terminal's don't move; a browser-only rule shouldn't reach them.
- **Commits carry no `Look-request:` trailer.**
- **The proof, for every page above:** a render beside the mock's page at 1440, 1000 and 420 wide, plus its open states (switcher, sheet, a chat with its composer focused, the tab current on each), and at 390 by 844 as `phone.html` frames it, screenshots saved on the builder in `~/src/fabrig/ui-phone/`, outside any worktree, the report saying which builder. Never committed. **Headless Chromium won't go below 500 wide by `--window-size`:** use an iframe of that width, or CDP's `Emulation.setDeviceMetricsOverride`, or the media queries lie.
- **One box walk at phone size,** on `temper box --fake-backends` through CDP at 390 by 844: the welcome, Forgejo and a model, Create, Home, "Start a chat", a message sent, back by "‹" to Chats.
- **Tests:** none new but what a fix needs; the sweep is the check. Throwaway scripts stay untracked and are gone before the push.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/client/browser.md`
- `temper/crates/temper-client-view/src/frame.rs`

## Beside it (fabrig/ui, 16:30Z)

Started before `ui-chat-draft` and `ui-away` land, so that it doesn't wait for them. Sweep what's on main. Those two plans keep their own pages' phone rules; don't touch their pages. When either lands, a catch-up brings its pages in: the proof task, last in this plan, covers whatever is on main by then, and names what wasn't there yet.

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
