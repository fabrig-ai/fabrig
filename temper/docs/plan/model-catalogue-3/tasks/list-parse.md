+++
id = "list-parse"
part = "provider-list"
kind = "feature"
needs = []
files = ["skein/crates/skein-llm/src/list.rs", "skein/crates/skein-llm/src/lib.rs", "skein/crates/skein-llm/src/tests/**", "skein/docs/design/llm.status.md"]
effort = "high"
minutes = 60
+++

# What a route's list of models says

## Goal

`skein_llm::list`: the pure reading of a `GET /v1/models` body for both routes, with no io. Nothing calls it yet; `list-exchange` will.

Acceptance:

- `contracts.md`, 1's names exist, with `target` and `parse` as `skein/docs/design/llm.md`, 4.9 says.
- A table test has a row for: OpenAI's plan form with a row of `visibility` `list` and one of `hide`; OpenAI's key form, names alone; Anthropic's whole form, with its efforts low to high whatever order the body has them, and an effort of `supported: false` left out; a row with no name, and one of 129 bytes, left out; a shown name past 64 bytes left out with its row kept; a window of 0 and one past `u32` left out with the row kept; 257 rows keeping the first 256; a body past `Limits.body`; a document with neither member; a body that isn't JSON; unknown members passed over.
- The last two and the over-long body are `Err(Unreadable)`; everything else is `Ok`, an empty list included.
- `llm.status.md`'s 4.9 line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 1.
- `skein/docs/design/llm.md`, 4.9 and 2.6; `skein/docs/design/json.md` for the reader.

## Owns and may change

Owns the header's files. `skein-llm/src/lib.rs` takes its `pub mod list;`. `skein-json` is already step code; add it to `skein-llm`'s manifest if it isn't there.

Options, each pre-approved:

- `skein-json`'s whole `parse` with a bound derived from `Limits.body` is enough; the selective `Tokenizer` isn't needed. If its token bound refuses a body within `Limits.body`, raise the bound you pass, not `Limits.body`.
- A `u64` number past `u32` reads as a field past its bound, not as unreadable.

## Steps

```rust
// skein-llm/src/list.rs
//! A route's list of models (skein/docs/design/llm.md, 4.9). Pure: holds no connection.
pub struct Listed { .. }
pub struct Limits { pub body: u32, pub rows: u32 }
impl Limits { pub const DEFAULT: Limits = Limits { body: 1 << 20, rows: 256 }; }
pub struct Unreadable;
pub fn target(provider: Provider) -> &'static [u8] { .. }
pub fn parse(provider: Provider, body: &[u8], limits: &Limits) -> Result<Box<[Listed]>, Unreadable> { .. }
```

Test in `skein-llm`'s own `--lib`: one table, each row a body's bytes, the route, what it gives. Then `cargo fmt`, `heavy cargo clippy -p skein-llm --all-targets -- -D warnings`, `heavy cargo nextest run -p skein-llm`, commit and push.

## Report

As the implementer's text says. Name the rule you broke for each row kind you made bite.
