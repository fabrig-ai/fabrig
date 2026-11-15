# A change's questions and early starts

## 1. Its questions and early starts

### Its questions

A change made with questions `up` (`README.md`, 1) relays its agents'
questions: its producing, its repairs and its resolutions ask it, it
passes each to its requester, and it answers the asker. A goal's
closer, the requester of each change it makes and a procedure, passes
each on to its goal and relays the answer back down
(`../branches.md`, 2). It decides
nothing itself (jig's `domain/tasks/proposals.md`
): the requester decides, and the change relays
(`../../../draft/notes.md`, N1). A change made with questions `none`
relays none. The routes are `none` and `up` alone: `foreman`, which had
a delegate of the forge's charter 3 settle each question first, is
retired (`../../../draft/stock-charters.md`, 3), since the implementer
now checks its spec itself before it asks. A `change` naming it, alone
or in a batch, is refused by name (`../../agent/tools.md`, 1), and a
stored setting naming it is refused, not read, naming the setting
(`../../../protocol/configuration/settings.md`, 2).

- **Its agents may ask.** It delegates its producing, its repairs and
  its resolutions with the Message family kept, so that each may ask
  its requester, the change, in the question form, its words carrying
  its options and its recommendation, after pushing what it has; a
  question in the line form (`../../agent/agents.md`, 2) reaches a person's client
  as its choices where it is relayed whole, and any other as words. An
  agent that asked is kept by its own question (jig's
  `domain/tasks/lifecycle.md`, 8): its run parks, no try spent, until the
  change answers, a `finish` before then parking it too, whatever its
  form, and the change's wait for it is no stall (`repairs.md`). Its
  reviews are delegated without the Message family, as under `none`.
- **Taken at its step.** Each step takes the questions its own agents
  asked (its producing task, and the repair or the resolution it has
  live), oldest first, as its open questions, kept on its record, and
  works them one at a time in that order: the next is passed up once
  the one before is answered. A question from any
  other task, and any other task's words, receive fallback feedback naming
  the enabled route and the answer form; words do not settle a question.
  Its requester's words while it has a live producer are passed to that
  producer instead (*Its requester's words and amendments*, below).
- **Its own agents' words.** Its agent's report goes in its `finish`,
  and the question form is its one use of `message` to its change. Its
  words settle nothing, and in a run that may wait, what reaches it
  before its `finish` refuses that `finish` until read (smith's
  `domain/run.md`, section 6), so feedback meant for a requester would
  cost it a turn. The `message` call itself refuses them, nothing sent:
  a call in the words form to the change from its producing task, or
  the repair or the resolution it has live, answers with an error
  saying `Your report goes in finish; this change takes no words. To
  ask: message target=this change, form=question.` Words or an answer
  from such an agent found in the change's inbox at a step receive those
  same words, as words, in place of the fallback feedback.
  A question of theirs past the open-queue ceiling keeps the fallback
  feedback (below). The stock coding charter tells its run the same, and
  to call `wait` after asking, never `finish`
  (`../../../protocol/configuration/stock.md`).
- **Which target.** Its change is its requester: the number its brief's
  `[Requested by task N]` line gives, never its own, `[This task: N]`. The
  stock coding charter says so. A project's own charter may not, so the
  `message` tool's refusal of a target the caller does not reference
  (jig's `domain/tasks/messages.md`, 5) says where to find it, for any run:
  `Authority refused: task reference. Your brief's [Requested by ...] line
  names your requester; [This task: N] is you.` An authority refusal lists
  its findings with `; ` between them and ends without one.
- **Bounded.** It counts the questions it takes, each as its turn comes,
  from its making or its last release, against its bound,
  `forge.change_policy.questions` (4 in `small`, 80 in `standard`, which
  `temper-box init`'s box runs, each below the 128 open questions a change's
  record keeps:
  `../../../protocol/configuration/limits.md`). The one past the bound,
  when its turn comes, is passed nowhere: the
  change holds itself, its words naming the bound and the question's
  asker and number, and the hold escalates to its requester (jig's
  `domain/tasks/proposals.md`). Released, it passes that question up
  (below), the first it counts since the release, as
  its repairs count again from one (`repairs.md`).
- **Passed up,** under `up`, or released past the bound,
  one at a time in the order taken: the change asks its requester in
  the question form, its words the asker's question whole, as its
  record kept them when it was taken, cut from its end to the message
  bound and marked. The change is
  kept by its own question while it waits (jig's `domain/tasks/lifecycle.md`,
  8): it steps as ever, and merges nothing, its merge waiting as for
  a pending dependency (`README.md`, 2, step 7). A repair's or a resolution's
  question passed up takes it out of its queue until answered, as a
  kept change's place is (`../queues.md`). Its requester, a goal among
  them, is woken by the question, which its run reads labelled with how
  to answer (`../../agent/README.md`, 4). A run that ends without
  answering leaves the question shown in every later run's brief, and
  its requester is woken again while it awaits nothing else (jig's
  `domain/engine/briefs.md`, 1.2, and `domain/tasks/lifecycle.md`, 2),
  so a requester with nothing else to wait for is never left idle with
  the change's question.
- **The answer relayed down.** The requester's answer, in the answer
  form to the change's question, is relayed at the step that takes it.
  A fresh requester run finds that durable question number and its authorized
  answerer with `read_task` on the change, even after consuming the original
  delivery. The producer's question number and a question's ordinal are not
  the change's answer number. A refused message directs it to that read.
  Words and amendment reasons do not answer questions. For a change with an
  enabled question route, fallback replies describe that route and the
  required answer form rather than claiming that the change takes none.
  The change answers its asker's question with the answer's words
  whole, and the asker's next run goes on from where it parked with the
  answer to read. The requester may first amend the change (its words
  widened, a Left item accepted: `../README.md`, section 5 of jig's `domain/tasks/README.md`),
  which reaches a live producer (*Its requester's words and
  amendments*, below), or add a change as the change's dependency, one
  it makes now or one it already has, a sibling change of the same
  goal among them, which orders the change's merge after it and holds
  the answer until it lands (below); its answer then says so.
- **Held for a dependency it did not start from.** An answer the
  change takes is held on its record (where it stands: its answer
  held) while some pending dependency of the change is not among those
  its task started from (*From what*, below) and is not a change into
  another repository; it is relayed at the first step at which none
  is, that dependency having ended done or been amended away. A
  dependency the requester adds as it answers is such a one: its
  work is not in the producer's workspace, and holding the answer
  until it lands is what makes the producer's next run start from the
  landing branch's tip, which then holds it (*The next producer merges
  the landing tip first*, below). The rule's reason is that work: an
  early producer's dependencies started from are already merged into
  its workspace, so an answer that finds those alone pending is
  relayed at once, and the producer goes on beside them. A change
  into another repository orders the landings alone (`../../landing.md`,
  4.1) and holds no answer. Whatever holds or relays an answer, the
  dependencies order the merge: nothing merges before they land
  (`README.md`, 2, step 7; jig's `domain/connectors.md`, 4.1).
- **Held for message room.** Apart from that rule, an answer is held
  on its record while the step that takes it has no message room left
  for it (below), and relayed at the next step that has room and no
  dependency holding it. A held answer is relayed by the same rules
  after a restart, and a change whose record holds one that no
  dependency holds has its next step due at once. The record's form is
  unchanged, so an answer an earlier build held for a dependency it
  had started from is relayed by the same rules.
- **An asker that ends first** (cancelled, or its run failing for good:
  its own `finish` parks it) ends its questions: an answer to a
  question passed up for it is taken and dropped, the change's history
  naming it. The
  change's own question stays until its requester answers it or it
  ends with the change. No question goes silently: the change tells its
  requester, in words and once, `Task A ended with its question N open;
  it reached no one: LINE`, where it had not been asked of the requester
  (taken), and `Task A ended; your answer to its
  question N goes no further: LINE`, at the step that takes or holds
  the answer to one that had. LINE is the question's first line, as its
  record keeps it. These words share the step's `tasks.batch`, and a
  question with no room for them stays on the record until a step has
  (below).
- **On its record and its panel.** Its open questions, in the order
  taken, each with its asker and number, its words whole and their
  first line, and where it stands (taken; asked of the
  requester; its answer held), and the questions
  taken against its bound, are kept on its record, so that a restart
  resumes each where it stood (`../README.md`, section 9), and shown on its panel
  (`README.md`, 5; `../../../client/forge/served.md`, 2) and by `ctl`'s `change show`
  (`../../../client/ctl/areas.md`, area 6). The printed numbers are labelled
  producer questions; the requester is directed to the change's task read
  for its answer number.
- **Its history** keeps every question and answer: the asked tasks'
  question positions and the change's own (jig's `domain/tasks/messages.md`,
  2). A ruling that concerns many changes is design: a designer's
  change into the landing branch, and amendments of the live changes it
  concerns.

A change keeps at most 128 open questions. Its configured question allowance
must be less than 128, leaving room for the question past that allowance. A
question beyond the open-queue ceiling is answered with fallback feedback
describing the enabled route and answer form; it never enters the relay (2026-11-04).

Every relay input read at a step is retained before any message is sent. A
question is taken once by its asker and number. Answers awaiting passing
down stay on the record even when the step has no message room.
Passing up after release does not check the spent allowance again. Ask and
answer messages share the step's `tasks.batch` with its other replies. Work left solely for lack of message room makes the
next step due at once (2026-11-04).

### Its requester's words and amendments

A change's requester reaches the change's live producer through the
change: by words, and by amending the change. Neither answers a
question (2026-11-15, `../../../draft/producer-reach.md`, 2 and 3). The
change's producer is its producing task (`README.md`, 2, step 1), live
from its making until it closes: waiting, active (running, parked, or
kept by its own question) or held (jig's `domain/tasks/lifecycle.md`).
Its repairs and resolutions are not its producer: their briefs are made
from what failed, not from the change's words, and words to the change
while one of them is its only live agent receive the fallback feedback.
What follows holds under either question route.

- **Its requester's words pass to its live producer.** Words in the
  words form from its requester, a task, taken at a step while the
  change has a live producer, are passed to that producer in the words
  form, and the change tells its requester so, in words, in the same
  step:
  - to the producer, `From task R, your change's requester:`, a blank
    line, then the words whole, R the requester's number, the whole cut
    from its end to the message bound and marked `…`;
  - to the requester, `Passed to task P, this change's producer.`, P
    the producer's number. Where the record holds an open question of
    P's, taken or asked, the reply goes on: ` Its question still waits,
    and words do not answer it: LINE. Answer it with message
    form=answer; read_task on this change gives the question's
    number.`, LINE the first line, as the record keeps it, of the
    first such question. The reply is cut from its end to the message
    bound and marked `…`.

  With no live producer, the requester's words receive the fallback
  feedback as before: the enabled route's (above), or a change with
  `none`'s (`README.md`, 1). Any other task's words receive the
  fallback feedback, and its own agents' words the words of *Its own
  agents' words* (above), as before. A party's words, a requester's
  among them, stay as `README.md`, 1 has them, taken and dropped: a
  procedure's step messages tasks alone (jig's
  `domain/tasks/messages.md`, 5), so it could not tell a party its
  words were passed.
- **Words never answer.** Passing words settles and clears no
  question: a question asked of the requester stays asked, an answer
  held stays held, and a producer kept by its own open question stays
  kept (jig's `domain/tasks/lifecycle.md`, 8), the words waiting in its
  inbox until that question's answer lets it go on. An answer from the
  requester that the relay does not take, to no question asked of it
  or a second answer to one answered, receives the fallback feedback as
  before and is not passed.
- **Durable as the answer relay.** Words taken are kept on the
  change's record, in the order taken, each with its sender, its number
  in the change's inbox, its words whole and the producer they were
  passed to, if they were, until its requester has been told. A step works them oldest first,
  each message as its message room holds it: it passes the words, then
  tells the requester, the words dropped from the record once passed
  and the entry once told, so that a step with room for one message
  passes them and the next tells. What a step leaves stays on the
  record, and its next step is due at once. At the step that passes
  them, the producer live then receives them; with none live, the
  requester receives the fallback feedback in place of the reply, and
  the words go no further. A refused step commits nothing: its words
  are taken, or passed, again at the step after it. A step refused because the
  producer's inbox or the requester's is full commits the failed-effect
  hold that a refused question commits (above), its words kept on the
  record, and the step after its release passes them. A restart resumes
  them from the record, a change holding words to pass having its next
  step due at once. Each is passed once.
- **Bounded.** The record keeps at most 16 words to pass. Words taken
  past them are not kept: the requester is answered at once, in words,
  `Not passed: 16 words already wait to reach this change's producer;
  send yours again once they are passed.`, as fallback feedback is,
  where the step has message room.
- **An amendment reaches its live producer.** An amendment of the
  change (jig's `domain/tasks/changes.md`), by whoever may amend it,
  taken at a step while the change has a live producer whose spec words
  are not those the change would give a new producer now (`README.md`,
  2, step 1: its words, then its scope, with *Early starts*' lines
  after their first line), amends that producer in the
  same step (jig's `domain/connectors.md`, 6.3): its spec as it stands
  with those words, naming its spec's revision as the core last told
  the change, and with the reason `Task C, your change, was amended to
  spec revision R. Your spec now holds its amended words; work to them
  from here on. Its reason: REASON`, C the change's number, R the
  change's spec revision after the amendment, REASON the amendment's
  own words, the whole cut from its end to the message bound and
  marked `…`. jig tells the producer as it tells any amended executor:
  a live run hears it as it arrives, a parked one is woken, and one
  kept by its own question hears it once that question is answered
  (jig's `domain/tasks/changes.md`, *An amendment is a message*). The
  producer's next run's brief reads the amended spec. Amendments taken
  at one step make one amendment of the producer, its reason the last
  one's. An amendment that leaves the producer's words as they are (its
  dependencies, its authority or its wake alone) amends nothing of it.
  With no live producer, nothing is amended: the next producer is made
  from the change's words as they stand. The amendment commits with the
  step that takes it, so a refused step leaves it in the change's inbox
  for the next, which names the revision as it then stands.
- **An amendment never answers.** It settles and clears no question:
  the change's question stays asked of its requester, and the
  producer's stays open.

### Early starts

The change procedure starts kept (jig's `domain/connectors.md`, 6.1): a
change made with a pending dependency is active and kept rather than
waiting, so its producer may work beside its dependencies while its
merge waits for them (`README.md`, 2, step 7; `../queues.md`). Its producer starts
early, by these rules:

- **When.** At producing, with no head of its own pushed, once each
  pending dependency that is a change into the same repository and
  landing branch has a checked head (below), and none of them is itself kept by a pending dependency.
  One level only: a dependency kept by its own question alone does not
  block. A pending dependency that is a change into another repository
  orders the landings alone (`../../landing.md`, 4.1): the producer neither
  waits for its push nor merges it. Any other pending dependency (a task
  that is no change, a change into another landing branch) has no head
  to start from, and the producer waits for it to be done. While the
  change waits so, at producing with no producer, its stall does not
  count.
- **A checked head.** A change keeps on its record its last checked
  head, `last_checked`: the last head one of its delegates (its
  producer, a repair or a resolution) pushed to its branch with its
  checks passed, as that delegate's answer reports it (the worker's
  `Landed.checked`, heard by the forge as `Pushed.checked`). A
  producer's result is such a push, since a change contract's finish
  checks and delivers (smith's `domain/run.md`, 8.5). An unchecked save
  (`../../agent/README.md`, section 5) never sets it, and a push made
  after the last checked one leaves it where it was, so a branch whose
  tip is a save keeps the checked head below it. A head found only by
  the read that frees a lost attempt's writer (`README.md`, 2, step 1)
  says nothing of checks and sets it neither. A report of a checked
  push sets it whether or not the change's head moved with it. A
  dependency's last checked head, never its branch's tip, is the head
  an early start counts and merges: the root gives each pending
  dependency's as `PendingKind::Change { checked, kept }`
  (`temper-engine-domain-forge-change`), and the change's start
  (`producer_start`) and the forge's read of its landing base for that
  start (`early_base`) both decide on that one fact, so they agree. A
  change's last checked head moving wakes its dependents' steps, as its
  branch's head observed does. It is stored with the change, so it
  holds across a restart. Why: on 2026-11-14 two changes started early
  from a sibling's save made before its question, and for an hour met
  that unfinished work's failures, asked about them and worked on a base
  that kept moving until the sibling landed
  (`../../../draft/early-start-checked.md`). A checked push is work its
  producer offered; a save is work in progress.
- **From what.** Its producer's workspace starts from the landing
  branch's tip with those last checked heads merged into it, one by one
  in the order of the change's dependencies (`../../worker/checkouts.md`).
  The change keeps each head it started from, with its dependency
  (`StartedFrom`), and marks those dependencies started from (jig's
  `domain/connectors.md`, 6.1). The checkout's selection names those
  recorded commits, never a branch's tip read afresh.
- **It is told.** The early producer's words carry one line per
  dependency it started from, in the order of its record:
  `[Started from change N at <commit>, not landed]`, N the dependency's
  task number and `<commit>` the head it started from, spelled as git
  spells it (`../../agent/README.md`, section 5). They are fixed facts,
  as a brief's `[This task: N]` is; what a producer and its requester do
  with them is the stock implementer's and coordinator's charters'
  (`temper/docs/charters/stock/`). They follow the first line of the
  change's words, so the producer's task keeps that line as its title,
  and a brief cut from its end (jig's `domain/engine/briefs.md`, 1)
  reaches them last. They are required words: when the spec's bound
  (beside its base and branch) can't hold the change's words, the lines
  and the scope, the scope gives way first; when it still can't hold
  the lines whole, they are cut from their end to the room left and
  marked `…`, never left out. For example, a change whose words are
  `Fix the parser`, a line break and its sections, started from
  dependency 12 at a SHA-1 head, gives its producer `Fix the parser`, a
  line break, `[Started from change 12 at <the head's 40 digits>, not
  landed]`, a line break, then its sections and its scope as before.
  Only a spec whose change's words alone fill its bound leaves them no
  room. They are made with
  the producer's words, so an amendment that renders them again keeps
  them, and a resumed run keeps its spec; a producer made once the early
  start is spent, or with no dependency started from, has none.
- **A conflict refuses it.** A head that does not merge cleanly fails
  the producer's preparation as a conflict naming it (`../../worker/checkouts.md`
  ). The change cancels that producer, its early start spent,
  and waits for its dependencies to be done; nothing is held or
  escalated, and its marks stay.
- **Once.** A change starts its producer early at most once. An early
  producer that ends failed, for whatever reason, holds nothing: the
  change waits for its dependencies to be done, then makes its next
  producer, whose failure for good holds it as `repairs.md` says.
- **The next producer merges the landing tip first.** A producer of a
  change that started early, or of a change into a landing branch whose
  rules ask no update (a goal's own branch, `../branches.md` and `gates.md`, section 2), whose
  workspace would start from the change's pushed branch or from saved
  work, starts instead from a merge in progress: that start, with the landing branch's tip as the
  base merged into it (`../../worker/README.md`, 4.1 and `../../worker/checkouts.md`). A clean merge is
  committed before its run starts, so the run finds a clean tree with the
  tip in its head. A conflict is left in its checkout for it to settle.
  Either way its first push delivers the merge. A tip that start already contains is no merge, so a resume with
  nothing new adds no commit, and the head its run reported stays the
  change's. One with nothing pushed or saved starts from the landing
  branch's tip, which now holds its dependencies' work. So a producer
  going on after an answer that added a dependency, or after an
  amendment naming a revision of the design that landed into the
  branch, works on the branch as it now stands, as one into main is
  brought up to date by the forge's update (`../README.md`, section 6).
- **Done while kept.** The early producer's result is the change's head,
  as any producer's: the change opens its pull request, and is checked
  and gated as its rules ask, but is not ready until its last dependency
  is done (`../queues.md`). Its head carries its dependencies' work as they
  had pushed it, which their landings contain.
- **A dependency started from stays.** Removing it would leave its
  unlanded work in the change's head, so an amendment leaving it out is
  refused, naming it (jig's `domain/tasks/changes.md`): its requester
  cancels the change and makes it again instead. One that fails or is
  cancelled holds the change, as any failed dependency does (jig's
  `domain/tasks/lifecycle.md`, 6).

## 2. Waiting on a person's approval

A change whose merge waits on a person's approval on the forge (`gates.md`, 2,
requirement 4), or whose step's facts show one short, shows under Needs you to whoever's review it waits on, with
its link (`../../../client/forge/README.md`, 4.1). It is a read, neither a task nor a
hold (`../../../draft/forge-identity.md`, section 8): a person task would ask for
an authority widened down the tree, to agents too, couldn't be opened by a
change made before its rule, and would need a stored number, a delegate slot
and a task per head; a hold escalates to the goal above first, wakes an
agent to no purpose and offers a Release that does nothing here. A change
it shows is out of its branch's queue meanwhile (`../queues.md`), so the
rest of the branch lands while a sign-off waits a night
(`../../../draft/what-reaches-you.md`, section 7).

- **The subject `approvals`:** one number, `APPROVALS_SUBJECT`, `(1 << 62) |
  (1 << 37)`, outside the task numbers as `TRIALS_SUBJECT` is (`../../trial.md`,
  section 6), of the party's own scope. The root makes its document, since
  roles and eligibility live with the people and the forge holds none: the
  core's ask names the watcher's party, and the root answers that party's
  rows. The party's own scope skips jig's project check (jig's
  `client/domain/contract/README.md`, 5.5), so the root filters every row by the
  watcher's role itself. One subject serves the deployment, as `dismissed`
  does (`../attention.md`, 1). It stores nothing.
- **The forge's part:** each live change, in a project that lands by queue or
  by turn, in state `Queued`, `First` or `Landing`, with a pull request, whose review
  facts (`gates.md`, 2) are at its last head, with an approval of an enforcing rule
  covering it, by its branch and the path sets it touches at that head
  (`gates.md`, 2, the kept `Touched`), still short and none refused, each judged as its
  panel judges them (`README.md`, 5). Of each: its task, project and forge; its
  repository's owner and name; its pull request's number; its title; when its
  facts at that head were first kept; whether its reviews were read whole;
  and its short approvals, each its scope, rule, role, required count, the
  people counted, and `sets`, the sets of its rule the change touches
  there, bit `n − 1` for set `n`, none for a rule naming no set. No change
  waits behind one it shows, which is out of its queue (`../queues.md`), so a
  row counts none. It carries its pull request's size at that head, as its
  approval evidence keeps it (`gates.md`, 2), none where it keeps none. Reviews read
  in part make a row as a whole read does. A change with no facts at its last
  head has none: one not yet judged, one that was held, and every change
  after a restart until its next step's facts are kept.
- **The root's part:** of those, the changes in a project where the watcher
  holds a role. The row's approval is the first short one whose role is the
  watcher's, else, for an owner of the project, the first short one; a
  watcher of another role has no row, nor has one already counted for that
  approval at the head. The row says whether the watcher's review would
  count, the first that holds of:
  - *too few:* fewer of the project's members count as people on the change's
    forge with the approval's role than it asks for (`eligible_people`, `gates.md`, 3),
    as when later membership changes, or a repository adopted from a forge
    with no link (`../adoption.md`, 6), leave a rule that can't be met;
  - *no account:* the watcher doesn't count as a person on that forge
    (`forge_human`, `../adoption.md`);
  - *another role:* the approval's role isn't the watcher's, as for an owner
    seeing a wait whose review isn't theirs to give;
  - otherwise *counts*.

  It names, too, the goal the change was made under, the outermost goal
  above its task, if any, for that goal's page to show it, and, for the
  row's approval, `sets`: each set of its rule the change touches, by
  number in order, `ApprovalSet { number, name }`, its name the project's
  classes' as the forge holds them (`../attention.md`, 6), empty for a number
  the classes hold no set at; none for a rule naming no set. The client
  says them on the card (`../../../client/forge/README.md`, 4.1). It carries
  the size the forge's part gives, each count kept in 32 bits as the row's other
  counts are, one past 4294967295 kept at it and the size marked cut, so
  that the card says so (`../../../client/forge/records.md`, the subject
  `approvals`).
- **Who can still sign:** the root's part names, too, who has signed the
  row's approval at the head and who can still give the next sign-off
  (`../../../draft/what-reaches-you.md`, section 7;
  `../../../draft/whole-story.execution.md`, N5). Nothing assigns a
  sign-off to one of them: any of them may give it.
  - `signed`: the people counted for the row's approval at the head, in
    the order the forge counted them, at most eight, each an
    `ApprovalPerson { person, name }`; `counted` counts them all, so the
    client says those past the eight as a number.
  - `can_sign`: the project's members that `eligible_people` counts for
    the approval's role on the change's forge (`gates.md`, 3), less the
    people counted for it at the head: the watcher first where among
    them, then in the order the people domain lists the project's
    members, at most eight, each an `ApprovalPerson`; and
    `can_sign_more`, how many more there are past the eight. The watcher
    is among them exactly where their review would count or the row is
    *too few* and they hold the role with a forge account.
  - A person's `name` is their identity's display name, its login where
    the name is empty, its first 64 bytes at most, cut at a character's
    boundary (skein's `bytes::utf8_prefix`); empty for a person the
    people domain has no identity of.
  - A row's largest size, which bounds how many rows a delivery holds
    (*Its rows*), counts sixteen such people at 64 bytes of name each.
- **Its rows,** oldest first by when their facts were first kept, then by
  task: at most 32, and as many as one whole delivery holds at the largest
  row, as the trials' are bounded (`../../trial.md`, section 6); `more` counts the
  rows past them.
- **Marked** wherever `changes` is (`changes_mark`), and when a change's
  review facts are kept anew or dropped (`gates.md`, 2). A role or a link that changes
  shows at the next mark.
- **The client** merges the rows with jig's inbox and draws each as a card
  (`../../../client/pages/README.md` and `../../../client/forge/records.md`; `../../../client/forge/README.md`, 4.1).
- **What a proposal says of such a rule:** the forge's read of a pending
  proposal's items (`../../../protocol/configuration/settings.md`, 2, *Read from the
  forge*) answers too whether one of its landing rules has an enforcing
  approval covering a writable repository on an owned Forgejo, a box's
  (`../../../protocol/setup/box.md`, 1), the root judging the coverage as it does for
  eligibility (`gates.md`, 3). There agents and CI jobs run as the box's user, who can
  read the person's write credential (`../../../protocol/setup/README.md`, section 9), so
  such a rule stops their mistakes, not one trying to get past it, and the
  client says so (`../../../client/pages/settings.md`, 2; `../../../draft/forge-identity.md`,
  section 6).

## 3. Open questions

- **A refused `message` keeps no record** (`questions.md`, 1): the call's refusal of an
  agent's own words is rendered and kept with no call's record, as the
  `goal` tool's refusal is, where a decided call's answer is kept
  (`../../routing.md`, 2). So a call asked again decides again, from the change as
  it then stands. Default: so.

- **A passed-up question whose asker ends** (`questions.md`, 1): its requester hears of the
  end when it answers, not before, since saying so once at the end would need a
  stored mark beside where the question stands. One that never answers hears
  nothing of it, and the question stays until the change ends. Default: so; it
  did reach someone.

- **A report's head past a merge** (`questions.md`, 1): a next producer's start that merges a
  tip it lacks makes a new head before its run if clean, or at its first push or
  save once its conflicts are settled. A report written before that merge names
  an older head; a contained tip makes no new one. Default: so; the head delivered
  names what is on the branch (`../../agent/README.md`, section 5).

- **A route `up` with no Message family to keep** (`questions.md`, 1): the
  change keeps its asking agents' Message family, never adds it, so a
  change whose own authority lacks it (made by a caller without it, or
  amended to an authority, whole, that leaves it out) delegates agents
  that cannot ask, though it would relay their questions. Refusing such a
  making or amendment would bind jig's amendment to a forge route.
  Default: so; `decided` tells such an agent that it can ask no one
  (`../../agent/tools.md`).
