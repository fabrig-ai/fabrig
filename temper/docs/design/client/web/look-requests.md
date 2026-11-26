# Look requests, end to end

A look request's round trip: drawing it, landing it, accounting for it and checking the rendered result.

## 1. In one page

- **A request says all it asks:** its pages and overlay show one state
  of each page; its message says, in words, every other state it
  touches, what it does not decide, and what is not the look's
  (section 2).
- **Drawn on a journey's print:** main's pages where a journey shows
  the state, or else a walk the request adds, printed by the person in
  seconds; a gallery state or a hand-made page last (section 2).
- **Requests stack on the person's branches:** the browser's on
  `look-requests`, the TUI's on `tui-requests`, so that the two can be
  asked for in parallel. The same `scripts/look-refresh.sh` keeps each
  on `origin/main` (`journeys.md`, 7; `../tui/journeys.md`, 10).
- **A goal each, read from a branch that never moves:** the person's
  "land them" starts one goal per request, oldest first. Before it
  starts, the request's commit is on the project's forge as
  `docs/look/NAME`, and the goal's words, from one template, pin it
  (section 7).
- **A landing takes the whole request:** every overlay rule and every
  page change into the stylesheets and view, for every state the
  request names, at every width, in the browser and the terminal. A
  walk it adds lands as its sources, never its pages, and its last
  commit says the request has landed (section 3).
- **Compared by a script, never committed:** whoever lands a request
  runs `scripts/look-compare.sh NAME C` in its own tree. It prints the
  request's journeys into scratch, compares each page three ways and
  the rules with the landed stylesheets, and writes the account's
  rows; main's pages follow from the journeys' writer
  (`../journey-writer.md`) after the landing (section 3).
- **Rendered after landing, with no one watching:** the journeys'
  writer's run renders the request's pages under their overlay beside
  main's print under the landed stylesheets, at three widths with every
  dialog and disclosure open, and writes the verdict into `looks.html`,
  which lands with the pages; each difference comes back as a repair
  (section 4).
- **A landing accounts for each item:** a table in its last commit,
  each rule and change landed, differing with its reason, or deferred
  to a named owner; the plan is complete when it lands with no row left
  open (section 5).
- **What is not the look's is split off,** never dropped: a question to
  the user, or a draft, named in the account (section 6).
- **This document is the brief:** a request's goal is pointed here by
  its words, and needs nothing else to start (section 7).

## 2. The request

- **Where it is drawn,** in this order:
  1. **Main's pages,** where a journey's print shows the state: the
     request edits them in place, as the journeys' writer last printed
     them (`../journey-writer.md`, 6). Before it is drawn, its branch is
     on `origin/main` through `look-refresh.sh`. A journey whose print
     lags a landing is the writer's to print in the world, never the
     person's.
  2. **A walk of the request's own,** where no journey shows it. The
     request adds a journey's directory under its root: a `walk` of
     version 2, a `README.md` and, where it needs the fake model's
     turns, `scripts.json` (`../journey-walks.md`, 2.1). The walk starts
     `after` the journey that comes nearest, so that its own lines are
     few (`../journey-walks.md`, 2.3). The person prints it with `cargo
     run -p temper-client-walk -- print --out DIR NAME`, copies DIR's
     pages of NAME into its directory, and draws on them.
  3. **A gallery state** (`gallery.md`), where the world can't set the
     facts: an account's usage, a long name, a box's front. The
     gallery's pages carry no overlay: the person copies the page into a
     hand-made directory and draws there, as for the next. The
     gallery's stylesheets are byte for byte the repository's own, so
     the copy links those, then `../overlay.css`, as a journey's pages
     do (`journeys.md`, 5.3), and there is no bundle to copy beside it.
     The directory's `README.md` names the state in the words
     `` the gallery's state `NAME` ``, and names no other, so that the
     render check renders its pages beside that state's print (section
     4).
  4. **A hand-made page,** last, in a directory of its own under
     `journeys/` whose `README.md` says it is hand-made and from what.
     Its values are written in by hand, and no journey's print will
     equal it.
- **Why a walk:** its sources land with the request (section 3), so the
  writer prints its state on main, the landing compares against it,
  the render check renders it, and the next request draws on it. A walk
  reaches what the playground's world reaches by a person's acts
  (presses, typing, approvals) and the fake model's turns. Its facts
  are fixed (`../journey-walks-world.md`, 1.1), so the same tree prints
  the same bytes. It prints no width: widths are the browser's, at the
  drawer's check and the render check.
- **The person's tools:** `temper-client-walk`, built in the person's
  checkout with `TEMPER_INTERNAL_BUNDLE=none cargo build -p
  temper-client-walk`, minutes once and seconds after, prints the walks
  of the checkout it is built from. The overlay, the pages' edits and
  `look-refresh.sh` are `journeys.md`, 7.
- **The diff is the request:** the overlay's rules, the pages' changes
  and the sources of a walk it adds. The browser's print, selectors,
  stylesheet and overlay files are `journeys.md`, 7; the TUI's frames
  and theme are `../tui/journeys.md`, 10.
- **One commit, one change,** subject `Look request (temporary): ...`,
  exactly one `Look-request: NAME` trailer directly above the other
  trailers. NAME is `[a-z0-9][a-z0-9-]*`, unique on its branch; the TUI's
  names start `tui-`. "Temporary" is the commit's, never the change's:
  each request on the branch is to land, unless it is held. A later
  request is drawn on the pages the earlier ones ask for.
  - **Its commit carries** the sources of a walk it adds, beside its
    drawn pages and its overlay's rules.
  - **A neutral author:** the project's forge is public, so the commit
    names no person, as every commit there does.
- **Held, at the top.** A request the person is still weighing carries
  a `Look-hold: WHY` trailer beside its `Look-request`, and sits above
  every request to land; a request drawn on a held one's pages is held
  with it. Releasing one drops the trailer. Whoever starts requests'
  goals reads these trailers and stops at the first held request
  (section 7). The refresh script keeps the requests on main by their
  `Look-request` names; requests above a held one can build on its
  pages.
- **Its message says, in words:**
  - what changes and why, in the person's terms;
  - each state the pages do not show that the change reaches (done,
    failed, open, typing, refused, empty, the narrow width), and how it
    looks there;
  - which old rules or parts the change replaces, for the landing to
    remove;
  - what it does not decide;
  - what is not the look's (an engine's fact, another plan's control),
    named as such (section 6).
- **Checked before it is committed,** by the drawer, as a check of
  their own and not a report: screenshots at 1440 and 420 pixels wide,
  and of each open state through a throwaway copy.

## 3. The landing

Each request is a goal of its own, started as section 7 says, its words
naming the request's commit C on the project's forge as
`docs/look/NAME`. A plan runs its focused and fuzzy suites alone,
starts no browser to compare, and is complete when it lands:

1. **Read it:** C's own diff (`git show C`) and whole message, fetched
   from `docs/look/NAME`. C's parent holds the requests below it, which
   are its base.
2. **Settle it with the design:** where the request departs from a
   design document, the landing changes that document with it; a
   conflict it cannot settle (a test that walks a part the request
   removes, a design choice it reverses) is a question to the user,
   never the landing's choice.
3. **Land the markup into the view:** each page change as the view's
   roles, names, words and look tokens, in every state the request
   names, not the recorded one alone. What the edits imply beyond the
   recorded pages is the view's for every state, and the mockups follow
   (`../mockups/README.md`, 1).
4. **Land the rules into the stylesheets:** each browser overlay rule
   into `temper.css` or `task-pages.css`, selectors and declarations as
   the overlay has them, and the rules the request replaces removed.
   The TUI's rules go into `temper-client-tui`'s theme, its frames'
   changes into its view and words (`../tui/journeys.md`, 10). The
   request's own commit, edited pages and overlay are never landed;
   main's overlays stay empty, which the delivery check holds
   (`../journey-writer.md`, 5).
   - **A walk the request adds lands as its sources:** `walk`,
     `README.md` and `scripts.json`, as the request has them. A line
     changes only where the landing renames a control the walk presses
     or a word it sees (`../journey-writer.md`, 2). Its pages never
     land: the writer prints them after the landing.
5. **The terminal:** a region, heading or order the request moves or
   removes is checked in the terminal's prints, which keep their
   meaning: a heading the browser drops by style stays a heading.
6. **Compare with `scripts/look-compare.sh NAME C`,** in the landing's
   own tree (`../journey-writer.md`, 6.1). It prints the journeys whose
   pages C changes into scratch with `temper-client-walk print`
   (`../journey-walks.md`, 5.1), a walk the request adds from its
   landed sources. It compares each page three ways with git, as C's
   parent holds it, as C holds it and as printed, and each rule C
   changes in an overlay with the landed stylesheets. Then it writes
   the account's rows (section 5), its exit 0 where every edit and
   rule landed. A row that differs is landed or explained; a conflict,
   the rules' cascade order and the TUI's rules are read. A walk that
   stops is repaired where the landing stopped it and named
   otherwise, never replaced by a comparison against main's earlier
   pages. A hand-made page is not compared: its values need not be a
   journey's. The print walks in a world, with no box or replay of a
   committed recording. The landing commits no page or file printed
   beside one: the journeys' writer writes main's pages after it lands
   (`../journey-writer.md`, 6).
7. **Account** (section 5), and bring the design text in line: the
   client's `../README.md` and `README.md`, the journeys' `README.md`
   files, `journeys.md`, 9, and the mockups' `../mockups/STATUS.md`.

Each commit of the landing carries the request's `Look-request: NAME`
trailer, a later fix of what it missed included; one that lacks it hides
the fix from `look-refresh.sh` and from whoever reads the request's
history. A goal's squash of the landing's change keeps the trailers of
its head commit's last paragraph (`../../protocol/tooling.md`, 4,
TRAILERS), so they reach main; the account above them stays with that
head, in the change's pull request. Where the landing departs, the
account says why, and later requests take the print as it is.

- **Its end:** the landing's last commit, the one carrying its account,
  carries `Look-landed: NAME` beside its `Look-request: NAME`. A repair
  of the request after landing ends the same way. Until that commit is
  on main, the request hasn't landed: a design commit carrying its
  `Look-request` may reach main before the landing's code does.

## 4. The render check

After the landing, outside the plan, with no one watching: the
journeys' writer's run, by a timer, within ten minutes of the commit
carrying the request's `Look-landed` reaching main, prints main's tip
and renders the request beside it with `temper-client-journeys
look-render NAME C` (`../journey-writer.md`, 4 and 6.2). A world print
compared byte for byte establishes no rendered equality with a
request; this does.

- **What is compared:** each page the request changes, and each page
  of a journey that its rules select an element of, from its own commit
  under its overlay and stylesheets, against the same page as the
  writer printed it, under the landed stylesheets and an empty overlay.
  A page drawn on a gallery state, its directory's `README.md` naming
  the state (section 2), is compared with that state as
  `temper-client-gallery` prints it from the same tree as the writer's
  print. Any other hand-made page, with nothing printed beside it, is
  listed as not compared.
- **How:** in the Chromium the browser tiers use, through skein's kit,
  from scratch copies, every request outside `file://` blocked; at
  1440, 1000 and 420 pixels wide, every dialog and disclosure open and
  nothing clicked. Each element the request touched, by its edit or its
  rules, with its descendants, is paired with the print's, and their
  computed style and box are compared, a box within 1 px equal.
- **States in words** aren't rendered: the account covers them
  (section 5), and the verdict says so.
- **The TUI's requests:** their themed frames are rendered the same
  way, as the browser's renders, not a terminal's; a replay and
  byte-equal print establish no real terminal's look, which stays
  temper/tui's (`../tui/journeys.md`, 11).
- **The result:** an entry in `looks.html` beside the request's
  journeys (`../journey-writer.md`, 6.3): identical, or each
  difference, by page, element, width and property, with the request's
  value against the landed one. It lands with the pages in the writer's
  change, opened as whoever starts look requests' goals, whose end
  brings them the verdict. Each difference becomes a repair goal on the
  person's yes, carrying the request's trailer (section 3); its last
  commit carries a new `Look-landed`, which the next run checks
  again.

## 5. The account

The landing's last commit carries a table: one row per overlay rule,
per page change and per state the message names.
`scripts/look-compare.sh` writes its rows (section 3, step 6): one per
page, hunk, conflict, stopped walk and rule, with its result, and one
to fill for the states. The landing fills that one, a row each, and
gives each row that differs, conflicts or is to read its result here.
The script's rows are the evidence; the judgement is the landing's.

| Item | Result |
|---|---|
| a rule's selector, a page's change, a state | landed; differs, and why; or deferred, to whom and where it is tracked |

- **Done for the plan** is every row landed or explained. Its render-check
  row says that the check follows landing, by the journeys' writer's run
  into `looks.html`; compilation and print checks do not establish
  rendered equality. Each difference the check finds comes back as a
  repair (section 4). A deferred row is carried by the design status or
  draft status that names its owner, not by the request.
- **A request is dropped** from its branch by `look-refresh.sh` once
  main's `looks.html` holds an entry for its newest `Look-landed`, or,
  for one landed with no `Look-landed`, once the last `Printed:` on main
  covers the commit carrying its trailer; a deferred row survives it
  only where the account put it (`journeys.md`, 7;
  `../journey-writer.md`, 6).

## 6. More than the look

- **An engine's fact:** where a request needs what the client is not
  given (an account's email, a set up's progress), the landing does
  what the view can, and the rest goes to the user as a question when
  it changes a stored format, a protocol or a public interface, or
  becomes a draft otherwise. The account names it, deferred.
- **Another plan's part:** where the drawn part belongs to a plan
  under way, the request is not done until that part carries the
  request's styles; the plan is told, and its commit that does it
  carries the trailer.

## 7. Who lands

- **"Land them":** the person says it to whoever they charge with
  starting look requests' goals. It approves the requests below the
  first held one (section 2) that haven't landed, meaning no commit on
  main carries their `Look-landed` (section 3).
- **A goal each,** in one milestone, each after the one before it,
  oldest first, since each is drawn on the pages the earlier ones ask
  for. The person may say two run side by side, as they did on
  2026-11-19.
- **Its branch:** before its goal starts, the request's commit C goes
  to the project's forge as `docs/look/NAME`, a branch that never
  moves. It holds the requests below it too, as their branch does. A
  request redrawn after its goal started takes a new name.
- **Its words,** from one template, M the commit of main that the
  request's branch stood on when C was copied:

  ```
  <the request's subject without "Look request (temporary): ", at most 60 characters>

  Land the look request NAME by temper/docs/design/client/web/look-requests.md,
  which is the whole brief: its commit C, on the forge's docs/look/NAME, drawn
  over main M. A walk it adds lands as its sources; its pages and overlay never
  land. Compare with scripts/look-compare.sh NAME C.

  <the request's message as its commit holds it, its trailers dropped>

  Tests cheap: focused and fuzzy suites only.
  ```

  Nothing in them is restated by hand: the request's commit says the
  rest, and this document is the whole brief. The branches, the order,
  the checks and what done means are here. temper's `AGENTS.md` points
  here, for any agent that meets a request.
- **Its questions:** whoever started it answers the goal's product
  questions from the request's message. What the message doesn't
  decide goes to the person, as one question whose choices are in the
  request's own terms.
- **What comes back to the person:** for each request, the account and
  the questions of section 6 at its landing, then its entry in
  `looks.html`, whether it rendered as drawn and where it didn't,
  beside main's pages (section 4).
- **Each round teaches the next:** the person walks the landed pages;
  each miss is fixed under its request's trailer, and its cause becomes
  a line of this document (a thing a message must say, a step, a
  check), so that the next round does not repeat it.

## 8. Later

- **temper runs the writer** (`../journey-writer.md`, 8), and the
  render check's timer goes with the script.

## 9. Open questions

None.
