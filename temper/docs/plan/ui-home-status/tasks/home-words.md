+++
id = "home-words"
part = "home-words"
kind = "feature"
needs = ["home-domain"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/changes.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/tests/changes.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# Home's kicker, headline, foot, and Chats' sentence

## Goal

Home on any project says what changed: `temper/docs/design/client/README.md`, 3.5, the kicker, the headline's landed sentence, the foot and Chats' sentence, from `Facts.status`, in both shells. This alone is worth landing. The lede and the byline are `home-brief`'s.

Acceptance:

- **The intro:** `notes/contracts.md`'s `Intro` and `intro_words`, the one function that composes it. To begin's own headline and lede stay, reached through it.
- **The kicker:** the day and time, then " · you last looked SPAN ago" with a `previous` over a minute old: `TimeForm::Looked`, "a minute", "2 minutes", "an hour", "9 hours", "a day", "3 days", rounded down. At exactly a minute old, nothing; the minute's redraw already there moves it.
- **The headline:** "One change landed since you last looked." or "N changes landed since you last looked.", from `Facts::landed_since`; nothing for 0; then a space and the needs sentence; as now until `Status::Shown`.
- **The foot,** `temper-home-facts`, a paragraph after the last section: "Landing: ", `landing_line` with `notes/contracts.md`'s `LandingFacts`, then " · " and "Changes", a link to `Address::Changes { project }`; drawn at `Status::Shown` and not while To begin shows. " · main red" takes the signal's colour.
- **One function serves both:** `pages::changes`' page calls the same `landing_line`, and its tests and goldens don't move.
- **Chats' sentence,** `.facts`'s face, after Chats' rows with no goal in the project (`root.has_goals(project)`), once the chats are delivered.
- **Gallery,** from fixtures: `home-computed`, `home-last-looked-none`, `home-landing-none`, `home-landing-red`, `home-chats-sentence`, in `names.txt` and `samples.rs`, each giving its `w::ProjectStatus` where its `Watch` arm is, and in `tests.rs`'s holds.
- **The terminal:** the kicker and headline read as one run of words, the foot a paragraph. `each_fixture_page_prints_as_its_golden_file` is reprinted where the meaning holds; a page with no status document doesn't move.
- `temper/docs/design/client/README.status.md`: *Not yet*'s entry names only what's left: lede, byline, Goals.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *Home on any project*; *Changes*, the landing's line; 5.12's times.
- The mock: `home` in `temper/docs/design/client/greenfield/app.js`, its last lines for the foot; `.facts` and `.kicker` in `mock.css`.
- `temper/crates/temper-client-view/src/pages/changes.rs`, `landing_line` and its caller at `:176`, for the parts' drawing.

## Owns and may change

Owns, as `files` lists them, and the terminal's goldens by regenerating. May change the status line above.

Options, each pre-approved:

- **`LandingFacts`' field types** bend to the words as they are; its name, `landing_line`'s, and one function serving both don't. A helper that draws `LinePart`s moves to where both can call it.
- **`temper.css`:** `.temper-home-facts` and `.temper-home-byline` have rules from an earlier look; move them under `/* ==== ui: home ==== */`, make the foot's selector reach Chats' sentence, and add no more than the mock asks.
- **"Turn on drafts and goals":** if `Settings` has no such switch on your base (`rg 'kind 7|drafts and goals' temper/crates/temper-client-view/src/pages/settings.rs`), don't draw it, and say so; if it has, draw it, to `Address::Settings { project }`.
- **A flash of the sentence** before `has_goals` is told is acceptable; it is one frame.
- **`look.rs`'s tokens:** the next free, and regenerate what counts them.
- **The phrasebook's bound** in `limits.rs` moves with your words.

## Steps

1. **`words.rs`:** the words, `TimeForm::Looked`.
2. **`pages/changes.rs`:** `LandingFacts`, the panel's facts made from it, its tests green and unmoved.
3. **`pages/home.rs`:** `intro_words`, `foot`, Chats' sentence.
4. **`temper.css`, gallery, goldens.**
5. **Tests,** `tests/home.rs`, each state from a document:
   `the_kicker_says_when_you_last_looked`, its table of spans; `no_previous_says_nothing`; `under_a_minute_says_nothing`; `the_headline_counts_what_landed`, 0, 1, many; `the_headline_is_as_now_until_status_is_shown`; `the_foot_says_what_is_queued`, its three shapes and red; `the_foot_links_to_changes`; `no_foot_while_to_begin_shows`; `chats_says_its_sentence_with_no_goal`; `chats_has_no_sentence_with_a_goal`.
6. **Check,** the implementer's text, with `temper-client-shell-terminal`, `temper-client-shell-web` and `temper-client-gallery`, then push.

## Report

As `notes/common.md`, Your report, with which goldens moved.
