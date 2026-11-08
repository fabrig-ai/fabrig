+++
id = "how-gallery"
part = "checks"
kind = "feature"
needs = ["how-page"]
files = ["temper/tools/gallery/src/samples.rs", "temper/tools/gallery/src/tests.rs", "temper/tools/gallery/src/worlds.rs", "temper/tools/gallery/src/worlds/ways.rs", "temper/tools/gallery/names.txt"]
effort = "high"
minutes = 100
+++

# The gallery's How work runs states

## Goal

Print How work runs in the gallery's six states of
`temper/docs/design/client/browser.md`, 7.2, `how` to `night-how`, so that after
landing its pages are shot at a laptop's and a phone's width, by day and by night,
beside the mockups `web/whole-story/how.html` and `pair/how.html`. Each is a world
at `Settings { project }` stepped by the documents and acts the state names, never
set by hand (7.2, *A state*).

Acceptance:

1. `samples.rs` holds the states `how`, `how-approve`, `how-draft`, `how-yours`,
   `how-card` and `night-how` at `Settings { project: 1 }`, each as 7.2's row
   says, `night-how` built on `how` as `night-ask` is on `ask-first-run`; the
   index links each to its mockup, `how-approve` to
   `../web/whole-story/pair/how.html` and the others to
   `../web/whole-story/how.html`.
2. Their `ways` deliveries are hand-made in `worlds/ways.rs`, as the `ask-ways-*`
   states' are, over a repository with CI and tiers (`how`, `how-draft`,
   `how-card`, `night-how`) or with neither (`how-approve`), and a project of its
   own (`how-yours`); `how-draft`'s kind 7 has Goals shown and its goal template's
   budget 80.
3. `tests.rs`' holds check each state's heading and the parts its row names: the
   way named; "Ask temper to change this"; the table "The path of a change" with
   a stop in each lane; "verify" before main's bar where tiers are; `how-draft`'s
   "goal branch" header and Budgets' row; `how-yours`' "A way of its own" and
   "Yours" tab; `how-card`'s card and "Open the reviewer ›". No other test is
   added; `names.txt` stays sorted.
4. `temper-client-gallery`'s focused tests pass, clippy clean, its CPU time
   before and after in the report (the gallery is over its budget,
   `browser.status.md`, 7.3: these six states are all it adds). The entry
   "**Not yet:** 7.2" naming `how` to `night-how` in
   `temper/docs/design/client/browser.status.md` is gone.

## Inputs

- `temper/docs/plan/ui-p3a-how/notes/common.md`; `notes/contracts.md`, C3.
- `temper/docs/design/client/browser.md`, 7.1, 7.2 (*A state*, the worlds, the
  rows `how` to `night-how`, `night-ask`) and 7.3.
- `temper/docs/design/client/README.md`, 5.13, *The page*.
- The code: `temper-client-gallery`'s `samples.rs` (`Sample`, `special_mockup`,
  the `night-*` bases), `worlds.rs` (how the `ask-ways-*` states take their
  deliveries and acts), `worlds/ways.rs` (`world`, `deliver`, `press`, `path`,
  `own`), `tests.rs` (`Holds`, `Part`), `names.txt`; `temper-client-view`'s
  `pages/how.rs` (`how-page`'s).

## Owns and may change

Owns: the gallery's `samples.rs`, `tests.rs`, `worlds.rs`, `worlds/ways.rs` and
`names.txt`, for these six states alone.

Reader and neighbour files: `temper/docs/design/client/browser.status.md`, the one
entry above, deleted. `ui-p1-rail`'s and `ui-p2-home`'s gallery tasks change these
files beside you: add your lines in place, and on a catch-up regenerate
`names.txt` rather than hand-merge it.

Pre-approved options:

- **If the first run's world** can't hold kind 7's Goals shown with a template's
  budget for `how-draft`: take the reference world's project, as its own ways are
  at numbers the stock leaves free, and say so.
- **If a mockup's figure** has no place in the model (a budget's "$80"): draw the
  model's figure, as 7.2 says, never invented.

## Steps

1. The deliveries in `worlds/ways.rs`; the six samples; the mockups' links.
2. The holds in `tests.rs`; `names.txt`.
3. Break one rule by hand (`how-draft`'s budget dropped from its delivery) and
   see its hold fail; restore.
4. Check as `notes/common.md` says: the gallery's tests print into a scratch
   directory, never committed. Delete the status entry. Commit and push: the
   message says the gallery prints How work runs' six states.

## Report

Follow the implementer's text. Also give the gallery tests' CPU time before and
after, and the rule broken and the failure seen.
