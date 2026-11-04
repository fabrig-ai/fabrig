# hearth-workspace: plan

Draft: hearth joins the root workspace (`inputs/request.md`). A port, no new work: no behaviour and
no design of hearth changes, and the design files take only the citations and status lines the
request allows.

## Parts and tasks

- **join** (slice 1)
  - `tasks/join-steps.md`: the 14 step crates join, built against skein by path; hearth's own
    workspace goes; the nextest clauses and the summed timeouts. Released first.
  - `tasks/join-worlds.md`: the service, binary, fakes and worlds join; the host trait, the service's
    bound, the ceilings; `fail_stop.py`; `always.txt`.
- **rules** (slice 2): `tasks/direction.md`: the check states hearth's place both ways, and `AGENTS.md`
  and the gate's README follow.
- **tier** (slice 2): `tasks/conformance-tier.md`: `conformance-hearth`, in the package map.
- **layout** (slice 3)
  - `tasks/manifest-paths.md`: the 11 `env!("CARGO_MANIFEST_DIR")` sites.
  - `tasks/test-layout.md`: 75 binaries into suites; the inventory test, the final nextest clauses, the
    citation and the status.
  - `tasks/fixture-readers.md`: the 119 fixtures' readers.
- **costs** (slice 3): `tasks/costs.md`: the measurements, in `status.md`, the plan's final status.

## Decisions

- **The port and the join are two tasks, not one.** The request asks for one unless a split lets the
  gate check each step. One does: the 14 step crates depend on nothing above them and on no fake or
  world, so they build and run alone, and on this cut's grep their port is renames and one local
  struct. The judgement (hosts, `worst_case`, ceilings) is all above them, in one task with one
  context. No build ran for this cut: this machine has no builder. The specs hold the survey's table
  as leads, checked by grep against the source, and the compiler has the last word.
- **The split goes no further:** about 150 lines in about 30 files don't pay for a slot each, and every
  stage would meet in `Cargo.toml` and `Cargo.lock`.
- **hearth's nextest clause has three forms,** because a filter's `binary()` that matches nothing may be
  refused: none of `fuzzy_*`, `conformance_*` or `recorded_*` exists until `join-worlds`, and no
  `fuzzy_*` after `test-layout`. `notes/contracts.md`, 2.
- **The test layout is one task, not one per world.** `tests/render/tests/decoders.rs` reads every world's
  fuzzy files by name, so per-world moves would need a both-layouts shim and a removal, for a mechanical
  job the request itself sizes as one task.
- **`service::worst_case` stays, under its old name, summing hearth's own layers and queues.** skein
  removed the io and file terms, and `programming-model.md`, 6.3, ends the whole-process figure. Dropping
  the function and its 1 GiB test would go further than the port. The worlds' whole-process checks become
  measured ceilings.
- **`Memory::Checked` changes only in the builder world.** The render world uses it as a flag beside its own
  `Meter::check`, which still enforces its bound.
- **The survey missed a trap:** with `told` uncapped, the two loops over `told.capacity()` lose their per-pass
  bound. `join-worlds` names them.
- **`recorded_*` leaves the gate's reach, as the request says,** though hearth's own `default` ran it, and
  `real` stays in `default`, because the request's tier names `conformance_*` and `recorded_*` and hearth's
  filter runs it. `conformance-tier` and `manifest-paths` run `recorded_*` once by hand.
- **`hearth/scripts/fail_stop.py` stays.** It doesn't duplicate the gate: nothing of the root's checks this,
  and over the root workspace it names `temper-stage-one` and `playground-serve`, two of temper's binaries.
  Those are outside this request, so the script keeps to hearth, and `status.md` names them for the user.
- **`hearth-render-world` joins `.config/gate/always.txt`,** by that file's own rule: its inventory test reads
  other packages' files. That runs it on every change, so `costs` measures it, and `status.md` names it.
- **`direction` waits for `join-steps`, not for all of the join:** its real-workspace assertions need hearth's
  members. Only the plan's last task reads it.
- **Status lives in `hearth/docs/design/STATUS.md`,** hearth having no per-document status files; `test-layout`
  adds the one `Departs` line the port needs (`domain/testing.md`, 8.4 and 9.2).
- **Nothing here asks the user anything yet.** The budget figures wait for them in `status.md`, as the
  request says, and block no task.

## Critical path

`join-steps`, `join-worlds`, `manifest-paths`, `test-layout`, `fixture-readers`, `costs`: about 8.5
implementer-hours by the specs' estimates. `direction` starts after `join-steps`, and `conformance-tier` after
`join-worlds`, each beside the path.

## Overlaps

- **`Cargo.toml`, `Cargo.lock`:** `debt-7` changes the lock. Not avoidable: the join is those files.
- **skein's own moves:** `debt-7` touches `skein-lib`, `skein-io` and `skein-shell`, three crates hearth uses,
  and `kv-chunk-target` touches `skein-kv`, which it doesn't. Those are additive as listed; if one lands and
  hearth stops compiling, the trunk's end check makes it a repair task.
- **`.config/nextest.toml`, `.config/gate/README.md`, `AGENTS.md`:** several tasks of this plan, in separate
  hunks, ordered by `needs` where they meet. No draft beside this one lists them.
