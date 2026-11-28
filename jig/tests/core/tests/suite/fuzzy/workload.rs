//! Workload sweeps (`jig/docs/design/domain/testing-workloads.md`, 7).
use jig_core_kit::{bounds, open, parks, people, workload};
use skein_world::{
    streams::Streams,
    sweep::{Sweep, cases},
    this_test,
};

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
    // and 29 answers an ended run and never answers. Together they strike every configured kind.
    cases(this_test!("people_act_on_tasks_at_any_step"), [1, 3, 29], |seed| -> Result<(), String> {
        let ledger = workload::run(Streams::new(seed), &configured, &sweep)?;
        sweep.ran(&ledger);
        Ok(())
    })
    .map_err(|case| case.to_string())?;
    sweep.check_fell()
}

#[test]
fn runs_park_resume_and_move_at_any_step() -> Result<(), String> {
    let configured = [
        parks::PARKS_WITH_A_QUESTION_OPEN,
        parks::ANSWERED_WHILE_PARKED,
        parks::ANSWERED_AFTER_ITS_END,
        parks::ANSWERED_TO_ITS_SUCCESSOR,
        parks::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS,
        parks::STUCK_IN_A_CIRCLE,
        parks::PARKS_ON_NOTHING,
        parks::WATCHES_A_TASK_THAT_CAN_ANSWER,
    ];
    let mut sweep = Sweep::new("runs_park_resume_and_move_at_any_step");
    for kind in configured {
        sweep.configure(kind);
    }
    // 11 covers the one-goal circle and the answer after ending; 61 covers
    // the two-goal circle by reading. 1763 parks on nothing and answers a
    // parked run; 3570 covers the control, successor and coordinator end.
    cases(this_test!("runs_park_resume_and_move_at_any_step"), [11, 61, 1763, 3570], |seed| -> Result<(), String> {
        let ledger = workload::run(Streams::new(seed), &configured, &sweep)?;
        sweep.ran(&ledger);
        Ok(())
    })
    .map_err(|case| case.to_string())?;
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
    // Each is necessary: 261 reaches all three funds edges, and 1731 fills
    // the kept-answer table with the goals' requests and person answers.
    cases(this_test!("the_cores_tables_and_funds_reach_their_bounds"), [261, 1731], |seed| -> Result<(), String> {
        let ledger = workload::run(Streams::new(seed), &configured, &sweep)?;
        sweep.ran(&ledger);
        Ok(())
    })
    .map_err(|case| case.to_string())?;
    for exclusion in open::EXCLUSIONS {
        if configured.contains(&exclusion.fault) {
            eprintln!("{} left out {}", exclusion.scenario, sweep.left_out(exclusion.scenario));
        }
    }
    sweep.check_fell()
}
