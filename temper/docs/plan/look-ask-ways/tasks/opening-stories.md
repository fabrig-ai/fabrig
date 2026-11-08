+++
id = "opening-stories"
part = "stories"
kind = "feature"
needs = ["opening"]
files = ["temper/tests/client/tests/suite/ways.rs", "temper/tests/client/src/front/ways.rs", "temper/tests/client-browser/tests/browser_all/browser_pages.rs"]
effort = "high"
minutes = 150
+++

## Goal

Prove the opening works for real, engine and client joined: drawn from
the `ways` read over a project with CI and both tiers and over one with
neither, "Yours", "Use this" refused in words, "Keep it" marking To
begin's step done, and no act for a reader who may not change settings
(`inputs/request.md`, *Its tests*; `temper/docs/design/draft/stock-ways.md`,
8). Then the browser tier's row, written and compiled.

Acceptance, in the joined client world's `tests/suite/ways.rs`, whose
four stories gain the page's tree (`world.tree(person)`) beside their
`StockWays` checks, and one table test:

- **The three ways drawn,** a table over two rows of the fixture's
  repository, CI and a tier following main and one following goal
  branches, and neither: for each way, `Act::WayTab` pressed, the tree's
  panel is named by the way, its line is the read's, and its path's
  fallback, a stop a line, reads in order as the request's pages draw
  it: You approve "you ask, a person", "produce, an agent", "check,
  code", "repair, an agent", "check, code", "you approve, a person",
  "update, code", "resolve, an agent", "merge, code", then "verify,
  code" (`stock-ways.status.md`, *Design departs*) and "main, a branch";
  Agents review and Draft first likewise from the request's `01.html` and
  `03.html` (their figures' `aria-label`s), Draft first's goal branch
  "goal branch, a branch"; with neither, the same less every "check" and
  the tiers' stops. The current tab carries "now". The expected words
  are the test's own table, from the request's pages, never read from
  the engine.
- **A stop's card:** on Agents review, `Act::WayStop` of the review opens
  a region named "Review" holding "Model", "Can" and "Asked to" and a
  link "Open the reviewer ›"; pressed again, it is gone.
- **"Yours":** a project whose rule 1 asks no CI (as the engine world's
  ways stories set it) draws a fourth tab "Yours" with "now", shown, its
  path with no "check", and no "Use this" or "Keep it" button.
- **"Use this" refused** (the box owner's story): the tree's alert under
  the foot holds the forge's words, "requires 1 distinct known forge
  people; currently 0 eligible", and the button "Use this" stands again.
- **"Keep it"** (its story): before, a button "Keep it" in the shown
  panel; after, To begin's step done (as the story checks now), no
  "Keep it" on the opening, and the toast "Agents review is now this
  project's own, as revision N." with the committed revision.
- **A member:** the tabs and paths drawn, no "Use this" and no "Keep it"
  in the tree.
- **The browser row:** in `browser_pages.rs`'s story "phone ask", after
  `see(ask())`: see the tabs named "Ways of working", press the tab
  "Draft first", see a button "Use this". Written and compiled (`heavy
  cargo check -p temper-client-browser-world --tests`), never run: the
  browser tier runs after landing.
- The client world's focused tests pass within the suite's budget.

## Inputs

- `temper/docs/plan/look-ask-ways/notes/common.md`;
  `temper/docs/plan/look-ask-ways/notes/contracts.md`, sections 1 and 3.
- `temper/docs/design/client/README.md`, 3.5, *Ask temper's page*; 5.13,
  *A way's acts*.
- `temper/docs/design/draft/stock-ways.md`, 8, and its status's *Design
  departs* ("verify" on main).
- `temper/tests/engine/domain/tests/suite/ways.rs`, `path_driver`, for how
  a repository with CI and both tiers is adopted in the engine's world.

## Owns and may change

- Owns the three files above.
- `temper/tests/client/src/front/ways.rs`: the fixture gains the
  repository's choice, CI and both tiers or neither, as `path_driver`
  adopts them, and the "Yours" project's rule; its existing callers keep
  their repository. If the joined fixture cannot carry tiers without a
  change to the engine's world crate beyond a fixture line, add that
  line in `temper/tests/engine/domain/src/` and name it.
- `temper/tests/client/src/**`: a helper the stories need to read the
  tree (finding a node by role and name under another), if none exists.
- Not yours: the client's crates. A story that fails on the page's
  behaviour is a finding for the report, with its log, not a change to
  the view.

## Steps

1. Read the four stories, the fixture, `path_driver`, and the request's
   pages' figures (`notes/common.md`).
2. Grow the fixture; add the table test and the tree's checks to the
   four stories.
3. Break a rule once by hand (draw the "Keep it" after the step is done,
   leave a tier's stop with neither) and see a row fail; restore.
4. `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(/ways/)'`; then `heavy cargo check -p
   temper-client-browser-world --tests`.
5. `cargo fmt`; `heavy cargo clippy -p temper-client-world -p
   temper-client-browser-world --all-targets -- -D warnings`. Commit with
   the trailer `Look-request: ask-temper-ways` and push.

## Report

As the implementer's text says; the stories' rows, their times, and the
browser row compiled and not run.

## Answers

2026-11-08. *The client world's inherited CPU cost exceeds 30 seconds:
separate repair, or widen this task?* Neither widening nor a new task
here: the overage is the base's (34.3 s for 429 tests before this
branch, 30.2 s for 431 after it), so this branch adds none, and its own
acceptance is met as far as it can be. Change nothing for it: keep the
pushed commit, keep the 30-second budget, and report the two
measurements and their logs under `## Left` ("the client world's
focused CPU is over 30 s at the base, not raised by this branch"). The
gate's budget check on the trunk, and its repair, take the package-wide
cost; this task does not hunt it.
