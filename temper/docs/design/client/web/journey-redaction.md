# A journey's redaction

`temper-client-journeys print FILE OUT` redacts before its replay's proof (`journeys-recordings.md`, 2.3). It reads the
recording as its lines (`journeys-recordings.md`, 1.2), and every byte field of each. It writes
nothing and prints no original. A browser's saved file already holds
the *link* and the *stored* replaced (`journeys-recordings.md`, 1.1): their values are `x`s
alone, which are no original, so `print` counts none of either. It
scans the redacted records, proves them against the redacted saved
marks while scanning the pages drawn at each turn's end, then prints
and scans the documents before writing into OUT. Nothing is put in
after the proof to replace varying facts; no recording is written and
OUT is outside both repository journeys roots (`journeys.md`, 5.1).

- **The originals:**
  - *link:* the `secret` record's value;
  - *stored:* each `stored` record's value, and each `save` of the
    stream's record (jig's `STREAM_KEY`), the same credential as it is
    issued. Such a value is a `Kept`: its head stands, so that the host
    still reads it, and its `record` is the original. A value that isn't
    one is an original whole;
  - *typed:* each value of an act whose event shows a secret's length
    (`Secret { len: N }`, `journeys-recordings.md`, 1.3), such as a typed key or a pasted sign-in;
  - *given:* each `--redact TEXT`, supplied by the person for any other
    text they want left out; no capture path is supplied implicitly;
  - *approval*, *credential* and *bearer:* the TUI's, the approval's
    secret, the credential the TUI takes and the bearer every exchange
    carries (`../tui/journeys.md`, section 5), which a browser's recording
    never gives. They are sought as every original is, and reach the
    shared code from the TUI's lines through `Originals::new` and
    `redact_field`.
- **The replacement** is `x`s, one for each byte, so that every frame,
  length prefix and base64 run keeps its size. Surrounding spaces, tabs,
  CRs and LFs of a typed or given original stand, outside it: the client
  trims them from a sign-in's paste before it carries the value, and a
  key's field carries them with it, including a pasted trailing newline.
  Both whole-value and trimmed-value digests are replaced.
- **Where:** every original is sought in every field, longest first, in
  these forms: its bytes; percent-encoded, either hex case, for a URL's
  query; `dom::print`'s escapes, for a mark's html; and Rust's Debug
  escapes, for an event's print. Its sha-256, with those spaces and
  without them, is sought as well and becomes the replacement's:
  `AddKey`, `ReplaceKey` and `FinishSignIn` carry the digest of the value
  they carry.
- **A form's size:** in a framed field (a `heard answered` body, a
  stream's record, an `exchange` body, a `save` or `stored` key or
  value, and `written`, `unwritten` and `erase`) a form becomes `x`s of
  its own length. In any other field, which no length prefix counts, it
  becomes the replacement.
- **Typing:** the recorder keeps each input act (`journeys-recordings.md`, 1.3), so a typed value
  shorter than 4 bytes, or the head of a longer typed value, is replaced
  in its own act alone, and sought nowhere else. The view draws a secret
  field's value empty, so no mark holds it.
- **A stream's pieces are joined first.** A `heard piece` is a fetch's
  chunk: it may end in the middle of a `data:` line, an original or a
  base64 group. Between an `open` or `heard opened` and the next, the
  pieces are one stream of bytes, cut into its lines; each `data:` line's
  canonical base64 is decoded to its record, replaced there, encoded
  back, and the bytes cut where the pieces ended, so that the
  replay is given the chunks it was given. A `data:` line that doesn't
  decode refuses the recording, by the line of the piece it starts on. A
  last line that the recording's end cuts short is decoded by its whole
  groups, the rest kept as text.
- **The rest is the proof's:** the host derives what it asks from what
  it heard, so a consistent replacement replays as the original did. An
  ask is proved, then: what is left of a typed value in an exchange
  differs from what the replay asks and is refused (`journeys-recordings.md`, 2.2). A report is
  only replaced and scanned (`journeys-recordings.md`, 1.7).
- **What `print` reports:** the count of each kind of original, and of the
  places replaced.
- **A real deployment serves its own bundle,** enabled for recording
  by the page setting or `--record` (`journeys-recordings.md`, 1.5).
- **Email addresses** are neither replaced nor refused unless a
  `--redact` names them: they may stand in a person's printed pages.
  Read the pages before sharing; the last scan's limits are
  `journeys-recordings.md`, 1.8.
