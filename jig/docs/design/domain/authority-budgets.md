# Authority: budgets and spend

## 1. Budgets and spend

- **Four numbers per funder:** its budget (what it was given), what it
  has spent itself, what the ended tasks it funded spent, and what it has
  reserved for the live tasks it funds. What it has left is its budget
  less the other three, and never below zero. A task is a funder; so is
  each party's pool and each project's spend, per period, whose numbers
  the tasks child domain keeps too, in the store (engine/decisions.md, 1.4).
- **Every reservation records its funder:** the task above, a party's
  pool, or a project's period. A batch reserves each delegate's budget
  from its creator in the commit that makes it; an accepted proposal
  reserves from its accepter.
- **`AsAllowed`.** A batch may give a task's budget as `AsAllowed`: as
  much as fits under its creator when the batch is admitted. This child
  resolves it as it checks the batch (`authority-checks.md`, 1.1), since what a pool has left is
  a live number no client can read without racing:
  - *for a party,* what their pool has left, capped by their role's
    spend;
  - *for a task,* what it has left;
  - *never more* than the cap the batch names with it, if it names one.

  At most one task of a batch gives `AsAllowed`, so what is left is never
  shared out by order: a batch where two do is refused as not well
  formed (tasks/README.md, section 4). It is resolved after the batch's named
  amounts are counted, from what they leave. The answer says what was
  given, and the task keeps that amount, as it would one given by hand.
  One that would give nothing is refused as lacking budget, saying what
  is left. A proposal names amounts, since what it needs is known before
  anyone covers it: a proposed batch giving `AsAllowed` is refused as
  not well formed (`authority.md`, section 9).
- **A deadline given as a span.** A batch may give a task's deadline as
  a span: that long after the decision that makes the task, a batch's
  admission or the acceptance of the proposal that asks for it. This
  child makes it a wall from that decision's wall time as it checks the
  batch (`authority-checks.md`, 1.1), before the deadline is checked against the creator's,
  and the task keeps the wall, as it would one given by hand. A
  proposal keeps the span until it is accepted. So a request names no
  clock of its sender's, and is the same ask each time it is sent
  (`../client/domain/contract/requests.md`, 1).

  Every check that judges a span against a wall, a bound's or a
  holder's deadline, judges the wall it would make at that check's own
  decision: the decision's wall time plus the span. A batch is judged
  so at its admission; a proposal at its admission (the ceiling, whether
  it could be accepted, where it waits), each time where it waits is
  decided again, and at its acceptance, by the accepter's authority,
  with the wall the task then keeps; and the acts a document offers, as
  the engine judges them. So a waiting proposal's span ages: one that
  fit its holder when proposed may not when accepted, and that
  acceptance is answered as any beyond its accepter's authority (`authority-checks.md`, 1.1),
  for the holder to pass the proposal up (tasks/proposals.md), or
  refused past the project's ceiling.

  A delegate gives its deadline as a run's call can: a span, or none,
  and then it ends with its creator's, as no delegate's deadline is
  later than its creator's (`authority.md`, section 5).
- **When a task ends,** in the commit that ends it, its reservation is
  released: its unspent budget goes back to its funder, and what it and
  the tasks it funded spent is added to its funder's "spent by ended
  tasks". Spend climbs the chain one end at a time, to the pool or period
  at the top, and is never counted twice nor lost.
- **A task moved** to a new requester (tasks/changes.md) is funded
  anew: its old funder is given back what the task has left and keeps
  what it spent; the new funder reserves what the task has left from its
  own.
- **Whoever widens a task funds it.** An amendment that widens a task's
  authority (`authority.md`, section 5) is funded by whoever gives the widening: its
  amender, when it is within the amender's own authority, or its
  accepter, when it was proposed (`authority.md`, section 9). From that commit, the
  giver is the task's funder:
  - *when it already was,* its reservation is resized to the task's new
    budget;
  - *otherwise,* the task is funded anew from it, as a moved task is:
    its old funder is given back what the task had left and keeps what
    it spent, and the giver reserves the task's new budget less what the
    task spent. Its requester, its tree and its delegates' funding stay
    as they were.

  An amendment that only narrows a task's authority, or changes none,
  leaves its funder as it was, a lower budget giving the difference back
  to it. So a task has one funder, and a widening is never split between
  two.

  A party gives from their pool in the project's open period, which the
  widening's decision opens where it is not open yet, as an acceptance's
  does; a task gives from its own numbers. So a party's widening of a
  task funded from an earlier period's pool, their own included, funds
  it anew from the open one, and a task's widening of a delegate another
  funds makes the amending task its funder. Every amendment names its
  giver so: a party's, alone or in a compound request (people/acts.md,
  1), a run's and a procedure step's. What a widening needs from its
  giver is what it reserves: the new budget less the task's budget where
  the giver already funds it, and otherwise the new budget less what the
  task and the ended tasks it funded spent. A party's widening is judged
  by that need (`authority-checks.md`, 1.4).
- **No overrun is recorded.** Every charge is reserved before it is
  made, or, for a program backend's completions, held in its run's
  reserve (below), so what is recorded against a task never passes its
  budget. A task left with too little for its next run or effect waits,
  or is held for its budget (tasks/lifecycle-holds.md, 1).
- **Periods.** A project's spend and a party's allotment are per period.
  A reservation counts in the period that funded it, and what it returns
  goes back to that period, so a reset frees nothing reserved. A
  recurring task's budget is carved again each period (tasks/README.md,
  section 5).
- **A zero-budget task may outlive its ledger.** A pool's, a recurring
  task's or a period's ledger closes once nothing is reserved on it and
  its project has a newer period (`../protocol/store/README.md`, 10.1).
  A task whose budget is 0 reserves nothing, so the ledger funding it
  may close while it lives; it alone can, since any budget keeps its
  ledger's reservation above 0. Its priced work is charged to its
  project's current period. A restart accepts it, and its end releases
  nothing and posts nothing to the closed ledger: what it and the tasks
  it funded spent, if anything, is added to its project's current
  period's "spent by ended tasks", the newest live period. No ledger is
  kept open for it, and it is never moved to another funder.
- **A change of spend per period resizes what it opened,** in the
  decision that commits it: the project's spend its open period, and a
  role's (`RoleSpend`, its spend alone, or a whole role's edit that
  changes it) the open pool of each party holding it, one set by hand
  among them (people/acts.md, 1). Each budget
  becomes the new spend, or what it has reserved and spent when that is
  more: a raise funds the next root task at once, and a cut takes
  nothing back. A pool reserves its budget from the project's period,
  so a role's raise that the open period has no room for is refused
  whole, naming what the period has left: its owners raise the
  project's first. A project's spend below a role's is refused as well,
  so a cut goes the other way, the role's first. Its writes are a pool per party in the role, within
  the project's parties (engine/README.md, section 7), all checked before
  any is made.
- **Periods are the application's to say.** The engine keeps no
  calendar: a period begins when the application's root says so
  (engine/vocabulary.md, 1.1), and the root may say when the next begins, if its
  source knows (a deployment configured with a monthly period does).
  The core keeps that with the period's record, and a project's
  document shows it (`../client/domain/contract/documents.md`, 1).
- **Standing work renews by period.** A standing task, recurring, a
  standing procedure (connectors-procedures.md, 1.6) or an agent task made standing
  (tasks/standing.md, section 4), has its tasks and its spend
  carved again from the project's spend each period, so a watch that
  runs for months never exhausts its allotment of tasks. Its delegates
  keep their reservations in the period that funded them, and return to
  it.
- **What is spent:**
  - *LLM completions,* priced as they are spent, on either backend. A
    run's assignment carries its models' prices (engine/executors.md, 1), from
    configuration, in the deployment's unit: input, cached input, cache
    writes when priced apart (at the input's rate otherwise), and
    output. A run prices each completion it makes, its sub-agents'
    included, and reports what it spent with each turn it commits
    (engine/executors-transcripts.md, 1), and the whole with its answer. The core charges
    each turn as it commits it.
  - *Priced effects.* A connector may price a kind of effect, in the
    deployment's unit: an environment for a week, a scale-up for a day.
    The price is the effect's maximum cost, part of its description when
    it is decided, checked against the task's budget as part of the
    effect's check (`authority-checks.md`, 1.2), and charged in the commit that decides it.
    What the connector later finds it cost below that is returned when
    it says so; it never charges more.
- **A run's budget** has turns, spend and time, and a reserve of turns
  and time for finishing: its turns, time and reserve its charter's, its
  spend its task's share for a run (below), reserved from what its task
  has left in the commit that claims it. The agent enforces it exactly
  (smith's `domain/budget.md`; hosts/charters.md): it reserves
  each completion's maximum cost before making it, winds down on its
  reserve when the rest is spent, and ends for its budget a completion
  that does not fit what is left at all. On a program backend, which
  makes its own completions so that the agent cannot reserve one before
  it is made (smith's `protocol/programs.md`, section 7), the program
  may take one turn past where the run should wind down, at most one
  completion of its main model, and that turn is spent within the one
  more its reserve holds (below). So a run never spends past its budget
  on either backend, and stops a little early rather than late.
- **A run's spend holds its reserve and one completion more.** The
  reserve's spend is its turns, each at the main model's most costly
  completion: a window of input at its highest input rate, plus its
  output at the output rate. The core places a run only if its spend
  holds the reserve's and one more such completion, and on a program
  backend one more again, for the turn the program may take past the
  wind-down, so the agent never refuses it for its budget; a task whose
  budget left cannot fund that waits, or is held for its budget (`authority-checks.md`, 1.3).
  The source its check chose says which backend the run has
  (accounts-sources.md, 1).
- **A run holds back at most half of what its task has left, where half
  funds a run.** A run's spend is its task's share for a run: what the
  task has left, and, in a task whose authority may start a delegate
  (its delegation names a kind, and a task and a depth above zero),
  half of what it has left, rounded down, where that half is above the
  deployment's `minimum_run_spend`; either way at most the deployment's
  `maximum_run_spend`. The other half stays with the task while the run
  is live, so the delegates it makes and the changes an application
  makes for it are funded from it: a batch carves them from what the
  run's reservation leaves (`authority-checks.md`, 1.1), as before. A task that can start no
  delegate (a person's task in no project, `authority.md`, 6.1; a delegate whose
  creator gave it no delegation) has nothing to keep that half for, and
  its run takes what it has left, capped.
- **Where half is too little, a run takes what is left.** No run is
  placed whose spend is at or below the deployment's minimum (`authority-checks.md`, 1.3), so
  a delegating task whose half is there would wait, or be held for its
  budget, though what it has left would fund a run: under temper's stock
  figures, where a run's minimum is half its cap
  (`temper/docs/design/protocol/configuration/stock.md`), a goal funded
  at most the cap would never run. Its share is what it has left,
  capped, as in a task that can start no delegate. Its run then keeps
  back for its delegates only what the cap leaves, and a batch it makes
  is carved from that, refused as lacking budget where it does not fit
  (`authority-checks.md`, 1.1), until the run ends and gives back what it did not spend. With a
  minimum of 50 and a cap of 100, a delegating task's share is 50 with
  50 left (no run), 99 with 99, 100 with 101, 51 with 102 and 100 with
  300: the share falls where half first passes the minimum, since above
  it the half rule holds unchanged.
- **The share is one function's.** It reads the task's authority as it
  stands, not how many tasks it may still make. Since the share is
  reserved like any run's spend, from what its task has left and never
  more, recorded spend still never passes a budget (`authority.md`, section 1), and what
  the run leaves unspent goes back to its task when it ends. A task
  whose share is too little for a run, at or below the deployment's
  minimum or short of the reserve and the completions above, waits, or
  is held for its budget, as one with too little left (`authority-checks.md`, 1.3;
  tasks/lifecycle-holds.md, 1). A delegating task whose half is above the
  minimum but short of the reserve keeps the half rule, and so waits or
  is held even where what it has left would fund the run (`authority.md`, section 12).
  One function of this child,
  `run_spend(rules, authority, numbers)` beside `left`, computes the
  share, and every run check the core makes asks it: a claim's, a
  start's, a due agent's readiness and a write a run's preparation
  names.
- **Spend lost with a worker.** A turn's spend is recorded when the turn
  commits. A worker that dies loses the turns it had not had
  acknowledged, and what they spent was spent but never recorded; the
  next attempt is budgeted from what the store recorded. That spend is
  the only spend beyond a budget, bounded by the turns and bytes a run
  may keep unacknowledged (hosts/hub.md, 4) at its models' prices.
- **Time is a deadline.** A task past its deadline is held, and its
  requester told. A task parked when its wall passes is made due then, so
  the hold doesn't wait for its next word; one running finishes its run
  first. Its delegates' deadlines are no later than its own.
- **Load is not spend.** A connector's requests to its system are its
  load (connectors.md, section 13).
