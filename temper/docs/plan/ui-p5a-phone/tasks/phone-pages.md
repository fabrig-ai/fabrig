+++
id = "phone-pages"
part = "pages"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-view/src/pages/needs.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/tests/needs.rs", "temper/crates/temper-client-view/src/tests/look.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/tools/gallery/src/tests.rs", "temper/docs/design/client/README.status.md", "temper/docs/design/client/browser.status.md"]
effort = "high"
minutes = 170
+++

# phone-pages: Needs you's phone bar, and 44 px targets on Home, Needs you and Work

## Goal

On a phone, Needs you's open card is the whole story's `phone-need.html`:
its own top bar, "‹ Needs you" back to Home and "1 of 2", in place of the
frame's switcher and Away chip, with the night switch kept; under it the card
alone, its question 19 px and its choices stacked full width at 44 px. The
dots and the "Next:" line leave the view. And every control of Home, Needs
you and Work stands at least 44 px tall to a press at 760 px and under, the
mockups' drawings kept.

Acceptance:

- `pages/needs.rs`, while a card is open (the flow), draws first the slot
  `look::NEEDS_BAR` with a link named "Needs you" to the Home that Done's
  "Home" opens (one helper serves both), then the place "1 of N" ("1 of N+"
  where cut) in a text styled `look::NEEDS_BAR_AT`, as
  `temper/docs/plan/ui-p5a-phone/notes/contracts.md`, *Needs you's phone
  bar*, has it. No `look::FLOW_HEAD`, `DOTS`, `DOT`, `DOT_DONE` or
  `NEXT_UP` node is drawn; those styles leave `look.rs` and their rules
  leave `temper.css` unless something else reads them.
- `temper.css`: over 760 px `.temper-needs-bar` is not displayed; at 760 px
  and under it is the page's top bar as the mockup's `.ptop` draws it
  (padding 12px 16px, the line under it, 14 px, the link first with its
  "‹" by `::before`, the place muted at 13 px, room at the right for the
  night switch, which keeps its fixed place), and on that page the rail's
  switcher and Away chip are not displayed, as a session's top bar hides
  them; the card's question is 19 px and its choices are full width, left
  aligned, at least 44 px tall.
- At 760 px and under, every control of Home (its cards' acts, "Decided for
  you · N ›", a decision's Overturn), Needs you (its choices, "Send", "Open",
  "What the agent saw", Accept, Reject, Release, Leave held, the review
  link, the Done column's "Home", a decision's Overturn, the needs bar's
  link) and Work (its filters and its cards' links) stands at least 44 px
  tall to a press, by `notes/common.md`'s *A 44 px target*. A link inside a
  line of words (a from line's title) is not a target of its own.
- A new table test in `temper/crates/temper-client-view/src/tests/look.rs`,
  `each_phone_target_stands_44_pixels`, reads `temper.css`'s
  `@media (max-width: 760px)` blocks and holds, a row per target class, that
  a rule selecting it gives it a `min-height` of 44 px or more (or a
  44 px `::after` target); `phone-tabs` adds its rows later.
- The view's Needs you tests hold the bar, its link's address on one and on
  two projects, its place, cut and not, and no dots or "Next:" line; the
  gallery's `needs-list` print holds `temper-needs-bar` and "1 of 3" (a row
  of an existing print test).
- The status entries this builds leave: `README.status.md`'s **Not yet**
  for 3.5, *Needs you*, *On a phone*; and in `browser.status.md`'s **Not
  yet** for 2.5, *What a phone presses*, the words "Home's," and "Work's,"
  and "Home's acts, Work's filters," (Ask temper's stay for `phone-tabs`).
- Each new check bites: drop the bar's link once, and drop a target's
  `min-height` once, and see the tests fail for that reason; restore.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Needs you*: *Its head*, *On
  a phone*, *The card*, *Done*.
- `temper/docs/design/client/browser.md`, 2.5: the phone's paragraph and
  *What a phone presses*.
- The mockups: `temper/docs/design/client/web/whole-story/phone-need.html`,
  `phone-home.html`, `whole-story.css` (`.ptop`, `.pbody .choice`,
  `.pchip`).
- `temper/docs/design/draft/surfaces.md`, 10.
- `temper/docs/plan/ui-p5a-phone/notes/contracts.md`, *Needs you's phone
  bar*; `temper/docs/plan/ui-p5a-phone/notes/common.md`.

## Owns and may change

- `temper-client-view`'s `pages/needs.rs` (the flow, Done's "Home" helper),
  `look.rs` (`NEEDS_BAR`, `NEEDS_BAR_AT` appended; the flow head's and
  next-up's styles removed), `words.rs` (no new word is expected: "Needs
  you" and the place's words exist; if one is needed, append it),
  `tests/needs.rs`, `tests/look.rs`.
- `temper.css`: rules under `/* ==== ui: needs ==== */`, `/* ==== ui: home
  ==== */`, `/* ==== ui: work ==== */` and `/* ==== ui: phone ==== */`; the
  flow head's, the dots' and next-up's rules removed where they stand. Not
  the rail's, the tab bar's or a session's rules: those are `phone-tabs`'.
- `temper/tools/gallery/src/tests.rs`: a row in an existing print test for
  `needs-list`; any existing row your change trips (repair it, keeping what
  it checks for the rest). Report the gallery's `--lib` CPU seconds before
  and after.
- `temper/docs/design/client/README.status.md` and `browser.status.md`:
  the entries the acceptance names, nothing else.
- If a journey's print changes (none is expected: no journey opens Needs
  you), reprint them as `notes/common.md` says.
- `ui-reaches` (N1) also edits `pages/needs.rs`; if its why and "Stop
  asking" have landed on your base, keep them in the card as they stand,
  under the bar.

## Steps

1. Read the flow in `pages/needs.rs` (`flow`, `main`'s Done branch) and its
   tests in `tests/needs.rs` (the `FLOW_HEAD`, `DOTS` and `NEXT_UP` rows).
2. Draw the bar in place of the flow head; lift Done's "Home" address into
   one helper the bar's link and Done share; remove the dots and next-up.
   `heavy cargo nextest run -p temper-client-view --lib -E 'test(/needs/)'`.
3. The stylesheet's phone rules for the bar, the card and the targets; the
   new table test in `tests/look.rs`;
   `heavy cargo nextest run -p temper-client-view --lib -E 'test(/look/)'`.
4. The gallery row: `heavy cargo nextest run -p temper-client-gallery --lib`
   (time it before and after).
5. Break each new check once, as the acceptance says.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-client-view -p temper-client-gallery --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-view`;
   `heavy cargo nextest run -p temper-client-journeys --lib -E 'test(=tests::the_committed_journeys_are_their_print)'`.
7. Delete the status entries, commit ("ui-p5a-phone: Needs you's phone bar,
   and a phone's targets at 44 px on Home, Needs you and Work") and push.

## Report

As the implementer's text says, with each rule broken and the failure seen,
the gallery's CPU seconds before and after, and the targets you sized, by
class.
