+++
id = "green-hold"
part = "green-hold"
kind = "feature"
needs = ["deployment-release", "green-repair"]
files = [
  "temper/crates/temper-engine-domain-forge-change/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge-change/src/domain.rs",
  "temper/crates/temper-engine-domain-forge-change/src/lib.rs",
  "temper/crates/temper-engine-domain-forge-change/src/tests/green.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 90
+++

# A change held on CI's reason goes on when CI passes

## Goal

`green-repair` made a step end a CI repair at a green head. A change whose own task is held takes no step until a person releases it, so its hold outlives its reason. Have a wake read as a step does while so held, and a read of CI passed end the hold, the deployment releasing the task. A person's release stays as `green-repair` left it.

Acceptance:

- A change held at its repair, failed for good, cancelled or stalled, or at its repair or checking past the bound or short of authority, with CI passing at its head afterwards, goes on with no person asked: its task is `Active` again, the change past `Repairing`, no fresh repair (`made` stays), and its escalation closed as on any release.
- `repairs`, `resolutions` and `updates` are as they were: only a person's release resets them.
- The task's history says the deployment released it.
- These stay held at a green head: a hold at `Gating`; `Hold::BranchMoved`, `BranchMissing`, `Retargeted`, `PullClosed`; `Hold::Questions`, `Rejected`, `Resolutions`, `Updates`; a held change whose repair's run is still in progress; a task a placed hold or a party's stop also holds. Each goes on by its own release as today.
- A wake at a head that is red or pending changes nothing but the panel's CI, which it now keeps current (8.8), and makes no effect.
- A release the deployment is refused leaves no harm: the change's state has moved, and its next release goes on with no fresh repair.
- `forge.status.md`'s entry `8.4: a hold on a CI repair's reason` is gone.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.4 ("A green head ends a CI repair", its **A hold ends too**); 8.8; 5.2, **Which change a hint wakes**.
- `jig/docs/design/domain/tasks.md`, section 7, **Who may**.
- `temper/docs/plan/repair-green/notes/common.md`, `notes/contracts.md`.

## Owns and may change

- Owns the files in the header's `files`, editing what `green-repair` left in them.
- `Decision` gains `EndHold`: the matches that list it take the arm their neighbours take, in `settle.rs`, `landing.rs` and `temper-engine-domain-forge/src/domain.rs`, each only for that.
- `start_change` holds the `StepChange` you need: factor its literal into a helper in the same file, no other edit there.
- If `reply_to: super::internal(u64::MAX)` can't take a release's answer as it takes a cancel's, add that arm where `internal` replies end, and say so in the report.
- If lifting needs anything more of jig than `deployment-release` gave, that is a `Question:`.

## Steps

**Stories first,** in `repair_green.rs`:

- `a_cancelled_repair_goes_on_when_ci_passes`: `repair_cancel.rs`'s `cancelled_repair`, then `ci_poll.rs`'s `status` passing, no hint, ticks until the poll: acceptance's first three lines.
- `a_repair_failed_for_good_goes_on_when_ci_passes`.
- `a_hold_past_the_repair_bound_goes_on_when_ci_passes`, from `was: Checking`, its count kept.
- `a_hold_short_of_authority_goes_on_when_ci_passes`, by `forge_routes/repairs.rs` or `repair_cancel.rs`'s funded worlds.
- `a_stopped_change_stays_held_at_a_green_head`: then released, it goes on with no fresh repair.
- `a_drifted_change_stays_held_at_a_green_head`.
- `a_red_or_pending_read_ends_no_hold`, with the panel's CI as read.

`green-repair`'s `a_cancelled_repair_released_at_a_green_head_makes_no_fresh_one` still sends the release with no tick after the status, so the release is first. It stays and passes.

**The policy.** Pure rows in `src/tests/green.rs`, one per pair in `held_on_ci`'s comment. `advance`'s last `Held` return becomes:

```rust
if let State::Held { was, why } = &change.state {
    if held_on_ci(&change.state)
        && green_head(facts).is_some()
        && !standing.delegate_running
        && facts.drift.is_none()
        && !facts.writer_taken
        && facts.pull != Pull::Closed
    {
        next.state = released_state(&next, was, *why, facts);
        next.since = now;
        next.owns_turn = false;
        return Stepped { change: next, decision: Decision::EndHold };
    }
    return Stepped { change: next, decision: Decision::None };
}
```

Going on is `repairing`'s and `checking`'s, on the step after: this state moves out of `Held` and no counter changes. Export `held_on_ci` from `lib.rs`.

**The root,** `forge_route.rs`:

- `forge::Request::WakeProcedure { task }` keeps what it does. Where `held_on_ci` is true of the change and `own_hold` of its task (`Phase::Held` with `why` one of `Procedure`, `EffectFailed`, `Stalled`, `Budget`, `placed` none, listing every variant), it also pushes the change's `StepChange`, with a fresh entry as `start_change` takes one, and `heard.released` false, whatever `due_reason` says: a release of its past mustn't read as a new one, or `release_change` resets the counters.
- A `StepChange` for a task not due already ends safely: `change_decision`'s `procedure_due` drops what isn't due. Keep that.
- `forge::Request::ChangeDecision`: `Decision::EndHold` goes before admission, as `Decision::Cancel` does, to a new `end_change_hold(domain, task)`:

```rust
domain.work.push(Work::Tasks(tasks::Event::Release {
    reply_to: super::internal(u64::MAX),
    by: tasks::Party::Deployment { project },
    operation: tasks::Release { task, again: None, message: None },
}));
```

as the deployment's cancel beside `wait_change_decision` does. `change_decision` lists `EndHold` as `unreachable!`, with its reason.

After the release, `start_change`'s `released` reads false, since the change's state is no longer `Held`, so the step goes on with its counters kept. A story holds this.

Each new rule's test must bite: drop `held_on_ci`'s check and see the drift story fail; set `heard.released` from `due_reason` and see the count story fail.

Then: `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`; `heavy cargo clippy -p temper-engine-domain-forge-change -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-forge-change -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-forge-world -p temper-engine-domain-world`, since a wake now reads for a held change. If the poll's reads push a counted budget in a world's test, that is a question of cost: report the measurements. Delete the status entry, commit and push.

## Report

As the implementer's text says, and: how many reads a held change's wake costs in the stories, and whether `internal` needed the option above.
