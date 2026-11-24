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

fn people_sweep(name: &'static str) -> Sweep {
    let mut sweep = Sweep::new(name);
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
    sweep
}

#[test]
fn people_act_on_goals_at_any_step() {
    let mut sweep = people_sweep("people_act_on_goals_at_any_step");
    // Keep the two draws covering the other acts. Seed 162's goal cancellation
    // races branch making and is excluded; seed 0 cancels after making settles.
    for seed in [0, 3, 162] {
        eprintln!("people_act_on_goals_at_any_step seed {seed}");
        let run = workload::run(&Streams::new(seed), &ACTS, &sweep).expect("world checks, Ends and act outcomes");
        sweep.ran(&run.ledger);
    }
    sweep.check_fell().expect("every configured act fell");
}

#[test]
fn a_restart_changes_no_outcome() {
    use temper_engine_domain_world::{forge_routes::restarts::Injector, outcome};
    use temper_faults::restarts::ENGINE;
    use temper_workload::{
        draw::{ENGINE as SHAPE, Workload},
        parties,
    };

    let mut sweep = Sweep::new("a_restart_changes_no_outcome");
    for kind in ENGINE {
        sweep.configure(kind);
    }
    temper_engine_forge_world::open::exclude(&mut sweep, &ENGINE).expect("engine exclusions");
    // Parties keep their existing narrow exclusions in both comparable runs.
    let parties_sweep = people_sweep("comparison parties");
    // Two start classes each carry a crash; the second signal carries a settled
    // stop. Three seeds are the minimum to cover those three distinct draws.
    for seed in [0, 2, 3] {
        eprintln!("a_restart_changes_no_outcome seed {seed}");
        let streams = Streams::new(seed);
        let work = Workload::draw(&streams, &SHAPE);
        let actions = parties::draw(&streams, &work, &ACTS);
        let before = workload::with_restarts(&streams, work.clone(), actions.clone(), &parties_sweep, None)
            .expect("without restarts");
        let before = outcome::read(&before.world.store, &before.main, &before.names, &before.workload, &before.ledger);
        let restarted =
            workload::with_restarts(&streams, work, actions, &parties_sweep, Some(Injector::new(seed, &ENGINE, 1000)))
                .expect("with restarts");
        let after = outcome::read(
            &restarted.world.store,
            &restarted.main,
            &restarted.names,
            &restarted.workload,
            &restarted.ledger,
        );
        eprintln!("seed {seed}: left {:?}; fell {:?}", restarted.left, restarted.ledger.fell());
        before.compare(&after, &restarted.left).unwrap_or_else(|finding| panic!("seed {seed}: {finding}"));
        sweep.ran(&restarted.ledger);
    }
    sweep.check_fell().expect("every ENGINE kind fell");
}
