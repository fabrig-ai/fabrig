# Look requests, drawn on walks and checked by tools

Draft, 2026-11-21, from a session with the user that day. Now that
journeys are walks (`../../client/journey-walks.md`), the user asked how a
look request should run from end to end: how it is drawn and filed,
what its goal needs to land it, and what checks the result. The user's
words: rendered journeys that "I can open from the browser as plain
HTML files in the fabrig git repo are the most convenient form", and no
screenshots. `../../client/web/look-requests.md` stays the brief for
whoever lands a request. This draft changes four things in it:
- a request is drawn on a walk;
- its goal reads it from a branch that never moves, with words from one
  template;
- its landing compares by a script;
- a check after landing renders it, with no one watching.

It supersedes the draft of 2026-11-04, whose vision
`../../client/web/look-requests.md` carries.

## 1. In one page

- **Drawn on a walk** (section 2): a request is drawn on a journey's
  print. That is main's pages where a journey shows the state, or else
  a walk the request adds, printed by the person in seconds. A
  hand-made page is the last resort.
  - A walk is source: it lands with the request, and the writer prints
    it on main. So a request drawn on a walk leaves its journey behind.
- **Read from a branch that never moves** (section 3): before its goal
  starts, the request's commit is on the project's forge as
  `docs/look/NAME`. The goal's words come from one template and pin
  that commit, so nothing is restated by hand.
- **Started on the person's word** (section 3): "land them" approves
  the requests below the first held one that haven't landed. Each
  becomes a goal, in branch order.
- **Compared by a script in the goal** (section 4):
  `scripts/look-compare.sh`, through the journeys tool's
  `look-compare` command, prints the request's journeys into scratch.
  It compares each page three ways with git, and the request's rules
  with the landed stylesheets. Then it writes the account's rows. It
  starts no browser, so a plan may run it. The landing's last commit
  says the request has landed.
- **Rendered in Chromium after landing** (section 5): the writer's run,
  within ten minutes of a request's landing, renders each newly landed request's pages
  beside the print at three widths. It compares the computed style and
  box of each element the request touched, and writes the verdict into
  a page, `looks.html`, beside the pages it prints.
- **No one watches it** (section 5): the run starts by itself, by a
  timer. Its change brings the verdict to whoever starts look requests'
  goals, who proposes a repair to the person for each difference. The
  person's branch drops a request once its verdict is on main.
- **Retired when it no longer matters** (5.5): the run lists the
  journeys requests added whose looks landed as drawn, and the person
  picks which go, one change each.
- **The person's look is main's pages** (section 6): plain HTML in the
  repository, with `looks.html` saying whether each request rendered as
  drawn.

## 2. The request

As `../../client/web/look-requests.md`, 2: one commit, one change, with
its trailer, its holds, and its message saying every state in words.
What changes is where it is drawn and what its commit carries.

### 2.1 Where it is drawn

In this order:

1. **Main's pages,** where a journey's print shows the state: the
   request edits them in place (`../../client/web/journeys.md`, 7;
   `../../client/tui/journeys.md`, 10).
2. **A walk of the request's own,** where no journey shows it:
   - The request adds a journey's directory under its root: a `walk`
     of version 2, a `../../domain/README.md` and, where it needs the fake model's
     turns, `scripts.json` (`../../client/journey-walks.md`, 2.1).
   - The walk starts `after` the journey that comes nearest, so that
     its own lines are few (`../../client/journey-walks.md`, 2.3).
   - The person prints it with `cargo run -p temper-client-walk --
     print --out DIR NAME`, copies DIR's pages of NAME into its
     directory, and draws on them.
3. **A gallery state** (`../../client/web/gallery.md`), where the world
   can't set the facts: an account's usage, a long name, a box's front.
   - The gallery's pages carry no overlay. The person copies the page
     into a hand-made directory and draws there, as for the next rung.
   - The gallery's stylesheets are byte for byte the repository's own,
     so the copy links those, as a journey's pages do
     (`../../client/web/journeys.md`, 5.3), and there is no bundle to copy
     beside it.
4. **A hand-made page,** last, in a directory whose `../../domain/README.md` says
   it is hand-made and from what. No print meets it, so neither the
   comparison nor the render check compares it (4.3, 5.2).

- **Why a walk:** its sources land with the request (4.1), so:
  - the writer prints the state on main;
  - the goal compares against it exactly;
  - the check after landing renders it;
  - the next request draws on it.
- **What a walk reaches:** what the playground's world reaches by a
  person's acts (presses, typing, approvals) and the fake model's
  turns. Its facts are fixed (`../../client/journey-walks-world.md`, 1.1), so
  the same tree prints the same bytes. It prints no width: widths are
  the browser's, at the person's own check and the check after landing.

### 2.2 The person's tools

- **`temper-client-walk`,** built in the person's checkout with
  `TEMPER_INTERNAL_BUNDLE=none cargo build -p temper-client-walk`. The
  build takes minutes once and seconds after. It prints the walks of
  the checkout it is built from.
- **The rest as now** (`../../client/web/look-requests.md`, 2;
  `../../client/web/journeys.md`, 7):
  - the overlay, the pages' edits and the message;
  - the drawer's own check before committing, at 1440 and 420 px and
    each open state. This check is the drawer's, not a report;
  - `scripts/look-refresh.sh`, with its drop rule as 5.4 says.

### 2.3 Its commit

- **The walk's sources** go in the request's one commit, beside its
  drawn pages and its overlay's rules.
- **A neutral author:** the project's forge is public, so the commit
  names no person, as every commit there does.

## 3. Its goal

### 3.1 The word that starts it

- **"Land them":** the person says it to whoever they charge with
  starting look requests' goals. It approves the requests below the
  first held one (`../../client/web/look-requests.md`, 2, *Held, at the
  top*) that haven't landed, meaning no commit on main carries their
  `Look-landed` (4.4).
- **A goal each,** in one milestone, each after the one before it,
  oldest first, since each is drawn on the pages the earlier ones ask
  for. The person may say two run side by side, as they did on
  2026-11-19.
- **Its branch:** before its goal starts, the request's commit C goes
  to the project's forge as `docs/look/NAME`, a branch that never
  moves. It holds the requests below it too, as their branch does. A
  request redrawn after its goal started takes a new name.

### 3.2 Its words

From one template. M is the commit of main that the request's branch
stood on when C was copied.

```
<the request's subject without "Look request (temporary): ", at most 60 characters>

Land the look request NAME by temper/docs/design/client/web/look-requests.md,
which is the whole brief: its commit C, on the forge's docs/look/NAME, drawn
over main M. A walk it adds lands as its sources; its pages and overlay never
land. Compare with scripts/look-compare.sh NAME C.

<the request's message as its commit holds it, its trailers dropped>

Tests cheap: focused and fuzzy suites only.
```

### 3.3 Its questions

Whoever started it answers the goal's product questions from the
request's message. What the message doesn't decide goes to the person,
as one question whose choices are in the request's own terms.

## 4. The landing

As `../../client/web/look-requests.md`, 3, with these changes.

### 4.1 The walk lands

A walk the request adds lands as its sources: `walk`, `../../domain/README.md` and
`scripts.json`, as the request has them. A line changes only where the
landing renames a control the walk presses or a word it sees
(`../../client/journey-writer.md`, 2). Its pages never land: the writer
prints them after the landing.

### 4.2 The comparison

`scripts/look-compare.sh NAME C`, beside `look-refresh.sh`, run in the
landing's own tree. It runs the journeys tool's `look-compare NAME C`
as `look-refresh.sh` runs `look-refresh` (`../../client/journey-writer.md`, 4;
`../../client/journey-writer-requests.md`, 1; `../../testing/tools.md`, 2):
the command asks for each effect
as a value (a child with its arguments, a file) and decides on the
answer, and its exit is the script's.

1. **Reads the request:** it fetches `docs/look/NAME`, checks that it
   holds C, and reads C and its parent P. P holds the requests below,
   so P to C is this request's edit alone.
2. **Prints** the journeys whose pages C changes into scratch, in the
   landing's tree: `TEMPER_INTERNAL_BUNDLE=none cargo run -p
   temper-client-walk -- print --out DIR JOURNEY...`.
   - A walk that stops is a row, `stopped`, with its line.
   - If the landing's own change stopped it, the landing repairs it.
     Otherwise the account names it. A stopped walk holds no landing
     (`../../client/journey-writer.md`, 1).
3. **Compares each page three ways, with git:**
   - The three are the page as P holds it (what the person drew on),
     as C holds it (what they drew), and DIR's (what landed).
   - Each is first made comparable: the print's generated ids
     (`skein-ui-*`) and the references to them are numbered by their
     element's place on the page.
   - `git merge-file` applies the person's edit to DIR's page.
     - Where the result equals DIR's page, every edit landed.
     - Each hunk between the two is an edit that didn't land, or landed
       otherwise.
     - A conflict, where both the person and the landing changed one
       tag, is a row to account for by reading.
   - A page of a walk the request adds has no P. Its whole page is
     compared with DIR's. The world prints the same bytes from the same
     tree, so each difference is this landing's or another landing's
     since M, never the world's.
4. **Compares the rules:** the rules C adds, changes or removes in an
   overlay, each with its selector, its declarations and the `@media`
   around it.
   - Each is matched against the landed stylesheets' rules of that
     selector and condition: landed, differing (in which declarations),
     or missing.
   - A removed rule is landed when no landed rule keeps it.
   - The order the rules cascade in, and the TUI's rules, which land in
     code, its theme (`../../client/tui/journeys.md`, 10), are listed for
     the landing to account for by reading. The render check sees what
     the order does (5.2).
5. **Writes the account's rows** (`../../client/web/look-requests.md`,
   5): one per page edit, conflict, stopped walk and rule, with its
   result. There is also one row to fill for each state the message
   names in words. A request of rules alone gets rule rows alone.
6. **Its exit:**
   - 0, every edit and rule landed;
   - 1, a row differs, is missing, conflicts or stopped;
   - 2, its arguments, a branch that doesn't hold C, or a request it
     can't read.

It starts no browser, terminal or box, so a plan runs it
(`../../client/web/look-requests.md`, 3). Its rows are the account's
evidence. The account's judgement stays the landing's.

### 4.3 What it can't compare

A hand-made page has no print to meet (2.1). Its rows say so, as
accounts do today.

### 4.4 Its end

The landing's last commit, the one carrying its account, carries
`Look-landed: NAME` beside its `Look-request: NAME`. A repair of the
request after landing ends the same way. Until that commit is on main,
the request hasn't landed: a design commit carrying its trailer may
reach main before the landing's code does.

## 5. After landing

### 5.1 The run

- **It starts by itself:** a timer ticks every ten minutes, until
  temper runs the writer on each new tip
  (`../../client/journey-writer.md`, 8). A tick runs the writer's run
  (`../../client/journey-writer.md`, 4) when a commit carrying a
  `Look-landed` has reached main since its last finished run, and
  otherwise once an hour has passed since that run. So a verdict comes
  within ten minutes of its landing, and main's many other landings
  cost the worker one run an hour.
- **Skipping:** the run does nothing when main's tip is the tip of its
  last finished run. A finished run is one that wrote nothing, or whose
  offer landed. Waiting on its own offer, as now, finishes nothing, so
  the next tick tries again. One run at a time, as now.
- **What it does:** it prints in place, then checks the requests landed
  since (5.2). It offers the pages and the verdicts as one change.
  - A check that can't finish restores the written files, exits 1, and
    leaves the next tick to start again.
- **Its offer's key** names the offered commit, not only the tip. A
  retry of the same offer opens nothing twice, and a new offer at the
  same tip opens anew.

### 5.2 The render check

- **Which requests:** each request whose `Look-landed` a commit on main
  carries, where:
  - that commit is newer than the request's entry in `looks.html`, or
    the request has none;
  - its commit C is at `docs/look/NAME`;
  - the run's print of each journey it draws on printed or stayed fresh.

  A journey whose print stopped keeps the request pending, with no
  entry, until it prints.
- **Which pages:** each page C changes, and each page of C's journeys
  that C's rules select an element of. Each is rendered twice:
  - from C's tree, under C's overlay and stylesheets;
  - from the run's print at the same path, under the landed
    stylesheets and an empty overlay.

  A page with nothing beside it is listed as not compared, which means
  hand-made. A request whose rules select nothing is "nothing to
  render".
- **How:** in the Chromium the browser tiers use, through skein's
  driver, which gains three reads through its protocol: a page's
  elements, one element's computed style, and one element's box.
  `temper-client-journeys` runs it as a command of its own
  (`../../client/web/look-requests.md`, 8).
  - Widths: 1440, 1000 and 420 px. Every dialog and disclosure is set
    open, and nothing is clicked.
  - Elements: those the request touched, meaning each element its edit
    changed (4.2's hunks) or one its rules select, with their
    descendants. They are paired by the two pages' alignment, and an
    element with no pair is reported as such.
  - What is compared: each pair's computed style and box. A box within
    1 px is equal.
- **Its bounds:**
  - It renders files from scratch copies, with no credential in reach.
  - Every request outside `file://` is blocked.
  - The values it reports are escaped before they reach `looks.html`.
- **What it says:** identical, or each difference: the page, the
  element, the width, the property, and the request's value against
  the landed one.
- **States in words** aren't rendered: the account covers them, and the
  entry says so.
- **The TUI's** themed frames are rendered the same way, and the entry
  says these are the browser's frames, not a terminal's. Their text
  was compared at the landing (4.2). A real terminal's look stays the
  after-landing owner's (`../../client/tui/journeys.md`, 11).

### 5.3 The verdict

- **`looks.html`:** one page beside each root's journeys,
  `client/web/journeys/` and `client/tui/journeys/`.
  - It is written by the writer alone (`../../client/journey-writer.md`,
    2).
  - It holds an entry per check: the request's name, C, the
    `Look-landed` commit, the tip rendered at, its result, and links to
    its pages.
  - A newer `Look-landed` adds a newer entry, and the newest stands.
  - An entry lost is checked again at the next run.
- **Its three gates** take it as a written file:
  - the run's own list of what it may commit
    (`../../client/journey-writer.md`, 4);
  - the delivery check (`tools/pre-pr/design.md`, 4);
  - the merge job (`tools/merge/README.md`).

  Since the run waits on its own offer, its change meets no other
  change to `looks.html`.
- **It lands with the print:** the run's change holds the pages and
  `looks.html`, and its message ends `Printed: TIP`. A landing that
  changes no page still gives a change, its verdict.
- **Who hears it:** the run opens its change as whoever starts look
  requests' goals, and the change's end brings them the verdict. Each
  difference becomes a repair goal on the person's yes, carrying the
  request's trailer (`../../client/web/look-requests.md`, 4).
- **A walk that stops** is named in the change's message, as now
  (`../../client/journey-writer.md`, 4), and reaches the same reader. Its
  repair is a goal on the person's yes.

### 5.4 The person's branch

`look-refresh.sh` drops a request once main's `looks.html` holds an
entry for its newest `Look-landed`. A request that landed with no
`Look-landed`, before this draft, is dropped by `Printed:` as now
(`../../client/journey-writer-requests.md`, 1).

### 5.5 Retiring a journey

Each request drawn on a walk leaves a journey behind (2.1), and the
writer prints every journey at each run, about a second each. A journey
whose look no longer matters goes.

- **Candidates,** listed by the run at the foot of `looks.html`: each
  journey whose directory a request's landing added, whose request's
  newest entry is identical, and that no walk starts `after`
  (`../../client/journey-walks.md`, 2.3). A journey another starts
  `after` is never listed, since its going would stop the other.
- **Who picks:** whoever starts look requests' goals reads the list
  when the person next says "land them", or when the person asks, and
  proposes which go, in the request's own terms. The person says which.
  The list is a mechanism; which look still matters is a judgement.
- **How it goes:** one change deletes the journey's directory whole:
  its `walk`, `../../domain/README.md`, `scripts.json` and pages. The delivery check
  lets it through as written files (`tools/pre-pr/design.md`, 4, check
  8), and refuses a `walk` deleted while its pages stay, which no run
  would remove (`../../client/journey-writer.status.md`). The merge job
  settles a deletion at a written path already
  (`../../client/journey-writer.md`, 5).
- **Its look stays reachable:** in main's history, and in the request's
  commit at `docs/look/NAME`. A later request can bring its walk back.

## 6. What the person sees

- **Main's pages are the landed look:** each journey's pages under the
  two roots, plain files that open in a browser from a checkout. A
  request drawn on a walk leaves its state there for good.
- **`looks.html`,** a page of the same kind, says for each request
  whether it rendered as drawn and where it didn't, and links its
  pages.

## 7. Its tests

Focused and native: no browser, terminal or box starts.

- **The comparison's decisions,** step tests in
  `crates/temper-client-journeys-tool`'s library beside the refresh's:
  the `look-compare` command on canned answers (git's output, the
  print's lines and pages), then its rows and exit. Each of its
  effects is a value these steps answer, so its package's suite gains
  no launch of its own (`../../testing/tools.md`, 6). Cases:
  - every edit landed: exit 0;
  - an edit missing: its hunk is a row, exit 1;
  - one tag changed by both sides: a conflict row;
  - generated ids renumbered by a moved element: no row;
  - a rule landed, one differing, one missing, one removed;
  - a request of rules alone: rule rows alone;
  - a new walk's page compared whole;
  - a stopped walk: a row, exit 1;
  - the TUI's rules listed for reading;
  - a branch that doesn't hold C: exit 2.
- **The run's decisions,** step tests of the `writer` command on
  canned answers, the check's among them. Cases:
  - a landed request with no entry gets one, and a checked one is
    skipped;
  - a newer `Look-landed` is checked again;
  - a journey whose print stopped keeps its request pending;
  - a hand-made page is listed as not compared;
  - a landing that changes no page still offers its verdict, with
    `Printed:`;
  - a failed check restores the written files, exit 1;
  - main's tip unchanged since a finished run: nothing runs;
  - a new offer at the same tip gets a new key;
  - a journey a request added, its verdict identical and no walk
    `after` it, is listed as a candidate; one another starts `after`,
    or whose verdict differs, is not.
- **The refresh's decisions,** step tests of the `look-refresh`
  command: a request is kept until `looks.html` holds its
  entry, and dropped then. One with no `Look-landed` is dropped by
  `Printed:`.
- **The render check's reading of Chromium:** cases in a story skein's
  browser tier already has, in the Chromium session it already starts,
  on two `file://` pages: no new browser start and no new test binary.
  Written and compiled in the plan, not run. None goes in temper's
  browser tier, whose stories sit at their CPU budget's edge.
  - Two pages differing by one rule give one difference at each width.
  - Equal pages give none.
  - A request to a network address is blocked.
- **The three gates** take `looks.html` as a written file:
  - the run commits it beside pages;
  - the delivery check refuses it beside a source;
  - the delivery check lets a journey's directory deleted whole
    through, and refuses its `walk` deleted with its pages kept;
  - the merge job settles it as any written file.
- **What it costs other work:** a goal that isn't a look request's
  runs none of this. The delivery check and the merge job match one more
  file name. The new step tests run where the journeys tool's library
  is selected, each within the step layer's reference. Chromium and the
  print run on the after-landing worker alone, after landing.

## 8. Later

- **temper runs the writer** (`../../client/journey-writer.md`, 8), and
  the timer goes.
- **A gallery state as a walk:** once the world can set an account's
  usage or show a box's front, the gallery's states become walks, and
  2.1's third rung goes.
- **A walk at a width:** a mark printed at a width, should a request
  need the narrow layout's markup rather than its look.

## 9. Open questions

None.
