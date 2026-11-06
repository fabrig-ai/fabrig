+++
id = "stream-fields"
part = "pieces"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-web/src/journey/stream.rs", "temper/crates/temper-client-shell-web/src/journey/stream/**", "temper/crates/temper-client-shell-web/src/journey/fields.rs", "temper/crates/temper-client-shell-web/src/journey/fields/**"]
effort = "high"
minutes = 50
+++

# A recording's streams joined, and every field of a line in one list

## Goal

The two readers the redaction and the scan share. A `heard piece` is a fetch's chunk, so a `data:` line, and an original inside it, may be split across pieces: `chat-draft`'s recording already holds such pieces. `Streams` joins them and decodes each record; writing back gives the same chunks. `fields_mut` lists every byte field of a line, with no `_`, so that a new record kind can't slip past the guard.

Acceptance:

- `Streams` and `fields_mut` as `notes/contracts.md` gives them.
- A recording read and written back unchanged is byte-identical, for each committed journey's `recording`.
- Every split of an event across two pieces, at each byte, reads the same record. A record changed in place, same length, writes back into pieces of the same lengths, and they decode to the changed record.
- A `data:` line that doesn't decode is `Undecoded`, by the piece it starts on.

## Inputs

- `temper/docs/design/client/journeys.md`, 3.2 and 3.6, *A stream's pieces are joined first*.
- `temper/docs/plan/real-journeys/notes/contracts.md`.
- `temper/docs/plan/real-journeys/notes/common.md`.
- `jig/crates/jig-client-protocol-http/src/stream.rs`, how the client reads the same bytes.

## Owns and may change

Owns the files in the header: new modules. Tests in `journey/stream/tests.rs` and `journey/fields/tests.rs`, each behind `#[cfg(test)] mod tests;`.

Neighbour files, these lines only:

- `temper/crates/temper-client-shell-web/src/journey.rs`: `mod fields;` before `mod line;`, `mod stream;` before `mod walk;`, `pub use fields::{Framing, fields_mut};` before `pub use line::..`, and `pub use stream::{Event, Plain, Streams, Undecoded};` before `pub use walk::..`. `crate::journey::Event` is free: the module has none; if it isn't, name yours `StreamEvent` and say so.

Options, each pre-approved:

- A stream's bytes that are neither `data: ` nor a blank line nor a comment (a refused stream's body) are `Plain`, whole, cut at `\n` or not as is simplest.
- If `skein_lib::base64` refuses what the client's incremental `Decoder` takes, use `Decoder`, and say so.

## Steps

```rust
#[test] fn an_event_split_at_each_byte_reads_the_same()      // one record, every cut in 1..len, two pieces
#[test] fn a_changed_record_keeps_its_pieces_lengths()       // every cut again; write; pieces decode to the change
#[test] fn three_events_and_a_comment_in_one_piece()         // Plain holds the comment
#[test] fn an_open_starts_a_new_stream()                     // a cut line never joins the next stream's first
#[test] fn a_bad_data_line_is_undecoded_by_its_first_piece()
#[test] fn a_last_line_cut_short_decodes_its_whole_groups()  // the rest is Plain
#[test] fn the_committed_recordings_read_and_write_unchanged()
#[test] fn every_field_is_listed()                           // a row per Line and Heard kind: each listed field replaced, the written line holds the marker nowhere
#[test] fn framing_is_the_contracts()                        // a row for each Framed kind and each Text kind
```

Then fmt, clippy on `temper-client-shell-web`, its tests, commit and push.

`Line` is in `journey/line.rs`, with an exhaustive `Clone` to read as the model. Don't change it.

## Report

As the implementer's text says. Say how many pieces of the committed recordings split a `data:` line: it goes in the design's reasons if a re-recording ever makes it none.
