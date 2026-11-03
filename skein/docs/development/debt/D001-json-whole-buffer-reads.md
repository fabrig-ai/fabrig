# D001: Whole-buffer JSON reads are hand-driven, with no slice-fed driver

- **Kind:** primitives
- **Where:** `skein-json/src/parse.rs`, `skein-json/src/tokenizer.rs`, `skein-oauth/src/json.rs`, `skein-forge`, `skein-fake-forge-github`, `skein-fake-forge-protocol`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the JSON pump row) and `reviews/2026-10-31-skein-areas.md` (A7, F1 and H6)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 1e)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high
- **Size:** M
- **Status:** open
- **Paid when:** skein-json has one slice-fed driver of the tokenizer; `parse` and `validate` both run on it; no step crate outside skein-json drives a tokenizer over a whole buffer by hand, and `skein-oauth`'s `Json` is built on `skein_json::parse` or the driver.
- **Measure:** `rg -c 'Tokenizer::new\(' skein/crates skein/testing skein/tests skein/examples -g '!**/skein-json/**' | awk -F: '{s+=$2} END{print s}'`: 41 at base (streaming users included; it falls as pumps go)

## What

- skein-json's `parse` (`skein-json/src/parse.rs`) and `validate`
  (`skein-json/src/tokenizer.rs`) are one loop written twice: each
  builds an `Env`, a tokenizer and two queues, then meets `Fill` and
  `Scan` from the slice by hand, ticking `input.len() * 4 + 16` times.
- Outside skein-json, 41 `Tokenizer::new` sites in 31 files make a
  tokenizer. The review counted about 25 of them as whole-buffer pumps,
  in three idioms: a hand `Fill`/`Scan` meeting (for example
  `skein-forge/src/github/decode/delivery.rs`,
  `skein-fake-forge-github/src/app.rs`), a copy into an `Intake` that
  `meet`s (`skein-forge/src/github/server/app.rs` and its siblings), and
  lib's `Held::answer`.
- `skein-oauth/src/json.rs` (397 lines) keeps its own `Json` value and
  pumps the tokenizer itself in `Json::from_bytes`. skein-llm's
  `openai/json.rs` has moved onto `skein_json::parse`, so it is no
  longer one of them.

## Why it matters

Every whole-buffer read carries its own copy of the meeting loop and
its tick bound. A fix to the loop lands in one copy at a time, and a
reader of each site has to check the copy again. Where skein offers
the one-call form, callers take it: jig's config document moved onto
`skein_json::parse` once it was found.

## What pays it back

A borrowed sibling of lib's `stream::Held` and a `Feed` in skein-json
that owns a tokenizer and answers one request per call (the review's
A7 F1). `parse` and `validate` become a few lines on it. Sites that
collect every token move to `skein_json::parse` and a `Document` walk;
sites that step a streaming decoder take `Feed`. `skein-oauth`'s `Json`
goes onto it last. The contract lands in lib.md and json.md first
(`skein/docs/design/json.md`, 5.3).

temper's three whole-buffer pumps are temper's own item,
`temper/docs/development/debt/D002-json-pumps.md`: they need only
`skein_json::parse`, which exists, so they do not wait on this one.
