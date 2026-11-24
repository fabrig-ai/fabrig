//! People act throughout seeded goal work (`temper/docs/design/testing/workloads.md`, 8.1).
use skein_world::{
    streams::Streams,
    sweep::{Exclusion, Sweep},
};
use temper_engine_domain_world::{acts, workload};
use temper_workload::parties::Act;

const ACTS: [Act; 11] = [
    Act::CancelGoal,
    Act::CancelChange,
    Act::CancelLanding,
    Act::HoldMember,
    Act::ReleaseMember,
    Act::AmendPassedDeadline,
    Act::AnswerLate,
    Act::NeverAnswer,
    Act::AnswerTwice,
    Act::AnswerEndedRun,
    Act::WordsToEndedProducer,
];

#[test]
fn the_workload_ends_green_at_each_seed() {
    for seed in 0..=2 {
        workload::without_acts(&Streams::new(seed)).expect("world checks, Ends and final tier green");
    }
}

#[test]
fn people_act_on_goals_at_any_step() {
    let mut sweep = Sweep::new("people_act_on_goals_at_any_step");
    let kinds = ACTS.map(acts::kind);
    for kind in kinds {
        sweep.configure(kind);
    }
    temper_engine_forge_world::open::exclude(&mut sweep, &kinds).expect("engine exclusions");
    for exclusion in &jig_core_kit::open::EXCLUSIONS {
        if exclusion.fault == jig_core_kit::open::AMENDS_A_PASSED_DEADLINE {
            sweep
                .exclude(Exclusion::new(exclusion.scenario, exclusion.fixed_by, exclusion.fault, exclusion.when))
                .expect("S15");
        }
    }
    // These two seeds are the first pair found whose fired (not merely drawn)
    // acts cover all eleven kinds. A single examined draw cannot cover them all.
    for seed in [3, 162] {
        eprintln!("people_act_on_goals_at_any_step seed {seed}");
        let run = workload::run(&Streams::new(seed), &ACTS, &sweep).expect("world checks, Ends and act outcomes");
        sweep.ran(&run.ledger);
    }
    sweep.check_fell().expect("every configured act fell");
}
