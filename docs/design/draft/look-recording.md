# Journeys: the client's pages, recorded

Draft, 2026-10-26, from a session with the user. A journey is a walk
through the browser's client recorded on a fresh `temper box
--fake-backends`, then printed natively as the client's own pages,
linked so that the same clicks walk them. The look is tuned on those
pages, over `temper.css`, and some journeys are committed and kept
current. Section 2 is the recording, 3 its replay, 4 the print, 5 the
look's edits, 6 a recording gone stale, 7 the first journey, 8 the work.

## 1. In one page

- **Why.** The mockups' pages and the client's are two markups: the
  mockups' own classes under `style.css` and the terminal sheets, and
  skein's `Page` with the `temper-` tokens under `temper.css`, which
  translates the first by hand (`client/browser.md`, 2.5). Every change
  to the look is translated again, and nothing shows that the client
  draws what the person saw. Comparing pictures (`draft/qa-web.md`)
  costs much and is inexact for this.
- **A journey is recorded in a browser:** everything the web shell's
  host is given, from the printed link's first page on, with each act
  as its control's role and name.
- **Its print is native and exact.** The shipped host, built natively,
  replays the recording and draws what the browser drew; skein's
  `Page` prints it as the gallery does. No browser, no box, the same
  bytes every time, times included.
- **Committed journeys stay current.** A test reprints each one and
  fails while its print differs, naming the command that reprints it,
  so a landing that changes a recorded page carries its new print.
- **The look is `temper.css`, on the print:** temper is the source of
  truth, and the mockups are reconciled to it. The pages link the shell's
  stylesheets from the checkout, then `overlay.css`, committed empty.
  The person tunes the look in the overlay and the markup in place;
  the diff is the request. It is done when the reprint equals the
  edited pages and the overlay is empty again.
- **A stale recording is recorded again.** A recording replays while
  the client asks the engine what it asked when it was recorded.
  Otherwise it is stale, and is recorded again on a fresh fake box.
- **The first journey** is the welcome's "Set up temper".

## 2. Recording

- **The recorder** is the web shell's, in a bundle built for it alone:
  `scripts/bundle.sh --loopback --record DIR`, the `record` feature of
  `temper-client-shell-web`, as `loopback-navigation` is. The shipped
  bundle has none. It adds nothing to the page, so the page is the
  shipped client's.
- **What it keeps:** the facts the host starts from (its seed, its
  clocks, the sign-in ways, the forges' sign-in facts, the address and
  the store's records), then, turn by turn, the turn's clocks and each
  report the host is given, in order: a due timer, an act, a place, a
  fetch's report as heard, the carried transport. With them, what the
  host asked each turn: its requests, its address shown or replaced,
  its deadline (section 3).
- **An act** keeps its control's role and accessible name, and its
  order among the page's controls with both ("Choose", the second).
- **A mark is a page as the person saw it.** Each act marks the page
  as it was drawn when the act came, keeping the live page's print
  (skein's `Page`) for the first replay's proof (section 3); the
  recording's end marks the page as it is then. One of the recorder's
  two keys marks the page with no act, for a state the page reaches by
  itself, such as a setup's progress; the other ends the recording and
  saves it as one file, through the browser's download.
- **The file** is text, one record per line, its kind first, bytes
  escaped, so that a recording reads in a diff.

## 3. Replay

- **Through the shipped host.** `temper-client-shell-web` builds its
  host for native as a library, as its `tests/host.rs` does for tests,
  so that the replay runs the host the bundle ships, never a copy. The
  replay starts it from the recording's facts and gives it each turn's
  clocks and reports in order, each act given to the node that carries
  its role and name on the page at that point.
- **The check:** each turn, what the host asks is what it asked when
  recorded. The recording's answers
  are to those requests; a client that asks otherwise (a new read,
  another request's bytes, another deadline) would be answered what it
  never asked. Then the journey is stale, as it is when an act's
  control is no longer on the page. Patches are not compared: they are
  what the print shows.
- **The first replay proves itself.** Its print at each mark is the
  live print the recording kept, or the recording is refused: the
  replay is not faithful, which is a defect to fix. Once they match,
  the live prints are dropped; the print is theirs from then on.

## 4. The print

`temper-client-journeys` (`tools/journeys`), a program of temper's
workspace beside the gallery, printing pages as it does
(`client/browser.md`, 7.1):

- `cargo run -p temper-client-journeys -- add NAME FILE` takes a
  saved recording as the journey NAME, replacing an earlier one of that
  name, replays it and prints it.
- `cargo run -p temper-client-journeys` reprints every journey from its
  recording, and names a stale one with the turn where it parted.
- It reads no configuration, environment, clock or network, and starts
  no server.

What it writes, under `docs/design/client/journeys/`:

- **A journey's directory,** `NAME/`: its `recording`, and a page for
  each mark, `01.html` onwards, in the marks' order.
- **A page** is the host's page at its mark: skein's `Page` with the
  shell's look, its root the document's `#app`, its links given the
  shell's hrefs and its title the shell's, as in the gallery. For a
  diff, one tag per line, each break inside a tag so that nothing drawn
  gains a space, and no `data-node`, whose numbers follow the order the
  nodes appeared in, not what they are.
- **Linked:** the control acted on at a mark carries the next page's
  name (`data-journey-next="02.html"`), and `journey.js` follows it on
  a click. Any other act on the page shows a note that the journey goes
  no further there, as the mockups' acts that lead to no drawn state
  do. What the browser does alone works as on the live page: hover,
  focus, a disclosure, typing, a narrower window.
- **The stylesheets:** each page links `skein-ui.css`, copied beside the
  journeys from `skein-ui-bundle` at each print, then the shell's
  `temper.css` and `task-pages.css` from the checkout itself, as
  `scripts/bundle.sh` orders them, then `overlay.css`. The bundler
  copies stylesheets and their `files/` as they are, so the look
  resolves as in the bundle, and a change to `temper.css` shows on the
  next reload, with no print.
- **Beside the journeys:** `overlay.css`, committed empty, and
  `journey.js`.
- **Opened through a server,** from the repository's root, since a
  browser refuses `temper.css`'s masks from a `file://` page:

  ```sh
  python3 -m http.server 8766
  # http://localhost:8766/docs/design/client/journeys/
  ```

**Its test,** in `tools/journeys`' own tests: every committed journey
reprinted into a scratch directory equals its print, byte for byte; one
that differs or is stale fails, naming the command that reprints. The
journeys together take at most 1 s of the default suite, as the
gallery's states do (`client/browser.md`, 7.3).

## 5. Editing the look

- **On the print:** the person serves the checkout, walks a journey,
  and writes rules in `overlay.css`, with the browser's tools (a
  DevTools workspace mapped to the checkout saves the Styles pane's
  edits to the file) or in an editor, then reloads. The markup is
  edited in place: a word, an order, a part moved or gone.
- **Selectors the view can carry:** the overlay selects by the
  `temper-` names, roles and regions, which the view sets, never by a
  part's position among its siblings.
- **The diff is the request:** the overlay's rules and the pages'
  changes, under `docs/design/client/journeys/`.
- **Done** is a landing that moves the overlay's rules into `temper.css`
  or `task-pages.css` and the markup's changes into the view, reprints,
  and empties the overlay. The reprint then equals the edited pages,
  compared as text, not as pictures; where it does not, the landing
  says why. What the edits imply beyond the pages recorded (the same
  part in a state no journey reaches) follows the mockups and the
  documents, as every change does (`client/mockups/README.md`, 1).

## 6. A recording gone stale

- **Recorded again, on a fresh fake box** (`../../guide/box.md`, "Trying
  the welcome with nothing real"), with the recording bundle, by the
  same acts, then `add NAME FILE`. The browser and the box are needed
  only when the client's exchange with the engine changes, never for a
  change to the view or the look alone.
- **A new recording reads anew:** its times are the new walk's, and so
  are the fake box's loopback ports wherever a page shows one, so the
  print's diff holds those as well as what changed.

## 7. The first journey

`welcome-set-up`, on a fresh `temper box --fake-backends`, as
`../../guide/box.md` walks it: the box's printed sign-in link opened, the
welcome drawn, "Set up temper" chosen. Two pages: `01.html`, the
welcome as it was when "Set up temper" was pressed, its "Set up temper"
leading to `02.html`, the Forge step's choices as they were when the
recording ended.

## 8. The work

1. **The host on native:** `temper-client-shell-web`'s host, with the
   modules it takes, built for native as a library as well as for the
   bundle.
2. **The recorder,** its feature and `scripts/bundle.sh --record`, its
   two keys and its file.
3. **`temper-client-journeys`:** `add` and the reprint, the replay's
   check, the first replay's proof, the print, `journey.js` and
   `overlay.css`.
4. **Its test,** within the budget of section 4.
5. **The first journey,** recorded on a fresh fake box and committed
   with its print.

## 9. What it changes

- **temper is the source of truth** for what it draws (the user,
  10-26). The existing mockups are reconciled to it gradually, and stay
  the design of what is not built yet.
- `client/browser.md`, 2.5: `temper.css` no longer takes its rules from
  the mockups' terminal sheets; the look is `temper.css`'s, tuned on
  the prints. Section 7's gallery stays for the states no journey
  reaches.
- `client/mockups/README.md`, 1, "Binding": the mockups bind what is
  not built yet. Where a mockup differs from what temper draws, the
  mockup is reconciled to temper.

## 10. Open

- **The gallery's states,** set from hand-written worlds: whether each
  moves to a journey once the fake box reaches it, its answers then the
  engine's own.
- **Later, if wanted:** fixed loopback ports for the fake box, so that a
  new recording differs from the last only where the client does; the
  recorder walking a stale recording's acts itself, so that the person
  only watches.
