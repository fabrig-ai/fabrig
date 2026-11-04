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
   - `tasks/skein-startup-store.md`, high, ~50 minutes, needs nothing: `skein_shell::store_file`.
   - `tasks/smith-startup-writers.md`, high, ~50 minutes, needs `skein-startup-store`: D009.

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

- **D009's call takes an open root.** The two sites differ: credentials open their directory through io's
  private-file checks, while the generated configuration's directory may be any directory a person named,
  so a private check there would refuse what runs today. `store_file` serves both, with
  `StoreDirectory::{Root, Private}` (`skein/docs/design/shell.md`, 6.7). The caller opens the root, so the
  errors both sites answer stay as they are.
- **D009 adds lines.** A skein host and its first tests, against two copies smith loses, and the item asks for
  a new cleanup host too: about +190 in skein, at most -50 in smith, both capped in their specs. Part
  smith-items may not end below its start, against the request's aim. The tasks' reports give the sum.

## Critical path

So far: `skein-concat`, then `smith-primitives`; `skein-startup-store`, then `smith-startup-writers`.

## Overlaps

So far, none: `fabrig-scale` and `v0.1.0-ladder` change no file of these tasks.
