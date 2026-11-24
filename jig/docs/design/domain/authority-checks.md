# Authority: checks

## 1. Checks

Every check answers one of four, each saying why:

- **allow;**
- **wait** for verdicts a connector cannot give yet (a health it has yet
  to read, a check still running), which nobody clears by deciding;
- **propose:** beyond the task's authority, within what someone above it
  may accept;
- **refuse:** beyond the project's policy or the deployment's rules, or
  against a requirement a connector refused, which no acceptance
  changes.

A check answers the strictest of what it found.

### 1.1 Batches

Before a batch commits (tasks/README.md, section 4): each task's authority fits
under its creator (`authority.md`, section 5), the creator's delegation allows each
task's executor kind and its charter, procedure, role or job kind, each agent task's LLM choice names
models its own authority allows and efforts its models declare
(tasks/README.md, section 3), and the batch's tasks, depth, budgets and
deadlines are within what its creator has left. Beyond any of these, the
answer is propose; beyond the project's ceiling, refuse. For now, an LLM
choice naming a model beyond its task's authority is refused, naming
the model, rather than answered propose: a proposal that widens a
task's models is later work. An account a task's LLM choice names that
does not serve its payer is refused, naming the LLM provider's rule
where it is a plan's (accounts-uses.md, 1.2).
Allowed, the answer gives the creator's numbers after the reservations,
and what the budget given `AsAllowed`, if any, resolved to (`authority-budgets.md`, section 1).

- **One check for every creator.** An agent's `delegate`, a procedure's
  step, a party's `Create` or `Delegate` and an accepted proposal's batch reach this
  check alike. Only the source differs: a task's authority and what it
  has left; or a party's role's authority and what their pool has left
  (1.4); or, for an acceptance, the accepter's (`authority.md`, section 9).
- **Propose is an answer, not a proposal.** A batch answered propose is
  refused, naming what it lacked and whether a proposal could be
  accepted; who would accept it is the offer's to say
  (`../client/domain/contract/documents.md`, 3). It becomes a proposal only when
  its creator asks for one (`authority.md`, section 9).

### 1.2 Effects

When a decision asks for an effect, whether an agent's tool, a
procedure's step or an accepted proposal, the core checks the effect's
description, which its connector gives (connectors-effects.md, 1.1):

- a grant of the task covers its kind and its resources, or the answer
  is propose;
- the project's ceiling and the deployment's rules allow it, and its
  resources' roles admit it, or the answer is refuse;
- its price, if it has one, is within what the task has left, or the
  answer is propose, as a widening of its budget;
- the verdicts its requirements need (`authority.md`, section 10) were given for the
  exact state the effect names, observed ones within their freshness:
  wait while one is wait, refuse once one is refuse;
- every requirement the policy marks as one that must be guarded is
  guarded by the effect's kind, or the answer is refuse.

The effect carries its condition to the system where the system can
check it: a rollback made only if the service is still at the version
decided on, a merge made at exactly the head decided on, so the guarded
verdicts given for that state stay true while it is made. Observed
verdicts promise only what their judges saw when the effect was decided.
Every check is against effective authority (`authority.md`, section 6).

### 1.3 Runs

Before a run is claimed and again as it starts: its deadline is not
past; the run's spend, its task's share for a run (`authority-budgets.md`, section 1: what the
task has left, half of it where the task may start a delegate and that
half is above the deployment's minimum for a run, capped per run by the
deployment), is above the deployment's minimum for a run and holds its
reserve and one more of its main model's most costly completions, and
on a program backend one more again (`authority-budgets.md`, section 1); the accounts its check
chooses each have a source usable on a host with room (accounts-uses.md,
1.3), from runtime accounts and their restored source rows, never an
endpoint account; and the application's credentials its workspace needs
are usable (engine/README.md, 6.1); a host the charter allows has room, declares its
task's models, and may take its payer's runs (engine/fleet.md);
and every resource its workspace writes, saved state
included, is covered by a grant of the task and held by the task or by
the task above it that holds it (tasks/holds.md). A run that fails
this check is not started: its task waits for its budget (held, if
nothing will free one), its account or credential, its host, or its
writer slot.

Allowed, the answer is the run's budget: the turns, time and reserve its
charter gives, and that spend, so the core places no run the agent would
refuse for its budget; and the accounts and sources chosen, which the
claim's commit records (engine/executors.md, 1). Each charter's turns and time
above its reserve's are configuration's to check
(`../protocol/limits.md`); the agent's own limits are
the fleet's, at each host's hello: the endpoints it declares, their window
and output bounds, the most turns, spend, time and waiting a run may have, and
the reserve's time against the checks' timeout (engine/fleet.md). A
start the agent refuses all the same fails its attempt as an assignment
its host found invalid, naming why (tasks/lifecycle-holds.md, 1).

The run-specific answer retains the exact provider/account/source evidence and
checked spend. Account checks include source usability and host eligibility;
application credentials are an independent bounded set. Missing main evidence,
an unusable source, an ineligible host or an unavailable application credential
waits. Duplicate providers and either set past its bound refuse before allowing.
The shared verdict used by other checks carries no run payload.

### 1.4 Parties' requests

Before a party's request is acted on (people/acts.md): their role
in the project allows the request and, for what gives authority (a task
created, a budget allotted, a proposal accepted, an amendment that
widens), what it gives fits under their role's authority and what their
pool has left.

- **A party's batch is 1.1's,** with the party as its creator: their
  role's authority in place of a task's, fitted as a delegate fits under
  its creator (`authority.md`, section 5), its delegation's kinds naming the executors
  and charters they may create with, and what their pool has left in
  place of what a task has left. A budget given `AsAllowed` is resolved
  here, against their pool and capped by their role's spend
  (`authority-budgets.md`, section 1).
- **A party's batch beneath a task** (`Delegate`, people/acts.md, 1) is
  1.1's twice. With the party as its creator, each task fits under
  their role's authority, its delegation's kinds naming each task's
  executor, as a `Create`'s does. With the task it is made beneath as
  its creator, it is within what that task has left (its delegation's
  tasks and depth, its budget, its deadline), as that task's own batch
  would be. Its budgets are carved from the task, never from the
  party's pool, and a budget given `AsAllowed` resolves against what
  the task has left, capped by their role's spend. Beyond either, it is
  refused, naming what it lacked, and makes no claim of a proposal:
  proposing one is later (tasks/gatherings.md, section 7).
- **A party's root has someone to escalate to.** A party becomes a
  root's requester, by a batch, a move or an acceptance, only while its
  escalations have a holder (people/acts.md, 1); otherwise the request is
  refused, whatever authority it fits.
- **A task amendment's offered task allowance** is a replacement lifetime
  limit under the giver's role, not the target's remaining allowance.
  Exhaustion does not prevent granting more tasks. The request still
  passes the same funding and authority-ceiling admission checks.
- **A party's widening is judged as `authority-budgets.md`, section 1 funds it.** An amendment
  that widens is judged whole, by the authority it leaves the task: at
  most their role's authority, its budget no more than their role's
  spend, however much of it the task holds already; and what it needs
  from them (`authority-budgets.md`, section 1, *Whoever widens a task funds it*) no more than
  their pool has left, an unopened pool read at what it would open
  with. Beyond their role or their pool it is beyond authority and may
  be proposed, since an accepter funds a proposal (`authority.md`, section 9); beyond
  the policy's ceiling or the deployment's rules it is refused. A
  task's document offers the amendment by this judgement's own numbers
  (`../client/domain/contract/documents.md`, 3), so a client proposes
  only what the request sent directly would be answered as beyond.
- **A compound request** is checked as people/acts.md, 1 says: each
  operation as it would be alone, in order.
- **Account requests** (accounts.md, section 5) are checked against the
  account's holder, not a role: a person's own accounts by that person,
  the deployment's by its first owners (accounts.md, 3.2). A plan's
  account's uses naming another person, or a project with another party,
  are refused, naming the LLM provider's rule (accounts-uses.md, 1.2). They
  give no authority and spend nothing.

- **Making a project** (people/acts.md, 1) is checked against the party's
  role in the deployment, owner, never a project's. It gives no
  authority and spends nothing past its policy's own spend per period.
  A configured policy is checked against the deployment's rules at
  every start (`../protocol/configuration-engine.md`, section 1), so the
  request never finds it above them.

### 1.5 Tool calls and reads

Before an agent's call is served: its family is among the task's tools,
and a read it makes is covered by a read grant on the resource it names.
A subscription to a connector's topic is checked as a read: `subscribe`
among the task's tools, and a read grant covering the resources the
connector says the topic concerns (connectors-facts.md, 1.3; hosts/charters.md
). A message to a task needs a reference to it (tasks/messages.md, 5);
a note needs its scope.

On a program backend, each built-in tool the run is granted has an ask
rule in the run's settings, so each call of the program's own tools
reaches smith's policy through the program's permission prompt before
it runs. The prompt's tool answers *allow*, with the call's input
unchanged, or *deny*, with a message, and the agent never changes a
call's input: smith decides the call as it decides one of its own
tools, within the run's tool families, and the host sees the call that
runs (smith's `protocol/programs-claude-code.md`, sections 1.3 and 1.4; hosts/charters.md
). So no call runs unseen by its host, and every call's
effects and reads pass these checks on either backend.
