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

Parts 3 and 4 (skein D017, jig D028) are being cut; they follow in later pushes.

## Critical path

`smith-bench-observing` → `smith-bench-collect` (~160 implementer-minutes), as it stands; beside it
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
