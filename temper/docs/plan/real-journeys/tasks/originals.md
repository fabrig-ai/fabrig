+++
id = "originals"
part = "originals"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-web/src/journey/originals.rs", "temper/crates/temper-client-shell-web/src/journey/originals/**"]
effort = "high"
minutes = 50
+++

# What is taken out of a recording

## Goal

The one list of originals: read from a recording's lines and the given texts, each with its forms and digests. It decides nothing about where they are replaced. Three things the client does shape it: a `stored` value is a `Kept` the host must still read; a typed key arrives an act at a time; and `AddKey`, `ReplaceKey` and `FinishSignIn` carry the value's sha-256, of the trimmed value for a paste.

Acceptance:

- `originals.rs` as `notes/contracts.md` gives it, each rule of `temper/docs/design/client/journeys.md`, 3.6's *The originals*, *The placeholder*, *Where* and *Typing* its own test.
- `Original`'s Debug prints its kind and length alone.

## Inputs

- `temper/docs/design/client/journeys.md`, 3.3 and 3.6.
- `temper/docs/plan/real-journeys/notes/contracts.md`.
- `temper/docs/plan/real-journeys/notes/common.md`.
- `temper/crates/temper-client-domain/src/pages/accounts/paste.rs`, about lines 60-78, the trim; `pages/accounts/request.rs`, about line 150, the digest; `jig/crates/jig-client-protocol-link/src/store.rs`, about lines 168-215, `Kept` under `STREAM_KEY`.

## Owns and may change

Owns the files in the header: a new module. Tests in `journey/originals/tests.rs`.

Neighbour files, these lines only:

- `temper/crates/temper-client-shell-web/src/journey.rs`: `mod originals;` after `mod line;`, and `pub use originals::{Kind, Original, Originals, SECRET_EVENT, kept_head, placeholder, typed};` after `pub use line::..`.
- `temper/crates/temper-client-shell-web/src/journey/line.rs`: nothing: `Line`'s variants are public.

Options, each pre-approved:

- `dom::print`'s escapes: take them from skein if a public function gives them; else spell the four (`<`, `>`, `"`, `&`) here with a line citing journeys.md, 5.3.
- Rust's Debug escapes: `format!("{:?}", text)` without its quotes for UTF-8 text; a value that isn't UTF-8 has no such form.
- jig's `STREAM_KEY` is `jig_client::STREAM_KEY`, as `records.rs` reads it.

## Steps

```rust
#[test] fn the_links_value_is_an_original()
#[test] fn a_stored_values_kept_record_is_the_original_and_its_head_stands()
#[test] fn a_stored_value_that_isnt_kept_is_an_original_whole()
#[test] fn a_save_of_the_stream_is_an_original()        // and a save of a request's 16-byte key is none
#[test] fn a_typed_value_needs_the_secret_in_its_event() // same act, plain event: none
#[test] fn a_typing_runs_heads_are_not_sought()          // each act of a value typed a byte at a time: one sought, counted once
#[test] fn a_short_typed_value_is_not_sought()
#[test] fn a_given_text_is_an_original()
#[test] fn surrounding_space_stands_outside_the_original() // the four bytes, both ends; placeholder(b" ab\n") == b" xx\n"
#[test] fn an_empty_or_x_original_is_none()
#[test] fn the_forms_are_each_once_bytes_first()         // a plain token has one; a text with `/ & "` has them all
#[test] fn the_digests_are_of_the_bytes_and_the_value()  // skein_lib::sha256::digest
#[test] fn longest_first_each_once()                     // the same bytes as link and given: one, with its first kind
#[test] fn debug_prints_no_bytes()
```

Build the `Kept` of a test with `jig_client_protocol_wire::Kept::new` and `w::KeptParts`, as `store.rs` does. Then fmt, clippy on `temper-client-shell-web`, its tests, commit and push.

## Report

As the implementer's text says.
