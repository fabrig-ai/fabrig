+++
id = "home-brief"
part = "home-brief"
kind = "feature"
needs = ["home-goals"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "medium"
minutes = 45
+++

# Home's lede and byline, and the brief's sentence

## Goal

Where the status agent has something new, Home says its words: `temper/docs/design/client/README.md`, 3.5, the headline's brief and *the lede and the byline*. It reads `Facts::brief`, whichever engine wrote it: the agent's run isn't needed.

Acceptance:

- `intro_words` gives the brief's `sentence` in place of the computed landed sentence, its `lede`, and a byline "Written by the status agent at 07:42", `at` by `TimeForm::Clock`, wherever `Facts::brief` holds; none otherwise. No computed prose.
- `temper-home-lede`, then `temper-home-byline`, 12 px in `--gf-faint`, as the mock's.
- An empty `lede` draws no lede, and the byline stays.
- The terminal: kicker, headline and lede read as one run of words.
- Gallery: `home-morning`, with everything: a brief above `seen`, needs, chats, queued, a previous, at the mock's words: "Tuesday, 08:04 · you last looked 9 hours ago", "41 changes landed overnight. Three things need you.", "first-run-projects is six of nine tasks in. Its store task hit the wasm size limit twice and was split in two.", "Written by the status agent at 07:42", "Landing: 3 queued · last landed 6 min ago · Changes".
- `client/README.status.md`'s *Not yet* loses this plan's entry, if `home-words` and `home-goals` left nothing else.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5.
- The mock: `intro` in `home`, `app.js`; `.lede`, `.byline` in `mock.css`.

## Owns and may change

Owns, as `files` lists them, and the terminal's goldens by regenerating.

Options, each pre-approved:

- **`home-morning`'s project** may be a sample's own; its clock is the gallery's, so Tuesday 08:04 may need that sample's `NOW`. If a sample can't set it, keep the gallery's clock, and say so.
- **The mock's "main green at 07:36"** is dropped by the request: don't draw it.

## Steps

1. **`intro_words` and the draw.**
2. **`temper.css`,** under the marker, only what `.byline` lacks.
3. **Gallery, goldens.**
4. **Tests,** `tests/home.rs`: `a_brief_above_seen_says_its_words`, `a_brief_at_seen_is_the_computed_words`, `no_brief_no_lede_no_byline`, `an_empty_lede_keeps_the_byline`, `the_byline_says_its_clock`.
5. **Check,** the implementer's text, with `temper-client-shell-terminal` and `temper-client-gallery`, then push.

## Report

As `notes/common.md`, Your report.

## Rebase (2026-11-05, step merge, foreman ui-home-status/home-brief/foreman-1)

Rebase Home’s brief onto integration, preserving the newer Needs session and Goals behavior.
The merge failed in five paths; this is a branch conflict, not an infrastructure failure.
Reconcile with `ebf85a0159` (main catch-up and Needs integration) and `7d6534e5c1` (rebased Home goals).
- `temper/crates/temper-client-view/src/pages/home.rs`: keep integration’s `number_word`, Needs session rendering, row bound, confirmation and Goals rendering. Apply the branch’s `Facts::brief()` sentence, optional lede, clock byline and terminal composition around them. Drop `HomeNeedsThree`; integration’s `HomeThree` already supplies “Three.”
- `temper/crates/temper-client-view/src/words.rs`: preserve integration’s vocabulary and IDs. The branch’s `HomeByline = 3289` collides with `NeedsLast`; assign a free ID. Add its enum, `ALL`, English phrase and capacity accounting. Retain the branch’s `Weekday` formatting and comma in `HomeDayTime`; recompute combined phrase, segment and word counts.
- `temper/crates/temper-client-view/src/tests/home.rs`: retain integration’s standing Needs watch setup, delivery, fixture limits, expanded number tests and session action assertions. Add the branch’s brief tests and weekday expectations without restoring the older fixture setup.
- `temper/tools/gallery/src/samples.rs`: retain integration’s Needs samples and Goals fixture. Add `home-morning` and `morning_status`; include it in the extracted `home_needs` helper and everyday/chat selections. Propagate the branch’s sample-aware `home_since(name, minutes)` into integration’s new helper call sites.
- Preserve both sides’ gallery names and assertions; reconcile `samples::N` with the combined sample list. Keep the authorized `worlds.rs` clock override for Tuesday 08:04.
- `temper/docs/design/client/README.status.md`: remove only the Home lede/byline gap, preserving integration’s other status entries.
Check the automatically merged domain, frame and gallery files for consistency. Run the focused Home view, terminal shell and gallery checks through `heavy` with `TEMPER_BUNDLE=none`; regenerate affected terminal goldens using their existing tool and report them.
