+++
id = "worker-link"
part = "worker-link"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-worker-link/**", "jig/tests/worker-link/**"]
effort = "high"
minutes = 160
+++

## Goal

The worker's side of the channel keeps a healthy link however busy it is,
and says why each channel or dial ended. Its silence deadline counts bytes,
a frame's piece included, as the engine's already does; its `Fact` carries
a `Loss` that tells silence, stall, the handshake's deadline, a full cap,
the busy table, its root's fault and a failed dial apart; and the joined
world, both real components, shows a link busy both ways for minutes is
never lost (`jig/docs/design/protocol/channel.md`, 9 and 10.1). This is
fault 1 of the request ("the worker relinks about every 2 to 2.5
minutes") at the lowest tier holding both link layers, with its
candidates ruled in or out by rows, failing rows first.

Acceptance:

- The failing rows (Steps, 1) are committed first, alone, and the report
  quotes how each failed on the trunk, or says it passed there and so rules
  its candidate out (`notes/common.md`, *Candidates*), per candidate:
  - a deadline that fires on a busy but healthy link;
  - output queued past the stall deadline while frames still drain;
  - a frame arriving in pieces past the silence deadline.
- The worker restarts its silence deadline on every read of bytes in the
  open phase.
- `Loss` and `Request::Fact { why: Loss }` exist as
  `jig/docs/plan/worker-relinks/notes/contracts.md`, section 2 has them,
  each site giving the cause its table names.
- Every existing test of `jig-worker-link-world` and of the component
  passes, edited only where it names a `Fact`'s type.
- The **Not yet** entry `9: silence counts frames on the worker's side ...`
  in `jig/docs/design/protocol/channel.status.md` is removed, its lines
  alone. That departs from the implementer's text, which closes
  `docs/design/` to you: the entry is this task's.

## Inputs

- `jig/docs/design/protocol/channel.md`: 9 (*Pings*, *Silence counts
  bytes*, *A stall deadline*, *A lost link says why*), 10.1 (*A loss says
  why*, *Local refusal names*), 10.3, 12 (the worker's world's rows).
- skein's `skein/docs/design/channel.md`, 7: *Drained* goes up each time a
  queued frame is handed below; the stall deadline resets on drained.
- `jig/docs/plan/worker-relinks/notes/contracts.md`, sections 2 and 3.
- `jig/docs/plan/worker-relinks/notes/common.md`.
- `jig/docs/plan/worker-relinks/inputs/request.md`, item 1.

## Owns and may change

- Owns the files in `files`: the component and its world, the joined
  world (`src/joined.rs`, `tests/suite/joined.rs`) included.
- `engine-link` runs beside you and adds `engine::Request::Ended { .. }`
  to the two matches over the engine's root requests in
  `jig/tests/worker-link/src/joined.rs`. Leave those two matches as they
  are; add what you need elsewhere in the file.
- Reader files, edits of a `Fact`'s type only:
  - `temper/crates/temper-service/src/worker.rs`, the `link::Request::Fact
    { why }` arm: its patterns become `Loss::Closed(Closed::...)`, and every
    other `Loss` maps to `None`, as `Owner`, `Framing`, `Stream` and
    `Truncated` do now. `temper/crates/temper-service/src/tests/worker.rs`:
    the `Fact` values it builds, wrapped the same way. These cross into
    temper as a required field's lines do (`notes/contracts.md`,
    section 3); `worker-reads` adds the log line later.
  - `jig/testing/jig-test-service/src/worker.rs` matches `Fact { .. }`:
    no edit expected.
  - Any other reader the compiler finds: the type path only, named in the
    report.
- Pre-approved:
  - **The joined world may hold write grants a frame at a time** for a
    side (a `Pipe` setting, as the engine's world's `grant_one_frame`), and
    send records on a schedule; that is yours to add.
  - **If the busy-link row fails,** the cause it shows is yours to fix in
    `jig-protocol-worker-link`; a cause in `jig-protocol-engine-workers`
    (`engine-link`'s) or in skein is a `Question:`, with the failing row
    pushed and named.
  - **If every candidate row passes on the trunk** but the piece-by-piece
    one, fault 1's cause is above the components (temper's worker,
    `worker-reads`): say so; that is an answer, not a failure.
- Not yours: `jig/crates/jig-protocol-engine-workers/**`,
  `jig/tests/engine-workers/**`, `temper/**` but the lines above, any
  `docs/` but the one status entry.

## Steps

1. **The failing rows, first, alone.** Commit them before any fix and run
   each on the trunk's component.
   - `jig/tests/worker-link/tests/suite/liveness.rs`, beside
     `a_ping_restarts_the_silence_deadline`:
     `a_frame_arriving_in_pieces_keeps_the_link_past_the_silence_deadline`.
     `silence_engine()`; at 10 s the first eight bytes of a turn-kept
     frame's header and body arrive (`buffer_input`), the rest at 20 s,
     no ping between: no `Request::Lost` at 24.999 s; then nothing more,
     and the link is lost at 35 s, the silence deadline from the last
     byte. It fails on the trunk at 15 s.
   - `jig/tests/worker-link/tests/suite/joined.rs`:
     `a_link_busy_both_ways_for_minutes_is_never_lost`, a table whose rows
     are loads, each over ten simulated minutes in one-second steps with
     both real components joined:

     | row | the worker sends | the engine sends | write grants |
     |---|---|---|---|
     | steady | a turn each second | its acknowledgement | free |
     | trickle | a turn and two facts each second | its acknowledgement and a message | one frame at a time on both sides, so a queue is never empty and frames still drain |
     | engine burst | nothing but pings | forty records each minute, at once | one frame at a time on the worker's reading side |

     Each asserts no `Lost` from either component, no `Fact`, and that
     every record arrived, in order.
2. **Silence counts bytes.** In `component.rs`, `down`'s `Event::Lower` arm:
   a `LowerEvent::Read(stream::Up::Bytes(_))` while the phase is `Open`
   restarts `connection.silence` to the silence deadline from now, before
   the machine takes the bytes, as the engine's component does
   (`jig-protocol-engine-workers/src/component.rs`, `lower`). The other
   resets stay.
3. **The shapes**, as `notes/contracts.md`, section 2 has them: `Loss` in
   `boundary.rs`, exported from `lib.rs`; `Request::Fact { why: Loss }`;
   `lose` and `begin_loss` take a `Loss`; each site gives the cause its
   table names. The world keeps `facts: Vec<Loss>`.
4. **The loss rows,** in `liveness.rs` (or a new `losses.rs` module, with
   its `mod` line): one table, `each_loss_says_why`, a row per `Loss` the
   world can reach: silence, stall (as `an_engine_that_stops_reading_is_lost_by_stall`),
   the handshake's deadline (`silence_next_engine` before the opening),
   a full cap (as `a_nonfact_that_fills_the_cap_loses_the_link`), the busy
   table (as `busy.rs`'s fill, one past its room), its root's fault
   (`Event::SourceFault`), a failed dial (`refuse_dial`), a refusal received
   (`Loss::Closed(Closed::RefusedPeer(..))`) and a stream that ends
   (`lose_channel`, `Loss::Closed(Closed::Stream)`). Each asserts the
   world's facts end with exactly that `Loss`, once. Break the rule once by
   hand (give silence as `Closed(Stream)` again; drop the bytes reset of
   Steps 2), see the rows fail for that reason, restore, and name both in
   the report.
5. **The readers,** as `Owns and may change` says; then
   `heavy cargo check -p temper-service --tests` and
   `heavy cargo check -p jig-test-service --tests`.
6. **Runs.** `heavy cargo nextest run -p jig-protocol-worker-link`,
   `heavy cargo nextest run -p jig-worker-link-world`. This changes when the
   worker's link is lost, which its readers' tests see: also
   `heavy cargo nextest run -p jig-test-service`,
   `heavy cargo nextest run -p jig-channel-world` and
   `heavy cargo nextest run -p temper-service -E 'test(/worker/)'`. Check
   each run's count.
7. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-worker-link -p jig-worker-link-world -p temper-service
   --all-targets -- -D warnings`; remove the status entry; commit saying
   why (the worker keeps a healthy link however busy, and each loss says
   why); the citations check; push.

## Report

As the implementer's text has it, and: per candidate, ruled in or out and
by which row, with the trunk's failure quoted where it failed; the rules
broken for each bite and the failures seen; each reader file edited and
how.
