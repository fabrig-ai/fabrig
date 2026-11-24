//! Workload sweeps (`jig/docs/design/domain/testing-workloads.md`, 7).
use jig_core_kit::{open, people, workload};
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
