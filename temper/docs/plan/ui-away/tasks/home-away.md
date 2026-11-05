+++
id = "home-away"
part = "home-away"
kind = "feature"
needs = ["away-switch", "home-decided"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# Home's away form

## Goal

With any decision showing, Home reads as the mock's *Back from away*: `temper/docs/design/client/README.md`, 3.5 (*Away*, *Home, while a decision shows*). The
target: `temper/docs/design/client/greenfield/app.js`, lines 363 to 385 (the scenario `away`); `mock.css`, 125 to 131 and 459 to 461; the words in `data.js`.

Acceptance:

- **The intro,** in `intro_words`, the one function: the kicker and the headline as 3.5 says, the needs sentence's place taken, `Facts.away` read as the rest of `Facts` is. Lede and byline
  stand. `times.next` asks to wake at `expires`.
- **"Decided for you · N,"** a `Role::Region`, the first after the intro: `HOME_HEAD`'s head, a level-2 heading, and "Limits" at the right, `Act::Away(Limits)`.
- **`DECIDED`, `DECISION`, `DECISION_TOOK`, `DECISION_WHY`, `DECISION_LATE`, `DECISION_OVERTURNED`** (`notes/contracts.md`, *The view*), a row per `Facts.away.rows`, drawn as 3.5
  says: "TITLE · 01:05", "Task N" where its title is unread; the question; took; why. On the right, by the first that holds: `Done`, "landed 03:12 · a new goal can undo it"; `Ended`, "ended 03:12 · a
  new goal can undo it"; overturned, "Overturned: it asks you again". `Live` and not overturned is `overturn`'s button: nothing here.
- **Needs' head:** "Still needs you · 2", "Still needs you" with none, while `Facts.away` is some; the rest of Needs as it stands.
- **`temper.css`,** under `/* ==== ui: away ==== */`: the mock's rules by those tokens, and its phone rules: `.why` hidden, padding 12 0, `.late` max-width 8em.
- **Gallery:** `home-away`, `home-away-one`, as `temper/docs/design/client/browser.md`, 7.2. Both beside the mock at 1440, 1000 and 420 wide (`notes/common.md`, *Render proofs*).
- **The terminal** reads each row as one run of words, its "Limits" numbered.
- `browser.status.md` loses both.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5 (*Away*, *Home on any project*), 5.12; `browser.md`, 7.2.
- The models: `pages/home.rs`' `intro_words`, `needs`, `goals`; `number_word`.

## Owns and may change

Owns, as `files` lists them. May also change: `tests/fixtures.rs`; the terminal's goldens, regenerated.

Options, each pre-approved:

- **The landed sentence's number:** digits, "57 changes landed.", "One change landed." for 1; the decided sentence's in number words to "Nine", digits after, as the needs' is.
- **`times.now` ahead of a row's `at`, or behind it:** 5.12's *clock* says what it says.
- **New `Word`s:** the four lines each; `limits.rs`' pins move with them.
- **Home's title** (`browser.md`, 7.1, a `home-*` state's title is its headline): whatever `intro_words` gives.

## Steps

1. `Word`s, tokens, `tests/look.rs`.
2. `intro_words`. `tests/home.rs`: `the_kicker_reads_since_to_now`, `the_kicker_reads_since_to_its_end_once_off`, `the_kicker_alone_with_no_row`, `the_headline_says_landed_then_decided`, `none_for_no_changes`,
   `one_thing_was_decided`, `lede_and_byline_stand`, `no_form_with_no_row_showing`.
3. The section: `the_section_comes_first_after_the_intro`, `its_count_is_every_row_showing`, `limits_goes_to_settings`, `a_row_says_title_and_clock`, `an_unread_task_says_task_n`, `done_says_landed`,
   `ended_says_ended`, `overturned_says_it_asks_again_and_strikes_took`, `ended_and_overturned_says_landed_and_strikes_took`, `needs_head_says_still`.
4. CSS, then the gallery. Compare with the mock; the words on the page are the mock's.
5. The check of the implementer's text, `-p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-shell-web`. Push.

## Report

As `notes/common.md`, Your report: the render's check, on which builder.

## Rebase (2026-11-05, step merge, foreman ui-away/home-away/foreman-1)

resolve both merge conflicts by combining phrase counts and preserving the trunk’s status corrections.
Rebase onto the landing trunk containing `5cb4f0e8f2` (“Merge main to retain adoption phrases alongside Away UI”).
- `temper/crates/temper-client-view/src/words.rs`: retain all thirteen Home-away words, their IDs, English forms and `ALL` entries. Combine the trunk’s +2 phrases/+6 segments with this task’s +13/+29: `PHRASES = 1697`, `SEGMENTS = 2957`, and `ALL` length 1098.
- `temper/docs/design/client/browser.status.md`: preserve the corrected Start goal departure from main’s `bf111565a8`, including `Chats { filter: All }` and the unbuilt Drafts filter. Remove only `home-away` and `home-away-one` from the 7.2 gaps; retain `home-away-overturned`, `settings-away`, and the 7.1 title gap.
Validate the combined phrase declarations and run the task’s required package checks through heavy with `TEMPER_BUNDLE=none`.
