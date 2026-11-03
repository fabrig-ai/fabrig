# fabrig-scale: plan

temper's defaults sized so a fresh box from `temper init` runs a project of fabrig's scale with
5x headroom (`inputs/request.md`): jig's store profile and its formats, the agent, the stock
settings and the forge's limits, every pinned test, and the resource gate's figures in the
status file. Written by the cut as it releases; each batch brings it up to date.

The standing rules are `notes/common.md`; the names and figures that cross tasks,
`notes/contracts.md`.

## What the surveys found (the walls)

- **tasks-v9's shape:** its largest task row holds (batch+1)² members, each priced as counts
  times largest items, so a batch of 160 makes skein refuse the schema (worst case past u32),
  and a 256 kB result, priced thirteen times over in a verdict's fields, breaks it at batch 4.
  jig's `protocol/store.md`, 5.1 already prices a collection the domain bounds by its bytes as
  one piece; tasks-v9 departs from it. The jig family (a designer) reshapes it, in place.
- **48 calls in one response:** smith's transcript schema holds 64 message blocks and 3 MiB of
  opaque bytes; 48 calls need 98 and about 6.4 MB (`smith-calls`).
- **A turn of smith's** is at most 16 MiB (its channel's `turn_body`), but the store's frame and
  the worker wire hold 8 MiB (`transcript-ceilings`).
- **80 questions on a change:** the forge keeps at most 16 open (`forge-questions`).
- **A tool call's payload** is bounded by the model's output (skein-llm: input within answer):
  at init's models' 8,192 output tokens, 10,662 bytes. The agent's `tool_payload` and
  `checkout.message_bytes` take that, not 128 kB; jig's message bound (262,144, at least a
  result) holds what procedures, persons and results carry.
- **The override refusal:** `validate_members` names three relations and marks every other
  override "invalid" when any other fails (`override-relations`).

## Decisions

- jig's `standard` is raised in place to the table (no production code names it); temper's
  application gains `standard` beside `small`; init writes `"standard"` on its three lines
  (contracts K1).
- `tasks.message_bytes` is 262,144, not the table's 128 kB: a result is a message in its
  parent's inbox, and jig holds a result within the message bound (`check_message`).
- Store formats change in place, as the user decided; existing stores are refused at open as
  jig's `store.md`, 5.3 says.

## Overlaps not avoided

- `realistic-defaults`, cut in flight, is an earlier edition of this draft on the same files
  (question `realistic-defaults`).
- `configuring-p3` (integration) owns `temper-service`'s `stock.rs`, `limits.rs` and their tests,
  `jig-protocol-config` pieces, and jig's client wire v19 output; `public-reading-build`
  (integration) touches `jig-protocol-config` and `temper-service/src/limits.rs`. This plan's
  first batch keeps off them but for `jig-protocol-config`'s `domain.rs` and its tests
  (`override-relations`); the stock and limits tasks come later, after their tasks.

## Parts and tasks

| Part | Task | Needs | State |
|---|---|---|---|
| refusal | `override-relations` (jig): every relation between two limits named | | released |
| ceilings | `transcript-ceilings` (jig): a turn of 16 MiB in the store's frame and the worker wire | | released |
| ceilings | `smith-calls` (smith): 48 calls per response, 26 conversations derive | transcript-ceilings | released |
| ceilings | `forge-questions` (temper): 128 open questions on a change | | released |

Slices: 1 (refusal, ceilings).

## Critical path, as it stands

The jig family (tasks-v9 reshaped, `standard` raised) → temper's application profile and
init's documents → the stock → the fresh-box test and the measurement. The first batch runs
beside the jig family's design.
