+++
id = "temper-room"
part = "temper-room"
kind = "feature"
needs = ["engine-room", "worker-room"]
files = ["temper/crates/temper-service/src/tests/link_room.rs", "temper/crates/temper-service/src/tests/fixtures/worker_peers.rs", "temper/crates/temper-service/src/workers.rs", "temper/crates/temper-service/src/lib.rs", "temper/crates/temper-engine-shell/src/serve/host.rs"]
effort = "high"
minutes = 180
+++

## Goal

On temper's stock settings, a run's start carries the largest charter
(its instructions and specification), transcript window, tree, inbox
and committed calls the settings admit, and one past each: each start
either crosses the link to temper's worker or fails with its reason
named, and the link stays up with its pings both ways. Never silence.
A record the engine's link does not send is a `worker unsent` line on
serve's stderr (`temper/docs/design/domain/worker.md`, 4.3, *A start
the link cannot carry*; `temper/docs/design/protocol/README.md`, 15.3).
This is the request's item 1 at temper's tier, on the stock settings,
and its item 2's line.

`engine-room` has given jig's component its room and its refusals, and
`worker-room` the worker's: this task holds temper's composition to
them.

Acceptance:

- `Note::WorkerUnsent` exists as
  `temper/docs/plan/worker-link-quiet/notes/contracts.md`, section 4
  has it; `Workers::route` pushes it once for each
  `kit::host::Output::Unsent`; serve prints it as section 4's line.
- The story of Steps, 2 passes on both stock profiles, its rows as the
  table there has them.
- A run refused past the room reaches serve's refused-run line once,
  its words whole (`refused_answer_reason_reaches_operator_once`, a row).
- Every existing test of `temper-service` and `temper-engine-shell`
  passes.
- The **Not yet** entries `4.3, *A start the link cannot carry*` in
  `temper/docs/design/domain/worker.status.md` and `15.3: the worker
  unsent line` in `temper/docs/design/protocol/README.status.md` are
  removed, their lines alone (`notes/common.md`, *Status lines*).

## Inputs

- `temper/docs/design/domain/worker.md`, 4.3 (*A start the link cannot
  carry*, and the operator line's paragraph after the list).
- `temper/docs/design/protocol/README.md`, 15.3 (the lines' table, its
  `worker unsent` row, and *A link that drops is never silent*).
- `temper/docs/design/protocol/channel.md`, 7 (*An assignment that
  cannot encode*) and 8 (temper's largest parts).
- `jig/docs/design/protocol/channel.md`, 5.2 and 9 (*Every frame fits
  the stream below*, *A start past the room fails its run, never the
  link*).
- `temper/docs/plan/worker-link-quiet/notes/contracts.md`, sections 1,
  2 and 4.
- `temper/docs/plan/worker-link-quiet/notes/common.md`.
- `temper/docs/plan/worker-link-quiet/inputs/request.md`, items 1 and 2.

## Owns and may change

- Owns the files in `files`: the new story's module, the joint fixture
  `worker_peers.rs`, `Workers::route`'s `Unsent` arm, the `Note`
  variant, serve's line.
- Neighbour files, as named:
  - `temper/crates/temper-service/src/tests.rs`: one line, `mod
    link_room;`, after `mod workspace_reuse;`, the file's last module.
  - `temper/crates/temper-service/src/tests/workers.rs`: a row in
    `refused_answer_reason_reaches_operator_once` (Steps, 4), and a
    test beside the `Note::WorkerLost` ones for the `Unsent` route.
  - Any other match over `temper_service::Note` the compiler names: its
    arm alone, in the form the neighbouring `WorkerLost` arm takes.
  - Other tests that build `WorkerPeers`: call sites only, if its
    constructor changes.
- Pre-approved:
  - **The joint.** Extend `WorkerPeers` (`fixtures/worker_peers.rs`), the
    real temper `Worker` on its socket joined to jig's real engine
    component: a constructor taking the engine's `kit::Limits` (the
    stock engine's, from `crate::workers::limits`), its byte pipes sized
    to the room, and the engine's writing side answering a `Room` past
    the engine's room never, as skein-io does. Default: as it is now.
  - **Stock settings.** A profile's settings are `fixtures::engine()`
    and the worker's fixture settings with `limits::profile(p)` and
    `jig_protocol_config::profile(p)` for their limits. If a fixture
    does not take a profile's limits whole, take the parts the link
    reads (`core`, `hub`, `link`, `io`) and say so.
  - **A start refused before the link** (one past a part's bound, which
    temper's or jig's encoding refuses) is answered as now, by
    `Workers::assignment` or by jig's component; the row asserts that
    answer's words.
  - **If a stock row's expected outcome is wrong** by the numbers (a part
    alone past the room that the table says crosses, or the reverse),
    the table's expectation is this cut's arithmetic: correct the row to
    what the numbers give, say so with the numbers, and keep going. A
    start that goes quiet, on any row, is a red: stop with a
    `Question:`, the row and the log pushed.
- Not yours: `jig/**`, skein, any `docs/` but the two status entries,
  `temper/docs/help/**`.

## Steps

1. **The line.** `Note::WorkerUnsent` in `lib.rs`, beside `WorkerLost`;
   `Workers::route`'s arm for `kit::host::Output::Unsent { worker,
   channel, unsent }` pushes it once, where `engine-room` left an arm
   doing nothing. In `serve/host.rs`, its line as contracts section 4
   has it, with one exhaustive match over
   `jig_protocol_worker_wire::Kind` for `<record>` and one over
   `skein_channel::Unsent` for `<reason>`; rows in the file's line table
   beside the `worker lost` ones: a past-the-room assignment
   (`temper serve: worker unsent worker=worker-a channel=7: assignment
   of 1300000 bytes past the room of 1048576 bytes`), a refused message
   (`... message of 4200 bytes refused: too-large`). The route's test in
   `tests/workers.rs`.
2. **The story,** `src/tests/link_room.rs`: one table,
   `a_stock_start_crosses_the_link_or_fails_with_its_numbers`, its rows
   run on each stock profile, `Small` and `Standard`. Each row builds an
   engine `temper_engine_domain::Assignment` and encodes it with
   temper's engine (`Workers::encoded_assignment`, or
   `Workers::assignment` for a start refused before the link), hands the
   event to the joint's engine component on a linked channel, then runs
   the joint for two minutes in one-second steps.

   | row | the assignment | expected |
   |---|---|---|
   | charter | its charter at the most the settings admit (`crate::assignment::charter_bound` of `hub.charter_bytes` and the agent's): the instructions and specification it carries | crosses |
   | window | a resumed window of `hub.transcript_bytes` | crosses |
   | tree | workspace items at the engine's largest `Items` | crosses |
   | inbox | `tasks.run_inbox` messages of `tasks.message_bytes` | crosses |
   | calls | `core.call_records` committed calls, each answer `core.call_answer_bytes` | past the room |
   | all | every part above at once | past the room |
   | one past, each | each of the first five rows with one byte more in its part | refused before the link, its words |

   *Crosses:* the worker's link hands its root that run's assignment, and
   no `Unsent`. *Past the room:* no assignment reaches the worker; the
   engine's root hears the run's answer refused, invalid start, its
   words `assignment of N bytes past the link's room of R bytes` with
   `R` the profile's `io.output` and `N` past it; one `Unsent`.
   *Refused before the link:* the run's refused answer, from temper's
   encoding (`Workers::assignment`) or from jig's component when its own
   translation fails (`assignment cannot encode`), whichever meets the
   part first; the row asserts its words and that no frame was sent.
   Every row: neither side loses the link (no `Lost`, `Fact` or
   `Ended`), pings cross both ways through the two minutes, and a small
   assignment for another run sent after it crosses. The calls row is
   the request's deployment, by reading: 32 answers of 69,632 bytes on
   small and of 135,168 on standard are past 1,048,576 alone.
3. **The stock makes both links:** in the same module, a test that each
   profile's engine `crate::workers::limits` and worker
   `worker_settings::link_limits` are `Some` and valid with their rooms,
   and that the largest frame of each kind but the starts is within the
   room, printing nothing.
4. **The run's line:** a row in `refused_answer_reason_reaches_operator_once`
   (`tests/workers.rs`): `Invalid::Start { cause: Other, why:
   b"assignment of 1300000 bytes past the link's room of 1048576 bytes" }`.
5. Break each rule once by hand (route no note; give the joint's engine
   component an unbounded room so the past row goes quiet), see the rows
   fail for that reason, restore, and name both in the report.
6. **Runs.** `heavy cargo nextest run -p temper-service --lib -E
   'test(/link_room/)'`, then `heavy cargo nextest run -p temper-service`
   and `heavy cargo nextest run -p temper-engine-shell`. Check each run's
   count.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-service -p temper-engine-shell --all-targets -- -D
   warnings`; remove the two status entries; commit saying why (a start
   temper's stock settings admit past the link's room fails with its
   numbers, and serve says so); the citations check; push.

## Report

As the implementer's text has it, and: each row's outcome per profile,
with the frame's bytes and the room where it was sent or refused; any
row whose expectation you corrected and the numbers; the rules broken
for each bite and the failures seen; each neighbour file edited and how.

## Answers

2026-11-09. *May charter+1 cross after the brief's fitting, with a separate
fixed-charter overflow row?* Yes, as the run recommended; fitting stays as it
is. `temper/docs/design/domain/agent.md`, 4.3, says no run is refused for its
brief's size: the brief is fitted, whole trailing sections omitted with their
line. So the table's `charter + 1` (one byte more in a brief section, the
specification) is *crosses*: the worker's root gets the assignment with the
fitted charter, its frame within the bound, the omitted section named by the
`[N sections omitted: ...]` line, no `Unsent`, and the link's checks as every
row. Add one row, `charter fixed + 1`: the fixed part (instructions, with an
empty brief) one byte past `charter_bound`, which `fitted_charter` returns
`None` for; it is *refused before the link* with `assignment cannot encode`
and no frame sent. Both rows are in `link_room.rs`; no fitting code, design or
other file changes. The other four `+ 1` rows stay as Steps, 2 has them.
