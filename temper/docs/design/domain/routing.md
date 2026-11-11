# The root's routing

## 1. From the protocol layer

Each event goes to the child it is for, by its variant: the store's
answers to the journal and to the load that asked (`root.md`, section 7); a
worker's hello and records, a party's requests, reads and watches, a
project's period (`../protocol/configuration/README.md`, section 6), accounts'
events and a git identity's generation to the core; the forge's events
to the forge; a timer to the child that armed it. What temper adds:

- **A worker's bytes stay with the root.** A turn's transcript and an
  answer's outcome and parts are kept under tokens the fleet names, until
  the core takes or drops them; the fleet sees their metadata alone.
- **An ended run's outcome** goes out to the protocol layer to read, with
  the change it may name; its typed end comes back to the core, temper's
  parts translated (`root.md`, 6.4).
- **An adoption,** an application request, goes to the core to admit by
  key, then to the forge (4).
- **A forge's set up,** `SetUp { forge }` (`../client/engine.md`
  ), goes to the forge, which decides it by the forge's offer
  (forge/adoption.md, 1), refusing it there or asking the protocol layer's box
  link, which carries the ask to the box and its lines back
  (`../protocol/forge/engine.md`, 8). The root's configuration carries each
  configured forge's number, kind, mode and setup start, and whether
  serve runs under the box, which the root gives the forge at
  construction; the root's event `BoxClosed`, from the link at the
  pipe's close, goes to the forge. It is keyed: the core admits it by key,
  of the deployment's scope, first, and its answer comes back from the
  link to the forge, which keeps it as the request's outcome, in memory
  alone. Nothing of it is committed: the states and offers it moves are
  in memory, as every connection's is.
- **A watch of the deployment's forges,** the subject `forges`
  (`../client/forge/deployment.md`), goes to the core to admit on the party's
  deployment role, and its documents come from the forge as a change's
  panel's do (below).
- **A hint that comes before decisions open** is dropped, since the
  restart reads afresh (`root.md`, 8.1).
- **A watch of the forge's subject,** a change's panel
  (`../client/forge/served.md`, 2), goes to the core to admit on the
  party's role in its project; the core's ask for its document goes to
  the forge (3), which makes it or answers unknown; the forge's marks
  of its subjects go to the core; and each document leaves held in the
  decision that asked for it, out to the protocol layer to encode once
  that decision's commit is durable.
- **A run's read as its person,** `see`'s for an address of the person's
  (`agent/ask-temper.md`, 1), goes to the core as the run's named action
  `PersonRead` (jig's `domain/engine/executors.md`, 3), its answers back to the
  call's flight; an application read of the forge's it makes, the core
  admits as a party's and asks of the forge with the person and the
  project's scope, and the forge's document for that person, made as its
  watch's is, goes to the same flight. Nothing of it is committed.

## 2. Host calls

- **Fenced, then decoded by hand-out.** The fleet admits a call only from
  its task's live attempt; the root hands it out (`HostRequest::Decode`),
  and the protocol layer decodes its input into the root's `Tool`
  (`../protocol/llm.md`, section 4). No domain parses a call's input
  (jig's `domain/engine/README.md`, 4.4).
- **Checked for shape** against the core's bounds before any child sees
  it: a call past them becomes its refusal, answered and recorded as any
  decided call is.
- **An undecodable call decides nothing.** The protocol layer answers it
  as an error naming what is wrong; the root relays that answer to the
  run with no commit and no record, and the run may ask again with a
  corrected input under a new name. A documentation read
  (`read_docs`) is answered the same way, as a success, from the
  binary's topics (`../protocol/documentation.md`, section 5): it
  decides nothing either.
- **Invalid or busy, answered at once:** a call naming no live attempt or
  no completion is answered invalid; one that finds the call records, or
  the loads it needs, full is answered busy, and its run asks again after
  a backoff.
- **Routed by tool** (agent/tools.md lists them). A call the core has a
  record of is answered from it (`root.md`, 6.3). Otherwise the core checks it
  against its task's authority; then jig's operations, notes and
  subscriptions to tasks and timers go to the core as named actions, and
  the forge's reads, effects and topics to the forge (5); a proposal of
  a forge effect goes to the forge, which keeps it, then to the core.
- **Task reads decide nothing.** `read_task` goes to jig before durable-call
  replay. jig checks the read family and task relationships, including the
  immediate requester retained on an ended delegate's owner row. The root
  translates its bounded answer through the fresh-read door; it makes no
  visibility decision. Escalations name the held task and hold revision and
  tell the recipient to use `read_task` for its source facts and hold account.
- **A task read's room.** The root names the room with the read: the
  reply bound of the agent running the reading attempt, within
  `core.call_answer_bytes` and within `root.journal.transcript_bytes` less
  the most the answer's metadata can take, since the door takes no delivery
  past that bound. That reply bound is the one the attempt's worker names in
  its hello (jig's `protocol/channel/records.md`, 1), as jig's fleet keeps it by
  the attempt (jig's `domain/engine/fleet.md`); where the worker names
  none, the local agent's `host_reply_text`. Every temper worker names its
  agent's `tool_payload` (`worker.md`, section 6), the local agent's
  in-process worker among them. Where neither names one, as for a worker
  that names none in a process with no local agent's metadata (`agent/tools.md`
  ), the room is `documents.text_bytes`: the root does not know that
  agent's reply bound, and an answer past it reaches its run as smith's
  `TooLarge` at every offset. The root answers `Domain::reply_bound` by a
  call's key for the protocol layer's render alike (`agent/tools.md`). A
  read holds its words, at
  most that room and no more than the source has left, and its metadata, from
  jig's answer until it is rendered; `fleet.calls` bounds the reads held at once.
  At `temper-box init`'s figures (`../protocol/setup/init.md`, 1) one read holds at most
  10,662 bytes of words, 4,576 of metadata (16 dependencies at 8 bytes, 82 question
  credits at 48, a hold's account of 512) and a carrier of about 200: about 15.4 KB,
  and about 4 MB at `fleet.calls`' 256, which a box's runs share.
  Where neither the run's worker nor a local agent names a reply bound,
  the read is in pieces of the deployment's `documents.text_bytes`: 512
  at `temper-box init`'s figures since 2026-11-05, and 128 in a document made
  before, at which a goal on a worker that named none read a 30.5 KB report
  128 bytes a call, about 240 calls (the request `goal-call-faults` of
  2026-11-08, item 3). A temper worker names its agent's, so its runs read
  in pages of that bound, 10,662 bytes at `temper-box init`'s figures.
- **Rendered by hand-out.** A decided call's answer goes out to be
  rendered (`HostDelivery::Render`) and comes back as smith takes it; the
  core keeps the rendered answer with the call's record, so a call asked
  again is answered without rendering again. A read's answer leaves
  through the door. A rendered call past the configured core call-answer
  byte bound is refused with the existing busy answer.

## 3. The core's asks of the forge

The closed list (jig's `domain/connectors.md`, section 2), each by the
forge's number:

| The core asks | The root |
|---|---|
| describe, keep or drop an effect; judge a requirement | gives it to the forge (5) |
| judge items of temper's kinds of settings; keep their values; which of their defaults differ | gives them to the forge (6) |
| the holdings of a task, of a delegate batch, of a procedure's batch | has the forge name the branches each task will hold, before the core admits the batch |
| take or hand down a hold; end a topic | gives it to the forge |
| a claim | claims the forge's writer slots for the attempt's branches in the claim's decision, and starts the run through the fleet, with its grants, once the claim commits (`root.md`, 6.1) |
| gather, cut or drop a brief section | gives it to the forge (`root.md`, 6.2) |
| adopt a repository; an application admitted, answered or dropped | gives it to the forge (4) |
| an attempt ended | gives the forge what the attempt pushed, then has it read its branches afresh before their writer slot frees (`root.md`, 6.4) |
| close, release | has the forge settle the task's effects, then release its holds (forge/branches.md) |
| project a goal; forget a projection | gives it to the forge's issues (forge/issues.md) |
| a subscription made or dropped | makes or drops the forge's subscription under the core's number, and answers the call |
| start a procedure | starts the forge's change procedure (4) |
| a step made or refused; a delegate or a repair refused | records the change's delegate, or holds the change |
| a document or a read of the forge's; a document dropped | gives it to the forge, which makes it from its records and working set, within the scope the party named (forge/README.md, section 2; `../client/forge/served.md`) |

## 4. The forge's routes

What temper adds to the core's usual routes (jig's `domain/engine/README.md`,
4.4):

| What arrives | Route | Then |
|---|---|---|
| a forge API answer, or a hint | the forge: its working set (forge/keeping-up.md, 2); each change it moves, stepped | commit; news to subscribers through the core; changes stepped |
| a change due | the core starts its procedure → the forge's change procedure, on fresh facts → its decision in the core's terms (`root.md`, section 5): a delegate, an effect, a wait, a hold, its result | commit; delegates made; effects made after the commit |
| a merge's outcome | the forge → the change's result; the landing as news on its branch's topic → each goal's subscription, waking it by overlap (forge/README.md, 5.3) | commit; the change closes, its branch released |
| a landing branch's CI failing | the change procedure asks a queue repair → a change task into that branch, first in its queue (forge/queues.md) | commit; the repair runs |
| an adoption | the core's people child admits it by key → the forge locates the repository → jig's adoption request on the party's behalf (`adopt_repository_ask`), which the core judges as the party's own → the forge reads what temper may do there, and seeds collaborators' roles (`root.md`, section 5) | commit; the outcome kept under the request's key, answered once durable (forge/adoption.md) |
| a goal changing | the core → the goal's issue → its effects | commit; effects made |
| a task closing | the core → the forge settles the task's effects, then releases its branches | commit; releases made |
| a run's answer | the fleet → the core, temper's parts translated (`root.md`, 6.4) | commit; acknowledged |

## 5. Effects and verdicts

- **From an agent's call:** the forge describes the effect (its kind, the
  resource's name, temper's access to it) and keeps it under a token; the
  core checks it against authority and asks for its verdicts; the root
  routes each to the forge's judges and back; the core's answer (allow,
  propose, wait, refuse) goes to the forge, which enqueues the effect
  under a key derived from the call, keeps it as a proposal, or drops it.
  All within one step, one decision (jig's `domain/root.md`, section 9).
- **From the change procedure:** its effect is described and checked the
  same way before the step's decision is complete; one the core refuses
  leaves the change's state as it was.
- **The forge is every requirement's judge:** temper's landing rules are
  the forge's criteria (`root.md`, section 5; forge/change/gates.md, 2). The forge judges two
  standing requirements over the landing rules it holds
  (`../protocol/configuration/settings.md`, 3), and the root gives it, with
  each ask, the merge's resources by the forge's names, the path read
  back as `root.md`, section 5 makes it.
- **Made after the commit:** the core's held output to make an entry is
  released once its commit is durable, fed back to the forge, which makes
  its API call (`Request::Forge`) behind every commit before it.
- **Priced at nothing.** Only LLMs are counted: the root gives no forge
  effect a price, and a forge request is load, within the forge's
  request budget (forge/keeping-up.md, 4), never spend.

## 6. Settings

The settings of temper's kinds are `../protocol/configuration/settings.md`
's.


- **temper's kinds are the forge's.** A change of settings naming items
  of temper's kinds reaches the root within the change's decision, as a
  requirement's verdict is asked (jig's `domain/engine/settings.md`, 5); the
  root gives each item to the forge, which accepts or refuses it with
  its reason, and carries the answer back. In the decision that commits
  the change the root gives the forge the core's word to keep the
  items' values, which the forge writes to its family `Setting`; a
  proposal's items are kept and ended likewise; then each live change
  the rules cover is stepped. At a start the root hands the forge
  temper's stock defaults of its kinds, and the core the forge's answer
  of which differ (jig's `domain/root.md`, section 8).
- **Typed at the entrance.** No domain decodes a value (`root.md`, section 10): an
  item of temper's kinds crosses jig as bytes, the encoding of
  `temper-client-protocol-wire`'s `LandingRule` or a charter's number,
  and the protocol layer decodes it where it decodes the request that
  carries it, a party's change or proposal of settings as the clients'
  component hands it in (1), a run's `propose_settings` as
  `temper-engine-smith` decodes the call (2). The root receives jig's
  request, bytes and all, with the typed values beside it, keeps them in
  flight for the request's decision (`root.md`, section 2), and gives the forge the
  typed values where the core hands it the items; a reset's and an
  accepted proposal's values are those the forge holds already. A value
  that does not decode is refused at the entrance as out of shape,
  naming the item, before the core sees the request. The forge's family
  `Setting` keeps the typed values, encoded by `temper-protocol-store`
  as every family of the forge's is (`../protocol/store.md`, section
  2).
- **The forge judges rule structure; the root joins approval eligibility.**
  When a rule is set, including acceptance of a proposal, the root checks
  its enforcing approvals against the core's known forge identities and
  project roles (forge/change/gates.md, 3). It refuses an unmet count before any
  settings become effective, naming the project, role and count with the
  remedy. Meaning and structural validity remain the forge's.
- **The forge's charter is the forge's own value,** read from its kind
  3 when a change is made, and kept on the change (`root.md`, section 5).

Both settings entrances decode a value of temper's kinds at the client
wire's schema ceilings for pattern segments, segment bytes, gates and
approvals. A value that does not decode whole is refused as out of shape at
its item. A value within those ceilings but beyond the deployment's bounds
reaches the forge, which refuses it as judged with its own pattern, gates or
approvals reason (decided 2026-11-03).

## 7. Periods, from the calendar

A project's spend is per period (jig's `domain/authority.md`, section 7),
and jig keeps no calendar: a period begins when the root says so (jig's
`protocol/configuration.md`, section 5). temper's root has a third
child, the calendar, which renews each project's period.

- **One span for the deployment,** the stock's 7 days
  (`../protocol/configuration/stock.md`), from each project's first
  period: a project's periods are weeks from the moment it was first
  funded. The calendar decides; the protocol layer only fires its
  deadline.
- **At a deployment's first start** the smallest source's periods open
  as they do now (`../protocol/configuration/README.md`, section 6), and the
  calendar says each one's next start, a span after its start.
- **A project made after it** has its first period opened by the core
  in the decision that makes it, with no next start (jig's
  `domain/people/acts.md`, 2); the root hears it was made, and the
  calendar says that period's next start, a span after it began.
- **At a next start** the calendar says the project's next period to
  the core (jig's `Event::Period`: the project, the period's number one
  more, its budget the project's spend per period as its policy holds
  it then, its next start a span later). What the last period reserved
  stays its own and returns to it (jig's `domain/authority.md`, section
  7); the new one starts full.
- **At each start,** before decisions open (`root.md`, section 8.1, step 6), the
  root gives the calendar each restored project's current period (its
  number, start and next start) and its policy's spend per period, from
  the core, and the calendar:
  - for a period with no next start (a first period whose decision
    committed before the calendar said one, as after a crash), says one,
    a span after the period's start;
  - for a next start already past, opens one new period at once, from
    the start's time, and not one per span missed: a deployment down
    for three weeks opens one week, and that project's later periods
    count from it, its alignment to its first period given up;
  - arms its deadline for the earliest next start.
- **Its deadline** is a timer of the root's, as the forge's are (1):
  armed through the journal for the earliest next start, a wall time,
  and armed again after each period it says, at each start, and when a
  project is made. When it fires, the calendar says, in one decision,
  every project whose next start has come, each its own `Event::Period`,
  within jig's project limit; a clock that moved meets the next firing.
  A stopping engine arms none (`root.md`, 8.2).
- **Bounded:** one deadline armed, the earliest; one record per project
  in memory, within jig's project limit; nothing stored of its own, the
  periods being the core's records. A renewal's decision takes two
  journal writes per project it renews (its period's ledger and its next
  start), within the commit's bytes in the store.
- **Seen and raised:** a project's document carries its period's start
  and next start (jig's `client/domain/contract/documents.md`, 1), and `temper
  ctl project show` prints them with what the period has left; `temper
  ctl project fund` raises the project's spend and a role's, the open
  period's with them (`../client/ctl/tools.md`, 3).
