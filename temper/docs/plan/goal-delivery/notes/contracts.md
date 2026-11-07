# goal-delivery: the shapes that cross tasks

The run's binding calls. The design says what they mean (`jig/docs/design/domain/hosts.md`, 8; `temper/docs/design/domain/forge.md`, 8.1, 8.5, 11.1; `temper/docs/design/domain/agent.md`, 4.3, 4.3.1; `temper/docs/design/domain/worker.md`, 5).

## jig: what grants `deliver`

`jig_protocol_charter::Charter` gains one field:

```rust
/// The application's workspace delivers for this run, outside a change among its outcomes
/// (domain/hosts.md, section 8). A fact of the connectors' items; it grants nothing alone.
pub delivers: bool,
```

- `assignment` sets it `false`. A root sets it on the assembled charter, as temper's `start.rs` sets `conventions`, before it calls `charter`.
- `charter` decides, for `Mode::Finite`: `outcome.change` if there is one; otherwise, when `outcome.report` is some and `workspace.modify` and `delivers` both hold, a `ChangeSpec { checks_must_pass: CONTRACT_CHECKS_MUST_PASS, fields: [title at most CONTRACT_TEXT_MAX, body at most CHANGE_BODY_MAX] }`; otherwise none. `Mode::Open` is none. `modify` stays `workspace.modify && deliver.is_some()`.
- `outcome` is untouched: such a run's `finish` offers a report and a failure.

Smith needs no change: it admits `grants.deliver` with no change among the outcomes (`smith/docs/design/domain/run.md`, 8.4) and refuses one with no writable directory, so temper sets `delivers` only where a push survived.

## temper: the `change` tool

- `temper_engine_domain::ChangeBranch` gains `Run`. `"branch": "run"` decodes to it.
- A run change's parameters are a `"mine"` change's: bytes 3 the branch, number 11 value 1, bytes 10 its title. Only the branch's letter differs: `tree_branch(repository, root, b'r', task, ...)`.
- Refusals, as `ChangeRefused::Words`, in this order:
  - `branch run: this task holds no goal branch in <owner/name>.`
  - `branch run: this task holds no run branch in <owner/name>.`
  - `branch run: change task <N> is on it already.`
  - `branch run: an earlier change of this call is on it.`
- Both "holds" read the forge's hold row (`domain.forge.hold(&name)`), `hold.task == task`. A run branch's row is made at the claim of a run that writes it.

## temper: the writer slot

In `temper-engine-domain-forge/src/domain.rs`, a change on a branch its requester holds, with `Writer::Run { task, .. }` in the slot where `task == hold.task` and `hold.task != row.task`:

- `Facts.writer_taken` is false;
- `branch_moved` isn't set, so a read head that differs from the prior one is `take_pushed_head`'s.

Every other writer stays as it is, a delegate's run and an entry among them. `change_branch_hold` keeps its answer for `items.rs`'s holder.

## temper: a goal's workspace

In `temper-engine-domain-forge/src/items.rs`:

```rust
pub struct Seed { /* as it is */ pub modify: bool }   // the run's charter's `modify`
pub struct Item { /* as it is */ pub delivers: bool } // this item is a goal's delivery
```

For a task holding its confirmed goal branch g in the repository, with `seed.modify`, and r its `seed.run_branch`:

| r's head | start | push | landing | delivers |
|---|---|---|---|---|
| none known | `Start::Base(g)` | r | none | true |
| known | `Start::Merge { branch: g, base: r's head }` | r | g | true |

With saved work, `Start::Saved(s)` replaces the start as it does for any task: push r, landing none, delivers true. Without `seed.modify`, the item is as it is today: `Start::Branch(g)`, no push, delivers false. A task holding no confirmed goal branch is as it is today, delivers false.

`temper_engine_domain::ForgeRepository` gains `pub delivers: bool`: the item's, and false where the push didn't survive `run_workspace`'s checks. It isn't translated to the worker's wire. `temper-engine-smith`'s `charter_with_history` sets `charter.delivers` when any repository's `delivers` is true.

## The story

`temper/tests/engine/domain/tests/suite/forge_routes/goal_delivery.rs`, test `a_goals_delivery_lands_into_its_goal_branch_by_a_change`. `goal-story` writes it ignored, and `goal-walks` lifts the ignore.
