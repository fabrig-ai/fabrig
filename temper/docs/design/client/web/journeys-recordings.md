# Recording and replaying journeys

Recording and replaying journeys are split from [the browser's journeys](journeys.md).

## 1. The recording

### 1.1 The recorder

- **On by the page's setting or its meta:** the page loaded with
  `internal-record=1` in its address, on any origin, or made with the
  `temper-record` meta, which `--record` asks of the deployment, runs
  the recorder (`shell.md`, 1, and `serving.md`, 1); without either, the same
  bundle records nothing and installs no key. The recorder needs no
  other setting. It adds nothing to the page: no node, no style, no request. The public
  pages (`public.md`) start no shell, and record nothing.
- **What it hears:** in `Shell::turn`, beside the host, the start's
  facts once, then the recorded turns' clocks, reports given to
  `Host::report` and the duties it returned, and each turn's deadline.
  `Push` and `Screen` reports have no recording record; 1.2 lists what
  the recorder keeps.
  It keeps them in memory as the file's lines (1.2), through
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
  a whole recording and a later one holds more. The person saves at the
  end of what they want to show. A key's turn that gives the host
  nothing, a save's, is not kept.
- **One load of the page.** A recording starts at that load's path and
  is whole at each save. A sign-in is no
  departure: the recording goes on through it and its end, the tab's
  address kept as its answer's is. A departure, a forge's or an
  authorization's, ends it: when a report's duties depart
  (`navigate`, `replace` or reload), the
  recorder keeps them, marks the end as the page is drawn then, and
  saves, before the shell carries the departure out; the rest of that
  turn is not kept, and the recorder keeps nothing more. A reload starts
  a new recording, and so does a return from a departure when the new
  load enables the recorder (`shell.md`, 1).
- **No sign-in in the file.** Each save, Alt+Shift+S's and the one
  before a departure, writes the recording with two of `journey-redaction.md`'s originals
  replaced as `print` replaces them, an `x` a byte, in every field and in
  a stream's joined records alike: the *link*, the `secret` record's
  value, and the *stored*, each `stored` record's value and each `save`
  of the stream's record, a `Kept`'s head standing. So a saved
  recording is never a sign-in. Nothing else is replaced: a typed
  value, such as a key pasted into the page, and what agents wrote stay
  for `print`'s redaction and last scan (`journey-redaction.md`; 1.7); on
  a real deployment, as on a real box, the person types no key (1.5). A
  recording whose streams don't decode (`journey-redaction.md`) is written with those two
  replaced in every field and every `heard piece` all `x`s, its size
  kept: no such recording can pass `print`, and none is a sign-in. The
  memory keeps the originals, so the recording goes on as it is, and
  the replacement is `journey::saved(recording: &[u8]) -> Box<[u8]>`, a
  pure function of the recording's bytes, which the shell's save calls
  on `Recorder::save`'s or `Recorder::end`'s file before the download,
  and the host's tests call alike; those two still give the recording
  as kept.
- **Bounded:** `limits::RECORDING_BYTES`, 64 MiB, bounds the recorder's
  admission of report, act and mark calls (`shell.md`, 1, `limits`).
  The start's facts are kept whole. A call whose pending lines,
  including its turn prefix, would take the file past the bound is not
  kept: the recorder writes an end mark of the page as drawn then and
  keeps no more. The turn's deadline and the closing end mark are not
  part of that admission size check. The host goes on as before. A
  later save gives what was kept; a reload starts a new recording.

### 1.2 The file

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
| `zone` | signed decimal offset in minutes | after `primary`, omitted when zero; the reader defaults to zero |
| `ways` | the `temper-sign-in` meta's content, the page's origin | when the page has the meta |
| `forge` | a forge sign-in meta's content | one per fact, in order; an off fact has the prefix `off ` |
| `path` | the address's path and query | |
| `secret` | the printed link's secret | when the address held one |
| `stored` | a small-store record's key and value | one per record read at start |
| `turn` | now and wall | each turn's clocks |
| `start`, `due`, `place`, `returned`, `tab-refused`, `heard`, `carried`, `act`, `copied` | the report | each recorded report, in its turn's order |
| `tab-open`, `tab-point`, `tab-close`, `copy`, `exchange`, `pieces`, `open`, `close`, `save`, `erase`, `writes`, `navigate`, `replace`, `show` | what the host asked | after its report, in order, the tab's duties before copy and transport |
| `deadline` | nanoseconds, or `none` | each turn's end |
| `mark` | the live page's html | before its act, or at the key, in a turn of its own (`turn`, `mark`, `deadline`) |
| `end` | the live page's html | the last line |

The recorder writes `journey 2`; the replay reads version 2 alone,
refusing another version. Each line ends in a newline, including the
last. The start's facts must be in the table's order; `end` occurs once,
last. The record codec is `journey/line.rs`, the start's reader
`journey/replay.rs`, in `temper-client-shell-web`.

- **The reports:** `start`; `due`; `place PATH`; `returned PATH`;
  `tab-refused`; `heard answered EXCHANGE STATUS BODY`,
  `heard begun EXCHANGE STATUS`, `heard part EXCHANGE BYTES`,
  `heard done EXCHANGE`, `heard failed EXCHANGE`, `heard opened`, `heard refused STATUS`,
  `heard piece BYTES`, `heard ended`; `carried WRITES`, then one line
  per carried report, `written KEY`, `unwritten KEY` or `lost
  EXCHANGE`; `act ACT [VALUE|EDGE] EVENT NAME` (1.3); `copied TOKEN
  done|refused|failed`.
- **The asks:** `tab-open WORDS`; `tab-point TO`; `tab-close`;
  `copy TOKEN WORDS`; `exchange ID BOUND METHOD PATH BODY`, then its headers
  as `NAME VALUE` pairs on the same line; `pieces ID BOUND PIECE METHOD PATH BODY`,
  then its headers as `exchange`'s are; `open PIECE PATH` and its
  headers; `close`; `save KEY VALUE`; `erase KEY`; `writes N`, when not
  zero; `navigate TO`; `replace TO`; `show PATH push|replace`.
  Copy words are redacted as navigation addresses are.
- **The marks:** a saved file holds HTML in each `mark` and `end`.
  The codec also reads a bare `mark` or `end`; the proof refuses one
  without a live print. Successful proof drops that HTML from its
  in-memory records before the printing replay (2.3). No recording is
  written by `print`.

### 1.3 An act's event

- **The act** is skein-ui's: `press`, `submit`, `toggle`, `input` with
  its value, `drawing` with its bytes, or `near head|foot`, the log
  part's report as it approaches an edge.
- **Its event** is the one the view makes of it, read through
  `Host::event_of` before the act is given: for a press, the client's
  action, such as a `Go` to an address, in the domain's own names. It is
  kept as the domain `Event`'s Debug print, compact, as bytes, which the
  replay compares and nothing parses. Not the control's role, words or
  place, so that the view may rename, restyle or move it and the
  recording can still replay. A change to the domain's names or to how
  Rust prints them can make it unreadable to the replay. A walk's
  controls are different: role, exact accessible name and order (`journeys.md`, 8.2).
- **The control's accessible name follows,** for a reader: the node's
  name, or else its subtree's text, as skein's fake person reads it. It
  is never checked.
- **An act the view makes no event of** is given to the host, which
  does nothing with it, and kept nowhere: no line, no mark.
- The welcome's "Set up temper" is kept as `act press "Act { action:
  Pages(Go { address: ... }) }" "Set up temper"`; a typed secret's event
  shows its length alone (`AccountKey { value: Secret { len: 12 } }`),
  its value being the act's until `print` takes it out (`journey-redaction.md`).

### 1.4 Marks

- **A mark is a page as the person saw it:** as drawn when an act came,
  before the act's turn changes it; as drawn when the mark key was
  pressed; and at the recording's end. Either way it is the page as the
  last turn's end drew it, so the replay takes a mark wherever it sits. Each keeps the live page's
  `Page::html()`, for the first replay's proof.
- **Typing marks nothing.** An `input` act is recorded and replayed but
  marks no page: a printed field takes typing as the live one does
  (`journeys.md`, 5.3), and the next act's mark shows what was typed. A `near` report
  also keeps no mark. These are the recorder's rules; a walk prints
  only its explicit `web mark` lines (`../journey-walks.md`, 2.4).
- **The page's own state:** skein's page draws one state that its host
  never hears of, an open menu (`Role::Menu`), and a mark taken with
  one open fails the first replay's proof. A recording that needs this
  state needs the page's events recorded too (`journeys.md`, section 10).

### 1.5 A person's file

A person records by hand on a deployment or a box of their own, with
fake backends or real accounts. A fresh `temper box --fake-backends`
starts as the help's `box` page describes (`temper help box`); its
printed sign-in link opens the browser. Any deployment can record by
`internal-record=1` or `--record` (1.1), serving its own bundle: no
separate recording bundle is needed.

A recording keeps what the page was given, including the model's words
and what agents wrote. On real accounts the person uses public
repositories and writes nothing private into chats. The saved browser
file holds no link sign-in or stored credential (1.1); typed values
still need care. `print FILE OUT` redacts every file alike, fake or
real (`journey-redaction.md`), and no recording is committed. The
person's pages may be drawn on wherever OUT is, as a hand-made page's
may (`look-requests.md`, 2).

A file belongs to the build that recorded it: the host's events, wire
and saved marks are that build's. Print it with that build. The tool
refuses a different layout by name (`journeys.md`, 5.1), not by trying to turn it into
a journey of main's.

### 1.7 The last scan

Before any page is written, `print FILE OUT` scans the redacted
records, the pages the proof draws and the documents to be printed.
The record scan comes before the proof; the drawn-page scan runs at
each turn's end during it; the printed-page scan follows the printing
replay (2.3).

- **Where it looks:** every field of each line, a stream's records
  decoded and its other text among them; each run's text and its
  writing call's input, joined by their offsets (`RunText`,
  `WritingCall`; `jig/docs/design/protocol/client/README.md`, 5), since a
  model's stream splits a token across deliveries that no one line
  shows; the page the
  replay drew at each turn's end, its escapes read back, so that what a
  page joins is seen, and each printed page.
- **What it refuses:**
  - *an original:* any form of a sought original (`journey-redaction.md`), or its sha-256;
  - *a key:* `sk-`, then at least 20 of `[A-Za-z0-9_-]`;
  - *a JWT:* three base64url parts joined by `.`, the first beginning
    `eyJ`, each at least 8 long;
  - *a bearer:* `Bearer `, any case, then at least 16 of
    `[A-Za-z0-9._~+/=-]`;
  - *a code:* `code=`, not after a letter or digit, then at least 8 of
    `[A-Za-z0-9._~%+/-]`;
  - *a refresh token:* `refresh_token`, up to four of `"`, `\`, `:`, `=`
    and spaces, then at least 8 of `[A-Za-z0-9._~%+/-]`.
- **A value of `x`s alone is a redacted value,** and passes.
- **A key inside a word is none:** `sk-` after two bytes of
  `[A-Za-z0-9_-]` (`task-pages`) doesn't match. One such byte may be a
  field's length prefix, which reads as a letter or a digit: a 51-byte
  key's reads `3`.
- **A refusal** names the record and the kind, never the bytes:
  the line and its record's kind, and where in it (a field, a stream's
  record, a run's text) or the turn or page that drew it. Nothing is
  written. An original is taken out by `print`, so a refusal of one is
  a split or a fault; a shape within one field can be taken out by a
  `--redact` before another print.

### 1.8 What is left to the person

The scan's shapes aren't all a secret could be. `--help` says so, and
`print` repeats the caution after its counts and destination. Stated here so that a person knows
before they record:

- **Any other private text** (a name, a host, a token of another shape)
  stands unless a `--redact` names it: read the pages before sharing.
- **Split across deliveries:** a run's text or a writing call's input is
  seen, joined, and refused, but not replaced: a `--redact` can't reach
  it. A person must make a recording that leaves that text out. Text split over any other records
  is seen only where a page draws it whole at some turn's end; if none
  does, it isn't seen.
- **Part of a secret** isn't sought: a key's last four on a page, or an
  original's head cut off by the recording's own end.
- **Other encodings** aren't sought: an original base64-encoded inside a
  field, say. No client exchange compresses what it carries.
- **A value the client reshapes** before it carries it isn't found whole.
  A GitHub App's key has its armour's line breaks restored, so its
  exchange no longer matches and the file is refused. Such a step
  cannot be printed from that file.
- **A `stored` request:** a request's `Kept` record, in flight at a
  reload, doesn't read as `x`s, so the replay fails and the file is
  refused: reload a settled page.
- **What the person holds:** the saved FILE keeps every original but
  the browser's link and stored (1.1), a typed value among them, so it
  stays out of the repository and goes once it is printed; a
  `--redact TEXT` stands in the shell's history. The printed pages stay
  outside the repository's journeys too (`journeys.md`, 5.1).

## 2. Replay

### 2.1 Through the shipped host

`journey::Replay`, native, starts the host from the recording's facts
as the shell's start does (`Host::new`, `forge_sign_in`, `start`) and
the page by `host::page`. For each turn it sets the clocks, gives each
report through `Host::report`, an act to the first node, in tree order,
on the page at that point of which the view makes the same event
(1.3), and draws at the turn's end. At each mark it yields the page
and, for an act's mark, every node on it that gives the act's event.

### 2.2 The check

- **Each report's duties,** written as their records, are the recorded
  lines, and each turn's deadline is the recorded one. The recording's
  answers are to those requests: a client that asks otherwise (a new
  read, another request's bytes, another deadline) would be answered
  what it never asked.
- **The replay stops** at the first duty or deadline that differs, a
  departure or a tab's open, point or close among them, or at an act
  whose event no control gives. `print` names the turn, recorded line
  and the recorded and asked record kinds, never their bytes (2.3).
  This is a refusal of a person's file, not a state of main's journeys.
- **Patches are not compared:** they are what the print shows.

### 2.3 The first replay's proof

`print FILE OUT` checks the layout and redacts the saved recording
(`journey-redaction.md`). It scans the redacted records (1.7), then
replays them with their redacted live prints: at each mark the replay's
`Page::html()` must be the saved HTML. A missing live print is refused;
a different one names the mark and the first byte that differs. A
replay that does not draw the saved page is not faithful. The defect
is fixed in the seam, redaction or replay, never hidden in the proof.

During that proof the drawn page at each turn's end is scanned too.
Once every mark matches, the saved HTML is dropped from the in-memory
records. Those records are replayed to make the print; no replacement
of varying facts follows the proof. Each printed document is scanned,
then the pages and their assets are written. No redacted recording is
written. All proof and scan refusals occur before output is written.
An I/O failure during writing may leave partial output.

A refusal names position and kind alone: a turn and line, a mark and
byte, a record's kind or a scan's kind of secret, never the record's
text, which could hold what redaction missed. The tool's safe refusal
is shared by its library and command.

## 3. Open questions

- **The page's own state at a mark** (`journeys-recordings.md`, 1.4): where a journey meets a
  skein menu, the page's events would be recorded and replayed beside
  the host's reports.
- **What a person's file cannot print** (`journeys-recordings.md`, 1.8): a token split across a
  run's deliveries is refused but not replaced, and a value the client
  reshapes can fail replay. Replacing these waits for a recording
  that needs them and a replacement that can preserve the proof.
- **Usage and spend figures** the page was given stay as they were in
  a person's print; whether they should.
