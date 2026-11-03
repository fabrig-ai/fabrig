+++
id = "ui-web-log"
part = "ui-web"
kind = "feature"
needs = ["ui-web-surface"]
files = ["skein/crates/skein-ui-web/src/log.rs", "skein/crates/skein-ui-web/src/log/**", "skein/crates/skein-ui-web/src/tests/log.rs", "skein/crates/skein-ui-web/src/tests/log/**"]
effort = "high"
minutes = 120
+++

# skein-ui-web: the session log, its text and its keeping

## Goal

The session log the page keeps while a visit runs, its text written and read back, and the
fold of what the world said between two inputs, which the page and a native replay share
(`skein/docs/design/playground.md`, 6 and 6.1). Portable, in `skein-ui-web`, so the page's
two platforms keep it alike.

Acceptance:

- `skein_ui_web::log` holds exactly `notes/contracts.md`, section 4: `Said`, `Saying`, `Entry`,
  `End`, `Read`, `Refused`, `read`, and `Log` with `new`, `sent`, `heard`, `text`, with the
  rules beneath it there.
- **The text** is playground.md, 6.1, exactly: the first line, each input's line with its
  fields in the order given, the `end` or `cut` line; numbers decimal with no leading zero;
  bytes in base64, the standard alphabet with padding (`skein_lib::base64`); digests as 64
  lowercase hex digits; every line ending in `\n`.
- **`Saying`** is 6.1's digest: SHA-256 (`skein_lib::sha256`) over the 65 bytes of the
  SHA-256 of the reports' lines, the SHA-256 of the printed lines' lines, and the stop byte;
  each report's and each line's text as 6.1 gives it, each ending in `\n`. A fresh `Saying`
  gives the digest of nothing said.
- **`Log`**: `sent` keeps an input until its stamp; `heard` pairs a `Stamp` with the oldest
  input waiting (none waiting: a panic, a broken contract), closing the fold since the input
  before into that entry's `said`; folds `Report`, `Printed` and `Stopped`; marks the end at
  each `Clock` (contracts, 4: `End::At` is the last clock's step and instant and the fold of
  what came after the last stamp and before that clock); ignores `Failed`. Past `bound` the
  log is cut: the input whose line would pass it (with the last line's room) and every one
  after is not kept, and the end is `Cut`.
- **`read`** refuses with `Refused::Format` a first line of another format, with
  `Refused::Version { found }` another version (named as the log writes it), and with
  `Refused::Line { line }` any other line that is not the format's, its number from 1: a
  field out of order or missing, a number with a leading zero or past its type, bad base64, a
  digest not 64 lowercase hex digits, a line after the last, no last line. It accepts what
  `text` writes and nothing looser.
- Step tests in `src/tests/log.rs` pass:
  - `a_log_reads_back_whole`: a log kept over every input kind (an exchange with headers and
    an empty body, an abandon, the stream opened with headers, closed, a read-on), reports of
    every kind, lines of both streams, a stop, clocks between: `read(&log.text())` gives each
    entry, its stamp, its digest, the seed, the start, the build, and `End::At`;
  - `a_native_log_names_no_build`: `build` `None` writes `-` and reads back `None`;
  - `the_digest_folds_each_stream_apart`: reports and lines interleaved two ways give one
    digest; two reports swapped give another; a stop changes it;
  - `the_digest_is_the_designs`: a golden: one report and one line folded give the digest
    computed by hand from 6.1's texts in the test (the hex written in the test);
  - `the_text_is_the_designs`: a golden of a small log's whole text, written in the test;
  - `what_comes_after_the_last_clock_is_not_in_the_end`;
  - `a_log_past_its_bound_is_cut` and reads back with `End::Cut` and the inputs before;
  - `another_format_or_version_is_refused`, naming the version;
  - `a_malformed_line_is_refused_naming_it`: one case per rule above;
  - `a_stamp_with_no_input_is_a_broken_contract` (`#[should_panic]`, with the expected
    message).
- `heavy cargo nextest run -p skein-ui-web --lib -E 'test(/log::/)'` passes, then the crate's
  lib tests, their counts in the report; clippy native and for wasm32 clean.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1 and 4.
- `skein/docs/design/playground.md`, 5 ("A slice's posts, in order"), 6 and 6.1;
  `skein/docs/design/lib.md`, 6.1 (base64).

## Owns and may change

- Owns the files in the header; `src/log.rs` exists with its module comment (task
  `ui-web-surface`), and `src/tests.rs` declares `mod log;` already.
- Read only: `bridge.rs`, `fetch.rs` (their types as they stand; a change is a question).
- Pre-approved: private submodules under `src/log/` (the text's writer and reader, the fold)
  as you see fit; `Vec` and `String` are allowed in this crate.

## Steps

1. `Saying` and its text forms; then `Log` and `text`; then `read`.
2. The step tests, the goldens computed by hand once and written in the tests.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`;
   `heavy cargo clippy -p skein-ui-web --all-targets -- -D warnings` and its wasm32 run; the
   citations check.
4. Commit (why: the page keeps a visit's log, which replays natively); push your branch.

## Report

As the implementer's text says, and also: a small log's text as your golden has it.
