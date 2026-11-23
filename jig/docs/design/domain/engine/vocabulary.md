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
  or refused, or a generation unauthorized (../accounts.md, 3.5); each
  program source's login on a worker, which follows its hello and which
  its half checks (`fleet.md`); a host lost or back;
- **from parties:** sign-ins and their ends, a credential to mint for an
  identity the application's server verified (../people/README.md, section 3);
  keyed requests, each one operation or a compound request of several
  (../people/acts.md, 1); reads, and watches opened and closed (`views.md`);
- **from accounts' protocol:** a generation granted, a failure, a
  sign-in's address for the client that asked and its subject once
  signed in, a revocation's outcome, the component closed
  (../accounts.md, 3.5 and section 5);
- **from the application,** through its root: each generation of an
  application's credential granted (`README.md`, 6.1); a project's period begun,
  saying when the next begins if its source knows. The core keeps no
  calendar, infers no cadence and schedules nothing of its own
  (../authority.md, section 7);
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
  admitted (../authority.md, section 7). A pool's amount left is a live
  number only the core holds, which no client could read and send back
  without racing. The task keeps the amount, and the answer, a party's or
  a run's, says what was given.
- **A compound request is one decision** (../people/acts.md, 1): the route
  above runs once per operation, in order, within one step of the root's,
  and its hand-offs are bounded by its operations (`README.md`, section 7).
