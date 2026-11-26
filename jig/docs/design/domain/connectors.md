# Connectors

Provisional, 2026-10-07, revised 2026-10-23. The contract an external
system meets to be driven by an application on jig: what a connector is,
what crosses between it and the core through the application's root, and
what every connector does: resources and holds, effects and the outbox,
facts and topics, procedures, requirements' verdicts, projections, reads
and brief sections, workspaces, drift and adoption. It is the fifth
primitive of model.md (3.5) in depth, and jig's API toward applications:
connectors are the application's, and this is what each owes and is
owed. What is still open is listed in section 16.

## 1. In one page

- **A connector is a subtree of the application's engine domain,** a
  child of its root, in a closed set the root matches on. It knows its
  own system and nothing of the others, nor of the core's types; the
  core knows it by number (model.md, 3.5).
- **One vocabulary crosses** (section 2), the same for every connector,
  in jig's terms on the core's side: resource names, effect
  descriptions, verdicts, topics' news, procedure steps. The root
  translates each connector's vocabulary to it and back.
- **The owner keeps the value** (model.md, section 5). A connector keeps
  what is in its own terms (an effect's typed request, a procedure's
  parameters and state, a brief section, a read's answer, a result)
  under a task's number, a call's name or a token, and hands it over when
  the root puts an output together.
- **Resources** are named as paths, which authority's grants cover by
  pattern. A connector says which kinds take holds, exclusive or pooled,
  and whether a taken hold refuses or waits.
- **Effects** are committed into the outbox with the decision that asks
  for them, and made after the commit, keyed. Each kind declares a
  recovery class, which says how an effect whose outcome is uncertain is
  resolved and how strong its promise of once is (`connectors-effects.md`, 1.3). A connector
  describes each effect for authority, priced at its maximum when it
  costs.
- **Facts are read afresh; events are hints.** A connector keeps a
  working set of what its system owns of the live work, and publishes
  what changed on topics, classified for each subscriber.
- **Procedures are the mechanics,** as executors of tasks: pure step
  machines, level-triggered, their state committed with each step. A
  step acts on tasks with the operations a party or an agent has, naming
  the revisions it was told, but decides no proposal or escalation.
- **Requirements are judged by their connector** (section 7): asked
  about an effect's exact state, it answers met, wait or refuse from its
  own facts, whichever connector makes the effect. A guarded requirement
  is checked again by the effect's own system as it applies the effect;
  an observed one holds only when the effect is decided, within its
  freshness.
- **What the application does there belongs to it.** A connector records
  what it made by key, writes only to what is owned (or as a
  participant, or through its own mechanics into what is shared), and
  treats any other change to what it relies on as drift.
- **Adopted as it is.** A system is plugged in with whatever access the
  application is given; nothing on it is required.

## 2. Where a connector sits

```
application root            routes between the core and every connector
├── core                    jig's core
└── <connector>             a connector's top: its resources, holds' kinds, topics, procedures' routing
    ├── …                   its own children, as its system needs
```

- **A connector's top is a child of the root,** and may have children of
  its own (programming-model.md, 4.5). Its top routes between them, as
  the root does between the core and the connectors.
- **What crosses** between the root and a connector, in the connector's
  vocabulary, translated by the root to and from the core's:

  | Down, root to connector | Up, connector to root |
  |---|---|
  | a live task names these resources; a task is erased, so nothing it named need be kept for it (3.4) | each resource's role and kind of hold; unknown; not adopted |
  | holds taken, handed down or released; a writer slot taken or freed | a pool's slots changed; a resource read afresh after a lost attempt |
  | describe this effect, from an agent's call or a step | its description: kind, resources, purpose, condition, form, price |
  | keep this effect under this key, or under this proposal; drop it | — (part of the decision) |
  | make this outbox entry (the journal released it) | made, with what it made; failed; uncertain |
  | judge this requirement for this effect's state | met; wait, for what; refuse, on which fact |
  | judge these items of your kinds of settings, for the revision they would make (section 17) | accept; refuse, why; leaves it as it is |
  | keep these items' values: a change committed, a removal, a reset; a proposal's items made, accepted, ended | — (part of the decision) |
  | the application's defaults of your kinds, at a start | which differ, with their sizes |
  | a procedure task: made with its parameters; activated; a message; closing; what its step made, with the revisions it names; a step refused, and why | a step's decision: effects, batches, amendments, cancels, proposals, messages, a hold, a result |
  | project this goal: its state, its plan, its milestones | the projection's effects |
  | release what this closing task held | the release's effects |
  | subscribe this task to this topic; unsubscribe | news for these subscribers, classified |
  | serve this read for a run | its answer |
  | gather this section for a brief, within this budget; cut it to a size; hand it over | ready, with its size, or why it could not (engine/briefs.md, 1.1); the section |
  | name what a run of this task needs prepared; hand it over | its workspace items, with their size, and the application's credentials they need; the items |
  | what a run of this task left on its resources | taken: facts updated, procedures stepped |
  | adopt this resource into this project | adopted, with what may be done there and the parties it knows; refused |
  | release this project, being deleted, its tasks erased: drop what you keep of its adoptions and of your kinds' settings in its scope (people/acts.md, 2); asked again at each start until answered | released, once that has committed; at once when nothing is kept |
  | an act's answer, under its key, once durable | an act in the deployment's name, under a key of its own: a person's conversation closed, the deployment's key added or account removed, one of its resources adopted, a project made or deleted (people/acts.md, 3) |
  | restart: restore these records | done |
  | catch up, from the opening: read afresh and settle the outbox | each half's report, its fresh reads in and its outbox settled |

- **The core's side is one crate of values,** `jig-domain-contract`, a
  kit's contract (programming-model.md, 4.5): the core's vocabulary and
  the children whose values it holds depend on it, and a connector may
  name it. It sits below them and every connector, depending on skein-lib
  and `jig-domain-link` alone, so the crate graph stays acyclic. Its
  values are the core's own, each moved down from the vocabulary or the
  child that defined it and still named along that path, so a caller
  builds through either.
- **One module per capability that carries values, and `ops`.** A value
  the asks and answers of one capability carry is that capability's
  module's: `lifecycle` (restoring, catching up, adopting), `resources`
  (names, roles, kinds of hold, a member's holdings), `effects` (an
  effect's key, description, form and recovery class, its outbox's
  outcomes, and authority's effects, judges, verdicts and findings),
  `topics` (news' classes), `parts` (references to what a connector
  keeps), `procedures` (a procedure's step and decision, with its
  delegates), `projections` (a goal's projected records) and `settings`
  (an application's settings items, their judgements and defaults). `ops`
  holds the operations' values, whichever capability carries them: a
  batch's members, a task's spec, contract, authority, funding, executor,
  phase, result, holds and words, its amendments, cancels and proposals.
- **Values, not decisions** (programming-model.md, 4.6): definitions, the
  methods that read or name a value, derived impls, and no state. A
  helper that reads a domain's limits or chooses an outcome stays in that
  domain as a free function: seeding a repair or a procedure's delegate
  (`RepairSeed` and `PersonSeed`, which carry the tasks child's events,
  stay in the core's vocabulary), and a word's payload size, a message's
  inbox family and a hold's check (the tasks child's).
- **Tasks are named by number.** The connector keeps the numbers of the
  tasks it serves as plain values; it never sees a task. What it projects
  of a goal, the root gives it, in the connector's own terms.
- **Its protocol is its own.** The root passes its requests down to the
  application's protocol layer and their answers up; its system's hints
  (webhooks, a stream of alerts) arrive as events no request caused.
- **Its store records are its own,** saved and erased as part of the
  decisions it takes part in, restored at a restart (engine/decisions.md, 1.6).

## 3. Resources

### 3.1 Names

- **A path of segments,** the connector's choice, stable for the
  resource's life: for `ops`'s infrastructure, `env`, the environment,
  then `service` and its name, or `pool` and its name. Names are bytes,
  compared segment by segment, and every name carries its connector's
  number.
- **Patterns** cover what begins with them, which is how authority grants
  effects on many resources at once (authority.md, section 4).
- **Kinds of effect and read** are the connector's, numbered, each with
  the resource kinds it applies to, and the order among them that
  authority holds as data. A kind of effect has one form and one
  recovery class (`connectors-effects.md`, 1.1, 1.3): writes that differ in either are different
  kinds, even on one object (creating an issue, editing it). A grant
  that should cover several names each, or one the order says implies
  the others.
- **Names after a task:** a connector may define resources named after
  the task that makes them, under a prefix of the deployment's, which a
  batch names symbolically (authority.md, section 4).
- **Symbols:** a connector may offer numbered symbols, each a resource
  it names after a task once the task has its number (a forge's *its own
  goal branch*), which a batch member or a template names
  (`../protocol/configuration-templates.md`, 3). The connector says, per symbol,
  the name it resolves to and the hold it takes, and which tasks may name
  it (tracked ones only, say); it refuses a batch naming a symbol it does
  not offer that task, naming the symbol (tasks/holds.md, 2).
  The connector is asked for a member's holdings with its number and its
  tree's root: a member of a batch made beneath a task, by a person or by
  a delegating task, has that task's root, and a member made as a root its
  own number.

### 3.2 Roles

Each resource a project adopts has a role (model.md, 6.1): owned, or
adopted to write into; a fork; context. Each object the application
deals with is owned or participating (model.md, 8.2). A connector keeps
both with the project, and every effect is checked against them before
any grant: no effect writes to a context resource, and an effect on a
participating object is a participant's write.

### 3.3 Holds

The core keeps holds and their queues (tasks/holds.md); the
connector says what they apply to:

- **which kinds of resource take holds,** and whether each is
  *exclusive* (one holder: a service being remediated, a branch being
  written) or *pooled* (counted slots: a pool's environments);
- **for a pooled resource, its slots,** from its facts or its
  configuration, told to the core whenever they change. A pool that
  shrinks below its holders keeps them, and the connector says which
  holder's allocation is gone, which is drift (tasks/holds.md, 6);
- **whether a taken hold refuses or waits** (tasks/holds.md, 2);
- **what a taking does** to the resource, as effects of its own (`connectors-effects.md`, 1.7):
  a branch made at its base's tip, say;
- **what a release does** to the resource, per way its task ended (`connectors-effects.md`, 1.6);
- **each held resource's report,** its role and hold kind, for every
  resource a batch member holds, sent before or with its holdings
  answer (3.1's symbols, tasks/holds.md, 2). The core keeps those a
  waiting batch names, refuses a batch whose reports its table cannot
  hold, and refuses one whose reports have not all come within
  `tasks.hold_wait`, without asking again (tasks/holds.md, 2).

Shared resources take no hold: writing into them is ordered by the
connector's own mechanics, such as a landing queue. A held resource may
be ordered by such a queue too: a goal's own branch, held by its goal,
that its goal's changes land into one at a time. The queue then orders
only writes made under the hold, by the holder's delegates, each taking
the resource's writer slot as it writes (tasks/holds.md, 1 and 3). Such a queue is
outside the waits the core checks for circles (tasks/holds.md, 7), so its
connector keeps it clear of them: a task kept by a dependency (tasks/lifecycle-endings.md,
1) leaves it, and comes back once it goes on, so that no task in it
waits behind one that waits for it.

### 3.4 Live resources

The resources live tasks name are the connector's live resources: what
its working set holds (`connectors-facts.md`, section 1) and what it reads afresh at a restart.
When no live task names a resource, it leaves the working set.

- **What an ended task named** may outlive its release as a record of
  the connector's own, which watches nothing, until the task is erased:
  temper's forge keeps the repositories an ended chat worked on, so that
  the chat reads their names while the store keeps it.
- **The core tells every connector when a task is erased:** `Unname`,
  in the decision that erases the task's full record, whichever erases
  it: a deleted project's erasure of its tasks (people/acts.md, 2), the
  forget of a task forgotten once read (people/README.md, section 5), or
  retention at the deployment's horizon, which keeps the task's summary
  a while longer (`../protocol/store/retention.md`, 6.2). From then on
  nothing it named need be kept for it. A connector that keeps nothing
  past a release takes `Unname` as nothing to do.

## 4. Effects and the outbox

Moved to [`connectors-effects.md`](connectors-effects.md), section 1.

## 5. Facts and events

Moved to [`connectors-facts.md`](connectors-facts.md), section 1.

## 6. Procedures

Moved to [`connectors-procedures.md`](connectors-procedures.md), section 1.

## 7. Requirements and verdicts

- **A connector defines requirements** on its own facts, each with the
  parameters it takes: observability's *healthy replica elsewhere*, *load
  below a level for a time*; a forge's *CI passed at this head*, *up to
  date with the base*. The policy names them as judges of other kinds of
  effect (authority.md, section 10), and the judging connector keeps
  their parameters as its own configuration.
- **Guarded or observed.** A requirement is *guarded* when the effect's
  own system checks it as it applies the effect, as the effect's
  condition (`connectors-effects.md`, 1.1): a branch's head, a service's version. It holds when
  the effect is applied. Every other requirement is *observed:* judged
  when the effect is decided, by whichever connector owns its facts,
  within a freshness the policy gives it, and it may stop holding before
  the effect is applied, as load may rise before a scale-down. The
  policy may mark a requirement as one that must be guarded; an effect
  kind whose system cannot guard it is then refused (authority.md,
  section 10).
- **Asked for a verdict,** the judge is given the requirement, the
  effect's resource names and the state the effect names. It finds the
  subject of its facts from those names, by its own configuration, and
  answers from its working set, in the same step:
  - *met:* its facts, read within the requirement's freshness, hold;
  - *wait:* its facts are unknown, older than the freshness, or pending,
    saying what it waits for; it reads afresh, and its news on the
    subject lets the asker ask again;
  - *refuse:* a fact failed, saying which.
- **Judged twice,** as the mechanics in the procedure that waits for its
  requirements, and as the rule at the effect (authority.md, section 10).
- **Pinned, where guarded.** A guarded verdict is for the exact state the
  effect names (a version, a head), which the effect's condition holds
  its system to as it applies it. An observed verdict promises only what
  its judge saw, within the freshness.

## 8. Projections

What a connector writes for people to read of a goal, such as an issue
on a forge or an incident page, is a projection: neither a task nor a
procedure, but a mechanism of the connector's, keyed to the goal.

- **Fed by the root:** the goal's state, its plan as its subtree's tasks
  and their phases, and its milestones as the tree's history records
  them, in the connector's own terms, whenever they change: a change to
  any task of the subtree, not only to the goal's own. Its milestones
  are the changes made to its subtree's tasks, and their holds and ends
  (tasks/lifecycle.md, 12.3), never a wake's phase. Milestones come
  as they are recorded, each with an identity of its own, so the
  connector keeps what it has written and adds to it; the plan comes
  whole, within the tree's limits (tasks/README.md, section 6). A nested
  tracked task has its own projection and remains in each enclosing goal's
  plan; its saves and milestones feed those goals too, through its end.
  Becoming tracked never leaves an enclosing plan row stale. The last
  feed, as the goal closes, is its closing state; a goal an amendment
  untracks is fed its state then, as a last feed (tasks/changes.md).
- **Where:** in the project's home for that connector (model.md, 6.1).
  A project with no home there has no projection there.
- **Its effects** have keys of their own (`connectors-effects.md`, 1.4), are checked against the
  authority the project's policy gives its goals' projections, and are
  made through the outbox like any other.
- **Its state** (what it last wrote, by a digest) is kept with the goal,
  and outlives it until its last write, made as the goal closes or is
  untracked, has settled.

Projection outbox outcomes settle resource writers without settling calls,
holding a task or waking a procedure. Made, Failed, Withdrawn and NotMade
release the writer; Uncertain and Held retain it for connector recovery.

## 9. Reads and brief sections

- **Reads an agent may call** (engine/executors-tools.md, 1): typed, each with an
  answer bounded in bytes and charged to the connector's request budget:
  a service's logs in a window, a metric's series, a pull request's
  diff. The call reaches the root decoded as the connector's request; the
  core checks the read grant (authority-checks.md, 1.5); the connector serves
  it; its answer goes out through the journal's door for outputs that
  decide nothing (root.md, section 3). The core never holds it.
- **Sections a brief carries** for the resources a task names, and for
  the documents its charter names (engine/settings.md, 9): a service's recent
  errors, an environment's state, a pull request against its base, a
  file at a branch's head. The brief child domain plans them (engine/briefs.md,
  section 1) by kind and budget; the connector gathers each, keeps it,
  and says its size, or says why it could not, in fixed words and how
  that counts (engine/briefs.md, 1.1). It gathers within the brief's
  deadline, which the root gives it with the gather: a section it reads
  in several reads ends at its cutoff, ahead of the deadline, with what
  it has and a marker for the rest (engine/briefs.md, 1, *Within the
  deadline*); asked to cut it to a smaller size, it cuts it by its
  own rules and says the new size; when the brief is complete, it hands
  the section to the root, which puts the assignment together (root.md,
  section 7).
- **A connector's result** reaches a dependent's brief the same way: the
  brief asks the connector for its section on that ended task's result.
- **Neither is state.** What a run reads is not kept by the connector
  beyond its answer, and a section beyond the brief it was gathered for.

## 10. Workspaces and what runs leave

- **What a run needs prepared** comes from the resources its task names.
  For each such resource, the connector that owns it says what the host
  is to prepare, in its own terms: a repository's checkout, where it
  starts and whether it may be written; nothing, for a resource reached
  through tools. These workspace items are handed to the root with the
  assignment (root.md, section 7), and the application's workspace child
  on the host prepares them (hosts/README.md, section 6).
- **The credentials they need.** As it gathers a run's workspace items,
  the connector names the application's credentials the host needs to
  prepare, deliver and save them (a token to fetch and push), by the
  application's number for each. The core grants the run the current
  generation of each, and the assignment carries them as grants
  (engine/README.md, 6.1). The connector never holds a value.
- **Context resources** of the project may be added read-only, as the
  connector says.
- **What a run left** on a connector's resources, as its host reports it
  in the run's answer (the heads it pushed, the state it saved), reaches
  that connector through the root: its writer slot's provisional moves
  are confirmed (section 11), its procedures step.
- **Saved state.** A run that parks, fails or is cancelled with
  unfinished work has it saved by its host where the connector says; the
  task keeps which resources have saved state (tasks/README.md, section 3), and
  the next run's workspace items start from it.

## 11. Ownership and drift

- **Owned objects are recorded by key** (`connectors-effects.md`, 1.4), so ownership is the
  store's, whatever the system shows.
- **Projections are written, never read back** (model.md, 8.1).
- **The application's own moves.** A held resource moves under the
  application's own hand in two ways: an effect the connector made,
  known by its key and its outcome, and a run's write, which the engine
  hears of only with the run's answer. While a resource's writer slot is
  taken, a move by the application's identity that the connector can
  tell is the writer's (a branch moving forward) is taken as the
  writer's, provisionally, and confirmed by what the run's answer
  reports, or, if its attempt is settled as lost, by a fresh read then.
  A procedure decides nothing on a resource while its writer slot is
  taken, but for a procedure on a resource its requester holds, whose
  writer is that requester's own run: the run that asked for it goes on,
  and the procedure decides on what it reads there.
- **Drift is a change the application did not make** to something it
  owns and relies on: a service restarted by another hand, an environment
  deleted, a branch moved by another identity or backwards, an object
  closed or retargeted. It holds the task that relies on it, saying what
  changed and when, and the client shows it; nothing is written over.
- **What is not drift:** a fact a decision reads is simply read again;
  a change to what others own is news. To the task that holds a
  resource to write it, a change made before its own write is a fact it
  reads again, and it decides on what it finds (a service restarted by
  another hand before the procedure holding it made its restart); to
  every other task relying on the resource, the change is drift.
- **Participating objects** are those others write too. What others
  write there is read into the inbox of the task that deals with the
  object, as news on its topic; what the application writes there is a
  participant's, possibly a proposal for a person to accept first.

## 12. Adopting a system

- **Adoption** adds a resource to a project with a role, by a party
  whose role allows it (people/acts.md). The connector reads what
  the application may do there and keeps it with the project, and the
  project's ceiling follows from it (authority.md, section 6). It may
  also say which parties it knows there, with their permission, from
  which the project's policy may seed roles (people/README.md, section 4). For
  a connector that projects goals, an adoption may name the resource
  the project's home there (model.md, 6.1). Configuration adopts
  nothing: a deployment's first start seeds projects without resources
  (`../protocol/configuration-engine.md`, section 1).
- **The project document** gets each adopted resource from the connector
  with its project, stable name, adoption role and home flag
  (`Event::Adopted`). Live hold reports do not replace these facts.

- **Nothing is required** of the system beyond the access the
  application is given. What is missing narrows what may be done there
  or slows it down, and the client says so: a system without hints is
  kept up by polling alone.
- **Provisioning** a system's objects (the system's own setup, not a
  task's resources) is an optional set of effects a party may ask for,
  never a step of adoption.
- **One deployment's.** The prefix under which a connector names what it
  creates is the deployment's; adoption refuses a resource where another
  deployment's objects already live under that prefix. Objects under the
  prefix that the store records no making of are another deployment's,
  or those of a store made anew's predecessor: the resource is refused,
  saying which, until a person chooses another prefix or clears them.
- **What may be done is read again** when the system refuses something
  the connector believed it could, and the project's ceiling narrows
  with it, for every later decision, existing tasks' included
  (authority.md, section 6).

## 13. Load

Each connector keeps its system's request budget:

- **Fresh reads first,** verdicts' and effects' own reads among them,
  then effects, then keeping up, then slow passes, with a share of each
  window kept for the last two.
- **Load follows change, never history:** what a connector asks of its
  system per pass grows with what changed and with the live resources,
  never with the system's past, and its worlds count it.
- **A refusal for the rate** stops every call to that system until its
  reset.

## 14. Restart

At a restart (engine/restart.md), a connector is given, when the core
asks for each step:

1. **its settings, then its records:** the application's defaults of
   its kinds, of which it tells the core which differ from those it
   stored (section 17); its live resources, its procedures' states and
   parameters, its outbox, its projections' states, what it made by key,
   and its own state, loaded from its live ranges;
2. **its catching up, at the opening** (`Ask::CatchUp`), once the
   engine's own records are in and its runs adopted, its two halves
   together while the engine serves:
   - **a fresh read:** it reads afresh what its system owns of its live
     resources, within its budget; it reports `RestartStage::ReadAfresh`
     once every fresh read has answered, and from then on its **fresh
     reads are in**;
   - **its outbox to settle:** it hands its restored entries to its
     outbox before any entry made since, and looks for every entry not
     committed as made, by its key or the state it leads to (`connectors-effects.md`, 1.3); it
     reports `RestartStage::Settled` once each has been looked at once;
   it has **caught up** once both reports are made, and the core records
   them (engine/restart.md, *The start's interface*). Nothing bounds it:
   a system that stays down keeps it catching up while the engine
   serves;
3. **its acts to send again:** each act in the deployment's name it
   keeps unanswered, sent again under its key once the core is ready,
   answered from its saved answer when it was made (people/acts.md, 3).

From the opening its procedures step as their wakes come. A decision
on a fact its system owns waits for that fact's first fresh read, or for
its fresh reads where the procedure names no read of its own (`connectors-facts.md`, 1.1); a
write waits for the entries before it on its lane, the restored ones
among them, and a write on another lane goes ahead (`connectors-effects.md`, 1.2). Nothing it
held in memory before the restart is needed, and nothing is judged
from what the store held alone.

Restoration reports each stored adoption to the core through
`Event::Adopted`, with its project, stable name, role and home flag.
The reference connector also emits `Request::Resource` to rebuild the
core's hold report. These reports ask for no external action. Restoration
makes no new effect or write.

- **Catching up reports each half once it has ended.** The fresh reads'
  report waits for every fresh read to answer; the settle's for every
  uncertain restored entry to have been looked at once. Neither waits
  for the other, and the engine is open before both. The ops world
  (`jig-ops-world`, its connector checks) and the reference connector
  world (`jig-test-connector-world`, its restart orders) hold this
  under load: each half reported only once it has ended, and the engine
  open before either.

## 15. The world

Each connector has its worlds: its subtree's, with its system's fake
below it and the root scripted above it; and the application's, where it
runs with the core and every other connector on jig's conformance world
(testing.md, section 5). What every connector's world checks, beyond its
own stories: keyed effects made once across restarts at drawn moments,
uncertain entries resolved as their recovery class says, with copies
arriving late, after their deadline and after a retry, and targets that
another hand reached too; deadlines that survive restarts made before
they pass; procedures that decide the same when stepped twice on the
same facts; procedures that amend, cancel with a grace and message their
delegates, an amendment refused as moved when a party amended the
delegate first, and the procedure stepping again on what it is told,
never writing over that change; verdicts that never say met on facts
older than their freshness; facts that change between a verdict and its
effect; pools that shrink while held and while waited for; a procedure
whose task is kept, its effect that waits for dependencies and its
result refused, stepping again when they are done, and its own queue
left while kept and rejoined after; topics whose
news reaches every subscriber once, classified; values it keeps for the
root handed over once, or dropped when the core drops their token; and a
load that follows change, the system preloaded with ten times the
history making the same calls while idle.

## 16. Open questions

- **Connectors added without a build:** the set is closed in the code,
  as the programming model's rules make it. A connector whose behaviour
  is data (a generic HTTP system) would be one connector of that kind,
  not a way around the closed set.
- **Classifying news with an LLM:** whether a cheap model should judge
  relevance where a connector's rules do not tell (tasks/README.md,
  section 8).
- **Freshness defaults:** how fresh each kind of observed requirement
  must be by default, before a policy says.

## 17. An application's settings

A connector may own kinds of settings of the application's (engine/settings-application.md,
1): the application declares each kind with its connector's number,
and gives the connector its defaults. The connector keeps the values in
its own family; the core keeps whether each setting follows its default
or a change, the revisions, the proposals and their sizes, never the
bytes.

- **Its defaults at each start,** before its records (section 14): the
  application's, which it compares with those it stored, telling the
  core which settings' defaults differ and their sizes, and naming, by
  kind, scope and number, each default it stored that the application
  no longer gives; it saves the new ones in the decision that makes the
  release's revision (engine/settings-application.md, 1). A default
  past its kind's bounds, a stored value it cannot read, or one of a
  kind no longer declared, refuses the start, naming it.
- **Judged in the change's decision.** A change naming items of its
  kinds reaches it as a hand-off within the decision's step, as a
  requirement's verdict does (section 7): the items, each its kind,
  scope and number and the value it would leave or its removal (a reset
  as the default it returns to), and the revision they would make. It
  accepts, with the charters each value names where its kind declares
  that it names some (engine/settings-application.md, 1); refuses with a reason in its own
  words, which the change's refusal carries to the party; or says an
  item leaves its setting as it is. A judge reads only its own state, and decides what the items
  mean in its system's terms.
- **Kept in the same decision.** Once the change is committed, it saves
  its items' values as the current changes, a removal as a change with
  no value, or erases the changes reset, and acts on them from then on. A proposal of settings
  naming its kinds hands it its items' values to keep, checked against
  the kinds' bounds, in the decision that makes the proposal; it erases
  them when the proposal is withdrawn or rejected, and an acceptance
  makes them the changes.
- **What its items replaced, for a restore** (engine/settings.md, 10). As it
  saves a committed change's items, it keeps beside each the value the
  setting held before (none for one that followed its default or was
  removed), by the revision the change made, and erases those of a
  revision in the decision that erases the revision's record from the
  history (engine/settings-store.md, 1); a release's revision asks it to keep
  none. Its worst case prices, for each record the history may keep, a
  whole set of its kinds' values. A restore's decision hands it the
  proposal the restore makes and its items of its kinds, each by kind,
  scope and number with the revision whose record replaced it; it keeps
  the value it kept as that revision's replaced one as the proposal's
  item, as a proposal's items are kept, and answers each item's size and
  the charters it names, checked against its kinds' bounds alone.
- **Read through it.** The application's children and clients read a
  value from the connector, by kind, scope and number: the newest,
  whatever revision a task it serves was made in. Never pinned.
- **Its world** scripts changes of its kinds, judged both ways, a
  proposal kept, accepted and withdrawn, and a restart that hands back
  the values and finds a changed default (section 15).
