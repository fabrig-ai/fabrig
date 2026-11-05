# ui-chats: the Chats page and every chat as the greenfield mock draws them

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo what look requests have changed. There is no supported deployment: formats, protocols and public interfaces may change freely. This plan touches none of them: it is view, css and the client's domain. The draft beside a chat is the next plan, `ui-chat-draft`: leave its places empty.

It starts after `look-round-4` has landed: the rail and tab bar, `temper-session`, `Configure { project }`.

## The mock

On branch `origin/ux-ideas/greenfield`, in `temper/docs/design/client/greenfield/`: `app.js` draws every page, `mock.css` styles it, `data.js` holds its samples. Fetch the branch and render it with `chromium --headless=new --no-sandbox --disable-gpu --allow-file-access-from-files --screenshot=OUT.png --window-size=W,H --virtual-time-budget=3000 file://...`; its state is in localStorage, so a state needs a throwaway copy of the page that sets `temper-greenfield-v2` first (`{"project":"hello"}` opens Hello's chats). Functions: `chatsPage`, `chatPage`, `composer`, `sendChat`, `render`'s thread scrolling. Pages: `chats.html`, `chat.html?id=T4` (its draft is the next plan's), `chat.html?id=T3` in Hello, `chat.html?id=T5` (closed), `chat.html?id=new`.

**The look's rule:** the selector named is the rule to copy, pixel for pixel, onto the client's own classes, in `temper.css`, using the `--gf-*` tokens (add `--gf-hint: #8a8a93`, `--gf-signal-ink: #9a3412`, `--gf-signal-line: #f1d5bd` where missing). The page's look is the mock's; the words below are exact. The mock's phone rules are in its last `@media (max-width: 760px)` block: copy those that concern these selectors.

**The terminal:** the view is shared. The terminal's prints keep working and keep their meaning: what the mock lacks but a print needs stays in the view, hidden by css in the browser, unless this request says to drop it.

## 1. The Chats page (`Address::Chats { filter }`, `/chats`)

The page's `start` field goes (section 2). It is a page of the frame, not a pane: drop its `Role::Complementary`, the `temper-sidebar` rules that only it uses, the "New chat" bar (`start`), the composer, the first-chat offer (To begin and Settings keep theirs), and the playground's parts (section 2).

A `Role::Main` with a new token `CHATS_PAGE`, drawn as `.main` > `.col`:

- **Head,** `.intro`: `.kicker` "PROJECT · chats"; level-1 `.headline` "Talk to an agent about OWNER/NAME." with the project's home repository, read as Home reads it (`pages/home.rs`' `repository`), or "Talk to an agent." with none; `.lede` "A chat answers a question or changes the code." (the next plan adds to it).
- **No repository,** the existing `repository_context` condition: `.band.quiet-band` holding `.tag` "No repository yet", "Chats in PROJECT talk and plan, but change no code until a repository is in." and a `.btn.small` link "Adopt a repository" to `Adopt { project }`.
- **The list,** `.section`: `.section-head` with `.filters` left (All, Live, Closed: `aria-pressed` buttons as today; the filter stays in the query) and `.btn.small` "+ New chat" right, a link to `NewChat`.
- **Rows,** `.goal-rows` > `.goal-row`, keeping the list's merge order and the row's stretched link: one link on the number, its `::after` covering the row, so that the row is the link.
  - Top line, `.top.mono`, a flex with `justify-content: space-between`. Left: "T9", then, for a configuring chat, " · Configuring". Right: the status, " · ", the age.
  - **Status** in lower case by css (`text-transform: lowercase`; the terminal keeps its words), `.state-needs` when the status token is `STATUS_YOURS` or `STATUS_WAITING`.
  - **Age:** Home's short form, as `pages/home.rs` says it in `.temper-home-when`.
  - Under it `.line`, the title; `.goal-row.ended .line` for an ended chat.
  - Project name: drop it from the row: the page is one project's.
  - Spend, delegates and "Closed. Its transcript is kept.": kept in the view, hidden in the browser.
- **Empty:** `.empty`, "None.", under every filter.
- **"More closed chats"** stays, a `.btn.quiet.small` under the rows.

Phone: as the mock: `.headline` 25 px, `.goal-row { padding: 12px 0 }`, `.goal-row .line { font-size: 15px }`; the frame's top bar and tab bar stay.

## 2. A new chat (`Address::NewChat`, `/chats/new`)

A new address (`browser.md`, 2.3) for the open project. `?at=start` goes, with `Chats`' `start` and every use of it: the rail's "+ New chat", the Inbox's, Home's "New chat", `pages.rs`, `route.rs`, `domain.rs`, `root.rs`. Each now goes to `NewChat`. The domain's `Chats` page keeps its fields for words and the start; `NewChat` opens that same state with the page's watches, so `Act::StartChat`, `Act::ChatWords`, `Act::Suggest` and `ready_to_start` keep working unchanged.

Draw it as `chatPage` draws `id=new`, on the layout of section 3:

- Path "PROJECT / Chats /" and the level-1 "New chat". `‹` on a phone goes to `Chats`.
- Byline: `choice::new_chat_model`'s line, as section 3's byline.
- In the thread: centred `.meta-line`: "Say what you want; the agent works in OWNER/NAME.", or with no repository "Say what you want. PROJECT has no repository yet, so the agent can talk and plan but not change code."
- Composer: placeholder and name "What do you want done?", focused as the page opens; `.composer-hint` "Enter sends · Shift+Enter for a new line". No "Start" button: the send arrow is the act, grey while empty or not ready.
- Sent: as today; while out, the arrow is disabled and the words stay; made, `Chat { task }` opens in place of this address; refused, jig's words under the composer, kept.
- **The playground** (`client/README.md`, 4.8, 5.6): its start, status, failure and "Try again" and its context line draw here, above the `.meta-line`'s place; its suggestions as `.starters` buttons under it. Everything that opened the Chats page for the playground opens `NewChat`. The playground's banner stays in the frame.
- With no usable model, the "No model yet · Connect a model" line sits above the composer.

## 3. One layout for every chat

Every chat and `NewChat` take `look::SESSION`, the layout `configure.rs` draws (`.session-app`). Retire `look::CHAT`, `CHATBAR`, and for chats `ENTRY`, `COMPOSER_AREA`, with the `temper.css` rules only they use. **The rail stays on every chat:** the user's choice in `configure-chat-in-settings`; the mock's rail-less chat is superseded. On a phone (760 px or less) rail and tab bar go, as `temper-session` already does.

`configure.rs`' `top` becomes the shared bar. A chat differs from a configuring chat only in: its path ("PROJECT / Chats /", `‹` to `Chats`; a configuring chat's still through `Settings`, its rail current on Settings) and the chooser (a configuring chat has none).

**The bar,** `.session-top`, its right part `.session-top .right`, in this order:

1. **Byline,** `.byline`, nowrap: "T4 · ", then the composer's line (5.5) as its words: account, " · ", model. It is the summary of the "Change model" disclosure, moved here from under the composer: the same pixels as the mock's until pressed, its hover the ink colour, its body a panel under the bar, 320 px wide, shadow as `.temper-projects-menu`'s, closing on a press outside or Escape. `choice::composer` keeps its feedback: a pending request, its outcome, a task's wait for its account and a refused next run go above the composer, not in the panel.
2. **Status,** quiet: 12 px, `--gf-faint`, lower case by css; `.state-needs`' colour when it waits for the person.
3. **Stop** and **Close,** while allowed, `.btn.quiet.small`. The mock has neither; they are the only way to stop or end a chat.
4. Spend, in the view, hidden in the browser.

The next plan adds the mode group and "Start goal…" after the byline: leave the right part a flex row that takes them.

On a phone the byline goes (`.session-top .right .byline`), and with it the chooser's summary: the "Change model" name shows instead, `.btn.quiet.small`, so the choice stays reachable.

## 4. The conversation

`.thread` > `.thread-inner`, 720 px centred, gap 20, scroll position kept at the end when the person was at the end (`render`). The conversation is still `[role=log]`.

- **The person,** `.turn-you`; **the agent,** `.turn-agent`, Markdown as today, its first and last margins zero. `.temper-who` and its clock are hidden from sight but stay for readers and the terminal, except "Being sent" and "Sent": 12 px faint, right aligned under the bubble. `.temper-day`: hidden. `.temper-marker`: `.meta-line`.
- **A delegate's words:** as `.turn-agent`, "TN" kept visible in 12 px faint above it.
- **Tool lines,** `.turn-tool`, with the pen icon (`ICON.pen`, added under `temper-client-shell-web/files/`):
  - the disclosure of other calls keeps its name and description ("2 tool calls", "read ×2"), as one `.turn-tool` line: pen, name in `--gf-muted`, description in `--gf-faint`; open, its work lines indented 24 px;
  - **a delegated change,** the card of 5.7, in the browser one `.turn-tool` line: pen, "Change", its title in `--gf-ink-2`, its state lower case, its result's words once it has ended ("CI passed. Landed · merge commit 1a2b3c4"), then a link "Open" to its page, underlined as `.turn-tool button`. Meta line and steps are hidden in the browser and stay for the terminal. The link's name stays "Task N"; its text in the browser is "Open" (a new `Word`; the terminal keeps "Task N ↗"). A held change shows "held" in `--gf-signal`.
  - **a goal,** a delegate that is tracked (from its `goal` call or its document): the same line with "Started goal", its title in `.file` mono, its state, "Open".
- **While it works,** `.typing`: three dots at the end of the log, named "Writing", while the chat's status is Starting, Writing or Working or a tool's and the window's last line isn't provisional text. Terminal: nothing new.
- **Provisional text:** `.turn-agent`, "writing" label hidden.
- **Cards in the conversation** (a proposal, a held task, a question's answer) take `.proposal`: white, radius 12, its shadow, 20 px 22 px; a kind's title in `.proposal-title`; `.proposal-split`, `.proposal-state`; first button `.btn.primary.small`, the rest `.btn.quiet.small`.
- **"Older":** a `.btn.quiet.small`, centred, at the head.
- **Ended:** no composer, as today; its end line in `.meta-line`'s look.

## 5. The composer

One composer, the session's (`session_composer`, `.convo-foot`, `.composer-wrap`, `.composer`, `.composer-hint`), on every chat. The `crate::choice::composer` block moves as section 3 says.

- Placeholder and text area's words: "Reply to T4" (`Word::MessagePlaceholder`, drop the ellipsis and "Message"; the name stays "Message").
- `Word::ChatSendHint`: "Enter sends · Shift+Enter for a new line", on configure's too.
- The send arrow: `ICON.send` on ink, 36 px, radius 10, grey (`--gf-line-2`) while disabled, as `.composer button`.
- Its column: `.convo-foot > *` 720 px centred, so that composer and turns share their edges; on configure's page the same.

## Design and the cut

- **Design first:** `temper/docs/design/client/README.md` (3.5's table, chatbar, chats list, new chat's bar, conversation; 5.3; 5.7's look), `client/browser.md` (2.3's addresses, 2.5's tokens), `client/journeys.md`, and their status files, in the same words as this request. Replace "the mock's `chats.html`" references in the pages' table with the greenfield mock's. The citation gate (`tools/citations --all`) must stay at zero.
- **One order,** a chain of `needs`, since all of it conflicts in `temper.css`, `temper/tools/gallery/src/tests.rs`, `names.txt` and the terminal's goldens (`temper-client-shell-terminal/src/tests/golden/chat*.txt`, `chats.txt`, `first_run.txt`): design; `NewChat` and the address's move, with the playground; the Chats page; the session bar and composer; the conversation; goldens, gallery, recording.
- **Gallery:** the fixtures `chats-solo`, `chats-empty`, `chats-reference`, `chats-first-run-*`, `chat-solo-T10`, `chat-solo-T6`, `chat-reference-T1`, `chat-first-run-*`, `chat-closing`, `configure-chat`, `configure-chats-tag` hold; add `chat-new`, `chat-new-no-repository`, `chat-working` (dots), `chat-change-running`, `chat-change-landed`, `chat-goal`, `chat-byline-open`, `chats-no-repository`, `chats-live`, `chats-closed`.
- **Recording:** `welcome-playground`'s walk changes, since it now passes `NewChat`: walk it anew on a fresh fake box and add it (`journeys.md`, 8). Say which journeys were reprinted and which walked.
- **No `Look-request:` trailers:** no look request lands here.

## The proof

For each of these, a render of the client beside the mock's page, at 1440, 1000 and 420 wide: Chats (All, Live, Closed, empty, no repository), New chat, a live chat with tool lines, a chat at work with dots, a chat with a change running and landed, a closed chat, a configuring chat, the byline open. Saved on the builder under `~/src/fabrig/ui-chats/`, outside any worktree, never committed; the report names the builder and lists each difference that remains, with why. Throwaway render scripts stay untracked and are gone before the push.

A test world: a chat started from `NewChat`, a change delegated, landed, and the chat closed, in the browser's world as in the terminal's.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's.

## Beside it

`look-round-5` lands `configure-temper-name` at the same time: the configuring chat's page and the conversation's and composer's shared column (a scrollbar's gutter on both edges, at most 720 wide, centred), which is the column every chat should have. Take its rules on catch-up instead of redrawing them. `ui-needs`, `ui-goals`, `ui-changes` and `ui-welcome` run beside it: keep to chats' pages.

## Sources

- `temper/docs/design/client/greenfield/app.js` (from `origin/ux-ideas/greenfield`: not on main)
- `temper/docs/design/client/greenfield/mock.css` (the same)
- `temper/crates/temper-client-view/src/pages/chats.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
