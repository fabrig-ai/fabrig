# Journeys

Provisional, 2026-10-26. A journey is a walk through the
browser's client (`README.md`) recorded on a fresh `temper box
--fake-backends`, or on a real box and redacted (3.5), then replayed
natively through the shipped host and printed as the client's own pages,
linked so that the same clicks walk them. The look is tuned on those
pages, and the committed journeys are kept current by a test. They are not the mockups' journeys
(`../mockups/README.md`, 4), which are worlds walked in the mockups.
The TUI's journeys are `../tui/journeys.md`'s, sharing this document's
file (3.2), redaction (3.6) and last scan (3.7). Section 2 is the host on native, 3 the recording, 4 its replay, 5 the
print, 6 the tests, 7 editing the look, 8 a recording gone stale, 9 the
journeys, 10 what is open.

## 1. In one page

- **temper is the source of truth** for what it draws (the user,
  10-26). A page a journey records looks as its print does:
  `temper.css` over skein's sheet, tuned on the print. The mockups are
  reconciled to it gradually, and stay the design of what is not built
  yet (`../mockups/README.md`, 1; `look.md`).
- **One seam, the host's.** The browser shell gives its host each
  report of a turn and carries out what the host asks back. The
  recorder keeps what crosses that seam; the replay gives it back to
  the same host, built natively (section 2).
- **Recorded in a browser,** in a bundle built for it alone, on a fresh
  box, fake or real (3.5): the facts the host starts from, then turn by
  turn the clocks, each report and what the host asked after it, each act as the
  event the view makes of it, never its control's words or place, and a
  mark for each page the person saw (section 3).
- **Replayed, it must ask what it asked,** and a control on the page
  must give each recorded event. A client that asks otherwise would be
  answered what it never asked: the journey is stale, and is recorded
  again (sections 4 and 8). The view may rename, restyle or move a
  control and the journey holds. The first replay proves itself against
  the live page's prints.
- **Recorded again by a command** while its acts hold: `again NAME`
  walks them on a fresh fake box in headless Chromium, as the browser
  tier drives it, and adds the result; no person walks it (8.1). A
  journey that starts past the first page says how to reach it in a
  setup file beside its recording (8.2).
- **Printed as static pages** by `temper-client-journeys`, under
  `docs/design/client/journeys/`: a page per mark, one tag per line, the
  acted control leading to the next page, the shell's stylesheets
  linked from the checkout, then an empty `overlay.css`; beside them the
  journey's `../README.md`, its description. No browser, no box, the same
  bytes every time (section 5).
- **Kept current by a test** that reprints every committed journey and
  fails while a byte differs, naming the command that reprints, so a
  landing that changes a recorded page carries its new print. It also
  fails while `overlay.css` holds anything (section 6).
- **The look is tuned on the print:** rules in the overlay, markup in
  place, each request a commit on the person's branch `look-requests`,
  kept on `main` by `scripts/look-refresh.sh` and never landed. A
  landing that carries a request's name moves it into the shell's
  stylesheets and the view, and reprints (section 7).
- **Fake, or real and redacted.** A recording keeps what the page was
  given as it was. On a real box `add` first takes out what must not be
  committed, proves the rest, and writes nothing until a last scan has
  passed; the scan knows what it knows, and the person reads the pages
  for the rest (3.5 to 3.8).

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
    page, and its links' hrefs set (`README.md`, 2.3), as the turn's
    end does today;
  - `deadline(&self)`, as today.
- **The page's look** (`Look` with `look::NAMES` and `look::ELEMENTS`,
  the prefix `temper`) is one function of the host module's,
  `page(bounds, primary) -> Page<'static>`, which the shell's start and
  the replay call.
- **An act's event,** `Host::event_of(&self, node, act) ->
  Option<Event>`, the event the view makes of an act on a node, without
  giving it: `temper_client_view::event(&View, node, act)` reads the
  view and changes nothing, so that the recorder and the replay may ask
  it of any node. The host's own act gives what it returns. With the view's tree, `Host::tree(&self) ->
  &Tree`, it is where an act is read and its node found again (3.3).

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
  `journey::Recorder`, which is platform-independent and wraps the
  seam: `Recorder::report(host, page, report) -> Duties` gives the
  report to the host and keeps both, what the shell is asked to do with
  a tab among them, and a tab the browser refused as a report (`../pages/README.md`
  , `ConnectModel`, The sign-in's tab), and `Duties::departs()` says when
  the duties leave the page. The shell and the host's tests call it
  alike; the wasm part only reads the keys and saves.
- **Two keys,** read by their `code` on the document before the page
  sees them, so that the page never does: Alt+Shift+M marks the page
  as it is drawn, with no act, for a state the page reaches by itself,
  such as a setup's progress; Alt+Shift+S saves the recording as it is
  then, closed by an end mark of the page as drawn, through the
  browser's download, as `journey.recording`, from a link never added to
  the document. The recorder goes on after a save, so that each save is
  a whole recording and a later one holds more; the person saves once,
  at the walk's end, and `again` saves as it goes (8.1). A key's turn that gives the host nothing, a save's, is not
  kept.
- **One load of the page.** A recording starts with the page, at the
  printed link's first page, and is whole at each save. A sign-in is no
  departure: the recording goes on through it and its end, the tab's
  address kept as its answer's is. A departure, a forge's or an
  authorization's, ends it: when a report's duties ask to `navigate` or `replace`, the
  recorder keeps them, marks the end as the page is drawn then, and
  saves, before the shell carries the departure out; the rest of that
  turn is not kept, and the recorder keeps nothing more. A reload starts
  a new recording.

### 3.2 The file

Text, one record per line, its kind first, then its fields, each after
one space. A number is decimal and a word bare (a role, an act, `none`);
bytes are in double quotes, with `\\`, `\"`, and `\xHH` for a byte
outside printable ASCII (0x20 to 0x7e). There are no other lines: no
comment, no blank.

| Record | Fields | Where |
|---|---|---|
| `journey` | `2`, the format's version | the first line |
| `seed` | the platform's seed | the start's facts, in this order |
| `clocks` | now and wall, in nanoseconds | |
| `primary` | `control` or `meta` | |
| `ways` | the `temper-sign-in` meta's content, the page's origin | when the page has the meta |
| `forge` | a `temper-forge-sign-in` meta's content | one per meta, in order |
| `path` | the address's path and query | |
| `secret` | the printed link's secret | when the address held one |
| `stored` | a small-store record's key and value | one per record read at start |
| `turn` | now and wall | each turn's clocks |
| `start`, `due`, `place`, `heard`, `carried`, `act`, `copied` | the report | each report, in its turn's order |
| `exchange`, `open`, `close`, `save`, `erase`, `writes`, `navigate`, `replace`, `show`, `copy` | what the host asked | after its report, in order |
| `deadline` | nanoseconds, or `none` | each turn's end |
| `mark` | the live page's html | before its act, or at the key, in a turn of its own (`turn`, `mark`, `deadline`) |
| `end` | the live page's html | the last line |

- **The reports:** `place PATH`; `heard answered EXCHANGE STATUS BODY`,
  `heard failed EXCHANGE`, `heard opened`, `heard refused STATUS`,
  `heard piece BYTES`, `heard ended`; `carried WRITES`, then one line
  per carried report, `written KEY`, `unwritten KEY` or `lost
  EXCHANGE`; `act ACT [VALUE] EVENT NAME` (3.3); `copied TOKEN
  done|refused|failed`.
- **The asks:** `exchange ID BOUND METHOD PATH BODY`, then its headers
  as `NAME VALUE` pairs on the same line; `open PIECE PATH` and its
  headers; `close`; `save KEY VALUE`; `erase KEY`; `writes N`, when not
  zero; `navigate TO`; `replace TO`; `show PATH push|replace`; `copy
  TOKEN WORDS`, whose words a redaction replaces as it does a `navigate`'s.
- **A committed recording's marks hold no html:** `add` drops it once
  the first replay has proved itself (4.3).

### 3.3 An act's event

- **The act** is skein-ui's: `press`, `submit`, `toggle`, `input` with
  its value, or `drawing` with its bytes.
- **Its event** is the one the view makes of it, read through
  `Host::event_of` before the act is given: for a press, the client's
  action, such as a `Go` to an address, in the domain's own names. It is
  kept as the domain `Event`'s Debug print, compact, as bytes, which the
  replay compares and nothing parses. Not the control's role, words or
  place, so that the view may rename, restyle or move it and the
  recording holds; a change to the domain's names, or to how Rust prints
  them, stales it (section 8).
- **The control's accessible name follows,** for a reader: the node's
  name, or else its subtree's text, as skein's fake person reads it. It
  is never checked.
- **An act the view makes no event of** is given to the host, which
  does nothing with it, and kept nowhere: no line, no mark.
- The welcome's "Set up temper" is kept as `act press "Act { action:
  Pages(Go { address: ... }) }" "Set up temper"`; a typed secret's event
  shows its length alone (`AccountKey { value: Secret { len: 12 } }`),
  its value being the act's until `add` takes it out (3.5, 3.6).

### 3.4 Marks

- **A mark is a page as the person saw it:** as drawn when an act came,
  before the act's turn changes it; as drawn when the mark key was
  pressed; and at the recording's end. Either way it is the page as the
  last turn's end drew it, so the replay takes a mark wherever it sits. Each keeps the live page's
  `Page::html()`, for the first replay's proof.
- **Typing marks nothing.** An `input` act is recorded and replayed but
  marks no page: a printed field takes typing as the live one does
  (5.3), and the next act's mark shows what was typed.
- **The page's own state:** skein's page draws one state that its host
  never hears of, an open menu (`Role::Menu`), and a mark taken with
  one open fails the first replay's proof. temper's rail draws two such
  menus, and no journey opens one (on a page that is no project's, its
  banner's "Menu" is a disclosure, the browser's own); a journey that
  does will need the page's events recorded too (section 10).

### 3.5 Fake, or real and redacted

A recording is made on a fresh `temper box --fake-backends`, started
as the help's `box` page starts one on fake backends (`temper help
box`), its printed sign-in link opened in the browser, and keeps what
the page was given as it was: the printed link's secret, the box's printed command, a
typed value, the fakes' loopback addresses, all of them the fake box's
dummies, spent or gone with the box.

Or by hand on a fresh box with real accounts, where a few things are
hard to show without real models (section 9 names such a journey real).
The box serves the recording bundle with `application.clients.bundle`
set in its configuration; it takes no flag and says nothing more. The
person uses public repositories and writes nothing private into the
chats: the model's words are kept as they were given. `add` redacts
every recording alike, fake or real (3.6).

### 3.6 Redaction

`add` redacts before its first replay's proof (4.3). It reads the
recording as its lines (3.2), and every byte field of each. It writes
nothing and prints no original.

- **The originals:**
  - *link:* the `secret` record's value;
  - *stored:* each `stored` record's value, and each `save` of the
    stream's record (jig's `STREAM_KEY`), the same credential as it is
    issued. Such a value is a `Kept`: its head stands, so that the host
    still reads it, and its `record` is the original. A value that isn't
    one is an original whole;
  - *typed:* each value of an act whose event shows a secret's length
    (`Secret { len: N }`, 3.3), such as a typed key or a pasted sign-in;
  - *given:* each `--redact TEXT`;
  - *approval*, *credential* and *bearer:* the TUI's, the approval's
    secret, the credential the TUI takes and the bearer every exchange
    carries (`../tui/journeys.md`, section 5), which a browser's recording
    never gives. They are sought as every original is, and reach the
    shared code from the TUI's lines through `Originals::new` and
    `redact_field`.
- **The placeholder** is `x`s, one for each byte, so that every frame,
  length prefix and base64 run keeps its size. Surrounding spaces, tabs,
  CRs and LFs of a typed or given original stand, outside it: the client
  trims them from a sign-in's paste before it carries the value, and a
  key's field carries them with it.
- **Where:** every original is sought in every field, longest first, in
  these forms: its bytes; percent-encoded, either hex case, for a URL's
  query; `dom::print`'s escapes, for a mark's html; and Rust's Debug
  escapes, for an event's print. Its sha-256, with those spaces and
  without them, is sought as well and becomes the placeholder's:
  `AddKey`, `ReplaceKey` and `FinishSignIn` carry the digest of the value
  they carry.
- **A form's size:** in a framed field (a `heard answered` body, a
  stream's record, an `exchange` body, a `save` or `stored` key or
  value, and `written`, `unwritten` and `erase`) a form becomes `x`s of
  its own length. In any other field, which no length prefix counts, it
  becomes the placeholder.
- **Typing:** typing is given an act at a time (8.1), so a typed value
  shorter than 4 bytes, or the head of a longer typed value, is replaced
  in its own act alone, and sought nowhere else. The view draws a secret
  field's value empty, so no mark holds it.
- **A stream's pieces are joined first.** A `heard piece` is a fetch's
  chunk: it may end in the middle of a `data:` line, an original or a
  base64 group. Between an `open` or `heard opened` and the next, the
  pieces are one stream of bytes, cut into its lines; each `data:` line's
  canonical base64 is decoded to its record, replaced there, encoded
  back, and the bytes cut again where the pieces ended, so that the
  replay is given the chunks it was given. A `data:` line that doesn't
  decode refuses the recording, by the line of the piece it starts on. A
  last line that the recording's end cuts short is decoded by its whole
  groups, the rest kept as text.
- **The rest is the proof's:** the host derives what it asks from what
  it heard, so a consistent replacement replays as the original did. An
  ask is proved, then: what is left of a typed value in an exchange
  differs from what the replay asks, and is stale (4.2). A report is
  only replaced and scanned (3.7).
- **What `add` prints:** the count of each kind of original, and of the
  places replaced.
- **Not done, the user's word, 2026-11-05:** `temper box --bundle` on
  real backends and its line under the sign-in block, since a real box
  already serves the recording bundle; email addresses, neither replaced
  nor refused, since the account's email may stand in a committed
  journey.

### 3.7 The last scan

Before anything is written, `add` scans what it would write, and what
the replay drew on the way.

- **Where it looks:** every field of each line, a stream's records
  decoded and its other text among them; each run's text and its
  writing call's input, joined by their offsets (`RunText`,
  `WritingCall`; `jig/docs/design/protocol/client/README.md`, 5), since a
  model's stream splits a token across deliveries that no one line
  shows; the page the
  replay drew at each turn's end, its escapes read back, so that what a
  page joins is seen, and each printed page.
- **What it refuses:**
  - *an original:* any form of a sought original (3.6), or its sha-256;
  - *a key:* `sk-`, then at least 20 of `[A-Za-z0-9_-]`;
  - *a JWT:* three base64url parts joined by `.`, the first beginning
    `eyJ`, each at least 8 long;
  - *a bearer:* `Bearer `, any case, then at least 16 of
    `[A-Za-z0-9._~+/=-]`;
  - *a code:* `code=`, not after a letter or digit, then at least 8 of
    `[A-Za-z0-9._~%+/-]`;
  - *a refresh token:* `refresh_token`, up to four of `"`, `\`, `:`, `=`
    and spaces, then at least 8 of `[A-Za-z0-9._~%+/-]`.
- **A value of `x`s alone is a placeholder,** and passes.
- **A key inside a word is none:** `sk-` after two bytes of
  `[A-Za-z0-9_-]` (`task-pages`) doesn't match. One such byte may be a
  field's length prefix, which reads as a letter or a digit: a 51-byte
  key's reads `3`.
- **A refusal** names the record and the kind, never the bytes:
  the line and its record's kind, and where in it (a field, a stream's
  record, a run's text) or the turn or page that drew it. Nothing is
  written, a journey of that name stands as it was, and no temporary file
  is made. An original is taken out by `add`, so a refusal of one is a
  split or a fault; a shape is taken out by a `--redact`, and `add` run
  again.

### 3.8 What is left to the person

The scan's shapes aren't all a secret could be. `--help` says so, and
`add` says it again after its counts. Stated here so that a person knows
before they record:

- **Any other private text** (a name, a host, a token of another shape)
  stands unless a `--redact` names it: read the pages before committing.
- **Split across deliveries:** a run's text or a writing call's input is
  seen, joined, and refused, but not replaced: a `--redact` can't reach
  it, and the journey is walked again. Text split over any other records
  is seen only where a page draws it whole at some turn's end; if none
  does, it isn't seen.
- **Part of a secret** isn't sought: a key's last four on a page, or an
  original's head cut off by the recording's own end.
- **Other encodings** aren't sought: an original base64-encoded inside a
  field, say. No client exchange compresses what it carries.
- **A value the client reshapes** before it carries it isn't found whole.
  A GitHub App's key has its armour's line breaks restored, so its
  exchange is stale and the recording refused: such a step stays out of
  a real journey.
- **A `stored` request:** a request's `Kept` record, in flight at a
  reload, doesn't read as `x`s, so the replay goes stale and the
  recording is refused: reload a settled page.
- **What the person holds:** the saved FILE keeps every original, so it
  stays out of the repository and goes once it is added; a `--redact
  TEXT` stands in the shell's history.

## 4. Replay

### 4.1 Through the shipped host

`journey::Replay`, native, starts the host from the recording's facts
as the shell's start does (`Host::new`, `forge_sign_in`, `start`) and
the page by `host::page`. For each turn it sets the clocks, gives each
report through `Host::report`, an act to the first node, in tree order,
on the page at that point of which the view makes the same event
(3.3), and draws at the turn's end. At each mark it yields the page
and, for an act's mark, every node on it that gives the act's event.

### 4.2 The check

- **Each report's duties,** written as their records, are the recorded
  lines, and each turn's deadline is the recorded one. The recording's
  answers are to those requests: a client that asks otherwise (a new
  read, another request's bytes, another deadline) would be answered
  what it never asked.
- **Stale** is where the replay stops: at the first duty or deadline
  that differs, a departure or a tab's open, point or close, asked or not
  asked, among them, or at an
  act whose event no control on the page gives. It names the turn, the
  recorded line and what the host asked instead.
- **Patches are not compared:** they are what the print shows.

### 4.3 The first replay's proof

`add` replays the redacted recording (3.6) with its redacted live
prints: at each mark the replay's `Page::html()` is the recorded one,
or the recording is refused, naming the mark and the first byte that
differs. Such a replay is not faithful, and the defect is fixed in the
seam or the replay, never recorded around. Once every mark matches, the
live prints are dropped: the print is the replay's from then on.

A refusal of `add`'s names the turn, the line and each record's kind:
never a line's text, which may hold what the redaction missed. The
reprint names them whole (5.1), its recordings having passed the scan.

## 5. The print

### 5.1 The program

`temper-client-journeys` (`bin/temper-client-journeys`), an internal binary
whose crate is `crates/temper-client-journeys-tool`, beside the gallery
(`gallery.md`; `../../testing/tools.md`, 3–4):

- `cargo run -p temper-client-journeys -- add [--redact TEXT]... NAME
  FILE` takes a saved recording as the journey NAME, replacing one of
  that name, redacts it (3.6), proves it (4.3), scans it (3.7) and prints
  it, in memory until the scan has passed. A NAME is `[a-z][a-z0-9-]*`.
- `cargo run -p temper-client-journeys -- add --help` says what `add` takes
  out and what it leaves to the person (3.8).
- `cargo run -p temper-client-journeys` reprints every journey from its
  recording. A stale one is named with its turn and line, its print
  left as it was, and the program exits 1 once the others are printed.
- `cargo run -p temper-client-journeys -- again NAME` records the
  journey NAME again while its acts hold (8.1).
- `add` and the reprint read no configuration, environment, clock or
  network, and start no server; `again` alone does. Its paths are its
  manifest's, as the gallery's are.

### 5.2 What it writes

Under `docs/design/client/journeys/`:

- **A journey's directory,** `NAME/`: its `recording`; its
  `../README.md`, the journey's description, what it walks and what each
  mark shows, and its `setup`, when it has one (8.2), which `add` and the
  reprint leave as they find them; and a page for each mark, `01.html`
  onwards, in the marks' order, replaced whole at each print.
- **Beside the journeys:** `skein-ui.css`, skein-ui-bundle's own
  (`skein_ui_bundle::STYLESHEET`, skein's `ui/web/README.md`, section 10), and
  `journey.js`, the program's, both written at each print; and
  `overlay.css`, the person's, made empty when missing and never written
  otherwise.
- **Generated:** the pages, `skein-ui.css` and `journey.js` are
  generated files, rewritten by the reprint, so that a rebase that
  conflicts in them is resolved by reprinting. A recording, a README, a
  setup and the overlay are sources, never regenerated. A stale journey fails the
  reprint: such a change waits until the journey is recorded again
  (section 8).

### 5.3 A page

- **The host's page at its mark:** `Page::html()` with its root the
  document's `#app` and its dialogs `open`, under the shell's title,
  the main heading then "temper", as the gallery prints a state
  (`gallery.md`, 1).
- **The document:** `<!doctype html>`, `html` with `lang="en"`; in its
  head the charset, the viewport, the title, the four stylesheets and
  `../journey.js`; in its body the page.
- **For a diff:** one tag per line, the break inside the tag before its
  `>`, so that nothing drawn gains a space; and no `data-node`, whose
  numbers follow the order the nodes appeared in, not what they are.
  skein's `dom::print` escapes `<`, `>`, `"` and `&` in text and
  attribute values alike, so the tags of `Page::html()` are found
  exactly.
- **Linked:** each control on a mark's page of which the view makes the
  act's event carries the next page's name,
  `data-journey-next="02.html"`, and, for an act other than a press, the
  act, `data-journey-act="submit"` or `"toggle"`: the act on any of them
  would have led there. `journey.js` follows a press's on a click, a
  submit's on its form's submit (Enter in its field), and a toggle's on
  its change; a click into a field to type in it never follows. Any other press, link or submit shows a note that the journey
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
  stylesheets and their `files/` as they are, so `temper.css`'s images
  and fonts resolve beside it in the checkout as in the bundle, and a
  change to it shows on the next reload, with no print.

### 5.4 Opening them

As files: a journey's `01.html` opened in the browser from the
checkout. Nothing on them is fetched under CORS, which a `file://` page
fails, since `temper.css` draws its icons as background images, never
masks (`look.md`). A server from the repository's root draws
them the same:

```sh
python3 -m http.server 8766
# http://localhost:8766/docs/design/client/journeys/
```

## 6. Its tests

- **The seam, in `temper-client-shell-web`,** natively, in the host's
  world of `tests/host.rs` (the platform's reports given by the test):
  a walk recorded by `journey::Recorder` and replayed; the file's lines
  read back to the records written, each escape among them, and another
  version unreadable; stale at a changed ask, a changed deadline and an
  event no control gives; a control's recorded name changed, holding;
  an act with no event, kept nowhere; and the proof refusing a mark
  whose html differs.
- **The print, in `crates/temper-client-journeys-tool`'s own tests:** every
  committed journey reprinted into a scratch directory equals its directory and
  the files beside it, byte for byte; one that differs or is stale
  fails, naming `cargo run -p temper-client-journeys`. A reprint keeps a
  journey's README.
- **No request lands as it is:** the same tests fail while
  `overlay.css` holds any byte, and for a journey whose `../README.md` is
  missing or empty (section 7).
- **The links by act,** in the print's unit tests: a press's control,
  a submit's field and a toggle's box, each with its attributes.
- **`again`'s walk, natively,** in the journey module's tests and the
  host's world, with no browser: a recording's walk read as its steps
  (each typed act one, a drawing and a moving key mark refused); a save
  replayed to its end finding the control of a step's event by its
  role, name and order; a still page told from a moving one by two
  saves, an exchange or a write in flight among them; and the check
  refusing a new walk whose acts, values, events or key marks differ.
- **A setup, natively,** in the journey module's tests and
  `crates/temper-client-journeys-tool`'s own: a setup's lines read back, each
  record among them, and another version, a line out of its place, a second `reload`
  and an unknown role each unreadable; `after` joining a named setup's
  steps without its `reload`, and a chain of two, a missing name and a
  name of its own refused; every committed setup read; and
  `project-landing`'s first steps, replayed against `welcome-set-up`'s
  recording, each naming a control of which the view makes that act's
  event, so that the two starts stay one.
- **`again`'s browser half has no story of its own,** as the recorder's
  keys have none: its first run, a journey recorded again, proves it,
  and skein's browser tier proves the kit's letter with Alt and its
  downloads (skein's `testing/browser.md`, section 9). No default test starts a
  browser or a box.
- **The recording bundle stays built:** the gate's browser-target
  check builds the shell with `loopback-navigation` and `record` too.
- **The recorder's keys and download** have no browser story: the first
  journey's recording proves them, and the browser tier proves only
  what a browser alone shows (`testing.md`). That a save
  leaves the recording going on is the recorder's, tested natively: two
  saves of one walk, the first a whole recording that the second
  extends.
- **The redaction, natively:** each kind of original (3.6) in each place
  it can appear: a plain field, a piece's base64, split across two pieces
  at every byte, and a URL's query, each failing first. The host's world
  walks a typed key, pressed, through `add`'s steps, so that its digest
  and its stream's name go and the replay holds. A committed journey
  redacted replays as it stands. Test data is fake, a token's shape built
  in the test from pieces. No test reaches a real host or account.
- **The scan, natively:** each shape in each place, a run's text split
  across deliveries among them; a length prefix of every byte before a
  key; and what must pass: `x`s, `task-` words, `code_challenge=`, and
  the committed journeys.
- **The binary's table,** in `bin/temper-client-journeys`' suite: help,
  version, the internal commands' pages and argument refusals, with no build
  or child started (`../../testing/tools.md`, 6). Its `arguments` and
  `refusals` process tests live there too; the library and look-refresh
  script tests stay in `crates/temper-client-journeys-tool`.
- **The refusal, as a process:** no original's bytes on stdout or stderr,
  a failed proof's among them, and nothing written on any path; in the
  print's tests, a refused `add` leaves its root as it was.
- **Their cost:** the journeys' reprint together at most 1 s of the
  default suite, as the gallery's states (`gallery.md`, 3); no fuzzy
  test.

## 7. Editing the look

The request's message, holds, landing, account and after-landing render
check are `look-requests.md`, sections 1 to 7. The journeys' side:

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
- **Requests stack** on a branch of the person's own, `look-requests`,
  one commit each, named by exactly one `Look-request: NAME` trailer
  (NAME `[a-z0-9][a-z0-9-]*`, unique on the branch), which a rebase keeps.
  A later request is drawn on the pages the earlier ones ask for. They
  are done in order, each
  read by its name on the branch as it is then, as its own commit's
  diff.
- **Kept on `main`:** `scripts/look-refresh.sh`, run by the person in
  their checkout with `look-requests` checked out and clean, fetches
  `main` alone, rebases the branch on `origin/main`, drops each request
  whose name a commit on `main` carries in that trailer, and pushes it
  with `--force-with-lease`, its lease the branch as the checkout last
  knew it, so that a branch moved from another checkout is refused,
  never overwritten. It says, for each request, whether it was kept or
  dropped and by which commit. Nothing is built on the branch, so
  moving it breaks no one, and its one name leaves no copies behind. A
  conflict is a line of a request's that `main` reprinted since: the
  script stops with the rebase in progress, naming the request; the
  person settles it and finishes the rebase (`git rebase --continue`),
  then runs the script again, which refuses while a rebase is in
  progress. It takes the TUI's branch, `tui-requests` (`../tui/journeys.md`,
  10), alike, with that branch's lease, requests and push; any other
  branch is refused.
- **Never landed:** the journeys' test refuses a request's pages, which
  differ from the reprint, and its overlay (section 6), so `main`'s
  overlay stays empty.
- **The landing** moves the overlay's rules into `temper.css` or
  `task-pages.css` and the markup's changes into the view, for every
  state, then reprints (`look-requests.md`, 3). Its own commits carry
  the request's trailer, never the request's commit itself. The account
  and what done means are `look-requests.md`, 5; the render comparison
  follows landing on main (`look-requests.md`, 4).

## 8. A recording gone stale

A recording goes stale only when the client's exchange with the engine
changes, or what the client can do on a page (an event no control
gives); never for a change to the view or the look alone, which a
reprint carries, since its acts are events, not controls.

- **While its acts hold,** `again NAME` records it again (8.1). No
  person walks it.
- **A real journey** (section 9) is never walked by `again`, which starts
  fake boxes alone: its person records it again by hand (3.5).
- **A setup that no longer walks** (a control renamed before the reload)
  stops `again`, naming its line; it is edited, and no recording changes
  for it (8.2).
- **When the walk itself changes** (a step gone, another added), the
  journey is walked anew in a browser on a fresh fake box (3.5), from
  its `../README.md` and the change that moved it, then added (`add NAME
  FILE`). The README changes with it, and its new pages are read as new.
- **A new recording reads anew:** its times are the new walk's, and so
  are the fake box's loopback ports wherever a page shows one, so the
  print's diff holds those as well as what changed (section 10).

### 8.1 Again

`cargo run -p temper-client-journeys -- again NAME`, in
`crates/temper-client-journeys-tool`. Its walk's logic is the journey module's,
beside the recorder and the replay, natively and without io: it takes each save's
bytes and answers what to do next. The program reads the saves from the
person's downloads and drives the box and the browser, as ordinary Rust
over skein's harness.

- **The walk** is the journey's recording read as its steps, in order:
  each person's act, with its value and event, each key's mark, and the end.
  An `input` is a step (typing is given an act at a time, so that the new
  walk asks what the old one asked after each). A `near` act is the page
  part's report, no step: a page reading by itself reads again on the new
  walk. It keeps no print mark before it, as an `input` keeps none, but
  still counts as a report when deciding whether the page is still or a
  key's mark caught it moving. Replay gives it as recorded (section 4). A
  `drawing` act is no person's to give, and a key's mark that caught the
  page moving (a report other than a due within half a second after it)
  is no state a still page shows: such a journey is refused before
  anything starts, and walked anew.
- **A fresh fake box** (3.5): `temper box --fake-backends`, built from
  the checkout with the recording bundle (`scripts/bundle.sh --loopback
  --record`), running a copy of the built binary and its fake-claude,
  every directory fresh in a scratch directory under `/tmp`;
  its printed sign-in link opened.
- **Headless Chromium, as the browser tier drives it** (`testing.md`
  ): skein's browser kit through the fake person's browser face and
  harness (skein's `ui/web/testing.md`, 2), Chromium from
  `SKEIN_TEST_CHROMIUM`, the person's downloads into the scratch
  directory.
- **The client first:** a page's load comes before the client in it
  starts, so the first save waits for the client's main region, which
  it draws once its recorder hears the keys.
- **Each step once the page is still.** A save (Alt+Shift+S) is the walk
  so far, a whole recording. The page is still when a save half a second
  after another (a second, before a key's mark) holds no report the other
  lacks but dues, no exchange asked but not heard, and no write asked but
  not carried. A page not still within 30 seconds stops `again`, naming
  the step. Then:
  - **an act:** the still save is replayed natively to its end
    (section 4); the first node outside a menu (`Role::Menu`, which no
    journey opens, 3.4) of which the view makes the step's event names
    its control by its role, its accessible name and its order,
    from 0, among the page's controls with both, and the face finds that
    control on the live page (the fake person's n-th match, skein's
    `ui/README.md` 14.1) and acts on it as a person: a press, a submit, a
    toggle, or the field's value selected and the act's value typed;
  - **a key's mark:** Alt+Shift+M; its two saves are three seconds apart, since a fake box's set up pauses for over a second;
  - **the end:** the still save is the new recording.

  A step whose event no control gives, in a still save, within 10
  seconds stops `again`: its acts no longer hold.
- **The check, then `add`:** the new recording's walk is the old one's,
  the same acts with their values and events in order and the key marks
  in their places, or `again` refuses, naming the first step that
  differs, and the journey is walked anew. The page may change between
  a still save and the act given; an act that then lands on a control
  making another event fails the check, and one that makes the same
  event is the same act. Then `add NAME FILE`, its first replay proving
  it (4.3), its README left as it was.
- **What it reads and starts:** the environment's Chromium, the clock
  and the loopback network; the box and Chromium, both stopped at its
  end, its scratch directory removed unless it failed, which it then
  names with what failed.

### 8.2 A setup

A journey whose recording starts past the first page has an optional file,
`setup`, beside its `recording`: what the person did on the fresh box before
the reload where the recording starts. `again` walks it first, by itself, and
the journey is then recorded as 8.1 has it. No setup, and `again` starts
at the printed link's first page, as 8.1 has it.

Text, one record per line, as the recording's (3.2): its kind first, then its
fields, each after one space, a word bare, bytes in double quotes.

| Record | Fields | What `again` does |
|---|---|---|
| `setup` | `1`, the format's version | the first line |
| `after` | a journey's name | the second line, or absent: that journey's steps come first, without its `reload`; the journey it names has no `after`, and isn't this one |
| `press`, `submit`, `toggle` | a role, a control's accessible name, then its order from 0, when it isn't 0 | finds that control on the live page and acts as a person does (8.1) |
| `input` | a role, a name, a value, then its order, when it isn't 0 | the field's value selected and the value typed |
| `see` | text | waits until the page shows the text |
| `reload` | a path and query | once, last of the steps: loads that address on the box's origin; the recording starts there, as any load starts one (3.1) |
| `mark` | a key mark's number in the walk from 1, text | after `reload`, in order, one to a mark: at that step of the walk, waits until the page shows the text, then goes on as 8.1 |

- **By role and name,** as a person finds a control, and never an event:
  nothing before the reload reaches the recording, which proves nothing of
  it. A role is a word of skein's roles, lowercase: `button`, `link`,
  `textbox`, `checkbox`.
- **Shared, once:** `project-landing`'s setup walks a fresh box to Home in
  Hello, and `chat-draft`'s is `after project-landing`, what it waits for
  on Home, and its own `reload`. `welcome-set-up`, with no setup, walks
  that start in its recording, and a test holds the two together
  (section 6).
- **Why a mark waits:** the page may stand still for seconds while a fake
  model's turn goes on, and two saves a second apart can't tell that
  from settled, where the person could (8.1). A `mark` names what the
  person waited for. A key mark with none falls as 8.1 has it.
- **Its bounds:** a step of a setup not done within 60 seconds stops `again`,
  naming its line and what it sought; a `mark`'s text not shown within 30
  seconds stops it, naming the mark.
- **A longer run than a test's:** `again` sets the harness's horizon (skein's
  `ui/web/testing.md`, 2) to ten minutes, since each of its steps has its own bound
  and a walk like `chat-draft`'s holds more than forty seconds of saves. It
  is a person's command and in no tier; every test keeps the harness's forty.
- **No secret:** a setup is committed as it stands, and `add`'s scan doesn't
  read it. A step that needs a secret is no fake box's.

## 9. The journeys

| Name | The walk | Its pages |
|---|---|---|
| `welcome-set-up` | on a fresh `temper box --fake-backends` (3.5): the printed sign-in link opened, the welcome drawn, "Set up temper" pressed, then Forgejo's "Run on this machine" pressed on the Forge step, which opens Forgejo's sheet, then the sheet's "Run on this machine", an empty repository chosen, which makes the forge and sets it up (`../pages/README.md`) | `01.html` and `02.html`, the welcome, the second as it was when "Set up temper" was pressed, which leads to `03.html` and `04.html`, the Forge step's choices, the second as it was when "Run on this machine" was pressed, which leads to `05.html` and `06.html`, Forgejo's sheet, the second as it was when its "Run on this machine" was pressed, which leads to `07.html` and `08.html`, the Model step, the set up running on, the second as it was when the recording ended; its `../README.md` says so |
| `welcome-playground` | on a fresh `temper box --fake-backends` (3.5): the printed sign-in link opened, the welcome drawn, "Try the playground" pressed | `01.html`, the welcome as it was when "Try the playground" was pressed, which leads to `02.html`, `NewChat` in the Demo project, the playground started, its banner's one line above its main, as it was when the recording ended; its `../README.md` says so |
| `project-landing` | on a fresh `temper box --fake-backends` (3.5): its `setup` (8.2): Forgejo run on this machine with an empty `work/hello`, the ChatGPT plan signed in, Ready's offered "Hello" created and its repository adopted; once Home says "work/hello is in Hello", `/projects/1` reloaded, where the recording starts, and the settled Home saved | `01.html`, Home in Hello with the ChatGPT plan, Forgejo, "work/hello is in Hello" and To begin's one step left, "Tell temper how you work", the rail's "Home" and "Ask temper" alone, and the phone's "Home", "Ask" and "Settings" tabs; the recording holds no act; its `../README.md` says so |
| `chat-draft` | on a fresh `temper box --fake-backends` (3.5): its `setup` (8.2), `after project-landing`, up to `/projects/1/settings` reloaded, "Chats can write drafts" on as a fresh project has it; then a chat started, "+ Draft" pressed, two turns of the fake model writing and changing its document (`../../testing/real-loop.md`), Split, Draft, then "Start goal…" and its goal's page, then its Draft link back to the chat | a chat in Chat, Split and Draft, the sheet, the goal's page and the return to the chat (`../../domain/agent/agents.md`, 3; `../pages/README.md`) |

A journey recorded on a real box says so in its walk here, as `real`,
and in its `../README.md`; none is yet (3.5).

The `configure-chat-draft` request's hand-made `configure-chat/02.html`
through `04.html` and its README are not journeys to reprint or land
(section 7). Its document pane and modes are shared with `chat-draft`;
Ask temper's proposal states, Start over and the rail strip are also
drawn by the gallery (`gallery.md`, 2). Its drawing and figure are a flow
(`skein/docs/design/ui/markdown.md`, 1), drawn in the client; the gallery measures
no box, so a print shows its fallback.
The `configure-temper-own-way` request's hand-made `configure-own-way/`
pages are not journeys to reprint or land either; what they draw, the
card and the stock reference beside the page, the request
`ask-temper-ways` replaces.

The `ask-temper-ways` request's hand-made `ask-temper-ways/01.html`
through `04.html` are not journeys to reprint or land (section 7): no
journey reaches the ways, and the gallery draws them and the states its
message names in words (`gallery.md`, 2, the `ask-ways-*` states).

The `bundle-loading` request's hand-made `bundle-loading/01.html` through
`04.html` are not journeys to reprint or land (section 7): they draw the
placeholder before the client starts (`look.md`).

The hand-made `chat-work/01.html` through `04.html` are the look request's,
not journeys to reprint or land (section 7). The gallery's isolated
`chat-work-*` states draw their conversation and their additional states
through the view (`gallery.md`, 2). Their summary follows the first
appearance of a tool, so settings precedes docs where it was called first.
The conversation is read by its log's edges (`../pages/chat/conversation.md`, 2), with no
Older control. `chat-draft`'s return to the chat holds its first activity
page until browser measures read more. The remaining live text and the
terminal and browser coverage departures are tracked in
`../../draft/chat-reading.status.md`, sections 3 and 5, and a numbered
list's look in `look.status.md`.

The `close-chat` request's hand-made `close-chat/01.html` through `03.html`
are not journeys to reprint or land (section 7). Their states are the gallery's
`chat-closing`, `chat-closing-work` and `chat-closed` (`gallery.md`, 2).
The other confirmations and failures use the same sheet (`../README.md`, 3.6);
their gallery states include Start over, Close, Accept, Reject and
Release, with its uncertain pair. These add no recorded journey or page.

The `model-picker` request's hand-made `model-picker/01.html` through
`07.html` are not recorded journeys to reprint or land (section 7). Its
Settings, Models page is drawn by the gallery's `models`, `models-add`,
`models-off`, `models-added`, `models-member` and `models-one-account`
states (`gallery.md`, 2). The account in `model-catalogue-2/models-proof`'s
last commit names each page 07 change and Settings overlay rule. The render
comparison at 1440, 1280, 1000 and 420 px and its screenshots follow landing,
owned by temper/ui (`look-requests.md`, 4).

The `chat-title` request's `chat-title/01.html` is a recorded chat with its
heading changed by hand, not a journey to reprint or land (section 7). No
recorded journey has a first prompt past its first sentence, so no print
changes with it; the view's table holds the states its message names in
words (`../pages/README.md`, *A chat's heading is short*). temper/ui's render
check follows landing (`look-requests.md`, 4).

The `connect-flows` request’s hand-made pages 01–23 are not journeys to
reprint or land (section 7). They draw Accounts, model connection,
Forges, forge connection and Settings’ “Add repositories” sheet, including
search, empty, copy and failed-copy states. The gallery draws these through
the shipped view (`gallery.md`, 2), and the two entry paths into the sheet
the request left undrawn, as `sheet-from-home` and `sheet-from-chats`
(`../forge/family.md`, 3). No request page, `connect.css` or
`adopt.js` is part of the print. The welcome’s existing recordings still
replay and reprint; the Settings sheet browser stories compile within the
plan. temper/ui compares the request with main after landing, at the
widths and open states in `look-requests.md`, 4.

The `chat-terminal` requests' hand-made `chat-terminal/01.html` through
`06.html`, their `terminal.css`, `terminal.js` and `fonts/`, are not
journeys to reprint or land (section 7): page 01 is a recorded chat with
">_", and 02 to 06 the terminal over it. The gallery's `chat-terminal-*`
states draw them through the view, and the states their messages name in
words (`gallery.md`, 2). The fonts are the bundle's (`look.md`).
`chat-draft`'s pages reprint with ">_" in their bar. temper/ui compares
the requests with main after landing (`look-requests.md`, 4).

The `ask-temper-start` request's hand-made `ask-temper-start/01.html` and
`02.html` are not journeys to reprint or land (section 7): a new project's
Home, whose "Start" and rail's "Ask temper" open Ask temper's page, and that
page before a first message, as `ui-ask-m1` landed them (`../pages/README.md`,
*To begin* and *Ask temper's page*). The request adds nothing to them; its
account matches each state with what main draws, through the gallery's
Home and `ask` states (`gallery.md`, 2). The `to-begin-two-steps` request
draws its two steps and its rail over those pages; `project-landing`'s
`01.html` reprints with them, and the gallery's `home-begin-*` states draw
the states its message names in words.

The `night-mode` request's hand-made `night-mode/01.html` to `03.html` and
its `night.js` are not journeys to reprint or land (section 7): the landing,
Ask temper as "Start" opens it, and `chat-draft`'s Split, under the night
look, with the switch at the rail's foot. The gallery's `night-*` states draw
those three pages through the view (`gallery.md`, 2); every journey's
print draws the switch, unpressed, since no recording keeps a night look.
temper/ui compares the three requests with main after landing
(`look-requests.md`, 4).

## 10. Open questions

- **The two-step Home and night requests** (9): temper/ui compares
  `ask-temper-start`, `to-begin-two-steps` and `night-mode` against main
  after landing at 1440, 1000 and 420 px, with menus, disclosures and
  each named state (`look-requests.md`, section 4). Use
  `project-landing`'s two-step print, the `home-begin-*` states and the
  three `night-*` states; include Home with a need and a chat left out
  with either step left (`home-begin-with-needs`), and their return after
  the last step or a skip. Check Ask temper's three phone tabs with its
  composer above, the switch in the folded rail open and closed and at
  a phone's chat bar, and shadows against the day values (`look.md`
  ). Default: replayed prints and hand-made
  gallery inputs, with no recording of a night choice; print equality
  establishes no rendered equality. Findings are filed back, and repairs
  carry the request's trailer.

- **Renewing `chat-draft` after stock removal** (8, 9): temper/ui walks
  it anew after landing, following the Draft link to the chat the goal
  names, then adds the whole recording and its prints. Removing the
  configuring chat changes task numbers, so the old walk's task 2 is
  no longer the return target. Default: retain the replayable recording
  until that renewal, including its old stock inventory; offering Ask
  temper in its prints does not establish removal of template 3 and
  charter 3 from its recorded answers.

- **Ask temper's first-run prints** (9): temper/ui compares
  `project-landing`, `chat-draft` and `welcome-playground` with the
  `ask-temper-place` request after landing, at 1440, 1000 and 420 px
  (`look-requests.md`, section 4). Include the rail's "Ask
  temper" row and the phone's "Ask" tab where the page shows navigation;
  a phone chat hides that navigation (`look.md`). Default:
  fake-box recordings and their replayed prints; print equality
  establishes no rendered equality.

- **The chat terminal's comparison** (9): temper/ui compares all four
  `chat-terminal` requests' hand-made pages and named states against
  the shipped view after landing, at 1440, 1000 and 420 px, with menus
  and disclosures open (`look-requests.md`, sections 4 and 5;
  `README.md`, section 3). Default: the nine gallery states and
  reprinted `chat-draft` and `welcome-playground` pages; they establish
  no rendered equality. Real-key checks remain `../README.md`, section
  5's work, distinct from journey replay.

- **The chat heading's comparison** (9): temper/ui compares `chat-title`'s
  hand-made page 01 and the request's named states against the shipped
  view after landing, at 1440, 1000 and 420 px, including open disclosures
  and menus (`look-requests.md`, section 4). Default: unchanged
  journey prints and the view's short-title and heading tables; they
  establish no rendered equality. Screenshots and findings belong to
  that comparison.

- **The Models page's comparison** (9): temper/ui compares `model-picker`'s
  hand-made page 07 with the gallery's Models states after landing, with
  screenshots at 1440, 1280, 1000 and 420 px
  (`look-requests.md`, section 4). Default: reprinted journeys and
  gallery states; print equality establishes no rendered equality.

- **The flow's rendered look** (9): temper/ui checks a flow in a chat
  and its document after landing (`README.md`, section 3). Default:
  the gallery prints its fallback, and scene bytes are checked in host
  tests.

- **The connect-flows comparison** (9): temper/ui compares the
  hand-made request's 01–23 against main after landing, including the
  open and named states, under `look-requests.md`, 4. Default:
  gallery and journey print checks establish no rendered equality;
  screenshots and any findings belong to that comparison.

- **The fresh project's draft flow on a real box** (9, `chat-draft`):
  a person's recording alongside the fake-model journey, including finding
  the goal from its chat and the project's bar (`../README.md`, section 5).
  Default: the fake-model recording starts with drafts on; a real recording
  remains the person's, as 3.5 requires.

- **Typing:** whether every act marks its page; the design marks
  none for an `input` (3.4), since a page per keystroke would make a
  walk of typing. The user's word.
- **The gallery's states,** set from hand-written worlds: whether each
  moves to a journey once the fake box reaches it, its answers then the
  engine's own (`gallery.md`). Ready and Model's forge done
  can, `project-landing`'s walk reaching a fake box's Forgejo connected, so
  that fewer states are hand-made (`look-requests.md`, 8); Model's
  forge failed waits on a fake box whose set up can fail. The two
  fetching states wait on a still page at the fetch: `welcome-set-up`'s
  recording holds the fetch's rows, but they keep the page moving, so
  its pages 07 and 08 fall at preparing (8.1;
  `../../protocol/setup/README.md`, section 9).
- **A render-check command:** in `temper-client-journeys`, taking a
  request's name. Default: each landing's own script
  (`look-requests.md`, 4 and 8).
- **The page's own state at a mark** (3.4): where a journey meets a
  skein menu, the page's events would be recorded and replayed beside
  the host's reports.
- **A new recording's noise:** ports fixed for a fake box that `again`
  starts, where it takes the kernel's (`../../protocol/setup/box.md`, 5), and
  which must not collide when several record at once, and a fixed start
  for its clock and the browser's, so that a recording made again
  differs from the last only where the client does.
- **A tab in a setup** (8.2): `project-landing`'s setup presses "Continue with
  ChatGPT", and the tab it opens stays beside the page. The next step's find
  brings the page to the front (skein's `testing/browser.md`, section 6), so a tab
  takes no step of its own; a record for one joins 8.2's table only if one
  needs a step. Default: none.
- **Later, if wanted:** the one-tag-per-line print as skein-ui-web's,
  for another application's journeys.
- **What a real journey leaves out** (3.8): a token split across a run's
  deliveries is refused and not replaced, and a value the client reshapes
  goes stale, so a walk that meets either is walked again without it.
  Whether `add` takes them out waits for a recording that can't be walked
  without them.
- **The recorded box:** the stream's credential is a loopback session of
  a box the person stops; whether `add` also asks them to stop it.
- **Usage and spend figures** the page was given stay as they were, on a
  real journey too; whether they should.
