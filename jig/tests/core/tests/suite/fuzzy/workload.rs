//! Workload sweeps (`jig/docs/design/domain/testing-workloads.md`, 7).
use jig_core_kit::{bounds, open, people, workload};
use skein_world::{streams::Streams, sweep::Sweep};

#[test]
fn people_act_on_tasks_at_any_step() -> Result<(), String> {
    let configured = [
        people::CANCELS_A_TASK,
        people::HOLDS_A_TASK,
        people::RELEASES_A_TASK,
        people::ANSWERS_LATE,
        people::NEVER_ANSWERS,
        people::ANSWERS_TWICE,
        people::ANSWERS_AN_ENDED_RUN,
        people::WORDS_AFTER_ITS_RUN_ENDED,
        open::AMENDS_A_PASSED_DEADLINE,
        open::WIDENS_GOAL_AUTHORITY,
    ];
    let mut sweep = Sweep::new("people_act_on_tasks_at_any_step");
    for kind in configured {
        sweep.configure(kind);
    }
    // NEW_SERVICE_RUNS_ON_AN_ACCOUNT cannot fall: the plan profile's two people
    // slots already hold the owner and member; a third service cannot be admitted.
    for exclusion in open::EXCLUSIONS {
        if configured.contains(&exclusion.fault) {
            sweep.exclude(exclusion)?;
        }
    }
    // Each is necessary: 1 amends a passed deadline, 3 widens authority,
    // and 9 never answers. Together they strike every configured kind.
    for seed in [1, 3, 9] {
        eprintln!("workload seed {seed}");
        let ledger = workload::run(Streams::new(seed), &configured, &sweep)?;
        eprintln!("workload fell {seed}: {:?}", ledger.fell());
        sweep.ran(&ledger);
    }
    sweep.check_fell()
}

#[test]
fn the_cores_tables_and_funds_reach_their_bounds() -> Result<(), String> {
    let configured = [
        bounds::KEPT_ANSWERS,
        open::START_WITH_NO_BUDGET,
        open::NEW_PARTY_ASKS_FOR_FUNDS,
        open::DELEGATING_RUN_AT_ITS_MINIMUM,
    ];
    let mut sweep = Sweep::new("the_cores_tables_and_funds_reach_their_bounds");
    for kind in configured {
        sweep.configure(kind);
    }
    // SUBSCRIPTIONS: capacity includes the reserved third subscription, whereas
    // these runs need only one timer; child answers are delivered without a watch.
    // CREATION_REPORTS: 64 slots, and these tasks name no held resources.
    for exclusion in open::EXCLUSIONS {
        if configured.contains(&exclusion.fault) {
            sweep.exclude(exclusion)?;
        }
    }
    // Each is necessary: 2 reaches the newcomer and minimum-run edges, 6
    // starts without its own budget, and 125 fills the kept-answer table.
    for seed in [2, 6, 125] {
        eprintln!("bounds workload seed {seed}");
        let ledger = workload::run(Streams::new(seed), &configured, &sweep)?;
        eprintln!("bounds workload fell {seed}: {:?}", ledger.fell());
        sweep.ran(&ledger);
    }
    for exclusion in open::EXCLUSIONS {
        if configured.contains(&exclusion.fault) {
            eprintln!("{} left out {}", exclusion.scenario, sweep.left_out(exclusion.scenario));
        }
    }
    sweep.check_fell()
}
