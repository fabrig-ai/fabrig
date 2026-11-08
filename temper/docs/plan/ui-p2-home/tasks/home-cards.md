+++
id = "home-cards"
part = "needs"
kind = "feature"
needs = ["all-home-page", "all-home-reads"]
files = ["temper/crates/temper-client-view/src/pages/home.rs", "temper/crates/temper-client-view/src/pages/home_cards.rs", "temper/crates/temper-client-view/src/pages/needs.rs", "temper/crates/temper-client-view/src/tests/home.rs", "temper/crates/temper-client-view/src/tests/needs.rs", "temper/crates/temper-client-shell-web/temper.css"]
effort = "high"
minutes = 170
+++

## Goal

Give a project's Home the whole story's needs: at most three cards, the
same as Home on All projects draws, with "All needs" and "N more", and no
Needs you region when nothing waits; close both Homes with the Ask
temper line; and give Needs you its Decided for you across projects and
the intro's decided sentence. Each Home draws its parts by
`parts::home`, Needs you by `parts::needs`.

Acceptance, as rows of the view's tables, in the browser and the
terminal shells:

- A project's Home with five open cards draws three `temper-need-card`s
  without the project's chip, "2 more" to the project's filtered Needs you,
  and the head "Needs you · 5" with "All needs"; with none open no Needs
  you region and no "Work carries on…" line, the headline saying "Nothing
  needs you."; "Answer in a row", the first-card-and-lines form and the
  phone's pale "Answer" block are gone; the phone's cards hold their
  full-width recommended act and "More".
- The Ask temper line on a project's Home and on `AllHome`: a field named
  "Ask temper" writing through `Act::AskLine`, its words read back from
  `Pages::ask_line`, and "Ask" sending `Act::AskWith` with them; none while
  To begin shows; left out on a phone by style.
- Needs you's Decided for you after the list or the Done column: its head
  "Decided for you · 4" and "for 24 hours, then in each task's history",
  rows from `all_home::decided` ("PROJECT · TITLE · 01:05", the question,
  what it took, "Allowed: WHY", "Overturn" on a live row), filtered by the
  address's project where it has one; the intro's heading ending " Four
  were decided for you." with four rows showing; none of it with none.
- A project's Home in its To begin, its Away form and its everyday form
  holds every row of its existing tables but those this change replaces,
  which change with it.
- The focused tests of `temper-client-view`, `temper-client-shell-terminal`,
  `temper-client-gallery` and `temper-client-world` pass; the gallery's
  `home-*` and `needs-*` rows hold as changed; a journey the replay calls
  stale is recorded again.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Home* (*Needs you*: *The
  cards*, *More*, *On a phone*; *The Ask temper line*), *Home on All
  projects* (*The Ask temper line*), *Needs you* (*Its intro*, *Decided
  for you*) and *A page's parts*.
- The mockups: `temper/docs/design/client/web/whole-story/home.html`
  (`.ask`), `needs.html` (its Decided for you), `phone-home.html`.
- `temper/docs/plan/ui-p2-home/notes/contracts.md`, sections 3 to 5.
- `temper/docs/plan/ui-p2-home/notes/common.md`.

## Owns and may change

- Owns the files above: `pages/home.rs`'s needs section and its phone
  block, `pages/home_cards.rs` (as `all-home-page` left it), `pages/needs.rs`'s
  intro and Decided for you, and their tests; `temper.css`'s `ui: home`,
  `ui: needs` and `ui: phone` parts for them, removing the rules of the
  forms replaced where they stand.
- May change: `src/pages/all_home.rs` (the Ask line's call alone);
  `src/look.rs` (append `temper-ask-line` and `NAMES`' row; remove the
  styles of the forms replaced if no page draws them); `src/words.rs`
  (append; remove the words no page says any more);
  `temper/tools/gallery/src/tests.rs` rows of the `home-*` and `needs-*`
  states this change moves, and `names.txt` regenerated;
  `temper/tests/client` stories that read Home's replaced forms through
  the view, changed with them.
- Options, each pre-approved: if Home's answered lines read wrong over
  the cards, keep them under the head as today and say so; if the
  terminal's Home prints the Ask line as a field it cannot type into,
  leave it out of the terminal's Home (the TUI's composer asks Ask temper
  there) and say so.

## Steps

1. Read `pages/home.rs`'s `needs`, `phone_need` and `main`, and
   `pages/all_home.rs`'s use of `home_cards::cards`.
2. `home.rs`: the needs section by `parts::home(&facts.parts_facts())`,
   through `home_cards::cards(…, across: false, …)`; the head; the empty
   line gone; the Ask line last (a function in `home_cards.rs` both Homes
   call).
3. `needs.rs`: the intro's decided sentence and the Decided for you
   region, by `parts::needs(&parts::needs_facts(root, now))`, its rows'
   drawing shared with `all_home.rs`'s.
4. `temper.css`, the rules from `whole-story.css`'s `.ask`, `.section`,
   `.row` and `.took`.
5. Tests, one row a case of the acceptance; break each new rule once and
   see its row fail. The gallery's moved rows, the journeys' replay and
   `again NAME` for a stale one.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-view -p temper-client-shell-terminal -p
   temper-client-gallery -p temper-client-world --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-client-view -p
   temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-world`. Commit and push.

## Report

What you changed, the forms and words removed, the gallery rows and
journeys moved, the calls you made inside the sketches, the tests and
their counts, each new row's rule and how it failed when broken, and
your implementation and check minutes apart.
