# debt-2: the plan

From `inputs/request.md` (fabrig/debt, 2026-11-04): four debt items, one per tree, each paid
on its own branch so that its **Paid when** holds. The four read none of each other's code, so
each is its own part and slice, released as soon as its tasks are written. `notes/common.md`
holds the standing rules every task names.

## Before the tasks

This cut wrote the design each task follows, pushed with the specs:

- **skein, `ui.md` 15:** `Limits::EMPTY` (no capacity: an empty tree without scenes, which a
  caller sets the fields it means over) and a public `Node::text_bytes` (the bytes the
  builder charges a node); the table's `text_per_node` row names the image address the code
  already counts. `ui.status.md` records both as **Not yet**.

## Parts and tasks

1. **skein-ui-limits** (skein D046; `skein/docs/design/ui.md, 15`), slice 1.
   - `tasks/skein-ui-empty-limits.md`, medium, ~45 minutes, needs nothing: the constant and
     the public count; skein's two literals and `memory.rs`'s two hand counts move onto them;
     the status's 15 words, the item and its row go; jig's D022 and D023 stop waiting on it.

The jig (D018), smith (D002) and temper (D005) parts follow once their surveys report.

## Critical path

So far: `skein-ui-empty-limits` alone (~45 implementer-minutes).

## Calls made

- **`Limits::EMPTY` is the constructor** D046 asks for: a `pub const` base that callers
  update with struct syntax (`Limits { nodes, depth, ..Limits::EMPTY }`), as skein's own
  `limits.rs` test already writes its cases. A function of the ten tree fields would make
  callers spell all ten again, positionally; a `Default` would hide that zero is a capacity.
- **D046's Measure reads 32 after, not 0:** the request keeps jig's 25 and temper's 7
  literals for a later round (jig's D022, temper's literals); skein's 2 go.
- **D046's accounting is `Node::text_bytes`,** the builder's own count made public, not a new
  function: it already is the one the builder charges, and skein's own `memory.rs` test
  copies it twice.
