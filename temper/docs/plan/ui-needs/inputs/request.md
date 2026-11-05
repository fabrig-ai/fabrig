# ui-needs: Needs you, one card at a time, and answering on Home

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo the look requests already landed; flow, look and mechanics are what count. This plan is the first of three for the area (`ui-needs`, then `ui-home-status`, then `ui-away`). It starts once `look-round-4` has landed.

## The mock

The mock is on main at `temper/docs/design/client/greenfield/` (its `README.md` says what's what). Open its pages from the checkout; nothing needs extracting.

Functions in `app.js`: `needsPage`, `home`, `askCard`, `choiceButtons`, `replyBox`, `answeredLines`, `shortOf`, `answer`, `undo`, `open`, `recent`, `navItems`, `tabbar`, `toast`. Rules in `mock.css`: `.btn` and its `.primary`, `.quiet`, `.small`, `.rec` (90-97), `.ask` and all under it (101-109), `.lines` (110-113), `.answered` (114-117), `.needs-cta` (118), `.flow-head`, `.dots`, `.flow-q`, `.flow .choices`, `.flow .pair`, `.next-up` (280-289), `.toast` (416-418), the phone block (430-470). Bring their values into `temper.css` under the client's selectors, with the `--gf-*` tokens already there.

## 1. The question's words

Decided, not a question: **no protocol, store or tool change.** A question already carries its options and recommendation in its words (`temper/docs/design/domain/forge.md`, 8.11). Fix that as a line form:

```
Fix the dropped watch in jig’s store, or keep it alive in temper serve?

MakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local.

Saw: The box test reproduces it every time.
Choice: In jig’s store (recommended)
Choice: In temper serve
```

- The first line is the **question**; the paragraphs up to the trailing lines are the **context**; a line beginning `Saw: ` is **what the agent saw**; each line beginning `Choice: ` is a **choice**, at most one ending ` (recommended)`. Trailing lines alone count.
- A parser in `temper-client-domain` (a pure function, with tests), used by Home and Needs you. Words with no `Choice:` line, or cut (`question.words.cut`), fall back: first paragraph the question, the rest the context, no choices, answered by Reply alone.
- `temper/crates/temper-service/src/stock/chat.txt`, the goal's paragraph, gets one sentence: "Write a question to your requester as one line asking it, a blank line, what they need to know, then a line starting Saw: with what you saw, and a line starting Choice: for each option, the one you recommend ending (recommended)." This spends charter 1's host part: update the stock figures its tests hold (`temper-service`'s `tests/stock.rs`, `temper-engine-smith`'s `tests/brief_fit.rs`, `../protocol/configuration.md`, 13.4), and check the stock brief still fits.
- A choice's button is the existing `Answer` act with the choice's words as the answer.

## 2. Needs you, the browser's page

`Address::Inbox` keeps its path and its filters. In the browser it draws one card at a time (the terminal: section 6). The level-1 "Inbox", the Needs and Updates sections, the filter menus and the page's "+ New chat" go, on this page. Bare `/inbox` works across projects, with the project's name before the owner on the mono line. The rail's and tab bar's "Needs you" go to the open project's filter.

Open cards are `inbox.needs()`, those marked later last. One is shown: the one a Home row asked for, else the first.

- **Column:** as `.col.narrow.flow`: 600 wide, centred, gap 32.
- **`.flow-head`:** `.dots` with `role=img`, named "2 of 3", a pip per item (22 by 4, radius 2, `--gf-line-2`; `.done` ink), done pips first; at the right "2 of 3" as `.kicker`. Total is the answered of this session's page (section 4) plus the open; where the read is cut, "2 of 3+".
- **Card** on no ground (`.ask` with no background, no shadow, padding 0), gap 16:
  - `.mono` line: TITLE (a link to the task) · kind · age, as Home's `ASK_META`. Kind: "a question", "a proposal", "held", "a person task", and **"funds"** for a held task whose reason is `HoldReason::Budget` (`ui-goals` says "needs funds" of the same hold, `HoldCategory::Exceeded`).
  - `h1.flow-q`: 26px/1.35, 500, -.01em; 23px on a phone.
  - `.ctx`, if any, `--gf-ink-2`.
  - `details` "What the agent saw", if any: `<summary>` width fit-content, 14px muted, `p` margin-top 6. Closed at first.
  - `.choices` in a column: min-height 54, padding 10 18, radius 12, 16px. The recommended one (with none, the first) `.primary`, with `.rec` "recommended" at the far right on a question alone, 12px, 400, opacity .72.
  - `.pair`: two columns, gap 10: **"Reply…"** and **"Later"**, min-height 46, radius 12, no border, `--gf-hover`, centred. On a proposal, a held task or a person task, the first is **"Open"**, to its task or chat page.
  - The reply form under the pair, hidden until "Reply…": a textarea (min-height 72, `--gf-line-2`, radius 8, `--gf-paper`), placeholder "Your words go to TITLE’s agent"; "Send reply" (primary, small) and "Cancel" (quiet, small).
- **Kinds:** a proposal's question line is its reason's first sentence, its facts and rows as `.ctx` beneath, then Accept (primary, no "recommended") and Reject (plain, its reason's textarea as today), through the confirmation they have today. A held task: its title and `card_words`, then Release (primary) and Leave held (plain). A funds card: "$S of $L used." as the question, in the task page's money form, its title as owner, Release and Leave held. A person task: its title, spec, its existing acts as plain buttons. No "Add $10": budgets are the low priority.
- **`.next-up`:** centred 13px `--gf-faint`: "Next: TITLE · SHORT", or "The last one". SHORT is the next card's question line; on a funds card "$S of $L used".
- **Answered**, under it when any: `section-title` "Answered", then `.answered` lines.
- **Done:** `.col.narrow`, no flow head: kicker "Needs you", h1 "Nothing needs you.", lede "Every question is answered. Work carries on; Home tells you what changed.", the `.answered` lines, a plain button-link "Home". ("Work carries on" follows Home's landed words, not the mock's "Goals".)

Acts: a choice answers and moves to the next card. "Send reply" answers in words. "Later" marks it, moves on, toast "Later: it stays in Needs you, after the others." A later card shows " · later" on Home's card line.

## 3. Home's needs, in place

Home's `needs` and `ask` in `temper-client-view/src/pages/home.rs` use the same parsed card, as `askCard`: mono line (with " · later"), `p.q` 17px, `.ctx`, saw, `.choices` in a row (min-height 40), then quiet "Reply" and quiet "Later", in place, the reply form under them. On a proposal or held card, its acts and quiet "Open" and "Later". The existing `temper-ask-*` rules hold most of this.

- `.answered` lines between the section's head and the card, those answered in the last 2 minutes.
- **Rows** under the card: SHORT at the left (`shortOf`: a funds row "TITLE: $S of $L used"), kind · age at the right, the whole a link that opens Needs you at that card.
- **Phone:** the existing CTA, its label "TITLE: SHORT", the button "Answer" for one and **"Answer, one at a time"** for more.
- **While "To begin" shows,** as `freshHome`: the card alone, no rows, no "Answer in a row".
- **Headline in number words:** "One thing needs you.", "Two things need you." to "Nine", digits after; "N+ things need you." when cut.

## 4. What the client keeps (in `Pages`, not on a page; gone on reload)

- **`later`:** a set of `ObjectKey`.
- **`answered`:** `{ object, title, label, at }`, a bounded list. Label: the choice's words, or `Replied: “…”`, cut at 57 characters plus "…" past 60. An item shows on Needs you for 10 minutes, on Home for 2.
- **`at`:** the card a Home row asked for.
- **The held send (Undo):** jig has no retraction, so a choice or reply is held in the client for 5 seconds, a domain deadline as `next_deadline` has; the card moves to answered at once, with the toast "Answered: LABEL" and its "Undo". Undo restores the card and sends nothing. At 5 seconds the `Answer` goes; after that the line shows no "Undo". A refused answer puts the card back with the refusal's words on it. A tab closed inside the 5 seconds sends nothing and the question stays open. Accept, Reject, Release and Leave held send at once, with no Undo.
- **`toast`:** one at a time, 4.5 seconds, with one optional act. Drawn as `Role::Status` at the end of the frame, in the mock's `.toast`: fixed, centred, bottom 28 (88 on a phone), ink ground. A new look token; other pages will reuse it.
- **The rail's count:** one standing `Subject::Inbox` watch once signed in, its rows counted by the open project; "N+" when the document is cut. `needs_total` stays for the terminal's `navigation`. A later item still counts.

## 5. Words

"Reply…", "Later", "Send reply", "Cancel", "The last one", "Next: ", "Answered", "recommended", "funds", "Open", "What the agent saw", "Answered: ", "Undo", "Answer, one at a time", "Your words go to ", "Later: it stays in Needs you, after the others.", "Every question is answered. Work carries on; Home tells you what changed.", "Nothing needs you.", "Home", and "N of M".

## 6. The terminal

The view is shared and the terminal's prints keep their meaning. Its `Inbox` stays the list it is, with Updates, filters and answering in words: a `Shell::Terminal` branch, as Home has. Its Home prints the choices as buttons. `each_fixture_page_prints_as_its_golden_file`: reprint where the meaning holds; the question line form reads plainly in its prints.

## Decisions already taken

- Updates leave the browser's page; no page replaces them. Keep `Seen` marking as it is.
- Later is session order, no store record.
- The chat's own questions in words stay as they are (`n-license` in the mock's hello): no inbox entry.
- No "Add $10"; funds use the held acts.
- `Goals carry on` stays the client's "Work carries on".

A change to a stored format, a protocol or a public interface needs no `Question:`, but none is expected here.

## How the cut shapes it

Tasks, chained by `needs`, since `temper.css`, `temper/tools/gallery/src/tests.rs` and `names.txt` and the terminal's golden pages conflict:

1. The words form, its parser, the charter sentence and figures.
2. `Pages`' session state, the toast, the standing count, the rail's and tab's filtered link.
3. Needs you, the browser's page, every kind and state.
4. Home's card, rows, lines, headline words, CTA.
5. The held send and Undo.
6. Design and proof: `temper/docs/design/client/README.md` 3.5 (Home, Inbox) and section 11 (remove "A question answered on Home" and "The mockups' question card" as settled, or say what's left), `client/browser.md` (the gallery's list), `temper/docs/design/domain/agent.md` or `forge.md` 8.11 for the words form. Every statement of "Inbox" that no longer holds in the browser.

**Gallery states** (`tools/gallery/names.txt`, in order, from fixtures): `needs-one`, `needs-saw-open`, `needs-reply-open`, `needs-second-answered`, `needs-later`, `needs-last`, `needs-done`, `needs-proposal`, `needs-held`, `needs-funds`, `needs-plain-words`; `home-morning-card`, `home-answered`, `home-one-need`.

**The proof:** each page rendered against the mock's at 1440, 1000 and 420 wide, plus its open states (saw open, reply open, toast showing), compared as `temper/docs/design/draft/look-requests.md`, section 4 does, with screenshots saved on the builder under `~/src/fabrig/ui-needs/`, outside any worktree, the report saying which builder. One live walk with `temper-fake`: a scripted goal asks in the line form, the person presses the recommended choice on Home, and the goal hears those words. Throwaway render scripts stay untracked and are gone before the push. No `Look-request:` trailers.

**Beside it:** `ui-home-status` and `ui-away` chain after this one and touch Home and the rail. Other areas may reuse the toast.

## Size

One cut is at most 90 minutes. Release early. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Render proofs

Chromium 147's new headless won't go under 500 px with `--window-size`, so a 420 px proof sets device metrics through the DevTools protocol (`Emulation.setDeviceMetricsOverride`). Any task may use a throwaway render script, untracked and gone before the push. Fonts: the mock's Google Fonts link needs the network; the client's own fonts are what's compared.

## Sources

- temper/docs/design/draft/look-requests.md
- temper/crates/temper-client-view/src/frame.rs
- temper/crates/temper-client-view/src/pages/inbox.rs
- temper/crates/temper-client-view/src/pages/home.rs
- temper/crates/temper-client-view/src/confirm.rs
- temper/crates/temper-client-domain/src/pages/inbox.rs
- temper/crates/temper-client-domain/src/pages/home.rs
- temper/crates/temper-client-domain/src/acts.rs
- temper/crates/temper-service/src/stock/chat.txt

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
