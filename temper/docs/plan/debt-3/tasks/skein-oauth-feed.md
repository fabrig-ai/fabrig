+++
id = "skein-oauth-feed"
part = "skein-json-feed"
kind = "feature"
needs = ["skein-feed-driver"]
files = ["skein/crates/skein-oauth/src/json.rs"]
effort = "high"
minutes = 30
+++

# skein D001: `skein-oauth`'s `Json` is built on `Feed`

## Goal

`Json::from_bytes` hand-pumps a tokenizer. It takes `tokenizer::Feed` and goes on filling its own
`Collector` (`skein/docs/design/json.md`, 5.3), so `Json` keeps its tokens, its surface and every
refusal. The item's **Paid when** names "`skein_json::parse` or the driver"; this is the driver.
The item stays open: leave its file.

Acceptance:

- `rg -n 'Tokenizer::new\(|Read::|Down::|tokenizer::up|tokenizer::down' skein/crates/skein-oauth/src/json.rs`
  prints nothing.
- `Json`'s fields, its public and crate-private functions but `from_bytes`' body, and `Collector`
  are untouched.
- Every test the Steps run passes unchanged, each count above zero.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `skein/AGENTS.md`.
- `skein/docs/design/json.md`, 5.3; `skein/docs/design/oauth.md`, 4 (the duplicate-field refusal);
  `skein/crates/skein-json/src/tokenizer/feed.rs` on the trunk.
- `skein/docs/development/debt/D001-json-whole-buffer-reads.md`.

## Owns and may change

- `skein/crates/skein-oauth/src/json.rs`: the body of `from_bytes` and the file's imports.

Pre-approved:

- **`Json` stays a token list, not a `Document`:** the cut chose this. `Document::member` picks the
  first of a repeated name, where oauth's `field` refuses it, and that would make a repeated JWT
  claim first-wins. `parse` would also move the `string_bytes` check on numbers to after the
  parse, turning `TooLarge` into `Malformed` for a later syntax error.
- `Event::Long`, `Skipped` and `Closed` are `Err(DecodeError::Malformed)`, in place of an
  `unreachable!` for the first two: `from_bytes` asks only `Next`, and a `Feed` answers
  `Failed(Unexpected)`, which `tokenizer_error` makes `Malformed` as the old loop's fall-through
  did. If clippy or the idiom prefer `unreachable!("only Next is asked")`, keep that instead.

## Steps

1. Sketch; the `document_bytes` check and everything after the loop stay:

   ```rust
   let mut feed = tokenizer::Feed::new(input, &limits.tokenizer_limits());
   let mut collector = Collector::new(limits);
   // Every token reads a byte, so the end comes within one answer more.
   for _answer in 0..=input.len() {
       match feed.answer(tokenizer::Request::Next) {
           tokenizer::Event::Token(token) => collector.push(token)?,
           tokenizer::Event::Done => return collector.finish(limits),
           tokenizer::Event::Failed(error) => return Err(tokenizer_error(error)),
           tokenizer::Event::Long(_) | tokenizer::Event::Skipped(_) | tokenizer::Event::Closed => {
               return Err(DecodeError::Malformed);
           }
       }
   }
   Err(DecodeError::Malformed)
   ```

2. One command at a time, each count above zero:
   - `heavy cargo nextest run -p skein-oauth`
   - `heavy cargo nextest run -p skein-fake-oauth -p skein-oauth-world -p skein-oauth-issuer-world`
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`; `heavy cargo clippy -p
   skein-oauth --all-targets -- -D warnings`.
4. Commit ("skein-oauth's Json is built on the tokenizer's Feed (json.md, 5.3)"), pushed.

## Report

As the implementer's text says, and also: which of the two arms you kept for `Long` and
`Skipped`.
