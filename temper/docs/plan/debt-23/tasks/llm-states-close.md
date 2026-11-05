+++
id = "llm-states-close"
part = "llm-states"
kind = "docs"
needs = ["llm-connection-phase", "llm-client-states"]
files = ["skein/docs/development/debt/D023-llm-client-states.md", "skein/docs/design/llm.status.md", "skein/docs/design/llm-connection.status.md", "skein/docs/design/link.md"]
effort = "medium"
minutes = 15
+++

# D023 goes, and the statuses say what's left

## Goal

The item's **Paid when** holds, so it goes, with what the statuses say of it.

Acceptance:

- These read 0 or nothing: `grep -c 'state == State::' skein/crates/skein-llm/src/client.rs`; `rg -n 'phase (==|!=) Phase::|idle_at' skein/crates/skein-llm-connection/src/call.rs skein/crates/skein-llm-connection/src/component.rs`.
- `skein/docs/development/debt/D023-llm-client-states.md` is deleted.
- `skein/docs/design/llm.status.md`'s **Departs** on 3, from "3:" up to "4.8:", reads:

  > 3: `skein/docs/foundation/programming-model.md`, 5.4 asks for one
  > explicit state enum per lifecycle, each state holding exactly its
  > data, no lifecycle in flags or `Option`s beside it, exhaustive matches
  > and one handler per cell; `skein-llm/src/client.rs` handles each event
  > in one function that matches the state inside, for which no reason is
  > recorded, and keeps the SSE reader, which 3's table has under Streaming,
  > on `Client` with the other machines, because `close_sse` closes them
  > from any state.

  What follows it, from "4.8:", doesn't change. Its date becomes today's.
- `skein/docs/design/llm-connection.status.md`'s **Departs** on 5, up to "4: beside its table", reads:

  > - **Departs:** 5: `skein/docs/foundation/programming-model.md`, 5.4 asks
  >   for one explicit state enum per lifecycle, each state holding exactly
  >   its data, no lifecycle in flags or `Option`s beside it, exhaustive
  >   matches and one handler per cell; `skein-llm-connection/src/call.rs`
  >   keeps `waiting` beside `Phase`, because the component recomputes it for
  >   every connection in every phase, and `reservation`, because it outlives
  >   the call's token until the pool's pass; neither file keeps one handler
  >   per cell, for which no reason is recorded.

  What follows it doesn't change. Its date becomes today's.
- `skein/docs/design/link.md`, 5's line reads "**The connections' lifecycles** stay the owners': the link holds only its own."
- `python3 -I -B tools/citations/citations.py --all` reads 0, `git grep -n 'D023-llm-client-states' -- skein` finds nothing, and `python3 -I -B tools/debt/debt.py | rg D023` finds nothing.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`, with this departure: this task edits the three design files above, in the words above.
- `DESIGN.md`, 2, 4

## Owns and may change

- **Owns:** the files in the header. No code.
- **If a search doesn't read 0,** stop and ask: the item stays until it does.
- **If `llm-client-states`' or `llm-connection-phase`'s reports say a named field stayed for another reason,** correct that clause to what's true, and say so.

## Steps

1. The searches; the four files.
2. The citations check. Commit and push.

## Report

The common note's items; each search's count.
