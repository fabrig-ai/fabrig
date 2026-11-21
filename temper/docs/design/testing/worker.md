# The worker's checks and faults

Provisional, 2026-11-21. What the worker's worlds check of its waits and its disk, and the faults they inject: its inspection adapter, the four faults of the worker and its machine, where each world calls the checks and sweeps the faults, and the worker's open problems as ignored scenarios. Written from `../draft/failure-worlds.md`, 3.1, 3.2, 4.3, 5 and 7's P109 row. The adapter's form, the shared checks, the fault ledger and sweeps are skein's (`skein/docs/design/testing/inspection.md`; `skein/docs/design/testing/sweeps.md`); `checks.md`, 6 places this adapter beside the engine's.

## 1. In one page

- **The adapter** (section 2), `temper/testing/temper-worker-inspect`, component `worker`: the checkout's waits (a prepare queued for a permit or waiting for another's store, an operation in flight) and the worker's bounded tables (workspaces, prepare permits, per-run scratch, the Git stores' packs, git's locks, run slots and trash), each entry with the rule that releases it.
- **The faults** (section 3), family `worker`, each recorded in the run's fault ledger:
  - a workspace fetch timing out partway, leaving git's locks and index [behaviour];
  - the fake disk filling [behaviour];
  - a trash entry that can't be removed [behaviour];
  - a placement refused as busy while build places are full [composition].

  All four are reachable: a legitimate sequence of events makes each. What each models is what the world tests; the real mechanism stays outside its proof.
- **Three worlds** (section 4): the checkout world and the worker's domain world inject the first two, and the domain world the fourth; the git protocol world injects the third. Each calls skein's two checks at its quiet point and sweeps its faults over 8 seeds.
- **Open problems** (section 5): P109's scenario, and a stuck trash entry holding back every entry behind it, found by this design. Each is an ignored scenario naming *worker disk*, the work the draft's section 7 names. Neither needs an exclusion, since no sweep's check finds either.
- **No fix:** the worker's disk handling is unchanged. The faults show what it does, and the scenarios what it should do.

## 2. The adapter

### 2.1 Its functions

A kit of temper's tree (`skein/docs/design/testing/inspection.md`, 2), one function per part a world holds, each adding to one inspection:

```rust
pub const COMPONENT: &str = "worker";
/// The waits the checkout documents, in its design's words (2.2).
pub const KINDS: &[&str] = &[
    "a prepare queued for a permit",
    "a prepare waiting for another's store",
    "an operation in flight",
];
/// `Inspection::new(COMPONENT, KINDS)`.
pub fn inspection() -> Inspection;
/// The whole worker: `inspect_root`, `inspect_checkout` over `domain.checkout()`, and `inspect_disk`.
pub fn inspect(domain: &temper_worker_domain::Domain, limits: &temper_worker_domain::Limits, disk: &skein_fake_checkout::Checkout, into: &mut Inspection);
/// The root's run slots.
pub fn inspect_root(domain: &temper_worker_domain::Domain, limits: &temper_worker_domain::Limits, into: &mut Inspection);
/// The checkout child: its holds' waits, its workspaces and its prepare permits.
pub fn inspect_checkout(checkout: &temper_worker_domain_checkout::Domain, limits: &temper_worker_domain_checkout::Limits, into: &mut Inspection);
/// What the fake disk holds for the checkout: per-run scratch, packs and git's locks.
pub fn inspect_disk(checkout: &temper_worker_domain_checkout::Domain, limits: &temper_worker_domain_checkout::Limits, disk: &skein_fake_checkout::Checkout, into: &mut Inspection);
/// The git component's trash, with its entries as the world lists them beneath `trash/`.
pub fn inspect_trash(component: &temper_worker_protocol_git::Component, trash: &[&[u8]], into: &mut Inspection);
```

- **The limits** are those the world steps the domain with: a domain reads some of them from each step's `Env` rather than keeping them (`checkout.preparations`), so a table's capacity comes from them.

- **Its fault kinds** are the kit's too, so every worker world names them alike (section 3.5).
- **The fake disk's layout** (where a workspace, a repository and a store sit on the fake checkout) is the domain-face fake's, which the worlds' translation writes (`fakes.md`, 3). The adapter reads it from the one place that writes it, never from a copy.
- **A waiter's name** is `hold <h>`, the hold's token in decimal, in every function, so that the waits and the tables of one inspection resolve. No other component of the worker's worlds names a hold. The system world's `task N` across engine and worker is later (section 7).

### 2.2 Its waits

| Kind | Waiter | Wakes | Bound |
|---|---|---|---|
| a prepare queued for a permit (`../domain/worker/checkouts.md`, *Preparations a few at a time*) | the queued hold | each active prepare's hold, as `Waiter` | none: its deadlines count from its admission |
| a prepare waiting for another's store (`../domain/worker/checkouts.md`, *One object store per repository*) | the waiting hold | the hold making the store, as `Waiter`; `Ended` where that hold has ended and no claim passed on | none |
| an operation in flight (`../protocol/git.md`, 7) | the hold | `Timer` at its deadline, and `Party("git")` | none: the git component's deadline and io's close end it, and a world may answer after the deadline |

A cancelled prepare waiting for its operation to end is the third kind: it ends once that operation does.

A check waiting for a build place is no wait of the worker's. Build places are the deployment's (`../domain/worker/README.md`, 5, *Its build places too*), and the domain world's fake builder (3.4) inspects its own, under component `builder`, kind `a check waiting for a build place`, woken by each check holding a place.

### 2.3 Its tables

| Table | Capacity | Entry and key | Rule | Release |
|---|---|---|---|---|
| workspaces | `checkout.workspaces` | each cached workspace, `workspace <t>` | its hold's release | `Pending` while a live hold holds it; `Kept` while idle, kept for its workstream until its slot is taken; `Due` while a hold that has ended still holds it |
| prepare permits | `checkout.preparations` | each active prepare, `hold <h>`; each queued prepare `Refused` since it was queued, with no bound | the prepare's settling | `Pending`; `Due` once the prepare has settled and still holds its permit |
| per-run scratch | `checkout.workspaces` × `checkout.repositories` | each repository's run `.git` on the fake disk (the run's view, and what its commands and checks write there: `../protocol/git.md`, 4.13), `workspace <t> <repository>` | the workspace's release | `Pending` while its workspace is cached; `Due` when no cached workspace has its token |
| Git stores' packs | 2 × `checkout.workspaces` × `checkout.repositories` | each store and each workspace repository holding packs on the fake disk (`fakes.md`, 3), `store <s>` or `workspace <t> <repository>` | the packing its own count triggers (`../domain/worker/checkouts.md`, *Kept packed*) | `Pending` at 75 packs or fewer; `Due` above 75, past the most a place holds between two counts |
| git's locks | none: `u64::MAX` | each `*.lock` on the fake disk beneath a workspace or a store's `refs/temper/<t>`, keyed by its path | the workspace's making again (`../protocol/git.md`, 4.1 and 7, *Kill at once*) | `Pending` while its workspace is untrusted or not cached; `Due` while the checkout trusts the workspace it belongs to |
| run slots | the hub's slots | each hosted run, `run <r> attempt <a>` | the run's end, its answer acknowledged | `Pending` |
| trash | none: `u64::MAX`, since `../protocol/git.md`, 8 bounds trash by what it took, not by a count | each entry beneath `trash/`, `trash/<n>` | the sweep (`../protocol/git.md`, 4.1) | `Pending` while a pass runs or is due, and after a pass that ended at a refused removal, its failure counted, which leaves what it hadn't reached for the next pass; `Due` once no pass runs or is due, the last ended with no failure, and the entry is still there |

The fake builder's table, `build places`, has the builder's places as capacity, each check holding a place as an entry keyed `run <r>`, released by the check's end (`Due` once its run has ended), and each check waiting for a place as `Refused`, since it asked.

### 2.4 The reads it adds

Each gains a `pub` read documented as its adapter's (`skein/docs/design/testing/inspection.md`, 2), a borrow, a count or an entry by key, under the strict subset. Their names and shapes are their implementer's; what they give is this:

- **`temper_worker_domain_checkout::Domain`:**
  - each cached workspace: its token, held or idle, the hold holding it, and whether the checkout trusts it or will make it again;
  - each store: its token, made or being made, the hold making it, and the holds waiting for it, in order;
  - the active prepares and the queued ones, each queued one with when it was queued;
  - each hold's operation in flight, with its deadline, and whether the hold has ended.
- **`temper_worker_domain::Domain`:** the hub's hosted runs and its slots, through `host()`.
- **`temper_worker_protocol_git::Component`:** the sweep's state: whether a pass runs or is due, the failures it has counted, and whether the last pass ended at a refused removal.

## 3. The faults

Each is drawn from the world's `Faults` stream, part `worker` (`skein/docs/design/testing/sweeps.md`, 2), and recorded in the run's ledger as it falls: `struck.request` names the operation or attempt it strikes (`hold <h> fetch`, `run <r> attempt <a>`), and its other identities are `None`. None of the six counters of the draft's 3.3 applies: a worker world spends no try.

### 3.1 A workspace fetch timing out partway

- **The fault** [behaviour, reachable]: an operation that fetches from the forge for a workspace is cut partway at its deadline, and answers `Failed { fault: TimedOut }`. What it leaves, in the domain-face fake (`fakes.md`, 3):
  - a `Fetch`: the store's import made; the store's reference for the workspace not moved, a lock left beside it (`refs/temper/<t>/<reference>.lock` in the store); the workspace's reference not moved;
  - a `CheckOut` or a `Merge` (each fetches the file contents it lacks, `../domain/worker/checkouts.md`, *One operation at a time*): the first half of the files it would write, in path order, written; `.git/index.lock` and a partial `.git/index` left; the head and merge metadata not moved.
- **Git's locks, as the fake models them:** while `.git/index.lock` exists in a repository, a `CheckOut`, `Merge` or `Commit` there fails `Broken`, as git refuses with `Unable to create '…/index.lock': File exists`; while a reference's `.lock` exists, an operation that moves it (`Fetch`, `Point`) fails `Broken`, as git's `cannot lock ref`. `Make` takes them with the workspace and its references in each store, as it takes everything there.
- **What the model tests:** the checkout never trusts a workspace whose operation timed out (`../protocol/git.md`, 7, *Kill at once*). Its next prepare for the workstream makes it again and ends ready, no lock is left in a workspace the checkout trusts (2.3, git's locks), the store keeps what the cut fetch imported, and no other workspace is touched.
- **Outside the proof:** real git's locking and what a killed git leaves besides (temporary packs, `shallow.lock`), and how long a real kill takes.
- **Ledger:** an instant, stopped at its start.

### 3.2 The fake disk filling

- **The fault** [behaviour, reachable]: the fake disk has a capacity in bytes, a world setting, over the contents of the fake checkout's regular files. At or past it, an operation that writes (`Store`, `Clone`, `Fetch`, `Point`, `CheckOut`, `Merge`, `Commit`, `Reset`, a `Reds` write) changes nothing. It fails `Broken`, with git's words, `No space left on device`, where its answer carries a diagnostic. `Make` and `Pack`, which free, and reads never fail for space. The disk fills with the world's own writes and frees as `Make` takes a workspace. A world draws the capacity from the bytes in use at a drawn moment, so that it fills partway through a run.
- **What the model tests:** an operation refused for space fails `Broken` and its prepare fails, naming its step and git's words (`../domain/worker/checkouts.md`, 1.1). Its workspace is made again at its next use. Every hold is released and no wait hangs. A refused write changes nothing.
- **Outside the proof:** a real file system's pressure, what real git writes and how much, and anything that would free a real disk beyond `Make` (the *worker disk* work).
- **Ledger:** from the first operation refused for space, stopped at the first that passes once the bytes in use fall below the capacity, or `None` if none does.

### 3.3 A trash entry that can't be removed

- **The fault** [behaviour, reachable]: in the git protocol world's io (`fakes.md`, 3), a path the world marks stuck refuses every removal with `Permission`, as a file another user owns or an immutable one does. The world marks an entry an operation has just set aside in `trash`, or a file beneath one.
- **What the model tests:** the sweep counts the failure and leaves the entry for the next pass (`../protocol/git.md`, 4.1). Every operation goes on as before, and the trash table finds nothing `Due` (2.3).
- **Outside the proof:** why a real file system refuses, and what a person does about it.
- **Ledger:** from the mark, `None`: the entry stays stuck for the run.
- **What it shows beyond its check:** a pass ends at its first refused removal, and the next pass starts again at the first entry. So a stuck entry first in `trash` holds back every entry behind it for good, and `trash` grows with every `Make`: section 5's second scenario.

### 3.4 A placement refused as busy while build places are full

- **The fault** [composition, reachable]: in the worker's domain world, a fake builder holds a few build places, as a deployment's hold does (`../domain/worker/README.md`, 5, *Its build places too*). A run's delivery check holds a place, or waits for one, until the world ends it. With every run slot taken by a run in its check, an assignment arrives and the worker refuses it `Busy`; the world places it again after its backoff, as the engine does.
- **What the model tests:** the refusal is `Busy` and the worker keeps nothing of the refused attempt: no workspace, prepare or slot. Once a check ends and its run ends, the attempt placed again is admitted.
- **Outside the proof:** the deployment's build places and their hold (`tools/tiers/design.md`, 3), smith's check clock while it waits for a place (the draft's 4.3, smith's run world), and the engine's placement (P41).
- **Ledger:** from the refusal, stopped when the attempt placed again is admitted.

### 3.5 Their kinds

In the adapter's kit, `temper_worker_inspect::faults`, each a `skein_world::faults::Kind` named as the draft's 4.3 names it:

```rust
pub const FETCH_CUT: Kind = Kind { family: "worker", name: "a workspace fetch timing out partway" };
pub const DISK_FULL: Kind = Kind { family: "worker", name: "the fake disk filling" };
pub const TRASH_STUCK: Kind = Kind { family: "worker", name: "a trash entry that can't be removed" };
pub const PLACEMENT_BUSY: Kind = Kind { family: "worker", name: "a placement refused as busy while build places are full" };
```

## 4. Where the worlds call them, and the sweeps

| World | Calls | At | Faults | Sweep's seeds |
|---|---|---|---|---|
| the checkout world, `temper-worker-checkout-world` | `inspect_checkout`, `inspect_disk` | each iteration, where it checks its bounds, before its domain reclaims (`tests/worker/checkout/src/world.rs`) | `FETCH_CUT`, `DISK_FULL` | 8, in its `fakes` binary |
| the worker's domain world, `temper-worker-domain-world` | `inspect`, and the fake builder's own | its quiet point: once its whole worker has settled, before time moves (`tests/worker/domain/src/whole.rs`) | `FETCH_CUT`, `DISK_FULL`, `PLACEMENT_BUSY` | 8, in its `suite` binary |
| the git protocol world, `temper-worker-protocol-git-world` | `inspect_trash` | after each operation's end, once the sweep's file operations due have run | `TRASH_STUCK` | 8 |

- **Each call** makes one `Moment` (the seed, the world's time, its iteration) and runs `check_waits` and `check_tables` over the world's inspections. The first `Err` is the run's finding, which fails it (`skein/docs/design/testing/inspection.md`, 5).
- **Each sweep** is a fuzzy test: it seeds each run with `Streams::new(seed)` (the workload from `Workload`, the faults from `Faults`'s part `worker`), folds each run's ledger, and ends with `check_fell` (`skein/docs/design/testing/sweeps.md`, 4). Today's tests in these worlds keep their own seeding.
- **No exclusion:** neither of section 5's problems is a finding of these checks. A finding of a check here that is the worker's own behaviour, not the adapter's or a fake's gap, is an open problem the draft's section 7 missed: it gets its ignored scenario and exclusion, by name, with its reason (`../draft/failure-worlds.execution.md`, 2).
- **The budget:** 8 + 8 seeds are the plan's for 4.3 (`../../development/failure-worlds-budget.md`). The git protocol world's 8 are beyond it, and draw a few short operations each.

## 5. Open problems: ignored scenarios

| Problem | Scenario | Where | It asserts | Fixed by |
|---|---|---|---|---|
| P109 | `growing_packs_trigger_packing` | the checkout's step tests, `temper/crates/temper-worker-domain-checkout/src/tests/` | a store into which 25 fetches have gone since it was last packed is named, with `store: Some`, by a `Pack` the next prepare asks before it ends ready (`../domain/worker/checkouts.md`, *Kept packed*) | worker disk |
| found here | `an_unremovable_trash_entry_holds_back_no_other` | the git component's step tests, `temper/crates/temper-worker-protocol-git/src/tests/files.rs` | with an entry first in `trash` that refuses removal and others after it, the next pass removes the others and leaves the stuck one, its failure counted | worker disk |

Each is `#[ignore = "<reason>"]`, its reason naming the work and what the worker does today. The fixing work removes the `#[ignore]` (the draft's section 6).

## 6. Testing

Focused, but for the sweeps:

- **Each fake behaviour's step tests:**
  - in the checkout world's `fakes` binary: a cut `Fetch`, `CheckOut` and `Merge` each leave what 3.1 says and move nothing else; each lock refuses what 3.1 says and `Make` takes it; at capacity, each writing operation fails as 3.2 says and changes nothing, while `Make`, `Pack` and reads pass; the packs of 2.3's table as the fake keeps them (`fakes.md`, 3);
  - in the git protocol world: a stuck path refuses every removal, and its neighbours are removed;
  - in the domain world: the fake builder grants its places in order of asking as each frees.
- **The adapter, in the world that reaches each read most cheaply:** a scenario where the check finds what it should and one where correct behaviour passes it, among them a lock planted in a workspace the checkout trusts (`Due`), a prepare queued while a permit is free, and a `trash` entry left after a pass that ended with no failure.
- **Each fault's model** (3.1 to 3.4) as a scenario, asserting what it tests.
- **The sweeps** of section 4 pass, each fault falling.
- **Section 5's scenarios** compile and are ignored.

## 7. Later

- Waiters named `task N` across the engine's and the worker's inspections, for the system world (`../draft/failure-worlds.md`, 5; its F9).
- skein's fake machine with a capacity and nodes that refuse removal, for the simulated tier's one-process world, where the git component runs on it.
- A store's own pack count read from the checkout, once *Kept packed* is built, beside the fake disk's.
