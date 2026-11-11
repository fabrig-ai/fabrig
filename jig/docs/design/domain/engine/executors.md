# Agent executors

## 1. From a task to a run

When an agent task is due (../tasks/lifecycle.md, 2), the core does as below. A
kept task is never due (../tasks/lifecycle.md, 8), and one kept while it prepares
has its claim refused, its preparation abandoned with no try spent; it
is due again once its dependencies are done.

1. **Checks** that the run may start, and with what budget
   (../authority.md, 8.3), takes its task's LLM choice: its `later` when the
   task's reasons hold no `first` and its charter doesn't resume, its first
   choice otherwise (../tasks/README.md, section 3, and ../tasks/messages.md, section 6); takes its models where
   its task was given none, the first it can use its main (../accounts.md,
   4.3), and chooses the
   accounts that pay for it: one
   for its main model's LLM provider, kept by its task, and one for each
   other LLM provider its sub-agents' models use, each with its first
   source usable on a host with room (../accounts.md, 4.3).
2. **Prepares it,** the task preparing over as many steps as it takes:
   takes its reasons, `first` and the kinds that woke it, which the tasks
   child gives with the activation (../tasks/messages.md, 6), and chooses its
   charter's parts those reasons meet, at the revision its preparation
   began under (`settings.md`, 3), which make its instructions (step 4) and the kinds
   it is offered (a message or an entry that would add to those reasons,
   arriving before the claim, prepares it again: ../tasks/messages.md, 6); gathers its brief (`briefs.md`, section 1), for why it runs; has each
   connector that owns a resource the task
   names gather its workspace items, and name the application's
   credentials they need (../connectors.md, section 10); and loads its
   transcript's saved window when it has one, for resuming or for the
   tail of a fresh brief after its window closed (2). An activation
   that needs this load stays due while the root is busy; one with no
   saved window can prepare its brief then. Either way it is given the calls
   committed after its task's last committed turn: a resumed run as a
   note after the transcript, a fresh one as a section of its brief.
3. **Commits the claim:** the task claimed, with its new attempt number,
   the revision its preparation began under (`settings.md`, 3), the host kind it is placed on, the accounts and sources its check
   chose, the task's kept account among them (../accounts.md, 4.3), the
   writer slots of the holds its workspace writes (../tasks/holds.md, 3), its
   reasons and the kinds it is offered (../tasks/messages.md, 6), and the messages
   its start carries, those it is offered (../tasks/messages.md, 2).
4. **Places it** through the fleet, once the claim is durable, on a host
   its charter allows, where its sources are usable and the host rule
   allows it (`fleet.md`), with an assignment the root puts together
   (../root.md, section 7):
   - its run and attempt, the core's names for them, and its
     conversation's first attempt, from which the engine's protocol layer
     derives the run's identity, for the LLM provider's prompt cache,
     the same for every attempt of a conversation and never shared by
     two (`../../protocol/channel/README.md`, section 6). No domain holds it;
   - its workspace: the connectors' items, and a workstream key, the
     number of the task that holds the resources written, so that the
     runs of one piece of work find their workspace cached;
   - its start's messages, those it is offered, named, oldest first; the
     rest it is offered are relayed as the run reads (4);
   - its **ask** for live content (`../hosts/README.md`, section 2): `text` set when
     its task is flagged conversation (../tasks/README.md, section 3), and `input`
     set by the same rule; `reasoning` is false. The core sets it with the
     run's prepared parts (step 2), from which the root puts the assignment together;
     the root copies it, kind by kind, and no host decides it. A task a
     conversation delegates is not flagged, so its run is not asked
     (`../hosts/README.md`, section 10);
   - its charter, in the core's terms: the task's instructions, its own
     or else its charter's followed by each part its reasons meet, in the
     charter's order, each after a blank line (`\n\n`; ../tasks/messages.md, 6); the
     brief, the tools of each host family its charter names and its
     authority holds (the charter's `tools`, a `ToolFamilies` mask, the
     same type as an authority's: a family neither names is not offered,
     so a run's host part spends nothing on it), the outcome its result
     contract asks for; its budget for the run: turns, spend and time,
     with a reserve of turns and time for finishing, its spend reserved
     from what its task has left in the claim's commit (../authority.md,
     section 7); and its models, from its task's LLM choice (../tasks/README.md,
     section 3): the main model with its effort, and those its
     sub-agents may use, each with its effort, its window (the input
     tokens a conversation's window may hold), its output (the most
     output tokens a completion may have) and its prices (../authority.md,
     section 7).
     The engine's protocol layer translates it into smith's (`protocol/charter.md`,
     section 2);
   - its transcript's current window, when it resumes, and the calls
     committed since its last turn;
   - its credential grants: for each account its check chose, its
     source's (../accounts.md, 3.4), a value's current generation with how
     it is sent, or a program source's name, which has its agent run the
     program backend (smith's `protocol/programs.md`, section 3); and
     the application's credentials its connectors named (`README.md`, 6.1).
     Each names its form; the form selects the table the protocol layer
     fills its value from (`../../protocol/channel/README.md`, section 7).

The check's allowance carries its checked spend, retained models, a concrete
channel and kind, and one provider/account/source binding per retained provider.
Main and same-provider sub-agents share their binding. A provider with no usable
serving account contributes no optional models; a usable optional provider's
retained models constrain the same host as main. Fleet preserves affinity and
free-slot preference among those hosts. An undeclared unused alternative does
not block the run; a declared model with inadequate bounds still refuses hello.

Preparation facts are read again after asynchronous work and before the claim.
The successful task claim keeps main's account in the same commit as the core's
complete bindings. A changed selection abandons preparation without a retry
timer. Start and queued placement retain the selected channel; loss of room
returns the claim for a fresh check, releasing its reservation without a failed
try. Source failures are authenticated against that claim, and their ordinary
terminal charges actual spend once and releases writers. A recovered refresh
cannot turn an unrelated later failure into an account wait.

Restoration refuses duplicate provider bindings and a surviving source whose
account or provider disagrees. Removed rows remain legitimate history. Adoption
offers grants only from the recorded surviving sources, never replacements;
an unavailable recorded grant can renew on its original source, and an absent
worker settles through the fleet's grace (`fleet.md`). A new assignment rechecks the
recorded bindings and current coverage.

The check joins each candidate host's kept program logins to the accounts' issued
names and their original holders. Every bound account must satisfy that host's
rule, including retained providers. Deployment-held logins restrict no run;
a deployment with at most one person is unaffected. A program source is chosen
only where its login is usable; the engine never takes one. A program source's
run is granted its source's name and runs on smith's program backend, for the
program its endpoint names (`../../protocol/configuration.md`, section 3).

## 2. Transcripts

- **A transcript belongs to its task,** whichever runs added to it. Its
  conversation is the turns from the last run that started fresh, and
  its current window the conversation's turns from the last that opens a
  window: a window turn, which a compaction made, or the conversation's
  first (smith's `protocol/transcript.md`, section 3). That conversation
  is the transcript's, which every agent task has; a task flagged
  *conversation* is another matter, whose turns are entries for its
  requester (`views.md`, 4).
- **Beside it, the task's activity:** the words parties wrote to it as
  they wrote them, its runs' beginnings and ends, the results and news it
  heard and the decisions made about it, kept in the order they
  committed, so a client reads a conversation whole after its words have
  been taken from the inbox (`../../client/domain/contract/README.md`, section 4).
  A failed run's end retains its class and the optional invalid-result
  kind computed for that attempt (../tasks/lifecycle.md, 7), beside the task record's
  current reason. Starting another attempt clears the current reason,
  never the ended run's activity.
  A message is a line in the activity of the task that hears it
  (../tasks/messages.md, 1): a party's answer to a question a task heard is that
  task's own line; a task's answer to the delegate that asked is the
  delegate's line, never repeated in the answering task's activity,
  whose turn holds the call that sent it; a read of the question joins
  its answer by a point read (`../../protocol/store/loads.md`, 3). Who may read
  an answer is the client's contract's (`../../client/domain/contract/README.md`,
  section 4).
- **Committed turn by turn.** A run sends each turn up as it ends: what
  the LLM said and the calls it made, their results, what it spent, the
  last inbox message it read, and whether it opens a window. The core
  commits the turn with its spend, with the messages offered to its
  attempt up to that one taken but those it says the run refused
  (../hosts/hub.md, 4): a message its attempt wasn't offered, by its parts or
  a pace, stays, whatever the fence passes (../tasks/messages.md, 2 and 6); and, when it
  opens a window, with its task's current window starting there; then
  it acknowledges it. A host keeps each turn until it is acknowledged,
  and a run with too many unacknowledged turns waits (../hosts/hub.md, 4).
  Admission is whole: the tasks child validates the attempt, turn and
  offered read-through before changing its spend: the read-through names
  a message offered to the attempt, still waiting or, for a proposal or
  an escalation, decided since it was offered (../tasks/proposals.md), and
  none past the attempt's highest offer; a refused turn changes nothing,
  and a duplicate accepted turn changes nothing again. The core keeps
  the ids it offered an attempt with the run's proof until a turn's
  fence reaches them or the run refuses them, so such a fence survives
  the entry's decision and the core's restart, and they hold their room
  in the run's inbox until then (4).
- **The core never looks inside.** A turn is bytes of the agent's own
  conversation vocabulary, bounded, which a client's protocol layer
  renders. The core reads only its size, its spend, what it took and
  whether it opens a window, which the turn carries beside its bytes.
- **What a turn's calls to the engine made is the core's to say.** As
  it commits a turn, the core writes beside it, from the parts of the
  answers it committed for the turn's calls (3), what each call made,
  under the call's name (its completion and its position,
  `../../protocol/channel/README.md`, section 6), as
  `../../client/domain/objects.md`, section 10, shows them: the tasks a
  delegation made, the proposal a `propose` made, a decision's object,
  what an effect made, a note. A call it holds no part for names
  nothing. No layer reads what a call made from its answer's words,
  which its protocol layer renders for the LLM (../hosts/charters.md):
  smith's transcript keeps a host tool's answer as text alone (smith's
  `protocol/transcript.md`, section 2).
- **Resuming.** A run that resumes is given its current window only,
  within the resume limit (the deployment's bound on the current
  window's bytes, across the attempts it spans, as the window crosses to
  a host: `../../protocol/channel/README.md`, 5.1), and, as a note after
  it, the calls committed after its last turn, with their answers. A
  start that crosses a worker's link carries those answers within the
  link's room: the newest whole and each older host answer cut to fixed
  words saying the call ran and how many bytes its answer had
  (`../../protocol/channel/README.md`, 8, *A resumed start fits its room first*),
  so however many calls a run made since its last turn, it is told of
  each and resumes. The core keeps every call as committed. One
  past the limit starts fresh from a brief that carries the transcript's
  tail. Earlier windows' turns stay readable by parties
  (`../../client/domain/contract/README.md`, section 4), and are never sent to a
  run. A transcript the agent cannot use fails its run as transient,
  saying so, and closes its window (its run proof's `CurrentWindow`
  marked closed): the next run is prepared fresh from that window's
  tail, and never resumes it. **Only the window's own history closes
  it:** a transcript the agent refuses or reports unusable, or a window
  whose load is unread. Any other failure, transient ones included (a
  preparation, a brief's gathering, a workspace another run holds), and
  an interruption (`restart.md`, 1; ../hosts/hub.md, 6), leave the window open, and the
  next attempt resumes it within the resume limit, whatever tries the
  task spent before. If a closed window's tail is unread, preparation
  continues fresh without the tail and spends no transient try. An
  open window whose load is unread counts one transient preparation
  failure and is closed; its retry continues fresh without that tail. A
  failed load, rather than an unread window, remains a preparation
  failure and closes nothing.
- **A turn too large for a page.** A window is loaded a page at a time,
  each within the store's page room. A turn past that room is loaded
  cut: its fields and its length, never its bytes
  (`../../protocol/store/transcripts.md`, 3), handed to the core in its place as
  `TranscriptLoad::Cut`. The core counts its length in the window's bytes,
  keeps the fixed text `[a turn too large to load]` in its place in the
  tail (cut to the tail's budget as any turn's text is), and resumes no
  window that holds one: the run starts fresh from a brief carrying the
  transcript's tail, as one past the resume limit does, a transition
  that counts nothing, no failure, no transient try, no refusal. So a run's
  window is the transcript's tail that fits, never a failure for the
  window's length or for one turn's size: a window's load fails only for
  what could not be read (a frame unread or refused, below) or a page its
  root could not take.
- **A window closed by an upgrade.** A turn is handed to a run in its
  current version, patched as read when it was written in an older one
  (`../../protocol/store/transcripts.md`, 6), and a run resumes from it. A patch
  marked fresh says instead that windows across it start fresh: smith's,
  when what its turns mean to an agent changes. Patching then closes
  each live task's current window in the copy, before the switch
  (`../../protocol/store/opening.md`, 2, step 4): its run proof's window keeps
  its attempt and turn, and is closed. The core restores it closed, and
  the task's next attempt starts fresh from a brief carrying the
  transcript's tail, as one past the resume limit does: the closed
  window's turns are loaded for their text alone, none is resumed, and
  the new attempt is its conversation's first. It is a transition, never
  a failure: the attempt counts as any attempt does, its brief's tail
  within the tail's budget, and no transient try, failure, refusal or
  unread transcript is counted; its activity shows a run begun fresh,
  as one past the limit. A closed window's tail the store cannot read
  (a frame unread, `../../protocol/store/transcripts.md`, 3) is left out: the attempt
  starts fresh without it, and nothing is counted. When a turn's rendered
  text needs cutting, a tail budget too small for the 19-byte cut marker
  gives no text for that turn. If no turn leaves text, the tail is empty:
  the closed window's next attempt still starts fresh, without a failure
  counted. A window page that cannot hold one turn hands that turn on
  cut (above). An open unread window counts one transient failure and
  is closed, its retry continuing fresh without that tail, as for an
  unread closed window.
  However the upgrade's
  stop classes the attempt it ends (temper's `draft/upgrades.md`, 7.2),
  a closed window is kept for its tail's load and never resumed, and an
  open one is resumed as any is (above). The run proof
  keeps the window as a `CurrentWindow` (its attempt, its turn, whether
  it is closed: core-v6's `Window`); the transcript a run prepares keeps
  the pair it loads from, and whether it is closed beside it.
- **No attempt goes on across a fresh mark.** An upgrade across one is
  a full one, its workers stopped first (temper's `draft/upgrades.md`,
  section 9), and the core holds to it whatever was stopped. The walk
  that closes the windows marks every run proof it visits `crossed`
  (core-v6's `RunProof.crossed`; `../../protocol/store/opening.md`, 2, step
  4): a claim resuming an earlier attempt's window, one whose attempt
  has yet to commit a turn and has no window, and one whose window an
  earlier mark closed alike. At the restart a crossed claim is never
  adopted (`restart.md`, step 3): in the engine's process it is settled at
  once, lost or not started as at any stop; on another worker its run is
  stopped when the worker lists it, and the claim ends lost once the run
  has stopped or the engine's grace has passed, as a claim lost with its
  worker does, so its writer slot is freed only once nothing of it may
  still write (../tasks/holds.md, 3). The mark is on the proof, so a crash after
  the switch, before the restart ends the claim, ends it at the next. Each claim's preparation writes a proof of its own, never
  crossed, so the next attempt is adopted at a later restart as any
  claim is, until another mark crosses it. So every window the next
  attempt opens is its own, and open.
- **Turns are numbered within their attempt,** from one. The task keeps
  the last turn committed for its live attempt, restored with the fleet's
  adoption of the claim, so a copy already committed is acknowledged
  again and dropped, after a restart too.

## 3. The engine's tools

A run reaches the engine through smith's host tools, declared in its
charter, which its host relays; their schemas, decoding and rendering
are the protocol layer's (../hosts/charters.md). Each is checked against
its task's authority (../authority.md, section 8), decided, committed, and
answered after the commit:

`delegate` and `amend` may name an `llm` choice: a main model and optional
effort, sub-agent model/effort pairs, and an optional account number. Host-tool
JSON names models and efforts by their configured indices; account numbers
are durable names. Omitting the choice keeps the delegate's charter default
or the amended task's existing choice. A choice grants no additional models:
the core checks inherited authority and the prospective financial payer.
A run's choice names no `later` (../tasks/README.md, section 3): a delegate it makes
takes one choice for every run, and an amendment it makes replaces the
amended task's choice whole, leaving it none.

| Tool | What it does | Answers |
|---|---|---|
| `delegate` | makes a batch of tasks as its delegates, each with what a party's `Create` names, its label, flags and LLM choice among them (../tasks/README.md, section 4) | their numbers and their specs' revisions, and what a budget given `AsAllowed` was given; refused, and why; beyond authority, naming what it lacked and whether a proposal could be accepted |
| `read_task` | reads its own task, its requester, live direct delegates, explicit references or escalation subjects in its project; retained ended direct delegates remain readable, and a proven requester chain permits descendant report reads through retained intermediate tasks | bounded specification words by default, or kept terminal report words with `field=result`, with the selected verdict code alongside verdict reasons; lifecycle, dependencies, spec revision, hold revision and reason, including connector hold words; durable unanswered question credits with their numbers, authorized answerers, addressed parties and activity positions, bounded by `tasks.inbox_messages` and available after the question delivery is consumed; `from` continues the selected field at the printed `next` byte offset |
| `message` | words, a question or an answer to a task it references | sent; no reference; the inbox full; too large for a run to read |
| `amend`, `release` | a task below it: an amendment naming its spec's revision as the run last read it, but for a change of priority alone, and adding or removing dependencies within its reach (../tasks/changes.md); a release naming its hold's, and, for an effect held uncertain, whether to make it again (../tasks/lifecycle.md, 6 and ../tasks/changes.md) | done, with the task's spec revision after the commit, the one its next amendment names (a release moves none); refused, and why; moved, with what it is now |
| `cancel` | a task below it and its tree, with a reason, and a grace if the run gives one, within the tasks child's bound: the task is asked to finish first (../tasks/changes.md) | done; refused, and why |
| `decide` | accepts, rejects or passes up a proposal or an escalation waiting for it, an escalation naming its hold's revision (../tasks/proposals.md) | done; beyond its authority, and passed up; moved, with what it is now |
| `propose` | an action beyond its authority, with a reason: a batch, an amendment, an effect, or a widening of its own authority (../authority.md, section 9) | proposed, and to whom; refused, when nobody may accept it (../tasks/proposals.md) |
| `withdraw` | its own pending proposal (../tasks/proposals.md) | withdrawn; already decided, and how |
| `move`, `succeed` | moves a task below it, or that it was introduced to and may amend, under another live task it may amend; names a live task the successor of an ended task below it (../tasks/changes.md) | moved; named; refused, and why |
| `hold`, `lift` | places a hold of its own, stop or drain, over the subtree of a task below it, with a reason, and optionally the event that lifts it: a connector's event its topic offers, a task's end, or a time (../tasks/lifecycle.md, 10); lifts a hold it may lift, naming its revision | the hold's number and revision; lifted; refused, naming its owner, and whether a proposal could be accepted |
| `settings`, when its charter gives the family | reads the settings, or one setting whole, as a party's read does (`../../client/domain/contract/documents.md`, 1), or one proposal of settings, a task's of its own project or a party's, as a party's proposal read gives it (below); proposes a change of settings against the revision it read, with a reason (`settings.md`, 6) | the settings; the proposal, with its review and its outcome; proposed, and to whom; stale, naming the newest; refused, when nobody may accept it |
| `introduce` | two tasks it references, each given a reference to the other, within their limits (../tasks/messages.md, 5) | done; no reference; refused, and why |
| `subscribe`, `unsubscribe` | a topic a connector offers to agents (../connectors.md, 5.3), under a read grant on the resources the connector says it concerns (../authority.md, 8.5); a referenced task's state; a timer (../tasks/messages.md, 4) | done; refused, and why |
| `effect` | one of a connector's kinds of effect offered to agents (../connectors.md, 4.5) | made, with what it made; committed and pending, past the call's deadline; waiting for a verdict; failed; refused, naming them, for a kind that waits for dependencies while its task is kept (../tasks/lifecycle.md, 8) |
| `note`, `recall` | writes a note in a scope, whole or by a line range; reads entries by name or by a search (`briefs.md`, section 2) | written; the entries |
| a connector's reads | reads its system (../connectors.md, section 9) | the answer, within the read's budget |
| an application's asking tools, when its charter gives the family | what the application declares to a run that explains to its person: the reads it serves, among them reads as its person (below), and the document it keeps for that run, each call decided by the asking family alone, whatever it reads or writes (../authority.md, section 3); the application answers it | the application's answer; beyond authority, naming the family |

**A run's reads are not recorded.** A connector's read, `read_task`, and the
`settings` family's reads decide nothing: checked against the task's
tools, served from the state as it stands, their answer leaves through
the journal's door for outputs that decide nothing (../root.md, section
3), and no call record keeps it. A read asked again under its name is
read again, and may see what changed since. A proposal of settings is a
decided call, as every other is.

**A run's proposal read** reads one proposal of settings by its proposer
and number, a task's, `t` its task and its proposal's number, or a
party's, `p` (../tasks/proposals.md), and answers what a party's proposal read
answers (`../../client/domain/contract/README.md`, section 4): a proposal waiting,
from the core's state, with its reason, its items and its review against
the newest settings (`settings.md`, 6), a value past the review's bound by its size; a
proposal decided, withdrawn or replaced, from its history row in the
store, with its reason, its items' heads, its state, who decided it and the
revision it made. It is checked as the family's other reads are, by the
`settings` family of the run's task, which already reads every waiting
proposal's head in the inventory, and no party's visibility stands in for
it: a run has none. A task's proposal is read only where that task is of
the reading task's project, and only a proposal whose action is settings
is read; any other, or none, answers unknown, as a setting that is not
there does. A stored read is checked again when its row loads: the
family, the project and the run's current attempt, as `read_task`'s is.

Task reads authorize against the run's own task relationships, not a person's
project visibility. A stored delegate's owner row proves its immediate requester.
For descendant reports, each requester link must be proved by a live or retained
record in the caller's project, with consecutive depths, up to the caller within
the task depth bound. Retained intermediate changes remain proof after ending;
a shared root or an unproved ancestor number alone grants nothing. The store's
task range supplies the linked records without restoring ended tasks. Loading
grants no message, amendment or subscription authority. Authorization and the
current attempt are checked again after a stored read. `field` accepts only
`specification` (the default) or `result`. Report, NoChange, Change and Verdict
results serve their kept words, including reports kept with cancellation. A
Verdict result read also carries its selected code as optional metadata, on
every chunk, separately from the bounded reason words. Specification reads
and other results carry no verdict code. Offsets and totals count only the
reason bytes. An unavailable terminal report is unknown. Both fields use bounded UTF-8 chunks
and reject offsets inside a character or past the source. A run's chunk is
bounded by the room its application names with the read, held to
`core.call_answer_bytes`, and not by `documents.text_bytes`, which is what a
client decodes (`../../client/domain/contract/README.md`, section 4): the application
knows what its run's reply holds. An application that cuts a chunk further to
render it prints `next` from its cut. Continue with the same
field and the printed `next` offset. Every chunk names the current spec revision; restart specification
reads at zero if it changes between chunks.

**A run's read as its person** is how an application's asking tool reads
for a run that explains to its person: one of a party's reads
(`../../client/domain/contract/README.md`, section 4), read as the run's task's
requester, a person, would read it, and of that task's project alone. The
application names it as the run's call, the named action `PersonRead`
with the party's `Read`; it decides nothing and keeps no call record, as
the run's other reads.

- **Checked first:** the asking family of the run's task (../authority.md,
  section 3), beyond authority naming the family; the run's current
  attempt; and its task's requester, which must be a person: a task
  requested by a task or by the deployment reads nothing as a person, and
  is answered unknown.
- **Of its project alone.** Admitted are the reads that name the task's
  project, or one task of it, and no other:
  - *naming the project:* the project and its spend (`Project`, `Spend`,
    never project zero's), a task query whose project key is it (`Tasks`),
    its notes in a goal's scope (`Notes`), notes of an exact scope that
    is the project's or a goal's of it (`ScopedNotes`), an inbox group
    whose filter names it (`Inbox`), an application's read in its scope
    (`Application`);
  - *naming one task:* its overview and words (`Task`, `TaskText`), its
    activity, a page beside a line of it and a turn (`Activity`,
    `ActivityBeside`, `Turn`), its ended delegates and history
    (`EndedDelegates`, `History`), its result and a reply (`Result`,
    `Reply`), its question, escalation or person form (`Question`,
    `Escalation`, `PersonTask`), and a proposal it made (`Proposal` of a
    task), where that task is of the project.

  Every other read is answered unknown, as a party's read of what they may
  not see is: the frame and the deployment, which span projects; the
  settings, which the `settings` family's reads serve (above); a party's
  proposal; and a read naming another project, or a task of one, though
  the person may read it from a client. So nothing of the person's other
  projects reaches the run.
- **As the person:** a read admitted so is made as `Read`s are for a
  party (`README.md`, 5.3, `views.md`), the requester its party: their visibility,
  their pages and bounds, their loads. In place of a sign-in, a stored
  read's completion checks again the run's current attempt, its
  requester, the person's role in the project, and that what the row
  names is of the project, and answers unknown where any but the attempt
  has moved; an attempt no longer current is answered nothing, as the
  run's other stored reads are (above). An application's read goes to
  its connector as a party's does, with the person and the project's
  scope, and the connector answers it to the run's call through the root
  (../connectors.md, 5.5).
- **Answered** to the root as the read's own answer, the document, the
  page, the turn, or why there is none, busy among them; the application
  renders it for its run. It marks nothing seen and moves no read
  position. Every turn it answers, on its own or in an activity page, is
  marked withheld, whoever requested the turn's task, since what the run
  reads joins its own transcript, which others read (`briefs.md`, 2.1, *Its runs'
  parts are its party's*).

**Calls are named, and decided once.** Every call names its task's
attempt and the run's own name for it. The core commits a call that
changes state together with its part of the answer, and answers a call
it has seen before from that record, never deciding it twice; what a
connector adds to an answer (what an effect made) it keeps under the
call's key, and the root puts the answer together again (../root.md,
section 7). The core keeps a task's records, by attempt and name, until a
turn of the task that carries their answers commits, or the task ends;
so calls an attempt made in a turn it lost are still there for the next
attempt to be told of (1). A run whose call was withdrawn, lost with
its channel, or answered busy asks it again with the same name while it
lives, so the LLM is told the outcome rather than trying anew; an
effect's key is derived from the call that asked for it (../connectors.md,
4.4).

Three tools a run has are not the engine's: `wait`, which the run and its
host serve, holding the slot until a message arrives (4); `finish`,
which the run serves, checking its result against the contract (smith's
`domain/run.md`, section 7), and whose result reaches the core as the
run's answer (../tasks/lifecycle.md, 7); and `deliver`, which its host serves
through the application's workspace (../hosts/README.md, section 7).

Beyond authority, a call is refused naming what it lacked; the run may
`propose` the same action with a reason. Calls never become proposals on
their own, so a person is never asked by accident.

**The tools are a party's operations.** Each engine tool that acts on
tasks, `introduce` aside, is one of the operations a party's requests
make (../people/acts.md, 1), checked by the same rules (../authority.md, 8.1 and
8.4), so a run and a party do the same things to the same tasks:
`delegate` is `Create`;
`message` is `Message`, and `Answer` for a question; `propose`,
`withdraw`, `amend`, `cancel`, `release` and `decide` are theirs by
name, and so are `hold` and `lift`. What only parties do stays theirs (../people/acts.md, 1): carrying out
person tasks, stopping runs, moving tasks, marking what they have seen,
and running the project. `introduce` is a task's alone: only a task
holds references (../tasks/messages.md, 5).

**A call names what it saw.** An amendment, a release and a decision
about an escalation name the revision the run was shown, as a party's
do: its brief shows its delegates' revisions (`briefs.md`, section 1), the messages
relayed to it show a hold's (../hosts/README.md, section 7), and `delegate`'s,
`amend`'s and `release`'s answers show the spec's revisions they leave.
One whose object has moved since is answered moved, with what it is
now, and the run decides again on that; a change of priority alone
names none, and the first commit decides (../tasks/changes.md). So no
call writes over a change its run was not shown.

## 4. Answers, waiting and parking

- **A run answers once:** finished, with its result; parked; or failed,
  with a typed failure. The core commits it, once it has every turn the
  answer counts, and acknowledges it; a finished task closes, a parked
  one follows the wake rules (../tasks/lifecycle.md, 2), a failed one retries or is
  held (../tasks/lifecycle.md, 6). What the
  answer says the run left on resources goes to their connectors
  (../connectors.md, section 10).
- **Finished done while kept, it parks.** A run whose task is kept
  (../tasks/lifecycle.md, 8) and finishes done, with a result its contract takes,
  is committed as parked: its turns first, then its saved state, kept,
  with no try spent, and a line in its task's activity naming the
  dependencies it waits for. Woken once they are done, its next run goes
  on from what it left, its brief or its start saying what it waited
  for (`briefs.md`, section 1), and finishes again. Kept by its own question, a run
  that finishes with a failure parks the same way (../tasks/lifecycle.md, 8). The tasks child decides it, as it
  decides every reason a task may not go on, so the run and its host
  know nothing of it.
- **Finished while it stands, it parks.** A run of a standing task
  that finishes, with any result, is committed as parked, its result
  discarded and its words a line in its activity
  (`../tasks/standing.md`, section 2).
- **Finished while its delegates are live, it parks.** A run that
  finishes, its finish not asking to cancel them, while its task has live
  delegates (../tasks/lifecycle.md, 7), is committed as parked, as a kept one is: its
  turns first, then its saved state, with no try spent and its finish not
  taken. Its run has ended by the time its finish is judged, on a program
  backend and a direct one alike, so a refusal naming the delegates would
  reach no one, and failing the attempt would cost a whole run and tell
  the next nothing. The task waits for its delegates, whose results wake
  it (../tasks/messages.md, 3), and a later run finishes once they are done. The
  finish put off leaves no trace for the next run: telling it would need
  a value stored for that run, and the delegate's result that wakes it,
  with its delegates' state in its brief (`briefs.md`, section 1), says what it needs
  (the plan `wake-faults`'s answer of 2026-11-08; ../tasks/README.md, section 8). The tasks child decides
  it.
- **Waiting holds the slot; parking frees it.** A run that calls `wait`
  keeps its host's slot while it waits, so a run that is talked with
  answers at once. Idle
  past its charter's threshold, it parks: every turn it took is committed
  first, and it ends. A coordinator's threshold is zero: it parks as soon
  as it waits.
- **Woken again,** a parked task is due. Its charter says whether the
  next run resumes the transcript or starts fresh from a brief, and a
  standing task's resumes only within its session (`../tasks/standing.md`,
  section 5); by
  default a charter for conversations resumes and a coordinator's starts
  fresh, its brief
  carrying its goal, its plan's state and its notes, and its start the
  messages that woke it (1).
- **Messages to a live run,** those it is offered (../tasks/messages.md, 6), are
  relayed as the run reads, named, in order, never more relayed and
  unread than the run's inbox takes; a turn that reads one takes it (2;
  ../tasks/messages.md, 2).
- **Not offered, a message stays** in the task's inbox, never relayed to
  the run, and is judged again when the run ends, waking the task if its
  kind does (../tasks/messages.md, 2 and 6).
- **An entry left undecided** when a run ends, a proposal or an
  escalation still waiting for the task's decision, wakes the task again,
  its reasons that entry's kind, unless the run was offered its kind: then
  it waits for a change, and is offered at the next wake (../tasks/messages.md, 6).
- **The first activation ends** when a run parks idle or finishes: the
  task's next runs hold no `first` among their reasons and take its
  `later` (../tasks/messages.md, 6). A run parked kept, and a failed attempt, leave it
  as it was.
- **Refused or unread, a message stays** in the task's inbox, untaken,
  and wakes the task again when the run ends (../hosts/README.md, section 7;
  ../tasks/messages.md, 2 and 3). A turn's commit never takes a message the run
  refused, whatever its fence covers (2).
- **A cancel** ends the run: the host cancels it and answers; what it
  committed before stays committed.
- **Nothing left to await.** After normal wakes, tasks asks the core to
  number the always-waking `NothingAwaited` notice for a result-ended
  task that parks idle with nothing pending (../tasks/lifecycle.md, 2). Tasks spends
  a run try on each such park and ends failed when those tries run out.
- **Saved state.** A run that parks, fails or is cancelled with unfinished
  work has it saved by its host, where its connectors say, and the next
  run of its task starts from it (../connectors.md, section 10).

## 5. Jobs

A job task (../model.md, 3.2; ../hosts/hub.md, 9; ../tasks/lifecycle.md, 3.1) becomes a
hosted run with no agent:

- **Its check** needs no LLM account, no charter, no model and no
  program's login: its kind among those the application declares, its
  items within the workspace's bounds, its spec and deadline within the
  tasks child's bounds, and its requester's authority over what its
  items write.
- **Its kinds are the application's.** It declares each to the core
  as it makes it (`jig-domain-core`'s `Config`), with the most bytes a
  spec and a result of it may hold and its longest deadline. A batch
  naming a job of a kind not declared, or a spec or deadline past its
  kind's, is refused; the kind's result bound is the job's `Job`
  contract. A job's executor is named by its kind in a delegation's
  kinds (../authority.md, section 3), and no party creates one
  (`../../client/domain/contract/documents.md`, 7).
- **Its claim** goes to a host whose hello declared its kind, whose
  bounds for that kind (its spec's and result's bytes, its deadline)
  cover the job's, and that has a free slot, as a run's does (`fleet.md`
  ), fenced by its attempt. Nothing of a run's placement applies: no
  charter's hosts, no model, no account's source, no login.
- **Its answer** commits its result as a delegate's result (../tasks/lifecycle.md,
  7), which its procedure reads at its next step; a failure is tried
  again within its tries, then holds it, its procedure hearing why.
- **Nothing of it is a transcript:** its spec and result are its
  record; a restart finds a claimed job's attempt as a run's (`restart.md`
  ) and settles it lost or answered.

The application's declaration of job kinds is ascending by number, each
once, as the worker's hello is. A batch naming an undeclared job kind is
refused as `Executor`; a job's spec or deadline beyond its declared bound is
refused as `Spec` (2026-11-04).

A job's transient preparation or failed delivery uses the transient retry
class; permanent preparation, stale or refused delivery, and an invalid
assignment use the permanent class. Its own failure and deadline use the
run's-own class; failure to start uses the agent's class; an oversized
result uses invalid; lost contact and shutdown use lost. Engine cancellation
ends it as cancelled. Busy before starting is a refusal, spending no try. A
finished job becomes its task's Job result, without asking to cancel
delegates (2026-11-04). A failed job's nonempty detail accompanies its
class as a bounded `FailedWithReason` (../tasks/lifecycle.md, 6); an invalid start's
reason does too. The engine retains it through retries into the hold.
