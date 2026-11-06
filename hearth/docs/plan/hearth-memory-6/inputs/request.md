# Request: hearth to the dynamic memory model, round 6: the service, and the end of D001 and D004

From fabrig/hearth, 2026-11-06, on the user's decision to migrate hearth, low priority. Rounds 1 to 5 removed `worst_case` and uncapped every machine. This is the last round and the hard one: `hearth-service-builder`'s `iterate`, and the domains' constants.

Read whole: `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, including "What each room check keeps" and "What the service round must finish"; `hearth/docs/development/debt/D004-service-drives-io-by-hand.md`; `skein/docs/foundation/programming-model.md`, 6.3, 7.1 and 7.2; `hearth/docs/design/domain/render.md`, 8.5. `skein/testing/skein-world/src/files.rs` and `skein/crates/skein-io/src/stage.rs` show the form this takes.

**What hearth does for a user doesn't change:** same specs, same digests, same refusals at the edges. **Both items are paid by this round:** their **Paid when** holds and their files go.

## 1. Stated orders first

`hearth/docs/design/domain/render.md`, 8.5 lists the orders. Each stays a named predicate or a machine's state, with its test. Every check D001's tables call a capacity between the root's own children goes. The kernel's edge stays. Don't take a check out until the row beside it says what keeps its order.

## 2. D004: the shared stage

Routing goes through `file_layer::owns`, the service calls `skein_io::stage::{up, down}`, and `tag` and `io::resume` go. Do this first: it takes `Io`'s constants out of the service. `hearth/tests/` knows about `tag` where it reads tokens: follow it.

## 3. D001: the constants and the queues

The domains' `MAX_OUT` and `hearth-protocol-builder`'s `MaxOut` go, with `within`, `io_within` and `protocol_within`. The queues between the root's own children are uncapped. `Service::new`'s `limits.queue >= 3` goes, and `hearth/tests/builder/tests/suite/bounds.rs` stops sizing a world on it. `limits.queue` stays only where an edge's queue uses it; if nothing does, it goes. No stored format or supported deployment holds it.

These were left open, and are decided here:

- **`answers` and a stuck client.** This is the one room check that does real work: it stops render outrunning a client that has stopped reading. Keep that behaviour by the model's means, a **watermark** (7.1): one number, a field of the builder's `Limits`, defaulting to what keeps pacing close to today's, about 30 queued records. Render is held at it, and what finishes admitted work is always taken. A world test holds it: a stuck client and many targets, peak under a named ceiling (`skein/docs/foundation/testing-strategy.md`, 6), and the ceiling bites when the watermark is removed.
- **The loop bounds.** `0..calls.capacity()` and its like go. Pace as skein's stage and the other trees' roots do. One pass mustn't run a whole render, or an equally plain statement of what stops it must be in the code and tested.
- **The store step's named predicate,** as D001 asks.
- **`work_pending`: the guard stays.** 384 fixed cases couldn't show what it keeps. Try once more, as D001 says, in about 45 minutes of work: pending cache work alongside stdin room, child exit and pipe-end events. If a test bites, it's added. If none does, the order remains a named predicate that `render.md`, 8.5 states with no test: say so in `render.status.md`, and add a small debt item with its search, under the next free ID in `hearth/docs/development/debt/`. D001 can still be deleted. Don't remove the guard either way.

## 4. Design

Where this changes what a document says, the document changes first, with its status: `domain/render.md`, 8.4 and 8.5, `domain/steward.md`, `README.md`, `domain/testing.md`. Keep section numbers. D001's list of checks goes from `render.md`, 8.5's citation with the file.

## 5. What to report

In the plan's final status, the gate's figures before and after: `default` and `fuzzy` CPU for hearth, with `tools/tiers/cpu.py` and `NEXTEST_DOUBLE_SPAWN=0`; the large render's and the builder scenarios' peaks; `max_out` and `worst_case` counts. The user holds hearth's focused suite to 30 CPU s; it's about 32 today. Don't raise that figure. Say whether this round got it under 30, and if not, what the largest costs are, for a small plan after this one.

## Rules

- Order the tasks so every landing keeps the gate green and each step can be checked; the service tasks are one family, and the fewer in parallel on `lib.rs` the better.
- A ceiling moved by this round gives the peak that moved it, and why. A failing test is a finding first: don't weaken one to get past it.
- `hearth/scripts/old-forms.txt` lowers with `--write`. D006, D009 and D013 are fabrig/debt's: leave them.
- A skein change hearth seems to need is a question, not a task. skein and jig are shared with other sessions.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- All parts must be in the plan when the cut ends.
