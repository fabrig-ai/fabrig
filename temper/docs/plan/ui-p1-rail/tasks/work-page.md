+++
id = "work-page"
part = "work"
kind = "feature"
needs = ["addresses", "work-list"]
files = ["temper/crates/temper-client-domain/src/pages/work.rs", "temper/crates/temper-client-domain/src/tests/pages/work.rs", "temper/crates/temper-client-domain/src/pages/changes.rs", "temper/crates/temper-client-view/src/pages/work.rs", "temper/crates/temper-client-view/src/tests/work.rs", "temper/crates/temper-client-view/src/pages/goals.rs", "temper/crates/temper-client-view/src/pages/changes.rs", "temper/crates/temper-client-view/src/tests/goals.rs", "temper/crates/temper-client-view/src/tests/changes.rs"]
effort = "high"
minutes = 270
+++

# Work's page: its head, its filters, its one list; Goals and Changes under it

## Goal

Draw Work as `temper/docs/design/client/README.md`, 3.5, *Work*, has it, from the
reads `addresses` opens on Work's five addresses and the places
`work::place` gives: its head (kicker, counted heading, lede, the landing's line),
its filters as chips, and under All the sections In flight, Drafts and Landed
this week, with a goal's, a change's, a chat's and a draft's cards and the landed
rows; under Drafts, the drafts. Goals and Changes become Work's filters: Work's
head and filters take their kickers' place, their headings at level 2. Kind 7's
Goals and Changes switches hide those two filters, as overrides. Both shells.

Acceptance:

1. `temper_client_domain::pages::work` gains C7 (`Overrides`, `Shown`, `shown`,
   `overrides`, `facts`), and `facts` gathers C4's facts from the goals', changes'
   and chats' pages: each goal row read (live and ended), each queued and landed
   change of the changes' document with where it belongs (its task's `goal`, else
   its chat root, else nothing, and nothing while unread), each of the person's
   chats read with whether it has a document; `None` until the three have answered.
2. `Address::Work` draws Work's page: its head and filters on every Work filter,
   then All's sections or the drafts, each card and row as 3.5 words them, each a
   link to its page. A section with nothing isn't drawn; with nothing at all,
   "Nothing in flight yet."; nothing claimed before the reads answer.
3. The filters: "All", then "Drafts · N", "Goals · N", "Changes · N" and "Chats ·
   N" by `work::shown`, the current one always and current; "Goals" to `Goals {
   project, filter: All }`, "Changes" to `Changes { project }`, "Chats" to `Chats {
   filter: All }`.
4. `Goals` and `Changes` draw Work's head and filters first, their filter current,
   then their own parts as 3.5, *Goals* and *Changes*, say, their headings at level
   2 and Changes' lede without the landing's line.
5. Kind 7's `goals` and `changes`, read as Hidden, hide their filters; unread or
   refused, nothing hides; what they hide stays under All.
6. In the terminal, Work prints as 3.5 says: its head, the filters numbered, each
   card a numbered link with its from line and headline, its rows under it.
7. The focused tests of `temper-client-domain`, `temper-client-view` and
   `temper-client-gallery` pass, clippy clean; the README.status.md entry "**Not
   yet:** 3.5, *Work*, *Goals* and *Changes*" is gone.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`; `contracts.md`, C2, C4, C6 and C7.
- `temper/docs/design/client/README.md`, 3.5: *Work*, whole; *Goals* (its intro,
  its rows, *A goal's state*, its headline); *Changes* (its order of parts, *The
  landing's line*, *Where a change came from*); *The Chats page*, a row's "writes
  NAME"; 5.8's header for "31 of 40 spent"; 5.12 for the clock form.
- `temper/docs/design/client/browser.md`, 2.5, *Work's look*.
- The look: `temper/docs/design/client/web/whole-story/work.html`,
  `pair/work.html`, `team/work.html`, and `whole-story.css`'s `.intro`, `.chips`,
  `.chip.sig`, `.shead`, `.card`, `.from`, `.who.a`, `.who.c`, `.who.p`, `.q`,
  `.rows`, `.row`, `.byline`.
- The code: `temper-client-domain`'s `pages/work.rs` (`place`, from `work-list`),
  `pages/goals.rs` (`Goals::live`, `ended`, `report`, `state`), `pages/changes.rs`
  (`Changes::state`, `from`), `pages/chats.rs` (`Chats::live`, `documents`,
  `known`, `documents_known`); `temper-client-view`'s `pages/goals.rs` (`main`,
  `row`, `ways`, `state_word`), `pages/changes.rs` (`main`, `landing_line`,
  `LandingFacts`, `ordinal`), `pages/chats.rs` (`row`, how it merges live and closed
  rows), `root.rs`'s dispatch.

## Owns and may change

Owns: `temper-client-domain`'s `pages/work.rs` (C7 beside `work-list`'s C4) and
`src/tests/pages/work.rs` (new rows and tests beside `work-list`'s); one public
accessor in `pages/changes.rs` giving a row's goal task number where `from` gives
its title; `temper-client-view`'s new `pages/work.rs` and `src/tests/work.rs`, and
`pages/goals.rs`, `pages/changes.rs` and their tests.

Reader and neighbour files, and what may change there:

- `temper-client-view`'s `root.rs`: one `else if let Address::Work { project, ..
  }` branch, after `Goals`' branch, calling `crate::pages::work::main`;
  `pages.rs`: `pub(crate) mod work;`; `src/tests.rs`: `mod work;`.
- `words.rs`: Work's words, keys 4650 to 4749 (C6), beside the Goals words; reuse
  Goals' and Changes' words where they say the same (the ordinals, "checking",
  number words).
- `look.rs`: Work's styles in slots 650 to 671 (C6).
- `temper.css`: Work's rules under a new marker `/* ==== ui: work ==== */`, just
  before `/* ==== ui: goals ==== */`; Goals' and Changes' kicker rules that go,
  where they stand.
- Tests that assert Goals' or Changes' intro: `temper-client-view`'s
  `tests/goals.rs` (`the_headline_counts_every_row_in_every_form`,
  `the_intro_waits_for_me_and_the_live_watch`, `the_lede_names_the_projects_ways`)
  and `tests/changes.rs` (`headings_and_ledes_name_counts_and_the_one_branch_name_across_repositories`,
  `loading_ended_and_empty_keep_their_own_words_and_the_landed_rule`); the gallery's
  rows for `goals-*`, `changes*` and `home-with-goals` where they read a kicker or a
  level-1 heading, and `each_settings_switch_off_keeps_its_value_and_changes_its_navigation`'s
  Goals and Changes lines (the switches now hide Work's filters). Rows alone; new
  states are `gallery`'s.

Not yours, left stale and listed under `## Left`: the terminal goldens
(`regenerate`); the client world's stories (`readers`). Not yours at all: the rail
(`rail`, beside you, owns `frame.rs`), Settings and Deployment (`deployment`, beside
you, owns `pages/settings.rs`).

Pre-approved options:

- **If a goal's way's name needs `me`:** use `goals::ways(me, project, words)` as
  Goals' lede does; with none, leave the part out.
- **If a card's facts are not yet read** (a change's title before the changes'
  document lists it, a goal's report): draw what is read and leave the rest out,
  claiming nothing, as Goals' rows do.
- **If drawing Work's head on Goals and Changes changes what their tests find by
  role and name:** change those lines to the new head, keeping each test's other
  checks.
- **If `place`'s `Facts` lack a fact a card needs** (a chat's start time): read it
  from the page state by the card's task number in the view; never change C4.

## Steps

1. **Domain:** C7 in `pages/work.rs`; `facts` from the three pages; the accessor in
   `pages/changes.rs`. Tests in `tests/pages/work.rs`: one table of `facts` over a
   domain fixture with Work open (the fixtures `tests/pages/goals.rs` and
   `tests/pages/changes.rs` deliver goals and a changes panel with): unread is
   `None`; each kind gathered; a change's goal, its chat, nothing, and unread; a
   chat with a document is a draft. One table of `shown` and one of `overrides`
   (unread, refused, Shown, Hidden). Break one rule by hand (a refused kind 7 hides)
   and see its row fail.
2. **View, Work's head and filters,** in `pages/work.rs`, as `pub(crate) fn
   head(..)` that `goals.rs` and `changes.rs` call too; the landing's line through
   `changes::landing_line` and `landing_parts`.
3. **View, All and Drafts:** the sections and the four cards, by `place` and the
   page states, each card's words by 3.5. `main` for `Address::Work`.
4. **Goals and Changes** under Work's head, as acceptance 4.
5. **Tests,** `tests/work.rs`: a table of the heading's forms (zero, one, words,
   digits, each part left out); the filters by counts and overrides, and their
   addresses; All with the whole story's small project (each card's from line,
   headline, rows and links; the sections' heads and counts); Drafts; nothing yet;
   unanswered reads claiming nothing; the terminal's print of one card. Break two
   rules by hand (a section drawn empty; an override hiding a filter while unread)
   and see their rows fail; restore.
6. **Readers** as Owns says, then **check** as `notes/common.md` says, for
   `temper-client-domain`, `temper-client-view` and `temper-client-gallery`, and
   `heavy cargo check --tests` for `temper-client-shell-terminal`,
   `temper-client-world` and `temper-client-tui`. Remove the status entry. Commit
   and push: the message says Work lists what exists, and Goals and Changes are its
   filters.

## Report

Follow the implementer's text. Also give the rules you broke and the failures you
saw; the words and keys added; the style slots taken; each test's CPU time; and,
under `## Left`, each stale golden and story by file and line.

## Rebase (2026-11-08, step merge, foreman ui-p1-rail/work-page/foreman-1)

Resolve words.rs by retaining both Work and Deployment words and combining their vocabulary counts.
The merge conflicts in `temper/crates/temper-client-view/src/words.rs`.
Rebase onto the landing trunk, which includes Deployment commit `bf66c99c4d782ebbdcf7a8bf8d14fbd817dac8ed` through integration merge `14e9a54f83`.
Both tasks changed the shared vocabulary declarations from the same base; preserve both additions.
For these two changes combined, set `PHRASES = 2159`, `SEGMENTS = 3818`, `ENGLISH_BYTES = 52660`, and `ALL: [Word; 1477]`.
Keep all 28 Work variants, their `ALL` entries, IDs and English phrases; retain Deployment’s 39 additions too.
Preserve C6’s separate keys: Work 4650–4749 and Deployment 4750–4799.
Keep Work’s `ChangesLede` edit and Deployment’s Settings/Workers wording edits.
Verify the combined declarations with the repository’s vocabulary checks and run the focused `temper-client-view` tests through heavy.
The log reports no other conflicting path; retain the automatically merged reader and gallery changes.
