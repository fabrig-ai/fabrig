# Connectors: effects and the outbox

## 1. Effects and the outbox

### 1.1 Effects

An effect is a typed request the connector defines, in its own terms,
which it keeps. What the core gets is its **description:**

- **its kind and its resources,** for grants and roles;
- **its purpose,** from which its key is derived (1.4);
- **its condition,** where it has one: the state it was decided against
  that the system can check as it applies it (a rollback only from the
  version decided on, a merge at exactly a head);
- **its form:** a **creation** (an environment, a pull request, a
  comment), a **transition** (a rollback, a merge, a scale to a size), or
  a **set** (a body, a label, a silence's end);
- **its recovery class,** which its kind declares (1.3);
- **whether it waits for dependencies,** which its kind declares: one
  that does (a merge, a deploy, a release) is what a task kept by a
  pending dependency must not make, and is refused while it is, naming
  them (tasks/lifecycle-endings.md, 1); one that does not (a comment, a status, a draft)
  is made as ever;
- **its price,** if its kind is priced: its maximum, in the deployment's
  unit (authority-budgets.md, section 1);
- **the state its requirements are judged at:** what the judges are
  asked about (`connectors.md`, section 7).

### 1.2 The outbox

- **An entry per effect,** committed with the decision that asks for it.
  Its connector holds the entries not yet made, which the store keeps
  too, as the connector's records.
- **Made after the commit,** when the journal releases the request to
  make it: in commit order per resource and one at a time on each, in
  parallel across resources within the connector's limits and request
  budget.
- **Each attempt is committed before it is sent,** with its absolute
  retry deadline (1.3), so no restart forgets when an attempt went out.
- **Its outcome is committed in turn:** made, with what it made; failed,
  typed; or uncertain. A task closing waits until every entry it asked
  for has settled (tasks/lifecycle.md, 1). The core asks each connector to
  settle a closing task's entries and waits for its word that it has,
  connector by connector; how many entries, and which, are the
  connector's to know and to show, never counted by the core.
- **Retries are the connector's:** a transient failure is tried again
  after a backoff, within its budget; a failure for good (forbidden, a
  conflict, the object gone) holds the task that asked, saying so
  (tasks/lifecycle-holds.md, 1); a procedure's task instead hears it and steps (`connectors-procedures.md`, 1.2),
  holding itself if it needs a decision (`connectors-procedures.md`, 1.3). A projection's or a
  release's failure for good is news for the goal, or the closing task,
  it was made for.
- **Withdrawn** when the task that asked for an entry is cancelled before
  the entry went out. An entry that went out is never withdrawn: what may
  have been made counts.

### 1.3 Uncertainty and recovery classes

- **Only what provably never reached the system did nothing.** Any other
  failure (a timeout, a reset after sending, an answer that does not
  decode) is uncertain: the write may still take effect, even after the
  connector has tried again.
- **Each kind of effect declares its recovery class,** from what its
  system offers, and the connector's world shows the class holds
  (`connectors.md`, section 15):
  - *keyed:* the system keeps the effect's key, or an operation's id,
    and refuses or returns a second request with the same one. An
    uncertain entry is looked for by its key, and tried again with the
    same key if not found; a late copy finds the first. **At most
    once.**
  - *conditional:* the system applies the effect only from the state it
    was decided against (a rollback only from the version decided on, a
    merge only at the head decided on), and the state it leads to can be
    told from any other's, from what the system keeps, which no other
    hand can erase (a pull request closed is still found; a branch
    deleted is not). An uncertain entry is looked for by that
    state, and tried again under the same condition; a late copy fails
    its condition. **At most once in effect.**
  - *idempotent:* a set, which may be applied twice to the same final
    state. It is written again. Where the system offers a version to
    check, the set carries it, so a late copy cannot land over a newer
    write; where it does not, the kind says a late copy may. A transition
    is idempotent only if every copy of it, in any order, leaves true the
    one state its kind names, or changes nothing: it is looked for by
    that state and made again if not found, and the kind says what else a
    copy may move. **The same final state, possibly more than once.**
  - *unrecoverable:* none of the above, such as a restart through an
    interface that gives no operation's id. An uncertain entry is never
    tried again by the engine: its task is held, saying so, for a person
    to decide whether to make it again (tasks/lifecycle-holds.md, 1). **At most once,
    by holding.**
- **Retry deadlines are absolute.** Each attempt to make an entry is
  committed, with its deadline (the wall time it is sent, plus the
  system's lifetime for a write, plus a margin for the clock), before it
  is sent. An uncertain entry that was not found is tried again once its
  deadline has passed, and only by committing a new attempt. A restart
  keeps the deadlines it loads, rather than counting a lifetime from its
  own start, so restarting never postpones an entry for ever. A clock
  that moved back only delays a retry; the margin covers a clock that
  moved forward by as much as the deployment configures.
- **A copy lands within its write's lifetime, or never, except for an
  idempotent transition.** The other classes' promises rest on it: an
  attempt is retried only once the lifetime of the one before has passed,
  so no copy of an earlier attempt lands after a later one is sent. A fake
  system that delivers copies later breaks this assumption, not the
  connector, unless the kind is an idempotent transition. A system whose
  writes cannot be bounded so has no class but unrecoverable, or an
  idempotent transition:
  no copy can undo the state it names, so its promise alone rests on no
  lifetime, and its world shows a copy landing past one.

### 1.4 Keys

- **Committed with the entry,** and derived from what the effect is for,
  never from when it was asked, so asking again for the same thing finds
  the same key:
  - an agent's effect: its task, its attempt and the call that asked
    (engine/executors-tools.md, 1);
  - a procedure's: its task and the purpose within it (provisioning its
    environment; rolling back to a version; merging at a head);
  - a projection's: its goal and what it writes;
  - a release's: the closing task and the resource.
- **The deployment's,** too: every key carries the deployment's
  identity, so two deployments on one system, or a store made anew, never
  find each other's objects. A key the connector writes itself (a marker
  in a body, an operation's id) carries the deployment's id. A key that
  is a name the system keeps (a branch, an environment's name) carries
  the deployment's prefix (`connectors.md`, section 12), which is the deployment's alone
  on that system, so the name stays readable and the grants that name it
  by pattern (authority.md, section 4) stay as they are.
- **Carried on what it creates,** where the system keeps it, so that a
  creation can be found by its key alone. The connector records every
  object it made, by key, in the store: the ownership of `connectors.md`, section 11.

### 1.5 Effects asked by agents

An agent asks for an effect with its `effect` tool (engine/executors-tools.md, 1),
naming one of the kinds the connector offers to agents. Its input
reaches the root decoded, as the connector's typed request; the
connector describes it; the core checks it (authority-checks.md, 1.2); and, in
the same decision, the connector keeps it under its key, or under the
proposal's number if it was proposed, or drops it. A check that answers
wait commits nothing: the call is answered so, and the run may ask again
or delegate to a procedure that waits. What the connector's procedures
own (provisioning, remediation, a change's way to landing) is not
offered to agents: a run that wants it done delegates it to the
procedure.

### 1.6 Releases

- **A task closes after its effects settle.** `task_effects_settled`
  requires every pending effect of the task to settle before the
  connector reports it closed. The ops world holds this under load in
  `jig-ops-world`,
  `connector_checks::infrastructure_orders::a_task_closes_after_its_effects_settle_under_load`.

When a task that held resources closes, its connector releases them
with effects of its own, after every other effect of the task has
settled (tasks/holds.md, 5): per kind of resource and per way the task
ended, what becomes of it. An environment a task provisioned is torn
down; a branch a landed change was pushed to is deleted; a failed one's
is kept for a person to read until its tree's root closes. A release's
effects are keyed by the task and the resource, and its failure for good
is news for the closing task.

Infrastructure's closing table is bounded by its configured tasks plus
procedures plus effects. This is independent of the refresh table's
bound: a restart can have more fresh reads under way than closing tasks.

### 1.7 Takings

- **A taking may have effects,** as a release does. When a task takes a
  write hold (tasks/holds.md, 2), the core tells the hold's connector, with a
  numbered outbox entry of the taking's own; per kind of resource, the
  connector says what taking it does: none, for most; a forge's goal
  branch is made at its base's tip.
- **Keyed by the task and the resource,** in the outbox as any effect
  (1.2), taking the resource's writer slot while in flight (tasks/holds.md,
  3). The connector orders its own later uses of the resource after
  them: a write into a branch still being made waits for it.
- **A taking's effect failed for good holds the task** (tasks/lifecycle-holds.md, 1),
  its words the connector's, saying why: the resource it could not make
  and what its system answered. The task keeps its holds, and its
  requester decides, as for any effect that failed for good.
