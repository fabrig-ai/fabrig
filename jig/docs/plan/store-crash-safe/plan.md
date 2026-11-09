# store-crash-safe: plan

A deployment's engine, killed twice by SIGKILL after ignoring SIGTERM,
left a store whose next start panicked:
`Tasks(Live(199)) links missing People(Person(102))`. The evidence is
outside the repository. The plan hunts the cause with failing rows over
the two candidate families a code read left standing, fixes what they
show, and turns the start's link panic into a refusal that names both
records, so a store like it stops with a reason.

## Parts and tasks

| Part | Task | Needs | Trees | Request item |
|---|---|---|---|---|
| `link-refusal` | `tasks/link-refusal.md` | | jig, temper (a reason's words, required arms) | 2 (a refusal, not a panic) |
| `crash-rows` | `tasks/crash-rows.md` | | jig (skein if a row shows it) | 1 (the failing story), 2 (the fix, if a kill is the cause) |
| `writer-rows` | `tasks/writer-rows.md` | `link-refusal` | jig | 2 (the fix, if a writer is the cause) |

Three tasks, within the request's four. Item 3 is dropped to
`worker-relinks` (below); item 4 is this cut's own writing.
`writer-rows` waits for `link-refusal` because both write
`jig-domain-core` and its rows read the refusal.

## Design written by this cut

- `jig/docs/design/domain/engine.md`, 5.1: *Whole across families,
  through a kill*; *Nothing links ahead of what it names*.
- `jig/docs/design/domain/engine.md`, 6: *It refuses a link it cannot
  make* and *A link that cannot be made* (`Failure::Link`, `Linked`),
  replacing the assertion of 10-28; 16: the open question on a record
  left out now says the start refuses, and the duplicate identity's
  question is settled into 6.
- `jig/docs/design/protocol/store.md`, 14: *The core's world on the
  simulated disk*.
- Status: `engine.status.md` **Not yet** 6 (`link-refusal` removes it)
  and 5.1 (`writer-rows`); `store.status.md` **Not yet** 14
  (`crash-rows`).

## Decided

- **No crash-only cause was found by reading.** A survey of skein-kv,
  `jig-protocol-store`, the core's routing and the start's restore found
  one decision is one skein-kv commit, applied to the map only once its
  frame is synced, recovery keeps the longest whole prefix and installs
  a repair snapshot before new commits, and every Person row restores.
  Reading proves nothing about the deployment's store, so the kill
  candidates get rows (`crash-rows`), at the lowest world holding both
  the store on the simulated disk and the start's link pass: the core
  world's directory-backed store.
- **A second family: a writer.** The panic shows the people child did
  not hold person 102 at the link pass, not that a kill split a
  decision. The request's own words ("person 102 a person task") allow
  that 102 was a task's number saved where a person's goes, by a route
  that names a person from outside the core and is not checked at
  admission. That needs no kill at all, and the store would have failed
  at any later start. `writer-rows` gives each such route a row.
- **The link check refuses** (engine.md, 6): `Failure::Link { record,
  linked }`, the first found, before any sweep. That replaces the
  user's assertion of 10-28 by the request's words ("stops with a
  reason, not a panic"). A duplicate person's identity, which the design
  called a link that cannot be made, refuses the same way. Repairing a
  store is not built.

## Dropped

- **Item 3, the clean stop,** to `worker-relinks`, not landed when this
  was cut: its `engine-link` (merged on its trunk) bounds the workers'
  channels' close by the stall deadline, and its `worker-reads` (in
  repair) stops an in-process worker's untaken output from holding the
  signal's delivery. A stop survey found no other part held unbounded
  by a fault: the remaining waits (commits, components, agent
  processes) are bounded only by their own deadlines, with no overall
  stop deadline, which would be a figure the request excludes. A task
  here would edit `worker-reads`' files (`temper-service/src/system.rs`).

## Overlaps with other drafts

- `temper/crates/temper-engine-shell/src/serve/host.rs`:
  `link-refusal` adds an arm to `start_failure` and a row to its words
  table; `worker-relinks/worker-reads` edits the same file's worker log
  lines, elsewhere in it.
- `jig/docs/design/domain/engine.status.md`: `worker-relinks/restart-loads`
  removes its **Departs** 9.1 entry; this plan's entries sit at the
  file's end, away from it.

## Critical path

`link-refusal` (150 minutes), then `writer-rows` (180): about 5.5 hours.
`crash-rows` (240) runs beside them from the start. A row that rules a
candidate in adds its fix to its own task; one that rules both out
leaves `link-refusal` standing as the request's guard and the cause
unknown, which the report then says.
