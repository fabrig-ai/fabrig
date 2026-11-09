+++
id = "temper-fits"
part = "temper-fits"
kind = "feature"
needs = ["fit-room"]
files = ["temper/crates/temper-service/src/tests/link_room.rs"]
effort = "high"
minutes = 60
+++

## Goal

On temper's stock settings, a producer resumed with every call its
settings admit committed since its last turn, each at the largest
answer, crosses the link to temper's worker, its newest answers whole
and the rest cut, on both profiles
(`temper/docs/design/domain/worker.md`, 4.1 *Transcript* and 4.3 *A
start the link cannot carry*). `fit-room` has fitted jig's link; this
task holds temper's composition to it, in `worker-link-quiet`'s stock
link story. This is the request's item 1, *the same holds on both stock
profiles*, at temper's tier.

`fit-room` merged only on a trunk holding `worker-link-quiet`, so your
base has its `temper/crates/temper-service/src/tests/link_room.rs`. If it
does not, report first line `Failed: base lacks worker-link-quiet's
temper-room (link_room.rs)`, changing nothing.

Acceptance:

- `a_stock_start_crosses_the_link_or_fails_with_its_numbers`'s rows
  hold the fit as Steps, 1 has it, on both profiles, and pass.
- Every existing test of `temper-service` passes.
- The **Not yet** entry `4.1, *Transcript*, and 4.3, *A start the link
  cannot carry*` in `temper/docs/design/domain/worker.status.md` is
  removed, its lines alone (`notes/common.md`, *Status lines*).

## Inputs

- `temper/docs/design/domain/worker.md`, 4.1 (*Transcript*) and 4.3 (*A
  start the link cannot carry*).
- `jig/docs/design/protocol/channel.md`, 9 (*A resumed start fits its
  room first*).
- `temper/docs/plan/resume-fits-room/notes/contracts.md`, sections 1, 2
  and 4.
- `temper/docs/plan/worker-link-quiet/tasks/temper-room.md`, Steps, 2:
  the story's table and what *crosses*, *past the room* and *refused
  before the link* mean there.
- `temper/docs/plan/resume-fits-room/notes/common.md`.

## Owns and may change

- Owns `link_room.rs`: the story's rows and their helpers.
- Neighbour files, as named:
  - `temper/crates/temper-service/src/tests/fixtures/worker_peers.rs`:
    an accessor for the assignment the worker's root was handed, if the
    joint gives none; its lines alone.
- Pre-approved:
  - **What the numbers give.** A row's outcome is the fit's arithmetic
    on the profile's figures (contracts section 2): correct each row to
    it and say so with the numbers. By this cut's reading, on temper's
    figures: the calls row crosses cut on both profiles; the all row
    crosses cut on `small` (its frame with every answer cut near
    444,951 bytes) and stays past the room on `standard`, by its inbox
    alone, its words naming the frame with every answer cut.
  - A start that goes quiet, on any row, is a red: stop with a
    `Question:`, the row and its log pushed.
- Not yours: jig, `workers.rs`, `lib.rs`, serve's lines, any `docs/`
  but the status entry, `temper/docs/help/**`.

## Steps

1. **The rows,** in `a_stock_start_crosses_the_link_or_fails_with_its_numbers`:
   - *calls* (`core.call_records` committed calls, each answer
     `core.call_answer_bytes`) crosses: the worker's root is handed
     every call by name and tool, in order, a newest run of answers
     whole and every older one `jig_protocol_worker_wire::fit::cut_words(len)`,
     the frame within the profile's `io.output`; no `Unsent`.
   - *all* as the numbers give (above); where it crosses, as *calls*;
     where it stays past, the run's refused answer with
     `assignment of N bytes past the link's room of R bytes`, `N` the
     frame with every answer cut, one `Unsent`.
   - every other row as it stands.
   Every row keeps the story's own checks: no loss either way, pings
   through the two minutes, and a small assignment after it crosses.
2. **Bite.** Break the rule once by hand (give the joint's engine
   component limits whose room is past the frame, so nothing is cut, and
   see the cut assertions fail; or assert the oldest answer whole) and
   restore; name it in the report.
3. **Runs.** `heavy cargo nextest run -p temper-service --lib -E
   'test(/link_room/)'`, then `heavy cargo nextest run -p temper-service
   --lib`. Check each run's count.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-service --all-targets -- -D warnings`; remove the
   status entry; commit saying why (on temper's stock settings a
   producer resumed with every committed call crosses the link, told of
   each); the citations check; push.

## Report

As the implementer's text has it, and: each row's outcome per profile,
with its calls, how many whole, and its frame's bytes against the room;
each row corrected and its numbers; the break and the failure seen;
the neighbour lines edited.

## Answers

2026-11-09. *Run 2 found the base lacks worker-link-quiet's `link_room.rs`;
what now?* The premise check did what it is for, and nothing changes: the
goal, the steps and the acceptance stand. This trunk took worker-link-quiet's
trunk at 17:15Z (`99ed26108e`); `temper-room`'s `link_room.rs` reached that
trunk only at 17:33Z (`85e9293560`), so this trunk does not hold it yet. Run
temper-fits again once this trunk holds it, whichever comes first: a merge of
`origin/worker-link-quiet/integration` at or after `85e9293560` into this
trunk, or worker-link-quiet landing and this trunk catching up with main.
Change nothing before then: do not take `link_room.rs` from temper-room's
branch or that plan's trunk, nor write the story afresh.

2026-11-09. *Run 3 found the base still lacks `link_room.rs`; what now?*
The answer above named the wrong commit: `85e9293560` merged
worker-link-quiet's `worker-room`, not `temper-room`. `temper-room`'s
`link_room.rs` is on its branch (`2b11d33350`, 17:01Z) and, at 17:44Z,
in neither `worker-link-quiet/integration` nor main. Otherwise the answer
stands: run temper-fits again once this trunk holds a commit of
`worker-link-quiet/integration` that contains `temper-room`'s merge
(`link_room.rs` present in the trunk's tree), by a merge of that trunk
into this one or by worker-link-quiet landing and this trunk catching up
with main. Until then change nothing, and do not take the file from
`temper-room`'s branch or write the story afresh.

2026-11-09. *Run 4 found the base still lacks `link_room.rs`; what now?*
Run 4 started at 17:47Z on `839bfc192c`, the trunk that took the run 3
answer, before the condition that answer set could hold: at 17:48Z
`temper-room` is still running in worker-link-quiet, unmerged into
`worker-link-quiet/integration` (`85e9293560`) or main. The run 3 answer
stands unchanged: run temper-fits again only once this trunk's tree holds
`temper/crates/temper-service/src/tests/link_room.rs` by a merge of a
`worker-link-quiet/integration` commit containing `temper-room`'s merge, or
by worker-link-quiet landing and this trunk catching up with main. A run
that finds it absent reports the same `Failed:` line and changes nothing.
