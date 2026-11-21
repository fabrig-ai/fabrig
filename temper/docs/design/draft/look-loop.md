# Look requests, drawn on walks and checked by tools

Draft, 2026-11-21, from a session with the user that day. Now that
journeys are walks (`../client/journey-walks.md`), the user asked how a look request
should run from end to end: how it is drawn and filed, what its goal
needs to land it, and what checks the result. The user's words:
rendered journeys that "I can open from the browser as plain HTML files
in the fabrig git repo are the most convenient form", and no screenshots.
`../client/web/look-requests.md` stays the brief for whoever lands a request. This
draft changes four things in it. A request is drawn on a walk. Its goal
reads it from a branch that never moves, with words from one template.
Its landing compares by a script. A check after landing renders it,
with no one watching. It supersedes the draft of 2026-11-04 that became
`../client/web/look-requests.md`.

## 1. In one page

- **Drawn on a walk** (section 2): a request is drawn on a journey's
  print. That is main's pages where a journey shows the state, or else a
  walk the request adds, printed by the person in seconds. A hand-made
  page is the last resort. A walk is source: it lands with the request,
  the writer prints it on main, and so every request leaves a journey
  behind.
- **Read from a branch that never moves** (section 3): before its goal
  starts, the request's commit is on the project's forge as
  `docs/look/NAME`. The goal's words come from one template and pin
  that commit, so nothing is restated by hand.
- **Started on the person's word** (section 3): "land them" approves
  every request on the branch that isn't held and hasn't landed. Each
  becomes a goal, in branch order.
- **Compared by a script in the goal** (section 4):
  `scripts/look-compare.sh NAME` prints the request's journeys into
  scratch. It compares each page three ways with git, and each of the
  request's rules with the landed stylesheets. Then it writes the
  account's rows. It starts no browser, so a plan may run it.
- **Rendered in Chromium after landing** (section 5): the writer's run,
  at each new tip of main, renders each newly landed request's pages
  beside the print at three widths. It compares the computed style and
  box of each element the request touched, and writes the verdict into
  `looks.md` beside the pages it prints.
- **No one watches it** (section 5): the run starts by itself when main
  moves. Its change brings the verdict to whoever starts look requests'
  goals, who proposes a repair to the person for each difference.
- **The person's look is main's pages** (section 6): plain HTML in the
  repository, with a journey for each look asked for, and `looks.md`
  saying whether each rendered as drawn.

## 2. The request

As `../client/web/look-requests.md`, 2: one commit, one change; its trailer, its holds,
its message saying every state in words. The changes are where it is
drawn and what its commit carries.

### 2.1 Where it is drawn

In this order:

1. **Main's pages,** where a journey's print shows the state: the
   request edits them in place (`../client/web/journeys.md`, 7;
   `../client/tui/journeys.md`, 10).
2. **A walk of the request's own,** where no journey shows it. The
   request adds a journey's directory under its root, with its `walk`
   of version 2, its `README.md` and, where it needs the fake model's
   turns, `scripts.json` (`../client/journey-walks.md`, 2.1). The walk
   starts `after` the journey that comes nearest, so that its own lines
   are few (`../client/journey-walks.md`, 2.3). The person prints it
   with `cargo run -p temper-client-walk -- print --out DIR NAME`,
   copies DIR's pages of NAME into its directory, and draws on them.
3. **A gallery state** (`../client/web/gallery.md`), where the world
   can't set the facts: an account's usage, a long name, a box's front.
   The gallery's pages carry no overlay. The person copies the page into
   a hand-made directory and draws there, as for the next rung.
4. **A hand-made page,** last, in a directory whose `README.md` says it
   is hand-made and from what. No print meets it, so nothing downstream
   compares it (4.3, 5.2).

- **Why a walk:** its sources land with the request (4.1). The writer
  prints the state on main, the goal compares against it exactly, the
  check after landing renders it, and the next request draws on it.
- **What a walk reaches:** what the playground's world reaches by a
  person's acts, which are presses, typing and approvals, and the fake
  model's turns. Its facts are fixed (`../client/journey-walks.md`,
  4.1), and it prints no width: widths are the browser's, at the
  person's screenshots and the check after landing.

### 2.2 The person's tools

- **`temper-client-walk`,** built in the person's checkout with
  `TEMPER_INTERNAL_BUNDLE=none cargo build -p temper-client-walk`. That
  takes minutes once and seconds after. It prints the walks of the
  checkout it is built from.
- **The rest as now:** the overlay, the pages' edits, the message, the
  screenshots at 1440 and 420 px and each open state
  (`../client/web/look-requests.md`, 2), and `scripts/look-refresh.sh`
  (`../client/web/journeys.md`, 7).

### 2.3 Its commit

- **The walk's sources** go in the request's one commit, beside its
  drawn pages and its overlay's rules.
- **A neutral author:** the project's forge is public, so the commit
  names no person, as every commit there does.

## 3. Its goal

### 3.1 The word that starts it

- **"Land them":** the person says it to whoever they charge with
  starting look requests' goals. It approves every request then on the
  branch that carries no `Look-hold` and whose trailer no commit on main
  carries.
- **A goal each,** in one milestone, each after the one before it,
  oldest first, since each is drawn on the pages the earlier ones ask
  for. The person may say two run side by side, as they did on
  2026-11-19.
- **Its branch:** before its goal starts, the request's commit goes to
  the project's forge as `docs/look/NAME`, a branch that never moves. A
  request redrawn after its goal started takes a new name.

### 3.2 Its words

From one template:

```
<the request's subject without "Look request (temporary): ", at most 60 characters>

Land the look request NAME by temper/docs/design/client/web/look-requests.md,
which is the whole brief: its commit C, on the forge's docs/look/NAME, over
main M. A walk it adds lands as its sources; its pages and overlay never land.

<the request's message as its commit holds it, its trailers dropped>

Tests cheap: focused and fuzzy suites only.
```

### 3.3 Its questions

Whoever started it answers the goal's product questions from the
request's message. What the message doesn't decide goes to the person,
as one question whose choices are in the request's own terms.

## 4. The landing

As `../client/web/look-requests.md`, 3, with these changes.

### 4.1 The walk lands

A walk the request adds lands as its sources: `walk`, `README.md` and
`scripts.json`, as the request has them. A line changes only where the
landing renames a control the walk presses or a word it sees
(`../client/journey-writer.md`, 2). Its pages never land: the writer
prints them after the landing.

### 4.2 The comparison

`scripts/look-compare.sh NAME`, beside `look-refresh.sh`, run in the
landing's own tree:

1. **Reads the request:** its commit C from `docs/look/NAME`, and C's
   parent P.
2. **Prints** the journeys whose pages C changes into scratch, with
   `temper-client-walk print --out DIR JOURNEY...`. A walk that stops
   is the landing's to repair (`../client/web/look-requests.md`, 3).
3. **Compares each page three ways, with git:**
   - The three versions are the page as P holds it, which the person
     drew on; as C holds it, which they drew; and DIR's, which landed.
   - `git merge-file` applies the person's edit to DIR's page. Where the
     result equals DIR's page, every edit landed. Each hunk between
     them is an edit that didn't land, or landed otherwise.
   - A page of a walk the request adds has no P. Its whole page is
     compared with DIR's, and a difference in the world's facts is
     named as one.
4. **Compares each rule:** each rule C adds to an overlay, its selector
   and declarations, against the landed stylesheets' rules for that
   selector. Each is landed, differing (in which declarations), or
   missing. The TUI's rules land in code, its theme
   (`../client/tui/journeys.md`, 10), so they are listed for the landing
   to account for by hand.
5. **Writes the account's rows** (`../client/web/look-requests.md`, 5): one per page
   edit and per rule, with its result, and one to fill for each state
   the message names in words.
6. **Its exit:** 0, every edit and rule landed; 1, a row differs or is
   missing; 2, its arguments, or a request it can't read.

It starts no browser, terminal or box, so a plan runs it
(`../client/web/look-requests.md`, 3).

### 4.3 What it can't compare

A hand-made page has no print to meet (2.1). Its rows say so, as
accounts do today.

## 5. After landing

### 5.1 The run

- **It starts by itself:** the writer's run
  (`../client/journey-writer.md`, 4) starts whenever main's tip moves,
  at most every ten minutes, on a timer of its owner's, until temper
  runs it (`../client/journey-writer.md`, 8). One run at a time, as
  now.
- **What it does:** it prints, then checks the requests landed since
  the last check (5.2). It offers the pages and the verdicts as one
  change.

### 5.2 The render check

- **Which requests:** each request whose trailer a commit on main
  carries, whose commit is at `docs/look/NAME`, and whose name
  `looks.md` doesn't hold yet.
- **Which pages:** each page C changes, from C's tree with C's overlay
  and stylesheets, beside the same path in the run's print, with the
  landed stylesheets and an empty overlay. A page with nothing beside it
  is listed as not compared: a hand-made page.
- **How:** in the Chromium the browser tiers use, through skein's driver,
  which gains a read of an element's computed style as it reads boxes
  today. A command of `temper-client-journeys` runs it
  (`../client/web/look-requests.md`, 8).
  - Widths: 1440, 1000 and 420 px, with every dialog and disclosure
    open.
  - Elements: those the request touched, meaning each element its edit
    changed (4.2's hunks) or one its overlay's rules select, with their
    descendants. They are paired by the pages' alignment.
  - What is compared: each element's computed style and its box. A box
    within 1 px is equal.
- **What it says:** identical, or each difference: the page, the
  element, the width, the property, and the request's value against
  the landed one.
- **States in words** aren't rendered: the account covers them.
- **The TUI's** themed frames are rendered the same way. Their text was
  compared at the landing (4.2).

### 5.3 The verdict

- **`looks.md`,** beside each root's journeys, is written by the writer
  alone (`../client/journey-writer.md`, 2). It holds one section per
  checked request: its name, its commit, the tip it was checked at, and
  its result. A request is checked once, and its section stays.
- **It lands with the print:** the run's change holds the pages and
  `looks.md`, and its message ends `Printed: TIP`, so
  `look-refresh.sh` drops the request (`../client/journey-writer.md`,
  6). A landing that changes no page still gives a change, its
  verdict, so every checked request is printed and dropped.
- **Who hears it:** the run opens its change as whoever starts look
  requests' goals, and the change's end brings them the verdict. Each
  difference becomes a repair goal on the person's yes, carrying the
  request's trailer (`../client/web/look-requests.md`, 4).
- **A walk that stops** is named in the change's message, as now
  (`../client/journey-writer.md`, 4), and reaches the same reader. Its
  repair is a goal on the person's yes. A stopped walk holds no landing
  (the user, 2026-11-18).

## 6. What the person sees

- **Main's pages are the landed look:** each journey's pages under the
  two roots, plain files that open in a browser from a checkout. A
  request drawn on a walk leaves its state there for good.
- **`looks.md`** says, for each request, whether it rendered as drawn,
  and where it didn't.

## 7. Its tests

Focused and native: no browser, terminal or box starts.

- **The comparison:** in `crates/temper-client-journeys-tool`'s suite,
  beside the refresh's tests, in scratch repositories with a fake
  print. Cases:
  - every edit landed: exit 0;
  - an edit missing: its hunk as a row, exit 1;
  - a rule landed, one differing, one missing;
  - a new walk's page compared whole;
  - the TUI's rules listed for the landing;
  - a request it can't read: exit 2.
- **The run,** with a fake check. Cases:
  - a landed request with no section gets one, and a checked one is
    skipped;
  - a hand-made page is listed as not compared;
  - a landing that changes no page still offers its verdict, with
    `Printed:`.
- **The render check's reading of Chromium:** a browser-tier story,
  written and compiled in the plan, not run. Two pages differing by one
  rule give one difference at each width, and equal pages give none.
- **The delivery check and the merge job** take `looks.md` as a written
  file (`tools/pre-pr/design.md`, 4; `tools/merge/README.md`).

## 8. Later

- **temper runs the writer** (`../client/journey-writer.md`, 8), and
  the timer goes.
- **A gallery state as a walk:** once the world can set an account's
  usage or show a box's front, the gallery's states become walks, and
  2.1's third rung goes.
- **A walk at a width:** a mark printed at a width, should a request
  need the narrow layout's markup rather than its look.

## 9. Open questions

None.
