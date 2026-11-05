# repair-green: plan

A green head ends a CI repair (`temper/docs/design/domain/forge.md`, 8.4), cut into three tasks. The rule is written into 8.4, which closes the open question in its section 20 and the `first-goal-fixes` departure in its status.

## Parts and tasks

| Part | Task | Rule | Crates it writes | Needs |
|---|---|---|---|---|
| deployment-release | `deployment-release` | what `green-hold` needs of jig | `jig-domain-tasks` | none |
| green-repair | `green-repair` | request's 1 (due, retrying, a task held), 2, 3, 4, 5, and "released after green" | `temper-engine-domain-forge-change`, `temper-engine-domain-forge`, `temper-engine-domain` | none |
| green-hold | `green-hold` | request's 1, a change whose own task is held | the same | both |

## Critical path

`green-repair` (130 minutes), then `green-hold` (90). `deployment-release` (35) runs beside the first.

## What was decided, and why

- **A task in jig, which the request hoped to avoid, because the reading shows it.** "Held" has two forms. A repair's task can be held while the change stays `Repairing` (8.4, a brief that can't be read): `green-repair` cancels it, needing nothing of jig. Or the change's own task is held after its repair failed or was cancelled, the first goal's case: it takes no step until released, so a green head can't reach it. Going on with no person asked needs a release no person made. jig lets the deployment cancel but not release, and `tests/control.rs` pins that. Releasing as the requester would write a false line into the task's history, and a queue repair's requester is the deployment anyway. So jig gains one narrow release, for a procedure's own hold. It is one guard in `control_entrance`, no client reaches it, and no format changes. Nothing else in the rule needed asking, so no `Question:`: 8.4 and `jig/docs/design/domain/tasks.md`, 7 say it.
- **Which reasons are CI's.** `Repair::Ci` and `Repair::Semantic`, whose reason is CI failed at the head. Section 20 asked whether the rule reaches a gate's and a base's reasons; request's 4 settles a gate's, and a landing gate's, resolution's and queue ejection's go with it. 9.1's ejection is not built yet, and its CI at its own head may well have passed, so 8.4 and 9.1 say now that a green head doesn't end it.
- **Rule 2's "finding nothing to do, as today".** Today it holds. At a green head that hold would meet rule 1 at once, so it holds nothing: same outcome, with no blip to a person. At a head not green it holds as today.
- **A run's brief takes no word.** A brief is gathered at a run's start, so a run already in progress can't be told by one, and one gathered as CI turned names no failed check, which the words already say (`no failed check named`).
- **A run ending into a retry wakes no change,** and no hook says so. While a run is in progress at a green head the wait is capped to the client's `poll`: level-triggered, a few reads, for a rare window. A retry whose backoff falls between reads may start again, which the rule allows: a run in progress isn't interrupted.
- **The ended repair stays counted,** so the bound, the panel and the result agree: `repairs: 1: CI failed: build, then CI passed at 0123456789ab`. Only a person's release resets the counts; the deployment's doesn't, since `start_change`'s `released` is false once the change's state has left `Held`.
- **A hold at gating isn't ended by CI.** `Hold::Repairs` and `Hold::Authority` there can be a gate's, and the state doesn't say. Its release already decides afresh and asks no repair at a green head. This is the rule's one limit, stated in 8.4.
- **A hold the green head can't reach until a wake.** A held change reads on a wake from a hint or from the poll's pull read when the pull's `ci` changes. With required contexts, a non-required check can keep the combined status unchanged, so such a flip waits for a hint or a person's release. That costs latency, not a fresh repair.
- **The stories are in two worlds.** The forge world, scripted root, for the policy, as the request asks; `temper/tests/engine/domain` for a task's phase, its cancel and the deployment's release, which only meet there.
- **No stored or wire change, no client change.** The panel's `repairing` simply moves on.

## Overlaps we could not avoid

- `temper-engine-domain-forge/src/domain.rs`: `ui-away/away-forge`, `ui-chat-draft/person-facts`, in other functions (`decide_step`, `step_change` here).
- `temper-engine-domain/src/route/forge_route.rs`, `temper/tests/engine/domain/tests/suite/forge_routes.rs`' `mod` lines: `ui-away/decided-tool`.
- `temper/docs/design/domain/forge.status.md`: `ui-away/away-proof`, `ui-chat-draft/person-facts`. One entry each.
- `change::Heard` literals in `temper/tests/engine/**`: `note-parts`' and `ui-away`'s stories, a line each.

Nothing of fabrig/ui's client files. `jig-domain-tasks` has no neighbour.
