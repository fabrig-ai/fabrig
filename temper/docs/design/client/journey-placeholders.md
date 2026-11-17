# A journey's placeholders

Provisional, 2026-11-17. A journey's recording, as `add` writes it,
holds a fixed placeholder for each value that varies between two walks
of the same screens and isn't a secret: the build, the loopback ports
and the approval codes. A browser's print also numbers skein-ui's
generated ids afresh on each page. Two recordings that differ only in
those values then normalise to the same bytes, and their prints are the
same wherever their screens are. Both clients' journeys share this
(`web/journeys.md`; `tui/journeys.md`), written from
`../draft/journey-placeholders.md`. Section 2 names the values, 3 how
they are replaced, 4 where `add`, `again` and the committed recordings
take them, 5 a print's ids, 6 the code, 7 its tests, 8 what is open.

## 1. In one page

- **Why:** every branch that records a journey again rewrote each of its
  files, even where no screen changed, since they held the build and the
  fake box's ports; two such branches then conflicted on every file.
- **What varies, replaced:** the TUI's build, as its start's `version`
  fact holds it; every loopback port; each approval code the TUI was
  shown (section 2). Each becomes a placeholder of its kind, the same
  bytes whatever the original's length, numbered by first appearance
  where a recording holds several.
- **Where:** in every field and every form, as redaction seeks its
  originals (`web/journeys.md`, 3.6), so the recording and its prints
  agree. A framed field takes a placeholder of the original's own width
  alone; an original that one can't take stands everywhere (3.3).
- **When:** `add` redacts, proves, then normalises, and the normalised
  recording must replay as the check has it before anything is written
  (4.1). `again` compares walks normalised (4.2). The committed
  recordings are normalised once, by `normalise`, and the print's test
  holds every committed recording to its own normalisation (4.3).
- **What stays:** times, the seed and the turns, which the replay needs
  as they were (2.4). So two walks' prints are the same bytes except
  where a page draws a time; their recordings still differ in their
  clocks and seed. A print reads the recording's clocks, never the
  printing host's, so one recording prints the same bytes on any day.
- **A print's ids:** skein-ui's ids follow the order the page's nodes
  were made, which the walk's turns decide; the browser's print numbers
  them afresh on each page (section 5).

## 2. What varies

### 2.1 The build

- **The original:** the TUI's start fact `version` (`tui/journeys.md`,
  3.2), when it is not empty and not the placeholder: the client's build
  as `VERSION (COMMIT)`, COMMIT twelve hexadecimal digits with `-dirty`
  when the checkout had tracked changes, or `unknown` (`temper-version`).
- **The placeholder:** `0.0.0 (000000000000)`, whatever the original's
  version, commit and dirty mark.
- **A browser's recording holds none:** its page is given no build.

### 2.2 Loopback ports

- **The originals:** each port of a loopback address in any field:
  `127.0.0.1:`, `localhost:` or `[::1]:`, then one to five digits that no
  digit follows, a port from 1 to 65535. The port is the original, keyed
  by its number, so one port under two hosts is one original; it is
  sought only after one of those hosts and its colon (3.1), so no other
  number is touched.
- **The placeholder:** `10001` for the first distinct port by first
  appearance (3.2), `10002` for the second, and so on to `10099`; a
  recording's hundredth distinct port and the later ones stand (3.3). It
  is a port no fake box takes, below every kernel's range for the ports
  it hands out (from 32768 on Linux, 49152 elsewhere), and of the five
  digits those always have, so that a framed field keeps its width.
- **Other hosts stay:** a real deployment's address is no loopback, and
  a real journey keeps it (`web/journeys.md`, 3.5).

### 2.3 Approval codes

- **The originals:** each approval code the TUI was shown, read from
  each `browse` address's `code=` by `ApprovalCode`'s reader, as the
  last scan reads it (`tui/journeys.md`, 5): its eight letters, as the
  wire carries them, and its shown form, four letters, a hyphen and four.
- **The placeholder:** the n-th distinct code by first appearance, from
  0, is the code alphabet's n-th letter eight times: the first
  `BBBBBBBB`, shown `BBBB-BBBB`, the second `CCCCCCCC`. A recording's
  twenty-first distinct code and the later ones stand.
- **A browser's recording holds none:** no browser journey approves a
  code.

### 2.4 What stays as recorded

- **Times** stand: each turn's clocks, the start's, a note's `until`,
  the browser's `zone`, and the engine's times inside the replies. A page
  draws a time in forms no substitution maps back ("Tuesday, 16:06",
  "today 16:07"), and the engine's times sit in wire bytes as bare
  numbers that nothing marks, so a shifted clock could neither be found
  whole nor proved (`web/journeys.md`, 4.3). So two walks' prints are
  the same bytes except where a page draws a time, and their recordings
  still differ in their clocks and seed. This departs from the draft,
  whose two recordings of one journey are byte-identical, prints
  included; the user agreed on 2026-11-17
  (`../draft/journey-placeholders.status.md`). A fixed clock and seed
  for `again`'s walks would close it (section 8).
- **A print reads no host's clock:** the replay that prints takes each
  turn's clocks and the browser's `zone` from the recording
  (`web/journeys.md`, 4.1; `tui/journeys.md`, 4.1), and the print adds
  none, so one recording printed on another day, or on another host,
  gives the same bytes. The print's test holds it, reprinting every
  committed recording on whatever day it runs (4.3).
- **The seed and the turns** stand: the host draws from the seed, and
  the turns' clocks, deadlines and a stream's pieces are when things
  happened, which the replay gives back as they were. Two walks never
  share them, so their recordings differ there.
- **What the fake box makes the same each time** needs nothing: a fresh
  box numbers its projects, chats and tasks alike.

## 3. How they are put in

### 3.1 Every field, every form

- **Every field** of every line, framed or text, a browser stream's
  records joined and decoded first (`web/journeys.md`, 3.6, *A stream's
  pieces are joined first*), and a mark's and the end's html or print
  where they hold one.
- **Every form** redaction seeks (`web/journeys.md`, 3.6, *Where*): the
  bytes, percent-encoded in either hex case, `dom::print`'s escapes and
  Rust's Debug escapes; and the TUI's own, a pasted address's decimal
  byte elements in a `Typed` event's print (`tui/journeys.md`, 5, *A
  pasted link's encoding*). A port's forms are those of its whole
  address, its host and colon with it, and only the port's digits are
  replaced. No digest is sought: nothing carries one of these values'.
- **Each form becomes the same form of its placeholder.** All forms of
  all originals are sought in the field as it was, so a placeholder is
  never sought again, and two originals swapped meet no chain.

### 3.2 By first appearance

The recording's lines in order; on a line, its fields in order, then
the records of a stream that start on it. Ports and codes are numbered
by where each distinct value first appears, so the box's own address
comes first (the browser's `ways` origin, the TUI's `address`), then
Forgejo's, as every walk of the journey makes them.

### 3.3 A framed field: its width, or the original stands

A framed field (`web/journeys.md`, 3.6, *A form's size*;
`tui/journeys.md`, 5, *Where*) holds wire bytes whose lengths nest, a
`text` inside held `bytes` inside a message
(`skein/docs/design/data/codec/README.md`, 3), and the recording doesn't
know where they are.

- **Of the original's width,** a placeholder replaces the form there and
  the field keeps its length.
- **Of another width,** it can't: that original stands, in every field
  and form, so the recording stays consistent and its replay holds.
  `add` names its kind and counts it as stood, so whoever adds sees why
  that recording will still change with the walk (4.1).
- **In the committed journeys none stands:** their framed ports have
  five digits and every code eight letters, and the build is only in a
  text field. Ephemeral loopback ports have five digits on Linux and
  macOS alike, so a port that stands is a case no committed journey has.
  This departs from the draft, whose framed field's length would follow
  the placeholder; the user agreed on 2026-11-17
  (`../draft/journey-placeholders.status.md`).

### 3.4 After redaction

- **On the redacted recording:** normalising follows redaction, which
  stays as it is. A value of `x`s alone is no varying original, and one
  inside a secret went with it, its shape gone.
- **A placeholder is no secret's:** never `x`s, and the last scan
  (`web/journeys.md`, 3.7) reads the normalised recording as any; a
  placeholder code is an approval code, which the TUI's scan cuts from
  what it reads as it cuts any shown code (`tui/journeys.md`, 5, *The
  last scan*).

### 3.5 The same bytes again

Normalising is a pure function of the recording's bytes, and normalising
a normalised recording gives its bytes: each placeholder is found as an
original of its kind, in the same order, and becomes itself. So one
recording given other builds, ports or codes normalises to the same
bytes, and a committed recording is its own normalisation (4.3).

## 4. Where it runs

### 4.1 `add`

In this order, everything in memory until the scan has passed
(`web/journeys.md`, 5.1; `tui/journeys.md`, 6.1):

1. **Redacts** (`web/journeys.md`, 3.6; `tui/journeys.md`, 5).
2. **Proves the redacted recording** against its live prints
   (`web/journeys.md`, 4.3; `tui/journeys.md`, 4.3), which drops the
   marks' html or prints.
3. **Normalises** it (section 3).
4. **Replays the normalised recording,** which must hold as the check
   has it (`web/journeys.md`, 4.2; `tui/journeys.md`, 4.2): a stale one is
   refused as the proof's stale is, naming the turn and the recorded
   line's kind, never its text.
5. **Scans** the normalised recording and the prints from that replay
   (`web/journeys.md`, 3.7); the pages drawn on the way were scanned in
   the proof's replay, to which normalising adds no secret.
6. **Writes** the normalised recording and that replay's prints.

- **Why the proof comes first:** a terminal frame lays its rows out by
  width, the header's padding, a rule's dashes and a truncation moving
  with a value's length, so a build placeholder of another width can't be
  put into the live frames by substitution. The proof shows the replay
  draws what the person saw; a consistent substitution keeps that, and
  step 4 refuses any ask it changed, and nothing is written unless it
  holds. The build's placeholder is one string for dirty and clean
  builds alike, so their prints don't differ. This departs from the
  draft, whose proof passes on the normalised recording; the user agreed
  on 2026-11-17 (`../draft/journey-placeholders.status.md`).
- **Its counts:** after redaction's line, `add` prints `placeholders:
  build: B; port: P; code: C; places replaced: N`, the distinct
  originals replaced of each kind and the places replaced. When an
  original stood (3.3), a second line names each kind that kept one and
  counts its distinct originals, in the order build, port, code, kinds
  with none left out: `stood as recorded, so this recording still
  changes with the walk: port: 1`.

### 4.2 `again`

- **The browser's** (`web/journeys.md`, 8.1): a step's event, from the
  committed recording, holds placeholders where the live page's holds
  the fake box's values. So each still save is normalised before it is
  replayed to find a step's control, and the new recording's walk is
  checked against the old one's with both normalised. A typed value is
  typed as the committed recording holds it, a placeholder included; no
  journey types one.
- **The TUI's** (`tui/journeys.md`, 8.1): its `walk` is a source and
  holds no varying value, its keys are checked against the new recording
  as it was saved, and a `see` is found in the live frames, so a `see`
  names no varying value.
- **Then `add`,** which normalises (4.1).

### 4.3 The committed recordings

- **`cargo run -p temper-client-journeys -- normalise`** takes every
  committed recording of both directories that isn't its own
  normalisation, normalises it, replays the result as 4.1's step 4 does,
  and writes it, naming each it wrote; then it reprints both directories.
  A recording whose normalised replay is stale is named with its turn and
  line and left as it was, and the program exits 1 once the others are
  written. It reads no configuration, environment, clock or network, as
  `add` and the reprint don't (`web/journeys.md`, 5.1).
- **Run once when this lands,** so that no journey is recorded again and
  no box is run for it, and again when a later change adds a kind.
- **Held by the print's test** (`web/journeys.md`, 6;
  `tui/journeys.md`, 7): it fails while a committed recording isn't its
  own normalisation, naming `cargo run -p temper-client-journeys --
  normalise`.

## 5. A print's generated ids

The browser's print (`web/journeys.md`, 5.3) numbers skein-ui-web's
generated ids afresh on each page, as it drops `data-node`:

- **What they are:** skein-ui-web names a node's parts by the node's
  number, `skein-ui-WORD-N` (`menu`, `options`, `option`, `label`,
  `description`, `state`), and an SVG gradient's `skein-ui-fade-N-I`.
  Those numbers follow the order the page's nodes were made, which the
  turns a walk took decide.
- **Numbered afresh:** within a page, each WORD's distinct numbers
  become 1, 2 and on, by where each first appears in the page's html,
  everywhere it stands: an `id`, an `aria-` reference, a `url(#...)`. A
  gradient's `-I` stays.
- **Nothing else changes:** the recording holds no html once added, and
  the TUI's pages name no node.

## 6. The code

- **`temper-client-redaction`** holds the shared part: the kinds,
  their placeholders and forms, finding the loopback ports, the numbering
  by first appearance, the framed-width rule and the counts. What crosses
  to the shells and the tool:
  - `pub enum Varying { Build, Port, Code }`;
  - `pub struct Normalised { pub recording: Box<[u8]>, pub build: u32,
    pub port: u32, pub code: u32, pub places: u32, pub stood: Stood }`:
    the distinct originals replaced of each kind, the places replaced and
    the distinct originals that stood;
  - `pub struct Stood { pub build: u32, pub port: u32, pub code: u32 }`:
    the distinct originals that stood, by kind.
- **Each shell's `journey` module** finds its own originals, from its
  lines and fields as `redact` does, and re-exports the three types:
  - `temper_client_shell_web::journey::normalise(recording: &[u8]) ->
    Result<Normalised, Unredacted>`, refusing what `redact` refuses;
  - `temper_client_shell_tui::journey::normalise(recording: &[u8]) ->
    Result<Normalised, Unreadable>`.
- **`temper-client-journeys-tool`** calls them in `add` and the TUI's
  `add` (4.1), in the browser's `again` (4.2), in `normalise` and the
  print's test (4.3), and numbers a page's ids in `print::pages`
  (section 5).

## 7. Its tests

Focused and native, within the journeys' cost (`web/journeys.md`, 6);
no fuzzy test, no browser, terminal or box.

- **The shared part:** each kind found and replaced in each form; ports
  numbered by first appearance, one port under two hosts one original;
  a number not after a loopback host untouched; idempotence; an original
  of another width than its placeholder in a framed field standing
  everywhere, counted; a placeholder's width in a text field free.
- **Each shell, beside `redact.rs`:** every field of its lines reached; a
  framed field keeping its length, with a same-width placeholder in it,
  and with an original of another width that stands; a browser stream's
  record replaced through its pieces; the TUI's decimal elements of a
  pasted address.
- **The tool:** a recording given two builds and two sets of ports
  normalises to the same bytes; each placeholder found in the
  recording's fields, its marks and its prints; a secret still `x`s; the
  proof passing on the redacted recording and the normalised one
  replaying; `add` printing its counts; `normalise`'s table row and help;
  the browser's `again` finding a control and checking a walk across
  placeholders; a page's ids numbered afresh.
- **The committed journeys** replay and print unchanged after their one
  normalisation: the print's test, by name.

## 8. Open questions

- **A fixed clock and seed** for the fake box `again` starts and the
  browser it drives: what would make two walks' recordings the same
  bytes, and their prints that draw a time too, with no placeholder for
  a time (2.4; `web/journeys.md`, 10). The user takes it up as its own
  follow-up.
- **Keeping a recording that still holds:** `again` could keep the
  committed recording when it still replays and the new walk prints the
  same pages, so that a recording changes only when its exchange does.
  Default: `again` adds what it walked.
- **Wire lengths by schema:** a placeholder of another width in a framed
  field, its nested lengths rewritten from the wire's schemas (3.3).
  Default: such an original stands.
- **A browser's build or code:** a browser journey that draws the
  engine's build or approves a code would find it in a framed field, by
  its shape. Default: the browser seeks only ports.
