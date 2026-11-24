# Messages

## 1. What a task hears

| Message | From | Woken by default |
|---|---|---|
| a result: done, failed or cancelled | a delegate | the last of its delegates; any failure |
| a question, expecting an answer | a delegate | yes |
| an answer | its requester | yes |
| a decision on its own proposal | the holder that decided | yes |
| an amendment; a cancel's grace, asking it to finish (`changes.md`) | its requester, a task above it, or a party | yes |
| a cancel | the same; the deployment, closing a conversation (../people/acts.md, 3) | always |
| words | a party; a task holding a reference to it | a party's, yes; a task's, by policy |
| news: a connector's event, classified | its subscriptions (4) | by policy |
| a notice: a task it references done or held; a proposal passing through; a hold it waits for | its subscriptions; the core | by policy |
| a notice: nothing left to await (`lifecycle.md`, 2) | the core | yes |
| a notice: its last pending dependency done or removed (`lifecycle-endings.md`, 1) | the core | yes |
| a timer | itself | yes |

Proposals and escalations waiting for its decision are not messages: they
are entries of its own (`proposals.md`), which wake it as they arrive, and
again when a run leaves one undecided only as 6 says.

## 2. The inbox

- **Durable.** A message is in the inbox once the commit that sent it
  is; it is taken once the commit of the run's turn, the procedure's
  step or the party's view that read it is. A run that dies without
  committing what it read leaves it for the next.
- **Bounded, with every message admitted at one entrance.** An inbox
  holds at most a configured number of messages and bytes, and every
  kind of message is admitted in one of three ways:
  - *room kept:* one result from each live delegate, one answer to each
    open question it asked, one notice with up to 256 bytes of answer
    words for each question it received for its person, one notice of
    up to 256 bytes for each question a procedure task asked it, until
    that procedure's step takes its answer (below), one decision
    on each of its pending
    proposals, and a cancel, its grace's ask to finish taking the same
    room, one ask in a task's life (`changes.md`), so none of them can be
    refused;
  - *merged:* news of one subscription with the news of it already
    waiting, since news is a hint to read afresh; notices likewise; a new
    amendment with one still waiting; timers;
  - *refused at the sender's entrance:* words and questions, past the
    bound: a party is told the inbox is full, an agent's tool call
    answers so.
- **Questions outlive their reads.** The asked task keeps each question
  it heard and its first answer in its question positions
  (`../../protocol/store/README.md`, 10.7), untouched by a run's read. Positions
  keep unanswered questions before answered ones; answered positions
  are evicted first, and an unanswered position is never evicted. A
  question past `inbox_messages` unanswered ones is refused at its
  sender's entrance, as words past the inbox's bound are.
- **A procedure's question keeps room for its notice.** A question a
  procedure task asks (`../connectors-procedures.md`, 1.3) is admitted
  only where the asked task's inbox has room for it and for one notice
  of up to 256 bytes: two slots, and its words' bytes and 256, as a
  question received for a person is. The notice's room stays kept from
  the question's admission until the procedure's step that takes its
  answer commits, or the question ends unanswered. That step may spend
  it on one words message to the answer's sender, the question's
  notice, admitted in the kept room and so never refused for the
  sender's inbox; a step that takes the answer and sends no notice
  gives the room back. Why: a procedure relaying the answer on may hold
  it, and its sender must hear so even while its inbox is full or the
  step's own message room is spent (temper's relays,
  `temper/docs/design/domain/forge/change/questions.md`, 1). Where the
  store keeps the room as a fact of its own, that fact is an added
  member with its default, none kept, so a question an earlier build
  admitted keeps no room and its notice is admitted as words are.
- **Open questions stay in front of it.** A question asked of the task
  is open until it hears an answer or the question ends
  (`lifecycle-endings.md`, 1). Read or not, an open question is shown to every
  run that starts from a brief: in its waiting section, with its asker,
  its number and how to answer, unless the run's start carries it as a
  message (../engine/briefs.md, 1.2). It is among
  every run's reasons (6), and, left open by a run that parks idle,
  wakes the task again (`lifecycle.md`, 2).
- **Words that direct it stay until a run records them.** A task
  **keeps words** when it is an agent task whose runs start from a
  brief, never resuming (../engine/executors-transcripts.md, 1), that is no
  conversation, does not stand (`standing.md`) and is not ended by its
  requester (`lifecycle-endings.md`, 2): a task that resumes holds what it read
  in its window, and a conversation or a standing task is read by its
  person turn by turn. In a task that keeps words, a words message (1)
  from a person, or from its requester where that is a task, is
  **kept** when a turn's fence takes it: the fence takes it from the
  inbox as it takes every message, and the task keeps it, whole, among
  its **unrecorded words**, oldest first, until a run of the task
  records it. Words from any other task, a delegate's or one it was
  introduced to, are news, taken as every message is.
  - *Recorded by the run's act.* A run records a message where the
    next run will find it, in a spec it writes or in a note, and says
    so: by naming it in `mark_read`, the act that says it was read, or
    in the `records` of a `note`, which records it once the note is
    written (../engine/executors-tools.md, 1). What records it is that act,
    never the core's reading of a spec or a note. Only a run of the
    task records its words; a number naming none of them records
    nothing and is answered so.
  - *Within the inbox's bound.* Unrecorded words count in the inbox's
    bound, its messages and its bytes, as words waiting in it do, so
    that words or a question past it are refused at the sender's
    entrance as the inbox full; the room kept (above) is unchanged.
  - *In front of every run.* Every run that starts from a brief shows
    them, whole, in its waiting section (../engine/briefs.md, 1.3);
    `words` is among its reasons while one is kept (6); one left
    unrecorded by a run that parks idle wakes the task once
    (`lifecycle.md`, 2); and the task's document lists them
    (`../../client/domain/contract/documents.md`, section 1). They end
    with the task.
  - *Why:* a run that read a person's words adding work, then parked or
    was stopped without carrying them into a spec or a note, left them
    nowhere a later run reads (dev.fabrig.ai, 2026-11-18: two of four
    asks dropped by one goal; two messages lost by another whose hung
    run was stopped).
- **Within what a run reads.** Every message an inbox admits fits a
  run's message bound as it is rendered (smith's `protocol/limits-derivation.md`,
  1.9), so no run refuses one as too large: words and questions larger
  are refused at their sender's entrance as too large, never cut (a
  party's by the people child, whose bound is at most the run's;
  ../people/acts.md, 1), and every other kind fits by its own bound.
  Configuration checks both against the deployment's message bound
  (`../../protocol/limits.md`), and the fleet admits no
  host whose agent takes less (../engine/fleet.md).
- **Carried whole and read oldest first,** within the run's inbox. A
  run's start carries the messages waiting in its task's inbox that the
  run is offered (6), each whole, oldest first, as many as the run's
  inbox takes (smith's `protocol/limits.md`, 2.3); a person's words are
  never cut. The rest wait in the inbox, in order, and those it is offered
  are relayed as the run reads, as what its policy lets through while it
  runs is, never more relayed and unread than the run's inbox takes
  (../engine/executors.md, 1 and 4).
- **Entries are read as messages are.** A proposal or an escalation
  waiting for a task shows in its inbox as an entry (`proposals.md`), relayed
  and read in order with its messages, and taking none of their room.
  An escalation's exact offered number is recorded as read only with the
  turn that reaches it, excluding refusals. The bounded set lasts for that
  attempt; a claim or restart offers unresolved entries afresh.
  One decided while offered to a live run keeps its place in that run's
  inbox until a turn's fence reaches it, so the core never relays more
  unread than the run's inbox takes (../engine/executors.md, 4).
- **Refused or unread, a message stays.** A message the run refused (too
  large, its inbox full, a name in use, or the run ending), or left
  unread (after the answer's fence, the last message it read), stays in
  the inbox, untaken, and wakes the task again when the run ends, as it
  would had it arrived then, after a pause where the run parked (3).
- **Not offered, a message stays too.** A message the run isn't offered
  (6) is neither carried at its start nor relayed to it; it keeps its
  place in the inbox, untaken, and is judged again when the run ends, as a
  message refused or left unread is (3). A turn takes only the messages
  offered to its attempt up to its fence, so a fence that passes one kept
  back takes nothing of it, nor of one a pace holds (3).

## 3. Wake policies

A wake policy is data, from a closed vocabulary:

- **for each kind of message** (1): whether it wakes the task, or waits
  for the next activation;
- **for news,** the class the connector gives each event for the task:
  *wakes*, *kept* (news, but nothing the task must act on now), or
  *dropped* (not the task's concern after all, and not kept). The task's
  policy may lower a class, never raise it;
- **markers:** names a connector offers on its
  news (a forge's commit trailers), numbered by the application across
  its connectors, at most 32, so that a policy holds them as one set
  of bits (`markers: u32`, the policy staying a copied value); the
  core passes the set to each connector with each subscription of the
  task's, a goal's among them, and again on an amendment of the
  policy, and the connector classes an event carrying one of its own
  as waking for the task; the class is still the connector's to give,
  and a connector ignores the bits that are not its own;
- **batching,** for kinds that allow it: wake once N have arrived, or
  the oldest is older than T, so a watch hearing a storm of alerts wakes
  once. A person's words, a cancel and an amendment are never batched,
  and never wait behind what a batch holds;
- **a pace,** for the kinds batching allows: wake at most once in a
  span. Such a message wakes the task at once when its last run
  started at least the span before, or it has never run; otherwise it
  waits until the span since that start has passed, and what arrived
  meanwhile wakes it once. A run started for any other message carries
  what the pace holds, as every run carries its inbox (2); a message
  the pace holds is not relayed to a live run, nor taken by a turn's
  fence that passes it, and is judged again when the run ends. A pace is
  not a delay: the first message after a quiet span wakes at once;
- **timers:** wake at a time, or every period.

A proposal or an escalation waiting for its decision always wakes a
task: no policy may keep a decision from whoever must make it. One a run
was offered under a part that covered it and left undecided wakes the
task again only when it changes (6).

**An escalation waiting for a task makes every message wake it.** While
an escalation waits for a task's decision, from its arrival until it is
decided, ends with its hold or waits elsewhere, every message the task's
inbox admits wakes the task at once, whatever its policy says of the
message's kind: a result that would ride along under `last_or_failure`
or `never`, news a pace or a batch holds or its connector keeps, and a
question, an answer, a notice or a timer the policy doesn't wake on.
Each is the change the escalation waits for (6). It is judged so
wherever the policy is read: its kind is among the reasons of the run it
wakes, a live run is relayed each result as one its policy lets wake it,
and a message a run leaves unread or isn't offered is judged so again
when the run ends, the park so left due waiting a pause first (below). Once no
escalation waits for the task, the policy judges as before, and a message
a batch or a pace holds wakes by its timer. A proposal waiting for its
decision changes no policy (`proposals.md`, Open questions). So a task
that parks leaving an escalation undecided, as a goal parks on its
repair, runs again on the next thing it hears, the repair's result among
them (decided 2026-11-24).

A message a run refused, left unread or wasn't offered (2 and 6) is
judged again when the run ends, as if it arrived then: it wakes the task
if its kind does, and otherwise waits for the next activation.

**A park that wakes at once waits a pause first.** A run that parks and
leaves its task due again at once, so judged, on a message, a proposal or
an escalation it left in place, claims again only after the pause a
refusal takes (`lifecycle-holds.md`, 1): no try spent, the pause growing with
each such park in a row, and from the first again once a park leaves its
task idle. The run after the pause is woken by the same message, as it
would have been at once. What arrives meanwhile waits for it, as for any
pause. So a run that parks without reading what wakes it never has one
start assigned again without backoff (the user, 2026-11-19).

Each charter brings a default policy, named with the charter in
configuration (`../../protocol/configuration-engine.md`, section 1) and kept with
it in the settings: an agent task made under the charter, by a batch
entry that names no policy or from a template, which names none
(`../../protocol/configuration-templates.md`, 1), takes the charter's default as
the settings hold it when the task is made, and keeps it as its own;
its creator narrows or widens it by naming a whole policy in the entry,
and an amendment changes it as any policy. A chat's default wakes on
its person's words and on its delegates' results; a coordinator's on
questions, escalations, failures and its last delegate's result
(`last_or_failure`; for a task its closer ends, a failure alone: `lifecycle-endings.md`, 3),
words from its delegates and its person's words, at once, and on news
its connectors class as waking at a pace, at most once in a span (the
reference's coordinator: 30 minutes); most other agents' on
nothing, since they finish within one run. A procedure's policy is its
owner's: it steps on whatever it reads.

A pace's span is nonzero. An invalid wake policy in a batch or an amendment
is refused as `Spec`; an invalid amendment changes nothing of the task
(decided 2026-11-02).

## 4. Subscriptions

A subscription is a task's standing interest in what changes:

- **a connector's topic,** such as a service's alerts or the landings on
  a branch: kept by the connector, which names the task by its number and
  classifies each event for it (../connectors-facts.md, 1.3);
- **another task's state:** done, held, or its result; for a task it
  references;
- **a timer.**

Subscriptions end with the task. A goal's subscriptions to the topics its
connectors offer for goals are the core's to keep, not its executor's
(../model.md, 6.2). Their number per task is bounded, by `subscriptions`
(`README.md`, section 6), but for one topic asked apart:

- **One topic apart.** An application may ask a connector's topic for a
  task apart from the bound (the tasks child's `SubscribeTopicApart`,
  beside `SubscribeTopic`): admitted while the task holds at most
  `subscriptions` subscriptions, so one past the bound, and refused
  otherwise only as any topic is (its task closing or ended, its number
  taken, no room in its inbox for its news, 2). Every other
  subscription stops at the bound, counting it. Nothing marks it once
  made: it is a topic subscription as any other, and the task holds at
  most one more than the bound, however many it is asked apart. So
  news the application owes a task, whatever the task subscribed to
  itself, such as a work item's to the one that holds it, isn't crowded
  out.
- **Kept to the bound and one more:** a task's record, its restore and
  the store's row hold `subscriptions` and one more; a client's view of
  them keeps its own bound (`../../client/domain/pages.md`).

## 5. Who may message whom

- **Along delegation:** a requester and its delegates, both ways.
- **By reference:** a task may message any task it holds a reference to:
  its requester, its delegates, and one it was introduced to. A task
  introduces two tasks it references by giving each a reference to the
  other, within their limits. References are kept with the task, and end
  with it.
- **Parties** message the tasks their role lets them see (../people/acts.md
  ).
- **No broadcast.** Who may talk to whom stays visible and bounded.
- **A conversation's questions:** a question a delegate asks its
  requester that is a conversation is the conversation's alone: it
  wakes it, and is no entry of its person's. The conversation answers
  it itself, in the answer form, or asks its own person in its own
  words: by a person task addressed to its requester, asking in words
  (../people/README.md, section 7), which its charter's authority allows, an
  entry of that party's inbox, whose answer reaches the conversation
  as that delegate's result.
- **A gathering's questions:** a question a delegate asks its
  requester that is a gathering goes to the gathering's requester,
  where that is a person: an entry of their inbox, whose answer reaches
  the delegate as its requester's answer (`gatherings.md`, section 4).

## 6. Why a run runs, and what it is offered

An agent task's charter may hold parts, each with instructions for some of
the reasons a run runs (`README.md`, section 3), so that a run reads the charter's own
instructions and the parts for why it runs, and is offered the messages
those parts are for.

- **A run's reasons** are `first`, its task's first activation, and the
  kinds of message that woke it, each named as jig's store names it
  (`../../protocol/store/records.md`). `first` lasts until a run of the task
  parks idle or finishes: a run parked kept, by its own question, a question
  it owes or a dependency (`lifecycle-endings.md`, 1), leaves the run that continues it `first`
  too, and so does a failed attempt for its retry (`lifecycle-holds.md`, 1). A first run's
  reasons are `first` plus the kinds that woke it, such as a person's words
  waiting at its start.
- **The kinds that woke it:** each kind of a message waiting in its inbox,
  or of an entry waiting for its decision (`proposals.md`), that its wake policy
  lets wake it when it is activated (3). A kind that rides along without
  waking it is no reason: a non-last result under `last_or_failure`, or
  news a pace holds, gives none, unless an escalation waits for the task,
  when every message wakes it (3).
- **An open question is a reason.** While a question asked of the task
  is open (2), `question` is among the reasons of each of its runs,
  whatever woke it and whatever its policy, as an entry waiting for its
  decision gives its kind. So under a charter in parts, a run woken by
  words, a result or news reads the part for questions too, and is
  offered what that part names.
- **Unrecorded words are a reason.** While the task keeps a words
  message unrecorded (2), `words` is among the reasons of each of its
  runs, whatever woke it and whatever its policy, so under a charter in
  parts a run reads the part for words and is offered what it names.
- **A continuation and a retry** run for the reasons of the attempt before
  them, kept with the task (`README.md`, section 3), plus the kinds that wake them at
  their own activation. A retry is due by its backoff, not by a message, and
  its start carries what is waiting then, not what the failed attempt read.
- **The kinds that choose parts:** `question`, `question_answered`,
  `question_ended`, `question_owed`, `escalation`, `proposal`,
  `person_stalled`, `result`, `notice`, `nothing_awaited`, `news`, `timer`
  and `words`. The others choose none and are **always offered**, each a
  reply to the task's own act or a change to what its run does: `answer`,
  `proposal_decision`, `settings_decision`, `effect_not_made`, `met`,
  `amendment` and `finish_requested`. A person's `words`, which no batch
  holds (3), and the entries waiting for the task's decision, which are
  relayed to a live run whatever its parts (`proposals.md`), are always offered
  too, though their kinds choose parts.
- **A part** is for the reasons in its `on`, a nonempty set of `first` and
  the kinds that choose parts, and has instructions of its own. A run reads
  its charter's instructions, then each part whose `on` meets its reasons,
  in the charter's order, each part once (../engine/executors.md, 1); a run no part is
  for reads the charter's instructions alone. A charter holds at most
  `tasks.charter_parts` parts (`../../protocol/limits.md`).
  One whose runs resume may hold parts, which a standing task reads at
  each session's start and keeps through the session (`standing.md`,
  5.4); any other task under it reads its charter's instructions alone,
  every run, and is offered every message: a resumed run keeps the
  instructions its conversation began with, which changing would lose its
  cache and leave its earlier turns read under other words.
- **What a run is offered:** under a charter with parts, the always-offered
  and every kind named in the `on` of a part it reads, for a standing task
  the parts its session's reasons chose (`standing.md`, 5.4); under a
  charter with none, for a task that does not stand under a charter whose
  runs resume, or for a task with instructions of its own (`changes.md`), every
  message, as a charter with no parts gave every run before. The parts are
  those its preparation chose (../engine/executors.md, 1, step 2).
- **A live run hears its results.** Beside what its parts offer, a
  claimed or running run is relayed each delegate's result its task's wake
  policy lets wake it (3), judged as it arrives and again as the run
  takes what waits: under `each`, every result; under `last_or_failure`, a
  failure or a cancel, and the result that leaves the task no live
  delegate, for a task its closer ends too, since a live run is no idle
  task and makes no closer (`lifecycle-endings.md`, 3); and under any
  policy, every result while an escalation waits for the task (3). Such a
  result chooses no part of the live run, which keeps the instructions it
  started with; one the run leaves unread is judged again when the run
  ends (3), where, for a task its closer ends, a last success wakes
  nothing (`lifecycle-endings.md`, 3) unless an escalation waits for it.
- **Every reason waiting at its claim.** A message or an entry that
  arrives between a run's preparation and its claim, and whose kind would
  join its reasons (one its wake policy lets wake it, of a kind that
  chooses parts, that its prepared reasons don't hold), prepares the run
  again, so that the run starts with every reason waiting for it, a due
  result among them, and every part those reasons choose. One whose kind
  would add no reason is offered or kept by the parts prepared. Each such
  preparation adds a kind to the reasons, so a run prepares again at most
  once per kind.
- **What isn't offered keeps its place** (2): it is neither carried at the
  run's start nor relayed to it, no fence takes it, and when the run ends it
  is judged again as if it arrived then (3), when it may wake the next run,
  which then runs for its kind.
- **An entry left undecided waits for a change, once a run covered it.**
  When a run ends with a proposal or an escalation still waiting for its
  task's decision, what the run was offered decides:
  - *covered:* the run was offered the entry's kind (a part it read names
    the kind in its `on`; or its charter has no parts, or its task has
    instructions of its own, so it read its instructions for every kind).
    The run had the entry and left it, so the entry doesn't wake the task
    again by itself. It is offered to the task's next run, whatever wakes
    it, its kind among that run's reasons (3), and it wakes the task
    when it changes: a new hold of its held task or a new holder makes a
    new entry, which wakes as an entry's arrival does (`proposals.md`).
    For an escalation, every message the task's inbox admits while it
    waits is such a change and wakes the task, whatever its policy says
    of the kind (3): a delegate's result that would ride along, its
    person delegate's among them, news, or a notice of its held task.
    For a proposal, a message about it, such as a notice, wakes by its
    own kind.
  - *not covered:* no part the run read names the kind, as a cut whose
    part is for `first` alone names no escalation. The task is woken
    again for the entry's kind, as an unread message would wake it, and
    its next run reads the part for it.
  - An entry the run refused (2) wakes the task again, covered or not.

  The held task and its entry are unchanged either way, and the entry's
  stall and routing stay in force (`proposals.md`). The task keeps the kinds
  its last run was offered (`../../protocol/store/records.md`), so a
  restart wakes an idle holder only for an entry its last run wasn't
  offered.
- **The set,** as the store keeps it, is bits: bit 0 for `first`, then
  bits 1 to 13 for the kinds that choose parts, in the order above; no
  other bit is set (`../../protocol/store/records.md`).
