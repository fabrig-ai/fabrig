# Amending, cancelling and moving

- **Who may:** a task's requester, and any task above it in the tree; a
  party in whose tree it is, or whose role in the project allows it
  (../people/acts.md, 1). Nobody else. An escalation's holder decides the
  escalation (`lifecycle.md`, 6; `proposals.md`). A party's request, an agent's tool and a
  procedure's step reach the same rules (../people/acts.md, 1; ../engine/executors.md, 3;
  ../connectors.md, 6.3). The deployment has a cancel within its project
  (`messages.md`, section 1) and one release: a procedure task of its project from a hold
  of its own, whose cause its connector found gone (../connectors.md, 6.3),
  never where a placed hold or a party's stop also holds it, and recorded as
  the deployment's. No party, tool or step asks for that release.
- **What an amendment may change:** the spec and its parameters; the wake
  policy; the dependencies, added or removed (below); the authority,
  narrowed freely and widened only within
  the amender's own (beyond it, refused, and proposed if the amender asks:
  `proposals.md`), the amender, or the proposal's accepter, funding the
  widened task from then on (../authority.md, section 7); an agent task's
  instructions; a procedure's parameters, which go to its connector; a
  person task's address, to another party or role, naming the person
  task's revision as an amendment names the spec's, a party who took it
  holding it no more; whether it is tracked, within goals per project
  (`README.md`, section 6), and its priority; an agent task's LLM choice, within the
  amender's authority as a batch's is (`README.md`, section 4). An amendment is
  refused once a task is closing. Its label and its flags are never
  amended (`README.md`, section 3).
- **Host specification parameters.** A host `amend` that supplies spec
  words but omits parameters keeps the live task's parameters. An
  explicit list replaces them whole, including an empty list. The host
  decoder carries omission separately from the decoded list; the core
  resolves it against the current specification before amendment
  admission and commit. Creation still defaults omitted parameters to
  an empty list; stored specifications remain whole.
- **A new LLM choice is for the next run.** A run in flight keeps its
  own models and account; the next run's check uses the new choice and
  chooses its account again when its main model's LLM provider changed
  (../accounts.md, 4.3). The new choice replaces the old whole, its `later`
  with it (`README.md`, section 3): one given without `later` leaves the task none, and
  the next run takes its first choice or its `later` by its reasons (`messages.md`, 6). A conversation that moves to another LLM provider
  or another account is carried over, its text, calls and results kept
  and the provider's opaque replay dropped (smith's
  `protocol/transcript.md`, section 4). An amendment naming another
  account of an LLM provider whose account the task keeps is refused
  while that account is limited (../accounts.md, 4.3; ../providers.md, 2.5 and
  3.4). An accepted explicit different account clears the pin in that
  amendment's commit, even when the preference itself was already the same;
  the next successful claim keeps the new account. Clearing the optional
  preference alone keeps the pin. The live claim retains its recorded binding.
- **On a task its requester ends, the next run is the next turn.** Its run
  listens after a reply (`lifecycle.md`, 9; smith's `domain/run.md`, section 6), and what
  reaches it is answered on that run's models, so a person who kept talking
  would keep the old choice. An amendment that changes such a task's LLM
  choice, any of its models, efforts or account preference:
  - **Says nothing.** One that changes nothing else puts no word in the
    inbox: it wakes no idle task, reaches no live run and is no message of
    the next run. Its history records it as any amendment's, and no provider
    is asked anything.
  - **Leaves what runs.** A turn under way finishes on its models, and a run
    left listening parks at its waiting time as ever.
  - **Marks a live run behind its choice.** Nothing more is relayed to that
    run. The next message whose wake rule would have it reach the run (`messages.md`, 3)
    stays in the inbox and stops the run, as a narrowing does: its terminal
    parks it with no try spent, and the task goes due for that message. The
    next run takes the new choice and carries the message. A turn under way
    when such a message comes stops with its run: what it committed stays,
    and the words it had not read are the next run's.
  - **Resumes as any change does.** The words are answered on the new
    choice. A changed model in a resumable window compacts first with the
    window's own model, when configured
    (smith's `domain/session.md`, 8.2): the summary names the old model,
    and the answer the new one. A window carried across an endpoint,
    dialect, account or backend boundary, or without its saved program
    session, compacts on the new model, dropping opaque replay. A changed
    effort alone keeps the window.

  Only such an amendment marks a run behind: nothing else that moves what a
  next run would take, a revision of settings or an account's state, stops a
  run. A task its result ends hears an LLM amendment as any amendment, and
  its next run takes the choice.
- **Adding and removing dependencies.** An amendment names a task's
  whole list of dependencies, with its spec's revision: what it names
  anew is added, what it leaves out is removed, at any phase before
  closing.
  - *What may be added:* a task the amender could name in a batch
    (`README.md`, section 4), or, for a task, any task below it. An added dependency
    moves no authority and funds nothing.
  - *Out of reach:* a dependency the amender may not add is refused as
    `Dependencies` naming exactly that dependency in the problem's
    `blocked_by`; a refusal of too many dependencies, or of one named
    twice, names none there. So the amender hears which dependency is
    outside its reach, and the other refusals stay told apart.
  - *An ended one* is judged on its stored row, which the core loads
    (../engine/README.md, 5.3), by its place in the tree, as a live one is. For a
    task amender, it is reached when the amender was introduced to it (`messages.md`, 5), or
    when its requesters, read up the tree until the first live one, reach
    the amender: a former delegate is one, and nothing above the
    amender's depth is read. For a party, it is reached when it or its
    root was the party's request, or the party reaches its live root; a
    maintainer reaches any task of the project. A stub the working set
    keeps of it holds no place, and a live task naming it widens no
    reach: neither decides. One the store does not hold is refused as
    unknown, naming it. The core loads the row only for an agent's tool
    and a party's own request. An amendment in a party's compound
    request, one proposed (`proposals.md`) when it is proposed and when it is
    accepted, and a procedure's step's are judged without it: there an
    ended dependency the working set holds no live row of is refused as
    unknown, naming it. That narrows reach, and widens none.
  - *By phase:* a waiting task waits for it as for any other, and leaves
    the queues it was in (`holds.md`, 2); an active one is kept (`lifecycle.md`, 8); a held one
    is released, when it is, with it pending.
  - *Its prerequisite:* one already done counts as met, the add recorded
    and nothing waiting, so an amender racing its end is not refused. One
    failed or cancelled is refused, naming it. One that fails or is
    cancelled later holds its dependent (`lifecycle.md`, 6).
  - *Circles:* an add that would close a circle of waits is refused,
    naming its tasks (`holds.md`, 7); a task after itself or after an ancestor is
    the plainest.
  - *Removing* one lifts it: a task that waits on nothing else goes on,
    a waiting one to its queues at its place, a kept one woken.
  - *Started from:* a dependency its procedure marked started from
    (../connectors.md, 6.1) is never removed: an amendment leaving it out
    is refused as `Dependencies` naming exactly it in the problem's
    `blocked_by`, as one out of reach is, whether it is pending or
    done.
  - *Kept:* each added dependency with when it was added and by whom
    (`README.md`, section 3), and its history as every amendment's.
- **Tracking and untracking.** A task an amendment tracks is a goal from
  that commit, with all a goal has (../model.md, 6.2). One it untracks is no
  goal from that commit: the core ends its goals' topics, and its
  projections write their last, as when a goal closes (../connectors.md,
  section 8). Neither touches its flags, so a conversation stays one:
  tracked, its turns are still entries for its requester, with their
  read position and unread count (`README.md`, section 3; ../people/README.md, section 5). No
  tracking is refused for a task's being a conversation.
- **A decision names what it saw.** An amendment names its task's spec's
  revision, and a release its hold's (`lifecycle.md`, 6), whoever makes it: a party,
  as their client showed it (`../../client/domain/contract/documents.md`, 2); a
  run, as it last read it (../engine/executors.md, 3); alone, or in a compound
  beside words or an amendment. One whose revision has moved is refused
  as moved, saying what it is. No message releases a task: words to a
  task its party stopped come with a release, which names the hold's
  revision as any release does (../people/acts.md, 1). A priority moves no
  revision, and an amendment of the priority alone names none: the
  first commit decides (../people/acts.md, 1). A cancel, a move and a stop
  name none.
- **Instructions are replaced whole.** An agent task runs with its
  charter's instructions until an amendment gives it its own. An
  amendment's instructions replace the task's whole instructions, and
  empty ones leave it with none, not with its charter's; an amendment
  that leaves them out changes nothing of them. Its history records the
  instructions as the task held them, its own or none of its own, never
  which charter's text was in effect. A task with instructions of its own
  reads them alone, none of its charter's parts with them, and its runs are
  offered every message, as under a charter with no parts (`messages.md`, 6).
- **An amendment is a message** to the task's executor, committed with
  the change: it wakes an agent, steps a procedure, and shows a person
  what changed. One that keeps an idle agent task (`lifecycle.md`, 8) wakes it only
  once it goes on. A run already live hears it as it arrives; a narrowing it
  cannot honour (a grant its live run is using) cancels that run, and the
  next starts with the narrower authority.
- **Cancelling closes down the tree.** A cancelled task, and every task
  below it, is closing (`lifecycle.md`, 1), each with the same reason: live runs are
  cancelled, pending proposals withdrawn, outbox entries not yet sent
  withdrawn, and those in flight settle, since what may have been made
  counts; a task waiting for holds leaves their queues. Each task ends
  once its own closing is done, deepest first, and its requester hears
  that it ended cancelled, with what it had done.
- **A project deleted** by the deployment's act (an owner's is refused
  while any is live: ../people/acts.md, 2) cancels each of its live tasks by the
  deployment's compound (below), its conversations, their delegates and
  its other roots composed into compounds as ../people/acts.md, 2 has it, and
  takes no new task from that decision; once none is live, each of its
  tasks is erased whole (`README.md`, section 3).
- **Cancelling with a grace.** A cancel may give a grace, within the
  longest the tasks child allows (`README.md`, section 6), so that a task finishes
  what it can before it is cut off: a conversation closed by its person,
  a run asked to report what it has. The task hears that it is asked to
  finish, with the durable `FinishRequested { revision }` message kind
  (`messages.md`, 1), separately reserved and never merged with amendments: an agent's
  run is woken by it,
  and a procedure's connector and a person task's holder hear it as they
  hear an amendment. It is cancelled, as above and with the cancel's
  reason, when the grace ends, unless it ended first. The grace's end is
  kept with the task, on a timer of the tasks child's, so a restart
  keeps it. A task that cannot finish on its own, waiting, held or kept
  with no live run (`lifecycle.md`, 8), is cancelled at once. A cancel made during a
  grace ends the task no later
  than that grace would: one without a grace cancels it at once. One with
  a grace asks nothing more, since a task is asked to finish once, by
  the first. If its grace ends sooner, the task's grace now ends with
  it, and the task is then cancelled with this later cancel's maker and
  reason, since that cancel is what ends it. If its grace ends no sooner,
  nothing of the task changes.
- **Moving a task.** A party may make themselves the requester of a live
  task they may amend, so that it outlives the task that asked for it,
  as a goal outlives the chat it came from. Its reservation moves to
  their funding (../authority.md, section 7), and its next run keeps its
  LLM account only if the account serves them (../accounts.md, 4.3); the
  task that asked for it keeps a reference to it, and does not wait for
  it as for a delegate. It becomes the root of a tree of the party's, so
  a move needs their escalations to have a holder, as their batch does
  (`README.md`, section 4).
- **Moving a task under another task:** whoever may amend both a live task and another
  live task may make the second the first's requester. The second must
  not be the first or below it, must be in the same project, and must
  have the depth and delegates' room the move needs; the tree it makes
  must close no circle of waits, checked as `holds.md`, 7 checks a new wait (a
  task moved under one it depends on is refused, naming the circle);
  the first's authority must be within the second's, or the move is
  refused, naming what it lacked, and may be proposed. Its reservation
  is taken from the new requester's available budget, in the move's
  decision, and refused, naming the shortfall, when it is not there;
  what it spent stays its old funder's. Its results, questions and
  escalations go to the new requester from then on; the old requester
  keeps a reference to it, and does not wait for it as for a delegate.
  A party's `Move` and a run's `move` make it (../people/acts.md, 1;
  ../engine/executors.md, 3).
- **A successor:** a live task in the same
  project that takes on what an ended task leaves.
  - *Named in the result,* by the ending task: one it references (a
    delegate of its requester it was introduced to, or its own
    delegate's), within the reach an amendment's dependency has
    (above).
  - *Named later,* while the ended task's handoff record lives (below):
    by its requester, a task above it, or a party in whose tree it is,
    or a maintainer, naming a live task within their own reach.
  - *What goes to it,* in the decision that names it, admitted whole
    against its bounds (its inbox's messages and bytes, its open
    questions, its subscriptions) or refused whole, naming the bound:
    the open questions the ended task left (below), each keeping its
    identity, so that its answer is delivered once, to the successor;
    the news the ended task kept unread, as news; its notes, by a
    reference to their scope (../engine/briefs.md, section 2); and, only when
    named in the result, its subscriptions. Its requester still hears
    its result.
- **Questions outlive their asker:** a task that
  finishes done or failed with person tasks still open that ask in
  words detaches them in its finish's decision, before its closing
  waits for its delegates (`lifecycle.md`, 1), so that they never keep it from
  ending. They move to its successor if its result named one, and
  otherwise stay in a handoff record of the ended task's, which keeps
  its stub until they end or a successor takes them. An answer to one
  reaches the successor, if one has been named by then, as the answer
  to the question; otherwise its requester, as news naming the question
  and the ended task. A cancelled task's questions are cancelled with
  it.
- **Several operations, one decision.** A party's compound request
  (../people/acts.md, 1) cancels, moves, amends, releases, messages and stops
  in one decision, all or none: so a conversation is closed in one
  decision, each of its live delegates cancelled or moved to its party,
  then itself cancelled with a grace; a project's goals are reordered at
  once; a held task is amended and released. The deployment's compound,
  a connector's act in its name (../people/acts.md, 3) or a project's
  deletion (../people/acts.md, 2), only cancels: an operation of another
  kind, or on a task of another project, refuses it whole.
