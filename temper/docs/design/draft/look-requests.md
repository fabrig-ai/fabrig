# Look requests, end to end

Draft, 2026-11-04, from a session with the user that day, who asked
for every detail of a look request's round trip to be written down, so
that whoever lands one knows what to do and misses nothing. It keeps
`../client/web/journeys.md`, section 7, as it stands (the requests on the
person's `look-requests` branch, `scripts/look-refresh.sh`, nothing of
a request's own landed) and makes its "Done" a thing that can be
checked. Its design document is to be a client document of its own,
beside `../client/web/journeys.md`, whose section 7 then cites it.

## 1. In one page

- **A request says all it asks:** its pages and overlay show one state
  of each page; its message says, in words, every other state it
  touches, what it does not decide, and what is not the look's
  (section 2).
- **A landing takes the whole request:** every overlay rule and every
  page change, into `temper.css` and the view, for every state the
  request names, at every width, in the browser and the terminal
  (section 3).
- **The render check follows the landing:** the request's pages under
  their overlay against the reprint under the landed stylesheets, at
  three widths with each disclosure and menu open, on main; what differs
  goes back as a finding (section 4).
- **A landing accounts for each item:** a table in its last commit,
  each rule and change landed, differing with its reason, or deferred
  to a named owner; the plan is complete when it lands with no row left
  open (section 5).
- **What is not the look's is split off,** never dropped: a question to
  the user, or a draft, named in the account (section 6).
- **This document is the brief:** whoever is handed requests to land is
  pointed here, and needs nothing else to start (section 7).

## 2. The request

- **On the latest prints.** Before a request is drawn, the branch is on
  `origin/main` (`look-refresh.sh`) and a stale journey recorded again,
  so that the reprint can equal the request's pages.
- **One commit, one change,** subject `Look request (temporary): ...`,
  the `Look-request: NAME` trailer directly above the other trailers
  (`../client/web/journeys.md`, 7). "Temporary" is the commit's, never the
  change's: each request on the branch is to land, unless it is held.
- **Held, at the top.** A request the person is still weighing carries
  a `Look-hold: WHY` trailer beside its `Look-request`, and sits above
  every request to land; a request drawn on a held one's pages is held
  with it. Releasing one drops the trailer. One branch keeps them all
  on `main` through `look-refresh.sh`, and lets the requests above a
  held one build on its pages (the user, 2026-11-04).
- **Its message says, in words:**
  - what changes and why, in the person's terms;
  - each state the pages do not show that the change reaches (done,
    failed, open, typing, refused, empty, the narrow width), and how it
    looks there;
  - which old rules or parts the change replaces, for the landing to
    remove;
  - what is not the look's (an engine's fact, another plan's control),
    named as such (section 6).
- **A state no journey reaches** may be drawn on a hand-made page, in a
  directory of its own under `journeys/` whose `README.md` says it is
  hand-made and from what. Its values are written in by hand, and no
  reprint will equal it.
- **Checked before it is committed:** screenshots at 1440 and 420
  pixels wide, and of each open state through a throwaway copy.

## 3. The landing

In order, for each request on the branch, oldest first, stopping at
the first held one (section 2). A plan runs its focused and fuzzy suites
alone, starts no browser to compare, and is complete when it lands (the
user, 2026-11-06):

1. **Read it:** its own commit's diff (`git show`) and whole message;
   the requests below it are its base.
2. **Settle it with the design:** where the request departs from a
   design document, the landing changes that document with it; a
   conflict it cannot settle (a test that walks a part the request
   removes, a design choice it reverses) is a question to the user,
   never the landing's choice.
3. **Land the markup into the view:** each page change as the view's
   roles, names, words and look tokens, in every state the request
   names, not the recorded one alone.
4. **Land the rules into the stylesheets:** each overlay rule into
   `temper.css` or `task-pages.css`, selectors and declarations as the
   overlay has them, and the rules the request replaces removed.
5. **The terminal:** a region, heading or order the request moves or
   removes is checked in the terminal's prints, which keep their
   meaning: a heading the browser drops by style stays a heading.
6. **Reprint** every journey whose pages change, and record one again
   where it has gone stale (`../client/web/journeys.md`, 8).
7. **Account** (section 5), and bring the design text in line: the
   client's `README.md` and `../client/web/README.md`, the journeys' `README.md`
   files, `../client/web/journeys.md`, 9, and the mockups'
   `../client/web/mockups/STATUS.md`.

Each commit of the landing carries the request's trailer, a later fix
of what it missed included; one that lacks it hides the fix from
`look-refresh.sh` and from whoever reads the request's history.

## 4. The render check

After the landing, on main, outside the plan.

- **What is compared:** each page the request changes, from its own
  commit with the overlay as it stood there, against the same page
  reprinted with the landed stylesheets and an empty overlay.
- **How:** headless Chromium at 1440, 1000 and 420 pixels wide, with
  each disclosure and menu forced open, comparing each element's
  computed style and box, then the screenshots for what styles miss.
- **States in words** are rendered too: through a gallery state or a
  hand-made page, beside the request's words.
- **The result:** identical, or each difference a finding filed back,
  whose repair carries the request's trailer.

## 5. The account

The landing's last commit carries a table: one row per overlay rule,
per page change and per state the message names.

| Item | Result |
|---|---|
| a rule's selector, a page's change, a state | landed; differs, and why; or deferred, to whom and where it is tracked |

- **Done for the plan** is every row landed or explained. Its render-check
  row says that the check follows landing and names its owner; compilation
  and print checks do not establish rendered equality. After landing the
  check files every difference (section 4). A deferred row is carried by
  the design status or draft status that names its owner, not by the request.
- **A request is dropped** from the branch by `look-refresh.sh` once
  `main` carries its trailer; a deferred row survives it only where the
  account put it.

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

- **Whoever is handed requests** gets their names and this document,
  and nothing else is needed to start: the branch, the order, the
  checks and what done means are here. temper's `AGENTS.md` points
  here, for any agent that meets a request.
- **What comes back to the person:** for each request, the account and
  the questions of section 6 at its landing, then the render check's
  screenshots of each changed page beside the request's.
- **Each round teaches the next:** the person walks the landed pages;
  each miss is fixed under its request's trailer, and its cause becomes
  a line of this document (a thing a message must say, a step, a
  check), so that the next round does not repeat it.

## 8. Later

- **A render-check command:** in `temper-client-journeys`, taking a
  request's name, so that section 4 is one command rather than each
  landing's own script.
- **Journeys reaching further:** the fake box's Forgejo set up
  finishing, a walk across the ChatGPT sign-in’s departure, and the
  welcome’s Ready, so that fewer states are hand-made.

## 9. Open questions

None.
