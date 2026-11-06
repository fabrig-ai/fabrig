+++
id = "scan"
part = "scan"
kind = "feature"
needs = ["stream-fields", "originals"]
files = ["temper/crates/temper-client-shell-web/src/journey/scan.rs", "temper/crates/temper-client-shell-web/src/journey/scan/**", "temper/crates/temper-client-shell-web/src/journey/line.rs"]
effort = "high"
minutes = 70
+++

# The last scan

## Goal

`journey::scan`: what must never be committed, found wherever it can be seen, and named by its record and kind, never its bytes. This is where a token split across deliveries is caught: `RunText` and `WritingCall` carry a model's stream in pieces no one line shows, so each is joined by its offsets. A hit there refuses; it isn't replaced.

Acceptance:

- `scan` as `notes/contracts.md` gives it. Each shape and rule of `temper/docs/design/client/journeys.md`, 3.7, has its test.
- **For each shape and each place, a failing test first:** the table below runs against a scan that finds nothing, every row seen failing, before the code that passes it.
- Every committed journey's lines (with no originals, `Originals::of(&[], &[])`) and each of its printed pages pass.
- `Found` and `What` hold no bytes.

## Inputs

- `temper/docs/design/client/journeys.md`, 3.7 and 3.8.
- `jig/docs/design/protocol/client.md`, 7, *A run's text*.
- `temper/docs/plan/real-journeys/notes/contracts.md`.
- `temper/docs/plan/real-journeys/notes/common.md`.

## Owns and may change

Owns the new module's files. Tests in `journey/scan/tests.rs`.

In `journey/line.rs`, `Line::kind` only: `pub`, returning `&'static str`, and its callers in that file by `.as_bytes()`.

Neighbour files, these lines only:

- `temper/crates/temper-client-shell-web/src/journey.rs`: `pub mod scan;` before `mod stream;`, and `pub use scan::{Found, Place, Shape, What};` before `pub use stream::..`.

Options, each pre-approved:

- **The join's key** is the watch, the attempt and the turn (`RunText::turn`, compared as its encoded bytes if it has no `Eq`), and for a writing call also its `id`. Deliveries are segments: sort by `from`, overlapping or touching ones join, a gap starts a new segment. Never allocate by `through` alone: it is the recording's word. A delivery whose `through - from` isn't its text's length is scanned alone, as the record it already was.
- A record that doesn't decode as a `StreamEvent` under `jig_client_protocol_wire::CEILINGS` joins nothing: its bytes have been scanned as a record.
- The first hit returns. Scanning order: fields in line order, then streams, then joins.
- **A record's other text:** only `RunText::text` and `WritingCall::input` join. If you find another field of a stream event that a model's stream splits, stop and ask.

## Steps

**The shapes,** exactly as 3.7 gives them, each a small function over bytes. A `x`-only value passes. The key's left rule is two bytes: `sk-` after two bytes of `[A-Za-z0-9_-]` doesn't match, because one such byte may be a length prefix.

**Originals:** each `forms()` and each digest's first half.

**`page`:** reads back `&lt;`, `&gt;`, `&quot;`, `&amp;`, then `bytes`.

The table, in `journey/scan/tests.rs`, lines built by hand; each secret built from pieces (`notes/common.md`):

```rust
// Shapes: key, JWT, bearer, code, refresh token (JSON-spelled and `=`-spelled), and an original.
// Places:
//   a text field       `act` name, a header's value
//   a framed field     `heard answered` body, behind a length prefix
//   a record           one `data:` event
//   a split piece      the same event in two pieces, at every byte of the line
//   a URL's query      `path`, `place`
//   a page             `page`, escaped as `dom::print` escapes
#[test] fn each_shape_is_found_in_each_place()
#[test] fn a_run_texts_split_token_is_found()      // each shape and an original, in two `StreamText`s, cut at every byte; Place::RunText; each delivery alone holds no hit at the middle cut
#[test] fn a_writing_calls_split_input_is_found()  // same, `WritingCall::input`
#[test] fn a_gap_joins_nothing()                   // offsets that don't meet: no false join
#[test] fn every_length_prefix_before_a_key()      // [0, 0, 0, b] then a key, for each b: found
#[test] fn a_51_byte_keys_own_prefix()             // [0, 0, 0, 51]
#[test] fn these_pass()                            // task-pages-and-more-than-twenty-bytes, temper-task-pages-a-long-name, risk-a-long-hyphenated-word-here,
                                                   // code_challenge=abcdefghijkl, status_code=200, Bearer tokens, sk- short, eyJ alone,
                                                   // refresh_token is, and each shape's value of `x`s alone
#[test] fn these_are_found_at_their_edges()        // exactly 20, 8, 16 long; one fewer passes; error_code= and device_code= are found; any case of Bearer
#[test] fn found_names_the_line_and_the_record()   // line number from 1, record word, place
#[test] fn found_holds_no_bytes()                  // Debug of a Found holds none of the secret
#[test] fn the_committed_journeys_pass()           // lines and every printed page; time reported
```

Build `StreamText`, `RunText` and `WritingCall` with `jig_client_protocol_wire` as `tests/suite/host/walk.rs` builds its events, and a `data:` line as its `Walk::streamed` does.

Then fmt, clippy on `temper-client-shell-web`, its tests, commit and push.

## Report

As the implementer's text says, with how each row failed first. **Say what the scan sees of a token split across pieces and across deliveries, and what it doesn't: it goes in this run's report to the user.**
