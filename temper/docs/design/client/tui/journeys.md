# The TUI's journeys

A TUI journey is its `walk` of version 2: what a person does and where
its frames are printed (`../journey-walks.md`, 2). The frames are text,
a line per row, and themed HTML pages linked by keys. The walk runs in
a world for the print, on a box through the drive session, or by hand
with the TUI's recorder. These are different ways to meet the shipped
host, not different sources for main's journeys.

## 1. In one page

- **The source is the walk,** with its README and, where it has its
  own, `scripts.json` (6.2). A walk names both clients on its lines:
  `tui key`, `tui see`, `tui approve` and `tui mark`, with browser lines
  where the journey needs them (`../journey-walks.md`, 2.2).
- **In a world:** `temper-client-walk` prints it in one process, the
  TUI's shipped host on the simulated service, with no terminal, box,
  recording or network (section 6; `../journey-walks.md`, 3.3, and `../journey-walks-world.md`, 1).
- **On a box:** the drive session takes the same lines, the TUI in a
  pseudo-terminal, after landing and outside every plan (section 8;
  `../drive.md`). It writes no page of main's.
- **By hand:** a person starts the TUI with `TEMPER_INTERNAL_RECORD`
  set, records into their private directory, and prints the saved file
  outside the repository with `temper-client-journeys print FILE OUT`
  (sections 2 to 5 and 6.1). No recording is committed.
- **One seam, the host's:** the recorder keeps reports and duties; the
  native replay gives the reports to that same host and checks what it
  asks. Redaction, the proof against saved live frames and the last
  scan guard a person's print (sections 3 to 5).
- **Main's pages have one writer,** after each landing
  (`../journey-writer.md`). A goal changes sources, compares by a print
  into scratch, and commits no page (section 10). The delivery check
  refuses written files beside other changes and a nonempty overlay.
- **The look is asked for on the frames:** a frame's text edited in
  place, the theme's overlay written, each request landing as the
  browser's do (section 10).

Section 2 is the build, 3 the person's recording, 4 its replay, 5 its
redaction and scan, 6 the print, 7 the tests, 8 the box, 9 the journeys,
10 the look and 11 what is open.

## 2. The build

- **The journey module,** `temper-client-shell-tui`'s `journey`, in
  every build: the file's lines, the recorder, the replay, the
  redaction's and the scan's TUI part. It has no walk logic of its own. The shared
  redaction and scan are `temper-client-redaction`'s, which the
  browser's journeys use too (section 5), so the client links no web
  shell (`../../protocol/binaries.md`, 4). It is ordinary Rust, as the
  browser's `journey` module is.
- **Switched on by `TEMPER_INTERNAL_RECORD`** (`../../cli/README.md`, 8):
  internal, read from the binary's captured environment where the TUI
  starts, by bare `temper` and `temper box` alike; set to
  any nonempty value, the start opens the TUI's directory for the
  recording and gives the run a recording (`run::Settings.record`);
  absent or empty, it gives none, and the run records nothing. The
  drive session sets it in the TUI's environment (8.2).
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
  Only a recording takes them: a run without one gives Alt+m and
  Alt+s to the host as any other key.
- **A save** writes the run's file whole, `recording-TIME` in the TUI's
  directory (`directory.md`, 1; *Each run its own file*, below), `0o600`, a
  synced temporary replacing it, through skein-io's file layer on the
  run's own io. The recorder goes on after a save, so each save is a
  whole recording and a later one holds more. Closing saves once more,
  as it is, before the terminal's `LEAVE`; a save that fails says
  nothing on the screen, and the next save retries.
- **Each run its own file:** a
  recording starts with the TUI and is whole at each save, and each run
  saves to a file of its own, `recording-TIME`, TIME the run's start,
  the wall clock at its first iteration, in UTC, in the basic form
  `YYYYMMDDTHHMMSSZ`: `recording-20261115T090032Z`. A save within the
  run replaces that run's file alone, and no run replaces another's.
  Two runs started within one second share a name, the later's saves
  replacing the earlier's: the name's grain is the second.
- **The last ten kept:** a run that records, once it has its file's
  name and before its first save, removes the oldest recordings beyond
  the newest nine, so that with its own the directory holds ten: of the
  directory's entries, the files named `recording-` and a TIME of that
  form exactly, oldest first by name, which is by time. It removes
  nothing else: a `recording` an earlier build saved, a save's
  temporary and the directory's other files stay. A removal that fails
  says nothing on the screen; the next run that records retries.
- **Said at the end:**
  once the terminal is left, a run that recorded writes one line to
  standard error, before any other line it writes there: "temper:
  recorded to PATH" where its last save, the closing one, stored the
  file, else "temper: couldn't save the recording to PATH", PATH the
  file's whole path. Standard error, since standard output is the
  terminal's and closes with it.
- **The file keeps every original:** it is the person's, in their
  private directory. `temper-client-journeys print` reads it without
  changing it (6.1); the saved file still holds every secret afterwards.
  Keep it out of the repository and remove it once printed.

### 3.2 The file

`../web/journeys-recordings.md`, 1.2's text: one record per line, its kind first, then
its fields, each after one space; a number decimal, a word bare
(`[a-z][a-z-]*`), bytes in double quotes with `\\`, `\"` and `\xHH`
for a byte outside printable ASCII. No other lines. A TUI's recording
is told from a browser's by its first fact after `clocks`, `screen`.

| Record | Fields | Where |
|---|---|---|
| `journey` | `4` | the first line; the replay reads versions 2 to 4 |
| `seed` | the host's seed | the start's facts, in this order (shared) |
| `clocks` | now and wall, in nanoseconds, at the host's making | (shared) |
| `screen` | rows, columns, `dark`, `light` or `none` | the start's size and the scheme the terminal says, `none` where it says none (`theme.md`) |
| `background` | red, green and blue, each a number | the terminal's answer to OSC 11, when the start has one (`host.md`, 1, `Start.background`) |
| `plain` | none | when the start's terminal draws no mark (`host.md`, 1, `Start.plain`; `pages/README.md`, 3) |
| `version` | the build as bytes | the start's `version`, when it is not empty (`host.md`, 1) |
| `address` | the origin as `Origin::print` prints it, as bytes; a bare `HOST:PORT`, as a recording made before it holds, reads as `http://HOST:PORT` | the start's origin (`README.md`, 1) |
| `local` | none | when the start has an engine's link file to read (`host.md`, 1, `Start.local`) |
| `box` | none | when the start is on a box (`directory.md`, 1, *On a box*; `Start.boxed`) |
| `front` | none | when the TUI is in front of a box (`box.md`, `Start.front`) |
| `set-up` | none | when the TUI in front is on a box set up before (`box.md`, `Start.set_up`) |
| `box-log` | the path as bytes | in front, the box's log's path, when the start has one (`box.md`, `Start.box_log`) |
| `stored` | a small-store record's key and bytes | one per record loaded (shared) |
| `credential` | the credential's bytes | when the start held one |
| `noted` | `until` and nanoseconds, or `signed-out` | the start's note, when it held one (`directory.md`, 1) |
| `turn` | now and wall | each turn's clocks (shared) |
| `start`, `due` | none | the `Start` and `Due` reports (shared) |
| `key` | the key (below), then its event's print in quotes, or `none` | a `Key` report |
| `resize` | rows, columns | a `Resize` report |
| `reply` | the exchange, the message's bytes | `Heard(Reply)` |
| `part` | the exchange, the part's bytes | `Heard(Part)` |
| `done` | the exchange | `Heard(Done)` |
| `unanswered` | the exchange | `Heard(Lost)` |
| `streamed` | `record` and its bytes, `heartbeat`, `full` with `component` or `transport` and the size, or `signed-out` | `Heard(Streamed)` |
| `gone` | none | `Heard(StreamGone)` |
| `saved`, `unsaved` | none | `Heard(Stored)`, `Heard(NotStored)` |
| `kept` | `done` or `failed` | a `Kept` report |
| `opened` | `done` or `failed` | an `Opened` report |
| `found` | the file's bytes, or `none` | a `Local` report |
| `handshake` | `untrusted`, `expired`, `not-yet-valid`, `other-name`, `failed` or `none` | a `Handshake` report (`host.md`, 1, *The handshake*) |
| `listening` | the port, or `none` | a `Listening` report (`pages/sign-in.md`, 4) |
| `returned` | `get` or `other`, the target as bytes, the `Host` as bytes or `none` | a `Returned` report |
| `box-line` | a line of the box's, as `BoxLine::print` prints it, in quotes | a `Front` report (`box.md`) |
| `box-stage` | the stage, as `BoxStage::print` prints it, in quotes | a `Stage` report (`box.md`) |
| `lent` | the number, then `exited` and its code, `killed` and its signal, or `not-started` | a `Lent` report |
| `send` | the exchange, the message's kind, its bytes, the credential's bytes or `none` | a duty's `Request::Exchange`, in order |
| `stream` | the credential's bytes or `none` | `Request::OpenStream` |
| `replacing` | the credential's bytes or `none`, then the replaced stream's bytes | `Request::OpenStream` with a replacement |
| `unstream` | none | `Request::CloseStream` |
| `save` | a record's key and bytes | `Request::Save`, one per record (shared) |
| `erase` | a record's key | `Request::Erase`, one per key (shared) |
| `keep` | the credential's bytes, or `none` | the duty `keep` |
| `note` | `until` and nanoseconds, or `signed-out` | the duty `note` |
| `signed` | the origin as `address` has it | the duty `address` |
| `read` | none | the duty `local` |
| `browse` | the address as bytes | the duty `open` |
| `listen`, `unlisten` | none | the duties `listen` and `unlisten` |
| `answer` | `redirect` and the address as bytes, or `not-found` | the duty `answer` |
| `quit` | none | the duty `close` |
| `stop-box` | none | the duty to stop the box (`box.md`, *Quitting*) |
| `stop-box-again` | none | the duty of the box's second stop (`box.md`, *Quitting*) |
| `lend` | the number and the name as bytes | the duty to lend the terminal (`box.md`, *Lending its terminal*) |
| `unlend` | none | the duty to cancel the loan |
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
  `sign-in-approval`, `request`, `read`, `watch`, `batch`, `unwatch`,
  `stream-gone`, `sign-in-link`, `sign-in-service`, `sign-out`; the last
  two carry no bytes, so their bytes field is `""`, and a reader refuses
  any other.
- **Any other `jig_client::Event`** reaching the host is no native
  shell's, and the recorder refuses it as its defect.
- **The duties' order:** each request of `asked` in order, then `keep`,
  `note`, `signed`, `read`, `browse`, `listen`, `unlisten`, `answer`,
  the box's stop, `lend`, `unlend` and `quit`, each when the duty holds it.
- **The start's facts** are in the table's order. Optional facts absent
  mean no background answered, a terminal that draws the mark, an empty
  build, no local link, not boxed or in front, not set up and with no
  box log, no stored records, credential or note. The reader also
  accepts `front` before `box`. The writer emits `journey 4`; accepting
  a file's version does not promise its build's wire can replay (6.1).
- **Unreadable:** another version, a record out of its place, an unknown
  kind or word, a field missing or over, a number out of its type's
  range; it names the line.
- **The live prints** are removed from the in-memory copy once the
  proof holds (4.3); the person's saved file is unchanged.

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
  own `Debug` forms (`host.md`, 1, *The TUI's events*): `Typed {
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
- **A mismatch** stops the replay: at the first duty, deadline or
  key's event that differs, a `browse` or a `quit` asked or not asked
  among them. It names the turn, the recorded line and what the host
  asked or made instead. Frames are not compared: they are what the
  print shows.

### 4.3 The first replay's proof

`temper-client-journeys print` proves the redacted recording against
its saved live prints: at each mark and at the end, the replay's
`Frame::print` must equal the saved print, or the file is refused,
naming the mark and the first line that differs, never its text. Once
every mark matches, the live prints are dropped from the in-memory
copy and the replay supplies the frames to print. The saved file is
not rewritten. The tool scans the redacted fields and replayed frames
before the proof, and the printed files before writing (section 5).
It changes no build, port, approval code or clock for the print.

## 5. Redaction and the last scan

The browser's, with the same code (`../web/journey-redaction.md` and `../web/journeys-recordings.md`, 1.7 to 1.8), over
the TUI's lines.

- **The shared code** gains three kinds of original, `Kind::Approval`,
  `Kind::Credential` and `Kind::Bearer`, each sought in the forms and
  digests every original is, and is reached from the TUI's lines
  through `Originals::new` and `redact_field`, beside `journey::originals`
  and `redact`, which the browser's lines keep. What the browser's
  journeys redact and refuse stays as it is.
- **The TUI's originals:**
  - *approval:* the secret of each `send` of kind `sign-in-approval`, as
    jig's wire reads that message's record; the same secret stands in
    the ask's `reply`; and that record's grant, where it has one, the
    one a `returned` record's target carries (`pages/sign-in.md`, 4);
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
  a `send`'s, a `part`'s and a `streamed record`'s bytes, a `stored`'s, `save`'s and
  `erase`'s key and bytes, and every credential's field, including `replacing`'s credential and
  stream bytes, are framed; a `key`'s paste and event, `found`'s,
  `box-line`'s, `box-stage`'s, `box-log`'s and `lend`'s bytes, `version`,
  `address`, `signed`,
  `browse`, `returned`'s target and `Host`, `answer`'s address, `mark` and `end` are text (`../web/journey-redaction.md`, *A form's
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
  code and encodings stay as `../web/journey-redaction.md` describes.
- **A grant's encoding:** likewise, a grant's base64url with no padding
  in a `returned` record's target, whether or not a take sent it, is
  replaced by the redacted grant's and scanned, so the request still
  reads and its take still replays.
- **The last scan** (`../web/journeys-recordings.md`, 1.7): every field of each line,
  every frame the replay drew at each turn's end as its text, and each
  printed text and page, for the originals and the shapes. An approval's
  code as shown, read from each `browse` address's `code=` and checked by
  `ApprovalCode`'s reader, is cut from a copy of the bytes before the
  shapes are sought, since the code is drawn on purpose and the *code*
  shape would take it; an original is still sought in the whole.
- **A refusal** names the line and its record's kind, and where (a
  field, a frame's turn, a printed file), never the bytes.
- **What is left to the person** is `../web/journeys-recordings.md`, 1.8's, and: a
  link typed key by key is never found whole, so its proof fails and
  nothing is written; paste it.
## 6. The print

### 6.1 The programs

**For journeys,** `temper-client-walk` (`../journey-walks.md`, 5):

- `temper-client-walk print [--out DIR] [NAME]...` prints the named
  journeys, or all, into DIR, a scratch directory it names when none is
  given. DIR is laid out from the repository root down, with the assets
  the pages need; it cannot be inside either journeys root.
- `temper-client-walk pages` prints in place for the writer alone
  (`../journey-writer.md`, 4). A completed walk replaces its numbered
  pages whole, removing any past its marks' count and any recording
  left in its directory. A stopped walk leaves that journey's files as
  they were.
- The TUI's host runs on the playground's simulated service, through
  the native transport's HTTP, with its asks, stream and deadlines
  carried on the world's time (`../journey-walks.md`, 3.3). No terminal
  or recording is involved. Its start is 80 columns by 24 rows, not
  local, boxed or in front; no scheme or background, drawing dark with
  no band; build `0.0.0 (000000000000)` at `http://127.0.0.1:10001`.
  The world's facts and schedule make the bytes reproducible.

**For a person's TUI recording,**
`temper-client-journeys print [--redact TEXT]... FILE OUT`, through
`temper/bin/temper-client-journeys`, crate
`temper-client-journeys-tool` (`../../testing/tools.md`, 3–4):

- `screen`, the first fact after `clocks`, identifies a TUI file. The
  tool checks its layout, redacts in memory, scans its fields and
  replayed frames, proves it, prints and scans the pages, writing
  nothing until all have passed (sections 4 and 5).
- Build the tool at the deployment's commit: the TUI layout check
  compares a present `version` fact with this build's temper version
  and refuses another by name, before proof or output. A file without
  that fact still has to pass the replay and proof.
- OUT is laid out as the repository, the TUI's print under
  `OUT/temper/docs/design/client/tui/journeys/NAME/`, with its assets.
  NAME is FILE's first filename component before a dot when that is a
  valid name, `recorded` otherwise. Use an OUT outside the repository;
  the tool refuses a path inside either writer-owned journeys root,
  including one reached through a symbolic link. It writes pages and
  assets, not a recording or a journey source. FILE is unchanged.

The TUI journeys live at
`temper/docs/design/client/tui/journeys/`. Only the writer writes their
pages; the recording tool's print is a person's private output.

### 6.2 A journey's directory

Under `temper/docs/design/client/tui/journeys/`:

- **Sources in `NAME/`:** `walk`, version 2, its own `README.md` saying
  what the walk and each page show, and `scripts.json` where the walk
  has its own model scripts (`../journey-walks.md`, 2.1 to 2.3).
- **Written by the writer alone:** `01.txt` and `01.html` onwards,
  numbered in the walk's marks' order. No recording is a source or
  proof for these pages.
- **Beside the journeys:** generated `theme.css` and `tui-journey.js`,
  also the writer's; `overlay.css`, the person's, empty on main, made
  empty when missing and never overwritten by the printer.

A walk prints exactly its marks; typing marks nothing, and the end
adds no page. A mark's page is linked by its key as
`../journey-walks.md`, 2.4 has it. A person's recording instead prints
its recorded marks and end frame, after the proof.

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
  the print's scheme, a row per line.
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
- **Linked by keys:** a walk's mark linked by 2.4 of
  `../journey-walks.md`, or a recording's mark taken before a key,
  carries the key's name
  and the next page, `data-journey-key="enter"` and
  `data-journey-next="02.html"` on its `pre`; `tui-journey.js` follows
  it on that key (a character by itself, `control c` by Control and
  `c`). `n` and `p` follow the neighbours, unless the mark's key is one
  of them. A mark without a key leads on by its neighbours alone.
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

Focused and native, using data the tests make, not main's recordings or
pages (`../journey-writer.md`, 7). The suites are:

- **`temper-client-shell-tui`, library binary
  `temper_client_shell_tui`:** `journey::line::tests` holds strict
  records, escapes and keys, batch parts and replacement streams, and
  terminal loans; `journey::record::tests` the ordered optional start
  facts. `journey::tests` records and replays the seam, refuses changed
  duties, deadlines and key events, keeps keys with no event, proves
  frames and checks that later saves extend a whole file.
  `journey::redact::tests` and `journey::scan::tests` hold the originals,
  the approval code exception, grants, pasted and found links, and the
  welcome and `/web` link across wrapped rows. `run::tests` holds the
  recording keys and the start facts handed to the recorder.
- **`temper-client-walk-lines`, library binary
  `temper_client_walk_lines`:** `walk::tests` reads version-2 lines and
  chains, refusing wrong-client marks and bad ancestors;
  `tui_print::tests` holds rows, palette and attribute classes, escaping,
  cursors, key links, neighbours and the assets beside the pages.
- **`temper-client-walk-world`, library binary
  `temper_client_walk_world`:** `walk::tests` holds TUI key links and
  reads the checkout's walk sources and scripts without starting a
  world. No verdict reads main's generated pages.
- **`temper-playground-world`, integration binary `suite`:** `tui`
  holds the shipped TUI on the simulated service: browser approval and
  its address restored, no approval spending its bound, a chat's
  native stream, stopping and restarting, and reproducible frames.
- **`temper-client-walk-tool`, integration binary `suite`:** small
  walks of its own hold TUI marks, key links and bounds, printed assets,
  exact page counts, and the longer change and design-session flows
  (`../journey-walks.md`, 7). No test walks main's journeys to their end.
- **`temper-client-journeys-tool`, library binary
  `temper_client_journeys_tool`:** `tests` holds empty overlays and
  nonempty journey descriptions, redaction and print refusals;
  `tui_driver::tests` holds terminal key bytes and a save detected by
  replacement even when its bytes are unchanged. `drive::tests::tui`
  holds lazy startup, explicit approval in a journey, save replay,
  looks, bounds and stops; `journey_walk::tests` holds chains through
  one session, scripts, ancestor marks skipped and first-failure stop.
- **`temper-client-journeys-tool`, integration binary `suite`:**
  `admission` holds recording prints with resolving assets, layout
  refusal before proof, unchanged build, ports and approval codes, and
  no output on refusal. `journey_pages` and `look_refresh` hold the
  writer and request refresh machinery (`../journey-writer.md`, 7).

These tests start no terminal, browser or box. Native frames and byte
equality establish no real terminal's rendered look; section 8's box
session is after-landing work, not a plan's check.

## 8. On a box

After landing, outside every plan, the drive session takes the
journey's lines on a box or deployment (`../drive.md`, 2 and 3;
`../web/journeys.md`, 8). It writes no page of main's.

- `temper-client-journeys journey walk NAME --bin DIR [--bundle DIR]
  --scratch DIR [--strict] [--ports LIST] [--widths LIST]
  [--deployment DIR]` reads NAME's version-2 walk and its ancestors,
  passes their scripts root first, skips ancestors' marks and walks
  the lines through one journey session. It stops at the first failed
  line, reporting the source journey and file line. Its scratch must
  be fresh, with no `config/` or `data/`, and cannot overlap a journeys
  root. Nothing is built.
- `temper-client-journeys drive --journey` with the session's build and
  scratch options takes the lines interactively. It implies neither a
  browser load of `/` after sign-in nor the TUI's approval. A recipe's
  session keeps its own automatic sign-in (`../drive.md`, 3.3).

### 8.1 The TUI's lines on a box

| Line | What the journey session does | Bound |
|---|---|---|
| `tui key KEY` | gives the terminal KEY, as 3.2 writes it, with paste bracketed as a terminal's paste | no key bound |
| `tui see TEXT` | saves, replays natively to the end (section 4), and waits until that frame's text contains TEXT | 30 s |
| `tui approve` | saves until the TUI has asked to browse an address; remembers the browser's current address, opens the requested one, finds and presses the button named "Approve", then loads the remembered address | 30 s for the approval |
| `tui mark` | answers a look: the saved replay's final frame, a row per line with trailing spaces cut; not kept and not a page written | 10 s |
| `tui stop` | ends the TUI, or succeeds if none runs; a later TUI line starts one | 10 s |

The first TUI line starts it where none runs, but signs it in only
when that line is `tui approve`. A journey that needs approval says so.
A save that never arrives fails after 10 seconds, naming the need for
the client's recording build. Failed approval attempts return the
browser to the remembered address. `tui mark` is not the recorder's
mark key: it is a look, not another saved mark. Ordinary drive sessions
and recipes refuse journey-only marks and approval (`../drive.md`, 3.3).

### 8.2 The terminal

The session starts `BIN/temper http://HOST:PORT` on a pseudo-terminal,
80 columns by 24 rows, sized before the TUI can read its size
(`../drive.md`, 2.2, 2.3 and 3.3). Output is drained and dropped; saves,
not terminal escape sequences, give the frame.

- **Its own runtime:** `XDG_RUNTIME_DIR` is `session/tui-runtime/`,
  apart from the box's `session/runtime/`, so no engine's link file is
  read at start and no approval is bypassed. `HOME`, `XDG_STATE_HOME`,
  `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME` and `TMPDIR` are
  in the session's private directories.
- **Its opener:** `session/bin/`, first in `PATH`, holds `xdg-open` and
  `open` linked to `true`. The opener succeeds but starts no tab; the
  session's browser carries out the explicit approval.
- **Its terminal environment:** `TEMPER_INTERNAL_RECORD=1`,
  `TERM=xterm-256color`, no `SSH_` variables and no `COLORFGBG`;
  `DISPLAY` is inherited or `:0`. The pseudo-terminal leaves OSC 11
  unanswered: Auto draws dark with no band (`theme.md`).
- **Its saves:** the session sends Alt+Shift+S and observes a file's
  replacement by device and inode, not changed bytes, then reads the
  whole save through the native replay (section 4). The observer
  currently looks for `session/state/temper/tui/recording`, whereas
  the recorder writes `recording-TIME` (3.1). Following that per-run
  file is a gap in [journeys.status.md](journeys.status.md); a missing save times out
  as 8.1 has it.
- **Its end:** the session stops the TUI and its process group, then
  the browser and box (`../drive.md`, 2.4). Its private session files
  stay until the next session, never becoming main's pages.

### 8.3 A box with the TUI in front

`temper box` can put the TUI in front of its child box (`box.md`;
`../../protocol/setup/box-front.md`, 1). A person can record that TUI by hand
and print the saved file with 6.1's recording command.

- **The printed link:** the host hears the box's `link` line as a
  `Front` report, recorded as `box-line`. Its secret is a *link*
  original, as is a handed local link (section 5). `/web`'s `browse`
  holds the same secret in the printed link's URL.
- **The welcome's link, wrapped:** at 80 columns the welcome and
  `/web`'s note draw the link across rows. Redaction and the last scan
  seek its URL-base64 across row breaks and indentation, keeping that
  whitespace while replacing the secret. A redacted recording can
  prove against the live frames across the wrap; this needs no change
  to the box driver.
- **A journey's walk needs a front host:** the drive session starts
  `temper` at the box's address, the box apart on pipes, and the world's
  TUI starts not in front. Neither starts `temper box` in the terminal
  for a journey or carries its front reports. The world refuses the
  host's stop and terminal-lending duties; no walk line lends a
  terminal or starts a box in front. The welcome's own frames and a
  loan's return are covered by host tests, not a journey's walk.

## 9. The journeys

| Name | The walk | Its pages |
|---|---|---|
| `sign-in-approved` | version-2 `walk` in one process on the playground's composition, no `temper box` or terminal (`../journey-walks.md`, 3.3, and `../journey-walks-world.md`, 1, and `../journey-walks.md`, 6): the browser signed in by the printed link at `/welcome`; the TUI started at the service's address, not signed in; the approval's frame seen and marked; `tui approve`, the approval given in the browser; the launchpad seen and marked | `01`, the approval's frame, "A tab opened in your browser", its code and its address; `02`, the launchpad of a service with no project (`pages/README.md`); its `sign-in-approved/README.md` describes the world's frames |
| `launchpad-chat` | version-2 `walk` in one process on the playground's composition, `after project-landing`, with its `scripts.json` (`../journey-walks.md`, 6): Hello with its owned `work/hello` and the ChatGPT plan; the TUI approved as `sign-in-approved`'s is; the launchpad marked; `/` typed and the commands marked; Esc; a question typed and marked, then sent, which opens an Ask temper chat; the scripted answer marked | `01`, the launchpad in Hello; `02`, the commands under the box; `03`, the question typed; `04`, the chat answered (`pages/README.md`); its `launchpad-chat/README.md` describes the world's frames |
| `launchpad-drawn` | version-2 `walk` in one process on the playground's composition, `after project-landing`, with its `scripts.json` (`../journey-walks.md`, 6): Hello, the TUI approved, to the launchpad; `/change` and a change asked in words, marked, then sent, which starts a chat; its scripted model starts a goal and says that a question waits, and the goal asks the chat's person a question with two choices, the first recommended, as a goal a chat calls asks (`../../domain/agent/agents.md`, 2); `/new` typed and marked, then sent, back to the launchpad, the chat under *Where you left off*, its question under Needs you and the quiet line's count, marked; `n` and the walk's prompt, marked; its first choice taken; the launchpad after the walk, marked | `01` and `02`, the words typed before Enter sends them; `03`, the launchpad as drawn, with the chat, its need and Start (`pages/launchpad.md`), `n` its key; `04`, the walk's prompt under the box (*Walking what needs you*), Enter its key; `05`, the launchpad after the walk, its note; its `launchpad-drawn/README.md` describes the world's frames |
| `design-session` | version-2 `walk` in one process on the playground's composition, `after project-landing`, with its `scripts.json` (`../journey-walks.md`, 6): Hello, the TUI approved, to the launchpad, marked; `/draft` and the work asked in words, marked, then sent, which starts a design session; the scripted drafter's reply writing the draft and ending with numbered choices, the pane at half; Ctrl-P to all, to folded and to half again, each marked; "1" typed, marked and sent, the next reply changing a section, marked new, and saying nothing blocks the work, marked; `/goal` typed and marked, its prompt of ways marked, the first taken; the chat again, its goal linked: `/goal` no longer offered, and the quiet line without the goal's part, which at 80 columns goes first (`pages/design-session.md`, 1, *narrow*; the goal's part is the 110-column golden's), marked | `01`, Hello's launchpad; `02`, the `/draft` words typed; `03`, the session at half with the drafter's choices under the box; `04`, the pane at all; `05`, folded; `06`, half again with "1" typed; `07`, Open questions marked new; `08`, `/goal` typed; `09`, `/goal`'s ways prompt; `10`, the returned chat with its goal linked (`pages/design-session.md`, 1); its `design-session/README.md` describes the world's frames |
The following are journeys the TUI wants, not directories or walks on
main. Their section 9 gaps are in [journeys.status.md](journeys.status.md); they have no pages to
open or recording to supply them.

| Name | The wanted walk | The frames it would show |
|---|---|---|
| `box-welcome` | the TUI in front of a box (8.3), from its welcome through the browser's steps to the playground | the welcome before the link is opened, a step under way, and the playground's launchpad (`box.md`) |
| `box-claude-sign-in` | the TUI in front, the welcome at Model with Forgejo connected; Claude Code's sign-in cancelled, then completed with a fake program, returning the terminal to the TUI | the welcome following Model, the cancellation note and the success note (`box.md`, *Lending its terminal*); the lent terminal itself is outside a TUI frame |
| `role-talk` | a standing role in Hello, `/talk`, words and its reply, an instruction proposed as a rule and accepted, then the launchpad and `/tell` | Team on Hello and Start's `/talk`; the talk's Now, its choices and rule prompt; the notices after leaving and filing a finding (`pages/talk.md`; `pages/launchpad.md`, 7) |

Each needs a version-2 walk and README, with scripts where its model
answers. The front-box pair also needs a driver that starts and carries
the front host (8.3), and the second a terminal loan. `role-talk` needs
its standing-role state expressed through client lines: a journey
admits no `ctl` line (`../journey-walks.md`, 2.2).

## 10. Editing the look

As the browser's (`../web/journeys.md`, section 7), on the TUI's print:

- **On the print:** the person serves the checkout, walks a TUI journey,
  and writes rules in `tui/journeys/overlay.css` (a palette's custom
  property, a class's paint), then reloads; a frame's words, order or
  layout is edited in place, in its `NN.txt` or `NN.html`.
- **Requests stack** on a branch of the person's own, `tui-requests`,
  apart from the browser's `look-requests`, so that the two are asked
  for in parallel: one commit each, with a `Look-request: NAME` trailer
  whose NAME starts `tui-`, kept based on `main` by the same
  `scripts/look-refresh.sh`, run with `tui-requests` checked out, and
  never landed (`../web/journeys.md`, section 7;
  `../web/look-requests.md`, 2).
- **Never landed:** the delivery check refuses a request's frames
  beside any other file, and its overlay (`../journey-writer.md`, 5), so
  `main`'s overlay stays empty.
- **Done** is a landing carrying the request's trailer in commits of its
  own that moves the overlay's rules into `temper-client-tui`'s theme
  (its palette, its tokens' paints, its borders and glyphs) and the
  frames' changes into its view and words. It compares by
  `temper-client-walk print --out DIR`, DIR's pages against the request's
  commit, equal where their facts agree, and commits no frame
  (`../web/look-requests.md`, 3). The writer prints main after landing.

## 11. Open questions

- **The rendered look:** a world and a native replay establish no real
  terminal's look or macOS runtime result. The user refines the frames
  through section 10's requests; the rendered check and the box session
  follow landing, outside a plan (`../web/look-requests.md`, 4).
  A growing answer, calls and a question's choices remain for a
  journey that catches them; the joint world and goldens cover them
  without establishing the rendered look (`../testing.md`).
- **Selectors by token:** a page's cells carry their paint, not the
  view's token, so an overlay retunes a palette colour or a paint
  wherever it falls, never one token alone. Default: palette classes;
  a token's class waits for a request that needs it.
- **Both schemes on one page:** a page draws one scheme. Default: one;
  a key that swaps `tui-dark` for `tui-light` is a request's.
- **The place "box" in a journey:** the world's TUI is not on a box and
  reads no engine's link file (`../journey-walks.md`, 3.3), so no walk
  draws "box" in the rule above a chat's composer (`pages/chat.md`, 1,
  *The rule above the box*). Default: the service's address, until a
  walk can sign in by the engine's link file or run in front of a box.
- **A resize in a walk:** a recording keeps it and replays it as a
  report; a walk has no line for it until a journey needs one.
