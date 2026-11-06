+++
id = "redact"
part = "redact"
kind = "feature"
needs = ["stream-fields", "originals"]
files = ["temper/crates/temper-client-shell-web/src/journey/redact.rs", "temper/crates/temper-client-shell-web/src/journey/redact/**"]
effort = "high"
minutes = 60
+++

# The originals out of a recording, each in every place

## Goal

`journey::redact`: every original replaced, in every field and in a stream's joined records, each frame keeping its size. It proves nothing and writes nothing: pure bytes to bytes.

Acceptance:

- `redact` as `notes/contracts.md` gives it. Each rule of `temper/docs/design/client/journeys.md`, 3.6, has its test.
- **For each kind and each place, a failing test first:** the table below runs against a `redact` that returns its input, every row seen failing, before the code that passes it. Your report says so, with the count.
- A framed field keeps its length; a `heard piece` keeps its length and its place. Bytes beside an original stand.
- Each committed journey's recording redacts: no error, and its `secret` value, if it has one, is gone from the output.

## Inputs

- `temper/docs/design/client/journeys.md`, 3.2, 3.3 and 3.6.
- `temper/docs/plan/real-journeys/notes/contracts.md`.
- `temper/docs/plan/real-journeys/notes/common.md`.

## Owns and may change

Owns the files in the header: a new module. Tests in `journey/redact/tests.rs`.

Neighbour files, these lines only:

- `temper/crates/temper-client-shell-web/src/journey.rs`: `mod redact;` before `mod replay;`, and `pub use redact::{Redacted, Unredacted, redact};` before the `pub use replay` lines.

Options, each pre-approved:

- A helper to replace every occurrence of one form in a field: yours, in this module. `skein_lib::bytes::find_from` is there.
- A `Line` that fails its own reading after replacement can't happen: `Line::write` escapes every byte. If you find otherwise, stop and ask.

## Steps

The order, inside `redact`: `lines(recording)`; `Originals::of`; each `typed` act's value becomes `placeholder(value)`; each field by `fields_mut`, each sought original longest first, each of its forms and digests; `Streams::read`, each event's record and each plain run the same way, as `Framed`; `Streams::write`; every line written.

- **A form's size:** `Framed`, `x`s of the form's own length. `Text`, the placeholder, the original's length (3.6). A digest becomes the placeholder's digest, in any field.
- **Count** each replacement in `places`.

The test table, in `journey/redact/tests.rs`, lines built by hand (`lines` reads them without the start's facts), each kind's value fake:

```rust
// Kinds: link (`secret`), stored (a `Kept` under `stream`, 16 bytes), the same under a `save`, typed, given.
// Places, each a helper taking the original's bytes and giving the lines that hold it:
//   a framed field     `exchange` body, length-prefixed
//   a framed field     `heard answered` body, length-prefixed
//   a text field       `act` event, `act` name, a header's value
//   a piece's base64   one `data:` event, the original inside its record
//   a split            the same event, in two pieces, at every byte of the line
//   a URL's query      `path`, `place`, `navigate`: its bytes, and percent-encoded where it has `/ + =`
//   a mark's html      `mark`, `end`: its `dom::print` escapes
#[test] fn each_kind_goes_from_each_place()
#[test] fn a_framed_field_keeps_its_length_and_its_neighbours()
#[test] fn a_stored_value_keeps_its_kept_head() // the output's value decodes as a `Kept`, its record 16 `x`s
#[test] fn a_percent_form_in_a_text_field_becomes_the_placeholder()
#[test] fn a_percent_form_in_a_framed_field_keeps_its_length()
#[test] fn a_digest_becomes_the_placeholders()  // in an `exchange` body: of the bytes, and of the value whole with its newline
#[test] fn a_typing_runs_heads_stay_in_their_acts() // `s`, `sk`, ... : each act's value `x`s, its value everywhere else, an `s` elsewhere standing
#[test] fn a_pasted_values_whitespace_stands()  // b"abcd\n" in the act; `abcd` in a body; `xxxx\n` and `xxxx`
#[test] fn longest_first()                      // `abcdef` and `abcd` given, both present
#[test] fn counts_each_kind_and_its_places()
#[test] fn an_undecodable_data_line_refuses()
#[test] fn the_committed_recordings_redact()    // for each: Ok; a `secret` line's value gone; time reported
```

Then fmt, clippy on `temper-client-shell-web`, its tests, commit and push.

## Report

As the implementer's text says, with the table's count and how each row failed first.

## Rebase (2026-11-06, step merge, foreman real-journeys/redact/foreman-1)

resolve journey.rs by retaining both redaction exports and prove_drawn
The merge conflicts in `temper/crates/temper-client-shell-web/src/journey.rs`.
Integration commit `2a48c5c00c7dc733646ec0194f8c59b2a8fe9f62` adds `pub use replay::prove_drawn;` beside this branch’s redaction export.
Rebase onto the landing’s current integration tip and retain all three adjacent exports:
```rust
pub use redact::{Redacted, Unredacted, redact};
pub use replay::prove_drawn;
pub use replay::{Mark, Refusal, Replay, Stale, prove};
```
Keep `mod redact;` before `mod replay;` and preserve the other module declarations and exports.
This is an adjacent insertion conflict; no redaction algorithm change is needed.
Run formatting and the focused `temper-client-shell-web` journey tests through heavy with `TEMPER_BUNDLE=none`.
