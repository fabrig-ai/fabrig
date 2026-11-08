+++
id = "how-world"
part = "checks"
kind = "feature"
needs = ["how-page"]
files = ["temper/tests/client/tests/suite/ways.rs"]
effort = "high"
minutes = 60
+++

# The client world: How work runs' path over CI and tiers, and over neither

## Goal

Show in the client's joint world, against the engine's own derivation, that How
work runs draws each stock way's path in lanes over a project with CI and tiers
and over one with neither (`temper/docs/design/client/README.md`, 5.13, *A path
in lanes*), as Ask temper's page already draws them, by rows of the existing
story `the_three_ways_draw_ci_and_tiers_or_neither`.

Acceptance:

1. In `temper/tests/client/tests/suite/ways.rs`,
   `the_three_ways_draw_ci_and_tiers_or_neither` also opens `Settings { project
   }` in each of its two worlds (CI and tiers; neither) and, for each of the three
   ways' tabs pressed there, checks the table "The path of a change": its header
   row names each stop's step in 5.13's words, a bar its words; each stop's button
   stands in its actor's lane and is named from the read as 5.13 says; the
   authority row stands between Agents and Code; the stops' order is `PATHS`'s.
   The heading names the current way, "Agents review".
2. No new test function and no new world: the rows reuse the story's two worlds.
3. The client world's focused tests pass with it, its CPU time in the report,
   clippy clean.

## Inputs

- `temper/docs/plan/ui-p3a-how/notes/common.md`; `notes/contracts.md`, C3 (the
  tree).
- `temper/docs/design/client/README.md`, 5.13, *The page* and *A path in lanes*;
  `temper/docs/design/domain/forge.md`, 8.13.
- The code: `temper/tests/client/tests/suite/ways.rs` (`with_repository`,
  `PATHS`, `NAMES`, `under`, `words`, `drawn_path`, `act`), the world's
  `front::ways` fixture; `temper-client-view`'s `pages/how.rs` (`how-page`'s).

## Owns and may change

Owns: `temper/tests/client/tests/suite/ways.rs`.

Reader and neighbour files: none. `ui-p1-rail`'s `readers` changes other stories
of this suite's directory beside you; this file's other tests stay as they are.

Pre-approved options:

- **If opening Settings from Ask** in the same world needs an event the story
  lacks: send the address as the story's `world.client(..)` start does, or the
  rail's link, whichever the world offers, and say which.
- **If a stop's card name** isn't in the panel the story reads: take it from the
  panel's way's path (`StockWay::path().stops()`), never written in the test.

## Steps

1. A helper beside `drawn_path` reading the table's columns: each header and the
   lane and name of its column's button.
2. The rows of acceptance 1 in the story, its expectations derived from `PATHS`
   and the read.
3. Break one rule by hand (in a scratch change of `pages/how.rs`, a code stop's
   button in Agents' row), see the story fail for it, restore.
4. Check as `notes/common.md` says, running the story by name. Commit and push:
   the message says the joint world reads How work runs' lanes over both
   repositories.

## Report

Follow the implementer's text. Also give the story's CPU time before and after,
and the rule broken and the failure seen.
