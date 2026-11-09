+++
id = "team-page"
part = "page"
kind = "feature"
needs = ["team-surface"]
files = [
  "temper/crates/temper-client-view/src/pages/team.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/team.rs",
  "temper/crates/temper-client-view/src/tests.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 200
+++

# Team's page: its intro, People, each person's line

## Goal

Draw Team in both shells from `pages::team::shown` and `pages::team::parts`
(`team-surface`'s): its intro, People as a grid of boxes, each person's
name, role and line, the look of the whole story's `team/team.html`.
`team-reads` gives `shown` its reads beside this task; until it lands the
page draws its "Reading" form, and the tests here draw hand-made `Shown`
values.

Acceptance, by `temper/docs/design/client/README.md`, 3.5, *Team* (its
intro, *People*, *A person's line*, *Its parts*, *Its reads*' last
sentence) and `temper/docs/design/client/browser.md`, 2.5, *Team's look*,
and `notes/contracts.md`, T4:

1. **The page** (`temper-client-view/src/pages/team.rs`, `pub(crate) mod
   team` in `pages.rs`): `main(b, root, w, project)` calls `draw(b, w,
   name, &team::shown(root, project))`, `name` the project's name as
   `pages/chats.rs`'s `project_name` finds it; `root.rs` draws `main` for
   `Address::Team { project }`, an `else if let` beside `Changes`'
   branch. `draw` opens `Role::Main` with `look::MAIN` and Team's style.
2. **Its intro:** a paragraph, the kicker "PROJECT · Team"; a level-1
   heading "Who owns what on PROJECT"; while `Shown::Read`, the lede, by
   the number of people: "Just you.", "Two of you.", "Three of you.",
   "Four of you.", then "N of you." in digits; with the document cut, "N
   or more of you." in digits; and while no person listed is `you`, "One
   person.", "Two people.", "Three people.", "Four people.", "N people.",
   "N or more people.".
3. **People:** a section, its head as `pages/settings.rs`'s
   `section_head` draws one, level-2 "People". While `Shown::Reading`,
   nothing under the head. While `Shown::Unreadable`, a muted paragraph
   "This project's people can't be read now.". While `Shown::Read` and
   `parts.people`, a group, Team's grid, holding a group for each person
   in `facts.people`'s order, Team's box: a strong text, "You" for the
   person reading, else their name; a muted small paragraph, their role:
   `Owner` "Owner of PROJECT", `Maintainer` "Maintainer", `Member`
   "Member", `Observer` "Observer", `Policy { role }` "Role N"; and, while
   `waits` is `Some`, a small paragraph, their line.
4. **A person's line,** WHOM "you" for the person reading, else their
   name: total 0, "Nothing waits on WHOM."; one kind, "N KIND waits on
   WHOM." for one and "N KINDS wait on WHOM." for more, KIND "question",
   "review", "proposal", "held task", "person task"; kinds mixed, "N
   things wait on WHOM."; `cut`, the number followed by "+", "2+
   questions wait on Sam.", "1+ question waits on Sam." for one.
   The triager's line is not drawn: `parts.triager` is never true yet, and
   this task leaves no code for it.
5. **Words** in `words.rs`, `Word::Team*`, ids 4900 to 4949 alone, each
   in its four places beside the Builders words, in both shells' words
   where a shell differs, as the file keeps them; the counted declarations
   (`PHRASES`, `SEGMENTS`, `ENGLISH_BYTES`) regenerated as
   `tests/accounts.rs`'s `english_phrase_declarations_match_the_vocabulary`
   says.
6. **The look:** tokens of Team's own appended to `look.rs`'s `NAMES`:
   `temper-team` (the main), `temper-team-intro`, `temper-team-people`
   (the grid), `temper-team-person` (a box), `temper-team-role`,
   `temper-team-line`; `temper.css`'s rules for them under a new marker
   `/* ==== ui: team ==== */`, just before `/* ==== ui: settings ==== */`,
   as browser.md 2.5's *Team's look* has them from `whole-story.css`
   (`.col.wide`, `.intro`, `.kicker`, `.headline`, `.lede`, `.shead`,
   `.grid3`, `.box`, `.small`, `.muted`), in the project frame's `--gf-*`
   variables as Goals' rules are, the grid one column at 760 px and
   under.
7. **Tests:** a new `temper-client-view/src/tests/team.rs` (`mod team;` in
   `tests.rs`), one table test `team_draws_what_it_shows`, each row a
   hand-made `Shown` drawn by `draw` and read by role and name: `Reading`
   (kicker, heading, "People", no box); `Unreadable` (its line); two
   people, the reader an owner with two questions and a review ("3 things
   wait on you.") and Sam a member with one question ("1 question waits
   on Sam."), "Two of you."; Sam's waits `None` (no line); a cut count
   ("2+ questions wait on Sam."); one person, nothing waiting ("Just
   you.", "Nothing waits on you."); a cut document of five ("5 or more of
   you."); two people, the reader not among them ("Two people."); each
   role's words; one row drawn in the terminal's words, if
   the file's other tests draw both shells. `tests/look.rs`'s CSS and
   names tests pass with the new tokens.
8. **Status:** delete, whole, the **Not yet** entries "3.5, *Team*: the
   view draws no Team page" in `temper/docs/design/client/README.status.md`
   and "2.5, *Team's look*: `temper.css` has no rule of Team's." in
   `temper/docs/design/client/browser.status.md`. No other design line.

## Inputs

- `temper/docs/plan/ui-p4a-team/notes/common.md`;
  `temper/docs/plan/ui-p4a-team/notes/contracts.md`, T2 and T4.
- `temper/docs/design/client/README.md`, 3.5, *Team*, whole but *In the
  rail*.
- `temper/docs/design/client/browser.md`, 2.5, *Team's look*, *Goals'
  look* (the `--gf-*` scoping) and *The project's frame*.
- The look: `temper/docs/design/client/web/whole-story/team/team.html`,
  its `header.intro` and its People `section`; `whole-story.css`, lines
  48 to 60 and 95 to 98.
- The code: `pages/chats.rs` (`main`, `intro`, `project_name`),
  `pages/settings.rs` (`section_head`), `cards.rs` (`text`),
  `pages/home.rs` (spelled numbers: `HomeTwo` to `HomeFour`), `words.rs`
  (four places a word lives; `english`), `look.rs` (`NAMES`, a style
  constant), `temper.css` (Goals' rules under its marker),
  `tests/accounts.rs` (the vocabulary's declarations).

## Owns and may change

Owns the files in the header. Not `frame.rs` (the rail is `team-rail`'s,
after ui-p1-rail), not `pages/home.rs` (ui-p2-home's): reuse its spelled
numbers' words only if they are `pub(crate)` already, else Team's own
words for "Two", "Three" and "Four".

Options, each decided:

- **The terminal prints a group's children as lines:** draw no group the
  terminal can't print; a box's three texts print as three lines.
- **A shell's words differ** (a "Shell::Terminal" arm): Team's words are
  the same in both shells unless a sibling page's differ for the same
  reason.
- **`NAMES` meets another plan's tokens on a catch-up:** renumber Team's
  (`notes/common.md`).
- **A golden or print of the terminal or the journeys changes:** none
  should, since no fixture or journey opens Team; one that does is a red,
  not stale.

## Steps

1. Read the Inputs. `draw`'s intro and People (1 to 4) with the words (5);
   the test's rows (7); bite: leave the reader where the document puts
   them, see the two people's row fail, restore; bite: say "wait" for one,
   see its row fail, restore.
2. The look (6); `heavy cargo nextest run -p temper-client-view --lib -E
   'test(/look/)'`.
3. The status entries (8).
4. Runs through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-view`, `-p temper-client-shell-web`, `-p
   temper-client-shell-terminal`, `-p temper-client-gallery`; `cargo check
   -p temper-client-world --tests`; clippy on every package changed; then
   as `notes/common.md` says before the push. Commit, saying that Team
   draws its people and what waits on each, and push.

## Report

The implementer's report: the page's tree as drawn, its words with their
ids, its tokens and rules, the rows and the bites with their failures, the
runs' counts, the new test's CPU time, and what is left.
