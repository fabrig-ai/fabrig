# resume-fits-room: the plan

A run resumed after a loss carries every call committed since its last
turn, each with its answer (`jig/docs/design/domain/engine.md`, 7.2). A
producer on a Codex program works an hour inside one turn and commits
many; on both stock profiles 32 committed calls at the largest answer
pass the link's 1 MiB room alone (small: 32 × 69,774 framed bytes;
standard admits 128 of 135,310). With `worker-link-quiet` such a start
fails by name; this plan makes it fit (`inputs/request.md`). The
evidence is outside the repository.

## The fix, chosen

**The engine's link cuts a resumed start's committed calls to its
room** (`jig/docs/design/protocol/channel.md`, 9, *A resumed start fits
its room first*). Every committed call stays, by its name and tool; from
the newest back, host answers stay whole while the frame fits; the rest
become the words `[this call ran; its answer of BYTES bytes was left out
of this start]`; a delivery's answer is never cut, since the worker
checks its evidence. Only a start still past the room with every answer
cut is refused, as `worker-link-quiet` refuses it.

Why this candidate, of the request's three:

- **The newest calls that fit, with the rest left out** would leave the
  program inconsistent: a call left out is one whose turn was never
  told, so the LLM never learns it ran, and makes it again (a second
  delegation, a second note). Cutting its answer instead tells the run
  every call it made, as smith's waking prompt tells any answered call
  (smith's `domain/run.md`, 6), and a program resuming its own session
  takes the same words as user messages (smith's `protocol/programs.md`,
  5.8). A read it still needs, it reads again.
- **Calls in pieces after the start** is a wire change: a new record
  kind, a protocol change, which the request makes a question first. It
  is not needed: the cut start fits.
- **A smaller bound on what one call record keeps** would cut the
  core's kept answers, which are also what a relink replays to a live
  run, for every run, to save room only resumed starts need; and it is a
  figure, which the request keeps out.

**At the link, not in the core.** Only the engine's link knows the whole
frame: the charter's bytes are made by its protocol layer, and the core
knows no wire sizes. Fitting per start, exactly, keeps every answer
whole that fits; a worst-case bound in the core would cut every resumed
start's answers on `standard`, whose inbox alone admits 21 MB. The core
keeps every call as committed; the engine's own hub, which crosses no
link, carries them all.

**The floor.** The smallest resumed start, every call `core.call_records`
admits cut beside an assignment's fixed fields, must fit the room:
jig's engine component is not made otherwise, and temper names the
bound at start (`temper/docs/design/protocol/configuration.md`, 7):
`engine.limits.io.output: N bytes, short of the smallest resumed start,
S bytes`. It is 6,817 bytes on `small` and 27,201 on `standard`, and no
document reaches it today (`io` takes no override; the wire's ceilings
keep it at most 218,177 bytes): it holds the rule if either changes.

**The story's tier.** The lowest tier holding both real sides of the
link with io's room is jig's joined world (`jig/tests/worker-link`),
whose pipe `worker-link-quiet`'s `engine-room` caps as io does; on each
stock profile's limits a resumed start with more committed calls at the
largest answer than one start carries crosses there, the worker's root
is handed every call, and its run's turn and answer go up. temper's
stock link story (`worker-link-quiet`'s `link_room.rs`) then holds the
same on temper's settings. jig's channel world holds the core too, but
runs on overridden small limits with no stock-profile selector and no
loss before a turn; giving it both would be the plan's largest work
and its costliest test, for what the core world already holds (the core
carries every call committed since the last turn, in commit order:
`settled_answers_resume_in_commit_order_and_a_replay_preserves_exact_bytes`).

## This plan's base, and what fab cannot hold

`worker-link-quiet` is this plan's base: its tasks give the engine's
link its room and refusal (`engine-room`), cap the joined world's pipe,
and write temper's stock link story (`temper-room`). None has merged.
This trunk holds that plan's design, and its code only once it lands
on main and this trunk catches up; fab holds no task for another
plan's.

So the tasks split by what they build on:

- `fit` and `named-floor` need none of that code and go first.
- `fit-room` needs `engine-room`'s code and `temper-fits` needs
  `temper-room`'s. Each checks its premise first; on a base without it,
  `fit-room` stops at once with `Failed:`, changing nothing, to be run
  again once this trunk has caught up with a main holding
  `worker-link-quiet`. The owner, temper/migration, holds that rerun.

## Parts and tasks

| Part | Task | Needs | Tree | Request item |
|---|---|---|---|---|
| `fit` | `tasks/fit.md` | | jig | 2 (the fit and the floor's measure) |
| `named-floor` | `tasks/named-floor.md` | `fit` | temper | 2 (the configuration refused, naming the bound) |
| `fit-room` | `tasks/fit-room.md` | `fit`; premise `worker-link-quiet`'s `engine-room` | jig | 1 (the failing story, joined, both stock profiles), 2 (the fit in the link, the floor in `valid`) |
| `temper-fits` | `tasks/temper-fits.md` | `fit-room` | temper | 1 (temper's stock settings, both profiles) |

Four tasks, the request's bound. Item 3, the design, is this cut's own
writing. Nothing is dropped.

## Design written by this cut

- `jig/docs/design/protocol/channel.md`: 5.2, a sentence in *A start the
  link cannot carry ends there too*; 9, *A resumed start fits its room
  first*; 11, the sizes' bullet and *The room is the stream's, not the
  domains'*; 12, *A resumed start fitted to its room*. Its status: two
  **Not yet** entries, 11 (`fit`'s) and 9, 11, 12 (`fit-room`'s).
- `jig/docs/design/domain/engine.md`, 7.2, *Resuming*: the sentence on a
  start that crosses a worker's link. Its status: a **Not yet** 7.2
  (`fit-room`'s).
- `temper/docs/design/domain/worker.md`, 4.1 *Transcript* and 4.3 *A
  start the link cannot carry*. Its status: a **Not yet** 4.1, 4.3
  (`temper-fits`').
- `temper/docs/design/protocol/configuration.md`, 7, the channel's
  checks: the smallest resumed start within `io.output`. Its status: a
  **Not yet** 7 (`named-floor`'s).

## Critical path

`fit` (60 min) runs now; `named-floor` (40) follows it, beside
`fit-room`. `fit-room` (120) can run only once `worker-link-quiet` has
landed and this trunk caught up, so the path is `fit`, then that wait,
then `fit-room` and `temper-fits` (60): about 240 implementer-minutes
beside the wait, which is `worker-link-quiet`'s own path (its
`engine-room` and `temper-room`, about 380 minutes, and its landing).

## Overlaps not avoided

- `fit-room` owns `worker-link-quiet`'s `engine-room` files
  (`jig/crates/jig-protocol-engine-workers/**`, the engine world, the
  joined world's two files) and `temper-fits` its `temper-room` story
  (`link_room.rs`, with an accessor in `fixtures/worker_peers.rs`), each
  only after that plan has landed: no two writers at once.
- `named-floor` adds a test to `temper/crates/temper-service/src/tests/checks.rs`,
  under `ui-ask-m4`'s `see-withheld` glob (`src/tests/**`), and one call
  line in `startup.rs` after `checks::hub`.
- `jig/docs/design/domain/engine.status.md`: this plan's 7.2 entry sits
  before the 17.5 entry, away from the 7.3 entries other plans keep.
- `jig/docs/design/protocol/channel.status.md` and
  `temper/docs/design/domain/worker.status.md` carry `worker-link-quiet`'s
  entries too: this plan's entries sit apart from them, between entries
  neither plan removes.
