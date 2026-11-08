+++
id = "account"
part = "account"
kind = "docs"
needs = ["opening-state", "path-look", "opening", "starters-out", "opening-gallery", "opening-stories"]
files = ["temper/docs/design/client/README.status.md", "temper/docs/design/client/browser.status.md", "temper/docs/design/client/mockups/STATUS.md", "temper/docs/design/client/journeys.status.md", "temper/docs/design/STATUS.md"]
effort = "high"
minutes = 90
+++

## Goal

Account for the look request `ask-temper-ways` as its landing's last
commit (`temper/docs/design/draft/look-requests.md`, section 5): a table,
one row per overlay rule, per page change and per state its message
names, each landed, differing with its reason, or deferred to a named
owner, none deferred to the request; and close the design's **Not yet**
entries the plan's tasks have built.

This task departs from the implementer's text: it is a docs task, and
it edits the design's status files above, and only those.

Acceptance:

- **The table,** in the commit message of this task's one commit, under
  a line "Account of ask-temper-ways", Markdown, `| Item | Result |`:
  - a row per rule of the request's `overlay.css` (`git show
    98322b3565:temper/docs/design/client/journeys/overlay.css`), by its
    selector as the request has it: "landed" where `temper.css` holds it
    as written, "landed as SELECTOR" where it is translated
    (`notes/common.md`, `temper.css`), "differs: WHY" where it isn't
    there (the wide and narrow SVGs, by the scene's layout; the line's
    fade, two tones, since a scene has no gradient; the caret, drawn by
    the scene; `aria-expanded` and `:focus-visible` on a stop, since a
    scene's stop carries no ARIA and the fallback's buttons take the
    keyboard: `temper/docs/design/client/README.md`, 5.16);
  - a row per page change of its four pages (the heading gone, the lede,
    the tabs, the axis, each panel's line, path, legend and act, the
    card), each "landed" with the file that draws it;
  - a row per state its message names (*What the pages show* and
    *States the pages do not show*, and *Replaces*), each "landed" with
    the test or gallery state that shows it: the opening drawn over CI
    and both tiers and over neither, a stop's card of each kind, "Use
    this" with its toast, its refusal in words, "Keep it" and To begin's
    step, "Yours", no act for a member, the terminal's path as text, the
    opening gone after a first message, Ask temper's replies drawing
    the same paths, and each replaced part gone;
  - the render check's row: it follows landing, on main, outside the
    plan, owned by temper/ui (`temper/docs/design/client/README.md`,
    section 11, *Ask temper's ways after landing*); compilation, the view's
    tests and the gallery's prints do not establish rendered equality.
  Every row is checked against the trunk: a rule by `rg` in `temper.css`,
  a state by its test or gallery state by name. A row that cannot be
  "landed" or "differs" with a reason the design already gives stops the
  task: a `Question:` naming the row, never a deferral.
- **Status:** `temper/docs/design/client/README.status.md`'s entries
  naming the plan `look-ask-ways` (3.5, *Ask temper's page* and
  *temper's own way*; 5.13; 5.16) and `browser.status.md`'s (2.5, 4.6,
  7.2) go where what they name is built; anything still short is said in
  their place, in the same form. `mockups/STATUS.md`'s row for Ask
  temper's page says what the client draws now: the lede, the ways' tabs,
  a path and a stop's card, the act, no heading, starters or own way's
  card, no reference beside the page. `temper/docs/design/STATUS.md`'s
  summary, if it names any of them. `journeys.status.md` only if an entry
  of it names the old opening.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports
  nothing broken.

## Inputs

- `temper/docs/plan/look-ask-ways/notes/common.md`.
- `temper/docs/design/draft/look-requests.md`, sections 3 (step 7) and 5.
- The look request's message, pages and `overlay.css`, as
  `notes/common.md` says to read them.
- `temper/docs/design/client/README.md`, 3.5, 5.13, 5.16 and section 11;
  `temper/docs/design/client/browser.md`, 2.5, 4.6 and 7.2.
- `DESIGN.md`, section 2 (status files).

## Owns and may change

- Owns the status files above; nothing else of `docs/design/`, no code.
- Options: if a rule or state turns out not landed and a few lines would
  land it, it is not this task's: name it in a `Question:` with the file
  that would hold it, so that a repair task carries it under the
  request's trailer.

## Steps

1. `git log --oneline --grep 'Look-request: ask-temper-ways'` on the
   trunk, to see the landing's commits.
2. Build the table row by row, checking each against the trunk.
3. Edit the status files; run the citations check.
4. Commit once, the table in its message, with the trailer
   `Look-request: ask-temper-ways`, and push.

## Report

As the implementer's text says; the table's counts by result.
