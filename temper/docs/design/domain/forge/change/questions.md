# A change's questions and early starts

## 1. Its questions and early starts

### Its questions

A change made with questions `up` (`README.md`, 1) relays its agents'
questions: its producing, its repairs and its resolutions ask it, it
passes each to its requester, and it answers the asker. It decides
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
  widened, a Left item accepted: `../README.md`, section 5 of jig's `domain/tasks/README.md`)
  or make a new change and add it as the change's dependency; its
  answer then says so.
- **Held until its dependencies land.** An answer the change takes
  while its task is kept by a pending dependency is held on its record
  and relayed at the first step at which none is pending, the
  dependency having ended done or been amended away; a requester that
  adds a dependency as it answers so makes the work wait for it.
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
  requester; its answer held for dependencies), and the questions
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

### Early starts

The change procedure starts kept (jig's `domain/connectors.md`, 6.1): a
change made with a pending dependency is active and kept rather than
waiting, so its producer may work beside its dependencies while its
merge waits for them (`README.md`, 2, step 7; `../queues.md`). Its producer starts
early, by these rules:

- **When.** At producing, with no head of its own pushed, once each
  pending dependency that is a change into the same repository and
  landing branch has a pushed head (its branch's head as the forge last
  read it, `../keeping-up.md`, 2), and none of them is itself kept by a pending dependency.
  One level only: a dependency kept by its own question alone does not
  block. A pending dependency that is a change into another repository
  orders the landings alone (`../../landing.md`, 4.1): the producer neither
  waits for its push nor merges it. Any other pending dependency (a task
  that is no change, a change into another landing branch) has no head
  to start from, and the producer waits for it to be done. While the
  change waits so, at producing with no producer, its stall does not
  count.
- **From what.** Its producer's workspace starts from the landing
  branch's tip with those heads merged into it, one by one in the order
  of the change's dependencies (`../../worker/checkouts.md`). The change
  keeps each head it started from, with its dependency, and marks those
  dependencies started from (jig's `domain/connectors.md`, 6.1).
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
