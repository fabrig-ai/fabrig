# Journeys

Provisional, 2026-10-26, from the user's draft
(`../draft/look-recording.md`). A journey is a walk through the
browser's client (`browser.md`) recorded on a fresh `temper box
--fake-backends`, then replayed natively through the shipped host and
printed as the client's own pages, linked so that the same clicks walk
them. The look is tuned on those pages, and the committed journeys are
kept current by a test. They are not the mockups' journeys
(`mockups/README.md`, 4), which are worlds walked in the mockups.
Section 2 is the host on native, 3 the recording, 4 its replay, 5 the
print, 6 the tests, 7 editing the look, 8 a recording gone stale, 9 the
journeys, 10 what is open.

## 1. In one page

- **temper is the source of truth** for what it draws (the user,
  10-26). A page a journey records looks as its print does:
  `temper.css` over skein's sheet, tuned on the print. The mockups are
  reconciled to it gradually, and stay the design of what is not built
  yet (`mockups/README.md`, 1; `browser.md`, 2.5).
- **One seam, the host's.** The browser shell gives its host each
  report of a turn and carries out what the host asks back. The
  recorder keeps what crosses that seam; the replay gives it back to
  the same host, built natively (section 2).
- **Recorded in a browser,** in a bundle built for it alone, on a fresh
  fake box: the facts the host starts from, then turn by turn the
  clocks, each report and what the host asked after it, each act as its
  control's role, name and order, and a mark for each page the person
  saw (section 3).
- **Replayed, it must ask what it asked.** A client that asks otherwise
  would be answered what it never asked: the journey is stale, and is
  recorded again (sections 4 and 8). The first replay proves itself
  against the live page's prints.
- **Printed as static pages** by `temper-client-journeys`, under
  `docs/design/client/journeys/`: a page per mark, one tag per line, the
  acted control leading to the next page, the shell's stylesheets
  linked from the checkout, then an empty `overlay.css`. No browser, no
  box, the same bytes every time (section 5).
- **Kept current by a test** that reprints every committed journey and
  fails while a byte differs, naming the command that reprints, so a
  landing that changes a recorded page carries its new print
  (section 6).
- **The look is tuned on the print:** rules in the overlay, markup in
  place, the diff the request; a landing moves them into the shell's
  stylesheets and the view, reprints, and empties the overlay
  (section 7).
- **Fake only.** A recording keeps what the page was given as it was,
  the fake box's dummies included; none is made against real accounts
  (3.5).

## 2. The host on native

- **A library on every target.** `temper-client-shell-web` is a
  `cdylib` and an `rlib`. Its `host`, `limits` and `records` modules
  build for native as for `wasm32-unknown-unknown`, their dependencies
  no longer under the wasm target; `lib`'s start, the DOM and the
  platform's parts stay wasm-only, as do `wasm-bindgen` and `web-sys`.
  `tests/host.rs` uses the library instead of including the modules by
  path. The replay runs this host, never a copy.
- **The `journey` module** (the file, the recorder, the replay) builds
  natively, for the program and the tests, and in the browser only with
  the `record` feature: the shipped bundle holds none of it.
- **The seam** is `host::Host`'s, and the shell's turn and the replay
  call it alike:
  - `clocks(now, wall)`, at the turn's start, as today;
  - `report(&mut self, report: Report) -> Duties`: one report given,
    and what the host asks after it. `Report` is `Start`, `Due`,
    `Act { node, act }`, `Place { path }`, `Heard(Heard)` (a fetch's
    report, through jig's `adapter::fetched`) and
    `Carried { reports, writes }` (skein's `transport::Carried`).
    `Duties` holds the `asked` (jig's `adapter::Asked`), the `writes`
    to answer, and the address to `navigate` to, to `replace` with, or
    to `show`, with whether it replaces. What `Shell::turn` does
    between a report and the platform today, `duties()` and the
    `navigate`, `replace` and `show` fields, moves behind it;
  - `draw(&mut self, page, ops)`: the turn's patches applied to the
    page, and its links' hrefs set (`browser.md`, 2.3), as the turn's
    end does today;
  - `deadline(&self)`, as today.
- **The page's look** (`Look` with `look::NAMES` and `look::ELEMENTS`,
  the prefix `temper`) is one function of the host module's,
  `page(bounds, primary) -> Page<'static>`, which the shell's start and
  the replay call.
- **The view's tree,** `Host::tree(&self) -> &Tree`, is where an act's
  control is read and found again (3.3).

## 3. The recording

### 3.1 The recorder

- **In a bundle of its own:** `scripts/bundle.sh --loopback --record
  OUT` builds the shell with its `record` feature beside
  `loopback-navigation`; the shipped bundle has neither. The recorder
  adds nothing to the page: no node, no style, no request.
- **What it hears:** in `Shell::turn`, beside the host, the start's
  facts once, then each turn's clocks, each report given to
  `Host::report` with the duties it returned, and the turn's deadline.
  It keeps them in memory as the file's lines (3.2), through
  `journey::Recorder`, which is platform-independent: the wasm part
  only reads the keys and saves.
- **Two keys,** read by their `code` on the document before the page
  sees them, so that the page never does: Alt+Shift+M marks the page
  as it is drawn, with no act, for a state the page reaches by itself,
  such as a setup's progress; Alt+Shift+S ends the recording with its
  end mark and saves it through the browser's download, as
  `journey.recording`, from a link never added to the document. After
  the save the recorder keeps nothing more.
- **One load of the page.** A recording starts with the page, at the
  printed link's first page, and ends at its save. A departure ends it
  too: when a report's duties ask to `navigate` or `replace`, the
  recorder keeps them, marks the end as the page is drawn then, and
  saves, before the shell carries the departure out; the rest of that
  turn is not kept. A reload starts a new recording.

### 3.2 The file

Text, one record per line, its kind first, then its fields, each after
one space. A number is decimal and a word bare (a role, an act, `none`);
bytes are in double quotes, with `\\`, `\"`, and `\xHH` for a byte
outside printable ASCII (0x20 to 0x7e). There are no other lines: no
comment, no blank.

| Record | Fields | Where |
|---|---|---|
| `journey` | `1`, the format's version | the first line |
| `seed` | the platform's seed | the start's facts, in this order |
| `clocks` | now and wall, in nanoseconds | |
| `primary` | `control` or `meta` | |
| `ways` | the `temper-sign-in` meta's content, the page's origin | when the page has the meta |
| `forge` | a `temper-forge-sign-in` meta's content | one per meta, in order |
| `path` | the address's path and query | |
| `secret` | the printed link's secret | when the address held one |
| `stored` | a small-store record's key and value | one per record read at start |
| `turn` | now and wall | each turn's clocks |
| `start`, `due`, `place`, `heard`, `carried`, `act` | the report | each report, in its turn's order |
| `exchange`, `open`, `close`, `save`, `erase`, `writes`, `navigate`, `replace`, `show` | what the host asked | after its report, in order |
| `deadline` | nanoseconds, or `none` | each turn's end |
| `mark` | the live page's html | before its act, or between turns at the key |
| `end` | the live page's html | the last line |

- **The reports:** `place PATH`; `heard answered EXCHANGE STATUS BODY`,
  `heard failed EXCHANGE`, `heard opened`, `heard refused STATUS`,
  `heard piece BYTES`, `heard ended`; `carried WRITES`, then one line
  per carried report, `written KEY`, `unwritten KEY` or `lost
  EXCHANGE`; `act ROLE NAME ORDER ACT [VALUE]` (3.3).
- **The asks:** `exchange ID BOUND METHOD PATH BODY`, then its headers
  as `NAME VALUE` pairs on the same line; `open PIECE PATH` and its
  headers; `close`; `save KEY VALUE`; `erase KEY`; `writes N`, when not
  zero; `navigate TO`; `replace TO`; `show PATH push|replace`.
- **A committed recording's marks hold no html:** `add` drops it once
  the first replay has proved itself (4.3).

### 3.3 An act's control

- **Its role, name and order,** read from `Host::tree` before the act is
  given to the host: the node's role, as skein's fake person names it,
  with `-` for a space (`button`, `text-field`); its accessible name,
  the node's name or else its subtree's text, as the fake person reads
  it; and its order, from 1, among the tree's nodes with that role and
  exactly that name, in tree order (`act button "Choose" 2 press`).
- **The act** is skein-ui's: `press`, `submit`, `toggle`, `input` with
  its value, or `drawing` with its bytes.

### 3.4 Marks

- **A mark is a page as the person saw it:** as drawn when an act came,
  before the act's turn changes it; as drawn when the mark key was
  pressed; and at the recording's end. Each keeps the live page's
  `Page::html()`, for the first replay's proof.
- **Typing marks nothing.** An `input` act is recorded and replayed but
  marks no page: a printed field takes typing as the live one does
  (5.3), and the next act's mark shows what was typed.
- **The page's own state:** skein's page draws one state that its host
  never hears of, an open menu (`Role::Menu`), and a mark taken with
  one open fails the first replay's proof. temper's view draws no such
  menu (its banner's "Menu" is a disclosure, the browser's own); one
  that does will need the page's events recorded too (section 10).

### 3.5 Fake only

A recording is made on a fresh `temper box --fake-backends`
(`../../guide/box.md`, "Trying the welcome with nothing real") and
keeps what the page was given as it was: the printed link's secret, a
typed value, the fakes' loopback addresses, all of them the fake box's
dummies, spent or gone with the box. Recording against real accounts
is not designed (section 10).

## 4. Replay

### 4.1 Through the shipped host

`journey::Replay`, native, starts the host from the recording's facts
as the shell's start does (`Host::new`, `forge_sign_in`, `start`) and
the page by `host::page`. For each turn it sets the clocks, gives each
report through `Host::report`, an act to the node its control names on
the page at that point, and draws at the turn's end. At each mark it
yields the page and, for an act's mark, the node acted on.

### 4.2 The check

- **Each report's duties,** written as their records, are the recorded
  lines, and each turn's deadline is the recorded one. The recording's
  answers are to those requests: a client that asks otherwise (a new
  read, another request's bytes, another deadline) would be answered
  what it never asked.
- **Stale** is where the replay stops: at the first duty or deadline
  that differs, a departure asked or not asked among them, or at an
  act whose control is not on the page. It names the turn, the
  recorded line and what the host asked instead.
- **Patches are not compared:** they are what the print shows.

### 4.3 The first replay's proof

`add` replays with the live prints: at each mark the replay's
`Page::html()` is the recorded one, or the recording is refused,
naming the mark and the first byte that differs. Such a replay is not
faithful, and the defect is fixed in the seam or the replay, never
recorded around. Once every mark matches, the live prints are dropped:
the print is the replay's from then on.

## 5. The print

### 5.1 The program

`temper-client-journeys` (`tools/journeys`), a program of temper's
workspace beside the gallery (`browser.md`, section 7):

- `cargo run -p temper-client-journeys -- add NAME FILE` takes a saved
  recording as the journey NAME, replacing one of that name, proves it
  (4.3) and prints it. A NAME is `[a-z][a-z0-9-]*`.
- `cargo run -p temper-client-journeys` reprints every journey from its
  recording. A stale one is named with its turn and line, its print
  left as it was, and the program exits 1 once the others are printed.
- It reads no configuration, environment, clock or network, and starts
  no server. Its paths are its manifest's, as the gallery's are.

### 5.2 What it writes

Under `docs/design/client/journeys/`:

- **A journey's directory,** `NAME/`: its `recording`, and a page for
  each mark, `01.html` onwards, in the marks' order, replaced whole at
  each print.
- **Beside the journeys:** `skein-ui.css`, skein-ui-bundle's own, and
  `journey.js`, the program's, both written at each print; and
  `overlay.css`, the person's, made empty when missing and never written
  otherwise.
- **Generated:** the pages, `skein-ui.css` and `journey.js` are
  generated files, owned by `scripts/regenerate.sh`, which rewrites them
  by the reprint, so that a rebase that conflicts in them is resolved
  by reprinting (agents' `landing-flow.md`, 3). A recording and the
  overlay are sources, never regenerated.

### 5.3 A page

- **The host's page at its mark:** `Page::html()` with its root the
  document's `#app` and its dialogs `open`, under the shell's title,
  the main heading then "temper", as the gallery prints a state
  (`browser.md`, 7.1).
- **The document:** `<!doctype html>`, `html` with `lang="en"`; in its
  head the charset, the viewport, the title, the four stylesheets and
  `../journey.js`; in its body the page.
- **For a diff:** one tag per line, the break inside the tag before its
  `>`, so that nothing drawn gains a space; and no `data-node`, whose
  numbers follow the order the nodes appeared in, not what they are.
  skein's `dom::print` escapes `<`, `>`, `"` and `&` in text and
  attribute values alike, so the tags of `Page::html()` are found
  exactly.
- **Linked:** the control acted on at a mark carries the next page's
  name, `data-journey-next="02.html"`, and `journey.js` follows it on a
  click. Any other press, link or submit shows a note that the journey
  goes no further there, as the mockups' acts that lead to no drawn
  state do. What the browser does alone works as on the live page:
  hover, focus, a disclosure, typing, a narrower window.
- **Its neighbours:** each page names the pages before and after it in
  its head (`<link rel="prev">`, `<link rel="next">`), and `journey.js`
  follows them on `p` and `n` outside a field. A page whose mark had no
  act, the mark key's, leads on by these alone.
- **The stylesheets,** by paths relative to the page:
  `../skein-ui.css`, then `temper.css` and `task-pages.css` from
  `crates/temper-client-shell-web/` in the checkout, in
  `scripts/bundle.sh`'s order, then `../overlay.css`. The bundler copies
  stylesheets and their `files/` as they are, so `temper.css`'s masks
  resolve beside it in the checkout as in the bundle, and a change to it
  shows on the next reload, with no print.

### 5.4 Opening them

Through a server, from the repository's root, since a browser refuses
`temper.css`'s masks from a `file://` page:

```sh
python3 -m http.server 8766
# http://localhost:8766/docs/design/client/journeys/
```

## 6. Its tests

- **The seam, in `temper-client-shell-web`,** natively, in the host's
  world of `tests/host.rs` (the platform's reports given by the test):
  a walk recorded by `journey::Recorder` and replayed; the file's lines
  read back to the records written, each escape among them; stale at a
  changed ask, a changed deadline and a control gone; and the proof
  refusing a mark whose html differs.
- **The print, in `tools/journeys`' own tests:** every committed
  journey reprinted into a scratch directory equals its directory and
  the files beside it, byte for byte; one that differs or is stale
  fails, naming `cargo run -p temper-client-journeys`.
- **The recorder's keys and download** have no browser story: the first
  journey's recording proves them, and the browser tier proves only
  what a browser alone shows (`browser.md`, section 4).
- **Their cost:** the journeys' reprint together at most 1 s of the
  default suite, as the gallery's states (`browser.md`, 7.3); no fuzzy
  test.

## 7. Editing the look

- **On the print:** the person serves the checkout, walks a journey, and
  writes rules in `overlay.css`, with the browser's tools (a DevTools
  workspace mapped to the checkout saves the Styles pane's edits to the
  file) or in an editor, then reloads. The markup is edited in place: a
  word, an order, a part moved or gone.
- **Selectors the view can carry:** the overlay selects by the `temper-`
  names, roles and regions, which the view sets, never by a part's
  position among its siblings.
- **The diff is the request:** the overlay's rules and the pages'
  changes, under `docs/design/client/journeys/`.
- **Done** is a landing that moves the overlay's rules into `temper.css`
  or `task-pages.css` and the markup's changes into the view, reprints,
  and empties the overlay. The reprint then equals the edited pages,
  compared as text, not as pictures; where it does not, the landing
  says why. What the edits imply beyond the pages recorded (the same
  part in a state no journey reaches) is the view's and `temper.css`'s
  for every state, and the mockups follow (`mockups/README.md`, 1).

## 8. A recording gone stale

- **Recorded again, on a fresh fake box** (3.5), with the recording
  bundle, by the same acts, then `add NAME FILE`. The browser and the
  box are needed only when the client's exchange with the engine
  changes, never for a change to the view or the look alone, which a
  reprint carries.
- **A new recording reads anew:** its times are the new walk's, and so
  are the fake box's loopback ports wherever a page shows one, so the
  print's diff holds those as well as what changed.

## 9. The journeys

| Name | The walk | Its pages |
|---|---|---|
| `welcome-set-up` | on a fresh `temper box --fake-backends`, as `../../guide/box.md` walks it: the printed sign-in link opened, the welcome drawn, "Set up temper" pressed | `01.html`, the welcome as it was when "Set up temper" was pressed, which leads to `02.html`, the Forge step's choices as they were when the recording ended |

## 10. Open questions

- **Typing:** the draft has every act mark its page; the design marks
  none for an `input` (3.4), since a page per keystroke would make a
  walk of typing. The user's word.
- **The gallery's states,** set from hand-written worlds: whether each
  moves to a journey once the fake box reaches it, its answers then the
  engine's own (`browser.md`, section 7).
- **The page's own state at a mark** (3.4): where a journey meets a
  skein menu, the page's events would be recorded and replayed beside
  the host's reports.
- **Later, if wanted:** fixed loopback ports for the fake box, so that a
  new recording differs from the last only where the client does; the
  recorder walking a stale recording's acts itself, so that the person
  only watches; and the one-tag-per-line print as skein-ui-web's, for
  another application's journeys.
- **Real accounts:** a recording against a box with real accounts would
  need its secrets kept out (the printed link's, a typed key's); not
  designed.
