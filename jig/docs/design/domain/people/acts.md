# What a party does

## 1. Requests

A party's requests are jig's plumbing: its primitives as operations, the
same an agent reaches through its tools (../engine/executors.md, 3), and checked by
the same rules (../authority.md, 8.1 and 8.4). Every request is keyed by
its client (a double click makes one request), checked against the
party's role, committed, and answered once durable (../engine/README.md, 5.2). A
task is in a party's tree when they requested it, or a task above it.

Ordinary deciding requests reserve a place for their original answer at
admission and retain it for the advertised retention, success and
permanent routed refusal alike. This capacity is shared engine-wide,
not per party. Unexpired answers are never evicted for another key. A
request refused for want of a place is answered busy, *answers full*,
with the table's size and its oldest answer's expiry
(`../../client/domain/contract/requests.md`, 1), so the person and the operator
see the bound and when it frees; its size is the deployment's
(`people.requests`), sized for a day of the deployment's deciding
requests at the retention it advertises.
Busy and not ready are not saved. After authentication and validation,
retained-key replay or conflict and pending-key joins or conflicts are
checked before a new request needs room. An identical pending request
joins its waiters; a different ask under that key conflicts.

A top-level `Seen` uses the same keyed transport and pending checks,
but bypasses completed-answer capacity and saves no `Answer`, on success
or routed refusal, so it progresses while the answers' places are full.
It needs the bounded pending, flight, waiter, load, journal and output
room. Ordinary admission counts retained answers plus only the ordinary
flights that reserve answers, excluding `Seen`; a bounded scan of
pending flights suffices, with no counter or limit of its own. Each
admitted ordinary flight thus retains its reserved place through saving
its answer.

Keeping no answer, a `Seen`'s key is free once it completes. A retry is
processed against the current subject and authority, and a different
ask under the freed key is a new request.

The operations as the client sends them are
`../../client/domain/contract/requests.md`, 2:

| Operation | Needs | Becomes |
|---|---|---|
| `Create`, a batch | member, within their role's authority and their pool (../authority.md, 8.4); its dependencies and inputs within their reach, and a holder for its escalations (below) | its tasks, with their dependencies, checked whole and made at once (../tasks/README.md, section 4), each with the fields it names (../tasks/README.md, section 3); the party their requester and their funder; a budget given `AsAllowed` resolved, and what was given answered (../authority.md, section 7) |
| `Propose`, an action | member; a batch or an amendment beyond their authority, of a kind a role the project's policy names may accept | a proposal of theirs, waiting for that role (../tasks/proposals.md), its number answered |
| `Restore`, a version of the settings, in the deployment's scope | a role in the deployment or a project; a revision the history keeps | a proposal of the settings as that revision left them, against the newest, waiting for the owners of its items (../engine/settings.md, 10) |
| `Propose`, a change of settings, in the deployment's scope | a role in the deployment or a project; the revision they saw; an owner who may accept each item (../engine/settings.md, 6) | a proposal of settings, waiting for those owners, its number answered |
| `Withdraw` | their own pending proposal | withdrawn (../tasks/proposals.md) |
| `Message` | the task in their tree, or maintainer | words in its inbox (../tasks/messages.md, 1), their message number answered |
| `Talk` | as `Message`, to a task that stands; no other party's talk open with it but a lapsed one | its words as a `Message`'s, and a talk opened, its open session ended, or their own talk carried on (../tasks/standing.md, 5.6); their message number answered |
| `Leave` | the talk open with the task, theirs, or maintainer | the talk ended, and its session (../tasks/standing.md, 5.6) |
| `Amend` | the task in their tree, or maintainer, and a change within their authority; the spec's revision they saw, but for a change of priority alone; maintainer, for tracked or priority; dependencies added within their reach, as a batch's | ../tasks/changes.md: its spec, instructions, authority, wake policy, a person task's address, whether it is tracked, its priority, its dependencies added or removed; a widened task funded from their pool (../authority.md, section 7) |
| `Cancel`, with or without a grace | the task in their tree, or maintainer; a grace within the tasks child's bound (../tasks/README.md, section 6) | the task and its tree closing (../tasks/changes.md); with a grace, the task first asked to finish, and cancelled when the grace ends unless it ended first |
| `Release` | the task in their tree, or maintainer; the hold's revision they saw; for an effect held uncertain, whether to make it again; for a task a placed hold holds, that they may lift it (../tasks/lifecycle.md, 10) | its hold lifted, the task judged again (../tasks/lifecycle.md, 6); from a placed hold that stands, the task exempt from it |
| `Stop` | the task in their tree, or maintainer | a placed hold of theirs in the stop mode over the task's subtree, its live runs cancelled (../tasks/lifecycle.md, 10) |
| `Hold` | a mode, stop or drain; a scope, a task in their tree (or maintainer) for its subtree, or the project for a maintainer; a reason; never in a compound request | a placed hold of theirs (../tasks/lifecycle.md, 10), its number and revision answered |
| `Lift` | a placed hold they may lift (../tasks/lifecycle.md, 10); its revision; whether every task it holds that nothing else holds is released with it; never in a compound request | the hold ended; with the release, those tasks judged afresh |
| `Move` | they may amend the task, and its requester is another (their own is not offered: it would change nothing); a holder for its escalations (below); or another live task they may amend, as its new requester | the party its requester, funded from their pool (../tasks/changes.md); or the task its requester, funded from its funding |
| `Succeed` | a task in their tree that ended, or any of the project's for a maintainer, while its handoff record lives; a live task within their reach | the live task its successor, what passes to it admitted whole (../tasks/changes.md) |
| `Decide` | the proposal or escalation waiting for them, as the requester of its tree or in a role the policy names; for an escalation, its hold's revision; for a proposal of settings, whether running work moves | accepted, if their authority covers it, and funded from their pool (../authority.md, section 9: a batch's tasks, an effect's price, the task an amendment or a widening changes), or, for a proposal of settings, by an owner of each item it names, funding nothing (../engine/settings.md, 6); rejected, with a reason; or passed up (../tasks/proposals.md) |
| `Answer` | addressed to them or their role; a person task's revision as they saw it; for a person task, none of its dependencies pending (`README.md`, section 7) | an answer to a question, or a person task's result (`README.md`, section 7) |
| `Take`, `HandBack` | a person task addressed to a role they hold | held by them, so no other answers it; or back to the role (`README.md`, section 7) |
| `Seen` | the party themselves | their read position moved over results, or over one conversation's turns (`README.md`, section 5) |
| a compound request | one project; each operation's own needs; within operations per request | its operations made in one decision, or none of them (below) |
| adopt a resource, as a home or not; rename the project; change the policy, roles or services | owner; a name no other project holds, in shape (../model.md, 6.1) | the project changed (2) |
| `ChangeSettings`, in the deployment's scope | owner in the deployment for the deployment's items, and owner of each project whose items it names (`README.md`, section 3 and 4); the revision they saw, the newest; the set within the checks and bounds of ../engine/settings.md, 1 and 8; every connector judging an item of its kinds accepting it | the settings changed in one decision, the new revision answered; running work staying or moved, as it asks (../engine/settings.md, 2 and 4) |
| `MoveToNewest`, in the deployment's scope | owner in the deployment; a revision held | every live agent task on it that may move set to the newest, in one decision, those moved and those that could not counted in the answer (../engine/settings.md, 4) |
| `MakeProject`, in the deployment's scope | owner in the deployment (`README.md`, section 3); a name in shape that no other project holds (../model.md, 6.1); a policy among the settings (../engine/settings.md, 1); room in the deployment's projects | a project, numbered next, its policy a copy of the configured one, its maker its owner and only party (2) |
| `RemoveProject` | owner of the project; a project that is live, not being deleted; a grace within the tasks child's bound (../tasks/README.md, section 6), or none; never in a compound request | the project closed in the decision that admits it, answered removed then, and deleted whole in the steps that follow (2), each of its live conversations asked to finish within the grace |
| `SetVisibility` | owner of the project, never project 0; a project that is live, not being deleted | the project public or private from that decision, `me` saying so (`public-reading.md`, 2) |
| edit or delete a note | the note's scope within their role | the note revised (../engine/briefs.md, section 2) |
| set a party's pool | owner | the pool opened or resized in the current period, what was spent and reserved kept |
| an account request: sign in, sign out, add, replace or remove a key, declare or check a program's login, order, uses, resume, remove an account | the account's holder, not a role: a person for their own, a first owner for the deployment's (../authority.md, 8.4) | ../accounts.md, section 5: the account's sources, states and uses changed; a value never committed |

- **Words are never cut.** A party's words, in a `Message` or answering
  a question, are within the people child's bound, which is at most what
  a run admits as one message (../tasks/messages.md, 2; ../engine/fleet.md); a
  `Create`'s spec is within the tasks child's too (../tasks/README.md,
  section 6). Words past either are refused at admission as too large,
  saying the bound.
- **A request refused** says why, with its facts: the role, the
  authority it lacked and by how much, and, where their role and pool
  decided it, whether a proposal could be accepted; a limit; the
  object's revision since; a task since ended
  (`../../client/domain/contract/README.md`, section 3). Who would accept a
  proposal is the offer's to say (`../../client/domain/contract/documents.md`, 3).
- **Proposing is explicit.** A `Create` or an `Amend` beyond the party's
  authority is refused, naming what it lacked and whether a proposal
  could be accepted; it never becomes a proposal by itself, so a person
  is never asked by accident, as a task's calls never are (../engine/executors.md,
  3). A service is a program, so the same holds for it. `Propose` asks,
  with a reason, and the party is its proposer: its decision, a
  rejection included, reaches them. A party proposes a batch, or an
  amendment, a widening of a task's authority among them, or a change
  of settings (../engine/settings.md, 6); an effect and
  a widening of a proposer's own authority are what tasks propose, since
  a party makes no effect, and their own authority is their role, which
  the policy sets (2; ../authority.md, section 9). Each document offers
  creating and amending as made, proposable and to whom, or refused, with
  bounds (`../../client/domain/contract/documents.md`, 3), so a client knows which
  before it sends anything; proposing implicitly is sugar above the
  plumbing, never the engine's.
- **A proposal needs someone who may accept it.** A party's proposal
  waits for the roles the project's policy names for its kind
  (../tasks/proposals.md). When the policy names none, `Propose` is refused
  at admission as lacking authority, saying that no proposal could be
  accepted; it is never committed as a proposal nobody may decide. A
  proposal of settings waits instead for the owners of its items, in
  the deployment and in the projects it names, and is refused so when
  there are none (../engine/settings.md, 6). A proposal already pending, and the proposals tasks make, wait as
  ../tasks/proposals.md says.
- **A party's root has someone to escalate to.** Every task a party's
  `Create` makes is a root, the party its requester. A party becomes a
  root's requester, by `Create`, by `Move`, or by accepting a proposal
  whose tasks are to be theirs, only when its escalations have a holder:
  their role decides escalations, or the project's policy names an
  escalation role (`README.md`, section 4). Otherwise the request is refused, saying
  so, as a proposal nobody may accept is: a held task never waits for
  nobody.
- **Where a party reaches.** A party's `Create` may name, as a
  dependency, a live task, or, as an input, an ended task's result: a
  task in their tree, or any task of the project for a maintainer, as a
  task's batch reaches its delegates and the tasks it was introduced to
  (../tasks/README.md, section 4, and ../tasks/messages.md, section 5); a batch gives its tasks no references.
  Their other operations reach what the table says.
- **What only parties do,** by design: carry out person tasks (`Take`,
  `HandBack`, and `Answer` with a result), stop runs, move tasks to
  themselves, mark what they have seen, make, run and delete projects
  (2), and act
  on the LLM accounts they hold (../accounts.md, section 5). A task places
  holds on the tasks below it too (../tasks/lifecycle.md, 10; ../engine/executors.md, 3), and
  only a party holds a project.
  Procedures decide nothing (../tasks/proposals.md).
- **A decision names what it saw.** A release, an amendment, a decision
  about an escalation and a person task's answer carry the revision the
  party was shown; one whose object has changed since is refused as
  moved, saying what it is now. A change of priority alone names none:
  the first commit decides.
- **A hold's revision moves with the hold:** when the task is held anew,
  or its reason changes; never when its escalation passes up. A release,
  and a decision about the escalation, name it, whatever held the task,
  the releasing party's own stop among them. Words release nothing: a
  `Message` adds words to the inbox, and words to a task the party
  stopped come with a `Release` in one compound request (below), naming
  the hold's revision as it would alone. Where the escalation waits is
  checked apart: deciding the escalation needs it to wait for the
  party, and is refused as not waiting for them otherwise; releasing,
  amending or cancelling the held task needs the task in their tree, or
  maintainer, wherever its escalation waits. So a release named with the
  revision from before a pass up is made, not refused as moved.
- **Two parties deciding one thing:** the first commit decides; the
  second is told it was decided, by whom, and how.
- **A compound request** holds several operations under one key, for a
  decision made of several: closing a conversation whose delegates are
  live, each cancelled or moved to the party, and the conversation then
  cancelled with a grace (`README.md`, section 6); ordering several goals at once;
  amending a held task and releasing it; words to a task the party
  stopped, and its release.
  - *What it may hold:* `Cancel`, with or without a grace; `Move`;
    `Amend`, priority included; `Release`; `Message`; `Stop`. The other
    operations keep requests of their own, `Hold` and `Lift` among
    them.
  - *One project:* every operation's task is in the request's project.
  - *In order:* its operations are admitted in order, each against the
    state those before it leave and needing what it would alone, with one
    pool counted across them (a move's reservation and the rest); a
    task's placed hold changes at most once in it (../tasks/lifecycle.md, 10).
  - *All or none:* they commit in one decision. The request has one key
    and one answer: all made, or refused, naming the first operation that
    failed and why, and nothing of it made.
  - *Numbered by place:* its operations take consecutive message
    numbers, one each, in the request's order, and the answer names the
    first, so the operation at place *i* took the first plus *i*: a
    `Message`'s words are numbered so, and an operation that adds nothing
    to its task's inbox leaves its number unused. Nothing more of an
    operation is answered (`../../client/domain/contract/requests.md`, 4).
  - *Revisions:* each operation names its revision where it would alone,
    so a `Release` names its hold's beside words or an amendment as it
    does alone; a change of priority alone names none, and the first
    commit decides.
  - *Bounded* by operations per request, a limit of this child's, never
    so small that closing a conversation with every delegate disposed
    of, or ordering every goal of a project, takes more than one
    request; configuration checks its floor
    (`../../protocol/limits.md`).
  - *Its worst case* is ../engine/README.md's, section 7.
- **Requests about an application's own objects** (an environment's end
  moved, an incident's page) are the application's: its client sends
  them, its root routes them to the connector they concern, and what they
  change in the core is an operation of the table above, judged as the
  party's own. This child keeps them by key as it keeps its own:
  admitted on the party's sign-in and, by the scope the request names,
  their role in its project, their role in the deployment, or nothing
  more for the party's own (`../../client/domain/contract/requests.md`, 3, and `../../client/domain/contract/README.md`, 5.5
  ); joined when sent twice; and their answer, the connector's
  outcome, kept under the key by token. One it refuses at admission
  (signed out, the party's role in the request's scope, a key sent again
  with another request, a limit) is answered with jig's refusal and never
  reaches its connector; only an admitted request is the connector's to
  decide, and answered with its outcome. A connector whose own bound on
  the answers it keeps is full refuses an admitted request at once, by
  the decision this child reserved for it before routing: a `Limit` of
  kind `ApplicationAnswers`, with the connector's number and its bound,
  which this child keeps under the key as its own refusals, so that the
  client is answered once and sends no more. Busy stays for what passes
  while the request waits, never for a table full until the retention
  ends. A connector's core part carries
  no model-name context: an explicit model selector (ALL or named,
  including empty) is refused at people admission with `Lack::Models`;
  an omitted selector keeps the operation's default.
- **A watch is not a request:** it decides nothing and keeps no answer.
  It is admitted on the party's sign-in and on what their role lets them
  see in its scope, the party's own on the sign-in alone, and ends when
  they leave (../engine/views.md).

## 2. The project itself

A project, its name, its resources and roles, its homes and its policy
are records in the store that owners change through requests like any
other. A rename reaches every client through `me`
(`../../client/domain/contract/documents.md`, 1) and changes nothing else (../model.md,
6.1). A policy change applies to every decision after it, the decisions
of tasks already made included: a narrowing makes their grants beyond it
unusable, and a widening does not widen them; a change of its spend per
period, or a role's, resizes the open period's budget, or its parties'
pools (../authority.md, section 7). What was committed before it is made,
and pending proposals are judged against it when accepted (../authority.md,
section 6, effective authority). An owner who wants work in flight
stopped cancels it. The policy's overrides of the application's
templates (../authority.md, section 6) reach only what clients create after
them: a task keeps the values it was made with
(`../../protocol/configuration.md`, 3.1).

Its owners change its policy part by part, or whole as an item of a
change of settings (../engine/settings.md, 2), so that one decision sets the
project up beside the deployment's settings: by one party who owns
both, where the change names the deployment's items too. Either way
the policy is the project's own record, applied as above, and is no
setting: it has no default, follows no revision and is never pinned;
the revision that set it records it whole and the policy it replaced,
so that restoring a version can set it back (../engine/settings.md, 10).

A project made after the first start (`MakeProject`, 1) is made in
the decision that admits the request, in one commit with its answer:

- **its number,** the deployment's next (../model.md, 6.1);
- **its name and its policy,** the configured policy as it stands then,
  copied: a later change to the configured policy, or its removal,
  reaches no project made from it, and its owners change it as any
  project's;
- **its roster,** its maker alone, as owner (`README.md`, section 4). Other owners
  of the deployment get no role in it; the maker gives roles, adopts
  resources whose connectors seed them, or makes services;
- **its first period,** its budget the policy's spend per period, with
  no next start, as the smallest period source opens a seeded project's
  (`../../protocol/configuration.md`, section 5), and with it the maker's
  pool. The application's root hears that the project was made, and a
  period source that keeps a calendar says its periods from then on, as
  for every project.

A crash keeps the project and its answer, or neither; a retry under the
key then makes it once. Two makes naming one name: the first commit
takes it, and the second is refused as *name taken*.

A project is deleted (`RemoveProject`) by its owners (1), or by the
deployment's act (3), whole: its decision closes it and keeps the
deletion, and the rest follows in kept steps, each in decisions of its
own, in this order.

Its conversations are closed as a person closes one, with a grace
(`README.md`, section 6), and the grace is the request's: a span within the tasks
child's bound (../tasks/README.md, section 6), or none. The engine reads no
template, so whoever asks gives it: a client the chat template's grace
as `me` offers it now, as it gives a close's
(`../../client/domain/client/helpers.md`, `shapes::remove_project`),
and an application deleting by the deployment's act its own chat
template's (3; `README.md`, section 6). A grace past the bound refuses the
request, a limit, as a cancel's is (1), before anything of the
deletion is made, so that none of the deletion's cancels is ever
refused for its grace.

- **Closed at once.** In the decision that admits the deletion, which
  answers it removed, the project takes nothing more: a task, words, an
  adoption, a change of its settings, policy or roles, a deployment's
  act on it, or any other request or watch on it is refused unknown, as
  on a project that never was. It leaves every list and every party's
  `me`, its roles are erased, and its name is free again from that
  decision; its number is never given again (../model.md, 6.1). A pending
  proposal of settings naming its policy is withdrawn in that decision,
  as no longer applying (../engine/settings.md, 6).
- **Its tasks ended, then erased.** Each live task of the project is
  cancelled by the deployment, with no reason, in the deployment's
  compound (../tasks/changes.md), as many compounds as the operations
  per request take, each in a decision of its own:
  - *a live conversation,* a root flagged conversation, as the
    deployment's `Close` closes one (3): each of its live delegates
    cancelled, then the conversation cancelled with the deletion's
    grace, which asks it to finish and cancels it if it has not ended
    when the grace ends (../tasks/changes.md), its message number a
    fresh one, as a close's is; with no grace, it is cancelled at once.
    A conversation and its delegates go in one compound, never split
    across two, as a close fits in one request (1, operations per
    request);
  - *every other live root,* with its tree, with no grace;
  - *a root already closing, or finishing under a grace,* a close's or
    the deletion's own, is not asked again: it ends when its grace does,
    unless it ended first.

  Once none is live, each of its tasks is erased whole, one at a time
  and with no horizon waited for, as retention would erase it at the
  horizon: its record, its rows in every index, its history, activity
  and terminals, its read positions and its transcript's file
  (../tasks/README.md, section 3; `../../protocol/store/transcripts.md`, 5, and `../../protocol/store/README.md`, 6.1). The
  decision that erases each one tells every connector so
  (`../connectors.md`, 3.4).
- **Then its own rows:** its policy's copy and its overrides of
  templates; its placed holds (../tasks/lifecycle.md, 10); its pools in the current
  period, and its period, closed into the history with their spend
  (below); its notes (../engine/briefs.md, section 2); the answers kept under
  parties' and connectors' keys that named it (1, 3); and the
  settings of its scope (../engine/settings.md, 5).
- **Its adoptions released:** each connector is asked to release the
  project, drops what it keeps of the project's adoptions and of its
  own kinds' settings in the project's scope, and answers once that
  has committed; the deletion ends when every connector has answered.
- **What outlives it:** its spend, in the deployment's and its parties'
  totals for the period, as closed periods and pools keep it, never per
  task; and an account whose only use was the project keeps that use,
  so that it pays for no one rather than for every payer (../accounts.md,
  4.2), while an account with other uses drops it. An act's or a
  request's answer that named the project is erased with it, so that,
  sent again, it is judged afresh and answered unknown, as one past its
  retention is. Each party's inbox and read positions drop what named
  its tasks.
- **Kept until whole:** the deletion is a row of the people child's,
  the project, the step it has reached (its tasks, its own rows, its
  adoptions released) and its grace, saved with each step's decision
  and erased with the last, at most one per project the deployment has
  room for. At each start, once the core is ready, each kept deletion
  goes on from its step with its grace, its project closed from the
  start as from its decision, so a deletion a restart cut ends whole
  and no start serves a half-deleted project; a grace past a bound
  lowered since is given as that bound. Its own answer is kept while it
  lasts: sent again under its key, the deletion is answered removed
  until it ends, and unknown after.

## 3. The deployment's acts

An application sometimes acts where no client is there to ask: at the
end of what a person's conversations were about, at a start that a
restart cut short, at the reset of what it made for a person (temper's
playground: temper's `protocol/setup/playground.md`). A connector then asks
the core for one of a few acts in the deployment's name, as the
deployment requests the tasks its configuration or a connector starts
(../tasks/README.md, section 3). An act has no party, no sign-in and no role: the
configuration's grant and the act's own subject are its whole check.

- **Who may:** a connector the configuration grants that kind of act
  (`../../protocol/configuration.md`, section 3), while the configuration
  names a first owner (`README.md`, section 3): the deployment's acts are its owners'
  decisions, made for them by a connector they configured. An act of a
  kind the configuration does not grant its connector is refused *not
  offered*; any act, while the configuration names no first owner, is
  refused as lacking the role, none held. Neither refusal is saved, so
  a later start's configuration decides again.
- **What it may,** each act on its own subject alone:

  | Act | Its subject | Becomes |
  |---|---|---|
  | `Close` | a live conversation of the named project (../tasks/README.md, section 3) whose requester is a person; a grace within the tasks child's bound, or none; a reason within a cancel's | in one decision, the deployment's compound (../tasks/changes.md), `README.md`, section 6's close with every delegate cancelled: each live task the conversation requested cancelled, then the conversation cancelled with the grace, each with the reason; answered as a compound request (1) |
  | `AddKey` | an LLM provider whose sources offer a key; the key, carried as a request carries one (../accounts.md, section 5); one of the deployment's accounts to join, or a new one | the key added, the deployment's (../accounts.md, 3.2 and section 5) |
  | `RemoveAccount` | an account the deployment holds | the account removed (../accounts.md, section 5) |
  | `Adopt` | one of the acting connector's own resources, as an owner's adoption names one, into a live project | adopted as an owner's adoption is, the connector asked and answering as it does a party's (../connectors.md, section 12), the project's roles seeded from the parties the connector knows there, and no party made its owner |
  | `MakeProject` | as `MakeProject`'s (1), and an owner of the deployment, to be its maker | the project made (2), that owner its maker and only party |
  | `RemoveProject` | a live project, not being deleted; a grace within the tasks child's bound, or none | the project deleted as its owners' `RemoveProject` deletes it (2), each of its live conversations asked to finish within the grace; answered removed once closed |

  A subject outside its act's is refused, naming it: a task that is not
  a live conversation of a person, an account the deployment does not
  hold, another connector's resource, a maker who is no owner of the
  deployment; a grace past the tasks child's bound is refused as a
  cancel's is, a limit.
- **Keyed as a party's request** (1): the connector names a key, in
  its own key space, apart from every party's and every other
  connector's, and its answer is kept beside parties' answers
  (`../../protocol/store/README.md`, 4.4), with the same retention, room, joins
  and conflicts. The same act under its key joins it while it is
  pending, and once decided is answered from its saved answer, made
  once; another act under the key is a key conflict. A saved answer is
  given again whatever the configuration grants since, as a party's is
  whatever their role is now, and a restart restores it whatever the
  configuration names, until the retention ends it. Busy and not ready
  are not saved, and the connector sends the act again.
- **Owned memory:** each retained act charges the larger of the close's
  reason (`words`), the adoption's whole owned payload (`amendment_bytes`,
  including its slice headers), and the project name (`NAME_BYTES`), plus
  the larger of the configured host-failure list and a refusal's 64 task
  numbers. No act carries party model names or a sign-in address. Grants
  charge the configured entries, one per connector, each with at most one
  entry per act kind (`size_of::<ActKind>()` times the number of kinds).
  The people constructor checks these configured bounds.
- **Answered to its connector once durable** (../engine/README.md, 5.2), through
  the root, with the outcome a party's request of its kind gets; each
  send, a joined one among them, is answered once.
- **Across a restart:** a connector keeps in its own records each act it
  has still to hear answered, under its key, committed with the
  decision that first asks for it, and sends each again at its start,
  once the core is ready, until it hears the answer. An act made before
  the restart is answered from its saved answer; an act the restart lost
  is made then. So a restart between an act's keep and its answer makes
  it once and answers it once. Past the retention it is judged afresh:
  a conversation closed since is refused as ended, an account removed
  since as unknown, a project deleted since as unknown, once its
  deletion has ended and its answer with it (2), which its connector
  takes as done.
- **The deployment, not a party:** an act makes no party, sign-in,
  inbox or role, and is in no `me`. A task it cancels is cancelled by
  the deployment, with its reason, and a project it deletes is deleted
  as its owners' deletion is. The value an `AddKey` carries is
  kept by the application first, as a client's request's is
  (`../../protocol/client/README.md`, section 3), and one the act leaves unused
  is said unused before its answer, for the application to erase. A
  project it makes has a party as its owner, the deployment's owner the
  act names.
