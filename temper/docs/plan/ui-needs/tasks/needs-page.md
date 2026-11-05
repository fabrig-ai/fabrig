+++
id = "needs-page"
part = "needs-page"
kind = "feature"
needs = ["question-parser", "needs-frame"]
files = [
  "temper/crates/temper-client-view/src/pages/ask.rs",
  "temper/crates/temper-client-view/src/pages/needs.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/tools/gallery/src/needs.rs",
]
effort = "high"
minutes = 110
+++

# Needs you, the browser's page: a question, one card at a time

## Goal

In the browser, `Address::Inbox` draws *Needs you* for a question, in every state but the other kinds
(`needs-kinds`): the flow's head, the card, its choices, pair and reply form, the next-up line, the
answered lines and Done. A card of another kind draws its question line, its mono line and its pair
alone until that task. The terminal's page stays the list it is.

Acceptance:

- `pages/inbox.rs`' `main` on `Shell::Browser` calls `pages/needs.rs`' `main`; on `Shell::Terminal` it
  draws what it does today, byte for byte.
- Every part of `temper/docs/design/client/README.md`, 3.5, *Needs you*, but *By its kind*'s other
  kinds, holds in tests in `src/tests/needs.rs`, through the browser's words (`browser_page_view`): the
  roles, names, words and acts, each named in the report.
- The values of `temper/docs/design/client/greenfield/mock.css`, `.btn`, `.primary`, `.quiet`, `.small`,
  `.rec` (90-97), `.ask` and under it (101-109), `.answered` (114-117), `.flow-head`, `.dots`, `.flow-q`,
  `.flow .choices`, `.flow .pair`, `.next-up` (280-289) and their phone lines (430-470) are in
  `temper.css` under the client's selectors, under the marker.
- The gallery has, from fixtures through the public events: `needs-one`, `needs-saw-open`,
  `needs-reply-open`, `needs-second-answered`, `needs-later`, `needs-last`, `needs-done`,
  `needs-plain-words`, in `names.txt` directly after `inbox-solo`, in that order. `needs-proposal`,
  `needs-held`, `needs-funds` are `needs-kinds`'.
- `inbox-solo`, `inbox-reference` and `inbox-empty` draw the new page, and their rows say so.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *Needs you* and *What the client keeps*.
- `temper/docs/design/domain/agent.md`, 8.2, *A question's line form*.
- `temper/docs/design/client/browser.md`, 7.2, the gallery's table.
- `temper/docs/design/client/greenfield/app.js`: `needsPage`, `choiceButtons`, `replyBox`,
  `answeredLines`; `data.js` for its words.

## Owns and may change

Owns `files`. May also change:

- `temper-client-view`: `src/pages.rs` (`mod` lines), `src/pages/inbox.rs` (the branch; `pub(crate)` on
  what the new files call), `src/pages/home.rs` (`need_title`, `need_when`, `need_kind` move to `ask.rs`,
  Home calling them there, its output the same), `src/confirm.rs` (the outcome's words as a function a
  card can call, its output the same), `src/look.rs`, `src/words.rs`, `src/limits.rs`, `src/tests.rs`.
- `temper-client-domain/src/acts.rs`: `Act::CardWords` takes words up to `env.limits.client.domain.text`,
  a person's words, rather than `CUE_BYTES`: a reply past 256 bytes would otherwise go cut with no word.
  The engine's own bound then refuses on the card. Its test in `src/tests/acts.rs`.
- `temper.css`, under `/* ==== ui: needs ==== */`, and the `.temper-page` rules' `:has`, if one catches
  this main.
- The gallery: `tools/gallery/names.txt`, `src/lib.rs` (`mod needs;`, and for `needs-saw-open` alone,
  `<details` forced open in the page as `<dialog` is), `src/samples.rs` (the rows, `N`, and `domain()`'s
  branch to `needs::domain(name)`), `src/tests.rs` (a `HOLDS` row each; the rows of `inbox-solo`,
  `inbox-reference` and `inbox-empty`; `times_read_against_each_mockup_worlds_fixed_now`'s two for
  `inbox-solo`, and its mockup map in `samples.rs`).
- Browser tests that walk what the page removes, surveyed; each now reaches the filter by its address
  and asserts the new page:
  - `temper-client-shell-web/src/host_tests.rs`: `a_filter_chosen_pushes_the_inbox_at_its_query`,
    `a_load_at_a_filtered_address_opens_the_inbox_filtered`,
    `a_stale_filter_link_opens_less_filtered_at_its_printed_address`;
  - `temper-client-shell-web/tests/suite/host.rs`, `a_start_at_a_stale_filter_shows_its_printed_address`;
  - `temper/tests/client-browser/tests/browser_all/browser_pages.rs`,
    `history_keeps_the_chat_focus_and_filter_across_reload_and_back` (h1 "Inbox" twice);
  - `temper-client-view/src/tests/rail.rs` (h1 "Inbox", the browser's arm), `src/tests/inbox.rs`,
    `updates_do_not_repeat_the_playground_project` (the browser's arm goes, its terminal arm stays);
  - `src/tests/{frame,chats,look}.rs`: a both-shell loop rooted at the Inbox address, only if it reads
    the removed words.
- The terminal's tests (`src/tests/{inbox,confirm,cards,times}.rs`, `temper/tests/client/**`,
  `temper/tests/one-process`, `temper/crates/temper/tests/process.rs`) and its golden
  (`temper-client-shell-terminal/src/tests/golden/inbox.txt`, built by hand) stay as they are. If one
  goes red, the branch is wrong.
- `temper/docs/design/client/README.status.md`, `browser.status.md`: what this task closes.

Options, each with its default:

- `.dots`: `Role::Figure` named "2 of 3", its pips `Role::Text` children, since `Role::Image` is an
  `<img>` with no children. `.flow-q` is `Role::Heading { level: 1 }`.
- The reply form is a `Role::Slot`, with `Role::TextArea` binding `Act::CardWords` and
  `Builder::placeholder`; the textarea takes the focus as the form opens, by `Builder::focus`, as the
  mock's does.
- The choice's `.rec` is a `Role::Text` inside the button, the button's name the choice's words alone.
- The "Home" link's project: `filter.project`, else `Pages::project`, else the first of `me.projects`;
  with none, no link.
- A title in the mono line for a filter with no project: PROJECT · TITLE · kind · age, with
  `Word::ToolSeparator`.

## Steps

1. `src/pages/ask.rs` and `src/pages/needs.rs` as `notes/contracts.md`, *A card, as the view reads it*,
   has them. The page, top to bottom:

   ```text
   Main (MAIN, NEEDS)
     until inbox.delivered: nothing more
     Slot FLOW_HEAD: Figure DOTS "2 of 3" { Text DOT[, DOT_DONE]... } · Paragraph KICKER "2 of 3"
     Slot ASK
       Paragraph ASK_META  (ask::meta)
       Heading 1 FLOW_Q
       Paragraph ASK_CONTEXT            if any
       Disclosure "What the agent saw" { Paragraph }  if any, not Expanded
       refusal's words                  if Session::refused
       Slot ASK_CHOICES: Button (RECOMMENDED where primary) { Text REC "recommended" }...
       Slot PAIR: Button "Reply…" Act::Needs(OpenReply) · Button "Later" Act::Needs(Later)
       Slot ASK_REPLY                    if replying
     Paragraph NEXT_UP "Next: TITLE · SHORT" | "The last one"
     Slot { Heading 2 "Answered", ask::answered_lines }  if any
   crate::confirm::main
   ```

   Done: `Main (MAIN, NEEDS)`, kicker "Needs you", h1 "Nothing needs you.", the lede, the lines, the
   link "Home" drawn as a plain button. Lines and total use `Session::answered(times.now,
   ANSWERED_ON_NEEDS, filter.project)`; `times.next` takes the oldest line's end. The total is lines
   plus `Session::open`, "+" where `inbox.cut[0]`, and the pips are that total.

2. `src/tests/needs.rs`, at the least: the example's card with each part in order; the recommended and,
   with none, the first as primary; `.rec` on a question; the pair's and form's acts; the form's
   placeholder; fallback words: no `.choices`, no saw; a cut question falls back; the head at 1 of 3, 2
   of 3, "2 of 3+" and its pips, done first; "Next: TITLE · SHORT" and "The last one"; `at` shown; later
   last; an answered line with and past its Undo; Done with and without lines; unfiltered: the project
   first; the empty column until delivered; a refusal's words on the card; that the terminal's page
   still has its h1 and filters.
3. The gallery's module, the fixtures in the mock's words, each state reached by `driver::step` with
   `Act::Needs`, so no state is set by hand. CSS. Readers.
4. Render each state at 1440, 1000 and 420 against `needs.html` in a throwaway script, as a check, not
   yet the proof.
5. `heavy cargo nextest run -p temper-client-view --lib`; `-p temper-client-domain --lib`; `-p
   temper-client-gallery`; temper-client-gallery`; `-p temper-client-shell-web`; `-p temper-client-world`; `-p
   temper-client-browser-world -E 'test(browser_pages)'`; the journeys' reprint (`welcome-playground` may
   still be named stale before `journey-again` lands; that isn't yours). fmt, clippy and the checks of
   the implementer's text. Commit and push.

## Report

As the implementer's text asks, and each difference you saw from the mock, with its reason.
