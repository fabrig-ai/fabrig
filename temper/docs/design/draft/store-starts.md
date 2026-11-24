# A store the engine wrote, the engine starts from

Draft, 2026-11-18, on the user's word. On 11-18 at 00:38Z a deployment's engine was restarted for a configuration change. It then refused every start for 26 minutes. Its store held two states that its own commits had made and that its start's checks refuse, yet an upgrade's check over the same store had passed. A hotfix on the release line freed the start.

## 1. What happens

- **A zero-budget task outlives its ledger, by design, but two places don't allow for it:**
  - retiring closes a pool, recurring or period ledger once nothing is reserved on it and a newer period exists (`jig/crates/jig-domain-tasks/src/retire.rs`);
  - a task whose budget is 0 reserves nothing, so its ledger closes while the task still names it as its funder. Its priced work goes to the current period (jig's core world, `jig/tests/core/tests/suite/procedure_seeds.rs`, `a_procedures_priced_effect_uses_the_current_period`). Only such a task can outlive its ledger, since any budget keeps the ledger's reservation above 0;
  - yet the start's funding check refuses such a task (`jig/crates/jig-domain-tasks/src/funders.rs`, `link`);
  - and its end expects its ledger to be live (`funders.rs`, `end`, "source preserved"), so ending it after its ledger has closed panics.
- **A task's rejection is recorded as a person's:** a hold's escalation can wait on the nearest live ancestor task (`EscalationHolder::Task`, `jig/crates/jig-domain-tasks/src/escalation.rs`). When that task rejects the escalation, its task number goes into `Escalation::Rejected`'s `by` (`jig/crates/jig-domain-core/src/routing/run/calls.rs`, `DecideEscalation`). The start's checks link `by` as a person (`jig/crates/jig-domain-core/src/start/links.rs`). No person has that number, so the start refuses.
- **The check before a swap doesn't run the start's checks:** `upgrade --check` rehearses the store's registry changes and jobs over a copy (`temper/docs/design/draft/archive/redeploys.md`, 5.1). A store that passes it can still fail a start.
- **The release line's hotfix** (branch `oob/hotfix-recurring-ledger`) works in three ways:
  - retire keeps a pool, recurring or period ledger open while any live task names it. That contradicts the core world's priced-effect case and reds six funding tests. One reason is that a task's end retires before its name leaves the live tasks (`jig/crates/jig-domain-tasks/src/closing.rs`, `end_task`);
  - a start moves an orphan that reserved nothing onto its source's newest open ledger, in memory only, so every start repeats the move;
  - a start no longer checks the parties a live task names as having taken, answered, granted, added, written, escalated, decided or cancelled.

## 2. Wanted

- **A zero-budget task may outlive its ledger:** ledgers close as they do today. A start accepts a live task whose ledger has closed when the task reserved nothing. Its end posts nothing to the closed ledger, and anything left to post goes to its project's current period, as its priced work does. No ledger is kept open, and no task is moved.
- **An escalation's rejecter is typed:** `Rejected` records whether a task or a person decided. The start links it by its kind, and reads show the kind.
- **The store's version moves, and its patch repairs what is already stored:**
  - each existing rejection is typed: a number that names a live ancestor task of the rejected task is that task's; any other is a person's. The decider is the nearest live ancestor, which can't end before the rejected task, so a task's rejection always names a live one.

  Patching a store twice changes nothing.
- **The start's checks stay whole:** none of the checks the hotfix softens is softened here. Each party a live task names is linked by its kind, and a start still refuses what it can't link.
- **The check before a swap runs the start's checks:** `upgrade --check` also restores the copy as a start does, through the domain's link and fit checks, and names the first record a start would refuse.
- **The client wire's version moves** if its hold escalation shows the rejecter.
- Revise `jig/docs/design/protocol/store/README.md`, 10.1, `jig/docs/design/domain/tasks/holds.md`, and `archive/redeploys.md`, 5.1, with their status files.

## 3. Out

- Whether a start should hold a task it can't link, rather than refuse the whole store.
- The release line: it keeps its hotfix until this lands, and then takes this in its place.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The tasks child's periods and startup tests (`jig/crates/jig-domain-tasks/src/tests/periods.rs`, `jig/crates/jig-domain-tasks/src/tests/startup.rs`):
  - a standing role's run asks a zero-budget person task in period 1, period 2 begins, and the old ledger closes;
  - a restart starts;
  - the person task then ends, with nothing posted and no panic;
  - the same holds for a pool's period.
- The tasks world's funding, recurring and standing-renewal stories, and the core world's procedure seeds (`jig/tests/core/tests/suite/procedure_seeds.rs`), pass unchanged.
- The start's links tests (`jig/crates/jig-domain-core/src/tests/start/links.rs`):
  - a hold rejected by its ancestor task, then a restart, starts;
  - a rejection, taking or answer that names a missing person still refuses (temper's `root_routes::restored_rejection_reaches_jigs_link_pass_after_temper_loads_it`, `temper/tests/engine/domain/tests/suite/root_routes.rs`, unchanged).
- The store's patch tests:
  - a store at the old version, holding a task's rejection, patches to one that starts;
  - patching it again changes nothing.
- The engine's upgrade check (`temper/bin/temper-engine/tests/suite/main.rs`, `temper/bin/temper-engine/tests/support/upgrade.rs`): a store a start would refuse is refused by the check, which names the record.

Run only the new and affected tests, by name.
