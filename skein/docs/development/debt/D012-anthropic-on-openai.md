# D012: Anthropic's dialect stands on `openai/`, with a mirror vocabulary

- **Kind:** coupling
- **Where:** `skein-llm/src/anthropic.rs`, `skein-llm/src/anthropic`, `skein-llm/src/openai`, `skein-llm/src/types.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T2, the skein-llm row; A9) and `reviews/2026-10-31-skein-areas.md` (A9)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 3)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** Anthropic's modules name nothing under `openai::`; what both dialects share (failure classes, rate headers, the JSON value layer) sits in a neutral module; `Stop`, `Usage`, `Failure` and `Output` are each defined once, with no identity mapping between copies.
- **Measure:** `rg -c 'openai::' skein/crates/skein-llm/src/anthropic.rs skein/crates/skein-llm/src/anthropic -g '!*tests*' | awk -F: '{s+=$2} END{print s}'`: 35 at base

## What

- `anthropic.rs` re-exports nine OpenAI items (`DecodeError`, `Failure`,
  `Json`, `Limits`, `ProviderError`, `RateLimit`, `Stop`, `Usage`,
  `classify`); Anthropic's modules name `openai::` 35 times.
- `openai/common.rs` defines `Stop`, `Usage` and `Failure` beside the
  crate's own in `types.rs`, mapped variant for variant; `Output` is
  defined in both `openai/response.rs` and `anthropic/response.rs`.

## Why it matters

Anthropic's failure classes and rate headers live in an OpenAI file, so
a change for one provider is made in the other's code, and each mirror
pair needs its mapping kept in step.

## What pays it back

A neutral `wire` module for what both dialects share, the crate's own
`Stop`, `Usage` and `Failure` used directly, and one `Output`.
