+++
id = "skein-feed-driver"
part = "skein-json-feed"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-lib/src/stream.rs",
  "skein/crates/skein-lib/src/stream/held.rs",
  "skein/crates/skein-lib/src/tests/held.rs",
  "skein/crates/skein-json/src/tokenizer.rs",
  "skein/crates/skein-json/src/tokenizer/feed.rs",
  "skein/crates/skein-json/src/parse.rs",
  "skein/crates/skein-json/src/tests.rs",
  "skein/crates/skein-json/src/tests/feed.rs",
  "skein/docs/design/lib.status.md",
  "skein/docs/design/json.status.md",
]
effort = "high"
minutes = 60
+++

# skein D001: `stream::Slice` and `tokenizer::Feed`; `parse` and `validate` run on the feed

## Goal

`skein_json::parse` and `tokenizer::validate` are one loop written twice, and each hand-codes
what `stream::Held` does for a box. skein-lib gains `stream::Slice`, `Held`'s borrowed sibling,
and skein-json gains `tokenizer::Feed`, one slice-fed driver of the tokenizer, on which `parse`
and `validate` both run (`skein/docs/design/lib.md`, 7; `skein/docs/design/json.md`, 5.3, both
written by this cut). The call sites move in this plan's later tasks, which need this one. The
item stays open: leave its file.

Acceptance:

- `stream::Slice`, `tokenizer::Feed` and `tokenizer::feed_bound` exist as sketched.
- `rg -n 'Queue|Read::|Down::|Tokenizer::new' skein/crates/skein-json/src/parse.rs` prints
  nothing, and `validate` makes no queue and meets no read itself.
- `Held` answers through the same code as `Slice`; `rg -n 'Read::' skein/crates/skein-lib/src/stream/held.rs`
  finds one `answer`'s match, not two.
- Every test of `skein-lib`, `skein-json`, `skein-json-world` and `skein-llm` passes unchanged,
  each count above zero, and the new tests pass.
- `lib.status.md` and `json.status.md` lose the words this closes.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `skein/AGENTS.md`.
- `skein/docs/development/debt/D001-json-whole-buffer-reads.md`, whole.
- `skein/docs/design/lib.md`, 5.1 and 7 (the contract of a stream, the "complete buffer"
  paragraph); `skein/docs/design/json.md`, 3.1 and 5.3.
- `skein/crates/skein-lib/src/stream/held.rs`; `skein/crates/skein-json/src/parse.rs`;
  `tokenizer.rs`, `validate` and `validation_bound`. `Pump` in
  `skein/crates/skein-forge/src/forgejo/tests/fixtures.rs` is `Feed`'s shape, as a test fixture.

## Owns and may change

- `skein/crates/skein-lib/src/stream/held.rs` (`Slice` beside `Held`); `stream.rs` (the export,
  and its module doc's `Held` line names `Slice` too); `src/tests/held.rs`.
- `skein/crates/skein-json/src/tokenizer/feed.rs` (new); `tokenizer.rs` (`mod feed; pub use
  feed::{Feed, feed_bound};`, `validate`'s body, `validation_bound`'s body); `parse.rs`'s body;
  `src/tests.rs` (one `mod feed;` line); `src/tests/feed.rs` (new).
- `skein/docs/design/lib.status.md`: remove ` 7: `stream::Slice`.`, date it today.
- `skein/docs/design/json.status.md`: remove `5.3: `Feed`, and `parse` and `validate` on it. `,
  date it today.

No call site outside skein-json moves here. `skein_http::sse::Data`, lib's `Held` under another
name, stays.

Pre-approved:

- **`Held` holds no copy of the logic:** the sketch's `Slice::answer` is the one match; `Held`
  answers by a `Slice` over its box, or both by one private function. Yours to choose.
- **A `Feed` given a request after `Done`, `Failed` or `Closed`** gets what the tokenizer does
  with it: `Feed` adds no state of its own beyond the step count.
- **`validate` keeps its answers:** `Done` from `Skip` is `Ok`, as its loop has it; `Failed(e)`
  is `Err(e)`, anything else `Err(Error::Unexpected)`.
- **If a one-entry queue overflows** on a path `parse` never took (`Skip`, `Text`): that is a
  finding against json.md, 5.3. `Pump` in the forgejo fixtures runs `Text` and `Next` on one-entry
  queues, so this is not expected. Stop and ask.

## Steps

1. Sketch:

   ```rust
   // skein_lib::stream
   /// A borrowed buffer, read as a stream: `Held`'s contract over a slice.
   /// It copies only each delivery, never the whole buffer (lib.md, 7).
   #[derive(Debug)]
   pub struct Slice<'a> { /* bytes: &'a [u8], at: usize, ended: bool */ }

   impl<'a> Slice<'a> {
       #[must_use]
       pub fn new(bytes: &'a [u8]) -> Slice<'a>;
       /// As `Held::answer`.
       pub fn answer(&mut self, read: Read) -> Option<Up>;
   }

   // skein_json::tokenizer, tokenizer/feed.rs
   /// One whole borrowed input driven one request at a time (json.md, 5.3).
   #[derive(Debug)]
   pub struct Feed<'a> {
       machine: Tokenizer,
       env: Env<Limits>,
       input: Slice<'a>,
       above: Queue<Event>,   // Queue::with_capacity(1)
       below: Queue<Down>,    // Queue::with_capacity(1)
       ticks: usize,          // input.len() * 4 + 16, saturating, across all answers
   }

   impl<'a> Feed<'a> {
       /// As for `Tokenizer::new`, `limits.chunk` must be positive and
       /// `limits.length` must be less than `u32::MAX`.
       #[must_use]
       pub fn new(input: &'a [u8], limits: &Limits) -> Feed<'a>;

       /// States `request`, then meets each demand from the input until the
       /// tokenizer answers: its one event for the request. `Failed(Unexpected)`
       /// past the step bound, or on a demand the input cannot meet.
       pub fn answer(&mut self, request: Request) -> Event;
   }

   /// `tokenizer_bound` and the feed's two queues (programming-model.md, 6.3).
   #[must_use]
   pub fn feed_bound(limits: &Limits) -> Option<u64>;
   ```

   `answer`'s loop is `Pump::next`'s: a tick; an event in `above` is returned; otherwise `below`
   holds `Down::Demand { read, room: 0 }`, and `input.answer(read)` goes up. A missing demand,
   a `Send`, a `Finish`, a demand with room and a `None` from the slice are all
   `Failed(Error::Unexpected)`, which is what `parse` and `validate` answer for them today.

2. `parse` and `validate`, each a few lines; the length pre-check and its `TooLong` stay first:

   ```rust
   pub fn parse(input: &[u8], limits: &tokenizer::Limits, retained: &document::Limits) -> Result<Document, ParseError> {
       if input.len() > usize::try_from(limits.length).expect("u32 fits usize") {
           return Err(ParseError::Tokenizer(tokenizer::Error::TooLong));
       }
       let mut feed = Feed::new(input, limits);
       let mut document = Builder::new(*retained);
       // Every token reads a byte, so the end comes within one answer more.
       for _answer in 0..=input.len() {
           match feed.answer(Request::Next) {
               Event::Token(token) => document.push(&token).map_err(ParseError::Document)?,
               Event::Done => return Ok(document.into_document()),
               Event::Failed(error) => return Err(ParseError::Tokenizer(error)),
               Event::Skipped(_) | Event::Long(_) | Event::Closed => {
                   return Err(ParseError::Tokenizer(tokenizer::Error::Unexpected));
               }
           }
       }
       Err(ParseError::Tokenizer(tokenizer::Error::Unexpected))
   }
   ```

   `validate` runs `Feed::new(input, &bounded)`, `answer(Request::Skip)`, then
   `answer(Request::Next)`. `validation_bound` reads `feed_bound(&bounded)`.

3. Tests.
   - `skein-lib/src/tests/held.rs`: `Slice` answers every row `Held` answers there, the two
     compared over one table of reads; a `Slice` leaves its buffer untouched and reads it once.
   - `skein-json/src/tests/feed.rs`:
     - one `Next` per token, then `Done`, over a document with whitespace, escapes and nesting;
     - `Text(2)` over a longer string answers `Long(len)`, and over a short one its token;
     - `Skip` answers `Skipped(n)`, then `Next` answers `Done`;
     - `Close` mid-document answers `Closed`; a feed dropped mid-document is fine;
     - cut-short, two-value and too-deep inputs answer the `Failed` the tokenizer gives;
     - the same document chopped by a hand-fed tokenizer (`super`'s test harness) and by `Feed`
       give the same events: the driver adds nothing;
     - `feed_bound` is `tokenizer_bound` plus two one-entry queues, and `None` where it is.

4. Narrow tests, one command at a time, each count above zero:
   - `heavy cargo nextest run -p skein-lib --lib -E 'test(held)'`
   - `heavy cargo nextest run -p skein-json`
   - `heavy cargo nextest run -p skein-json-world`
   - `heavy cargo nextest run -p skein-llm`

   Departure from the implementer's text: `skein-llm` stands for skein-json's direct dependents,
   since it reads both `validate` and `parse` and no answer of either changes. Public items are
   only added, so no other tree is checked.
5. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`; `heavy cargo clippy -p
   skein-lib -p skein-json --all-targets -- -D warnings`.
6. Both status files; `python3 -I -B tools/citations/citations.py --base origin/main` and
   `--all`.
7. Two commits ("skein-json gets one slice-fed driver, and parse and validate run on it (json.md,
   5.3)"; "lib and json status: Slice and Feed are built"), pushed.

## Report

As the implementer's text says, and also: the lines `parse.rs` and `validate` lost; whether
`Held` delegates to `Slice` or both to a private function.
