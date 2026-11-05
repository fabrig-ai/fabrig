+++
id = "home-card"
part = "home-card"
kind = "feature"
needs = ["needs-kinds"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
]
effort = "high"
minutes = 90
+++

# Home's needs, in place

## Goal

Home's *Needs you* draws the same parsed card as the mock's `askCard`, answers in place, and its lines,
answered lines, headline and phone block read the open cards.

Acceptance, each holding in tests, in both shells where it draws in both:

- **The card** (`ask` in `pages/home.rs`, through `pages/ask.rs`): `ask::meta` with " · later" where
  `Session::later`; `.q`, 17px; `.ctx`; the saw's disclosure; a question's `.choices` in a row
  (min-height 40), then quiet "Reply" and quiet "Later", in place, `ask::reply_form` under them; words in
  no line form: quiet "Reply" and "Later" alone; on a proposal or a held task its acts as `needs-kinds`
  has them, quiet "Open" and quiet "Later"; `Session::refused`'s words on the card.
- **It answers in place:** the acts are `Act::Needs(..)`; nothing here goes to `Address::Inbox` but a
  line, "Answer in a row" and the phone's block.
- **Answered lines,** between the section's head and the card: `Session::answered(times.now,
  ANSWERED_ON_HOME, Some(project))`, `times.next` taking the oldest's end.
- **Lines** under the card: `Session::open` after the first, at most five: SHORT at the left (a funds
  line "TITLE: S of L used"), kind · age at the right, the whole a link binding `Act::Needs(Open {
  object })`.
- **The phone:** `HOME_CTA` says "TITLE: SHORT", its go "Answer" for one card and "Answer, one at a
  time" for more; to Home's filtered `Address::Inbox`. The browser's alone.
- **While `facts.to_begin()`:** the card alone, no lines, no "Answer in a row", and on a phone the card
  stays, with no CTA, as the mock's `freshHome` has it.
- **The headline:** "One thing needs you.", "Two things need you." to "Nine things need you." in number
  words, "10 things need you." and on in digits, "N+ things need you." when cut, over the open count
  `facts.needs` already holds. The section's "Needs you · N" counts the open cards too.
- **The terminal:** its Home prints the choices as buttons, "Reply" and "Later", the lines, and the
  answered lines, each line one readable run of words; `w.shell` where a part is drawn by style alone.
- **`temper.css`:** the mock's `.ask` and `.choices`, `.lines` (110-113), `.answered`, `.needs-cta`, the
  phone's block, onto the `temper-ask-*`, `temper-home-rows` and `temper-home-cta` rules, removing or
  changing them where they stand; the `.temper-ask-context` and `-rec` rules gain their tokens.
- **Gallery:** `home-morning-card`, `home-answered`, `home-one-need`, in `names.txt` directly after
  `home-github`; `home-needs-several`, `home-at-work`, `home-needs-under-begin` still pass, their rows
  changed for the new parts.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *Home on any project*, *Needs you*, *What the client
  keeps*.
- `temper/docs/design/client/greenfield/app.js`: `home`, `askCard`, `freshHome`, `shortOf`, `recent`;
  `mock.css` as above.

## Owns and may change

Owns `files`. May also change:

- `temper-client-view`: `src/pages/ask.rs`, `src/words.rs` (the number words as words; "Answer, one at a
  time"; `HomeAskLine`'s use), `src/look.rs` and `src/tests/look.rs` (tokens), `src/limits.rs`.
- `temper-client-domain/src/pages/home.rs`, only if `Facts` needs a field; the count is already there.
- `temper.css`.
- The gallery: `tools/gallery/names.txt`, `src/samples.rs` (`home_domain`'s words in the line form for
  the new states; acts on its `Question`, empty today, so that its choices draw), `src/tests.rs`
  (`HOLDS` rows, and the rows for `home-at-work`, `home-needs-several`, `home-needs-under-begin`).
- Tests pinning today's card, surveyed, in `src/tests/home.rs`: `everyday_headlines…`,
  `first_need_keeps…`, `unread_need…`, `additional_needs…`, `nonquestion_need…`, `read_need…`,
  `phone_answer_name…`, `inbox_act()`.
- `temper/docs/design/client/README.status.md`, `browser.status.md`: what this task closes.

Options, each with its default:

- A card whose page isn't read yet: its mono line alone, as today.
- A line for a card not yet read: its title and kind, as today's row, with no SHORT.
- `HOME_IN_A_ROW` stays where it is, and goes while `facts.to_begin()`.

## Steps

1. The headline's words. The card, then the lines, the answered lines, the block, `to_begin`.
2. `src/tests/home.rs`: every acceptance above; the terminal's print of a card with choices; a later
   card's line; lines' `Act::Needs(Open)`; Home's count falling as a card is answered.
3. Gallery, CSS, render against `home.html` at 1440, 1000 and 420 as a check.
4. `heavy cargo nextest run -p temper-client-view --lib`; `-p temper-client-gallery`; `-p
   temper-client-world`; the reprint. fmt, clippy and the checks of the implementer's text. Commit and
   push.

## Report

As the implementer's text asks, and each difference from the mock, with its reason.
