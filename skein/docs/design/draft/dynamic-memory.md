# Dynamic memory inside, limits at the edges

Draft, 2026-10-28, revised the same afternoon after two rounds of work.
How skein's services hold memory: limits stay where a service meets its
peers; inside it, containers grow as they fill, the queues between its
own components have no capacity, and admission waits on a watermark
instead of a worst case proven at startup. It changes
programming-model.md sections 1, 3, 6 and 7, testing-strategy.md's
memory checks, and notes.md. Sections 1 to 5 are the goal; section 6
says what two capped rounds proved; section 7 is the way to the end,
in parallel lanes; section 9 says when it is done. Read it with
[the programming model](../../foundation/programming-model.md) and
[the notes](../../foundation/notes.md).

## 1. In one page

- **Limits stay at the edges.** Message size, unparsed input, queued
  output per connection, connections, one request in flight, a
  product's quotas and the store's disk budget: what a peer or a person
  can push into a service stays limited, and is refused at the entrance
  (programming-model.md, 7). The kernel's queues and a host's channel
  are edges too.
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
- **Orders are stated, never implied by room.** A room check that also
  kept an order becomes a named predicate with a world test of that
  order under load (section 4.6).
- **Large variable bytes are metered where they enter.** Transcripts,
  turns and documents held in memory draw on a byte budget at
  admission, as the store's data does on its stored-data limit today.
  `memory_bytes` becomes that budget, not a figure the worst case must
  fit (section 4.5).
- **No process-wide worst case.** No per-process heap formula, no
  startup fit check, no exact memory figures or margins. Formulas that
  size an edge stay: a message's envelope, a decoder's retained bound, a
  record on disk. The counting allocator stays, as a ceiling on the
  peaks of fixed scenarios.
- **What stays:** the one loop, sans-io steps, slabs and their
  generational handles, the lifecycle, determinism and replay, owned
  bytes moved and never shared, and refusal at the entrance.

## 2. Why

Measured on 2026-10-28 on temper: the stock `temper init` deployment,
`small` profile, the engine and its worker in one process, idle; heap
by valgrind's massif, sizes from `/proc`.

- **The worst case binds nothing.** The engine's is 38.3 GB and the
  worker's 42.3 MB (306 GB under `standard`). The shipped declarations
  are hundreds of terabytes, so the startup check never refuses.
- **Resident is small; reserved is not.** 33.5 MiB is resident, against
  598 MiB of heap requested at startup, 585 MiB of it by
  `Queue::with_capacity`. One queue, jig-protocol-store's orphaned
  transcripts, reserved 458 MiB for entries that are usually a handful.
- **A reserved ring becomes resident.** A `VecDeque`'s head walks its
  whole buffer: a queue reserved for 15 million entries that never held
  more than two was 458 MiB resident after 10 million rounds of two
  pushes and two pops; one grown on demand stayed at 128 KiB. lib's
  `Queue` and `Intake` are rings, so a busy service drifts toward its
  whole reservation.
- **Counting is cheap; propagation is not.** `worst_case` bodies are 1
  to 2% of each repository's hand-written code. The cost is in what the
  bounds drag along: parents summing their children's `max_out`,
  inequalities between limits, limits derived across repositories,
  exact figures regenerated at every change of a type's size (in
  temper's history, 581 of 4,620 commits changed `worst_case` code and
  133 regenerated or pinned memory figures), and margins that turn a
  downstream repository red when an upstream one moves.
- **Static memory suits narrow services.** TigerBeetle allocates
  everything at startup, with few entity kinds, one message size and
  compile-time limits. temper has many entity kinds, variable-length
  payloads and limits that must agree across skein, smith, jig and
  temper; services of that shape allocate dynamically and meter at
  admission.

## 3. What changes in skein

- **Done:** `Queue::uncapped`, a queue whose `push` cannot fail;
  `Journal::uncapped`, `Journal::decide`, `Decision::push_write` and
  `push_held`, a decision that cannot fail; `Journal::backlog`, the
  commits made and not yet answered, the outputs held and those at the
  door. The old forms stay beside them until no repository uses them.
- **To come:** a named `Journal::stop` (a root stops by `failed(0)`
  today); then `Queue` and `Intake` grow on demand under their cap,
  once the exact accounting that prices their growth is gone (7.1, 7.5).
  `List`, `Stack` and `Slab` already cost what they hold and stay.
- **Last:** the old forms leave lib (`JournalRoom`, `takes`, capped
  decisions, `max_out` helpers), and with them every repository's last
  uses.

## 4. The watermark rule

### 4.1 The backlog

A root's backlog is its journal's writes not yet committed, the outputs
it holds until their commit, and its own work queue, counted in
entries: one per root, entries rather than bytes, which section 4.5
meters apart.

- **A root with a journal:** `Journal::backlog`, plus its work queue's
  and its store answers' lengths (temper's engine root).
- **A root without one:** the entries in the uncapped queues between
  its own children (smith's agent service: its inter-layer queues and
  its program's lower queue). The queues it shares with an edge, the
  host's channel or the kernel's, stay capped and outside the backlog.

### 4.2 What waits at the entrance

New external work: a party's call, a worker's record that starts work,
a forge event, a timer that starts work, a setup or a restart. While
the backlog is at or past the watermark, admission hands the event back
as it does today, the loop offers it again, and peers wait as they do
now: a worker's link answered busy sends again. An event that mixes new
and admitted work (a link coming up, a worker's or a forge's answer) is
split, so that only its new work waits.

### 4.3 What is always taken

Whatever finishes or advances work already admitted: operations'
completions, the store's answers (committed, loaded, unloaded), io's
results, and released deliveries. The backlog then always drains, and
admission cannot deadlock. What is taken still waits on a real sink: a
socket's or a destination's credit, the channel to the host, a file
table; never on room between two of the root's own children.

### 4.4 Its value

One number per root, in a `Limits` field of its own, defaulting to what
keeps pacing close to before: temper's engine root to its journal's
`commits` (3 at `small`), smith's agent service to its `queue`. It is a
limit on pacing, not on memory. A backlog can pass it by one admitted
event's work; nothing counts by how much. Where a person's reads wait
behind it, it rises (for temper's engine root, `commits` plus the
door's outputs).

### 4.5 Bytes

A domain counts the bytes it holds of variable-length data
(transcripts, turns, documents) against a budget and answers "full" at
the entrance when past it, as the store answers on its stored-data
limit (programming-model.md, 6.1). `memory_bytes` names that budget.
This is notes.md's byte budget, built after the roots move (7.4).

### 4.6 Orders

A room check can keep an order no one wrote down: in smith's agent
service, `room() >= max_out` on a queue whose capacity equalled
`max_out` kept one entrance's requests from being overtaken by the
next. So each root's move starts by listing what its room checks
remove, each a capacity or an order; each order becomes a named
predicate (`work.is_empty()`, `domain_requests_dispatched`) with a
world test of that order under load, written and green before the
swap.

## 5. What it gives up

- **A provable ceiling.** None binds today (section 2).
- **An amplification bug becomes an out-of-memory abort,** not a
  refusal: one event that makes unbounded internal work. Against it:
  each event's fan-out bounded in review, the simulator holding the
  peaks of fixed scenarios under ceilings, and the resident size
  reported as a metric.
- **Some pacing.** New work waits at the watermark while admitted work
  drains; in temper's worlds a person's watch waited a little longer
  (its longest wait 5 to 11 offers), and released continuations stopped
  waiting at all.
- **Exhaustion tests of internal containers at capacity 2.** The tests
  of the entrance limits stay.
- **Allocation-free hot paths,** already given up for bytes.

## 6. What two rounds proved

Round 1 (smith's agent service) and round 2 (jig-protocol-store's
orphan queue, then temper's engine root), 10-28, each capped:

- **A root's move is a predicate swap,** and it shrinks code: smith's
  agent service +124/−167 production lines in about two hours; temper's
  engine root +114/−194, then −57 more for its capped journal seam, with
  its swap itself 16 minutes. No test was deleted but those whose only
  subject was a removed inequality or reservation.
- **Uncapping a reservation is the memory win:** the orphan queue's
  move took the idle engine's heap from 615,432 to 146,700 kB (VmData,
  −457.7 MiB); resident size did not move, since the reservation was
  never touched.
- **Room checks keep orders** (4.6): one in smith, three explicit ones
  in temper's root, each now tested.
- **Exact accounting blocks growth.** Growing an existing ring broke
  the worlds' exact ownership accounting (a harness fills a ring outside
  the process's calls), and priced honestly it moved every downstream
  worst case past a margin, turning other repositories red. Growth
  waits for 7.1.
- **An upstream change that moves a downstream figure lands in one set
  with that downstream.**

## 7. The way there

Lanes that run at once, each landing alone unless it moves another
repository's figure, then in one set with it. The ratchet holds
throughout: in each repository's gate, the uses of the old forms
(`max_out`, `JournalRoom`, `.takes(&`, `worst_case`) may only fall, and
only the director raises one.

### 7.1 Exactness retires

What blocks growth and makes the churn. Per repository, each
independent where no downstream figure moves:

- **temper:** the exact memory-term files and their byte-for-byte
  reader become one ceiling per process with headroom; the
  declarations' band check and `work_estimate` go.
- **jig:** the startup-deficit pin and its growth file become a
  ceiling.
- **smith:** the margin floors and the growth ratchet become ceilings.
- **skein:** its saturation equalities become scenario ceilings, and
  the worlds learn to account for rings a harness fills, so that rings
  can grow: in one set with every downstream.

A ceiling is an upper bound with headroom on a measured peak or, until
7.6, on a formula; it moves by hand when a real change passes it, never
on a type's size.

### 7.2 The other roots

temper's worker, jig's domains with journals, smith's other services,
each as in 4.6: the order list, the order tests, the swap. Independent
of one another and of 7.1. hearth may keep its caps (8).

### 7.3 The remaining reservations

The idle engine still asks 143 MiB of heap it doesn't touch. Each
internal queue that reserves, found by its share of VmData, moves onto
`Queue::uncapped`: one crate a landing, independent.

### 7.4 The engine root's follow-ups

The watermark in its own `Limits` field (4.4); the mixed events split
(4.2); then the byte budget (4.5), first for transcripts and turns.

### 7.5 Rings grow

After 7.1's skein part: `Queue` and `Intake` grow under their cap.

### 7.6 The rules and the end

programming-model.md 1, 3, 6 and 7, testing-strategy.md's memory
checks and notes.md rewritten to this model, as soon as two or three
roots show its shape; this draft then retires. Last, in one focused
window: the old forms leave skein and every repository bumps; the
per-process formulas and the startup fit checks go, with the help pages
that promise an exact deficit; the limit keys that go are accepted and
ignored for a transition, since jig's documents are strict JSON; the
rungs are walked again.

## 8. Open questions

- **hearth,** a narrow daemon, may keep the static model: the rule lets
  a service keep its caps.
- **jig's generated codecs:** their wire-size formulas stay as edges;
  whether their per-type heap formulas are still needed once the
  process sum goes.
- **The byte budget's shape** (notes.md, "A byte budget"): one budget,
  or one per consumer.
- **The watermark's unit** stays entries until a root shows bytes
  matter.

## 9. Done when

- No repository uses the old forms but at an edge, and each ratchet's
  counts are zero.
- No exact memory figure, margin or growth pin; no startup fit check;
  scenario peaks under ceilings.
- `Queue` and `Intake` grow; the idle engine's VmData is what it holds.
- The rules say this model, and rung A walks green on the end's main.
