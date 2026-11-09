+++
id = "engine-link"
part = "engine-link"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-workers/**", "jig/tests/engine-workers/**", "jig/tests/channel/**", "jig/testing/jig-channel-kit/**"]
effort = "high"
minutes = 200
+++

## Goal

The engine's side of the worker channel never keeps a lost connection, and
says why each one ended. A worker lost fifty times, each way a link is lost,
is taken back each time; a closing connection whose channel never ends is
aborted below at its deadline; the workers' part closes within the stall
deadline of a close whatever its workers do; and every accepted connection's
end goes up once as `Request::Ended` with its `Loss`
(`jig/docs/design/protocol/channel.md`, 9 and 10.2). This is fault 2 of the
request ("the engine stopped taking the worker back") and the workers'
part of fault 3 ("didn't stop on SIGTERM"), with the failing rows first.

Acceptance:

- The failing rows (Steps, 1) are committed first, alone, and the report
  quotes how each failed on the trunk, or says it passed there and so rules
  its candidate out (`notes/common.md`, *Candidates*).
- `Loss`, `Request::Ended` and `host::Output::Ended` exist as
  `jig/docs/plan/worker-relinks/notes/contracts.md`, section 1 has them,
  `Ended` exactly once per accepted connection, a connection turned away
  for room included, with the first cause as its `why`.
- A connection that leaves its open phase holds the stall deadline from
  then; past it, not yet terminal, the component asks the host to abort its
  socket (`Request::Abort`), whether or not its channel has ended.
- In `jig-engine-workers-world`: the fifty-loss table, the closing rows and
  the `Ended` rows pass; every existing test passes unchanged but for the
  compiler-forced edits named below.
- In `jig-channel-world`: the fifty-cut story passes.
- The **Not yet** entry `9, 10.2: no connection's end says why ...` in
  `jig/docs/design/protocol/channel.status.md` is removed, its lines alone.
  That departs from the implementer's text, which closes `docs/design/` to
  you: the entry is this task's.

## Inputs

- `jig/docs/design/protocol/channel.md`: 9 (*A lost link says why*), 10.2
  (*Every connection's end says why*, *Closing is bounded*, *Shutting
  down*), 10.3 (the connection's states), 4.1 and 4.2 (the opening's checks,
  jig's reasons, a replaced channel), 12 (the machine world's rows and the
  protocol world's fifty-cut story).
- `jig/docs/design/domain/hosts.md`, 6.6 (*A worker comes back as often as
  it is lost*).
- `jig/docs/design/protocol/service.md`, 11.3 (the host's pull and its
  outputs).
- `jig/docs/plan/worker-relinks/notes/contracts.md`, sections 1 and 3.
- `jig/docs/plan/worker-relinks/notes/common.md`.
- `jig/docs/plan/worker-relinks/inputs/request.md`, item 2.

## Owns and may change

- Owns the files in `files`: the component, its host and its worlds.
- Reader files, compiler-forced edits only, one arm each:
  - `jig/testing/jig-test-service/src/workers.rs`: `Output::Ended { .. } => {}`
    (the reference service keeps no log).
  - `jig/tests/worker-link/src/joined.rs` (owned by `worker-link`, which runs
    beside you): `engine::Request::Ended { .. }` joins the arms that drop or
    keep the engine's root requests at its two matches; nothing else there.
  - `temper/crates/temper-service/src/workers.rs`, `route`: the arm
    `kit::host::Output::Ended { .. } => {}`; `worker-reads` turns it into a
    log line. `temper/crates/temper-service/src/tests/fixtures/worker_peers.rs`:
    `engine_link::Request::Ended { .. }` beside `Lost { .. }`. These cross
    into temper as a required field's lines do
    (`notes/contracts.md`, section 3).
  - Any other exhaustive match the compiler finds on `Request` or `Output`:
    one arm, named in the report.
- Pre-approved:
  - **`Ended` stays out of the world's root list.** The engine-workers world
    collects `Request::Ended` apart (`World::ended()`), so tests that count
    `world.root()` or assert its exact sequence (`cascades.rs`, `liveness.rs`,
    `records.rs`, `referee.rs`) keep their meaning unchanged.
  - **If no world input keeps a channel from ending after its loss** (the
    machine ends promptly whatever io does), say so in the report with what
    you tried; keep the row as a guard, add the closing deadline anyway, and
    remove the entry.
  - **If the fifty-loss table passes on the trunk for every mode**, fault 2
    is not in this component: say so, keep the table as a guard, and make
    the channel world's story the one that looks for it through real io.
  - **If the channel world's fifty-cut story runs over the focused guide**
    (1 s, `skein/docs/foundation/testing-strategy.md`, 8), find why first
    (8.1, question 2); if it is the cuts' own cost, cut the story to the
    fewest cuts that pass the old connection's closing deadline twice, and
    report both times. Never move it to another suite.
  - **A fix the rows show outside this crate** (skein's `channel` or `io`,
    the fleet) is a `Question:`, with the failing row pushed and named.
- Not yours: `jig/crates/jig-protocol-worker-link/**` and
  `jig/tests/worker-link/**` but the arms above (`worker-link`'s),
  `temper/**` but the arms above, any `docs/` but the one status entry.

## Steps

1. **The failing rows, first, alone.** Commit them before any fix, on the
   trunk's component, and run each; quote each failure in the report.
   - In `jig/tests/engine-workers/tests/suite/`, a new module `relinks.rs`
     (with its `mod` line in `main.rs`): one table test,
     `a_worker_lost_fifty_times_each_way_is_taken_back_each_time`. A row is
     a way of losing the link; each row makes a world, then fifty times:
     `reporting(WORKER_A, SECRET_A)`, lose the link that way, `run`, and
     advance to the loss's own deadline plus the stall deadline. Then it
     asserts that every cycle's hello reached the root
     (`Request::Hello` count 50), that each lost connection is `gone`, and
     that the root's `Lost` count is the hellos' less the last open one.
     The rows:

     | row | how the link is lost |
     |---|---|
     | silence | the worker sends nothing past the silence deadline |
     | stall | `withhold_write_room`, one record queued (`Event::TurnBusy`), past the stall deadline |
     | held send | a record handed below whose send is never settled, past the stall deadline (the world gains a way to hold a send below, Steps 3) |
     | stream failed | `fail_stream` |
     | stream ended | `end_stream` |
     | not admitted | `Event::NotAdmitted` for its channel |
     | replaced | the next cycle's dial comes while this channel is still open |
     | framing | `inject` a frame header that does not parse |

   - In `stopping.rs`, a row beside
     `a_closing_channel_is_lost_when_its_output_stalls`:
     `a_close_with_a_send_held_below_closes_within_the_stall_deadline`:
     a worker whose record was handed below and never settled,
     `Event::Close`; at the stall deadline less a millisecond it is not
     gone, at the deadline it is, and the component reports `Closed` once.
   - In `jig/tests/channel/tests/suite/channel.rs`, the protocol world's
     story `a_worker_cut_fifty_times_is_taken_back_each_time`: a new
     `Story::Cuts { times: u32 }` in `jig/testing/jig-channel-kit`, whose
     fault hook injects a reset on the worker's link after each hello
     (as `Story::Reconnect`'s does once), fifty times, no run hosted; the
     outcome counts the engine's hellos and the cuts, and the test asserts
     `Verdict::Passed`, fifty cuts and fifty-one hellos admitted by the
     fleet. Name its time from nextest in the report.

2. **The shapes**, as `notes/contracts.md`, section 1 has them: `Loss` and
   `Request::Ended` in `boundary.rs` (its `Debug` shows the variant and
   counts, as `Request`'s does), exported from `lib.rs`; `Output::Ended` in
   `host.rs`, the worker's configured name filled in from the component's
   table. The component records the first cause on the connection
   (`Connection` gains it, and whether `Ended` went up) at each site that
   decides one: `refuse` (its reason), `fire`'s loss branch (`Silence`,
   `Stall` or `Deadline`, whichever deadline passed), `heard`'s
   `Ended`/`Refused`/`Unsent::Full`/`OutputFailed` arm, `closed` (its
   `Closed`: `RefusedHere(r)` is `Refused { reason: r }`, `RefusedPeer(r)`
   is `RefusedPeer`, `Framing` is `Framing`, `Stream` and `Truncated` are
   `Stream`, `Owner` is `Stopping` when the component is closing, else
   `Stream`), the shutdown in `fire` (`Stopping`), `accepted`'s turn-away
   (`NoRoom`, or `Stopping` once closed) and `Event::Gone` with none
   recorded (`Stream`). `Ended` goes up at the first cause, once.
   `Event::Gone` removes the connection as now.

3. **The closing deadline.** Every transition to `Phase::Closing` that is
   not terminal sets `deadline` to the stall deadline from now (today
   `fire`'s loss branch clears it). In `fire`, a closing connection past
   its deadline that is not yet terminal pushes `Request::Abort` for its
   connection and becomes terminal, and gives `Ended` if it has not. The
   channel's own `Closed`, if it comes later, adds nothing (`closed` keeps
   its `terminal` guard). The world's held send: a `Net` setting that hands
   a `Send` below and never answers its settlement, nor its cancel; its
   `Abort` ends the stream as the world's `Abort` does today.

4. **The `Ended` rows,** in `relinks.rs`: a second table,
   `each_connections_end_says_why_once`, whose rows reach each `Loss` the
   world can reach (each row of Steps 1, plus: an unknown worker,
   `Refused { reason: Reason::Unauthorized.code() }`; a full component,
   `NoRoom`; a close, `Stopping`; the hello's deadline, `Deadline`), and
   assert `world.ended()` holds exactly one `Ended` for the connection with
   that `why`, and its `worker` and `channel` (`None` before
   authentication and before the hello). Break the rule once by hand (give
   `Ended` twice; drop the first-cause guard), see the rows fail for that
   reason, restore, and name both in the report.

5. **The readers,** as `Owns and may change` says; then
   `heavy cargo check -p jig-test-service --tests`,
   `heavy cargo check -p temper-service --tests` and
   `heavy cargo check -p jig-worker-link-world --tests`.

6. **Runs.** `heavy cargo nextest run -p jig-protocol-engine-workers`,
   `heavy cargo nextest run -p jig-engine-workers-world`,
   `heavy cargo nextest run -p jig-channel-world`,
   `heavy cargo nextest run -p jig-test-service`. This changes when a
   closing connection's socket is aborted, which its readers' tests see:
   also `heavy cargo nextest run -p jig-worker-link-world --test suite -E 'test(/joined/)'`
   and `heavy cargo nextest run -p temper-service -E 'test(/worker/)'`.
   Check each run's count.

7. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-engine-workers -p jig-engine-workers-world -p
   jig-channel-world -p jig-channel-kit -p jig-test-service --all-targets
   -- -D warnings`; remove the status entry; commit saying why (a worker's
   lost connection never holds its room or the engine's stop, and each
   end says why); the citations check; push.

## Report

As the implementer's text has it, and: each failing row's failure on the
trunk, or that it passed there; per mode of the fifty-loss table, whether
it reproduced fault 2; what the channel world's story found and its time;
the rules broken for each bite and the failures seen; each reader file
edited and how.

## Answers

2026-11-08. **May `Story::Cuts` alone use a longer simulated completion
deadline, or should its cut count fall to fit 300 seconds?** The longer
deadline, for `Story::Cuts` alone; keep fifty cuts. The 300 seconds in
`jig/testing/jig-channel-kit/src/referee.rs` is the referee's story horizon
in simulated time, not a product figure: *No figure changes* covers the
profiles' intervals, deadlines, graces and limits, every fixture's
`limits()`, and the CPU budgets and wall limits, and none of those moves.
Fifty cuts is the design's (`jig/docs/design/protocol/channel.md`, 12), and
immediate cuts never reach the worker's `proved` interval, so its redial
backoff climbs to its `ceiling` by the configured profile, which the run
measured (18 hellos by 300 s). So:

- Put the horizon in one place, a referee method used by `next_deadline`,
  `overdue` and `observe_engine` in place of the three literals: 300 s for
  every other story, unchanged; for `Cuts { times }`, 300 s plus `times`
  times the worker link's configured redial `ceiling` (read from the
  profile the world runs, not a literal), with checked arithmetic.
- The story's CPU time still answers to the focused guide (1 s): if it
  passes it, the pre-approved rule in *Owns and may change* holds as
  written (find why; then the fewest cuts that pass the old connection's
  closing deadline twice). Name its nextest time in the report.
- If the story still fails within the longer horizon, that is the finding
  the story exists for (fault 2 through real io): report how far it got.
