+++
id = "fit-room"
part = "fit-room"
kind = "feature"
needs = ["fit"]
files = ["jig/crates/jig-protocol-engine-workers/**", "jig/tests/engine-workers/**", "jig/tests/worker-link/src/joined.rs", "jig/tests/worker-link/tests/suite/joined.rs"]
effort = "high"
minutes = 120
+++

## Goal

The engine's link fits a resumed start's committed calls to its room
before measuring it, and is not made under limits whose smallest resumed
start passes the room (`jig/docs/design/protocol/channel.md`, 9, *A
resumed start fits its room first*; 11). On each stock profile, a
producer resumed with more committed calls at the largest answer than
one start can carry crosses the joined link and goes on. This is the
request's item 1, the failing story at the lowest tier holding both
real sides of the link with io's room, and item 2's fix in place.

## Premise: worker-link-quiet's engine-room

This task builds on `worker-link-quiet`'s `engine-room`
(`temper/docs/plan/worker-link-quiet/notes/contracts.md`, section 2),
which this trunk holds only once that plan has landed on main and the
trunk caught up (`notes/common.md`, *This plan's base*). Before anything
else, check your base: `jig/crates/jig-protocol-engine-workers/src/limits.rs`
has `pub room: u32` in `Limits`, the component refuses an assignment
past it (`assignment of BYTES bytes past the link's room of ROOM bytes`),
and `jig/tests/worker-link/src/joined.rs`'s pipe leaves a room past its
cap unanswered. If any is missing, change nothing, push nothing, and
report first line `Failed: base lacks worker-link-quiet's engine-room
(<what is missing>); run again once this trunk has caught up with a main
holding worker-link-quiet`. That departs from the implementer's text:
it is no question, the rerun is the owner's.

Acceptance:

- The failing rows (Steps, 1) are committed first, alone, and the
  report quotes how each failed on the base.
- An assignment is fitted as contracts section 3 has it, and `valid`
  holds its new rule.
- The rows of Steps, 1 pass, each bitten (Steps, 4).
- Every existing test of `jig-protocol-engine-workers`, its world, the
  worker-link world and `jig-test-service` passes; a row whose
  assignment passed the room by its committed calls alone is corrected
  as *Owns and may change* says.
- The **Not yet** entries `9, 11, 12, *A resumed start fits its room
  first*` in `jig/docs/design/protocol/channel.status.md` and `7.2,
  *Resuming*` in `jig/docs/design/domain/engine.status.md` are removed,
  their lines alone (`notes/common.md`, *Status lines*).

## Inputs

- `jig/docs/design/protocol/channel.md`, 5.2 (*A start the link cannot
  carry ends there too*), 9 (*A start past the room fails its run, never
  the link*, *A resumed start fits its room first*), 11 (*The room is
  the stream's, not the domains'*), 12 (*A frame past the room, in the
  machine worlds*; *A resumed start fitted to its room*).
- `jig/docs/design/domain/engine.md`, 7.2 (*Resuming*).
- `temper/docs/plan/resume-fits-room/notes/contracts.md`, sections 1, 2
  and 3; `temper/docs/plan/worker-link-quiet/notes/contracts.md`,
  section 2.
- `temper/docs/plan/resume-fits-room/notes/common.md`.

## Owns and may change

- Owns the files in `files`: the fit's call and `valid`'s rule in the
  component, the engine world's rows, the joined world's story.
- Neighbour files, as named:
  - `jig/tests/worker-link/Cargo.toml`, `jig/tests/engine-workers/Cargo.toml`:
    a dependency line, if a row needs a crate the world lacks.
  - `jig/testing/jig-test-service/src/limits.rs`: none expected; if its
    engine `workers` limits fail the new rule, a question, since that
    would be a stock figure.
- Pre-approved:
  - **Where the fit sits.** Default: in the path from `deliver` to the
    room's measure, on the translated `wire::Assignment`, with the
    component's `wire::v9::Limits` and `limits.room`; a job never. If
    `engine-room` measures the encoded frame rather than the record, fit
    the record before it is encoded and say so.
  - **Fixtures below the new rule.** A test's own limits whose room is
    below their smallest resumed start take a room at it: the row's
    input, not a figure; name each in the report.
  - **A row that passed the room by its committed calls** (in the engine
    world's room table, or `engine-room`'s joined
    `a_start_past_the_room_is_answered_and_the_link_lives`) now fits:
    make it pass the room by its charter instead, keeping what it
    asserts, and say so with its numbers.
  - **Stock limits in the joined world.** The engine's from
    `jig_test_service::limits::for_profile(p, &jig_protocol_config::profile(p)).workers`;
    the worker's as the world's fixtures build them, with the profile's
    sizes and the room its io grants. If `jig-test-service` gives the
    worker's link limits publicly, take those and say so.
- Not yours: `jig-protocol-worker-wire` (call `fit` as it is; a change
  it needs is a question), temper, skein, any `docs/` but the two status
  entries.

## Steps

1. **The failing rows,** committed alone first:
   - **The engine's world,** rows in the room table `engine-room`
     wrote (`jig/tests/engine-workers`, whatever its name on your base;
     channel.md, 12, *A frame past the room, in the machine worlds*):
     an assignment past the room by its committed calls alone, at a
     row's own small room, is sent, its newest answers whole and the
     rest `cut_words(len)`, its frame within the room, no `Unsent`; and
     limits whose smallest resumed start passes their room are not made
     (`valid` false, `Component::new` refused).
   - **The joined world's story,** a new table in
     `jig/tests/worker-link/tests/suite/joined.rs`,
     `a_resumed_start_fits_the_room_on_each_stock_profile`, its rows run
     on `Small` and `Standard` limits:

     | row | the assignment | expected |
     |---|---|---|
     | few calls | a resumed window of two turns and three committed host calls at the profile's `core.call_answer_bytes` | crosses whole |
     | past the room by its calls | the same window and as many committed calls at `core.call_answer_bytes` as the room holds whole, plus two, with a delivery among the oldest | crosses cut |

     *Crosses whole:* the worker's root is handed the run's assignment
     with every committed call as sent. *Crosses cut:* it is handed
     every call, by name and tool, in order, the delivery whole, a
     newest run of host answers whole and every older one
     `cut_words(len)`, and the frame within the room. Every row: the
     run's root then sends a turn and an answer up, each kept by the
     engine's side; no `Unsent`, no loss on either side for two minutes
     in one-second steps, pings both ways.
2. **The fit.** In the component, as contracts section 3 has it:
   `Whole` and `Cut` send the returned record; `Past` takes the room's
   refusal as it is, its words naming `Past`'s bytes.
3. **The floor.** `valid(&Limits)` also holds `smallest_resumed(...)` is
   `Some` and at most `room`.
4. **Bite.** Break each rule once by hand, alone: skip the fit (the
   past row is refused with its bytes and the room); drop the floor (the
   limits row is made). See the rows fail for that reason, restore, and
   name both in the report.
5. **Runs.** `heavy cargo nextest run -p jig-engine-workers-world --test
   suite -E 'test(/room/)'`; `heavy cargo nextest run -p
   jig-worker-link-world --test suite -E 'test(/joined/)'`; then `heavy
   cargo nextest run -p jig-protocol-engine-workers`, `-p
   jig-engine-workers-world`, `-p jig-worker-link-world` and `-p
   jig-test-service`. The fit changes what the component sends, so its
   readers' tests run too: `heavy cargo nextest run -p temper-service
   --lib`; `temper-service`'s `link_room` rows for committed calls are
   `temper-fits`', and fail here as the numbers give: list them under
   `## Left`. Check each run's count.
6. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo
   clippy -p jig-protocol-engine-workers -p jig-engine-workers-world -p
   jig-worker-link-world --all-targets -- -D warnings`; `heavy cargo
   check -p temper-service --tests`; remove the two status entries;
   commit saying why (a resumed producer past the room by its committed
   calls now crosses, told every call, rather than failing each time it
   resumes); the citations check; push.

## Report

As the implementer's text has it, and: each joined row's outcome per
profile, with its calls, how many whole, and its frame's bytes against
the room; each fixture whose room was raised to its smallest resumed
start; each row corrected and its numbers; the breaks of Steps, 4 and
the failures seen; under `## Left`, every `temper-service` row the fit
turns red, by name.

## Answers

2026-11-09. *Run 2 found the base lacks worker-link-quiet's engine-room;
what now?* The premise check did what it is for, and nothing changes:
the skeleton, the steps and the acceptance stand. On 2026-11-09
worker-link-quiet's `engine-room` sits on that plan's trunk, but the
plan has not landed on main (`worker-room` parked, `temper-room`
waiting), so this trunk cannot hold it yet. Do not take engine-room's
code from its branch or trunk, nor build `Limits.room`, the refusal or
the pipe's cap here: they are that plan's. Each run checks the premise
first, as *Premise* says; while any part is missing, it fails with the
same first line, changing and pushing nothing.

2026-11-09, again. *Run 3 found the base still lacks worker-link-quiet's
engine-room; what now?* The same answer: nothing changes. main still
lacks `Limits.room` (worker-link-quiet: `engine-room` on its trunk,
`worker-room` parked, `temper-room` waiting), so this trunk cannot hold
it. The rerun waits for that plan to land and this trunk to catch up;
until then each run fails at the premise, changing and pushing nothing.

2026-11-09, a third time. *Run 4 found the base still lacks
worker-link-quiet's engine-room; what now?* Nothing changes. At
13:04Z main (`22e30f0c5e`) still lacks `Limits.room`; worker-link-quiet
has `engine-room` on its trunk, `worker-room` parked and `temper-room`
waiting. The rerun waits for that plan to land and this trunk to catch
up; until then each run fails at the premise, changing and pushing
nothing.

2026-11-09, a fourth time. *Run 5 found the base still lacks
worker-link-quiet's engine-room; what now?* Nothing changes. At 13:11Z
main (`22e30f0c5e`) still lacks `Limits.room`; worker-link-quiet has
`engine-room` on its trunk, `worker-room` parked and `temper-room`
waiting. The rerun waits for that plan to land and this trunk to catch
up; until then each run fails at the premise, changing and pushing
nothing.
