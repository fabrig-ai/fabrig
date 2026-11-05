# repair-green: the shapes that cross tasks

Binding for `green-repair` and `green-hold`. All in `temper-engine-domain-forge-change`, `boundary.rs` and `domain.rs`, unless named.

## `Heard` gains one field (`green-repair`)

```rust
/// Its delegate has a claimed or running attempt, or is closing: it finishes
/// as a run does (temper/docs/design/domain/forge.md, 8.4).
pub delegate_running: bool,
```

The root fills it, in `temper-engine-domain/src/route/forge_route.rs`, from the task of `row.delegate`:

| The delegate's task | `delegate_running` |
|---|---|
| none, or no longer live | false |
| `Phase::Waiting` | false |
| `Phase::Active`: `Idle`, `Due`, `Preparing`, `BackingOff` | false |
| `Phase::Active`: `Claimed`, `Running` | true |
| `Phase::Closing` | true |
| `Phase::Held`, `was` is `Active(Claimed or Running)` or `Closing` | true |
| `Phase::Held`, any other `was` | false |
| `Phase::Ended` | false |

## `Decision` gains three variants

```rust
/// Cancel the live CI repair: CI passed at this head and no run of it is in progress (8.4).
CancelRepair { head: Commit },   // green-repair
/// A CI repair ended at this green head and the change goes on from it. The top
/// records it and steps on: it never reaches the root (8.4).
RepairEnded { head: Commit },    // green-repair
/// The change's own hold ended at a green head, with no party's release. The root
/// releases its task as the deployment's (8.4).
EndHold,                         // green-hold
```

## The policy's reading (private to `domain.rs`, except where named)

```rust
// Repair::Ci and Repair::Semantic are CI's reason; Repair::Gate and Repair::Landing are not.
fn ci_reason(why: Repair) -> bool;
// The branch's head, where facts.ci is at it and reads Passed.
fn green_head(facts: &Facts) -> Option<Commit>;
```

`green-hold` adds, public, for the root's check before it reads:

```rust
/// A hold whose reason is CI failed at the head (8.4).
/// `was` Repairing with a CI reason: Failed, Stalled, Repairs or Authority.
/// `was` Checking or CancellingGates: Repairs or Authority.
/// Every other pair is false: Gating, drift, Questions, Rejected, Resolutions, Updates.
#[must_use]
pub fn held_on_ci(state: &State) -> bool;
```

## A cancelled repair's reason

`CI passed at head ` then the head's 64 hex digits, as `cancel_change_gates` writes `CI failed at head `.

## The change's result

`temper-engine-domain-forge/src/result.rs`, on the account's repairs, after the reason it was asked for, `repaired`'s words:

```rust
/// A CI repair ended by a green head (domain/forge.md, 8.4). Nothing to say where none was asked.
pub(crate) fn ci_passed(&mut self, head: &Commit, bound: u32);
```

It adds `, then CI passed at ` and `crate::commit_text(head)`: `repairs: 1: CI failed: build, then CI passed at 0123456789ab`.

## The deployment's release (`deployment-release`)

`tasks::Event::Release { reply_to, by: Party::Deployment { project }, operation: Release { task, again: None, message: None } }` is admitted for a procedure task of that project, with no placed hold, in `Phase::Held` with `why` one of `Procedure`, `EffectFailed`, `Stalled`, `Budget`. Everything else from the deployment stays `Refusal::Reference`.
