+++
id = "needs-list"
part = "needs"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-view/src/pages/needs.rs", "temper/crates/temper-client-view/src/tests/needs.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/tools/gallery/src/needs.rs", "temper/tools/gallery/names.txt"]
effort = "high"
minutes = 180
+++

## Goal

The browser's Needs you as the whole story's `needs.html` draws it: on
a wide page a list of the open cards, oldest first, the shown one open
and the others closed rows that open in place; its intro and head; each
card's from line opening with its project's chip across projects; *Tell
it something else*, an always-open field, in place of "Reply…" and
"Later"; an approval's "Review on FORGE ↗" first. On a phone it stays one
card at a time, with the same card. It reads only what the domain reads
today: no new watch.

Acceptance:

- View table rows (`src/tests/needs.rs`) for: the list with three open
  across two projects, the first open and two closed rows in order, each
  closed row's from line and SHORT; pressing a closed row opens it and
  closes the first, on an unfiltered and on a project-filtered address,
  the address unchanged; the intro's kicker unfiltered ("Needs you · all
  projects") and filtered ("Needs you · PROJECT"), and its heading the
  needs sentence; the head "Needs you · 3" and "oldest first"; the chip on
  the unfiltered from line and none filtered by a project; a question's
  field named "Or tell it something else", "Send" only while it holds
  words, sending the card's reply; no "Later" and no "Reply…" in the
  browser; an approval's "Review on GitHub ↗" before "Open"; a phone's
  one-card flow keeping its dots and next-up line with the new card.
- The terminal's inbox is unmoved: its tests pass unchanged.
- Gallery states `needs-list`, `needs-list-second`, `needs-tell`,
  `needs-approval-first` and `night-needs` (`temper/docs/design/client/browser.md`,
  7.2) print and hold their rows; `needs-list`'s Decided for you joins in
  a later task. The existing `needs-*` states hold their rows as changed.
- The focused tests of `temper-client-view`, `temper-client-domain`,
  `temper-client-gallery` and `temper-client-world` pass; a journey the
  replay calls stale is recorded again.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Needs you* (all of it:
  its opening, *The column*, *Its intro*, *Its head*, *The card*, *By its
  kind*, *An approval*, *SHORT*, *`.next-up`*, *Done*, *Its acts*) and
  *What the client keeps* (the reply's words, *At*).
- The mockups: `temper/docs/design/client/web/whole-story/needs.html` and
  `phone-need.html`, and `whole-story.css`'s rules for `.card.need`,
  `.from`, `.chip`, `.choices`, `.choice.rec`, `.field`, `.shead`, `.row`; their words, parts,
  order and look bind. Skip the parts other milestones own:
  `temper/docs/plan/ui-p2-home/notes/common.md`, *Not to touch*.
- `temper/docs/design/client/forge.md`, 8, *Its acts* (the approval's
  review link).
- `temper/docs/plan/ui-p2-home/notes/common.md`;
  `temper/docs/plan/ui-p2-home/notes/contracts.md`, sections 4 and 5
  (the acts and the style names this task adds).

## Owns and may change

- Owns: `temper-client-view/src/pages/needs.rs` and its tests
  `src/tests/needs.rs`; the `/* ==== ui: needs ==== */` and `/* ==== ui:
  phone ==== */` parts of `temper-client-shell-web/temper.css`;
  `temper/tools/gallery/src/needs.rs` and the gallery's needs states.
- May change, for this page alone: `temper-client-view/src/look.rs`
  (append `temper-need-row`, `temper-need-tell`, `temper-needs-intro`,
  `temper-needs-head`, `temper-chip` and the `NAMES` rows; remove a
  style this page no longer draws, if no other page draws it);
  `src/words.rs` (append the page's words; remove words no page says any
  more); `src/shared*.rs` and `src/pages/inbox.rs` where the browser's
  card is drawn from shared code, keeping the terminal's output as it is;
  `temper/tools/gallery/src/samples.rs` (the new states' rows, appended
  beside the needs states), `src/tests.rs` (their rows; the changed rows
  of existing needs states), `names.txt` (regenerated, sorted).
- `temper-client-domain/src/pages/needs.rs`, one edit: in `Act::Open`'s
  arm, after `pages.needs.at = Some(object);`, return with no `Show`
  while the address is `Address::Inbox { .. }`, so that a closed row opens
  in place and keeps the filter; add its row to the domain's needs table
  (`src/tests/pages/needs.rs`). Another task adds an `AllHome` arm to the
  same match below it: keep your edit above the `let project = match`.
- `temper/tests/client` stories that walk "Later" or "Reply…" through the
  browser's view: change them with the page (the domain's acts stay).
- Options, each pre-approved: if a choice's line-form detail ("what each
  choice does") is in the context today, keep it there; if the field
  needs the reply opened (`needs::Act::OpenReply`) before its words are
  kept, open it as the card opens and say so; if an existing `needs-*`
  gallery state shows only "Later" or "Reply…", keep its name and give it
  the closest state the page now has, its row saying so.

## Steps

1. Read `src/pages/needs.rs` (`main`, `flow`, `card_main`, `choices`,
   `context`) and `src/tests/needs.rs`'s table helpers first.
2. The browser's page (`w.shell == Shell::Browser`), the shown need from
   `session.shown(inbox, approvals)`, the open ones from
   `session.needs(inbox, approvals)`:
   - the intro (`temper-needs-intro`): kicker, `h1` the needs sentence
     (Home's words, as `pages/home.rs` says them), while a card is open;
   - on a wide page, the head (`temper-needs-head`): level-2 "Needs you · N"
     and "oldest first"; then each open need in order: the shown one as
     today's card, every other a `temper-need-row` button (from line and
     SHORT, `shared::need_line`'s words) bound to `needs::Act::Open {
     object }`;
   - on a phone, today's `.flow-head` and one card, as now; the browser
     draws both and `temper.css` shows the one for the width, as Home's
     phone block does;
   - the card's from line: `temper-chip` with the project's name first
     while the inbox's filter has no project, then today's mono line;
   - *Tell it something else* (`temper-need-tell`) where the `.pair` was,
     on a question and a person task in words: a textarea named and
     placeholdered "Or tell it something else", bound as the reply form's
     textarea is, and "Send", primary and small, bound to
     `needs::Act::SendReply`, drawn while the card's words are not empty;
     "Open" stays on a proposal, a held task and another person task; no
     "Later";
   - an approval: its review link first and primary, then "Open".
3. `temper.css`: the rules under the two markers, from `whole-story.css`'s
   `.card.need`, `.from`, `.chip`, `.choices`, `.field`, `.shead` and
   `.row`; remove the `.pair` rules the page no longer draws, where they
   stand.
4. Tests: rows in `src/tests/needs.rs`'s tables, one a case of the
   acceptance; the domain's `Act::Open` row. Break each new rule once by
   hand and see its row fail (the implementer's text).
5. Gallery: the five states, with deliveries of three open needs across
   two projects (a question with its choices and context, a funds card, an
   approval) in `src/needs.rs`; `needs-list-second` presses the second
   row; `needs-tell` writes words in the field; `night-needs` is
   `needs-list` by night. Regenerate `names.txt`. Report the gallery's
   `--lib` CPU seconds before and after.
6. Journeys: run the replay (`heavy cargo nextest run -p
   temper-client-journeys`); for a journey it calls stale, `again NAME`
   (`notes/common.md`, *Runs*).
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-view -p temper-client-domain -p
   temper-client-gallery -p temper-client-world --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-client-view -p
   temper-client-domain -p temper-client-gallery -p temper-client-world`.
   Commit and push.

## Report

What you changed, the gallery states and their rows, the journeys
recorded again, the gallery's CPU seconds before and after, the calls you
made inside the sketches (where the phone and the wide forms split), the
tests and their counts, each new row's rule and how it failed when
broken, and your implementation and check minutes apart.
