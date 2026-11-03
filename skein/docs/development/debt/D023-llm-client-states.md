# D023: skein-llm's client and llm-connection's call use data-less states

- **Kind:** complexity
- **Where:** `skein-llm/src/client.rs`, `skein-llm-connection/src/call.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T4, the skein-llm row) and `reviews/2026-10-31-skein-areas.md` (A9)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 2)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** the client's and the call's states hold their data and transitions use `mem::replace` then follow; no `state == State::` test decides a transition.
- **Measure:** `grep -c 'state == State::' skein/crates/skein-llm/src/client.rs`: 12 at base

## What

`Client` (`skein-llm/src/client.rs`) has a data-less `enum State` beside
its fields and tests it with `state == State::..` 12 times; the review
counted 39 fields and 8 data-less states. llm-connection's `Call` has
the same shape.

## Why it matters

`skein/docs/foundation/programming-model.md`, 5.4. The client is what
every LLM caller in fabrig runs.

## What pays it back

Data-carrying states for the client, then the call, one task each.
