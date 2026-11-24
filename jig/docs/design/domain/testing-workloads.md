# Testing the domain: workloads drawn from the seed

Provisional, 2026-11-24. jig's part of the failure worlds' workloads
(`temper/docs/design/draft/failure-worlds.md`, 4.4, 4.6, 4.7 and 5): a
random workload in the core world, of tasks, questions, words, funding
and authority; one in the engine-clients world, of sign-ins and their
exchanges; people's acts and runs that park, resume and move at any step;
tables and funds at their bounds; and the sweeps over them. Each is
drawn from the seed's streams and recorded in a fault ledger, and each
sweep folds its seeds as skein's sweeps do
(`skein/docs/design/testing/sweeps.md`). The checks are those the core
kit's `World` and the engine-clients kit's `Peer` already run at their
quiet points (`testing-fakes.md`, 1.2); the open problems' exclusions are
`testing-fakes.md`, 1.3's. temper's workloads, and the kinds' families
across the trees, are `temper/docs/design/testing/workloads.md`.

## 1. In one page

- **The kinds** (section 2): jig's acts, parks and tables, beside the open
  problems' kinds that `jig_core_kit::open` and
  `jig_engine_clients_kit::open` hold.
- **The core world's workload** (section 3): `jig_core_kit::workload`
  draws a workload of tasks from `Workload`, each party's acts from its
  part of `Parties`, and drives the kit's `World` through them, each task
  held to an expectation that follows from the acts that strike it.
- **Parks** (section 4), **tables and funds at their bounds** (section 5),
  and **the engine-clients world's sign-ins** (section 6) over the same
  form.
- **Four sweeps** (section 7), each taking its exclusions from the kits'
  `EXCLUSIONS`.

## 2. The kinds

Each a `skein_world::faults::Kind`, its family the draft's section 4
family and its name the act or fault in that section's words, fixed by
value. jig's sources name no temper path or word (`testing-fakes.md`,
1.3), and these kinds name none.

`jig_core_kit::people`, `FAMILY = "people and parties"`:

| Constant | Name |
|---|---|
| `CANCELS_A_TASK` | `cancelling a task` |
| `HOLDS_A_TASK` | `holding a task` |
| `RELEASES_A_TASK` | `releasing a held task` |
| `ANSWERS_LATE` | `answering a question late` |
| `NEVER_ANSWERS` | `never answering a question` |
| `ANSWERS_TWICE` | `answering a question twice` |
| `ANSWERS_AN_ENDED_RUN` | `answering a question of a run that has ended` |
| `WORDS_TO_AN_ENDED_PRODUCER` | `words to a task whose producer has ended` |

with `jig_core_kit::open`'s `AMENDS_A_PASSED_DEADLINE`,
`WIDENS_GOAL_AUTHORITY` and `NEW_SERVICE_RUNS_ON_AN_ACCOUNT`.

`jig_core_kit::parks`, `FAMILY = "runs that park, resume and move"`:

| Constant | Name |
|---|---|
| `ANSWERED_WHILE_PARKED` | `a parked run's answer arrives while it is parked` |
| `ANSWERED_AFTER_ITS_END` | `a parked run's answer arrives after it ended` |
| `ANSWERED_TO_ITS_SUCCESSOR` | `a parked run's answer reaches its successor` |
| `COORDINATOR_ENDS_WHILE_A_CHILD_WAITS` | `a coordinator ends while a child waits for its answer` |

with `jig_core_kit::open::PARKS_WITH_A_QUESTION_OPEN`.

`jig_core_kit::bounds`, `FAMILY = "the engine's own bounds"`, a table's
kind named as its adapter names the table (`testing-fakes.md`, 1.1), in
force while the table is at its bound, recorded by
`FaultLedger::saturation` (`skein/docs/design/testing/sweeps.md`, 3):
`pub const fn table(name: &'static str) -> Kind`; `SUBSCRIPTIONS`
(`subscriptions`), `CREATION_REPORTS` (`creation reports`), `KEPT_ANSWERS`
(`kept answers`); with `jig_core_kit::open`'s `START_WITH_NO_BUDGET`,
`NEW_PARTY_ASKS_FOR_FUNDS` and `DELEGATING_RUN_AT_ITS_MINIMUM`.

`jig_engine_clients_kit::bounds`, the same family: `PENDING_EXCHANGES`
(`a sign-in's pending exchanges`).

## 3. The core world's workload

The module `workload` of `jig-core-kit`, over its `World`
(`jig_core_kit::world::World`), at the core world's standard profile and
the stock policy's funds, so that a goal's run and its delegates share
its budget as a deployment's do (the draft, 4.7).

- **The draw,** from `Stream::Workload`, part `tasks`, alone: two parties,
  an owner and a member of one project; two to four goals, each a party's
  top task, its party drawn, both parties holding one once there are two;
  each goal zero to two delegates, each of which may have one of its own;
  each task's run asking its requester one question with chance one
  third, and taking a party's words with chance one quarter; each goal's
  budget drawn within the pool the profile gives, and its deadline drawn,
  short enough to pass before its work ends with chance one quarter.
  Tasks are named `t0`, `t0.0`, `t0.0.0` by their place, and the run keeps
  each name's task number, so that a finding and an act name a task
  whatever number it took.
- **Each task's run** is scripted by its place, as the kit's stories
  script runs (`jig_core_kit::stories`): it delegates its children,
  asks its question and waits for the answer, takes its words, and
  finishes once its children have; a parked run resumes as its wake
  brings it (`stories/per_wake.rs`).
- **The acts,** from `Stream::Parties`, each party its own part, `owner`
  or `member`: for each act the sweep configures, with chance one half,
  one target of the party's tasks its moment allows, at that moment, and
  then a drawn number of the world's drain iterations, so that over the
  seeds it falls at any step. Moments are read from the core's committed
  rows: the task exists, its question is open, it is held, it is held for
  its deadline, its run has ended, its producer has ended. Each act is
  made through the people's route (`jig_core_kit::fixtures::ask` and
  `ask_as`: `Ask::Cancel`, `Stop`, revisioned `Release`, `Amend`, `Say`,
  `AnswerQuestion`), and recorded in the run's ledger as it falls, an
  instant, `Struck::default()` with `request` the target's name.
  `AMENDS_A_PASSED_DEADLINE` takes its party from both, so that the
  amending party isn't always the goal's starter.
- **The expectation,** `jig_core_kit::workload::Expect`, one for each task:

```rust
pub enum Expect {
    /// Ends, done, failed or cancelled, within its bound past the last act and fault.
    Ends,
    /// A party holds it, or never answers it: it may stay held or waiting.
    Waits,
}
```

  `Waits` for a task held and never released, or whose question is never
  answered, and for its ancestors waiting on it; `Ends` for every other.
  The bound is explicit and nonzero: simulated minutes and drain
  iterations, the run's own. A task past its bound, or ended otherwise
  than its expectation allows (a cancelled task held, a released task
  never ending), is a finding naming the task's name and number, the seed
  and the moment.
- **What a run asserts:** the `World`'s checks at every quiet point, as
  they stand; each task's expectation; each act's own outcome where the
  design gives one: an answer twice reaches its question once, an answer
  to an ended run is dropped and its sender told once
  (`tasks/messages.md`), words to an ended producer take their route's
  fallback.

## 4. Runs that park, resume and move

Over the same workload, each drawn from `Faults`, part `parks`, at a
drawn step of its target's life, and recorded as it falls:

- **`PARKS_WITH_A_QUESTION_OPEN`:** a run parks with its question to its
  requester open, and, drawn: **`ANSWERED_WHILE_PARKED`**, the answer
  comes while it is parked and its next run reads it once;
  **`ANSWERED_AFTER_ITS_END`**, its task ends first and the answer is
  dropped, its sender told once; **`ANSWERED_TO_ITS_SUCCESSOR`**, the
  answer reaches the run that resumed it, not the parked one.
- **`COORDINATOR_ENDS_WHILE_A_CHILD_WAITS`:** a goal's run ends while a
  delegate waits for its answer; the delegate's wait ends as its design
  says, woken or ended, never left with no wake (the core world's checks
  hold it).

Each takes `Ends` but where a party never answers. P43's exclusion leaves
out `PARKS_WITH_A_QUESTION_OPEN` while its requester watches it
(`testing-fakes.md`, 1.3).

## 5. Tables and funds at their bounds

Over the same workload, at the standard profile's tables:

- **The tables:** `subscriptions`, `creation reports` and `kept answers`
  reach their bounds as the workload's goals, delegations and answers
  fill them; the run calls `FaultLedger::saturation` with
  `jig_core_kit::bounds`' kinds after each quiet point's inspection.
- **The funds:** the workload draws, from `Faults`, part `funds`:
  `START_WITH_NO_BUDGET`, a goal started with no budget of its own;
  `NEW_PARTY_ASKS_FOR_FUNDS`, a party joining the project mid-run asks the
  period for funds; `DELEGATING_RUN_AT_ITS_MINIMUM`, a delegating task
  given a budget near the minimum run spend. Each is recorded as the
  world makes it, and its exclusion (P93, P98, P82) leaves it out while
  its condition holds at that moment.

## 6. The engine-clients world's sign-ins

The module `workload` of `jig-engine-clients-kit`, over its `Peer`
(`jig_engine_clients_kit::peer::Peer`):

- **The draw,** from `Stream::Workload`, part `sign-ins`: two or three
  parties, each signing in and sending two to four signed requests
  (`Peer::post_signed`), some answered only after a drawn delay.
- **The acts,** from `Stream::Parties`, each party its own part:
  `RESENDS_A_PENDING_REQUEST`, a party sending a request again under the
  same key while the first is pending, at any pass after it was sent;
  P117's exclusion leaves it out while the first is parked.
- **The bound:** the run calls `FaultLedger::saturation` with
  `PENDING_EXCHANGES` after each quiet point's inspection.
- **What a run asserts:** the `Peer`'s checks as they stand; each request
  answered once within its bound, a resend's answer the first's, and no
  exchange left pending once its answer came.

## 7. The sweeps

Each a test of its world's fuzzy suite, the loop skein's: for each seed,
the seed to stderr, a run from `Streams::new(seed)`, its ledger folded,
its findings returned at once, and `check_fell` after the last.

| Sweep | World | Family | Configures | Exclusions |
|---|---|---|---|---|
| `people_act_on_tasks_at_any_step` | core | people and parties | section 2's `people` kinds, `AMENDS_A_PASSED_DEADLINE`, `WIDENS_GOAL_AUTHORITY`, `NEW_SERVICE_RUNS_ON_AN_ACCOUNT` | S15, P77, P76 |
| `runs_park_resume_and_move_at_any_step` | core | runs that park, resume and move | section 2's `parks` kinds, `PARKS_WITH_A_QUESTION_OPEN` | P43 |
| `the_cores_tables_and_funds_reach_their_bounds` | core | the engine's own bounds | `SUBSCRIPTIONS`, `CREATION_REPORTS`, `KEPT_ANSWERS`, `START_WITH_NO_BUDGET`, `NEW_PARTY_ASKS_FOR_FUNDS`, `DELEGATING_RUN_AT_ITS_MINIMUM` | P93, P98, P82 |
| `sign_ins_resend_and_fill_at_any_step` | engine-clients | people and parties; the engine's own bounds | `RESENDS_A_PENDING_REQUEST`, `PENDING_EXCHANGES` | P117 |

- **The engine-clients sweep** holds both families: the world is the
  cheapest for each, and its one bound and one act are drawn by the same
  sign-ins.
- **Exclusions** are taken from `jig_core_kit::open::EXCLUSIONS` and
  `jig_engine_clients_kit::open::EXCLUSIONS` for each fault the sweep
  configures, and asked with `leaves_out` at the moment the act or fault
  would fall, its conditions read from the core's committed rows or the
  world's own state.
- **A kind no draw can reach** at the world's limits is not configured,
  and is named with why beside the sweep's `configure` calls.
- **Seeds:** the fewest at which every kind it configures falls, pinned,
  within the tree's fuzzy budgets as they stand; the change that adds a
  sweep gives its CPU and instructions from the suite's reports.
- **A finding** fails the sweep. One outside the open problems of
  `testing-fakes.md`, 1.3 that is the product's is asked up before
  anything is written for it; once confirmed it takes its scenario and
  its narrowest exclusion, its reason naming it as repaired as a red.

## 8. Testing

Focused and fuzzy suites only.

- **The kinds:** each kind's family is its module's, names unique within
  each family, `bounds::table` of each adapter's table name giving its
  constant.
- **The draws,** in the kits' own tests: one seed draws one workload,
  the same whether or not `Parties` was drawn first; each party's acts
  are the same whatever the other's; an act is drawn only on a target its
  moment allows, and a release only after its hold.
- **The expectation:** a task held to `Ends` that ends passes, and one
  left held past its bound is a finding naming it; `Waits` passes a task
  left held.
- **Each sweep,** in its world's fuzzy suite: every configured kind fell,
  and the world's checks and the tasks' expectations held.

## 9. Later

- The comparison of a run with restarts and one without
  (`temper/docs/design/draft/failure-worlds.md`, 3.5) over the core
  world's workload, beside the conformance world's referee.
