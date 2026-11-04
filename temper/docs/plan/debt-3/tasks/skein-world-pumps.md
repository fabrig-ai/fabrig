+++
id = "skein-world-pumps"
part = "skein-json-feed"
kind = "feature"
needs = ["skein-feed-driver"]
files = [
  "skein/crates/skein-forge-setup/src/tests/github_manifest.rs",
  "skein/tests/fake-peers/tests/forge/github.rs",
  "skein/tests/forge/github/src/runner.rs",
  "skein/tests/forge/setup/src/decode.rs",
]
effort = "medium"
minutes = 40
+++

# skein D001: skein's test worlds take `Feed`

## Goal

Four test sites drive a tokenizer over a whole buffer by hand. Each takes `tokenizer::Feed`
(`skein/docs/design/json.md`, 5.3). The item stays open: leave its file.

Acceptance:

- `rg -n 'Tokenizer::new\(' skein/crates/skein-forge-setup/src/tests/github_manifest.rs
  skein/tests/fake-peers/tests/forge/github.rs skein/tests/forge/github/src/runner.rs
  skein/tests/forge/setup/src/decode.rs` prints nothing (4 at base).
- Every test the Steps run passes unchanged, each count above zero.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `skein/AGENTS.md`.
- `skein/docs/design/json.md`, 5.3; `skein/crates/skein-json/src/tokenizer/feed.rs` on the trunk.

## Owns and may change

Only these functions, and each file's imports:

- `skein/crates/skein-forge-setup/src/tests/github_manifest.rs`, `tokens`: a `Feed` collection into
  its `List<Token>`, which `wanted`'s comparison reads; its panics stay.
- `skein/tests/fake-peers/tests/forge/github.rs`, `decode_with`: its panics on a malformed body stay.
- `skein/tests/forge/github/src/runner.rs`, `decode`: its `Intake` of `bounds.writer.length` also
  refused a longer body, so a check `bytes.len() > bounds.writer.length` keeps the "dialect
  intake bound" error first. Its "dialect decoding progress bound" error stays at the loop's end.
- `skein/tests/forge/setup/src/decode.rs`, `drive`: it keeps its `json.string` shrink. It drops its
  `usize` return if its one caller, `decode_answer`, still discards it: check that first, and if it
  is read, keep the count of answers.

Pre-approved:

- A site's `chunk` or `string` override goes in the `Limits` it hands `Feed::new`.
- `drive`'s closure stays. It reads the event by reference, so `event(&feed.answer(demand))`.
- `0..=bytes.len()` bounds the answers, since an answer but the last reads at least a byte; the
  panic or error at the loop's end stays.

## Steps

1. The shape, here `runner.rs`:

   ```rust
   if bytes.len() > usize::try_from(bounds.writer.length)? {
       return Err("dialect intake bound".into());
   }
   let mut feed = tokenizer::Feed::new(bytes, &bounds.json);
   let mut request = tokenizer::Request::Next;
   let mut rows = vec![];
   for _answer in 0..=bytes.len() {
       let (next, step) = decoder.step(feed.answer(request));
       match step {
           github::decode::Step::Demand => {}
           github::decode::Step::Row(row) => rows.push(row),
           github::decode::Step::Done(document) => return Ok((rows, document)),
           github::decode::Step::Malformed(_) => return Err("dialect document malformed".into()),
       }
       request = next;
   }
   Err("dialect decoding progress bound".into())
   ```

2. One command at a time, each count above zero:
   - `heavy cargo nextest run -p skein-forge-setup --lib -E 'test(manifest)'`
   - `heavy cargo nextest run -p skein-fake-peers-tests`
   - `heavy cargo nextest run -p github-conformance`
   - `heavy cargo nextest run -p skein-forge-setup-world`
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`; `heavy cargo clippy -p
   skein-forge-setup -p skein-fake-peers-tests -p github-conformance -p skein-forge-setup-world
   --all-targets -- -D warnings`.
4. Commit ("skein's test worlds read whole JSON buffers through Feed (json.md, 5.3)"), pushed.

## Report

As the implementer's text says.
