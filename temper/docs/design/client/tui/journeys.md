# The TUI's journeys

Provisional, 2026-11-08. A TUI journey is a walk through the TUI
(`README.md`, 1 to 3 and `pages.md` and `theme.md` and `README.md`, 4; `../../draft/tui.md`, section 5) recorded
at its host's seam on a fresh `temper box --fake-backends`, replayed
natively through the shipped host, and printed as its frames: each as
text, a line per row, and as a page drawn in the theme's colours,
linked so that the same keys walk them. They are the browser's journeys
(`../web/journeys.md`) for the TUI's host, and share their file's syntax,
their redaction and their last scan, with the same code. `journeys.md`, section 2 is
the build, 3 the recording, 4 the replay, 5 the redaction and the scan,
6 the print, 7 the tests, 8 recording again, 9 the journeys, 10 editing
the look, 11 what is open.

## 1. In one page

- **One seam, the host's** (`README.md`, 3): the run gives its
  host each report and carries out the duties it asks back; the
  recorder keeps what crosses that seam, and the replay gives it back to
  the same host, natively.
- **Recorded in a build of its own:** `temper-box-shell` built with its `record`
  feature records every TUI it runs, from its start, into the TUI's
  directory; the shipped `temper-box` holds none of it (section 2).
- **The browser's file, the TUI's records:** the same text, one record
  per line, the same shared records (the start's seed and clocks, a
  turn, a deadline, a mark, the small store's records), and records of
  the TUI's own for its keys, its transport, its credential and its
  browser (section 3). One format, sharing what fits
  (`../../draft/tui.md`, section 8's default).
- **Replayed, it must ask what it asked,** each key giving the event it
  gave; otherwise the journey is stale and recorded again (sections 4
  and 8). The first replay proves itself against the live frames.
- **Redacted and scanned by the browser's code** (`../web/journeys.md`, 3.6
  and 3.7), which gains the TUI's originals: the approval's secret, the
  credential the TUI takes and the bearer every exchange carries
  (section 5).
- **Printed by `temper-client-journeys`,** under
  `docs/design/client/tui/journeys/`: a text and a page per mark, a
  generated `theme.css` and `tui-journey.js`, and an empty `overlay.css`
  (section 6). No terminal, no box, the same bytes every time.
- **Kept current by the journeys' test,** with the browser's, which
  also fails while the TUI's `overlay.css` holds anything (section 7).
- **Recorded again by a command:** `again NAME` walks the journey's
  `walk` on a fresh fake box, the TUI in a pseudo-terminal and the
  approval given in headless Chromium, and adds the result (section 8).
- **The look is asked for on the frames:** a frame's text edited in
  place, the theme's overlay written, each request landing as the
  browser's do (section 10).

## 2. The build

- **The journey module,** `temper-client-shell-tui`'s `journey`, built
  with the crate's `record` feature and in its tests: the file's lines,
  the recorder, the replay, the redaction's and the scan's TUI part, and
  the walk's logic. With `record` the crate depends on
  `temper-client-shell-web`, natively, for the shared redaction and scan
  (section 5). It is ordinary Rust, as the browser's `journey` module is.
- **The recording build:** `temper-box-shell`'s feature `record`, which no other
  package enables, turns on the crate's and has the binary give the run
  a recording (`run::Settings.record`). A build of the workspace that
  compiles the journey module for `temper-client-journeys` gives the run
  the code but never a recording, so a `temper-box` built without its own
  `record` records nothing.
- **Nothing else changes in the TUI:** the recorder adds no node, no
  paint, no request and no file but its own (3.1).

## 3. The recording

### 3.1 The recorder

- **What it hears:** in the run, beside the host, through
  `journey::Recorder`, which wraps the seam: the start's facts once, as
  the run gives its `Start` (3.2); then each iteration that reports
  anything as a turn, its clocks, each report given to `Host::report`
  with the duties it returned, and the turn's deadline. It keeps them in
  memory as the file's lines. The host's tests call it alike.
- **Two keys,** read by the run from the terminal's keys before the host
  is given them, so the host never is: Alt+Shift+M marks the frame as
  drawn, with no key, for a state the TUI reaches by itself, such as an
  approval given in the browser; Alt+Shift+S saves the recording as it
  is then, closed by an end mark of the frame as drawn. A mark is kept
  in a turn of its own (3.2); a save keeps nothing.
- **Each key in two forms:** Alt+m is the mark key too, and Alt+s the
  save key, for a terminal that drops the Shift from a Meta key it
  doesn't bind, as emacs' vterm does: it sends Alt+Shift+M as Alt+m.
  Only a recording takes them: a run without one, as the shipped
  `temper-box`'s always is (section 2), gives Alt+m and Alt+s to the host as
  any other key.
- **A save** writes the file whole, `recording` in the TUI's directory
  (`README.md`, 2), `0o600`, a synced temporary replacing it,
  through skein-io's file layer on the run's own io. The recorder goes
  on after a save, so each save is a whole recording and a later one
  holds more. Closing saves once more, as it is, before the terminal's
  `LEAVE`; a save that fails says nothing on the screen, and the next
  save tries again.
- **One run, one recording:** a recording starts with the TUI and is
  whole at each save; the next TUI run replaces it.
- **The file keeps every original:** it is the person's, in their
  private directory, and `add` takes it (section 6). It is never
  committed as it is.

### 3.2 The file

`../web/journeys.md`, 3.2's text: one record per line, its kind first, then
its fields, each after one space; a number decimal, a word bare
(`[a-z][a-z-]*`), bytes in double quotes with `\\`, `\"` and `\xHH`
for a byte outside printable ASCII. No other lines. A TUI's recording
is told from a browser's by its first fact after `clocks`, `screen`.

| Record | Fields | Where |
|---|---|---|
| `journey` | `2` | the first line (shared) |
| `seed` | the host's seed | the start's facts, in this order (shared) |
| `clocks` | now and wall, in nanoseconds, at the host's making | (shared) |
| `screen` | rows, columns, `dark`, `light` or `none` | the start's size and the scheme the terminal says, `none` where it says none (`theme.md`) |
| `background` | red, green and blue, each a number | the terminal's answer to OSC 11, when the start has one (`README.md`, 3, `Start.background`) |
| `address` | `HOST:PORT` as bytes | the start's address |
| `local` | none | when the start has an engine's link file to read (`README.md`, 3, `Start.local`) |
| `box` | none | when the start is on a box (`README.md`, 2, *On a box*; `Start.boxed`) |
| `front` | none | when the TUI is in front of a box (`box.md`, `Start.front`) |
| `stored` | a small-store record's key and bytes | one per record loaded (shared) |
| `credential` | the credential's bytes | when the start held one |
| `noted` | `until` and nanoseconds, or `signed-out` | the start's note, when it held one (`README.md`, 2) |
| `turn` | now and wall | each turn's clocks (shared) |
| `start`, `due` | none | the `Start` and `Due` reports (shared) |
| `key` | the key (below), then its event's print in quotes, or `none` | a `Key` report |
| `resize` | rows, columns | a `Resize` report |
| `reply` | the exchange, the message's bytes | `Heard(Reply)` |
| `unanswered` | the exchange | `Heard(Lost)` |
| `streamed` | `record` and its bytes, `heartbeat`, `full` with `component` or `transport` and the size, or `signed-out` | `Heard(Streamed)` |
| `gone` | none | `Heard(StreamGone)` |
| `saved`, `unsaved` | none | `Heard(Stored)`, `Heard(NotStored)` |
| `kept` | `done` or `failed` | a `Kept` report |
| `opened` | `done` or `failed` | an `Opened` report |
| `found` | the file's bytes, or `none` | a `Local` report |
| `box-line` | a line of the box's, as `BoxLine::print` prints it, in quotes | a `Front` report (`box.md`) |
| `send` | the exchange, the message's kind, its bytes, the credential's bytes or `none` | a duty's `Request::Exchange`, in order |
| `stream` | the credential's bytes or `none` | `Request::OpenStream` |
| `unstream` | none | `Request::CloseStream` |
| `save` | a record's key and bytes | `Request::Save`, one per record (shared) |
| `erase` | a record's key | `Request::Erase`, one per key (shared) |
| `keep` | the credential's bytes, or `none` | the duty `keep` |
| `note` | `until` and nanoseconds, or `signed-out` | the duty `note` |
| `signed` | `HOST:PORT` as bytes | the duty `address` |
| `read` | none | the duty `local` |
| `browse` | the address as bytes | the duty `open` |
| `quit` | none | the duty `close` |
| `deadline` | nanoseconds, or `none` | each turn's end (shared) |
| `mark` | the live frame's print | before its key, or at the mark key, in a turn of its own (`turn`, `mark`, `deadline`) (shared) |
| `end` | the live frame's print | the last line (shared) |

- **A key:** a bare word for a key with no text, `enter`, `tab`,
  `backtab`, `backspace`, `delete`, `up`, `down`, `left`, `right`,
  `home`, `end`, `pageup`, `pagedown`, `escape`; or a word and its
  bytes: `char` and the character's UTF-8, `paste` and the pasted bytes,
  `control` and the letter, `alt` and the character
  (`skein_ui_screen_terminal::Key`).
- **A message's kind,** `jig_client::Message`'s variant in kebab case:
  `authorize`, `approval-ask`, `approval-read`, `approval-decide`,
  `sign-in-approval`, `request`, `read`, `watch`, `unwatch`,
  `stream-gone`, `sign-in-link`, `sign-in-service`, `sign-out`; the last
  two carry no bytes, so their bytes field is `""`, and a reader refuses
  any other.
- **Any other `jig_client::Event`** reaching the host is no native
  shell's, and the recorder refuses it as its defect.
- **The duties' order:** each request of `asked` in order, then `keep`,
  `note`, `signed`, `read`, `browse` and `quit`, each when the duty holds
  it.
- **The start's facts** are in the table's order, `background` after
  `screen`, `local` after `address`, `box` after `local`, `front` after
  `box` and `noted` after `credential`. A recording with none of them
  reads as a start with no background answered, no engine's link file,
  not on a box, out of front and with no note, so one made before them
  replays as it stands; the version stays `2`.
- **Unreadable:** another version, a record out of its place, an unknown
  kind or word, a field missing or over, a number out of its type's
  range; it names the line.
- **A committed recording's marks hold no print:** `add` drops it once
  the first replay has proved itself (4.3), as the browser's does.

### 3.3 A key's event

- **Every key the host is given is kept,** with the event its view makes
  of it, read through `Host::event_of` before the key is given: the TUI
  event's `Debug` print, compact, as bytes, or `none`. Unlike a
  browser's act, a key with no event is kept too, since it may move the
  focus that the next key acts on.
- **Typing marks nothing:** a key whose event is `Typed` marks no
  frame; any other key with an event marks the frame as drawn before it
  (3.4). A key with no event marks nothing.
- **The events' prints,** `temper_client_tui::Event`'s, with their fields'
  own `Debug` forms (`README.md`, 3, *The TUI's events*): `Typed {
  value: [104, 105] }`, for the link's field and the box alike;
  `SignIn`; `Send`; `Command { command: Help }`; `Act { act: Choice(Model
  { index: 0, account: Some(3) }) }`, the root's act; `Aside { object: Task(9)
  }`; `Dismiss`. The record holds them as bytes, with no record kind of
  their own (3.2, `key`), and a replay compares them as it finds them.
- **The mark and save keys,** in either form, never reach the host while
  recording: a mark is kept in its own turn; a save writes the recording
  with its end mark, keeping no key record (3.1). Without a recording,
  both forms reach the host as any other key.

### 3.4 Marks

- **A mark is a frame as the person saw it:** as drawn when a key with
  an event came, before its turn changes it; as drawn when the mark key
  was pressed; and at the recording's end. It keeps the live frame's
  print, `Frame::print` (skein's `ui/screen.md`, section 4: its text,
  its paints and its cursor), for the first replay's proof.
- **The frame is the host's:** `Host::frame` at the window's size, so a
  mark holds the whole screen (`../../draft/tui.md`, 1).

## 4. The replay

### 4.1 Through the shipped host

`journey::Replay`, native, makes the host from the recording's facts as
the run does (`Host::new` with `limits::tui()`, the seed and the
clocks), then, turn by turn, sets the clocks and gives each report:
`Start` from the facts, a key as recorded, a transport's event, a
resize, `Kept`, `Opened`, `Local` with the bytes found, `Front` with a
`box-line`'s bytes read back by `BoxLine::read`. At each mark it
yields the host's frame.

### 4.2 The check

- **Each report's duties,** written as their records, are the recorded
  lines, and each turn's deadline is the recorded one.
- **Each key's event** read before it is given (3.3) is the recorded
  one.
- **Stale** is where the replay stops: at the first duty, deadline or
  key's event that differs, a `browse` or a `quit` asked or not asked
  among them. It names the turn, the recorded line and what the host
  asked or made instead. Frames are not compared: they are what the
  print shows.

### 4.3 The first replay's proof

`add` replays the redacted recording with its live prints: at each mark
the replay's `Frame::print` is the recorded one, or the recording is
refused, naming the mark and the first line of the print that differs,
never its text. Once every mark matches, the live prints are dropped,
and the print is the replay's from then on.

## 5. Redaction and the last scan

The browser's, with the same code (`../web/journeys.md`, 3.6 to 3.8), over
the TUI's lines.

- **The shared code** gains three kinds of original, `Kind::Approval`,
  `Kind::Credential` and `Kind::Bearer`, each sought in the forms and
  digests every original is, and is reached from the TUI's lines
  through `Originals::new` and `redact_field`, beside `Originals::of`
  and `redact`, which the browser's lines keep. What the browser's
  journeys redact and refuse stays as it is.
- **The TUI's originals:**
  - *approval:* the secret of each `send` of kind `sign-in-approval`, as
    jig's wire reads that message's record; the same secret stands in
    the ask's `reply`;
  - *credential:* the start's `credential` and each `keep`'s bytes, the
    credential the TUI takes; the same bytes stand in the take's
    `reply`;
  - *bearer:* each `send`'s and `stream`'s credential, the bearer every
    exchange carries;
  - *link:* the secret of each `send` of kind `sign-in-link`, as jig's
    wire reads it, the printed link pasted or the engine's link read;
    and the secret of each `found`'s link, as
    `jig-client-protocol-http`'s `printed` reads it, so that a file read
    and never sent is redacted too, the handed link of a TUI in front
    among them; and the secret of each `box-line` record's `link` line, the
    box's printed link, read alike, which the welcome draws
    (`box.md`);
  - *given:* each `--redact TEXT`.

  The same bytes under two kinds, a credential kept and carried, are
  sought once and counted under each, as the shared code counts. The
  TUI has no *typed* original: no
  event of its view shows a secret's length, and the paste's link is
  found by its secret.
- **Where:** every original in every field of every line. A `reply`'s,
  a `send`'s and a `streamed record`'s bytes, a `stored`'s, `save`'s and
  `erase`'s key and bytes, and every credential's field are framed; a
  `key`'s bytes and event, `found`'s and `box-line`'s bytes, `address`, `signed`,
  `browse`, `mark` and `end` are text (`../web/journeys.md`, 3.6, *A form's
  size*). A TUI's
  stream records come whole from the native shell, so no pieces are
  joined.
- **The proof holds:** the host derives what it asks from what it heard,
  so the take's message holds the redacted secret it was answered, and
  every exchange the redacted credential it kept.
- **A pasted link's encoding:** the TUI also replaces and scans the
  link secret's canonical URL-base64 in the pasted address, in a
  `found`'s and in a `box-line` record's `link` URL, and its decimal byte
  elements in the `Typed` event's Debug print. Each becomes those
  encodings of the redacted secret, so the paste and its event, and the
  box's line, still replay. This is the TUI's addition; the browser's
  code and encodings stay as `../web/journeys.md`, 3.6 describes.
- **The last scan** (`../web/journeys.md`, 3.7): every field of each line,
  every frame the replay drew at each turn's end as its text, and each
  printed text and page, for the originals and the shapes. An approval's
  code as shown, read from each `browse` address's `code=` and checked by
  `ApprovalCode`'s reader, is cut from a copy of the bytes before the
  shapes are sought, since the code is drawn on purpose and the *code*
  shape would take it; an original is still sought in the whole.
- **A refusal** names the line and its record's kind, and where (a
  field, a frame's turn, a printed file), never the bytes.
- **What is left to the person** is `../web/journeys.md`, 3.8's, and: a
  link typed key by key is never found whole, so its proof fails and
  nothing is written; paste it.

## 6. The print

### 6.1 The program

`temper-client-journeys` (`../web/journeys.md`, 5.1) takes, reprints and
records again the TUI's journeys beside the browser's:

- `add [--redact TEXT]... NAME FILE` reads FILE's first fact after
  `clocks`: `screen` makes it the TUI's, taken under
  `docs/design/client/tui/journeys/NAME/`. A NAME is one journey's
  across both directories: `add` refuses a TUI recording under a name
  the browser's directory holds, and the reverse. It redacts (section
  5), proves (4.3), scans and prints in memory, writing nothing until
  the scan has passed, and prints its counts as the browser's does.
- The reprint, `cargo run -p temper-client-journeys`, prints both
  directories' journeys.
- `again NAME` records the TUI journey NAME when its directory holds a
  `walk` (section 8), the browser's otherwise.

### 6.2 What it writes

Under `docs/design/client/tui/journeys/`:

- **A journey's directory,** `NAME/`: its `recording`; its `../../README.md`,
  what it walks and what each mark shows; its `walk` (8.1); and for each
  mark, `01.txt` and `01.html` onwards, in the marks' order, replaced
  whole at each print.
- **Beside the journeys:** `theme.css` and `tui-journey.js`, the
  program's, written at each print; and `overlay.css`, the person's,
  made empty when missing and never written otherwise.
- **Generated:** the texts, the pages, `theme.css` and `tui-journey.js`.
  A recording, a README, a walk and the overlay are sources.

### 6.3 A frame's text

`NN.txt` is the mark's frame as `Frame::text` gives it: a line per row,
UTF-8, a row's trailing spaces removed, for a diff. It holds no paint and
no cursor.

### 6.4 A page

- **The document:** `<!doctype html>`, `html` with `lang="en"`; in its
  head the charset, the viewport, the title `NAME NN · temper`, the
  stylesheets `../theme.css` then `../overlay.css`, its neighbours
  (`<link rel="prev">`, `<link rel="next">`) and `../tui-journey.js`; in
  its body one `pre` of the classes `tui` and `tui-dark` or `tui-light`,
  the recording's scheme, a row per line.
- **A row's cells** are runs of one paint: the base's, the terminal's
  defaults (`theme.md`), as plain text, any other in a `span`
  whose classes name its foreground and background by the theme's
  palette (`f-accent`, `f-dim`; `temper-client-tui`'s `theme::palette`),
  then its attributes (`bold`, `dim`, `italic`, `underline`, `reverse`).
  A colour that is the terminal's default gets no class: the `pre`'s own
  colours stand in for it. A colour that is no palette's is the span's
  `style`, `color:#RRGGBB` or `background:#RRGGBB`. The cursor, when the
  frame has one, is a `span` of the class `cursor` around its cell.
  Text escapes `<`, `>`, `&` and `"`.
- **Linked by keys:** a mark taken before a key carries the key's name
  and the next page, `data-journey-key="enter"` and
  `data-journey-next="02.html"` on its `pre`; `tui-journey.js` follows
  it on that key (a character by itself, `control c` by Control and
  `c`). `n` and `p` follow the neighbours, unless the mark's key is one
  of them. A mark of the mark key leads on by its neighbours alone.
- **`theme.css`,** from `theme::palette` for both schemes: each
  scheme's palette as custom properties under `.tui-dark` and
  `.tui-light`; on the `pre`, the scheme's stand-ins for the terminal's
  default text and background, `ink` and `bg`; a rule per palette colour
  for `f-` and `b-`, a rule per attribute, a monospace font and the
  cursor's look. A frame keeps the terminal's default as a colour of its
  own (skein's `ui/screen.md`, section 3), and only the page draws it,
  in the stand-ins.

### 6.5 Opening them

As the browser's (`../web/journeys.md`, 5.4): a journey's `01.html` from the
checkout, as a file or through a server from the repository's root.

## 7. Its tests

As the browser's journeys' (`../web/journeys.md`, section 6), within their
cost, in `temper-client-shell-tui`'s journey module (the host's world:
reports given by the test) and `tools/journeys`' own tests:

- **The seam:** a walk recorded by `journey::Recorder` and replayed; the
  file's lines read back to the records written, every record and key
  and each escape among them, and another version, a record out of its
  place and an unknown kind unreadable; stale at a changed ask, a
  changed deadline, a key whose event changed and a `browse` not asked;
  a key with no event kept and replayed; the proof refusing a mark whose
  print differs; two saves of one walk, the first a whole recording that
  the second extends.
- **The redaction and the scan, natively:** a failing test first for
  each TUI original, in each place it can appear (the ask's `reply`,
  the take's `send` and `reply`, `credential`, `keep`, each `send`'s and
  `stream`'s credential, a pasted link's `key` and its `send`); the
  shared code's new kinds in the browser's tables; the approval's code
  passing the scan in a `browse` address and a frame, and a `code=` of
  any other value still refused; a redacted approval walk replaying as
  it stands.
- **The print:** every committed TUI journey reprinted into a scratch
  directory equals its directory and the files beside it, byte for byte;
  one that differs or is stale fails, naming the command that reprints;
  the tests fail while the TUI's `overlay.css` holds any byte, and for a
  journey whose `../../README.md` is missing or empty; a page's runs, palette
  classes, attributes and key links in the print's unit tests.
- **The walk, natively:** a `walk` read back, each record among them,
  and another version, a record out of its place and a second `end`
  unreadable; a recording checked against its walk; a TUI journey's
  `setup` read with its `after` joined from the browser's directory, and
  one holding a `mark` refused, naming its line.
- **`again`'s half that drives a terminal and a browser** has no story of
  its own: the first journey, recorded by it, proves it (8.2).
- **Their cost:** the reprint of both directories together within the
  browser's 1 s of the default suite; no fuzzy test; no default test
  starts a terminal, a browser or a box.

## 8. Recorded again

### 8.1 The walk

A TUI journey's `walk`, beside its recording, is what the person did,
step by step: `again` follows it, and needs no earlier recording, so the
first one is made by `again` too. Text, one record per line, as the
recording's (3.2).

| Record | Fields | What `again` does |
|---|---|---|
| `walk` | `1`, the format's version | the first line |
| `see` | text | waits until the TUI's frame shows the text: a save, replayed natively to its end (section 4), gives the frame |
| `key` | a key, as 3.2 writes it | gives it to the TUI through its terminal, as a person types it |
| `mark` | none | the mark key in its Shift form, Alt+Shift+M (3.1) |
| `approve` | none | the browser opens the address of the latest save's last `browse` and presses its button "Approve", found by role and name as skein's fake person finds it |
| `end` | none | last, once: the save key in its Shift form, Alt+Shift+S (3.1), whose save is the new recording |

- **Its bounds:** a `see` not shown within 30 seconds, or an `approve`
  whose page shows no "Approve" within 30 seconds, stops `again`,
  naming the step.
- **The check, then `add`:** the new recording's `key` records are the
  walk's `key` steps, in order, and it holds a `browse` before each
  `approve`; or `again` refuses, naming the first step that differs.
  Then `add NAME FILE`, its first replay proving it, its README and walk
  left as they were.

### 8.2 Again

`cargo run -p temper-client-journeys -- again NAME`, in `tools/journeys`,
for a TUI journey, as the browser's `again` is (`../web/journeys.md`, 8.1),
ordinary Rust over skein's harness:

- **Built from the checkout:** the client `temper` with recording enabled (2), and
  `fake-claude`, and the browser's bundle (`scripts/bundle.sh
  --loopback`), each copied into a fresh scratch directory under `/tmp`.
- **A fresh fake box** (`../web/journeys.md`, 3.5), `temper box
  --fake-backends` from that copy serving that bundle, every directory
  in the scratch directory, and headless Chromium from
  `SKEIN_TEST_CHROMIUM`, as the browser tier drives it, its printed
  sign-in link opened, so that the browser is signed in.
- **The TUI in a pseudo-terminal** of 80 columns by 24 rows (skein's
  harness starts a binary on one), `temper http://HOST:PORT` at the
  box's address, with `XDG_STATE_HOME` and `HOME` in the scratch
  directory, `XDG_RUNTIME_DIR` a directory of its own there,
  `tui-runtime`, apart from the box's, so that it finds no engine's link
  file and the journey walks the approval (`README.md`, 2), and,
  reading none at start, is not on a box: the rule above a chat's box
  names the engine's place by its `HOST:PORT` (`pages.md`), no
  `COLORFGBG`, so that it asks the terminal its background, which the
  pseudo-terminal leaves unanswered, and draws dark with no band
  (`theme.md`, *The band*), `TERM` `xterm-256color`, no `SSH_` variables, `DISPLAY` as
  `again`'s or `:0`, and `PATH` led by a scratch directory whose
  `xdg-open` is a link to `true`, so that the TUI's opener runs, exits
  0, and leaves the tab to `again`. Its output is read and dropped.
- **A setup,** where the journey's directory holds one: its `setup`
  file, in the browser's format and with its rules (`../web/journeys.md`,
  8.2), its `after` naming a browser journey, read from the browser's
  journeys' directory, as `project-landing`'s steps to a project with a
  model. `again` walks it in the signed-in browser, its `reload` loaded
  last where it has one, before the TUI starts, so that the TUI opens on
  what the setup made; a `mark` in it is refused when it is read, since
  the TUI's `walk` marks its own frames. Its steps and their bounds are
  the browser's; nothing of them reaches the recording.
- **A save** is told by the `recording` file replaced (3.1).
- **What it reads and starts:** as the browser's; the box, Chromium and
  the TUI all stopped at its end, the scratch directory removed unless
  it failed, which it then names with what failed.
- **A run that stops on a moving screen** is tried again, at most three
  times, each on a fresh fake box.

### 8.3 A box with the TUI in front

What `again` does to record a journey of `temper-box` with the TUI in
front of it (`box.md`; `../../protocol/setup/box.md`, 6),
where 8.2 says otherwise:

- **The box in the pseudo-terminal:** `temper-box --fake-backends` from
  the scratch copy is itself started in the pseudo-terminal of 80 by 24,
  in a session of its own, with the TUI's environment of 8.2 but
  `XDG_RUNTIME_DIR`, the box's, so that the box runs its front there and
  the front opens that terminal and takes its foreground. The front is
  the same `temper-box`, built with `record`, and records into the TUI's
  directory as bare `temper` does (3.1). No `temper` is started apart,
  and no approval is walked: the front signs in by the link the box
  hands it.
- **The printed link,** which the box writes to no stream `again` reads,
  is read from the latest save's last `box-line` record holding a `link`
  line, and opened in the browser, which it signs in; the walk's record
  `open`, with no field, says when.
- **The browser's steps, mid-walk:** the journey's `setup` (8.2), from
  the welcome to the playground, is walked in the signed-in browser at
  the walk's record `setup`, with no field, while the TUI follows it,
  rather than before the TUI starts; a `see` after it waits for the
  frame the steps lead to.
- **The welcome's link, wrapped:** at 80 columns the welcome draws the
  printed link over rows. The redaction seeks a `box-line` link's secret in
  each `mark`'s and the `end`'s print across the rows it wraps over too,
  the print's rows read as one where the link continues, each row's
  indentation and line feed left out, and the last scan reads the
  frames alike, so that the link is redacted in a mark's print across
  the wrap.
- **Why not yet:** 8.2's `again` drives `temper` in the pseudo-terminal
  and the box apart, on pipes, reading the box's printed link from its
  stdout; a box whose TUI is in front needs the terminal itself and
  prints no link on a stream. Until 8.3 is built, a recording made by
  hand whose marks draw the welcome's link wrapped fails its first
  replay's proof, as a link typed key by key does (section 5), and
  nothing is written.

## 9. The journeys

| Name | The walk | Its pages |
|---|---|---|
| `sign-in-approved` | on a fresh `temper box --fake-backends`: the browser signed in by the box's printed link; `temper` started at the box's address; the approval's frame; the approval given in the browser; the launchpad, with no project yet | `01`, the approval's frame, "A tab opened in your browser", its code and its address, marked by the mark key; `02`, the launchpad of a box with no project (`pages.md`), at the recording's end; its `../../README.md` says so |
| `launchpad-chat` | on a fresh `temper box --fake-backends`: its `setup` (8.2), `after project-landing`, to Hello with the ChatGPT plan and "work/hello is in Hello"; then as `sign-in-approved` to the launchpad; `/` typed and the commands; Esc; a question typed and sent, which opens an Ask temper chat; the fake model's answer; the end | the launchpad in Hello, the commands under the box, the chat while it is written where a save catches it, and the chat answered at the recording's end (`pages.md`); its `../../README.md` names each frame |
| `launchpad-drawn` | on a fresh `temper box --fake-backends`: its `setup` as `launchpad-chat`'s, to Hello; then as `sign-in-approved` to the launchpad; `/change` and a change asked in words, which starts a chat; its fake model starts a goal and says that a question waits, and the goal asks the chat's person a question with two choices, the first recommended, as a goal a chat calls asks (`../../domain/agent/agents.md`, 2); `/new` back to the launchpad, the chat under *Where you left off*, its question under Needs you and the quiet line's count; `n` and the walk's prompt; its first choice taken; the walk's end; the end | `03`, the launchpad as drawn, with the chat, its need and Start (`pages.md`, *The launchpad*); `05`, the walk's prompt under the box (*Walking what needs you*); `07`, the launchpad after the walk, its note, at the recording's end; `01`, `02`, `04` and `06`, the frames before the keys that act (3.3); its `../../README.md` names each frame |
| `design-session` | on a fresh `temper box --fake-backends`: its `setup` as `launchpad-chat`'s, to Hello with its home repository; then as `sign-in-approved` to the launchpad; `/draft` and the work asked in words, which starts a design session; the fake model's reply writing the draft and ending with numbered choices, the pane at half; Ctrl-P to all, to folded and to half again; "1" typed and sent, the next reply changing a section, marked new, and saying nothing blocks the work; `/goal`, its prompt of ways, the first taken; the chat again, its goal linked: `/goal` no longer offered, and the quiet line without the goal's part, which at the recording's 80 columns goes first (`pages.md`, *A design session*, *narrow*; the goal's part is the 110-column golden's); the end | `03`, the session at half with the drafter's choices under the box; `05`, the pane at all; `07`, folded; `10`, Open questions marked new; `12`, `/goal`'s ways prompt; `14` and `15`, the returned chat with its goal linked, marked and at the recording's end (`pages.md`, *A design session*); `01`, Hello's launchpad; `02`, `04`, `06`, `08`, `09`, `11` and `13`, before the keys that act (3.3); its `../../README.md` names each frame |
| `box-welcome` | on a fresh `temper box --fake-backends` with the TUI in front (8.3): the welcome as the box starts; the printed link opened in the browser; the welcome following the browser's steps to the playground, walked mid-walk; the playground's launchpad; the end | `01`, the welcome before the link is opened; `02`, the steps followed, a step under way; `03`, the playground's launchpad at the recording's end (`box.md`); its `../../README.md` names each frame |

## 10. Editing the look

As the browser's (`../web/journeys.md`, section 7), on the TUI's print:

- **On the print:** the person serves the checkout, walks a TUI journey,
  and writes rules in `tui/journeys/overlay.css` (a palette's custom
  property, a class's paint), then reloads; a frame's words, order or
  layout is edited in place, in its `NN.txt` or `NN.html`.
- **Requests stack** on a branch of the person's own, `tui-requests`,
  apart from the browser's `look-requests`, so that the two are asked
  for in parallel: one commit each, with a `Look-request: NAME` trailer
  whose NAME starts `tui-`, kept on `main` by the same
  `scripts/look-refresh.sh`, run with `tui-requests` checked out, and
  never landed (`../web/journeys.md`, section 7;
  `../web/look-requests.md`, 2).
- **Never landed:** the journeys' test refuses a request's frames, which
  differ from the reprint, and its overlay, so `main`'s overlay stays
  empty.
- **Done** is a landing carrying the request's trailer in commits of its
  own that moves the overlay's rules into `temper-client-tui`'s theme
  (its palette, its tokens' paints, its borders and glyphs) and the
  frames' changes into its view and words, and reprints.

## 11. Open questions

- **A terminal's Meta keys:** the user records by hand on their laptop
  after landing to check marking with Alt+m and saving with Alt+s in
  emacs' vterm. The automated walk keeps the Shift forms (8.1).
- **The frames' refinement:** the user refines `sign-in-approved` and
  `launchpad-chat` on their frames (sections 9 and 10), with later
  surfaces refined before they are designed (`../../draft/tui.md`,
  section 3). The look requests `tui-launchpad` and
  `tui-chat-title-below` have their render check and
  walk after landing, owned by temper/tui's after-landing check; differences
  return as repairs carrying its trailer (`../web/look-requests.md`,
  sections 4 and 5). A replay and byte-equal print establish no real
  terminal's look. A growing answer, calls and a question's choices
  remain for a journey that catches them; the joint world and goldens
  cover them without establishing the rendered look
  (`../testing.md` and `../README.md`, section 5).
- **Selectors by token:** a page's cells carry their paint, not the
  view's token, so an overlay retunes a palette colour or a paint
  wherever it falls, never one token alone. Default: palette classes; a
  token's class on its cells waits for a request that needs it.
- **Both schemes on one page:** a page draws its recording's scheme.
  Default: one; a key that swaps `tui-dark` for `tui-light` is a
  request's.
- **A real box's TUI journey:** none in the first pass; it would be
  recorded by hand and redacted as the browser's real journeys are
  (`../web/journeys.md`, 3.5).
- **The place "box" in a journey:** `again` keeps the TUI apart from the
  box's link file so that it walks the approval (8.2), so no journey
  draws "box" in the rule above a chat's box (`pages.md`, *The
  rule above the box*), and `launchpad-chat/04` has `127.0.0.1:PORT`
  where the look request `tui-chat-title-below` drew "box:40691"; the
  view's rows hold "box". Default: so, until a journey signs in by the
  engine's link file, or `box-welcome` is recorded in front of a box
  (8.3).
- **A resize in a walk:** recorded and replayed as a report; `walk` has
  no step for it until a journey needs one.
- **A new recording's noise:** a fake box's port and the clocks change
  with each `again`, as the browser's (`../web/journeys.md`, section 10).
- **Redaction in `again`:** a recording keeps the recorder's OS account
  name, the label the fake box gives Me, and a program's reply may name
  the scratch directory `/tmp/temper-journey-PID-N` (8.2); neither is
  drawn, and `add --redact` refuses an admitted recording. The
  `launchpad-chat`, `launchpad-drawn`, `sign-in-approved` and
  `design-session` recordings are committed so (the user, 2026-11-11).
  Default: so, until a later change gives the fake box a fixed account
  label under `again` and redacts the scratch prefix in what it records.
