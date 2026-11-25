# A task's waits

Provisional, 2026-11-24. What a task waits for, as records: each wait's
kind, what it waits for, since when, until when and what happens then,
and who hears; how the core computes them, how a connector's procedure
reports its own, and how they read across a restart and an upgrade.
Written from `temper/docs/design/draft/bounded-waits.md`, 3, 4.1 and 9.
What happens when a bound comes is the tasks' design for jig's kinds,
and the application's for a procedure's (the draft, 4.2): this document
gives every wait its bound and names what happens at it.

## 1. In one page

- **A wait is a record** (2): the task it waits at, its kind, its object
  by number, its *since*, its bound with what happens at it or the reason
  it has none, and who hears.
- **The kinds are one closed set** (3): jig's eight, which the core
  computes, and a procedure's four, which its connector reports.
- **Computed, never stored** (4): the core computes a task's waits as it
  makes a line or a document, from the facts it keeps, in one order. A
  line carries the first and the nearest bound; a document those two and
  every one of its own.
- **Every wait has a bound or a reason** (5): its own clock where it has
  one, else its kind's figure from the `waits` table of the core's
  limits, else one of four reasons for none.
- **A procedure reports its own** (6): its connector tells the core what
  its task waits on, in the same record, and the core lists it as its own.
- **Old work starts its clock at the upgrade** (7): each kind's floor,
  kept once, is the earliest *since* a wait of that kind reads.

## 2. The record

| Member | What it is |
|---|---|
| task | the task the wait is at; on a line that names a wait in its subtree (4), that task |
| kind | one of section 3's |
| object | what it waits for, by number, in its kind's form (3) |
| since | when this wait began (3), never when its task began, and never before its kind's floor (7) |
| bound | the instant it ends if nothing moves it first, with what happens then: *hold*, *pass up*, *overdue* or *give up* (the draft, 4.2); or none, with its reason (5) |
| hears | the task's escalation holder, as an escalation of the task would name it now (`../people/README.md`, 4): a task, a person or a project's role; none where nothing would hold it |

- **One wait, one episode:** a wait's identity is its task, kind, object
  and *since*. A wait whose object or *since* moves is another wait, and
  a step that changes nothing it waits on keeps the wait it was in.
- **Whose vocabulary:** the record is the core's, beside the documents
  that carry it (`vocabulary.md`). It names the owners' values: the
  tasks' holders and resources, the people's entry kinds, the next run's
  wait as the task's document gives it. An application's connector names
  it for its procedures (6), and the client contract carries it as it is
  (`../../client/domain/contract/documents.md`, 1).

## 3. The kinds

A closed set, so that every view, test and tool reads the same names.

| Kind | Object | Since |
|---|---|---|
| `party` | an entry waiting at a holder, a task, a person or a role: a question, a proposal, an escalation or a person task, by the people's entry kind and its number | its arrival at that holder: the question's asking, the proposal's or the escalation's own *since*, the person task's wait (`../people/README.md`, 7) |
| `question` | the task's own open question to the task that requested it, by number, and that task, its answerer | its asking |
| `dependency` | the pending task | its addition, or the task's making for a dependency it was made with |
| `resource` | the resource, by its name, and the task holding it where one does | its queue join, or the fresh join a return to the queue makes (`../tasks/holds.md`, 2) |
| `account` | the next run's wait for an account, a source or a credential, as the task's document names it (`../accounts-uses.md`, 1.3), with its reset where one is known | its entry into its phase |
| `host` | the next run's wait for a host, a worker or a slot, as the document names it | its entry into its phase |
| `delegate` | the live delegate | its making |
| `news` | the task's subscriptions by number and its armed timer: what can wake it | its park: its run parked, or its live run waiting as the fleet last observed it |
| `step` | a procedure's step, and what it waits on: an effect, a job, a read or a write | reported (6) |
| `room` | a table's room, and the entries holding it, with their owners | reported |
| `turn` | the task holding the turn, or the queue's head | reported |
| `sign-off` | the rule, and the roles that can still sign | reported |

- **A procedure's things** (`step`, `room`, `sign-off`) are each a code
  of its connector's, a number and the task owning it where one does:
  the connector's codes are its own, as an application's parts' kinds
  are (`views.md`, 2), and its client names them. At most four, the
  oldest first.
- **A task backing off between tries** is in an `account` wait, whose
  reset is its retry; **one whose worker is suspended** is in a `host`
  wait, whose reset is the suspension's end.
- **A proposal waiting at a policy** (past its tree's root, or a
  settings proposal at the deployment's) is a `party` wait at the role
  that decides it, as the people's inbox shows it
  (`../people/README.md`, 4 and 5): a settings proposal at the
  deployment's `owner` role. One still pending when its policy names no
  role that may decide its kind (`../tasks/proposals.md`, *Where they
  wait*) waits at its project's `owner` role, the role that may change
  that policy and so the only one that can move it; no inbox gains an
  entry by it. Either is the project's last holder: *overdue* at its
  stall (5).
- **A task closing** (`../tasks/lifecycle-endings.md`) waits on its own
  ending's settlement, which its phase names: no wait of this record
  (section 9).

## 4. A task's waits

- **Which:** a live task has
  - each entry of its own waiting at a holder (`party`): its questions to
    a party, its proposals, its escalation while held, and, as a person
    task, itself;
  - each question of its own to its requester still unanswered;
  - while it is kept (`../tasks/lifecycle-endings.md`, 1), each pending
    dependency;
  - while it waits in a resource's queue, that resource;
  - while its next run waits, what it waits for (`account` or `host`);
  - while it isn't working, each live delegate;
  - while it is idle with no live delegate: its procedure's waits where
    its connector has reported any (6), else `news`.

  A task is **working** while its run is live and not waiting: it has its
  `party` and `question` waits, and, kept, its `dependency` waits, since
  the keep refuses its effects that wait for dependencies while the run
  finishes its turn (`../tasks/lifecycle-endings.md`, 1); none other. Its
  line's first wait stays none and what it waits on stays *nothing*
  (`views.md`, 3); its nearest bound counts them. An ended task has none.
- **Passed up, still one wait:** a question passed up
  (`../tasks/messages.md`, 2) is one wait of its asker's, at its current
  holder, since its arrival there: a `question` wait while a task holds
  it, a `party` wait while a party does. A person task offered to its
  next holder (`../people/README.md`, 7) is one `party` wait, at that
  holder, since its offer there; its first addressee's inbox entry is no
  second wait.
- **Their order:** as listed, each group oldest first by *since*, then by
  number. `task show` lists them so.
- **A line's first wait** is the oldest entry waiting at a person or a
  role in the task's live subtree, itself included, as `views.md`, 3's
  first rule finds it; else none while the task is working; else the
  first of its own. Its **nearest bound** is the earliest bound among its
  first wait and its own waits, with that wait's kind and what happens at
  it. Both come from the walk the line's wait takes today, within tasks
  per tree.
- **A document's waits** are its line's first wait and nearest bound,
  the same two its line carries, then every wait of the task's own, in
  order, within the document's lines, and their count.
- **What a task waits on** (`views.md`, 3) is a line's summary of these:
  its values are those that section gives, computed from the records,
  so the two never disagree.

## 5. Bounds

- **A wait with its own clock keeps it** (the draft, 4.1): an
  escalation's and a proposal's stall, a person task's stall and a
  resource queue's wait, each its limit counted from the wait's *since*
  as the record has it (7), so a wait begun before the upgrade counts from
  the upgrade. A procedure reports the instant of a clock of its own (6).
- **A wait with no clock** takes its kind's figure from the `waits` table
  of the core's limits, one name per kind: `waits.question_ms`,
  `waits.party_ms` (a question waiting at a person or a role),
  `waits.news_ms`, `waits.host_ms`, `waits.account_ms`, `waits.step_ms`,
  `waits.room_ms` and `waits.sign_off_ms`. Each is positive, taken at
  the next start, and overridden by a deployment's `limits` setting as
  the core's other limits are (`settings.md`). Its stock is the user's,
  in jig's standard profile (the draft's status, Open). Its bound is its
  *since* plus the figure, or the task's deadline where that comes first.
  An `account` or `host` wait with a known reset counts the figure from
  the reset instead, where that is later.
- **A `dependency` or `delegate` wait** is bound by the task's deadline.
  Every deadline a bound takes here is the one the task is held at, its
  deadline moved later by the time the task spent suspended
  (`../tasks/lifecycle-holds.md`, 1), so that a record's bound is the
  instant its outcome is taken.
- **No bound,** each for its reason, which the record names and the views
  show with the wait's age:
  - *no deadline:* a `dependency` or `delegate` wait in a task with none;
  - *standing:* the `news` of a standing task (`../tasks/standing.md`) or
    a conversation, with no deadline, whose work is to wait for its
    party's words;
  - *behind task N:* a wait behind another task, a turn or a room, which
    that task's own waits bound (the draft, 4.1), reported (6);
  - *an operator's hold:* reported (6).
- **What happens at the bound,** by kind:

| At bound | Kinds |
|---|---|
| *hold* | `dependency`, `delegate`, `news`, `resource`, `account`, `host`, and a procedure's `step` and `room` unless it reports otherwise |
| *pass up* | `party` at a holder past which another holder covers the entry; `question` |
| *overdue* | `party` at the project's last holder, which no other covers; `sign-off` |
| *give up* | a read its caller may lose, reported |

- **Who covers what passes up:** an escalation, a proposal and a
  question pass to the next holder their route finds, a task or a party
  (`../tasks/proposals.md`, *Where they wait*); a person task passes only
  to a party, since a party answers it (`../people/README.md`, 7). A
  `question` wait whose route finds no holder past its current one is
  *overdue*.
- **Taking the outcome:** jig's kinds' outcomes are taken where their
  owners say: a hold, and a question's pass up, by the core's bound alarm
  (`../tasks/lifecycle-holds.md`, 1; `../tasks/messages.md`, 2); a
  resource's hold, an escalation's and a proposal's pass up, and a
  person task's offer, by their own clocks (`../tasks/holds.md`, 2;
  `../tasks/proposals.md`; `../people/README.md`, 7). An *overdue* wait
  is taken by no one: past its bound it is overdue, with no timer after
  (`../tasks/proposals.md`, *Overdue at the last holder*).

## 6. A procedure's waits

- **Its connector reports them** to the core, through the root, as it
  reports its parts' marks (`views.md`, 2): the task, and its waits, at
  most four, each a kind, an object, a *since*, and a bound: its own
  clock's instant with what happens then, its kind's figure with what
  happens then, or none with its reason. A report replaces the last one;
  an empty report leaves the task to `news`.
- **When:** whenever they change, after a step that changes them or a
  fact it reads that does (`../connectors-procedures.md`, 1.2), and once
  for each of its live procedure tasks after a start, as its catching up
  begins (`restart.md`). A read again that changes nothing reports
  nothing, so its wait keeps its *since*.
- **The core keeps** each live procedure task's last report, in memory
  alone, within its live tasks, and drops it when the task ends. A report
  naming a task that isn't a live procedure of that connector, or past
  its bounds, is refused whole, and the connector is told why.
- **It lists them as its own:** the core floors their *since* (7) and
  makes a figure's bound from it, as for jig's kinds.

## 7. Across restarts and upgrades

- **A restart** finds the same waits: each is computed again from the
  stored facts section 3 names, with its *since* unchanged. A
  procedure's come back with its connector's report after the start
  (6), their *since* from its own stored facts; a live run's `news`
  starts again from its next observation, since its place lived only in
  memory.
- **Each kind's floor** is the wall time at which the deployment first
  kept a wait of that kind: for jig's kinds, the first start of a build
  that computes them, set by that start's restore; for a procedure's,
  the core's first report of it. The core keeps the floors in its
  deployment record, an optional member whose absence is no floors, and
  sets each once.
- **A wait's *since*** is the later of its fact and its kind's floor. So
  a store the previous build wrote opens with every wait's *since* at
  the first start, none past its bound, and its bound counting from
  then (the draft, 3.4).
- **A fact a later build adds** to a wait is an optional member; its
  owner reads its absence as the first start of the build that reads it.

## 8. Who reads them

- **A task's line and its document** (`views.md`, 3), and through them the
  client contract's (`../../client/domain/contract/documents.md`, 1).
- **The worlds' record check** at each driver's quiet point
  (`../testing-fakes.md`, 1.1 and 1.2).
- **A bound's outcome:** the tasks' own design for jig's kinds (5,
  *Taking the outcome*), the application's for a procedure's (the
  draft, 4.2).

## 9. Open questions

- A closing task's waits on its run, its effects and its releases: no
  kind of section 3 names them, and their bound is their phase's.
- Bounds learned from how long each kind of wait takes on a deployment,
  rather than fixed figures (the draft, 13).
