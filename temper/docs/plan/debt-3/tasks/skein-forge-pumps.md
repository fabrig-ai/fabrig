+++
id = "skein-forge-pumps"
part = "skein-json-feed"
kind = "feature"
needs = ["skein-feed-driver"]
files = [
  "skein/crates/skein-forge/src/github/server/app.rs",
  "skein/crates/skein-forge/src/github/server/writes.rs",
  "skein/crates/skein-forge/src/github/server/queries.rs",
  "skein/crates/skein-forge/src/github/decode/delivery.rs",
  "skein/crates/skein-forge/src/github/tests/fixtures.rs",
  "skein/crates/skein-forge/src/forgejo/tests/fixtures.rs",
  "skein/crates/skein-forge/tests/suite/github_start_plans.rs",
]
effort = "high"
minutes = 90
+++

# skein D001: skein-forge's whole-buffer reads take `Feed` and `parse`

## Goal

skein-forge drives a tokenizer over a whole buffer by hand in ten places. Each takes
`tokenizer::Feed` or `skein_json::parse` (`skein/docs/design/json.md`, 5.3), with the refusals it
gives today. The item stays open: leave its file.

Acceptance:

- `rg -c 'Tokenizer::new\(' skein/crates/skein-forge` lists only `tests/common/forgejo.rs:1`
  (11 at base, in 8 files): `drive_parts` feeds a body in pieces and counts its steps for a fuzz
  bound, and stays.
- `rg -n 'Intake|intake' skein/crates/skein-forge/src/github/server skein/crates/skein-forge/src/github/decode/delivery.rs`
  prints nothing.
- Every test the Steps run passes unchanged, each count above zero. No public signature changes.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `skein/AGENTS.md`.
- `skein/docs/design/json.md`, 5.2 and 5.3; `skein/crates/skein-json/src/tokenizer/feed.rs` on the
  trunk.
- `skein/docs/development/debt/D001-json-whole-buffer-reads.md`.

## Owns and may change

Only the pump functions below, the walker in `queries.rs`, and each file's imports:

| Site | Today | Takes |
|---|---|---|
| `src/github/server/app.rs`, `hook_url` | `Intake`, steps a decoder | `Feed` |
| `src/github/server/writes.rs`, `parse` | `Intake`, `Parser` asks `Text` and reads `Long` | `Feed` |
| `src/github/server/queries.rs`, `tokens` and its walker | `Intake`, keeps 16 tokens | `skein_json::parse`, a `Document` walk |
| `src/github/decode/delivery.rs`, `decode` | hand `Fill`/`Scan`, steps a decoder | `Feed` |
| `src/github/tests/fixtures.rs`, `decode_with`, `delivery_decode` | `Intake` | `Feed` |
| `src/forgejo/tests/fixtures.rs`, `capped`, `Pump`, `request` | `Intake` | `Feed`; `Pump` goes |
| `tests/suite/github_start_plans.rs`, `decode` | `Intake` | `Feed` |

Stays: `tests/common/forgejo.rs`, `drive_parts`, and everything calling it.

Pre-approved:

- **Each site's pre-check stays at the site:** `body_bytes` in `hook_url`, `parse` and `tokens`, and
  `json.length`'s `Malformed::Json(TooLong)` in `delivery::decode`, whose fault must stay the
  same whichever the tokenizer would reach first.
- **`writes.rs` takes `Feed`, never `parse`:** `Parser` keeps a field up to its cap with `Text`
  and reads `Long`. `parse` would fail a long string with `StringTooLong` and drop the per-field
  caps.
- **`queries.rs`'s cap stays 16 tokens:** `skein_json::document::Limits { tokens: 16, text:
  limits.body_bytes }`, not the body's length. It bounds the scratch: the walker accepts 13 tokens
  at most, so no test can tell 16 from more, and every refusal is `None`.
- A fixture keeps its signature, its `chunk` override (`Feed::new(body, &tokenizer::Limits {
  chunk, ..limits.json })`) and its panics on a refusal. A `Feed` past its bound answers `Failed`,
  which a fixture's decoder reports as a refusal where the old pump panicked; no input reaches
  the bound, so this changes no test.
- Where `Pump` has callers that keep it as a field, they hold a `Feed<'_>` instead.

## Steps

1. The loop, with `first` as each site has it. An answer but the last reads at least a byte, so
   `0..=body.len()` bounds them, as `parse` does:

   ```rust
   let mut feed = tokenizer::Feed::new(body, &limits.json);
   let mut request = tokenizer::Request::Next;
   for _answer in 0..=body.len() {
       let (demand, step) = decoder.step(feed.answer(request));
       match step {
           decode::Step::Demand => request = demand,
           decode::Step::Done(document) => { /* as today */ }
           decode::Step::Row(_) | decode::Step::Malformed(_) => return None,
       }
   }
   None
   ```

   `writes.rs` reads `request = parser.event(feed.answer(request), specs, limits)?;` and returns
   `parser.fields` on `Request::Close` before asking it, as it does today.

2. `queries.rs`: `tokens` becomes `parsed(body, limits) -> Option<Document>`, its `body_bytes` check,
   then `skein_json::parse(body, &limits.json, &json_document::Limits { .. }).ok()` (`document` in
   this file is the dialect's module: alias skein-json's). The walker reads records through a small
   cursor, where it read `core::slice::Iter<'_, Token>`:

   ```rust
   #[derive(Debug)]
   struct Walk<'a> { document: &'a Document, at: u32 }
   impl<'a> Walk<'a> {
       fn peek(&self) -> Option<&'a Compact>;
       fn next(&mut self) -> Option<&'a Compact>;     // moves past it
       fn expect(&mut self, kind: Kind) -> Option<()>;
       fn text(&mut self, kind: Kind) -> Option<&'a [u8]>;   // the next record, of that kind
   }
   ```

   `string`, `graphql_body` and `read_variables` keep every check and every order; `tokens.next()
   .is_some()` at the end reads `walk.peek().is_some()`.

3. One command at a time, each count above zero:
   - `heavy cargo nextest run -p skein-forge`
   - `heavy cargo nextest run -p skein-fake-forge-github -p skein-fake-forge-protocol -p skein-forge-connection`,
     which read `hook_url`, `write_body`, `graphql_body` and `delivery::decode`.

4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`; `heavy cargo clippy -p
   skein-forge --all-targets -- -D warnings`.
5. Commit ("skein-forge reads whole JSON buffers through Feed and parse (json.md, 5.3)"), pushed.

## Report

As the implementer's text says, and also: what `rg -c 'Tokenizer::new\('` reads for the crate; any
site that gave a different refusal once moved.
