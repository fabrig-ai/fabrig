+++
id = "sheet"
part = "sheet"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/task-pages.css",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 60
+++

# The requests' overlay rules, moved into temper.css

## Goal

Move every rule of the look requests' overlay (the tip of `origin/look-requests`, `7fba5d0fe3`,
its `temper/docs/design/client/journeys/overlay.css`, 132 lines) into the shell's stylesheet,
`temper/crates/temper-client-shell-web/temper.css`, so that the bundle draws the welcome
playground's chats page, its banner and its switcher as the sheets plus that overlay drew them,
at every width (`temper/docs/design/client/browser.md`, 2.5; `temper/docs/design/client/journeys.md`,
7, "Done"). This task is CSS alone: the markup the rules select is drawn by the view's tasks
(`chats-pane`, `banner-switcher`) beside it, and a rule whose markup no task draws yet stays
for the plan that draws it (below).

Acceptance:

- Each declaration of the overlay holds in the merged sheets, for the elements it selected, at
  every width the overlay's media queries name and between them: the computed result of
  `skein-ui.css`, `temper.css`, `task-pages.css` loaded in that order equals that of the same three
  plus the overlay, for the requests' `02.html` (its markup at `7fba5d0fe3`, with the one change
  below), at 1280 px, 1000 px and 600 px wide, the switcher open and closed.
- The overlay's rules are merged, not appended: a declaration whose selector the sheet already
  has replaces the sheet's in that rule; a new selector's rule sits in the sheet's section for
  the part it draws (the banner's beside `.temper-banner` and the menu's rules; the switcher's
  after the menu's; the chats pane's beside the chats rules, `.temper-page` and
  `.temper-heading`), with a comment in the sheet's form (`/* The chats page's pane: ... */`),
  from the overlay's own comments. No rule of the sheet is left that a merged one fully
  supersedes.
- `git diff HEAD 7fba5d0fe3 -- temper/docs/design/client/journeys/overlay.css` is the whole of
  what moved: nothing beyond it changes, but the cascade repairs below.
- `temper/docs/design/client/browser.status.md` loses the entry this task closes (below).
- `heavy cargo nextest run -p temper-client-gallery` and `-p temper-client-bundle` pass, as do
  `-p temper-client-journeys` (its tests link the sheets).

## Inputs

- `temper/docs/plan/look-welcome-chats/notes/common.md` (the requests, how to read them).
- `temper/docs/design/client/browser.md`, 2.5: "The banner is one quiet line", "The chats page
  reads as a terminal", "The navigation", "The composers"; and "The main's scale" for what it
  says of lengths (below).
- `temper/docs/design/client/journeys.md`, 7 ("Selectors the view can carry", "Done").
- `temper/docs/design/client/README.md`, 5.1 (the banner: the switcher, "Set up temper"), 3.5
  (the chats page's pane) for what the markup holds.

## Owns and may change

- `temper/crates/temper-client-shell-web/temper.css`: the overlay's rules merged in; the
  sheet's rules they supersede edited or removed; the `:root` variable `--temper-faint` takes the
  overlay's `#8c867d`.
- `temper/crates/temper-client-shell-web/task-pages.css`: only where a rule there, loaded after
  `temper.css`, would now win over a merged declaration the overlay used to win over (below).
- `temper/docs/design/client/browser.status.md`: its 2.5 entry for the banner and the chats
  page (written by the cut) goes; the rest word for word.

Pre-approved, with their defaults:

- **The cascade:** the overlay loaded last, so each of its declarations beat any sheet rule of
  equal specificity, in a media query or not, in `task-pages.css` too. Merged into the sheet, a
  declaration can lose to a later sheet rule it used to beat (`@media (max-width: 1100px) {
  .temper-banner { gap: 11px; padding-inline: 17px; } }` against the overlay's `.temper-banner`
  gap and padding is one). For each such later rule, change it so the overlay's value holds
  (drop the declaration it overrode, or move the merged one after it); list each in the report.
  A rule of higher specificity that beat the overlay keeps beating it: leave it.
- **The heading is level 1:** the pane's "Chats" is the chats page's level-1 heading
  (`README.md`, 3.5), so the overlay's `.temper-sidebar > .temper-heading h2` selects `h1` in the
  sheet. Check that `h1` there renders as the overlay's `h2` did (`.temper-main h1, .temper-main
  h2` and `.temper-heading h2` are the sheet's rules in play); add what `.temper-heading h2` gave
  the `h2` that the `h1` lacks only if it shows.
- **Lengths stay the overlay's:** the person tuned them on the print as drawn, so they move as
  they are, not rescaled by 14/12 (`browser.md`, 2.5, "The main's scale", is for lengths taken
  from the mockups).
- **Rules for markup the view doesn't draw yet:** the switcher's Rename (`.temper-project-rename`,
  `.temper-project-rename-form`) and "+ New project" (`.temper-project-new`) rules move too,
  merged after the switcher's: `first-run-projects`' step 3 draws that markup
  (`temper/docs/plan/look-welcome-chats/notes/contracts.md`, "The switcher's markup"). Their
  selectors keep the overlay's form.
- **Rules no view draws any more:** the playground's invitation (`.temper-invitation*`) and the
  menu's direct project buttons (`.temper-menu > button`, `.temper-menu > button[aria-pressed=true]`,
  `.temper-menu > button:hover`, and their `@supports` and 760 px forms) lose their markup to
  `chats-pane` and `banner-switcher`. Remove the menu's direct-button rules where the overlay's
  `.temper-projects > button` rules replace them, keeping what the menu's other buttons (the
  corner's `Sign out`, which is `.temper-corner > button`) need; keep `.temper-invitation*` as it
  is (whether its suggested requests stay is `chats-pane`'s; a later task removes what is dead).
- **A selector by position** (`div:first-child`, `div:last-child:not(:first-child)`,
  `p.temper-account-line:first-of-type`) the overlay holds stays as it is: it selects among the
  chatbar's and the chooser's own children, as the sheet's chatbar rules already do.

## Steps

1. Read the overlay at `7fba5d0fe3` and the sheet. For each overlay rule, find the sheet's rules
   for the same elements (`rg -n` by class), and merge as above. Keep the sheet's comment form.
2. For the cascade, walk each merged declaration's later rivals in `temper.css` and
   `task-pages.css`, as above.
3. Check by looking, if you can: serve the checkout from the repository root
   (`python3 -m http.server 8766`, `journeys.md`, 5.4) with the requests' pages and overlay in a
   scratch copy outside the repository's tracked files (`git show 7fba5d0fe3:...` into a
   temporary directory under your slot, deleted before the push), and compare the computed styles
   in a headless browser if one is on the machine (`SKEIN_TEST_CHROMIUM`). If none is, say so in
   the report; the acceptance's comparison is then by reading, rule by rule, and the report lists
   each overlay rule with where it went.
4. `heavy cargo nextest run -p temper-client-gallery`, `-p temper-client-bundle`, `-p
   temper-client-journeys` (their counts).
5. The status entry; citations (`notes/common.md`); one commit, its message ending with these
   trailer lines, one each, in this order:

   ```
   Look-request: terminal-chats-experiment
   Look-request: chats-left-pane-experiment
   Look-request: chats-pane-head-experiment
   Look-request: banner-project-switcher-experiment
   Look-request: switcher-new-project-experiment
   Look-request: pane-playground-note-experiment
   Look-request: pane-note-after-chats-experiment
   Look-request: banner-leave-playground-experiment
   Look-request: switcher-rename-project-experiment
   ```

   Push.

## Report

`notes/common.md`'s and the implementer's text's. Beside them: each overlay rule and where it
went (merged into which rule, or new after which), each later rule changed for the cascade, the
sheet rules removed, and how the result was checked (a browser's computed styles, or reading).
