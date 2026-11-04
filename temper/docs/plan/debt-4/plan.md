# debt-4: the plan

From `inputs/request.md` (fabrig/debt, 2026-11-04): four debt parts, one per tree, each paying
its item so that the item's **Paid when** holds. They read none of each other's code, so each
is its own part and slice, released as soon as its tasks are written. `notes/common.md` holds
the standing rules every task names.

## Parts and tasks

1. **temper-forms** (temper D003, D004; `skein/docs/design/lib.md, 6`), slice 1. `notes/temper-forms.md`
   holds the sweep's table, what stays and its pre-approved fallout.
   - `tasks/skein-now-with.md`, interface, medium, ~15 minutes, needs nothing.
   - `tasks/temper-sweep-engine.md`, medium, ~75 minutes, needs nothing.
   - `tasks/temper-sweep-forge.md`, medium, ~60 minutes, needs nothing.
   - `tasks/temper-sweep-service.md`, medium, ~55 minutes, needs nothing.
   - `tasks/temper-sweep-client.md`, medium, ~45 minutes, needs nothing.
   - `tasks/temper-sweep-worker.md`, medium, ~25 minutes, needs nothing.
   - `tasks/temper-sweep-worlds.md`, medium, ~35 minutes, needs nothing.
   - `tasks/temper-env-now.md`, medium, ~15 minutes, needs `skein-now-with`.
   - `tasks/temper-sweep-paid.md`, docs, medium, ~15 minutes, needs all of the above.

2. **smith-bench-adapters** (smith D010; `smith/docs/design/benchmarks.md, 6`), slice 2.
   - `tasks/smith-bench-observing.md`, high, ~70 minutes, needs nothing: a replay is a Codex
     observation, `Capture::Codex` and `ProcessResult.observer` go, `process.rs` names no agent.
   - `tasks/smith-bench-collect.md`, high, ~90 minutes, needs the first: the runner's collection,
     archives, source selection and startup reading become adapter methods; the item goes.

3. **skein-kv-states** (skein D017; `skein/docs/foundation/programming-model.md, 5.4`), slice 3.
   `notes/kv-store.md` holds the binding shape, what each flag was and what stays.
   - `tasks/skein-kv-admission.md`, high, ~45 minutes, needs nothing: `map::Queued`; the O(n²) copy
     goes.
   - `tasks/skein-kv-table.md`, high, ~80 minutes, needs the last: `State`, `Dir`, `Done`, and `up`
     by state, then event.
   - `tasks/skein-kv-compaction.md`, high, ~70 minutes, needs the last: one `Compaction`.
   - `tasks/skein-kv-recovery.md`, high, ~75 minutes, needs the last: `Serving`, `Repair`, and
     reading stops at the first failure.
   - `tasks/skein-kv-subset.md`, high, ~50 minutes, needs the last: no module-wide `#![expect]`, the
     batch in `Writing` and `Syncing`; the item goes.

Part 4 (jig D028) is being cut; it follows in a later push.

## Critical path

`skein-kv-admission` → `-table` → `-compaction` → `-recovery` → `-subset` (~320 implementer-minutes),
one writer on one crate; beside it `smith-bench-observing` → `smith-bench-collect` (~160) and
`temper-sweep-engine` → `temper-sweep-paid` (~90).

## Overlaps

Against the trunks of `fabrig-scale` and `playground-browser`, 2026-11-04: of the 142 files the
sweep touches, five are also changed there: `temper-engine-domain/src/limits.rs`,
`temper-protocol-store/src/tests/records.rs`, `tests/client/src/front.rs`,
`tests/engine/domain/src/forge_routes.rs` and `tests/engine/domain/tests/suite/forge_routes/goal_tool.rs`.
The sweep cannot leave them (a measure must read 0), and each edit is one line; `temper-sweep-paid`
sweeps what such a merge brings in.

## Calls made

- **D004's 78 are not all `Env`s.** 58 rewrap an `Env`. 13 read a `skein_shell::Now`, which has no
  `with`, so skein-shell gains `Now::with` (`skein/docs/design/shell.md`, 5), an addition, and
  `temper-env-now` takes those sites. 7 read `temper-client-tools`' test `Harness`, which holds its
  own clocks; one `Harness::env` serves them, and those fields stay.
- **D003 reads `"u32 fits usize"` as the text,** which is what the **Paid when** names, and so 111
  sites at base; the other `usize::try_from` texts are outside it. The note sweeps nested calls the
  Measure's regex does not see, and leaves `size_of` and sums or differences of lengths.
- **D010's "run.rs" reads as the runner:** `run/cache.rs`, `run/startup.rs`, `run/traces.rs` and
  `run/outcomes.rs` branch on the agent too, so they are in the search; a new arm would edit them
  otherwise. `run.rs`'s seeded author (`user.name=Smith benchmark`) and the stored `codex_home` stay.
- **A replay is `Observing::replay()`,** a Codex observation with no preparation, named by no agent
  at the runner; it keeps today's `result.json`, so it collects nothing and answers today's cache
  text.
- **D017 is five tasks, one writer, under one note.** The item says one state at a time; the outer
  shape goes first, so the flags are removed from handlers that are already per state and event.
  The store's lifecycles are `State`, `Phase`, `Compaction`, `Serving` and `Repair`.
- **`repairing` and `repair_required` are one value,** by reading every write and read of both, and
  `Repair` stays on `Store` because `open_failed` keeps it across a failed start.
- **Recovery stops at the first failure or damaged frame,** which is what kv.md, 3 says and what
  `skein/docs/design/kv.status.md` records as a departure. Today it reads on to the file's end. The
  visible difference is only in a doubly bad file, or a read failing after the damage.
- **A bug found, not fixed:** a failed create or directory sync on a rollover stops the store with
  no event and leaves its queued commits unanswered. D017 keeps behaviour, so it stays; this cut
  wrote it into `skein/docs/design/kv.status.md`, 4 as a **Departs** entry.
