# debt-7: the plan

From `inputs/request.md`: three independent parts. `notes/common.md` holds the standing rules every
task names. This file grows with each release; what is here so far is what has been written.

## Parts and tasks

1. **smith-items** (`skein/docs/design/lib.md, 6`), slice 1. One task per smith item, as the request
   asks, plus what skein must offer first.
   - `tasks/smith-unused-deps.md`, medium, ~15 minutes, needs nothing: D014.
   - `tasks/smith-kind-text.md`, medium, ~20 minutes, needs nothing: D019.
   - `tasks/skein-concat.md`, medium, ~15 minutes, needs nothing: `skein_lib::bytes::concat`.
   - `tasks/smith-primitives.md`, medium, ~75 minutes, needs `skein-concat`: D016.
   - D009 follows in a later batch.

2. **jig D005 and temper D001:** being cut.

3. **skein D029:** being cut.

## What the cut decided

- **D016's `join` is paid here, by a separate skein task.** The item says it waits on skein's D007, and
  its **Paid when** needs smith's `join` gone. `skein-concat` adds `bytes::concat` alone: a branch stays
  in its tree, and a missing generic mechanism goes into skein first. D007 stays open, its copies
  untouched, and its text stops citing smith's D016. So the smith items are one task each, as the request
  says, and each skein mechanism they need is one more.
- **D014 has seven lines, not six.** `smith-local-service`'s dev-dependency on itself turns on its
  `fixture` feature, and has no comment. It stays, with one.
- **D014's check is not built.** The item's **Paid when** does not ask for it.
- **D016's measure misses longer receivers:** 90 against 105 by a wider search. The spec holds both.

## Critical path

So far: `skein-concat`, then `smith-primitives`.

## Overlaps

So far, none: `fabrig-scale` and `v0.1.0-ladder` change no file of these tasks.
