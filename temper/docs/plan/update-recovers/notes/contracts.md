# update-recovers: the run's binding calls

The design is `skein/docs/design/forge-client.md`, 7.3 and 7.5, `jig/docs/design/domain/connectors.md`, 4.3, `temper/docs/design/domain/forge.md`, 6 and 10, and `temper/docs/design/domain/landing.md`, 5.2. This note holds the shapes that cross tasks.

**No stored format or protocol changes.** `EffectResult` keeps its five variants and `Hold` its own: `temper-protocol-store` and every `temper-client-*` stay as they are. If a task finds it needs either, that is a `Question:`, not an edit.

## The class

`skein_forge_client::recovery(&Write::Update { .. })` is `Recovery::Idempotent`, for both styles. `temper_engine_domain_forge::effect_shape(11)` is `(EffectForm::Transition, Recovery::Idempotent)`. A kind stays one class: there is no class by style.

## What the client settles an update with

| What happened | Outcome |
|---|---|
| found holding its base (on an answer, or by a later look) | `Made { made: Made::Updated(head), found }` |
| checked again, head or base moved | `Failed(Stale)` |
| not found once `failures >= write_attempts` | `Failed(Timeout)` |
| the check's reads or the send unavailable, `write_attempts` times | `Failed(Unavailable)` |
| a typed refusal | `Failed(Conflict)`, `Failed(Protected)` and the like |

Never `Held`. `retry.rs`, `not_found` needs no change: it reads the class.

## Let go, in the top

`temper-engine-domain-forge/src/domain.rs`, `change_outcome`, for a settled entry whose write is `Write::Update { .. }` (both styles):

| Outcome | `row.effect` | `row.pending` | `row.effect_account` |
|---|---|---|---|
| `Failed(Timeout \| Unavailable \| Stale)` | `EffectResult::None` | `None` | the account, kept |
| `Failed(Conflict)` | `Conflict`, as now | `None` | as now |
| any other `Failed`, `Raced`, `Held`, `Withdrawn` | `Failed`, as now | `None` | as now |

A held update already stored, answered `again`, comes back through `Remake` and settles as above.

`release_change`, same file: when the change being released is held from `Updating` or `Preparing` and `row.effect` is `Failed`, `row.effect` becomes `None`, so the release decides again. `Conflict` is kept: it still resolves after a release.

## Deciding again, in the machine

`temper-engine-domain-forge-change/src/domain.rs`. `EffectResult::None` in `updating` and `preparing` is split from `Pending`, which still waits. On `None`:

| `facts.branch` | `facts.contains_base` | `updating` | `preparing` |
|---|---|---|---|
| `None` | any | `waiting` | `waiting` |
| any | `Unknown`, `Pending` | `waiting` | `waiting` |
| `from` | `Failed` | `update_due(change, from, facts.base_tip, ready_since, ..)` | `prepare_due(change, from, main, ready_since, ..)` |
| `from` | `Passed` | `state = pushed(head)`, `last_head = head`, `since = now`, `Decision::None`; `clean` unchanged | the `Made` arm |
| moved | `Passed` | the `Made` arm | the `Made` arm |
| moved | `Failed` | `hold(change, Hold::BranchMoved)` | `hold(change, Hold::BranchMoved)` |

`prepare_due` is `ready`'s body from its `updates` check on, moved out and called from both, so it holds `Updates` at the bound. `update_due` is as it is. `updating` takes `ready_since`, which `step` now ignores.

The top always shows `Pending` while `row.pending` is set, so `None` in these two states means a write let go or a result released, never a write in flight.

## The hold's words

`temper-engine-domain/src/route/forge_route.rs`, the change's hold: `Hold::Updates` stays `tasks::Hold::Procedure`, and, where the row has an `effect_account`, it is `StepEnd::HoldFrom` with the account's words and entry. With none, as now. `Hold::Failed` and the words of `Outcome::Held` are as they are.
