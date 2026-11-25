# bounded-waits: every wait named, bounded and visible

Draft, 2026-11-24, on the user's word: the second of the five drafts the user accepted on 2026-11-20, from the review of goal execution on the reference deployment, 13 to 20 November. It is failure-worlds' *bounded waits* (`failure-worlds.md`, 7). Its evidence is in section 2. Its reasons are in [bounded-waits.notes.md](bounded-waits.notes.md), its execution in [bounded-waits.execution.md](bounded-waits.execution.md).

## 1. In one page

A task, a change or a landing that waits says what kind of thing it waits for, but rarely which one, since when or until when. Many waits have no bound at all, and those that have one often just read again when it comes. When a wait can never end, nothing notices: people and scripts find it by reading ctl's text, hours later. On the reference deployment that cost a four-hour stop of main's landings, goals that ended with their children's questions unanswered, answers held without telling their sender, and a held landing left unnoticed for six days.

This draft makes every wait a record and gives each one an end:

1. **A wait is a record** (3): its kind, the thing it waits for by number, since when, its bound, what happens at the bound, and who hears. jig computes it for each task, temper's procedures report theirs in the same form, and old records read as new ones.
2. **Every wait has a bound** (4), its own clock where it has one, else a figure from one table in the settings. At its bound a wait never just reads again: most kinds hold with words naming what they waited for, keeping whatever the task owns, and the hold escalates as any hold does.
3. **A wait that can't end is caught when it parks** (5). A task whose every wake leads back to its own answer, or to nothing, spends its tries at each park and is then held naming the chain.
4. **Words reach their reader, and their sender knows** (6). A held answer tells its sender what holds it. No message is taken and then dropped silently. A task may wait on any task its party can read.
5. **Every fixed table says what a full table does** (7): its request waits with a bound, or is refused with words, and what is stored is checked against its room when the engine starts.
6. **Holds always say why and how out** (8), and an entry at a person shows how overdue it is.
7. **One wait-for view** (9): `task show` gives a wait's since and bound, and a project read lists every live wait, nearest bound first, as text and as JSON.
8. **The team's watchers retire** (10), each replaced by one of the above.

What it leaves to the other drafts is in section 11.

## 2. Where it starts

| What happened | Why it went on | Section |
|---|---|---|
| Main took no landing for four hours on 11-24: landing 15091 read the same 24-commit comparison more than 400 times, answered each time, while its round "waited for a fresh forge read" | A read again at a deadline is a retry, not a bound: nothing ended the wait, and nothing said it had stopped advancing | 4 |
| Goals ended with their children's questions unanswered (P43): 4166, 4623, 4839, 4999 | A goal watching a change whose producer waited on the goal's own answer was woken once per newest question, then left idle (`question_woken`). Nothing saw the circle | 5 |
| A relayed producer question went unanswered (P85) | An answer held for a sibling dependency tells its sender nothing (`temper/crates/temper-engine-domain-forge-change/src/relay.rs`, `Stands::Held`); its scenario reproduces that path, though the original incident's dependency was never confirmed | 6.1 |
| A change could not park waiting on another goal's task (P113) | An ordinary task-made batch or watch may name only tasks it references. With nothing awaited, `wait` answers "nothing more will come", so the change held a question open, and that held its parent | 6.3 |
| A triager couldn't file a finding (P86) | Four person-task subscriptions used up the bound. The repair admits one topic past it, so the reverse order still fails | 6.4 |
| Goal 11067 slept through the repair it waited for (11-24), and its close's hold reached the owners an hour later | A delegate's result that its wake policy doesn't let wake it rode along, while an escalation waited at the goal | 6.5 |
| A parked start and its resend jammed a shared sign-in (P117) | The parked act kept its exchange, two of two, so no other request on that sign-in could stream | 7 |
| A goal's first assignment couldn't be encoded (P63) | Its fixed charter outgrew a hidden allowance; startup now checks the configured room, but not a charter already stored | 7 |
| Landing report 6978 has been held since 11-18, unnoticed | A hold waiting at the project's last holder has no bound and shows no age | 8 |
| Party-placed stops and some holds show no words | A placed hold's reason may be empty, and an automatic hold may carry only its category's phrase | 8 |
| The team wrote and keeps a dozen watchers: goal watches, health filters, a landing-stall parser, dashboards | No read lists what waits, since when and until when, so scripts scrape ctl's text. The landing-stall parser has matched no line since 11-20 | 9, 10 |

The inventory these rows rest on, taken at main 362c345f74, is in the notes.

## 3. A wait is a record

### 3.1 What it holds

Every wait is one record:

- **kind:** one of the kinds in 3.2;
- **object:** what it waits for, by number: a task, a change, a landing, a question, a party or role, a resource, an account or credential, a host, a forge read or write;
- **since:** when this wait began (3.3), never when the task began;
- **bound:** the instant it ends if nothing moves it first (4), or none, for the few waits section 4.1 names;
- **at bound:** what happens then, one of *hold*, *pass up*, *overdue* or *give up* (4.2);
- **hears:** who is told at the bound: the task's escalation holder, as escalations name it (`jig/docs/design/domain/people/README.md`, 4).

A wait is computed wherever it can be derived, as jig computes what a task waits on today (`jig/docs/design/domain/engine/views.md`, 3), so that no restart finds it stale. Its identity is its task, kind, object and *since*: one wait, one episode, so its one wake and its notices are counted once (3.4).

### 3.2 The kinds

A closed set, so that every view, test and tool reads the same names:

| Kind | Object | Whose |
|---|---|---|
| `dependency` | the pending task | jig |
| `delegate` | the live delegate, the oldest first | jig |
| `question` | the task's own open question, and its answerer | jig |
| `party` | a question, proposal, escalation or person task waiting at a party or role | jig |
| `news` | the subscriptions and timers that can wake it, by number | jig |
| `resource` | the resource and the task holding it | jig |
| `account` | the account, source or credential, and its reset if one is known | jig |
| `host` | the kind of host or slot it needs | jig |
| `step` | a procedure's step and the effect, job, read or write it waits on | temper |
| `room` | the outbox, and the entries holding it, with their owners | temper |
| `turn` | the landing or change holding the turn, or the queue's head | temper |
| `sign-off` | the rule and the roles that can still sign | temper |

`news` names what can wake the task. `step` covers a change's steps, a landing's round and its *awaited* fact (`../domain/landing-waits.md`, 2), a closer's step, a goal branch's making, a release's cleanup, and the steps of follows and tiers, each with its own object. A read's own queue inside the root that holds no task, an archive load waiting for a slot for one, is not a wait in this sense: it has its call's deadline.

### 3.3 Since

- **A wait that has a record** takes its *since* from it: a question's admission, a dependency's addition, a delegate's making, a queue join, an escalation's arrival at its holder, a person task's start.
- **`news`** is since the task parked, not since its subscriptions were made: a watch made days ago starts no clock.
- **`step`** is since the procedure last took a step. A read that changes nothing, or a poll that reads again, starts no new wait; a step that advances does.
- **A wait behind another** (`room`, a follower's `turn`) is since it joined the line.

### 3.4 Across restarts and builds

- **Restart.** A wait is recomputed from the stored facts above, so a start finds the same episode, its *since* unchanged, and its one wake and notices already counted where they are stored. A wait whose place lives only in memory, a forge read's admission line for one, is asked again at start and starts again from then, naming the restart.
- **The previous build's records** (each build reads the previous one's) have none of the facts this draft adds: a held answer's blocker and its time, a landing wait's *since*, a wait's notices. Each is an optional member with a default. An old record with no *since* reads as since the start of the first build that read it, so no old work passes its bound at the upgrade; its bound counts from then.
- **A crash at a bound** commits nothing half: the hold, the pass up or the notice is one decision, made again at start if it wasn't kept.

## 4. Every wait has a bound

### 4.1 Whose clock

- **A wait that has its own clock keeps it.** A resource queue's 120 seconds, an escalation's and a proposal's one-hour stall, a change's 30-minute forge stall and its CI clock, which counts running, waiting for a runner and unknown apart (`../domain/forge/change/stalls.md`, 2), a landing's prepare job and its gate's timeout: each stays as it is, and is its wait's bound.
- **A wait with no clock** gets one figure per kind from one table in the settings' limits, stock figures and a deployment's own. Its bound is its *since* plus the figure, or the task's deadline where that comes first. The stock is meant to be generous: a bound catches a wait that has stopped, not a slow one. The new kinds' stock figures, the user's of 2026-11-24:

  | Kind | Stock figure | Then |
  |---|---|---|
  | `question` from a task to its requester | 2 h | passed up |
  | `party`, a question to a person (escalations and proposals keep their 1 h stall) | 8 h | passed to the next holder |
  | `news`, for a task with a deadline or one that isn't a standing role | 24 h | held |
  | `host`, and `account` with no known reset | 2 h | held |
  | `step` waiting on a read or read admission | 30 min | held |
  | `room` behind an entry whose owner has ended | 30 min from that end | held |
  | `sign-off` | 24 h | overdue |
- **A wait behind another task is bounded by that task.** A follower waiting for a turn, or a waiter behind the outbox's earlier entries, has no clock of its own while what it waits behind lives: what bounds it is the turn holder's own waits, or the waits of the entries' owners. Its record names them, so a stall shows where it is. A `room` wait behind an entry whose owner has ended, P124's leak, has nobody's wait to end it, so from that owner's end it takes its kind's figure.
- **No bound:** a `dependency` or `delegate` wait in a task with no deadline (it shows its age); a standing role's `news`, since waiting for news is its work; an operator's landing hold, which is a person's act (`../domain/landing-settings.md`). Each shows its age in the views.

A read again at a deadline stays as it is, and is no bound: `poll` reads a fact afresh and the wait goes on (`../domain/landing-waits.md`, 3). The wait also has its bound, counted from its *since*, which no read moves. A wait that reads again and again without its procedure taking a step reaches its bound like any other.

### 4.2 What a bound does

| At bound | Kinds | What happens |
|---|---|---|
| *hold* | `dependency`, `delegate` (at the task's deadline), `news`, `resource`, `account`, `host`, `step`, `room` | The task is held, with words naming the kind, the object, since when and what it last saw: "Waited 4 h for a fresh read of the comparison between ceebfe4a3e16 and 5ddd6bc722da; 412 reads answered, none advanced the round." The hold escalates as any hold does (`jig/docs/design/domain/tasks/lifecycle-holds.md`, 1) |
| *pass up* | `party` at a task or person, `question` to a task | The entry passes to the next holder that covers it, as an escalation's stall does today (`jig/docs/design/domain/tasks/proposals.md`, *Stalled*) |
| *overdue* | an entry at the project's last holder, `sign-off` | The entry stays where it is and is marked overdue, with no timer after: it shows its age in the holder's inbox, in Needs you and in the views (9). A sign-off shows in the Needs you of every role that can still sign |
| *give up* | a forge read that its caller may lose | The read is answered as failed, with words, and the caller's own failure rules apply |

- **A hold keeps what the task owns.** A task held at a bound keeps its place in a queue or in the outbox's line, its reservations, its turn and its effects in flight, as any hold does today. A write in flight is never given up: an uncertain write keeps its read-back as its wake, and its outcome is recorded whenever it comes. The landing train's turn is the third draft's (11).
- **A release starts a fresh wait,** with a new *since*. The hold and the wait before it stay in the task's history and its record's age.
- **A question passed up stays one question.** Whoever may answer at either holder may answer it, once: the first answer settles it, and a later one is refused, naming who answered.

So every wait either moves, or becomes a hold, an entry passed up or an overdue entry, within its bound.

### 4.3 What counts

A bound is wall time from *since*. A worker's suspension or a forge outage the engine records moves it by its gap, as suspension moves a task's deadline today. Time a wait spends on held capacity, a gate starved of cores for one, is a later design's (11).

## 5. A wait that can't end is caught when it parks

### 5.1 What can wake a parked task

A task parked, with no live run, can be moved by its **wakes**: a timer it armed; a subscription to a task that can still end or message it; an entry waiting at a party; or a source of messages its wake policy lets through, a requester's words to a chat or a standing role among them. Its **prerequisites**, dependencies and resource holds, are a separate matter: they are bounded by its deadline and by the queue's bound (4.1), and jig checks them for circles when they are made (`jig/docs/design/domain/tasks/holds.md`, 7).

A wake is **dead** when it can't come: a subscription to a task that waits, through an open question or a held answer, on this task's own answer; a task that has ended without messaging; an entry that has been withdrawn.

### 5.2 Stuck

A task is **stuck** when it parks with an open question asked of it and every wake it has is dead, or when it parks with no wake at all. The check runs in the decision that parks it, walking the edges jig keeps, within the bound of tasks per tree that its reverse walks use today, and again when an edge it walked, or a wake's eligibility, changes. A timer counts as a live wake: a task that keeps waking on its timer and parking again reaches its `news` bound instead (4).

A stuck task is woken at each park with its run's tries counted, and past its tries it is held, with the question's words and the chain that leaves it stuck, by task and question. The hold escalates. Each open question counts on its own, which retires the one wake per newest question (`question_woken`). A watch that can still bring the answer leaves the task alone.

A run's `wait` that would park with nothing awaited answers "nothing more will come" as today. A wait on another goal's task (6.3) means a run no longer keeps a question open just to stay alive.

## 6. Words reach their reader, and their sender knows

### 6.1 A held answer tells its sender

An answer that a change's or a closer's relay holds, for a pending dependency or for want of message room, tells its sender that it is held, by what and since when: "Your answer to question 228316 is held until change 15252 ends." The notice is an entry in the sender's own inbox, whose room is taken when the question is asked, so a relay short of room can still send it, once. When the answer is offered to the producer's run, its sender hears it was delivered. An answer whose asker has ended tells its sender so, as today. The relay's record keeps the blocker and its *since*, so `change show` and the views name them (9).

### 6.2 Nothing is taken and dropped

Every message a route accepts is delivered, recorded, or refused with words saying why and what to do instead. A party's words to a change with no live producer are refused, naming the amendment that would carry them (`../domain/forge/change/questions.md`, 1). A party's words to a change with a live producer pass to it, as a task's do. An answer that matches no open question, at a change or at a closer, is refused to its sender, naming the question that is open.

### 6.3 A wait on any task its party can read

A task may depend on, or watch, any task in its project that the party who requested its tree can read, not only one it references. The admission checks that read access when the edge is made, with the circle check for a dependency (5.1). A later loss of access ends the watch, with a notice to the watcher. A change's `after` may name such a task, and so may a run's `subscribe`. Person and authorized-maker batches keep the wider admission they have.

### 6.4 Subscriptions count alike

A task's subscriptions, to tasks, topics and timers, share one bound in any order: four ordinary subscriptions and a findings topic fit whichever comes first. A task restored with more than the bound keeps them.

### 6.5 An escalation waiting at a task is woken by any change

While an escalation waits at a task whose run left it undecided, any message reaching the task is the change it waits for, and wakes it, a result its wake policy would otherwise let ride along included (`jig/docs/design/domain/tasks/proposals.md`, *Waking*). Other wake policies stay as they are.

## 7. Every fixed table says what a full table does

temper and jig size their tables, slots and rooms by fixed figures. Each one says, in its design, what a request does when it finds the table full: it **waits**, as a `room` wait with its bound, or it is **refused** with words naming the table and its figure. None leaves a request taken and unanswered.

- **A held slot is held only while it works.** A sign-in's exchanges are taken by requests in flight; a parked act or a resend holds none, so the sign-in's other requests are answered (P117).
- **What is stored is checked against its room at start.** A stored charter, template or setting whose fixed part no longer fits its configured room is refused at start, naming it and the room, not at its first use (P63's residual).

## 8. Holds always say why and how out

- **Every hold has words.** They name the cause, the object and since when, and its way out names the act that ends it. A placed hold needs a reason: a party's stop gives one, or its act is refused. A category's phrase alone is no longer enough.
- **A hold waiting at a person shows how overdue it is.** An escalation, proposal or person task waiting at a task or a person passes up at its stall, as today. At the project's last holder it is marked overdue (4.2): its age shows on the holder's inbox entry, in Needs you, oldest first, and in the waits view. A person task left past its stall is offered to the next holder too, and its first addressee can still answer it.

## 9. One wait-for view

- **A task's line** gives its first wait with its *since* and its nearest bound: "waiting on: party 34's answer to question 228316 · since 06:07Z · passes up at 07:07Z". `task show` lists every wait the task has, in order.
- **The project's waits:** a read, `waits --project P [--kind K] [--party N] [--due WITHIN]`, lists every live wait in the project, a line each: the task, its kind and object, since, bound and what happens at it, and who hears. It is ordered by bound, then task number, overdue entries first, and paged. Each wait is listed once, at the task it waits at, not again at each ancestor whose line summarizes it. `--json` gives the records for tools, and the web client has the same list as a page in Work.
- **A milestone** names each live goal's first wait and its bound, in `milestone show` and on its page.
- **A landing's wait** gives its *since* and bound in `landing show` and `landing list`, which name the wait today without its timing. The public landing shows the wait's name only.
- **A party's inbox** gives each entry's age and whether it is overdue, and Needs you puts the oldest first.

The view is a read of what the core computes (3.1), so it is never stale and costs nothing to keep.

## 10. The team's watchers retire

The watchers live outside temper, in the team's directories, and their owners retire them once this draft's milestone has landed and reached the reference deployment:

| Watcher | Replaced by |
|---|---|
| goal watches (agents' `bin/bell/goal-watch`, temper/dev's `goal-watch.sh` and its filters) | holds that escalate to their goal's requester party (8), and the bridge reading the waits view's JSON |
| `health.py` and `tick.sh`, with their filters on hold words | bounds that hold with words (4.2), and the waits view filtered by `--due` |
| `stuck.py`, `devwatch.py`, `inventory-progress.py`, `liveness-tick.sh` | stuck detection (5) and the waits view |
| landing-stall and stale-landing watches | landing waits with bounds (4, 9), and the third draft's liveness of the landing train |
| ops' dashboard and inbox watcher | the waits view's JSON, and the owners' inbox with ages (8) |

## 11. Out

- **The landing train's liveness**, one owner for a landing's members and its turn: the third draft. Its waits use this draft's record.
- **One way to classify a red, and one release rule:** the fourth draft.
- **Provider failures, accounts and funding:** the fifth draft. Account waits are named and bounded here, nothing more.
- **Capacity and held time,** the gate's reserved capacity and bounds that count only time a wait could have moved: a later root design.
- **The figures:** the user's.

## 12. Tests

Focused and fuzzy suites only.

- **Every wait is a record:** failure-worlds' check that every wait has a wake (`failure-worlds.md`, 3.1; `../testing/engine-checks.md`, 2.2) also checks, at each driver's quiet points, that every live wait has a kind, an object, a *since* and either a bound or one of 4.1's reasons for none. The deadline checks between quiet points stay, so a transition in flight can settle and starvation still fails.
- **No wait passes its bound silently:** at each quiet point, a wait past its bound has become a hold, an entry passed up, an overdue entry or a failed read.
- **Stuck is caught, and only stuck:** sweeps that make circles of questions and watches, and waits on nothing, see each stuck task spend its tries and be held, naming the chain; a control whose watch can still bring the answer is left alone.
- **Old records and crashes:** a store the previous build wrote opens with every wait's *since* at the first start, none past its bound; a crash at a bound's decision makes it again once.
- **The catalog's scenarios pass** (`failure-worlds.md`, 7): P43, P85, P113, P117, P124 and P63's residual run and pass, and P86's other order has its test.
- Focused tests for each kind's bound and words, the held answer's notice, a party's refused words, a wait on another goal's task, and the waits read's text, paging and JSON.

## 13. Later

- Bounds learned from how long each kind of wait takes on a deployment, rather than fixed figures.
- A wait-for graph drawn on its page, for a person following a stuck tree.
