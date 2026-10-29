# The application's root

Provisional, 2026-10-07, revised 2026-10-28. What an application's
engine root is and does: the top-level domain every application on jig
writes, which composes jig's core with the application's connectors and
its own child domains. It says what the root carries, the one shape
every root has, what jig asks of skein-lib's journal, how the root
routes, translates and puts outputs together, its part in restart and in
effects, and what it must never do. The core is engine.md; connectors
are connectors.md. What is still open is listed in section 14.

## 1. In one page

- **The application owns the root.** skein's model nests step machines,
  and the top one is always the application's (programming-model.md,
  4.5). The root composes the core, the application's connectors, jig's
  host and smith's `smith-inline-agent` if it runs agents in the engine,
  and its own child domains. It alone faces the application's protocol
  layer.
- **The root routes and translates; it never decides or orders.**
  Everything that makes jig's promises true (what commits together, what
  waits for a commit, the order of a restart, the check on every effect)
  is decided by the core or kept by the journal. The root carries it, in
  a shape that is hard to get wrong.
- **One shape** (section 3): admission before any child changes; one
  decision gathered from every child the event reaches; at most one
  commit; outputs out only through the journal.
- **The journal is skein-lib's** (section 4): a generic container that
  numbers commits, holds outputs until their commit is durable, and
  releases them in order. It is the root's only way out.
- **The owner keeps the value** (section 7). The root puts each output
  together from its owners' parts in the step it is decided: an
  assignment from the core's part and the connectors' sections and
  items, a call's answer from the core's part and a connector's.
- **Boilerplate, not judgement.** What is left to the application is
  which child an event goes to and the small total functions between
  vocabularies. A missing arm breaks the build. A wrong arm (an event
  routed to the wrong child, an effect's kind or resources mistranslated)
  is a bug the journal cannot see: the conformance world catches it
  where its scenarios reach, since every write and output passes through
  the journal, the fakes report what they observed in their own terms,
  independently of the root's translation, and its worlds generate kinds
  and names across connectors (testing.md, section 5). What no scenario
  reaches is the application's to test.
- **Copied, not invented.** `ops`'s root is the reference every root is
  copied from (section 12).

## 2. What a root is

```
<application>-domain        the root: the journal, routing, translation, putting outputs together
├── core                    jig's core (engine.md)
├── <connector>…            the application's connectors (connectors.md)
├── jig-domain-host         jig's hub, for runs on the engine's slots, if the application runs agents there (hosts.md)
├── smith-inline-agent      the hub's agent in the engine, smith's: its domains in this process (hosts.md, 5.2)
└── …                       the application's own child domains
```

- **Its state:** its children's states, the journal, and nothing else
  that outlives a step but what a child asked it to carry back (a token
  to resume). It has no records of its own in the store.
- **Its vocabulary** is what crosses the domain boundary: the store's
  commits and loads, the workers' channel, the clients' requests, each
  connector's protocol, the LLM accounts' driver, timers; and, when it
  runs agents in the engine, `smith-inline-agent`'s LLM requests and
  their terminals, and smith's payloads it hands out to be decoded or
  encoded (section 5). It is the union of its children's, wrapped: an
  exhaustive enum per direction, with one variant per child.
- **Its children are siblings,** sharing no types (programming-model.md,
  4.5). `jig-domain-workers` is a shared leaf value crate, like skein-lib:
  it owns the checked `SourceName`, `Worker` and immutable configured-membership
  minter. It knows no fleet or accounts state. The core knows each connector
  by a number the root gives it in configuration; each connector knows nothing
  of the core.
- **No root above it.** In one process, an engine's root and its
  workers' roots stay apart, each under its own protocol layer, joined
  only by bytes over in-process addresses (`../protocol/service.md`,
  section 10).

## 3. The one shape

```
try_step(root, event):                                          ← before any child changes
    the store's answers, a released output, a completion of admitted work:   always taken
    an event with admitted and new work:  split: the admitted part taken, the rest handed back
    new external work:  handed back while the backlog is at the watermark   (section 10)
step(root, event):
    d = journal.decide()                        → a decision that cannot fail
    route the event to its child; while a child asks for a hand-off,
        route it to the child that answers, and its answer back   (section 5)
    every write any child asked for, wrapped    → d.push_write
    every held output, put together             → d.push_held   (section 7)
    every output marked now                     → journal.now
    journal.accept(d)                           → at most one commit, to the store

iterate, after the step:
    store answers commit n  → journal.committed(n)
    store fails commit n    → journal.failed(n): nothing after it leaves; the engine stops
    journal.release(out)    → held outputs whose commit is durable, in order, within a bound;
                              those addressed to a child are fed back to it as events
```

- **No outbound queue.** The root's entry points take the journal and no
  other way out: what leaves the domain is what `iterate` takes from the
  journal. A root cannot send a held output early without writing code
  to get around the journal.
- **Every request says what it is.** The core and the connectors mark
  each request as part of the decision, held, or now (engine.md, 4.2).
  The root keeps the mark: a held output goes to `d.hold`, a now one to
  `journal.now`. The door for outputs that decide nothing and show
  nothing durable is one call, so review finds every use.
- **Admission is all or nothing,** by a watermark (skein's
  programming-model.md, 7.1). New external work is admitted while the
  backlog, the journal's (`backlog()`) with the root's own work and
  queued store answers, is under the watermark; an event handed back
  changes nothing, and its sender keeps it and offers it again, as
  engine.md, 5.1 says. What finishes admitted work is always taken, so
  the backlog drains. An event that carries both is split: the root
  steps the admitted part and hands back the rest, an event of its own
  holding only the new work. The connector captures the rest's baseline
  from its kept state before settlement and names it in the rest:
  observability's `HealthChanged { from }` holds the healthy replica
  count; infrastructure's `ServiceDrift { from }` holds the revision,
  and `EnvironmentDrift { from }` holds the optional cached presence or
  absence (`Option<Option<EnvironmentFact>>`). When the rest runs, it
  compares that baseline with the kept state: matching replica counts
  or revisions send no news. An environment rest acts only while a made
  environment is kept absent; a baseline of cached absence suppresses
  another deletion record, but still notifies reliant tasks
  (connectors.md, section 11; ../examples.md). Below the watermark, the
  connector steps the whole event as settlement then rest through the
  same step function, within one decision; verdicts precede health news.
  The split takes the connector's environment to check validity: an
  invalid mixed fact settles without changing state and leaves no rest.
  No route reserves room. Synchronous hand-offs finish within their
  decision, store answers keep arrival order, and a kept turn precedes
  its terminal acknowledgement (sections 4 and 5;
  programming-model.md, 7.2).
- **One decision per event.** Hand-offs complete within the step;
  nothing of a decision is left for a later step but what a child holds
  in its own state, saying so (a brief gathering, a load awaited).
- **Finishing is the core's.** The pure, allocation-free `finish_owed`
  answer states whether closure owes (1) an eligible projection feed,
  update or final forgetting; (2) publication of gathered subject changes;
  (3) reconciliation of retained writing with its current proof, watched
  or not; (4) an unreported watch ending required by sign-in, role,
  visibility or stopping state; or (5) a dirty deployment header or the
  core's first drain notification. A projection pass that emits continues
  to defer watch finishing. Accepting a decision or hearing `Committed`
  does not itself discharge finishing. Work a child holds awaiting its
  next input is not actionable finishing. Before omitting a closure, the
  root also checks its own work, connectors, fleet, due activations and
  timers at the supplied clocks. A due timer, including one refused
  admission, forbids omission. `quiescent` includes this same finishing
  derivation and its existing broader idle fences; false does not promise
  that finishing can advance (engine.md, 6.1 and 11; people.md, 3).
- **Released outputs addressed to a child** (the core's resumption after
  a claim commits, a connector's entry to make) are fed back to it as
  events by `iterate`, each as one more step of this shape.
  A released callback is always taken: no decision refuses it, so none is
  lost under load, and a callback's terminal acknowledgement follows the
  turn it kept.
- **Completions pass held new work in arrival order.** The engine's
  `passing()` permits admitted completions to pass a held call, stopping
  at the next new event; later records keep their order. The reference
  service holds this at the watermark in `jig-test-service`,
  `tests::engine::completions_pass_a_held_call_in_order`.

## 4. The journal

The journal is a container of skein-lib's (skein's `lib.md`), generic
over the writes and outputs it holds, as lib's queue is over its items.
It inspects nothing it holds. What jig asks of it:

- **Uncapped** (`Journal::uncapped`): its commits, held outputs and door
  grow as they fill. Its `backlog()` counts the commits made and not yet
  answered, the outputs held and those at the door, for the root's
  watermark (section 3).
- **jig's journal sizes** are `JournalSizes { commits, writes, door, release }`,
  owned by `jig-protocol-config` and `jig-test-domain`. `commits` and
  `writes` supply the store's commit edge; `release` bounds the held
  outputs released per call. The journal's containers remain uncapped.
- **A decision:** a value, `#[must_use]` and not `Clone`, from
  `decide()`, that gathers one decision's writes and held outputs
  (`push_write`, `push_held`). It reserves nothing and cannot fail. The
  store's commit is an edge: it refuses a commit of more writes than its
  `Limits.commits.writes` (`sizes::Commits`, `../protocol/store.md`,
  5.4), and a decision past it stops the root.
- **At most one commit per decision,** numbered in order; a decision
  with no write makes no commit, and its held outputs wait for the last
  commit made.
- **Tags:** each held output carries the number of the commit it follows,
  so it is released only once that commit is durable.
- **The door:** `now(output)` leaves at once, in order with other now
  outputs, untagged.
- **Release in order,** a bounded number per call, so a step hearing the
  store does not release an unbounded group at once.
- **A failed commit:** nothing tagged with it or after it is released,
  and the journal reports stop.
- **A stop of the root's own** (`stop()`): a root that cannot go on stops
  its journal, and nothing held or at the door leaves again.
- **The old forms,** `takes(room)`, `JournalRoom` and the capped decision,
  stay beside these until no root uses them.
- **The subset:** generic types and lib's own containers, no traits
  defined, no closures (programming-model.md, 10.2).

## 5. Routing

- **Synchronous hand-offs finish before the next entrance.**
  `decision_work_finished` requires the decision's work queue to be
  empty before it is accepted. The core world holds this in
  `jig-core-world`,
  `route_orders::a_wide_route_finishes_within_its_decision_before_a_later_entrance`;
  the ops world holds the connector's whole route in `jig-ops-world`,
  `orders::an_application_route_finishes_before_the_next_entrance`.
- **Store answers keep their arrival order.** `store_answers_fifo`
  requires queued answers to be routed before later store answers,
  including while new work waits at the watermark. The core world
  holds this in `jig-core-world`,
  `orders::queued_read_answers_keep_their_arrival_order_under_load`;
  the ops world holds it in `jig-ops-world`,
  `orders::read_answers_keep_completion_arrival_order_under_load`.
- **A kept turn precedes its terminal acknowledgement.**
  `prior_held_delivery_dispatched` requires the earlier released
  delivery to be dispatched before the later acknowledgement, including
  callbacks addressed to a child. The core and ops worlds hold this
  under load in `jig-core-world` and `jig-ops-world`, each in
  `orders::turn_kept_precedes_terminal_acknowledgement_under_load`.
- **An early account request follows the previous move's dispatch.**
  `move_requests_dispatched` requires every stage of the previous move
  to be dispatched before another early account request starts. The
  reference service holds this under load in `jig-test-service`,
  `tests::accounts::an_early_account_request_waits_for_the_previous_moves_dispatch`.
- **A sign-in's ending follows its own durable commit.**
  `ending_commit_durable` requires the ending's commit number to be no
  greater than the durable number before its secret is erased. Earlier
  durability cannot release a later ending. The reference service holds
  this under load in `jig-test-service`,
  `tests::clients::sign_in_endings_follow_their_own_commits_under_load`.

- **From the protocol layer:** each event goes to the child it is for:
  - the store's pages and restores to the core or to the connector whose
    range it is;
  - the workers' channel to the core, but for what a run's answer says
    it left on resources, which goes to the connectors that own them;
  - a run's call to the core's fleet, which fences it; the admitted call
    is handed out to be decoded (`../protocol/channel.md`, 6.3), and its
    typed request goes to the core, or, for a connector's read or effect,
    to that connector first (section 9);
  - clients' requests, reads and watches to the core; what the core
    admits for an application's own object or subject goes, as a
    hand-off, to the connector it names, which decides the request or
    makes the document (`../client/domain/contract.md`, 3.3;
    connectors.md, 5.5);
  - a connector's system's events and answers to that connector;
  - a timer to the child that armed it.
- **Hand-offs between children,** within one step: what the core asks of
  a connector (describe, judge, keep, drop, gather, cut, hand over, step,
  project, release, subscribe, restore, read afresh, settle, decide a
  party's request, make a document) goes to the connector the core
  names by number, and its answer back; what a connector says up (a
  step's decision, news, an outcome, a pool's slots, drift) goes to the
  core. What the core says of a procedure's step, the task it made, goes
  to the procedure's connector in the same decision, and is never
  dropped (connectors.md, 6.3).
- **Settings of the application's kinds** (engine.md, 17.5): a change's
  items of a kind go, as a hand-off within the change's step, to the
  connector the kind names, and its judgement back to the core; the
  core's word to keep or erase values, at a change's commit and a
  proposal's making and ending, goes to it in the same decision. The
  root routes by the kind's connector number, which may name any child
  of its own, and reads nothing of the values. A child of the
  application that needs a value reads it from the connector that keeps
  it, by kind, scope and number (engine.md, 17.5).
- **A project made** (`people.md`, 5.2): the core's word that it was
  made, released once its commit is durable, goes to the application's
  period source, which, keeping a calendar, says the project's periods
  from then on; the reference service's, keeping none, takes nothing
  from it.
- **To the hub inside the engine:** the core's assignments, relays and
  cancels for runs placed on the engine's own slots go to the hub, and
  what it says goes back to the core as a worker's would (hosts.md,
  6.1).
- **Between the hub and its agent capability:** `smith-inline-agent`, in
  the engine. The root translates between them as a worker's root does
  between its hub and either of smith's agent capabilities, with the
  same functions, since both kinds speak smith's parent face (hosts.md,
  sections 4 and 5).
- **Below the inline agent:** its LLM completions go to the protocol
  layer, which runs them on smith's LLM component, keeping each slot's
  result contract beside it, and their terminals come back to the run
  that asked (`../protocol/llm.md`, section 3; hosts.md, 5.2). They pass
  through neither the hub nor the core. The root drains the inline
  agent's facts and content (smith's `domain/host.md`, 9.1), and a run's
  text comes from smith's LLM component in the protocol layer
  (`../protocol/llm.md`, section 3); both reach the hub as text and
  activity (hosts.md, 6.5).
- **smith's payloads, by hand-out:** as the root routes one between the
  hub and `smith-inline-agent`, it hands it out to the protocol layer to
  be decoded or encoded, as a host tool's call is, and routes on what
  comes back; it never encodes or decodes one itself (hosts.md, 6.8).
- **Nothing is routed by content** beyond the variant: which child an
  event is for is in its type.

## 6. Translation

- **Small total functions,** one per pair of vocabularies, matching
  exhaustively, with no `_` arm, so a change on either side breaks the
  build in one place.
- **The core's connector vocabulary** (engine.md, 4.1) is translated to
  each connector's own and back: a connector's number, its resources'
  names as paths, its effect descriptions, its verdicts, its news.
- **The hub's agent vocabulary** is translated to smith's parent face and
  back, with the same functions whichever of smith's agent capabilities
  the root composes, smith's payloads among it by hand-out (section 5;
  hosts.md, section 4).
- **The store's vocabulary.** The root's `Key`, `Record`, `Write` and
  `Range` wrap the core's and each connector's: one variant per child.
  Its keys order first by child, then by the child's own order, so a
  child's ranges stay contiguous. The core's live families and each
  connector's are ranges of their own, which the restart script loads
  (section 8).
- **Configuration** is split the same way: the root gives each child its
  part, and the core each connector's number.

## 7. Putting outputs together

What leaves the engine is often made of parts that different children
own (core.md, section 5). The root puts each together in the step it is
decided:

- **An assignment:** the core's part (the run, its attempt, its charter,
  its core sections, its transcript's current window, its calls, the
  start's messages) names the connectors' sections and workspace items
  by token; the root asks each connector to hand over what it keeps
  under those tokens, and wraps the whole in the application's
  assignment, which the journal holds until the claim commits.
- **A call's answer:** the core's part (status, the effect's key, why it
  was refused) and, for an effect made, the connector's part (what it
  made), kept under the call's key; the root asks again when the call is
  asked again.
- **A run's read's answer** is the connector's alone, and goes out
  through the door; a party's read of a connector's document is held
  behind the last commit made (engine.md, 5.2).
- **A connector's result in a dependent's brief** is a section like any
  other.
- **A token the core drops** (a brief abandoned, a call withdrawn) is
  dropped by its owner too: the root passes the drop on in the same step.

The parts are taken in the step the output is decided, so nothing waits
in the root between steps; each owner hands over its part once.

## 8. Restart, from the root

The core runs restart as a script (engine.md, section 6). The root:

1. loads the core's live ranges, the deployment's and then the
   settings' first, and restores them into the core, then says done;
2. for each connector the core names, loads its kinds' settings from
   its family, hands it the application's defaults for them and passes
   to the core which defaults changed, then asks it for its live
   ranges, loads them, restores them into it, and says done;
3. tells the core when the fleet's claims have been given, and passes on
   workers' hellos as they dial in;
4. asks each connector to read afresh, and says done when each has;
5. asks each connector to settle its outbox, and says done when each
   has;
6. lets the core open decisions.

The root does only the step it is asked for, and the core asks for the
next only once this one is done, so the order is the core's.

A restart that fails leaves its decision unaccepted, and the root takes
no decision after it: it once relied on the open reservation to refuse
them, and with decisions that cannot be refused it says so itself.

## 9. Effects and verdicts, from the root

- **From an agent's call:** the call reaches the root as the connector's
  typed request; the root gives it to the connector, which describes it
  and keeps it under a token; the description goes to the core; the
  core's check asks for verdicts, which the root routes to each judge it
  names and back; the core's answer (allow, propose, wait, refuse) goes
  to the connector, which keeps the effect under its key or the
  proposal's number, or drops it. All within one step, one decision.
- **From a procedure's step:** the step's decision comes up from the
  connector with its effects described; the root routes their checks as
  above, and the core's answers back to the connector, before the step's
  decision is complete.
- **Across connectors:** the judge of a requirement may be any connector;
  the root routes by the number the core names, never deciding which.
- **Made after the commit:** the request to make an outbox entry is a
  held output addressed to its connector, released once its commit is
  durable and fed back to the connector by `iterate`.

## 10. Worst cases and limits

- **Each child declares its limits:** its entrances' counts and sizes,
  and the formulas that size its edges (a record, a frame, a message).
  No child declares an output maximum or the most an event makes it
  write, since no route reserves room.
- **The root's watermark,** one number in entries, a field of the root's
  `Limits`, defaulting to the journal's commits in flight plus the
  door's outputs (`JournalSizes.commits` and `JournalSizes.door`):
  `default_watermark` adds `commits` and `door`. The commits alone bind after
  one decision that holds several outputs, since held outputs count in
  the backlog (temper's root is at the same formula). It limits pacing, not memory
  (programming-model.md, 7.1); where a person's reads wait behind it, it
  rises.
- **No sum along routes,** and no worst case of the root's: the
  per-process estimate goes with the startup fit check
  (programming-model.md, 6.3).
- **The core's routes,** one number in its `Limits`: the most hand-offs
  one core decision routes, sibling continuations included. It is the
  bound of every drain within a step (each drain a `for` over at most
  `routes`, its predicate asserted empty after: section 5 and
  engine.md, 4.4), a loop's bound, not a reservation: nothing is sized
  by it. The profiles set it (small 20,783, standard 151,655); an
  application whose root wraps jig's core takes it from jig's profile.
- **The application's limits** are configuration, given to each child as
  its own; the conformance world runs every story under tiny limits and a
  low watermark.

## 11. What a root never does

- decide anything of the core's or a connector's: what to commit, what
  waits, whether an effect may be made, which judge to ask, the order of
  a restart;
- send an output any way but through the journal;
- hold anything between steps but its children and the journal;
- read or rewrite a value it carries between children, beyond wrapping
  it;
- give one child another child's type;
- route by content beyond the variant.

## 12. The reference root

`ops`'s root (`../examples.md`) is the reference: written to be copied,
commented as such. Its crate has one module per part of this document:

```
<application>-domain/src/
├── lib.rs          the tree, re-exports
├── boundary.rs     the root's vocabulary: events in, requests out, one variant per child
├── store.rs        Key, Record, Write, Range: the children's wrapped
├── domain.rs       Domain, step, fire, release: the one shape (section 3)
├── route.rs        which child each event and each hand-off goes to (section 5)
├── translate.rs    the small total functions (section 6)
├── assemble.rs     outputs put together from their owners' parts (section 7)
├── restart.rs      the script's steps, as the core asks (section 8)
├── limits.rs       the children's limits, and their sum (section 10)
└── tests.rs
```

A new application copies it, replaces `ops`'s connectors with its own,
and lets the compiler list the arms to fill.

## 13. The world

The root has no world of its own beyond its application's: the
application's worlds run its whole domain on jig's conformance world
(testing.md, section 5), whose referee checks, from outside, that the
root kept every promise the conformance world checks: nothing a party,
a host or a system saw depended on a commit that was lost; every effect
made as its recovery class allows across a crash at every commit; no
effect made without its verdicts; a restart's steps in the core's order;
every effect a fake system observed matching, in that system's own
terms, the effect the core decided. Its own stories are the
application's.

## 14. Open questions

- **Generating the boilerplate:** whether the root's routing and
  wrapping, which have one shape, are best copied and filled by agents
  (section 12), or generated from a short description, which the subset's
  rules on macros would confine to a build step.
