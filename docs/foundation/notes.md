# Notes

Provisional, 2026-10-03, revised 2026-10-28. What belongs with
programming-model.md and testing-strategy.md but is not a rule: the
questions still open, why the memory strategy is what it is, and why facts
are never lost inside a process.

## Open questions

- **Affine owner handles.** An `Owned<T>`, neither `Copy` nor `Clone`,
  returned by `insert` and consumed by `retire`, would let only an
  entity's owner end it, checked by the compiler. It is awkward for
  top-level entities (who holds the `Owned<Conn>` of an accepted
  connection?). Decide after the first service.
- **Kinds in tokens.** Nothing but the record variant stops a layer from
  decoding a token as the wrong kind (programming-model.md, 4.2). If that
  turns out to be a real mistake, give `Token` a kind tag that
  `from_token` checks.
- **Moving review-only rules into checks.** Some rules are held in review
  only (programming-model.md, 10.3):
  - a small `syn`-based checker could hold no `async`, closures, `loop`
    or `while`, tuple scrutinees, or trait and generic definitions in
    step code. Without closures, `dyn` or traits there, every call is
    static, so it could find recursion too;
  - lints could hold two more: `unused_results`, or `#[must_use]` on
    lib's functions, for an ignored `Option`; and `BTreeMap` and
    `BTreeSet` among the step crates' disallowed types.

  Write the checker if review proves not to be enough.
- **The byte budget's shape** (programming-model.md, 6.1): one budget for
  every large consumer of bytes in a process, or one per consumer, each
  kept in the layer that owns it. io could also grant output room within
  a global queued-output budget, so that pressure turns into the ordinary
  backpressure chain rather than a new kind of refusal.
- **The watermark's unit** (programming-model.md, 7.1) stays entries until
  a root shows that bytes matter.
- **A slow fact sink.** With no room reserved, facts wait in their queue
  while a backpressuring sink is slow, and nothing counts them against
  their root's watermark. Whether that queue joins the backlog, should a
  trace ever fall behind.
- **Narrow services may keep their caps.** A daemon with few entity kinds
  and one message size (hearth) can keep the static model; the rule lets
  a service keep a cap where it has one.
- **Generated codecs' heap formulas.** Their wire-size formulas stay as
  edges; whether their per-type heap formulas are still needed once no
  process sums them.
- **Operations the kernel cannot interrupt.** A process's end is bounded
  by io's deadlines (programming-model.md, 5.2), but a cancelled
  operation keeps its entity settling until the kernel completes it, and
  a write to a filesystem in uninterruptible sleep (a hard-mounted NFS
  whose server went away) may never complete. The process then cannot
  end, and its supervisor's kill waits on the same kernel. Whether to
  refuse roots on such filesystems at startup, or only to say that they
  hold a process's end.
- **Who names the last word.** The teardown invariant starts at the last
  word (testing-strategy.md, 6), which a scenario's referee names today.
  A service could mark its own, as a fact the harness reads, so that
  every scenario checks the same moment without saying so. Decide when a
  second service's worlds use the invariant.
- **A deterministic TLS.** A test-only crypto provider that draws from the
  seed would let TLS join the replaying tiers (testing-strategy.md, 4.4).
  Whether rustls's unbuffered connection allows it, and whether it tests
  enough of the real provider to be worth it.
- **Where transcripts come from** (testing-strategy.md, 4.1), and how they
  are kept current when a peer's protocol changes.

## Why facts are never lost inside

The trace queue was an open question: where the queue lives, and what
happens when its sink falls behind. It is one of the step's output
queues, with no cap like the others, so emitting a fact cannot fail
(programming-model.md, section 3). Each sink downstream then drops and
counts, or backpressures through its stream, by its declared policy.

- **Loss belongs at the edge.** A queue that dropped inside the process
  would lose facts before any sink could say so, and a reader could not
  tell a quiet run from a lossy one. Keeping every fact until a sink has
  decided means the sink that decided counts what it lost. Each loss is
  then counted once, where it happened.
- **Each sink has its own reason.** A projection to a peer that falls
  behind must not slow the service, so it drops and counts. A trace read
  later as the record of what happened must be complete or say it is
  not, so it backpressures by default: a local disk is fast next to most
  peers a service waits on, so letting facts wait for it costs little.
  One policy for both would be wrong for one of them.
- **Waiting at the sink is the ordinary backpressure.** It is room first
  at an edge (programming-model.md, section 2) applied to one more
  stream, not a new mechanism.

What it gives up: while a sink that backpressures is slow, its facts
wait in memory, and a stalled one would keep them for good, so a stream
that can stall has a deadline that abandons it (io.md, 5.1), and the
sink records that it was abandoned.

## Why limits at the edges and owned bytes

The memory strategy of programming-model.md, section 6, and what it was
chosen against. It replaced, on 2026-10-28, counted entities sized at
startup and a worst case that had to fit.

Every entity's bytes are already capped by protocol limits: the largest
message, the unparsed input, the queued output. What a peer or a person
can push in is therefore bounded where it enters, and that is where a
refusal is cheap. Inside, a bound on every container is a second bound on
the same bytes, and it costs more than it gives:

- **The worst case bound nothing.** The sum of every container at its cap
  was tens of gigabytes for an idle engine that held tens of megabytes,
  and the declarations that had to cover it were hundreds of terabytes,
  so the startup check never refused.
- **A reservation is paid for.** Queues sized at startup asked hundreds
  of megabytes of heap for entries that were usually a handful, and a
  ring's head walks its whole buffer, so a busy service drifted toward
  its whole reservation in resident memory.
- **Propagation was the cost, not counting.** Parents summed their
  children's largest output, limits had to satisfy inequalities with one
  another and across repositories, and exact figures regenerated at
  every change of a type's size turned a downstream repository red when
  an upstream one moved.
- **Room checks hid orders.** A check that a queue had room for the most
  one entrance could produce also kept that entrance's requests from
  being overtaken; once named, an order is a predicate with a test,
  which a capacity never was.
- **Static memory suits narrow services:** few entity kinds, one message
  size, compile-time limits. Services with many entity kinds, payloads
  of variable length and limits that must agree across repositories
  allocate dynamically and meter at admission.

Against budgets fixed at startup for bytes as well (byte pools):

- **No mutable state shared between layers.** A payload moves from io to
  protocol to domain as a `Box`: a move the compiler checks, no copy, and
  every layer's state stays private. With byte pools, either every step
  gets `&mut` to one shared pool, or each layer has its own and copies at
  every boundary.
- **Exact sizes.** A 100-byte message takes 100 bytes, not a slot of a
  size class, and there are no size classes to tune.
- **Fewer cells.** Making a payload cannot fail, so there is no "pool
  empty" transition at every point that makes one; refusals happen at the
  entrances only.

Against the language's heap with no structure:

- **The admission check is the container** at an entrance: a full slab
  is the refusal; there is no separate counter to keep in step with the
  entities.
- **Handles come with the slab:** a generation check and constant-time
  lookup, instead of a map from id to entity.
- **Exhaustion is testable at the entrances:** a simulation with a
  capacity of 2 reaches every admission point.

What it gives up: a provable ceiling; an amplification bug (one event
that makes unbounded internal work) aborts rather than refuses, against
which each event's fan-out is bounded in review and the simulator holds
the peaks of fixed scenarios under ceilings; some pacing, since new work
waits at the watermark while admitted work drains; and constant-time
allocation, since the general allocator sits in the hot path and
fragmentation can push the resident size above the live bytes.

Also rejected:

- **`Rc<RefCell<T>>`, `Arc<Mutex<T>>`.** Reachability decides lifetime,
  which the lifecycle forbids, and borrow errors move from compile time to
  runtime panics.
- **References in long-lived state, arenas with lifetimes.** Lifetime
  parameters spread to every type that touches them, and state stops
  being a plain value that can be snapshotted and compared.
- **Custom allocators per layer.** The allocator API is unstable, and
  everything builds on stable Rust.
- **Shared immutable buffers (`Arc<[u8]>`) as the ownership system.** At
  most an optimisation, confined to io.
