+++
id = "goal-asks"
part = "goal-asks"
kind = "feature"
needs = ["settings-away", "decided-charter"]
files = [
  "temper/crates/temper-client-domain/src/pages/start_goal.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/goal.rs",
  "temper/crates/temper-client-domain/src/tests/pages/start_goal.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/start_goal.rs",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/start_goal.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# "When it needs you" on Start goal, and "How it runs" on a goal's page

## Goal

Fabrig/ui's ruling of 2026-11-05: this plan builds "When it needs you" on the one Start goal sheet, with `Asks: always` and its charter sentence, and changes the goal page's "How it runs" word. The target:
`temper/docs/design/client/greenfield/app.js`, lines 483 and 643 to 646. This spec holds the design, since 5.14 is on the trunk only since `client-base`.

Acceptance:

- **The sheet,** `pages/start_goal.rs`, a choice group with the legend "When it needs you", as `ways` is drawn: "Follow the away switch", small "Asks while you’re here; while you’re away, decides within the project's limits and tells you.", chosen by default; "Always ask", small "Waits for you on every question it can’t answer from the plan."
- **Its act** in the domain: `Act::StartAsks { always: bool }`, as `Act::StartWay`.
- **`Asks: always`:** `words()` in the domain's `pages/start_goal.rs` gains that line, exactly, where "Always ask" is chosen, and none otherwise. The root's rail and the charter's sentence are already built: `decided-tool`, `decided-charter`.
- **A goal's page,** `Word::GoalAlwaysAsks` stays for a goal whose words hold the line, " · always asks you"; every other goal says " · follows the away switch", a new `Word::GoalFollowsAway`. One function reads the line, in the domain's `pages/goal.rs`.
- **Design,** yours to write here, in the words of this spec: `client/README.md`, 5.14: "**Not drawn:**" loses "When it needs you", and the section says the choice as above; 5.8's *How it runs* says both words; section 11's lines that say these wait for the switch go. `README.status.md` moves with them. Keep each section's number.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md` (*The goal's words*).
- `temper/docs/design/client/README.md`, 5.8, 5.14, section 11.
- The models: `temper-client-view/src/pages/start_goal.rs`' `ways`, `temper-client-domain/src/pages/start_goal.rs`' `words`.

## Owns and may change

Owns, as `files` lists them. May also change: `tests/fixtures.rs`; the gallery's `start-goal*` and `goal-*` samples where their words move; the terminal's goldens, regenerated; `temper/docs/design/client/browser.md`'s row for a state whose words move.

Options, each pre-approved:

- **A goal page with no words delivered:** it says neither word, its way alone, until they are.
- **`ui-chat-draft`'s sheet on a chat's page,** if it has landed: it builds the same `words()`, so it takes the choice by the same act where it draws `ways`; otherwise leave it, and say so.

## Steps

1. The domain's act and words. `src/tests/pages/start_goal.rs`: `always_adds_the_line`, `follow_adds_none`, `the_default_follows`. `src/tests/pages/goal.rs`: `the_line_reads_always`, `no_line_follows_the_switch`, `a_line_inside_a_sentence_doesnt_count`.
2. The sheet and the page. `tests/start_goal.rs`, `tests/goal.rs`.
3. The design and the gallery.
4. The check of the implementer's text, `-p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal`, citations. Push.

## Report

As `notes/common.md`, Your report.
