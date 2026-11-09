+++
id = "worker-reads"
part = "worker-reads"
kind = "feature"
needs = ["engine-link", "worker-link"]
files = ["temper/crates/temper-service/src/worker.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/workers.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/lib.rs", "temper/crates/temper-service/src/tests/worker_admission.rs", "temper/crates/temper-service/src/tests/worker.rs", "temper/crates/temper-service/src/tests/workers.rs", "temper/crates/temper-service/src/tests/fixtures/worker_peers.rs", "temper/crates/temper-engine-shell/src/serve/host.rs", "temper/docs/design/domain/worker.status.md", "temper/docs/design/protocol/README.status.md", "jig/docs/design/domain/hosts.status.md", "temper/tests/one-process/src/review_delivery.rs"]
effort = "high"
minutes = 210
+++

## Goal

temper's worker reads its link and fires the link's deadlines on every
pass, whatever its held entrance holds, and its entrance holds the
engine's output cap, so new work waiting at the watermark never stops its
pings; and a link that drops is never silent on either side: each side
prints a line naming the worker and the `Loss`, and a failed workers'
listener prints when it fails (`temper/docs/design/domain/worker.md`, 3.1;
`temper/docs/design/protocol/README.md`, 15.3; jig's
`jig/docs/design/protocol/channel.md`, 9). This is fault 1 of the request
at temper's tier (the likeliest cause, plan.md, *Decided*) and the log
lines of faults 1 and 2. Failing story first.

Acceptance:

- The failing story (Steps, 1) is committed first, alone, and the report
  quotes its failure on the trunk.
- The worker routes its link's io and fires its link on every pass once
  its git is ready, whatever `engine` and `held` hold; a link step's whole
  output is still dispatched before the link steps again
  (`worker.md`, 3.1, *A link invocation's output precedes the next
  invocation*).
- `held` holds the engine's cap's frames
  (`jig_protocol_worker_wire::sizes::engine_cap(&sizes).frames`, the
  link's sizes) and three more for the link's own state; `HELD_ENTRANCE`
  goes. New work that finds it full faults the link
  (`link::Event::SourceFault`), and is dropped: the engine resends what it
  must after the next hello (jig's `protocol/channel.md`, 10.3).
- The lines of `temper/docs/design/protocol/README.md`, 15.3 print exactly
  as the table and its paragraph have them, each pinned by a row of the
  shell's line tests.
- Every existing test of `temper-service` and `temper-engine-shell` passes,
  but the one Steps, 3 rewrites to the new rule.
- The **Not yet** entries `3.1: the link is read, and its deadlines fired,
  only once ...` in `temper/docs/design/domain/worker.status.md`, `15.3: no
  loss line ...` in `temper/docs/design/protocol/README.status.md`, and
  `6.6: a loss is told without why on either side ...` in
  `jig/docs/design/domain/hosts.status.md` are removed, their lines alone.
  That departs from the implementer's text, which closes `docs/design/` to
  you, and from the one-tree rule for the jig entry: all three are this
  task's.

## Inputs

- `temper/docs/design/domain/worker.md`, 3.1: *The link is read and fired
  on every pass*, *The entrance holds the engine's cap*, and the orders
  that stay (*What passes a held assignment*, *A link invocation's output
  precedes the next invocation*).
- `temper/docs/design/protocol/README.md`, 15.3: the table's `lost` and
  `temper serve: worker ...` rows and *A link that drops is never silent*.
- jig's `jig/docs/design/protocol/channel.md`: 9 (*Reading never waits for
  the root*, *A lost link says why*), 10.1 (*A loss says why*), 10.2
  (*Every connection's end says why*), 10.3 (a domain output for a channel
  that is gone is dropped and resent after the next hello), 11 (the caps).
- `jig/docs/plan/worker-relinks/notes/contracts.md`: the engine's `Loss`
  and `Output::Ended` (section 1) and the worker's `Loss` in `Fact`
  (section 2), as `engine-link` and `worker-link` built them.
- `jig/docs/plan/worker-relinks/notes/common.md`.
- `jig/docs/plan/worker-relinks/inputs/request.md`, item 1.

## Owns and may change

- Owns the files in `files`.
- Reader files:
  - `temper/tests/real-loop/src/service.rs` and
    `temper/tests/playground/tests/suite/world.rs` read `Note` by
    `matches!`/`if let`: no edit expected; a compiler-forced arm only.
  - `temper/crates/temper/src/tests/help.rs` builds a `Note`: no edit
    expected.
  - Any other exhaustive match on `Note` the compiler finds: one arm.
- Pre-approved:
  - **The engine's notes path.** `Workers` may keep a small queue of
    notes that `Engine` drains into `engine.notes` where it drains its
    other notes; or `route` may take the notes queue. Yours to choose.
  - **`link_taken`'s other half** (the socket notes drained) may stay as a
    condition on stepping the link if dropping it would let socket notes
    outrun their bound; say which.
  - **The listener's line at once:** a `Note` pushed where `system.rs`
    sets `End::Stopped(Stop::Listener { .. })` for the workers' listener,
    printed with the exit line's words; the exit keeps its own line.
  - **If the engine's cap's frames are fewer than 192,** the entrance
    shrinks to them: the story of Steps, 1 and the existing
    `a_full_held_entrance_lets_a_passing_answer_pass` fill it by its new
    size, read from the worker, not a literal.
  - **If a story shows the engine sending more new work than its cap**
    without breaking its domain's bounds, that is a `Question:`, with the
    story pushed; never widen the entrance by a guess.
- Not yours: `jig/**` but the one status entry, `temper/docs/help/**`
  (`server.md`'s `linked` stays true), every figure (the watermark, the
  link's deadlines), `temper-engine-domain`.

## Steps

1. **The failing story, first, alone.** In
   `temper/crates/temper-service/src/tests/worker_admission.rs`:
   `a_worker_whose_engine_records_wait_keeps_its_link`. A socket stand and
   a `LinkPeer` opened on it (as
   `the_link_is_not_read_until_its_last_output_is_taken` does); the held
   entrance filled (as `a_full_held_entrance_lets_a_passing_answer_pass`
   fills it) and one more new record waiting in `stand.worker.engine`;
   then twenty seconds of the service's passes (`crate::iterate`) in steps
   of one second, the peer's engine component stepped on the same clock,
   both pinging when idle. Assert the peer's component never loses the
   channel (no `Lost`, no `Ended`), the worker's pings reached it, and the
   worker's link gave no `Fact`. On the trunk the worker neither reads nor
   pings, and the peer loses the channel at its silence deadline, 15 s.
   `LinkPeer` panics on a `Lost` today: record it instead, so the story
   fails on its assertion. Commit the story with the fixture's change
   alone, before the fix.
2. **The rule.** In `system.rs`, `route_events` no longer stops routing at
   a link whose output waits; in `worker.rs`, `iterate` fires the link once
   git is ready, whatever `engine` holds; `take_engine` moves the engine's
   records into `held` as now. `held` is made with the cap's room (Goal);
   a new record that finds it full faults the link and is dropped.
3. **The pinned test.** `the_link_is_not_read_until_its_last_output_is_taken`
   pins the old rule: rewrite it to the new one,
   `the_link_is_read_while_its_last_output_waits`: the same setup, the
   socket's `Closed` routed in the first pass. Keep its comment's citation
   right.
4. **The engine's lines.** `workers.rs`, `route`: `Output::Ended { worker,
   channel, why }` gives `Note::WorkerLost { worker, channel, why }` when
   `channel` is some, else `Note::WorkerOpeningEnded { worker, why }`,
   the latter once until that worker's `why` changes or its hello is heard
   (`Output::Hello`), keyed by the worker's name (`None` keys alone). In
   `temper-engine-shell/src/serve/host.rs`, `note_line` prints them as
   15.3's rows have them, the engine's `Loss` in 15.3's words (a
   `Refused { reason }` as `refused <reason>`, a `RefusedPeer` as
   `refused-by-peer <reason>`, `<reason>` by the existing `refusal`'s
   names, 2 as `terms`, 3 as `malformed`).
5. **The worker's line.** `worker.rs`, `link`: the `Fact { why }` that
   follows the `Lost` of a link that printed `linked` gives
   `Note::Lost { worker, address, why }`, once per such link; a `Fact`
   with no linked link before it gives no line (the refused lines stay as
   they are). `note_line` prints `lost <worker> <address> <why>`, the
   worker's `Loss` in 15.3's words (`Closed(Stream)` as `stream`,
   `Closed(Truncated)` as `truncated`, `Closed(Framing)` as `framing`,
   `Closed(Owner)` as `owner`, `Closed(RefusedHere(r))` as `refused
   <reason>`, `Closed(RefusedPeer(r))` as `refused-by-peer <reason>`, and
   `silence`, `stall`, `handshake`, `full`, `busy`, `fault`,
   `unreachable`).
6. **The listener's line,** as `Owns and may change` says.
7. **The line tests,** in `host.rs`'s tests beside
   `worker_link_notes_print_each_address_and_refusal_exactly`: one table
   per side, a row per `Loss` variant (and per reason name for the
   refusals), each the whole line exactly; and in
   `temper-service/src/tests/worker.rs` beside the refusal table: a linked
   link lost by silence gives one `Note::Lost`, a refused dial gives none,
   and in `tests/workers.rs`: an opening refused twice for the same reason
   gives one note, a third after a hello gives another. Break each rule
   once by hand (print the old silent path; drop the once-guard), see the
   rows fail for that reason, restore, and name them in the report.
8. **Runs.** `heavy cargo nextest run -p temper-service -E 'test(/worker|workers|system|hello_refused|stop/)'`,
   then `heavy cargo nextest run -p temper-service` whole, since the
   routing change reaches every io event of a worker;
   `heavy cargo nextest run -p temper-engine-shell --lib`;
   `heavy cargo check -p temper-real-loop-world --tests` and
   `heavy cargo check -p temper-playground-world --tests`. The one-process
   world composes a worker: `heavy cargo nextest run -p
   temper-one-process-world`. Check each run's count.
9. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-service -p temper-engine-shell
   --all-targets -- -D warnings`; remove the three status entries; commit
   saying why (a worker's link keeps its pings under load, and a link that
   drops says why on both sides); the citations check; push.

## Report

As the implementer's text has it, and: the story's failure on the trunk;
the entrance's size from the cap, and the cap's frames; whether
`link_taken`'s notes half stayed; the rules broken for each bite and the
failures seen; every reader file edited and how.

## Answers

2026-11-08. *Widen to the one-process recovery failure, or leave it to a
follow-up?* Widen: it is this branch's (green on the base and on
`engine-link`, red here, green with the old routing gate back; `notes/common.md`),
and the task is not done while
`one_process::a_fresh_goals_unencodable_review_releases_the_host_and_cancels`
is red. `temper/tests/one-process/src/review_delivery.rs` joins `files`
for observation only: diagnostics there are throwaway, and a committed
edit may only sharpen what it reports (its `overdue` message); its
stages, assertions, `DEADLINE`, `STEPS` and overrides stay as they are.
The fix goes in this task's product files, pinned first by a failing row
in `temper-service`'s `worker_orders` or `worker_admission`, committed
alone before the fix. Look first at the orders `worker.md`, 3.1 keeps:
reading the link every pass must not let what waits behind a held
assignment (a cancel or grant naming a run, the link's own state: ready,
lost, a grace passed) pass it; the recovery run stuck at attempt 6 reads
like a run started across such an overtaken record or the review's host
not released in order. If the cause proves to be the fixture relying on
an order the design does not promise, or a figure, stop with a
`Question:` and the evidence; never change a figure or what the story
expects.

2026-11-09. *Let the review-delivery fixture wake itself once its awaited
close is durable, or require a product order that gives it another pass?*
The fixture: the run's trace shows the product done at 5.39 s (every task
ended, the host released, nothing pending) and the judge never asked to
act on it. skein's `Referee::next_deadline`
(`skein/testing/skein-world/src/referee.rs`) is "the earliest of its
expectations' deadlines and of what it will act on": a judge whose
`progress` would queue a command must name that time, and this one names
only `DEADLINE`, so it leaned on extra passes the old routing gate
happened to leave. No product order changes, and no new product row is
owed. In `review_delivery.rs`, `Judge` keeps a wake: `idle` (or
`observe`) sets it to `now` when the current stage's condition in
`progress` holds (Cancel: change and review ended; Close: every task
ended; Recovery: the recovery task turned and idle; RecoveryClose: every
task ended), `act` clears it, and `next_deadline` is the earlier of the
wake and `DEADLINE` while not stopping. Read the conditions from one
place, so `progress` and the wake cannot disagree; clear the wake in
`act` so a met condition never spins the world at one instant. Stages,
assertions, `DEADLINE`, `STEPS`, overrides and every figure stay; the
sharpened `overdue` message stays. Acceptance is the one-process test
green with that change alone (and the runs of Steps, 8); say in the
report that the fix is the referee's wake, citing `referee.rs`. Other
referees of `temper/tests/one-process` with a fixed deadline only are
not this task's; name them under `## Left`.
