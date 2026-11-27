//! Frozen counterexamples to the landing liveness invariant (engine-checks.md, 2.2).
use skein_lib::{Duration, Env, Queue, Time, Wall};
use skein_world::inspect::{Inspection, Moment};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change::{Cause, State};
use temper_engine_domain_forge_landing::{Leave, Phase};
use temper_engine_forge_world::{LIMITS, changes, landings};

const MOMENT: Moment = Moment { seed: 7, at: Time::ZERO, iteration: 3 };

fn change(task: u64, state: State) -> forge::ChangeRow {
    let mut row = changes::change_row();
    row.task = task;
    row.change.task = task;
    row.change.state = state;
    row.pull = Some(task);
    row
}

const READY: State = State::Ready { head: [1; 32], ready_since: Wall::EPOCH };

fn fixture(changes: Vec<forge::ChangeRow>, landings: Vec<forge::LandingRow>) -> forge::Domain {
    let mut domain = forge::Domain::new(
        &LIMITS,
        MOMENT.seed,
        forge::Config { push: None, namespace: b"checks".as_slice().into(), writers: Box::default() },
        &[],
        &[],
        false,
    )
    .expect("fixture bounds");
    let env = Env { limits: LIMITS, now: Time::ZERO, wall: Wall::EPOCH };
    let mut out = Queue::uncapped();
    domain.restore_begin(env.wall);
    for record in changes
        .into_iter()
        .map(|change| forge::Stored::Change(Box::new(change)))
        .chain(landings.into_iter().map(forge::Stored::Landing))
    {
        forge::step(&mut domain, &env, forge::Event::Restore { record }, &mut out);
    }
    domain.fit_restored(&LIMITS).expect("restored fixture fits");
    domain
}

fn finding(domain: &forge::Domain) -> Result<(), String> {
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::forge(domain, &mut inspection);
    inspection.check_violation(MOMENT)
}

#[test]
fn a_landing_cannot_keep_an_unavailable_member() {
    for (state, why) in [
        (None, "missing"),
        (Some(State::Ended), "ended"),
        (
            Some(State::Held {
                was: Box::new(READY),
                hold: temper_engine_domain_forge_change::Hold::new(Cause::Failed, Wall::EPOCH),
            }),
            "held",
        ),
    ] {
        let member = change(51, READY);
        let landing = landings::landing_row(&member);
        let domain = fixture(state.into_iter().map(|state| change(51, state)).collect(), vec![landing]);
        assert_eq!(
            finding(&domain),
            Err(format!(
                "seed 7: at 0 ns, iteration 3: engine task 61: a landing that can't move: member change 51 has a {why} task"
            ))
        );
    }
}

#[test]
fn a_landing_cannot_keep_a_member_still_releasing_after_effect_settlement() {
    let member = change(51, READY);
    let landing = landings::landing_row(&member);
    let mut domain = fixture(vec![member], vec![landing]);
    let record = forge::Stored::Release(forge::ReleaseRow {
        room: None,
        task: 51,
        root: 51,
        ending: forge::ReleaseEnding::Failed,
        close_pull: None,
        comment: None,
        pending: None,
        failed: false,
        effects_settled: true,
        releasing: true,
    });
    let env = Env { limits: LIMITS, now: Time::ZERO, wall: Wall::EPOCH };
    forge::step(&mut domain, &env, forge::Event::Restore { record }, &mut Queue::uncapped());
    assert!(finding(&domain).expect_err("closing member").ends_with("member change 51 has a closing task"));
}

#[test]
fn a_landing_cannot_keep_another_landings_member() {
    let member = change(51, READY);
    let older = landings::landing_row(&member);
    let mut newer = older.clone();
    newer.landing.task += 1;
    let domain = fixture(vec![member], vec![older, newer]);
    assert_eq!(domain.landing_of(51), Some(61));
    assert!(
        finding(&domain)
            .expect_err("member is claimed")
            .ends_with("task 62: a landing that can't move: member change 51 belongs to landing 61")
    );
}

#[test]
fn a_landing_cannot_use_a_train_that_another_holds() {
    let members = vec![change(51, READY), change(52, READY)];
    let older = landings::landing_row(&members[0]);
    let mut newer = landings::landing_row(&members[1]);
    newer.landing.task += 1;
    let domain = fixture(members, vec![older, newer]);
    assert_eq!(domain.landing_of(52), Some(62), "the train, not the member, is claimed");
    assert!(
        finding(&domain)
            .expect_err("another landing holds the train")
            .ends_with("task 62: a landing that can't move: landing 61 holds its train")
    );
}

#[test]
fn a_landing_due_to_step_is_checked_only_after_that_step() {
    let landing = landings::landing_row(&change(51, READY));
    let task = landing.landing.task;
    let domain = fixture(vec![], vec![landing]);
    let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
    temper_engine_inspect::forge_with_due(&domain, &mut inspection, |number| number == task);
    assert_eq!(inspection.check_violation(MOMENT), Ok(()));
    assert!(finding(&domain).is_err(), "the same row with no wake given is a violation");
}

#[test]
fn a_landing_gathering_its_steps_reads_is_not_checked_yet() {
    let mut world = landings::turn_world();
    world.produce(b"temper/c51");
    landings::ready_change(&mut world, 51, b"temper/c51");
    let row = landings::landing_row(world.change(51));
    let task = row.landing.task;
    landings::register(&mut world, row);
    world.slow_calls(Duration::from_secs(10));
    world.event(forge::Event::RelayRefused {
        task: 51,
        questions: world.change(51).questions.clone(),
        words: b"held member".as_slice().into(),
    });
    world.event(forge::Event::StepLanding {
        task,
        entry: 100,
        heard: forge::LandingHeard {
            job: None,
            outcomes: Box::default(),
            dependencies: Box::default(),
            held: Box::new([51]),
            ended: Box::default(),
            failure: Box::default(),
            preparation: None,
            kept: Box::default(),
            released: false,
            cancelled: false,
        },
    });
    world.run_for(1);
    assert!(matches!(world.change(51).change.state, State::Held { .. }));
    assert_eq!(world.landing(task).landing.phase, Phase::Waiting, "the gather has not decided yet");
    assert_eq!(world.inspection().check_violation(MOMENT), Ok(()));
}

#[test]
fn only_active_members_of_an_unheld_landing_count() {
    let member = change(51, READY);
    let waiting = landings::landing_row(&member);
    let mut held = waiting.clone();
    held.landing.phase =
        Phase::Held { why: temper_engine_domain_forge_landing::Hold::Read, was: Box::new(Phase::Waiting) };
    let mut left = waiting.clone();
    left.landing.members[0].left = Some(Leave::Cancelled);
    let mut landed = waiting;
    landed.landing.members[0].landed = Some([1; 32]);
    for row in [held, left, landed] {
        assert_eq!(finding(&fixture(vec![], vec![row])), Ok(()));
    }
}
