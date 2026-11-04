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

Parts 2 to 4 (skein D017, jig D028, smith D010) are being cut; they follow in later pushes.

## Critical path

`temper-sweep-engine` → `temper-sweep-paid` (~75 + 15 = ~90 implementer-minutes), as it stands.

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
