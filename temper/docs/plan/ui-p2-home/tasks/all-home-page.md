+++
id = "all-home-page"
part = "across"
kind = "feature"
needs = ["all-home-surface", "needs-list"]
files = ["temper/crates/temper-client-view/src/pages/all_home.rs", "temper/crates/temper-client-view/src/pages/home_cards.rs", "temper/crates/temper-client-view/src/tests/all_home.rs", "temper/crates/temper-client-view/src/tests/home_cards.rs", "temper/crates/temper-client-shell-web/temper.css"]
effort = "high"
minutes = 200
+++

## Goal

Draw Home on All projects as the whole story's `home.html` and
`phone-home.html` draw it, from `all_home::Facts`: its intro (the kicker,
the landed and needs headline, the briefs joined, the byline), Needs you
as at most three cards across projects, Decided for you across projects,
Projects with their health chips, and the phone's form. Home's cards are
one function, which the next task gives a project's Home too. Needs
you's Done "Home" leads here with two projects or more.

Acceptance, as rows of the view's tables (`src/tests/all_home.rs`,
`src/tests/home_cards.rs`), in the browser and the terminal shells:

- `draw` over hand-made `all_home::Facts`: the kicker with "you last
  looked 9 hours ago" and " · away 23:10 to 08:00", neither without its
  fact; each `Landed` form's sentence ("31 changes landed across four
  projects.", "3 changes landed in vsr.", "One change landed in vsr.",
  "The last change landed 2 hours ago.", none) followed by the needs
  sentence; the lede joining up to four "PROJECT: sentence" runs, the
  name in the strong weight, and the byline "Written by the status agent at
  07:52"; no lede or byline with no brief; Decided for you's region, its
  head "Decided for you · 4" and "What reaches you", at most seven rows of
  "PROJECT · TITLE · 01:05", the question, what it took, why, and
  "Overturn" on a live row bound to `away::Act::Overturn` with the row's
  project; Projects' region with a row a project, its name a link to its
  `Home`, "main" `temper-chip-good`, "main red" `temper-chip-signal`, no
  chip while unknown, and no Projects with one project; every part left
  out while its `parts::home` rule fails.
- `home_cards::cards`: three cards of five open, oldest first, each a
  `temper-need-card` with its from line (the project's chip with
  `across`, none without), its question, its acts (a question's choices,
  the recommended one first and primary, then "Tell it something else"
  bound to `needs::Act::Open`; a proposal's Accept and Reject; a held
  task's Release and Leave held; an approval's "Review on GitHub ↗" then
  "Tell it something else"); "2 more" linking to the unfiltered Needs you;
  the phone's form, the recommended act full-width and "More".
- Needs you's Done "Home" opens `AllHome` with two projects in `me`, the
  open project's `Home` with one.
- The rail's "Home" is current on `AllHome`; the tree holds one main.
- The focused tests of `temper-client-view`, `temper-client-shell-terminal`
  and `temper-client-gallery` pass; a journey the replay calls stale is
  recorded again.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Home on All projects*
  (all of it but *Its reads* and *The Ask temper line*, which are
  `all-home-reads`' and `home-cards`'), *Home* (*The cards*, *More*, *On a
  phone*; *Home on any project* for the kicker's SPAN, the needs sentence,
  the byline), *Away* (*Home, while a decision shows*: a decision's row),
  *Needs you* (*Done*) and *A page's parts*.
- The mockups: `temper/docs/design/client/web/whole-story/home.html`,
  `phone-home.html`, and `whole-story.css`'s `.intro`, `.kicker`,
  `.headline`, `.lede`, `.byline`, `.section`, `.shead`, `.card.need`,
  `.from`, `.chip`, `.chip.good`, `.chip.sig`, `.choices`, `.tbl`, `.row`,
  `.took`, `.phone`; their words, parts, order and look bind.
- `temper/docs/plan/ui-p2-home/notes/contracts.md`, sections 2, 3 and 5.
- `temper/docs/plan/ui-p2-home/notes/common.md`.

## Owns and may change

- Owns `temper-client-view/src/pages/all_home.rs` and
  `src/pages/home_cards.rs` (new) and their tests (new), and the
  `/* ==== ui: home ==== */` and `/* ==== ui: phone ==== */` parts of
  `temper.css` for these parts.
- May change: `src/pages.rs` (two `mod` lines); `src/root.rs` (the
  `AllHome` arm `all-home-surface` left draws `pages::all_home::main`
  instead of an empty main); `src/look.rs` (append `temper-need-card`,
  `temper-need-from`, `temper-need-acts`, `temper-need-more`,
  `temper-chip-good`, `temper-chip-signal`, `temper-projects`,
  `temper-decided-head`, and `NAMES`' rows; `temper-chip` is
  `needs-list`'s); `src/words.rs` (append this page's words);
  `src/pages/home.rs` (make `pub(crate)` the helpers this page reuses:
  the intro's kicker and needs sentence, `decision`, `number_word`; no
  change to what a project's Home draws); `src/pages/needs.rs` (Done's
  "Home" address alone); `src/frame.rs` (the rail's "Home" current on
  `AllHome`, if `all-home-surface` left it otherwise).
- Options, each pre-approved: if a decision's row on a project's Home
  draws from `home::AwayForm` and this page's from `all_home::Decided`,
  share the row's drawing through one function taking a `DecisionLine`
  and an optional project name; if "What reaches you" has no open project
  to lead to, leave it out and say so; if a card's acts for a proposal or
  a held task need the confirmation sheet's state from the page, draw them
  as Needs you's card does and say where they diverge.

## Steps

1. Read `src/pages/home.rs` (`main`, `intro_words`, `intro`, `decided`,
   `decision`, `needs`, `phone_need`, `foot`) and `src/pages/needs.rs`
   (`card_main`, `choices`), and `src/tests/home.rs`'s helpers.
2. `home_cards.rs`: `cards` as the contracts' section 5, reusing
   Needs you's choice drawing for a question's choices and `shared`'s need
   words for the from line.
3. `all_home.rs`: `main(b, root, w, bindings, times, …)` reads
   `all_home::facts(root, times.now)`, its parts from
   `parts::home(&facts.parts_facts())`, and calls `draw`; `draw` lays the
   parts in the design's order, each in its region; the needs from
   `root.pages().needs().needs(inbox, approvals)` when the open inbox is
   unfiltered; the phone's kicker, byline and "Decided for you · N ›" in
   styles of their own, shown by `temper.css` on a phone alone, as Home's
   phone block (`look::HOME_CTA`) is today.
4. `temper.css`: the rules under the two markers, from `whole-story.css`.
5. Tests, one row a case of the acceptance; break each new rule once and
   see its row fail.
6. Journeys: the replay (`heavy cargo nextest run -p
   temper-client-journeys`), `again NAME` for a stale one.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-view -p temper-client-shell-terminal -p
   temper-client-gallery --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-view -p temper-client-shell-terminal -p
   temper-client-gallery`. Commit and push.

## Report

What you changed, the helpers of `home.rs` this page reuses, where the
phone's form and the wide one split, the calls you made inside the
sketches, the tests and their counts, each new row's rule and how it
failed when broken, and your implementation and check minutes apart.

## Rebase (2026-11-08, step merge, foreman ui-p2-home/all-home-page/foreman-1)

preserve both vocabularies and recompute words.rs declarations during rebase
The merge conflicts only in `temper/crates/temper-client-view/src/words.rs`, where both branches update vocabulary counts and `ALL` length.
Main commit `f358753b09` adds four role-reading words; integration merge `c3612fec99` carries them alongside this plan’s vocabulary.
Keep `RoleReading`, `RoleUnreadable`, `RoleRuleReading`, and `RoleRuleUnreadable`, including their IDs and English forms.
Keep all sixteen Home/card words from `d9a00af040`, including IDs 4325–4340 and their English forms.
For the inspected integration tip, the combined declarations are `PHRASES = 2117`, `SEGMENTS = 3752`, `ENGLISH_BYTES = 51813`, and `ALL` length 1435.
Regenerate counted declarations with `tests::accounts::english_phrase_declarations_match_the_vocabulary`, through heavy with `SKEIN_CODEGEN_WRITE=1`; then verify that test without generation.
Retain the automatic merges in `temper.css`, `pages.rs`, and `root.rs`, and run the task’s focused checks after resolving the conflict.

## Answers

2026-11-08. *May this task add the missing `Address::Role` /
`Address::Rule` arms to the domain's matches?* No need: the trunk already
has them (`ce8d221c87`, *Keep Home matches exhaustive after role pages
merge*, in `pages.rs`, `pages/all_home.rs` and `pages/away.rs` of
`temper-client-domain`). Merge the trunk (`ui-p2-home/integration`) into
`ui-p2-home/all-home-page-r1` (`15e46ee9b3`), keeping the words.rs
resolution above; then regenerate the counted declarations as the Rebase
section says (the counts there were read at an earlier tip: trust the
generator's, not those numbers), verify without generation, and run the
task's focused checks. This task edits no domain file.

## Rebase (2026-11-08, step merge, foreman ui-p2-home/all-home-page/foreman-3)

resolve launchpad-chat conflicts by regenerating the journey from the combined code
Merge the current `ui-p2-home/integration` into the task branch, preserving both sides’ source changes.
The nine conflicts are under `temper/docs/design/client/tui/journeys/launchpad-chat/`: `01`–`04`, each `.html` and `.txt`, plus `recording`.
Task commit `7222026a34` refreshed these independently of integration commit `3f59975fc0`, which merged main and regenerated the same journey.
Main’s `48ea166473` and `a08df03746` also refreshed it; frame 04 differs in whether the opening question appears, beyond changing ports and clocks.
Resolve provisionally with one complete artifact set, then run `heavy cargo run -p temper-client-journeys -- again launchpad-chat` against the merged code.
Keep the resulting recording and all generated frames together; do not splice recording events or hand-edit frames to match either side.
Generation is authorized by the task and `temper/docs/design/client/tui/journeys.md`, sections 6 and 8.
On `a key mark caught a moving page`, retry generation at most three times.
Run the focused journey replay and the task’s focused checks through heavy; report regenerated artifacts and test counts.

## Rebase (2026-11-08, step merge, foreman ui-p2-home/all-home-page/foreman-4)

preserve Home and How module registrations and regenerate the combined vocabulary
Merge current `origin/ui-p2-home/integration` into the task branch.
The conflicts are in `temper/crates/temper-client-view/src/pages.rs` and `src/words.rs`.
Integration commit `9430783b5b` brings main’s How work runs changes from `db801753b4` and shared ways module from `a4a965ba4d`.
In `pages.rs`, retain all four registrations: `all_home`, `home_cards`, `how`, and `ways`.
In `words.rs`, retain the task’s sixteen Home/card words, IDs 4325–4340, and their English forms.
Retain integration’s thirty `How*` words, IDs 4900–4929, and its removal of obsolete Settings words.
Keep the existing role-reading vocabulary.
Combine the `ALL` entries provisionally so the crate compiles; regenerate its length and the counted declarations instead of choosing either side’s counts.
Through heavy, run `tests::accounts::english_phrase_declarations_match_the_vocabulary` in `temper-client-view` with `SKEIN_CODEGEN_WRITE=1`, then verify the same test without generation.
Retain the automatic merges in `temper.css`, `look.rs`, and `tests.rs`.
Run the task’s focused checks and journey replay through heavy; regenerate any journey reported stale using its tool and report the resulting artifacts and test counts.
