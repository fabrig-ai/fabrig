+++
id = "engine-room"
part = "engine-room"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-workers/**", "jig/tests/engine-workers/**", "jig/tests/worker-link/src/joined.rs", "jig/tests/worker-link/tests/suite/joined.rs"]
effort = "high"
minutes = 200
+++

## Goal

The engine's side of the worker's link never asks its stream for more
room than it grants. An assignment or a job whose frame is past the
stream's room fails its run with words naming its bytes and the room,
told once as `Unsent`; the link, its pings and the worker's other runs go
on (`jig/docs/design/protocol/channel.md`, 5.2 and 9). This is the
request's item 1 and 2 at the lowest tier holding both real link
components: the failing story first.

Today the channel asks io for room for the whole frame, io leaves an ask
past its output cap unanswered, the frame waits below for good, and while
it is queued the component arms no ping
(`jig/crates/jig-protocol-engine-workers/src/connection.rs`,
`next_deadline`), so the worker loses the link at its silence deadline
(`notes/common.md`, *What the plan is for*).

Acceptance:

- The failing rows (Steps, 1) are committed first, alone, and the report
  quotes how each failed on the trunk.
- `Limits.room`, `valid`'s new rules, `Unsent`, `Bound`,
  `Request::Unsent` and `host::Output::Unsent` exist as
  `temper/docs/plan/worker-link-quiet/notes/contracts.md`, section 2
  has them, and a record not sent takes section 2's steps, with its
  words.
- A job that cannot encode is answered `job cannot encode` as that
  section says, no longer discarded.
- Every existing test of the component, its world and the joined world
  passes, edited only where it builds `Limits` or matches the requests.
- The **Not yet** entry `5.2, 9, 11, 12: the engine's side has no room`
  in `jig/docs/design/protocol/channel.status.md` is removed, its lines
  alone (`notes/common.md`, *Status lines*).

## Inputs

- `jig/docs/design/protocol/channel.md`: 5.2 (*An assignment that cannot
  encode ends at the engine's component*, *A start the link cannot carry
  ends there too*), 9 (*Pings*, *A stall deadline*, *Every frame fits the
  stream below*, *A start past the room fails its run, never the link*),
  11 (*The component's own limits*, *The room is the stream's, not the
  domains'*), 12 (*A frame past the room, in the machine worlds*).
- `skein/docs/design/lib.md`, 7.1 (a Room's bytes fit the lower's cap);
  `skein/docs/design/channel.md`, 7 (*Checked at the entrance*, *Drained*).
- `temper/docs/plan/worker-link-quiet/notes/contracts.md`, sections 1
  and 2.
- `temper/docs/plan/worker-link-quiet/notes/common.md`.
- `temper/docs/plan/worker-link-quiet/inputs/request.md`, items 1 and 2.

## Owns and may change

- Owns the files in `files`: the component, its world, and the joined
  world's two files, which `worker-room` leaves to you but for one
  `room:` line in the worker's `Limits` literal of `src/joined.rs`, if it
  has one.
- Reader files, the lines named alone:
  - `temper/crates/temper-service/src/workers.rs`, `limits`: `room:
    jig.io.largest_room(),` in the `kit::Limits` literal; `route`: an arm
    `kit::host::Output::Unsent { .. } => {}`. `temper-room` gives that arm
    its note later (`notes/common.md`, *Cross-tree lines*).
  - `temper/crates/temper-service/src/tests/fixtures/worker_peers.rs`:
    `room:` in its `engine_link::Limits` literal, the worker's io's
    `largest_room()` from the settings it prepares (`own`'s or the
    worker settings' `jig.limits.io`); any match over the component's
    requests gets the arm the compiler names, doing nothing.
  - `temper/crates/temper-service/src/tests/workers.rs`: a match over
    requests or outputs, if the compiler names one: the arm alone.
  - `jig/testing/jig-test-service/src/limits.rs`, `workers`: `room:
    jig.io.largest_room(),`. `jig/testing/jig-test-service/src/workers.rs`:
    `Some(kit::host::Output::Unsent { .. })` beside `Ended`'s arm.
  - Any other literal or match the compiler names: the `room:` line
    (that site's io `largest_room()`, or in a world without io the room
    its scripted stream grants) or the arm alone, named in the report.
- Pre-approved:
  - **The worlds' scripted streams take a room.** `src/net.rs` of the
    engine's world and `src/joined.rs`'s `Pipe` gain a cap: an
    `OutputDown::Room` past it is never answered, as skein-io's
    (`skein/crates/skein-io/src/stream.rs`). Default: no cap, so every
    existing row behaves as it does.
  - **Row sizes.** If the world's `SIZES` admit no assignment whose frame
    passes every other kind's largest frame (so that a room between them
    exists for `valid`), raise the world fixture's `SIZES.charter` alone
    and say so; that is a row's input, not a figure.
  - **Measuring.** Measure the frame where you judge best (before or
    after encoding); a frame already encoded and refused is dropped
    whole.
  - **If a channel `Unsent` other than `Full` cannot be reached by a
    world row** (both sides refuse a terms gap at the opening, so it is a
    broken contract), cover it with a `--lib` test of the component
    driving the machine directly, or leave it unreached and say so.
- Not yours: `jig/crates/jig-protocol-worker-link/**` and the rest of
  `jig/tests/worker-link/**` (`worker-room`'s), skein, `temper/**` but
  the lines above, any `docs/` but the one status entry.

## Steps

1. **The failing rows, first, alone.** They set only the worlds' stream
   caps (pre-approved above), so they build on the trunk; commit them
   before any fix and run each on the trunk.
   - `jig/tests/engine-workers/tests/suite/liveness.rs`: one table,
     `a_start_past_the_room_is_answered_and_the_link_lives`. A worker
     linked and hello heard; the net's cap `ROOM`; then one record;
     then two minutes in one-second steps.

     | row | record | `ROOM` | expected |
     |---|---|---|---|
     | assignment at the room | an assignment | its frame's bytes | the assignment reaches the scripted worker whole |
     | assignment past the room | the same assignment | its frame's bytes less one | no assignment frame; the run's answer refused, invalid start, cause other, words `assignment of N bytes past the link's room of N-1 bytes`; one `Request::Unsent { unsent: Unsent { kind: Assignment, bytes: N, bound: Room(N-1) }, .. }` |
     | job at the room | a job | its frame's bytes | the job reaches the worker |
     | job past the room | the same job | its frame's bytes less one | no job frame; its `JobAnswer` refused, `JobInvalid::Start`, words with `job`; one `Unsent` |

     Every row: no `Request::Ended` and no `Request::Lost`, a `Ping`
     reaching the scripted worker at each ping interval through the two
     minutes, and an `acknowledge` sent after the record reaching the
     worker. On the trunk the past rows fail: no answer, no ping, the
     connection ended by its stall deadline at 30 s.
   - `jig/tests/worker-link/tests/suite/joined.rs`:
     `an_assignment_past_the_engines_room_keeps_the_link_and_fails_its_run`.
     Both real components joined; the engine's writing side capped at
     `ROOM`, an assignment's frame bytes less one; `engine_down` that
     assignment; two minutes in one-second steps; then a second, small
     assignment for another run. Asserts: neither side loses the link
     (no `Lost`, no `Fact`, no `Ended`), pings cross both ways through
     the two minutes, the engine's root hears the first run's refused
     answer and one `Unsent`, and the worker's root receives the second
     assignment and not the first. On the trunk it fails: the worker's
     link is lost by silence at 15 s.
2. **The shapes**, as `notes/contracts.md`, section 2: `room` in
   `limits.rs` and in `channel`'s inputs where it needs it; `valid`'s two
   rules; `Unsent` and `Bound` in `boundary.rs`, exported from `lib.rs`;
   `Request::Unsent`; `host::Output::Unsent`, mapped in `host.rs` as
   `Ended` is. The world fixtures' `LIMITS` take a `room` that keeps
   every existing row as it is (the default net has no cap, so any
   valid room past the world's largest start).
3. **Measure before the channel.** In `component.rs`, `down`/`send`: each
   record's frame is measured against `self.limits.room` before
   `ChannelRequest::Send`; past it, the record takes contracts section
   2's steps and the channel never sees it. A channel `Event::Unsent`
   other than `Full` in `heard` takes the same steps with
   `Bound::Channel(why)` for the record just sent (the send's events are
   answered within the same `down`; keep what you need to know which
   record it was, inside the component). `Full` stays a loss.
   `assignment_refused` takes its words; a job's refusal is its sibling.
   A job whose `translate::down` gives nothing is answered `job cannot
   encode`.
4. **The pings after.** A record not sent leaves nothing queued, so
   `next_deadline` arms the ping again; check the past rows see the
   first ping one interval after the record, not one interval after the
   last drain, and say which you found.
5. **More rows,** in the same table or the component's `--lib` tests, as
   you judge: limits whose largest message, relayed answer or grant frame
   passes the room are not valid (`valid` false; the host's
   `Workers::new` refuses them); a room below 270 is not valid. Break
   each rule once by hand (measure nothing; send a refused start
   anyway), see the rows fail for that reason, restore, and name both in
   the report.
6. **The readers,** as *Owns and may change* says; then `heavy cargo
   check -p temper-service --tests` and `heavy cargo check -p
   jig-test-service --tests`.
7. **Runs.** `heavy cargo nextest run -p jig-protocol-engine-workers`,
   `heavy cargo nextest run -p jig-engine-workers-world`, `heavy cargo
   nextest run -p jig-worker-link-world`. This changes what the engine's
   component does with a record it cannot send and what limits it takes,
   which its readers see: also `heavy cargo nextest run -p
   jig-test-service`, `heavy cargo nextest run -p jig-channel-world` and
   `heavy cargo nextest run -p temper-service -E 'test(/worker/)'`. Check
   each run's count.
8. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-engine-workers -p jig-engine-workers-world -p
   jig-worker-link-world -p jig-test-service -p temper-service
   --all-targets -- -D warnings`; remove the status entry; commit saying
   why (a start the link can't carry fails its run, never the link); the
   citations check; push.

## Report

As the implementer's text has it, and: each failing row's trunk failure
quoted; what step 4 found; the rules broken for each bite and the
failures seen; each reader file edited and how; the candidates of the
request's item 1 this task's rows rule in or out (the room: in; a frame
past the worker's terms: out, the opening's `terms_gap`).
