# jig's design: the rails

## 1. The rails

### 1.1 The barrier

A child domain decides immediately. Its state changes as it decides, and
it emits its writes and its outward requests at once. It never waits for
durability; durability is its parent's job. For each call to its entry
point, the root does these things:

1. **Admission:** before any child changes, it checks that there is room
   for one more commit. If there is not, it answers busy.
2. **Gathering:** every write from every child the event touched goes
   into one numbered commit.
3. **Holding:** every output that someone outside could act on is tagged
   with that commit. That covers an effect, an assignment, an
   acknowledgement, and an answer to a person or to a tool call. The
   output is held until the store says the commit is durable.
4. **Releasing:** held outputs leave in their order. After a failed
   commit, nothing leaves, and the engine stops.

The order of a restart, and the authority check on every effect, belong
to the same set of duties.

Together, these make "once" and "nothing ahead of its commit" true.
Connectors are the application's children, so their writes and outputs
pass through the application's root, and that root is on this path
whatever jig does. Its mistakes pass every test of expected behaviour and
show only when the engine crashes:

- **An output that leaves early:** an effect made before the commit of
  its outbox entry, made but never recorded;
- **A decision split in two:** a call's record in one commit and its
  outbox entry in the next;
- **Admission after a change:** a connector changed, then no room found
  to commit what it changed;
- **Restart out of order:** the core deciding before a connector has
  restored its outbox, and asking again for an effect already in flight;
- **An effect that skips authority:** a procedure's effect reaching its
  connector's outbox without the core's check.

The rails below make each of these hard to write, loud when written, and
caught by a test.

### 1.2 The journal, in skein-lib

The barrier cannot live in the core: the core may not hold the
application's records and outputs, since its code may not be generic.
The barrier is not policy either. It is a data structure, as a queue is.
So it goes into skein-lib, generic over the writes and outputs it holds,
as lib's containers are generic over what they hold. skein/docs/foundation/programming-model.md,
10.2 allows exactly this: lib may use generic types, and application code
does not hand-roll data structures.

- **What it does,** in short (`domain/root.md`, section 4, has it whole):
  it admits a decision by counts (commits in flight, writes per commit,
  outputs held); it gathers one decision's writes and outputs into a
  value that makes at most one numbered commit; it tags each output with
  the commit it follows, or the last commit made if the decision made
  none, and releases outputs in order as the store answers, nothing
  after a failed commit; and its worst case is priced like every lib
  container's.
- **What it does not do:** inspect what it holds. Whoever owns the bytes
  checks byte bounds before handing them over. The deployment's record
  and numbers are the core's.
- **It keeps to the subset:** generic types and lib's own containers, no
  traits defined, no closures. Its design is a section of skein's
  `runtime/lib.md`.

### 1.3 One shape for every root

The root's entry points have no outbound queue of their own. What leaves
the domain is taken from the journal by the service's `iterate`, so the
journal is the only way out. In outline (`domain/root.md`, section 3,
gives the shape whole):

```
step(root, event):
    journal.takes(room for this event)?  else: busy     ← before any child changes
    d = journal.decision()
    route the event through the children: each write and each outward request goes into d
    journal.accept(d)                                   → at most one commit out
store answers n → journal.committed(n) → the outputs held for n leave, in order
```

Outputs that decide nothing and show nothing durable leave through one
explicit door of the journal, untagged, so review finds every use. These
are, among others, a run's text streamed to watchers as it is written,
and answers to a run's read tools (`domain/engine/vocabulary.md`, 1.2). What parties read and watch shows state, so it is
held like any other output (`domain/engine/decisions.md`, 1.2;
`client/domain/contract/reads.md`, section 1, and `client/domain/contract/watches.md`, section 1).

### 1.4 Ordering belongs to the core

- **Restart is a script** (`domain/engine/restart.md`; `domain/root.md`,
  section 8). The core asks the root for one step at a time:
  1. the core's own live ranges, loaded;
  2. connector `k` restored: the root asks the connector for its live
     ranges, loads them and feeds them back;
  3. runs adopted, as workers dial in;
  4. connectors reading afresh what their systems own of the live work;
  5. the outbox settled: every entry that may have been made is looked
     for by its key before it is made again;
  6. decisions open.

  The root does the step it is asked for, and the core asks for the next
  one only after hearing that this one is done.
- **Effects follow one loop** (`domain/engine/vocabulary.md`, 1.4;
  `domain/connectors-effects.md`, section 1):
  1. a connector proposes an effect, from a procedure's step or from an
     agent's call;
  2. the core admits it against authority;
  3. the connector records the outbox entry, in the same decision;
  4. the connector makes the effect only when the journal releases it.
- **Write holds:** the core keeps holds and writer slots, in tasks
  (`domain/tasks/holds.md`). A connector tells the core when it has
  read a resource afresh after a lost attempt.

### 1.5 Mistakes are loud

- **Worst cases are declared.** Each child declares the most it writes
  and outputs per event, and the root adds them up along the route
  (skein/docs/foundation/programming-model.md, 4.5).
- **The journal asserts.** A decision that exceeds what it reserved stops
  the engine (fail-stop), rather than leaving state ahead of the store.

### 1.6 Mistakes are caught: the conformance world

- **What jig ships:**
  - the fakes for its own peers: the store on skein-kv's in-memory mode,
    scripted workers and scripted people;
  - the referee for `domain/model.md`'s promises;
  - the faults: crashes at every commit, commits held, commits failed.
- **What the application adds:** its whole domain and the fakes of its
  systems.
- **Generic in the world, concrete in the domain.** A world is ordinary
  Rust (skein/docs/foundation/programming-model.md, 10.2), so the conformance world may be
  generic over the application. The application's domain stays concrete.
- **The referee looks from outside.** Before a fake system saw any
  effect, that effect's record was durable. No run was assigned without
  a durable claim. No person was answered ahead of their commit. Every
  effect was made as its recovery class allows.

### 1.7 A reference root

jig's example application, `ops` (`examples.md`), is a small system for
running services in production. It is chosen because it shares none of
the shapes of work of an application that writes code:
- two connectors, observability and infrastructure;
- a stream of events, mostly noise;
- effects on shared and scarce resources that cost money;
- its agents inside its engine.

It is complete: a root, its connectors, their fake, a client and worlds.
Its root is the template every application copies. With
exhaustive matches and one shape (1.3), roots look alike, and agents fill
in their arms.

### 1.8 What is left to the application

Which child an event goes to, and the small total functions between
vocabularies. A mistake there is an ordinary bug in the application, and
its own stories find it. It cannot break jig's promises silently,
because every write and every output still passes through the journal,
and the conformance world crashes the application at every commit.
