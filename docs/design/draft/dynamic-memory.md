# Dynamic memory inside, limits at the edges

Draft, 2026-10-28. A proposal to change how skein's services hold
memory: limits stay where a service meets its peers; inside it,
containers grow as they fill, the queues between its own components
have no capacity, and admission waits on a watermark instead of a worst
case proven at startup. It would change programming-model.md sections
1, 3, 6 and 7, and notes.md. Section 6 is a first step capped at six
hours (the stab), after which the user decides whether to go on. Read
it with [the programming model](../../foundation/programming-model.md)
and [the notes](../../foundation/notes.md).

## 1. In one page

- **Limits stay at the edges.** Message size, unparsed input, queued
  output per connection, connections, one request in flight, a
  product's quotas and the store's disk budget: what a peer or a person
  can push into a service stays limited, and is refused at the entrance
  (programming-model.md, 7).
- **Inside, memory is dynamic.** Containers grow as they fill, under a
  cap where one remains. The queues between a service's own components
  have none, so a push cannot fail and a decision always completes.
- **No room is reserved between components.** A parent no longer needs
  its children's largest output (`max_out`), and limits no longer have
  to satisfy inequalities with one another. Room first stays at the
  real sinks only: sockets, pipes and disk.
- **Admission by watermark.** A root admits new external work only
  while its backlog is under its watermark; whatever finishes work
  already admitted is always taken (section 4).
- **Large variable bytes are metered where they enter.** Transcripts,
  turns and documents held in memory draw on a byte budget at
  admission, as the store's data does on its stored-data limit today.
  `memory_bytes` becomes that budget, not a figure the worst case must
  fit (section 4.5).
- **The worst-case formula goes,** after the stab: no `worst_case` per
  part, no startup fit check, no exact memory figures. The counting
  allocator stays, as a check of peaks in fixed scenarios.
- **What stays:** the one loop, sans-io steps, slabs and their
  generational handles, the lifecycle, determinism and replay, owned
  bytes moved and never shared, and refusal at the entrance.

## 2. Why

Measured on 2026-10-28 on temper: the stock `temper init` deployment,
`small` profile, the engine and its worker in one process, idle; heap
by valgrind's massif, resident size from `/proc`.

- **The worst case binds nothing.** The engine's is 38.3 GB and the
  worker's 42.3 MB (306 GB under `standard`). temper init's template
  sets the engine's `memory_bytes` to 630 TB, so the startup check never
  refuses.
- **Resident is small; reserved is not.** 33.5 MiB is resident, against
  598 MiB of heap requested at startup, 585 MiB of it by
  `Queue::with_capacity`. One queue, jig-protocol-store's orphaned
  transcripts (`transcripts/drop.rs`), is sized `store.budget / 18`
  and reserves 458 MiB for entries that are usually a handful.
- **A reserved ring becomes resident.** A `VecDeque`'s head walks its
  whole buffer. A queue reserved for 15 million entries that never held
  more than two was 458 MiB resident after 10 million rounds of two
  pushes and two pops; one grown on demand stayed at 128 KiB. lib's
  `Queue` and `Intake` are rings, so a busy service drifts toward its
  whole reservation, and a long-lived connection toward its intake's.
- **Counting is cheap; propagation is not.** `worst_case` bodies are 1
  to 2% of each repository's hand-written code. The cost is in what the
  bounds drag along. temper-engine-domain's root sums every child's
  `max_out` (`route_bound`) and checks some twenty inequalities between
  limits before it computes its worst case. Limits are derived across
  repositories: the template's turn bounds come from smith's agent
  profiles and are regenerated. Internal capacities surface as product
  behaviour: ctl's "the engine holds its N answers of the last day;
  room again at TIME". In temper's history, 581 of 4,620 commits change
  `worst_case` code, and 133 regenerate or pin memory figures.
- **Services that do well with static memory have a narrow shape.**
  TigerBeetle allocates everything at startup, but has a few entity
  kinds, one message size and limits that are mostly compile-time
  constants, with the propagation in one sum and one file of
  compile-time assertions. temper has many entity kinds,
  variable-length payloads and limits that must agree across skein,
  smith, jig and temper. Services of that shape (ScyllaDB, Postgres)
  allocate dynamically and meter at admission.

## 3. What changes in skein

- **`Queue` and `Intake` grow on demand under their cap,** so they hold
  what they hold, not their reservation. `List`, `Stack` and `Slab`
  already cost what they hold (a `Slab` writes its slots at startup, but
  slabs are rare) and stay as they are.
- **An uncapped `Queue`** for the hops between a service's own
  components: its `push` cannot fail.
- **An infallible decision.** `Journal::decision` without a
  `JournalRoom`, and `Decision::write` and `hold` without a `Result`.
- **A backlog for the watermark:** `Journal::backlog`, the entries
  pending and held, beside `takes`.
- **The old forms stay beside the new** until every repository has
  moved (section 7), so skein's landing breaks no repository and each
  one bumps when it migrates.

## 4. The watermark rule

### 4.1 The backlog

A root's backlog is its journal's writes not yet committed, the outputs
it holds until their commit, and its own work queue, counted in
entries.

### 4.2 What waits at the entrance

New external work: a party's call, a worker's record that starts work,
a forge event, a timer that starts work, a setup or a restart. While
the backlog is at or past the watermark, admission hands the event back
as it does today (temper-engine-domain's `admission.rs`), the loop
offers it again, and peers wait as they do now: a worker's link
answered busy sends again.

### 4.3 What is always taken

Whatever finishes or advances work already admitted: operations'
completions, the store's answers (committed, loaded, unloaded), io's
results, and released deliveries. The backlog then always drains, and
admission cannot deadlock. temper's root gates released deliveries on
journal room today; under this rule it takes them.

### 4.4 Its value

One number per root, in its `Limits`, defaulting to what its journal's
limits hold today, so pacing stays close to now. It is a limit on
pacing, not on memory. A backlog can pass it by one admitted event's
work; nothing counts by how much.

### 4.5 Bytes

A domain counts the bytes it holds of variable-length data
(transcripts, turns, documents) against a budget and answers "full" at
the entrance when past it, as the store answers on its stored-data
limit (programming-model.md, 6.1). This is notes.md's byte budget, not
built in the stab.

## 5. What it gives up

- **A provable ceiling.** None binds today (section 2).
- **An amplification bug becomes an out-of-memory abort,** not a
  refusal: one event that makes unbounded internal work. Against it:
  each event's fan-out bounded in review, the simulator asserting
  queue peaks per scenario, and the resident size reported as a metric.
- **Exhaustion tests of internal containers at capacity 2.** The tests
  of the entrance limits stay.
- **Allocation-free hot paths,** already given up for bytes.

## 6. The stab

Six hours of wall time, beside the fleet's work, not a pause: one owner
and two or three implementers.

### 6.1 Holds for its six hours

- **skein-lib:** no other change.
- **The chosen root's files:** temper-engine-domain's `admission.rs`,
  `route.rs`, `domain.rs`, `decision.rs` and `limits.rs`; or smith's
  agent service, if a mission is changing temper-engine-domain (STATUS
  says).
- **configuration.md soft-paused:** the work on temper's
  `docs/design/protocol/configuration.md` waits, to cut the fleet's
  load and churn (the user, 10-28).

### 6.2 Steps

1. **Settle section 4** in this draft (`land --docs`).
   programming-model.md changes only on a go (section 7).
2. **lib, additively** (section 3), with its tests. lib's checks that
   a peak equals its worst case become at most for the growing
   containers; a new test holds a ring's resident bytes near what it
   holds. Land it; every repository stays green without a bump.
3. **The root.** temper bumps skein, and the root swaps its room check
   (`route_takes`, `takes`) for the watermark of section 4, keeping its
   scheduling's shape: the work queue, the store answers first, the
   held events. `route_bound`, `route_room` and the inequalities go.
   The root's worst case counts the journal at its watermark, an
   estimate until the formula goes (section 7). Tests that exhaust the
   journal's room are deleted or rewritten for the watermark. Land it.
4. **Measure** (6.3).

### 6.3 The report

The owner's closing note, for the user:

- lines deleted and added, code and tests, in lib and in the root;
- tests rewritten and tests deleted;
- the time each step took, the gates that failed and the rebase
  conflicts met during the window;
- the stock deployment's resident size after a rung A walk, before and
  after;
- whether rung A walks green on the migrated main;
- a recommendation: go on incrementally (section 7), or stop.

### 6.4 When it ends

- **Done:** lib landed with every repository green; the root migrated
  and landed, its scheduling's shape kept (a predicate swapped, not a
  restructure); rung A green; the report written.
- **Stopped:** the root needs its scheduling restructured beyond the
  predicate, a rung regresses, or six hours pass with the root
  unlanded. lib's additive change stays, since it breaks nothing and
  ends the ring's residency; the root's branch is kept unlanded beside
  the report.

## 7. After the stab, on the user's go

- **The rules:** programming-model.md sections 1, 3, 6 and 7, and
  notes.md, rewritten to this model, and this draft retired.
- **One crate a landing,** beside other work and around the crates
  active missions are changing: the roots first (temper's engine and
  worker, smith's agent service, jig's domains), then the rest.
- **A ratchet in each gate:** the count of the old forms' uses
  (`max_out`, `JournalRoom`, `takes`, `worst_case`) may only fall.
- **A final window, focused, of two to four hours:** the old forms
  leave skein and every repository bumps; the limit keys that go are
  accepted and ignored for a transition, since jig's documents are
  strict JSON; temper's memory figures, and land's regeneration of
  them, go; the rungs are walked again.
- **Later, as the product decides:** the refusals people see that come
  from internal capacities (ctl's retained answers, the archive
  deferred while read slots are full) become quotas, become time-based,
  or go.

## 8. Open questions

- **The watermark's unit:** entries or bytes, one per root or one per
  child.
- **hearth,** a narrow daemon, may keep the static model: the rule lets
  a service keep its caps.
- **jig's generated codecs** keep their per-type `worst_case` for now;
  whether decoding needs them once the sum goes.
- **The byte budget's shape** (notes.md, "A byte budget"): one budget,
  or one per consumer.
