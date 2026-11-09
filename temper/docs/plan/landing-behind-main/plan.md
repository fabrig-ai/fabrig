# landing-behind-main: the plan

Two faults a real Forgejo deployment met (`inputs/request.md`): a change
far behind main waited at its turn until its stall held it (A), and a
resolution's every start was refused at its worker for its charter's size
(B). Four tasks, one per crate, each with its failing rows first. All four
need nothing of each other and go out with the first push.

## Design written by this cut

- `temper/docs/design/domain/forge.md`, 5.3, *Containment needs only
  ancestry*: any commit the range lists disproves containment, whether or
  not more follow, and so does a range too large to list; an empty page that
  says more follow stays unknown; a catch-up's range reads by the same rule.
  8.4, *Stalls*: a head behind its base is no wait. 8.4, *A resolution*: its
  pull section carries a file whole where it fits, or names it for the
  checkout. 8.7, rows 2 and 6: their Wait and Refuse cells agree with 5.3.
- `temper/docs/design/domain/agent.md`, 4.3, *The brief fits the room its
  charter leaves*: the engine fits a charter to the smaller of
  `hub.charter_bytes` and its worker's allowance; section 10 gains *A
  charter where the process has no local agent*.
- `temper/docs/design/protocol/configuration.md`, 13.8, *A brief*: agrees
  with 4.3.
- `skein/docs/design/forge.md`, 5.2, the `Read::Commits` row: Forgejo's plan
  keeps the oldest commits it can hold and answers a page they hold whole.
- Status, one **Not yet** per task, each its task's to remove:
  `forge.status.md` (5.3, 8.4, 8.7 at the file's end; 8.4 *A resolution* in
  its `Not yet:` list, apart), `agent.status.md` (4.3, at its end),
  `configuration.status.md` (13.8, at its end), skein's `forge.status.md`
  (5.2).

## Decided

- **Which limit refused task 199's charter.** The worker's smith charter
  allowance, `run_bytes`, which its host takes as `charter_bytes`: the larger
  of 65,536 and twice `tool_payload`, so 65,536 on the stock agent's
  `tool_payload` of 10,662 (`temper-init`'s `agent.json`). It refuses at
  `jig-protocol-worker-agents/src/route.rs`, `decode`:
  `smith::Charter::new(charter, host_limits.charter_bytes)`, the
  `Undecoded::Charter` that the worker reports as `Invalid(Start(Other)):
  charter` (`temper-worker-domain/src/agents.rs`). The engine had fitted the
  charter to `hub.charter_bytes`, 262,144 under the `standard` profile
  `temper init` selects. Neither is the worker's jig hub bound, which refuses
  as `Invalid::Charter`, nor smith's decoding ceilings (`section_body`,
  262,144). So any charter owning more than 65,536 bytes was refused on every
  attempt, before the worker's own fit could cut it.
- **The engine's bound.** Where the process has a local agent, its
  allowance, as a task read's room takes its reply bound. Where it doesn't
  know its worker's, smith's floor, 65,536: the smaller of the two defaults,
  as the request decides. The request's "as `worker-reply-bound` sizes
  reads" names a mechanism that doesn't exist yet: how the engine learns a
  remote worker's bounds is `agent.md`, section 10's open question, a
  protocol change, so it isn't built here. A consequence: a remote worker
  that takes more, the reference's 127,480 among them, gets its brief cut
  at 65,536 until it does (written in that question).
- **Forgejo keeps the oldest commits.** Forgejo lists a comparison newest
  first, so a plan that stopped at its capacity holding what came first
  would answer a page from the middle of the range. Keeping the oldest
  commits, giving up the newest held as older ones come, keeps page 1 exact,
  and a page those don't hold whole stays `TooLarge`.
- **A large file is measured against the section's room.** The connector
  knows its section's room, not the charter's. A file that fits the section
  but not the charter is cut at the charter's fit with its own line, as any
  section is. That gap is written in `forge.md`, 8.4.
- **Tests are rows of tables:** the forge world's First table on the fake
  forge (A, temper), the Forgejo plan's table (A, skein), one story in
  `temper-service` running both the engine's encoding and the worker's
  decoding (B), and the connector's brief table (B). The existing ancestry
  tables give up the rows whose meaning changed, and the catch-up's start
  table takes them.

## Parts and tasks

| Slice | Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|---|
| 1 | `ancestry` | `ancestry-lacks` | `temper-engine-domain-forge` (`ancestry_status`); forge world tests | | 90 |
| 1 | `commits-capacity` | `forgejo-commits-oldest` | `skein-forge` (`forgejo/plans/commits.rs` and its tests) | | 75 |
| 2 | `charter-bound` | `charter-worker-bound` | `temper-service` (`assignment.rs`, `settings.rs`, `startup.rs`, one line of `workers.rs`, tests) | | 90 |
| 2 | `conflict-files` | `conflict-files-named` | `temper-engine-domain-forge` (`brief.rs` and its tests) | | 60 |

`ancestry-lacks` and `conflict-files-named` share a crate on separate
files (`domain.rs`, `brief.rs`) and separate test files, so they run
together. `walk = false` for both slices.

## Critical path

Every task is released with the first push and needs none other: the
longest, `ancestry-lacks` and `charter-worker-bound`, about 90 minutes
each.

## Overlaps with drafts beside this one

- `worker-relinks/worker-reads` (not merged) is changing
  `temper-service/src/workers.rs`. `charter-worker-bound` edits one line of
  it, the `charter_bytes` member in `Workers::configured`, far from that
  task's hunk on its trunk; it stays off that task's tests and fixtures.
- `forge.md`, `agent.md`, `configuration.md` and their status files have
  edits in flight on the ui-reaches, look-round-25 and ui-p3e-words trunks,
  none in or beside the hunks this cut wrote.
