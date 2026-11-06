+++
id = "replay-drawn"
part = "drawn"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-shell-web/src/journey/replay.rs"]
effort = "medium"
minutes = 20
+++

# The proof gives each page it draws

## Goal

The last scan reads the page at each turn's end, because a page joins what no line shows. `prove_drawn` proves as `prove` does and hands over each turn's page; `Replay::next` and `prove` keep their signatures and what they do.

Acceptance:

- `prove_drawn` as `notes/contracts.md` gives it, called once for each `deadline` of the recording, after that turn's `draw`, with that turn's number.
- `prove` calls it with a visitor that does nothing. The proof's own tests stay as they are, and green.
- `journey.rs` re-exports it, by a line of its own.

## Inputs

- `temper/docs/design/client/journeys.md`, 3.7 and 4.3.
- `temper/docs/plan/real-journeys/notes/contracts.md`.
- `temper/docs/plan/real-journeys/notes/common.md`.

## Owns and may change

Owns `journey/replay.rs`.

Neighbour files:

- `temper/crates/temper-client-shell-web/src/journey.rs`: a new line, `pub use replay::prove_drawn;`, just before `pub use replay::{..}`, which stays as it is, so that this branch meets the others' `pub use` lines cleanly.
- `temper/crates/temper-client-shell-web/tests/suite/host.rs`: the one test below, and its `use` line.

Options, pre-approved: `next` may become a private `next_drawing(&mut self, drawn: &mut dyn FnMut(u32, &Page<'static>))` that `next` calls with a visitor that does nothing, or another shape that keeps `Replay` free of a lifetime.

## Steps

```rust
#[test]
fn the_proof_gives_each_turns_drawn_page() {
    // walked(): count its `deadline` lines; prove_drawn's visitor called that many times,
    // turn numbers rising by one from 1; the last page's html is the recording's `end` mark.
}
```

Then fmt, clippy on `temper-client-shell-web`, `temper-client-journeys` and `temper-client-shell-web`'s tests, commit and push.

## Report

As the implementer's text says.
