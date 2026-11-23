# The engine's executors: tools

## 1. The engine's tools

A run reaches the engine through smith's host tools, declared in its
charter, which its host relays; their schemas, decoding and rendering
are the protocol layer's (../hosts/charters.md). Each is checked against
its task's authority (../authority-checks.md, section 1), decided, committed, and
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
| `read_task` | reads its own task, its requester, live direct delegates, explicit references or escalation subjects in its project; retained ended direct delegates remain readable; any task below it, at any depth, through a proven requester chain of live or retained intermediate tasks; and the dependencies of its immediate requester, live or retained, read only | bounded specification words by default, or kept terminal report words with `field=result`, with the selected verdict code alongside verdict reasons; a live task's `field=result` read at offset 0 answers its specification read, saying its result comes when it ends; lifecycle, dependencies, spec revision, hold revision and reason, including connector hold words; durable unanswered question credits with their numbers, authorized answerers, addressed parties and activity positions, bounded by `tasks.inbox_messages` and available after the question delivery is consumed; `from` continues the selected field at the printed `next` byte offset |
| `message` | words, a question or an answer to a task it references | sent; no reference; the inbox full; too large for a run to read |
| `mark_read`, in the `message` family | records words its own task keeps unrecorded (../tasks/messages.md, 2), named by their message numbers, at least one and at most `tasks.inbox_messages`, once the run has carried them into a spec or a note: each named is no longer kept, shown or listed | recorded, naming them; apart, the numbers that named none of its unrecorded words, already recorded or never kept, which record nothing and are no refusal |
| `amend`, `release` | a task below it: an amendment naming its spec's revision as the run last read it, but for a change of priority alone, and adding or removing dependencies within its reach (../tasks/changes.md); a release naming its hold's, and, for an effect held uncertain, whether to make it again (../tasks/lifecycle-holds.md, 1 and ../tasks/changes.md) | done, with the task's spec revision after the commit, the one its next amendment names (a release moves none); refused, and why; moved, with what it is now |
| `cancel` | a task below it and its tree, with a reason, and a grace if the run gives one, within the tasks child's bound: the task is asked to finish first (../tasks/changes.md) | done; refused, and why |
| `decide` | accepts, rejects or passes up a proposal or an escalation waiting for it, an escalation naming its hold's revision (../tasks/proposals.md) | done; beyond its authority, and passed up; moved, with what it is now |
| `propose` | an action beyond its authority, with a reason: a batch, an amendment, an effect, or a widening of its own authority (../authority.md, section 9) | proposed, and to whom; refused, when nobody may accept it (../tasks/proposals.md) |
| `withdraw` | its own pending proposal (../tasks/proposals.md) | withdrawn; already decided, and how |
| `move`, `succeed` | moves a task below it, or that it was introduced to and may amend, under another live task it may amend; names a live task the successor of an ended task below it (../tasks/changes.md) | moved; named; refused, and why |
| `hold`, `lift` | places a hold of its own, stop or drain, over the subtree of a task below it, with a reason, and optionally the event that lifts it: a connector's event its topic offers, a task's end, or a time (../tasks/lifecycle-holds.md, 2); lifts a hold it may lift, naming its revision | the hold's number and revision; lifted; refused, naming its owner, and whether a proposal could be accepted |
| `settings`, when its charter gives the family | reads the settings, or one setting whole, as a party's read does (`../../client/domain/contract/documents.md`, 1), or one proposal of settings, a task's of its own project or a party's, as a party's proposal read gives it (below); proposes a change of settings against the revision it read, with a reason (`settings-proposals.md`, 1) | the settings; the proposal, with its review and its outcome; proposed, and to whom; stale, naming the newest; refused, when nobody may accept it |
| `introduce` | two tasks it references, each given a reference to the other, within their limits (../tasks/messages.md, 5) | done; no reference; refused, and why |
| `subscribe`, `unsubscribe` | a topic a connector offers to agents (../connectors-facts.md, 1.3), under a read grant on the resources the connector says it concerns (../authority-checks.md, 1.5); a referenced task's state; a timer (../tasks/messages.md, 4) | done; refused, and why |
| `effect` | one of a connector's kinds of effect offered to agents (../connectors-effects.md, 1.5) | made, with what it made; committed and pending, past the call's deadline; waiting for a verdict; failed; refused, naming them, for a kind that waits for dependencies while its task is kept (../tasks/lifecycle-endings.md, 1) |
| `note`, `recall` | writes a note in a scope, whole or by a line range, and, with `records`, records the unrecorded words it names as `mark_read` does, once the note is written, none when it is refused; reads entries by name or by a search (`briefs.md`, section 2) | written, and what `records` recorded and named in vain, as `mark_read` answers; the entries |
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
answers (`../../client/domain/contract/reads.md`, section 1): a proposal waiting,
from the core's state, with its reason, its items and its review against
the newest settings (`settings-proposals.md`, 1), a value past the review's bound by its size; a
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

- **Below it.** A task below the caller, at any depth, is read as a delegate
  is, its specification and its result alike, with its state: each requester
  link must be proved by a live or retained record in the caller's project,
  with consecutive depths, up to the caller within the task depth bound.
  Retained intermediate tasks remain proof after ending; a shared root or an
  unproved ancestor number alone grants nothing. So a goal reads the coding
  agent under one of its changes, or a task under its closer, by the number
  its messages name.
- **Loaded as cheaply as it can be.** The store's task range supplies the
  linked records without restoring ended tasks. A result read of a stored
  task loads its row joined (`Scope::Task`), with its requester chain. A
  specification read of one loads its row by the text load (`Scope::TaskText`),
  as a client's does, and only where that row is not related otherwise loads
  it again joined, whose chain proves it. The joined load keeps its join and
  byte bounds: a read past them answers busy, as it does today.
- **Its requester's dependencies.** A task listed among the dependencies of
  the caller's immediate requester (../tasks/README.md, section 3), of the
  caller's project, live or retained, is read too, both fields. So a change's
  coding agent reads the changes it waits on, and so does every other task
  that change delegates to, a repair or a resolution. A requester stays live
  while its delegates are (../tasks/lifecycle.md, section 1), so the core's
  record names its dependencies. Nothing else follows: the caller's own
  dependencies are no requester's, a task further up lends none, and a sibling
  not among them, another goal's task or another project's stays refused. The
  rule names no executor, charter or procedure.
- **Read only.** Loading grants no message, amendment, hold, control or
  subscription authority: a dependency stays no reference
  (../tasks/messages.md, 5).

Authorization and the
current attempt are checked again after a stored read. `field` accepts only
`specification` (the default) or `result`. Report, NoChange, Change and Verdict
results serve their kept words, including reports kept with cancellation. A
Verdict result read also carries its selected code as optional metadata, on
every chunk, separately from the bounded reason words. Specification reads
and other results carry no verdict code. Offsets and totals count only the
reason bytes. A live task's result read at offset 0 answers as its
specification read at offset 0 does, the same header, state, hold and
questions and the specification's first chunk, marked as standing for a
result that comes when the task ends; at any other offset it is unknown. An
ended task's unavailable terminal report is unknown. Both fields use bounded UTF-8 chunks
and reject offsets inside a character or past the source. A run's chunk is
bounded by the room its application names with the read, held to
`core.call_answer_bytes`, and not by `documents.text_bytes`, which is what a
client decodes (`../../client/domain/contract/reads.md`, section 1): the application
knows what its run's reply holds. An application that cuts a chunk further to
render it prints `next` from its cut. Continue with the same
field and the printed `next` offset. Every chunk names the current spec revision; restart specification
reads at zero if it changes between chunks.

**A run's read as its person** is how an application's asking tool reads
for a run that explains to its person: one of a party's reads
(`../../client/domain/contract/reads.md`, section 1), read as the run's task's
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
  party (`decisions.md`, 1.3, `views.md`), the requester its party: their visibility,
  their pages and bounds, their loads. In place of a sign-in, a stored
  read's completion checks again the run's current attempt, its
  requester, the person's role in the project, and that what the row
  names is of the project, and answers unknown where any but the attempt
  has moved; an attempt no longer current is answered nothing, as the
  run's other stored reads are (above). An application's read goes to
  its connector as a party's does, with the person and the project's
  scope, and the connector answers it to the run's call through the root
  (../connectors-facts.md, 1.5).
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
effect's key is derived from the call that asked for it (../connectors-effects.md,
1.4).

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
make (../people/acts.md, 1), checked by the same rules (../authority-checks.md, 1.1 and
1.4), so a run and a party do the same things to the same tasks:
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
