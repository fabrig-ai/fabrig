# Blame and tries: what the engine blamed and spent, judged against the cause

Provisional, 2026-11-21. The check of `../draft/failure-worlds.md`, 3.3,
for gates: what the engine retried, repaired and held after a gate went
red, judged against the faults the world injected. It is a referee
expectation (`checks.md`, 6): it reads a world's fault ledger (skein's
`testing/sweeps.md`, 3) and what the world saw the engine do, never the
engine's state. Its kit is `temper/testing/temper-blame`
(`temper_blame`), ordinary test Rust on `skein-world` alone, so that
every temper world with a ledger calls the same check: the engine's
forge world first (`fakes.md`, 2), the composed system world later.
Runs' blame, the providers' and a session's retries and a task's tries,
joins it from the providers' work (section 7).

## 1. In one page

- **The table** (section 2): for each class of a gate's failure, its
  expected gate retries, repairs and what a hold names, as the draft's
  3.3 gives them for gates.
- **Kinds and classes** (section 3): each fault kind a gate row covers
  maps to its class; a kind no row covers (a busy read, a lagging view)
  maps to none and is judged by no row.
- **The judgement** (section 4): each reaction the world saw, a gate run
  again, a repair asked, the base's own repair, a hold, is explained by
  the faults that struck its identities at or before it; a reaction no
  fault of the right class explains is a finding, and so is a gate
  retried past its bound or a wrong tree repaired twice.
- **The controls** (section 5): two faults on one change at once, and a
  red base under a green candidate, each a focused scenario of the
  engine's forge world that the check passes.
- **Where the design differs from a row** (section 6), the difference is
  an ignored scenario, which a sweep leaves out by name.

## 2. The table, for gates

| Class | Gate retries | Repairs | A hold names |
|---|---|---|---|
| runner: a runner stopped mid-job (set up failed, lost, cancelled by no one of temper's), a gate with no report, a gate that never reports, a job's wait for a runner | one per fault, to the gate's bound | none | the runner |
| starved: a gate red with no failing test and its tests unobserved | as the runner's | none | the runner |
| change: the candidate's own red, its tree wrong | none | one per wrong tree | the change |
| base: a red inherited from the base | none | none of the change's; the base's own, once | the base |

- **The other counters** of the draft's table, a provider's retries, a
  session's and a task's tries, and a person's asks, are no gate's: no
  gate fault spends one, and a world that runs tasks counts them by the
  runs' rows (section 7).
- **The bounds:** a landing's gate is retried at most its project's
  `gate_retries` per key (`../domain/landing-settings.md`, 1.1;
  `../domain/landing.md`, 7.4); a change's CI at most `reruns` per job at
  one head (`../domain/forge/change/README.md`, 2, step 3).

## 3. Kinds and classes

```rust
pub enum Class { Runner, Starved, Change, Base }

/// The class of a kind a gate row covers; `None` for any other.
pub fn class(kind: skein_world::faults::Kind) -> Option<Class>;
```

| Kind | Class | Recorded by | Strikes |
|---|---|---|---|
| `ci/set_up`, `ci/lost`, `ci/runner_wait` | runner | the fake forge (skein's `testing/fakes/forge.md`, 2.4) | `gate_run`, `runner` |
| `ci/silent` | runner | the fake forge | `gate_run`, or `candidate` for a context's |
| `ci/cancelled` | runner | the world, as it cancels a run as another user | `gate_run` |
| `ci/missing_report` | runner | the world, as a failed run of a gate that reads a report completes with none uploaded | `gate_run` |
| `ci/starved` | starved | a world that starves a gate | `gate_run` |
| `ci/wrong_tree` | change | the world, for each candidate a wrong tree is in: each head it pushes wrong, and each landing candidate holding one | `candidate` |
| `ci/base_red` | base | the world, as it makes a base red | `base` |

Every other kind, the fake's `ci/rerun` included, has no class.
`temper_blame` exports each kind of this table as a `Kind` constant
(`WRONG_TREE`, `BASE_RED`, ...), which worlds record with.

## 4. The judgement

```rust
/// What a hold's kind blames.
pub enum Names { Runner, Change, Base }

pub enum Reaction {
    /// A gate run again after its run `run` ended: its failed jobs rerun,
    /// or the gate dispatched afresh. `bound` is the gate's (section 2).
    GateRetry { gate: String, run: String, bound: u64 },
    /// A repair asked of `change` for its red at `candidate`, on `base`.
    Repair { change: String, candidate: String, base: String },
    /// The branch's own repair, asked once for its red at `base`.
    BaseRepair { base: String },
    /// A hold, or a pause, of `object`, blaming what `names` says.
    Hold { object: String, names: Names, run: Option<String>,
           candidate: Option<String>, base: Option<String> },
}

pub struct Observed { pub at: skein_lib::Time, pub reaction: Reaction }

/// Judges each reaction in order, counting what it attributes on the
/// ledger (`Counter::GateRetries`, `Counter::Repairs`); the findings.
pub fn judge(ledger: &mut FaultLedger, observed: &[Observed], seed: u64) -> Vec<String>;
```

- **Identities** are the ledger's (skein's `testing/sweeps.md`, 3): a
  run's id in decimal, a commit's hex as the engine names it, a change
  as `task N`, a gate as its repository and key.
- **A reaction is explained by** the faults that fell at or before it on
  an identity it names, stopped or not (`FaultLedger::fallen`): a
  retry's run; a repair's candidate and base; a base repair's base; a
  hold's run, candidate and base.
- **A gate retry** needs a runner or starved fault among them, and counts
  one gate retry on each. A gate's retries past its `bound` are a finding.
- **A repair** needs a change fault on its candidate, and counts one
  repair on it. One that has none is a finding naming what did strike:
  the base's red, the runner's, or nothing. A change fault counted more
  than one repair is a finding.
- **A base repair** needs a base fault on its base, and counts one repair
  on it.
- **A hold** needs a fault among its explaining ones whose class names
  what it names: runner and starved the runner, change the change, base
  the base.
- **A finding** reads `blame: <reaction> of <object> at <time>, seed
  <seed>: <why>; struck by <family/name, ...>` (`nothing` where none
  struck), one line each, in the reactions' order.
- **What the check leaves:** a fault that never got the reaction its row
  expects is no finding here, since whether a goal goes on is
  `../draft/failure-worlds.md`, 3.4's; each control asserts its own
  counts.

**In the engine's forge world** the world builds the reactions from
what it sees (`temper/tests/engine/forge`): a gate retry from a
`Write::Rerun` of a run's job, or a fresh `Write::Dispatch` of a gate
key whose last run ended in a fault; a repair from a change's
`Delegate::Repair` decision; a base repair from `Decision::QueueRepair`;
and a hold from each decision that holds, named as:

| Names | The engine's holds |
|---|---|
| the runner | a landing's `Hold::Faults` and `Hold::Timeout`; a change's `Hold::Stalled` while it checks |
| the change | a change's `Hold::Repairs`, `Hold::Failed` and `Hold::NoTest` |
| the base | a change paused for its branch's red; a branch repair that can't be admitted |

A hold of another kind is no gate's, and the check leaves it.

## 5. The controls

Focused scenarios of the engine's forge world, each passing the check,
each asserting its own counts:

- **Two faults on one change at once:** a landing of one change whose
  tree is wrong (`ci/wrong_tree` on the candidate) and whose gate's job
  is lost at its first attempt (`ci/lost` on the run). The gate is
  retried once, for the lost job, and the change is repaired once, for
  its tree: the retry is the runner's and the repair the change's.
- **A red base under a green candidate:** a queue whose branch's tip CI
  is red (`ci/base_red` on the base) while a change's own head is green.
  The change is not repaired; the branch gets its own repair once, and
  the change waits paused for the branch's red.

Each has a twin the check finds: the same ledger with the reactions of a
policy that blamed the wrong cause (the lost job repaired; the green
change repaired), in the kit's step tests.

## 6. Where the design differs from a row

The draft's rows are the target (`../draft/failure-worlds.md`, 3.3).
Where the engine's design differs, the difference is an ignored
scenario of the engine's forge world, its reason naming the work that
fixes it, and the sweep that would meet it leaves out exactly its
combination:

| Scenario | It asserts | The design today | Fixed by |
|---|---|---|---|
| `a_runner_fault_past_its_reruns_holds_without_repair` | a change's CI failed by its runner past `reruns` holds naming the runner, and asks no repair | repaired (`../domain/forge/change/README.md`, 2, step 3) | a change's reds |
| `a_cancelled_check_is_the_runners` | a change's CI job cancelled by no one of temper's is rerun as the runner's | the change's (the same step) | a change's reds |
| `a_red_inherited_from_the_base_asks_no_repair` | a landing gate red only for what its base already fails asks no repair, and holds naming the base | repaired (`../domain/landing.md`, 7.3) | a change's reds |

A sweep that finds another difference adds its row here, with its
scenario and exclusion, in the same change.

## 7. Open questions

- **Runs' blame:** the rows for a rate window, spent credits, a refusal,
  a settled stop and a crash, with the providers' and a session's retries
  and a task's tries, as reactions of their own. Default: the providers'
  work adds them to this kit and document, in their own sections.
- **A hold naming the runner itself:** the fake forge names a runner only
  in its pool (skein's `testing/fakes/forge.md`, 2.3), and the engine's
  runner holds name the gate and its run. Default: a runner's hold counts
  as naming the runner.
