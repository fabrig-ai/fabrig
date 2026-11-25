# Proposals and escalations

A proposal is an action a task or a party may not take alone, waiting
for a holder of the authority it needs; an escalation is a held task
waiting for a decision. ../authority.md, section 9 says what may be
proposed, what an action needs, and what an acceptance funds.

## 1. Proposing

- **Proposing is asked for.** Nothing becomes a proposal on its own: a
  batch or an amendment beyond its maker's authority is refused, naming
  what it lacked and whether a proposal could be accepted (`README.md`, section 4),
  and its maker proposes it, with reasons: a run with its `propose`
  call (../engine/executors-tools.md, 1), a procedure in its step (../connectors-procedures.md, 1.3), a
  party with a `Propose` request (../people/acts.md, 1). So a person is never
  asked by accident, by a program or by their own client. The one
  exception is a result's follow-ups beyond its task's authority, which
  its verdict's contract allows, and which become one proposal as the
  result commits (`lifecycle.md`, 7).
- **What each may propose.** A task proposes any action: a batch, an
  amendment, an effect, or a widening of its own authority. A party
  proposes a batch or an amendment: a party makes no effect, and their
  authority is their role's, which the project's owners change
  (../people/acts.md, 2). Either may propose a change of settings, in the
  deployment's scope (below), and a task's
  release from a placed hold, which waits for the hold's owner alone
  (`lifecycle-holds.md`, 2).
- **A proposal of settings** (../engine/settings-proposals.md, 1) holds a set of settings
  against a revision. It is proposed by a party with a role in the
  deployment or a project, or by a run whose charter gives it the
  settings family; no task's authority covers it, so it waits from the
  start for the owners its items need: the deployment's, and each
  project's whose items it names. The core keeps its set under its
  number; this child keeps the rest as for every proposal. Accepting it
  is the change's decision, the accepter choosing whether running work
  moves; a revision made since its own withdraws it, as no longer
  applying.
- **A run's fresh proposal of settings replaces its own.** A task holds
  one pending proposal, and another is refused as busy, but for this: a
  task whose pending proposal is of settings, proposing settings again,
  withdraws its own in the commit that admits the new one. The new one's
  every check runs first, so one refused leaves the pending one as it
  was. Nothing else is replaced: a pending proposal of any other kind
  still refuses the next, a proposal of settings refuses any other kind,
  a party's stays one at a time, and no other proposer's is touched. The
  one replaced ends as any task's withdrawn proposal does: its set goes,
  and a decision asked of it after is refused, naming nothing, and changes
  nothing. Its proposer hears no message of it, having asked.

## 2. State and admission

- **Their state.** A proposal: its proposer, a task or a party; the
  action, as the call, the step or the request asked for it; its reason,
  in words;
  whether the tasks it makes are to be the accepter's; where it waits;
  and pending, accepted, rejected or withdrawn. An effect's own
  description stays with its connector, under the proposal's number
  (../model.md, section 5). An escalation is part of its held task's state:
  one per held task, never more. Both are kept in the store, and pending
  ones in this child domain.
- **Admitted once.** A proposal is admitted at its entrance, the
  `propose` call, the step or the party's `Propose` that asks, against the
  pending proposals allowed per proposer and in the deployment, and
  refused there if they are full; an escalation needs no room beyond its
  held task. Neither takes room in any inbox: each shows as an entry in
  the inbox of whoever it waits for, and those it passes on the way hear
  a notice, which merges.

## 3. Where they wait

- **Where they wait.** At the nearest task above the proposer, or the
  held task, whose authority covers the decision (../authority.md,
  section 9), skipping procedures, kept tasks and conversations: procedures
  decide nothing, kept tasks start no run to decide (`lifecycle-endings.md`, 1), and a
  conversation is ended by its requester (`lifecycle-endings.md`, 2). Above the tree's
  root task is whoever requested it: a party is the next holder there,
  when their authority covers it; past them, or past a root the
  deployment requested, it waits for the parties whose role the project's
  policy names for that kind of decision (../people/README.md, section 4). A
  party's own proposal waits there from the start. Where one waits is
  decided again when that holder ends, is held or kept, or no longer
  covers it, and when it stalls.
- **A proposal nobody may accept is refused.** A proposal is never
  committed to wait for nobody. When the first holder it would wait for
  is the parties the project's policy names (a party's own proposal; a
  task's, when the deployment requested its tree's root, as for a
  standing procedure's or a recurring task's tree, or when nothing in
  its tree covers it) and the policy names no role that may accept its
  kind, it is refused to its proposer as beyond authority, saying that
  no proposal could be accepted, and nothing of it is committed. A
  party's request is refused so at admission (../people/acts.md, 1); a run's
  `propose` call is answered so; a procedure's step's proposal is
  refused in its decision. A proposal already pending waits as this
  section says.

## 4. Waking

- **Waking.** Its arrival wakes the task it waits for (`messages.md`, 3), is relayed
  to that task's live run as messages are, whatever the run's parts (`messages.md`, 6),
  and is in the brief of its next run (../engine/briefs.md, section 1). Left
  undecided when that run ends, it wakes the task again for its kind
  unless that run was offered its kind, when it waits for a change (`messages.md`, 6).
  An escalation's change is any message reaching its task: while one
  waits for a task, every message the task hears wakes it, whatever its
  policy says of the kind, a delegate's result that would ride along
  included (`messages.md`, 3); a proposal's is a new entry or a message
  about it, waking by its own kind. For a party,
  it appears in their inbox at once. Waiting for a person neither decides
  the escalation nor resets its stall time.

## 5. Deciding

- **Deciding.** A holder accepts, rejects with a reason, or passes it up
  to the next holder that covers it. An accepted proposal is made as the
  proposer's action, with the accepter's authority funding what it
  needs, in the commit that accepts it; the tasks it makes are the
  proposer's (a task's delegates, a party's roots), or, when the proposal
  asked, the accepter's (a chat proposing a goal asks so); a task it
  amends or widens is funded by the accepter from then on, whoever
  proposed it (../authority-budgets.md, section 1). A rejection is
  a message to the proposer: a task hears it in its inbox, and a party
  finds it in theirs (../people/README.md, section 5). A party's proposal to amend
  a task is the party's, not that task's. A party who does not cover a
  proposal may reject it or pass it up, never accept it. At the parties
  the policy names there is no one further: they accept or reject, and
  the proposal waits for them, however long. Passing a proposal up where
  nobody further may accept it is refused, saying there is no one
  further.
- **An escalation's decider has its kind.** An escalation left held
  records who left it so, a task or a person, never a bare number, as a
  decided proposal records a party: a task when it waited for a task, the
  nearest live one above the held task; a person when it waited for a
  person or for a policy's role, whose member decided. A task that
  decides it is an ancestor of the held task and ends after it
  (`lifecycle.md`, 7), so a restart finds it live: the start links the
  decider by its kind, a person among the people and a task among the
  live tasks, and refuses a store where it cannot
  (`../../protocol/store/README.md`, 10.1). The held task's document
  names the decider as a task or a person with its number; the client
  wire and the client's words name its number alone, and whether they
  show its kind is an open question (below). The escalation's holder,
  decision and revision are as before.
- **A decided entry stays read.** A holder's run may decide an entry in
  the turn that first reads it, as a model does: the decision commits
  with its call, before the turn naming the entry as read. The entry
  leaves the holder's inbox with the decision, but its number stays a
  fence the run may name for the attempt it was offered to, until a
  turn's fence reaches it or the attempt ends; the turn is admitted,
  taking nothing more for it (`messages.md`, 2). Only an entry offered to that
  attempt stays so: a fence naming one never offered is refused, as
  before.

## 6. Stalled and withdrawn proposals

- **Stalled** past its bound, undecided, a proposal or an escalation
  passes up to the next holder that covers it, and past the last, waits
  for the parties the policy names.
- **Withdrawn** when the proposer withdraws it (a run's call, a
  procedure's step, a party's `Withdraw`), when its task's fresh proposal
  of settings replaces it (above), when a proposing task closes,
  or when the state it was made against has changed so that it no longer
  applies (a batch naming a task since ended); an escalation ends with
  its hold. The
  proposal a task's result made is the exception: its closing waits for
  it to be decided (`lifecycle.md`, 7), and only a cancel of the task withdraws it.

## 7. Open questions

- **An escalation's decider on the client wire.** The chip's hold on the
  client wire carries a rejection's decider as its number alone
  (`HoldEscalationRejected.by`), the engine giving a task's or a
  person's number alike, and the client's contract and words show it so
  (`../../client/domain/contract/documents.md`, 1). Showing its kind, a
  `Requester`'s `Task` or `Person`, changes the client wire's layout
  (`../../protocol/client/versions.md`, 1) and records again the journeys
  that show a task's chip; whether and when is a later change of the
  client wire's (B1's requester, 2026-11-18).
- **A proposal waiting for a change.** An escalation waiting for a task
  makes every message wake it (`messages.md`, 3); a proposal its holder's
  run left undecided still waits for a new entry or a message about it.
  Whether a proposal should make every message wake its holder too is
  undecided: the fix of 2026-11-24 was asked for escalations alone.
