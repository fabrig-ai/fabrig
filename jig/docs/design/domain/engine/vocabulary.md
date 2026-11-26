# The core: its vocabulary toward the root

## 1. Its vocabulary toward the root

### 1.1 What it hears

- **from the store,** through the root: the answers to its loads, in
  pages; the records restored at a restart; that the loading it asked for
  is done. It does not hear commits' answers: the journal does (../root.md,
  section 4), and feeds back what the core asked to resume after one;
- **from hosts:** a host's hello with its slots, what it hosts and its
  agent's bounds; turns, with the messages a run refused, calls, answers
  and facts of runs; notices relayed from a run's agent, a source
  limited (with how long until its limit resets, where its LLM provider
  gives that, which the core makes a time on its clock as it hears it)
  or refused, or a generation unauthorized (../accounts-sources.md, 2); each
  program source's login on a worker, which follows its hello and which
  its half checks (`fleet.md`); a host lost or back;
- **from parties:** sign-ins and their ends, a credential to mint for an
  identity the application's server verified (../people/README.md, section 3);
  keyed requests, each one operation or a compound request of several
  (../people/acts.md, 1); reads, and watches opened and closed (`views.md`);
- **from accounts' protocol:** a generation granted, a failure, a
  sign-in's address for the client that asked and its subject once
  signed in, a revocation's outcome, the component closed
  (../accounts-sources.md, 2, and ../accounts.md, 5);
- **from the application,** through its root: each generation of an
  application's credential granted (`README.md`, 6.1); a project's period begun,
  saying when the next begins if its source knows. The core keeps no
  calendar, infers no cadence and schedules nothing of its own
  (../authority-budgets.md, section 1);
- **from connectors,** each by number, in jig's terms (../connectors.md,
  section 2): resources' roles and kinds of hold; pools' slots; effects'
  descriptions; verdicts; procedures' step decisions; outbox outcomes;
  classified news for subscribers; sections and workspace items ready,
  with their sizes; adoptions; drift; a restart step done;
- **from itself:** an output it asked to resume once released (1.2), and
  its children's timers, through `fire`.

### 1.2 What it asks for

Each request the core makes says which of three it is:

- **part of the decision:** a record saved or erased, which goes into
  the decision's commit; and the hand-offs the root completes before the
  step returns: describe this effect, judge this requirement, keep or
  drop this effect, take this procedure's parameters, gather or cut this
  section;
- **held:** what must wait for the decision's commit to be durable: an
  assignment, a message relayed to a run, a cancel, an acknowledgement,
  an answer to a call or to a party, a load (so that it sees every commit
  before it), an outbox entry to make, a resumption of the core's own; and
  a party's read's answer and a watch's delivery, behind the last commit
  made (`decisions.md`, 1.2);
- **now:** what decides nothing and shows nothing durable: a run's text
  streamed to watchers, the answer to a run's read, a run's admitted call
  handed out to be decoded (1.4), a host's refusal before anything was
  admitted.

The root keeps each as the core said (../root.md, section 3); the journal
releases held ones in order, and the conformance world checks that
nothing held left early (../testing.md, section 5).

### 1.3 Decisions and hand-offs

A decision is a route through several children and connectors, completed
within one step of the root's (skein/docs/foundation/programming-model.md, 4.5):

- **One event in,** from the root.
- **Hand-offs out and back:** the core asks the root for what only a
  connector can say (a description, a verdict, a section's size), and
  the root asks the connector and gives the answer back to the core,
  before its step returns. Hand-offs are bounded per decision and
  acyclic: the core never asks a connector something whose answer needs
  another of the core's answers.
- **One end:** the core says the decision is complete, and the root
  accepts it into the journal (../root.md, section 3).

What must first wait for a load, a read or a host is a state of its own
(a task preparing, a brief gathering), and its decision is made in the
step its last answer arrives.

### 1.4 The usual routes

- **Sibling routes finish within their decision.**
  `sibling_routes_drained` requires every sibling continuation to finish
  before the core returns its decision, so a later entrance cannot split
  the decision's writes. The core world holds this under load in
  `jig-core-world`,
  `route_orders::a_wide_route_finishes_within_its_decision_before_a_later_entrance`.
- **A task's generated requests finish within its decision.**
  `task_routes_drained` requires the requests a task route generates to
  be routed before that decision ends. The core world holds this in
  `jig-core-world`,
  `route_orders::a_task_route_finishes_its_generated_requests_within_its_decision`.
- **A party's continuation finishes within its decision.**
  `party_routes_drained` requires the people route's continuation and
  answer to stay in the decision that admitted the request. The core
  world holds this in `jig-core-world`,
  `route_orders::a_people_route_finishes_its_continuation_within_its_decision`.
- **The fleet's follow-ups finish within their decision.**
  `fleet_routes_drained` requires the fleet route's generated follow-ups
  to finish before that decision ends. The core world holds this in
  `jig-core-world`,
  `route_orders::a_fleet_route_finishes_its_follow_ups_within_its_decision`.

| What arrives | Route | Then |
|---|---|---|
| a run's turn | fleet (fenced) → tasks (the inbox taken) → authority (spend) → views | commit; the turn acknowledged |
| a run's call to an engine tool | fleet → authority (the check) → tasks or notes | commit; the call answered |
| a run's call for an effect | fleet → the connector describes it → authority, its judges asked → the connector keeps it | commit; answered when made, or past its deadline |
| a run's call for a read | fleet → authority (the grant) → the connector serves it | answered now; nothing committed |
| a run's answer | fleet → tasks (result checked, task closing or idle; the messages it refused or left unread stay, and wake it by its policy) → authority (spend); what it left → its connectors | commit; acknowledged; effects made |
| a procedure's step | its connector → authority (each effect, its judges; each batch) → tasks | commit; effects made; tasks started |
| a connector's news | classified by the connector → tasks (inboxes, wakes) | commit; due tasks run, procedures step |
| an outbox outcome | its connector → tasks (settled; closing advances) | commit |
| a party's request | people → authority (their role and pool; a budget given `AsAllowed` resolved; a deadline given as a span made a wall) → tasks | commit; the party answered, with what an `AsAllowed` budget was given |
| a party's compound request | people (one project, within operations per request) → for each operation in order: authority, one pool counted across them → tasks | one commit, or none; one answer: all made, or the first operation refused and why |
| a project made (a party's `MakeProject`, or the deployment's act) | people (the maker an owner of the deployment) → the core (the name in shape and free, a configured policy, room; the next number) → authority (the policy copied) → people (the maker its owner) → tasks (its first period, with no next start) | commit; the maker answered; the root hears the project made |
| a task due | tasks → authority (the run may start) → brief, with its connectors gathering, and its transcript's current window loading → tasks (claim, with its inbox's messages for the start) → fleet (place) | commit the claim; assign |
| a task closing | tasks → its connectors (release) | commit; releases made |
| a goal or any task in its subtree changing | tasks → every enclosing goal’s projecting connectors, including a nested tracked task’s own | commit; whole plans and milestones made |
| a project's period | authority (its spend within the policy) → tasks (the period's funding opened; recurring tasks' batches; standing allotments renewed) | commit; tasks started |
| a timer | its child | as its child says |

- **A run's call is fenced, then decoded.** The fleet admits a call only
  from its task's live attempt; the core hands the admitted call out, and
  the engine's protocol layer decodes its input and gives the typed
  request back to the root, which routes it as the rows above say
  (`../../protocol/channel/owners.md`, 1.3; ../root.md, section 5). No domain parses a
  call's input.
- **A budget given `AsAllowed`** is resolved by authority as the batch is
  admitted (../authority-budgets.md, section 1). A pool's amount left is a live
  number only the core holds, which no client could read and send back
  without racing. The task keeps the amount, and the answer, a party's or
  a run's, says what was given.
- **A compound request is one decision** (../people/acts.md, 1): the route
  above runs once per operation, in order, within one step of the root's,
  and its hand-offs are bounded by its operations (`README.md`, section 7).

## 2. The engine

What a root calls: `Engine`, the core behind five free functions, and its vocabulary, `In` and
`Out` (`temper/docs/design/draft/jig-domain-api.md`, 4.1 and 4.5). For now it passes through:
what crosses it is section 1's vocabulary whole, and it changes no behaviour of the core's.

### 2.1 Its functions

- **`Engine`** holds the core and nothing else. `Engine::new(config, limits)` builds it as
  `Core::new` does, refusing what that refuses with the same `Problem`.
- **`step(engine, env, input, out)`** takes one input as one decision and appends its outputs to
  `out`, in the order the core made them. For `In::Legacy(event)` they are exactly what the core's
  own step returns for `event`, each as `Out::Legacy`.
- **`fire(engine, env, timer, out)`** fires one due timer the same way, as the core's `fire` does.
- **`due(engine, now, wall, admit)` and `next_deadline(engine, now, wall, admit)`** keep today's
  timers, clocks and admission: `now` is the engine's clock and `wall` the wall's, since a
  sign-in expires on the wall. `Admit::All` admits every timer, as the core's `due` and
  `next_deadline` do; `Admit::Finishing`, while the root holds new work at its watermark
  (`../root.md`, 3), admits only a timer that advances admitted work, as `finishing_due` and
  `next_finishing_deadline` do (`jig-domain-core/src/routing/timers.rs`).
- **`idle(engine)`** is the core's own quiescence, the journal's left to the root that holds it:
  the core's `quiescent(journal_idle)` is `idle(engine) && journal_idle`. It includes the
  finishing the core owes (`../root.md`, 3).
- **Where they live.** `Engine` and the functions are the `engine` module of `jig-domain-core`; `In`,
  `Out` and `Admit` are the same module of `jig-domain-core-vocabulary`, which the core's re-exports
  whole (`README.md`, 3). No crate root re-exports them while the core's own `step` and `fire`
  hold those names.

### 2.2 In and out

- **`In::Legacy` carries today's `Event`, and `Out::Legacy` today's `Request`,** and neither enum
  has another variant. A request keeps its mark (1.2) inside `Out::Legacy`, and the root keeps it
  as it does today (`../root.md`, 3).
- **The ports come beside `Legacy`.** Each port of the draft (`temper/docs/design/draft/jig-domain-api.md`, 4.2 and 5) arrives as variants of `In` and `Out` and takes its cases out of
  `Legacy`, a milestone at a time, until `Legacy` is empty and goes.

### 2.3 The core, while the move runs

- **The whole core, by method:** `Engine::core` gives it shared and `Engine::core_mut`
  exclusive (`engine.core()`, `engine.core_mut()`). So every use a root, a kit or a world makes
  of `Core` today (its methods, its children's state, its counters, the step's other entrances
  `want_accounts` and `resume_fleet`) compiles through them unchanged. Each milestone narrows what
  passes through, and the clean-up removes them.
- **Never by `Deref`,** since a deref would make each use of the core read as a use of the engine:
  through two named methods every remaining use stays in sight, to review and to the counts. The
  counts of `temper/docs/design/draft/jig-domain-api.md`, 16 that read a use of the core (its
  methods called, its children's state reached) read `core()` and `core_mut()` as they read
  `core.`, and do not count those two, so a root that holds `Engine` in place of `Core` lowers
  no count by doing so.
- **Why a pass-through:** built alone, the engine would be checked by nothing. Passing through, it
  is checked by every root's suites as each root adopts it, and each later move is a real one
  (`temper/docs/design/draft/jig-domain-api.md`, 15).

## 3. Later

- **The engine as the API** (`temper/docs/design/draft/jig-domain-api.md`, 4 to 6, by its
  execution file): the ports beside `Legacy`; connectors' declarations given to `Engine::new`;
  one class per input, and `idle` in place of `quiescent` and `finish_owed`; then `Legacy` and the
  access gone, and `In` and `Out` named `Event` and `Request` once today's are.
