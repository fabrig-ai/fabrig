+++
id = "skein-fake-forge-pumps"
part = "skein-json-feed"
kind = "feature"
needs = ["skein-feed-driver"]
files = [
  "skein/crates/skein-fake-forge-github/src/app.rs",
  "skein/crates/skein-fake-forge-github/src/manifest.rs",
  "skein/crates/skein-fake-forge-github/src/git.rs",
  "skein/crates/skein-fake-forge-github/src/tests/fixtures.rs",
  "skein/crates/skein-fake-forge-protocol/src/observations/read.rs",
  "skein/crates/skein-fake-forge-protocol/src/tests/fixtures.rs",
  "skein/crates/skein-fake-forge-protocol/tests/suite/memory.rs",
]
effort = "high"
minutes = 75
+++

# skein D001: the fake forges' whole-buffer reads take `Feed` and `parse`

## Goal

`skein-fake-forge-github` and `skein-fake-forge-protocol` drive a tokenizer over a whole buffer by
hand in seven places. Each takes `tokenizer::Feed` or `skein_json::parse`
(`skein/docs/design/json.md`, 5.3), with the refusals it gives today. Their streaming readers
stay. The item stays open: leave its file.

Acceptance:

- `rg -c 'Tokenizer::new\(' skein/crates/skein-fake-forge-github` prints nothing (4 at base).
- `rg -c 'Tokenizer::new\(' skein/crates/skein-fake-forge-protocol` lists only `src/server.rs:2`,
  `src/server/git.rs:1` and `src/tests/git.rs:1` (7 at base): the first two read an HTTP body as it
  arrives; the third feeds a body in pieces.
- `rg -n 'Held' skein/crates/skein-fake-forge-github/src skein/crates/skein-fake-forge-protocol/src/observations`
  prints nothing.
- Every test the Steps run passes unchanged but for `memory.rs`'s lines, each count above zero.
  `tests/suite/memory.rs`'s bound still holds.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `skein/AGENTS.md`.
- `skein/docs/design/json.md`, 5.2 and 5.3; `skein/crates/skein-json/src/tokenizer/feed.rs` on the
  trunk.
- `skein/docs/development/debt/D001-json-whole-buffer-reads.md`.

## Owns and may change

Only the functions below, and each file's imports:

| Site | Today | Takes |
|---|---|---|
| `skein-fake-forge-github/src/app.rs`, `read` and `claims_read` | hand `Fill`/`Scan`, collects every token | `skein_json::parse`, a `Document` walk |
| `skein-fake-forge-github/src/manifest.rs`, `tokenize` | `Intake`, collects every token | `Feed`; its walkers stay |
| `skein-fake-forge-github/src/git.rs`, `decode` | `Held` over a copy, steps a decoder | `Feed` |
| `skein-fake-forge-github/src/tests/fixtures.rs`, `decode_with` | `Intake` | `Feed` |
| `skein-fake-forge-protocol/src/observations/read.rs`, `tokenize` | `Held` over a copy, tracks the refusal's member | `Feed` |
| `skein-fake-forge-protocol/src/tests/fixtures.rs`, `Pump` | `Intake` | `Feed`; `Pump` goes |
| `skein-fake-forge-protocol/tests/suite/memory.rs`, the client's decode | `Held` | `Feed` |

Stays: `skein-fake-forge-protocol/src/server.rs`, `src/server/git.rs`, `src/tests/git.rs`.

Pre-approved:

- **`manifest::tokenize` takes `Feed`, not `parse`:** it keeps a `List<Token>` for three walkers
  (`manifest.rs`, `mutations.rs`, `comment_edit.rs`; 112 `Token::` lines), and `manifest_bound`
  prices that list. Those stay as they are.
- **`observations`' `tokenize` keeps `variant`, `member` and its `Reason`s:** a `Failed` answer is
  `refuse(&variant, &member, Reason::Json)` with what it had read by then; a full token list is
  `Reason::Limit`, and so is the end of its loop. `parse` would lose those, so it must not take it.
- **`claims_read`'s limits stay:** every `tokenizer::Limits` field `JWT_BYTES`, and
  `document::Limits { tokens: bound, text: bound }`; every refusal `Refused::Malformed`. A `Long`
  record is `Malformed` (parse keeps none).
- A fixture keeps its signature and its panics on a refusal; where `Pump`'s callers hold it, they
  hold a `Feed<'_>`.

## Steps

1. A stepped decoder, here `git.rs`:

   ```rust
   let mut decoder = git::server::decoder(name, limits);
   let mut feed = json::Feed::new(body, &limits.documents.json);
   let mut request = git::server::body_first(&decoder);
   // An answer but the last reads at least a byte.
   for _answer in 0..=body.len() {
       match git::server::body(&mut decoder, limits, &feed.answer(request)) {
           git::server::BodyStep::Demand(next) => request = next,
           git::server::BodyStep::Decoded(call) => return Some(call),
           git::server::BodyStep::Malformed => return None,
       }
   }
   None
   ```

2. A collection, here `manifest::tokenize` (its `u32::try_from` stays, as does its capacity):

   ```rust
   let mut feed = tokenizer::Feed::new(input, &limits.json);
   let mut tokens = List::with_capacity(bound);
   for _answer in 0..=input.len() {
       match feed.answer(tokenizer::Request::Next) {
           tokenizer::Event::Token(token) => tokens.push(token).ok()?,
           tokenizer::Event::Done => return Some(tokens),
           tokenizer::Event::Failed(_)
           | tokenizer::Event::Long(_)
           | tokenizer::Event::Skipped(_)
           | tokenizer::Event::Closed => return None,
       }
   }
   None
   ```

3. `claims_read`: `read` becomes a call of `skein_json::parse` with those limits, its error
   `Refused::Malformed`. The walk reads each record with `document.token(at)` for `at` in
   `0..document.len()`, and matches `record.kind`; `Kind::Number | Kind::String` reads
   `document.text(record)`. The order of its checks stays: a repeated claim stays refused, which
   this walk does itself. `memory.rs`'s bound reads `json::feed_bound(..)` in place of its
   `tokenizer_bound` and two `Queue::price` lines.

4. One command at a time, each count above zero:
   - `heavy cargo nextest run -p skein-fake-forge-github`
   - `heavy cargo nextest run -p skein-fake-forge-protocol`
   - `heavy cargo nextest run -p skein-fake-peers-tests`, which runs both fakes end to end.

5. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`; `heavy cargo clippy -p
   skein-fake-forge-github -p skein-fake-forge-protocol --all-targets -- -D warnings`.
6. One commit per crate ("the fake forges read whole JSON buffers through Feed and parse (json.md,
   5.3)"), pushed.

## Report

As the implementer's text says, and also: what `rg -c 'Tokenizer::new\('` reads for each crate;
any site that gave a different refusal once moved.
