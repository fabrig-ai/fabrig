# projection-full-outbox: the plan

A deployment's engine spun inside one `Engine::step` (`inputs/request.md`):
a goal's issue projection whose next write met the forge outbox at its
bound emitted `ProjectAfter { goal, when: None }`, the root answered by
projecting it again at once, and the outbox, which frees only by a
write's outcome or a withdrawal, was still full. The repair makes the
projection wait and gives the root a backstop.

Design written by this cut:

- `temper/docs/design/domain/forge.md`, 6: a new bullet, *The outbox is
  bounded*: nothing is committed past `entries`; room frees as an entry
  leaves; a projection that meets it full waits (section 12).
- `temper/docs/design/domain/forge.md`, 12: a new bullet, *A full outbox
  defers it*: the projection waits for room, neither failed nor settled,
  tried again after the policy's interval as a failed write is; a restart
  tries it at once; once a step has met the full outbox, a projection due
  at once in that step waits the interval too while the outbox stays
  full, so the step ends.
- Its status, `forge.status.md`: two **Not yet** entries for 12, apart,
  one per task, each removed by its task.

No draft carries this: the source is a request, so no draft status moves.

## Decided

- **The wait is the timer, not a wake when an entry frees** (the request's
  choice, by the code's idiom). Every wait of a projection in the forge
  is a `projection_due` timer: `fail_projection`'s retry, a failed
  outcome's retry, `issues::Decision::Wait`; restore re-arms it for a row
  with nothing in flight; and no path in the forge wakes a waiter when
  another entry leaves the outbox. A wake on free would add state and a
  restart rule for one case. The cost is latency: up to the interval (5 s
  as configured) after room frees, the cadence the design already gives an
  issue's body.
- **Not failed.** The wait sets the due time as `fail_projection` does but
  marks nothing failed and emits no `ProjectionFailed`: a full outbox is
  not the projection's failure. An entry number already in the outbox
  keeps `when: None`, since the root asks again with a fresh number.
- **The backstop is step-scoped, as the request words it.** The forge
  keeps a volatile mark, *a projection met the full outbox since the
  root's step began*; the root clears it as each of its public entries
  (`step`, `fire`, `release`) begins, and its `ProjectAfter { when: None }`
  arm re-projects only unless the mark is set and the outbox is still
  full, arming the projection's timer at `wall + interval` instead. A
  stateless guard (never re-project into a full outbox) was simpler but
  would also delay, in any step, re-projections that need no write, such
  as a closing projection's settle; the mark confines the delay to the
  step that met the full outbox.
- **Tests:** the failing test is a forge table (`full`, `taken`, `freed`)
  that fails on an assertion, not a hang. The root's story, the
  deployment's case ending its step, runs only after the fix has merged
  (`requeue-backstop` needs `room-wait`), so no slot runs a test that
  loops forever. The backstop's own table fails first on its `met` row.
- **No figure moves:** the outbox's bound, the interval and every limit
  stay; no store, schema or golden changes.

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `full-outbox` | `room-wait` | `temper-engine-domain-forge` (`domain.rs`: `project`, `wait_for_room`, the mark; `tests/projection.rs`); its `forge.status.md` entry | | 60 |
| `full-outbox` | `requeue-backstop` | `temper-engine-domain` (`route/forge_route.rs`'s `ProjectAfter` arm; `domain.rs`'s `step`, `fire`, `release`; new `tests/projection_room.rs`); its `forge.status.md` entry | `room-wait` | 60 |

Contracts between them: `notes/contracts.md`. Standing rules:
`notes/common.md`. Slice 1 is the part; `walk = false`.

## Critical path

`room-wait` (released with the first push), then `requeue-backstop`,
which reads the mark `room-wait` makes. Two tasks, one per crate, in
sequence: about two hours of implementer time.

## Overlaps with drafts beside this one

- `temper/crates/temper-engine-domain-forge/src/domain.rs`: `ui-reaches`
  (`decide-forge`) changed `step`'s `SetAway` and `Decided` arms, far from
  `project`, the `Domain` fields and the accessors this plan edits.
- `temper/crates/temper-engine-domain/src/route/forge_route.rs`:
  `ui-reaches` changed the `Decided` arm, far from `ProjectAfter`.
- `temper/docs/design/domain/forge.md`: `ui-reaches`, `ui-p3f-design` and
  `ui-p3e-words` change 5.5 to 5.9, 8.13 and 20; this plan's lines are in
  6, after its table, and 12.
- `temper/docs/design/domain/forge.status.md`: `ui-p3e-words` adds a
  line near 106 (the 4.6 entry) and `ui-reaches` removes 5.9 lines near
  380; this plan's two entries sit after the 12.1 paragraph (about 143)
  and after the 4.5 `DismissMaking` paragraph (about 170), away from
  both.
- `temper/crates/temper-engine-domain/src/domain.rs` and `src/tests.rs`:
  no draft beside this one changes them.
