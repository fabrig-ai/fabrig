//! One person's latch releases a goal's batch (temper/docs/design/testing/tiers.md, 1.1).
use super::{Key, Script, VecDeque, World, authority, call, engine, goals, parties, people, run, tasks};
use jig_domain_core::{documents, reads};
use serde_json::json;
use skein_lib::Wall;

const GOAL: u64 = 1;
const LATCH: u64 = 2;
const REPORTERS: [(u64, &str, &str); 2] = [(3, "@turn-latch-x", "X ended."), (4, "@turn-latch-y", "Y ended.")];
const LATCH_WORDS: &str = "Release the turn landing latch";
const ANSWER: &[u8] = b"Ready together";
const GOAL_REPORT: &str = "Both reports ended.";

fn config(seed: u64) -> engine::Config {
    goals::config_with_executors(seed, &[authority::Executor::Charter(1), authority::Executor::Role(0)])
}

fn limits(limits: &mut engine::Limits) {
    goals::limits(limits);
    limits.authority.executors = 2;
    limits.tasks.executor_kinds = 2;
    limits.documents.executors = 2;
}

fn planner() -> Script {
    let authority = |spend| json!({"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":spend},"notes":0});
    let mut batch = vec![json!({
        "executor":{"kind":"person","person":1},"spec":{"words":LATCH_WORDS},
        "contract":{"kind":"report","words":64},"authority":authority(0)
    })];
    for (_, cue, _) in REPORTERS {
        batch.push(json!({
            "executor":{"kind":"agent","charter":1},"spec":{"words":cue},
            "contract":{"kind":"report","words":64},"authority":authority(64),
            "dependencies":[{"kind":"batch","number":0}]
        }));
    }
    Script {
        cue: b"@turn-latch ".as_slice().into(),
        turns: Box::new([
            call(b"delegate", &serde_json::to_vec(&json!({"batch":batch})).expect("delegate JSON")),
            call(b"wait", b"{}"),
        ]),
    }
}

fn report(cue: &[u8], text: &str) -> Script {
    Script {
        cue: cue.into(),
        turns: Box::new([call(b"finish", &serde_json::to_vec(&json!({"report":text})).expect("report JSON"))]),
    }
}

fn start() -> World {
    let planner = planner();
    let mut world = World::configured_limits(961, planner.clone(), false, config, limits);
    world.scripts = VecDeque::from([planner]);
    world.scripts.extend(REPORTERS.map(|(_, cue, text)| report(cue.as_bytes(), text)));
    let user = temper_engine_domain_forge::user_key(1, 7);
    world.party = parties::Party::new(
        1,
        parties::Role::Owner,
        2000,
        Box::new([parties::Act::SignIn { provider: user.provider, subject: user.subject }]),
    );
    for _ in 0..super::STEPS {
        world.tick();
        if world.party.sign_in.is_some() {
            assert_eq!(world.party.person, Some(1), "the authenticated owner is the latch's assignee");
            let started = world.notes_act(
                goals::finite_goal(
                    b"@turn-latch Release two reports",
                    &[people::Executor::Charter(1), people::Executor::Role(0)],
                ),
                [41; 16],
            );
            assert!(
                matches!(started, people::Reply::Outcome(people::Outcome::Created { first: GOAL, last: GOAL, .. })),
                "start receipt: {started:?}"
            );
            park_goal(&mut world);
            return world;
        }
    }
    panic!("the owner did not sign in");
}

fn park_goal(world: &mut World) {
    for _ in 0..super::STEPS {
        world.tick();
        assert!(world.observed.assignments <= 1, "a reporter was assigned before the latch was answered");
        if world.observed.answers == 1 && world.observed.gone == 1 && world.engine_quiet() {
            assert_eq!(world.assignment().task, GOAL);
            assert!(matches!(world.agent().answer(), run::Answer::Parked { .. }));
            return;
        }
    }
    panic!("the goal did not park after delegating the batch");
}

fn task(world: &mut World, number: u64) -> Box<documents::Task> {
    let answer = world.notes_read(reads::Read::Task { task: number });
    let task = match answer {
        reads::Answer::Document(document) => {
            let documents::Document::Task(task) = *document else { panic!("live task {number} overview") };
            task
        }
        reads::Answer::Ended(ended) => {
            let reads::EndedValue::Task { document, .. } = ended.value else { panic!("ended task {number} overview") };
            document
        }
        other => panic!("task {number} read: {other:?}"),
    };
    assert_eq!(task.chip.number, number);
    task
}

fn dependency(task: &documents::Task, pending: bool) {
    let [fact] = task.dependency_facts.as_slice() else { panic!("one dependency: {task:?}") };
    assert_eq!((fact.number, fact.pending), (LATCH, pending));
}

fn ended_report(task: &documents::Task, expected: &[u8]) -> Wall {
    assert_eq!(task.chip.phase, documents::PhaseLine::Ended);
    let Some(documents::Ending::Done(documents::ResultValue::Report(text))) = &task.result else {
        panic!("report from {}: {:?}", task.chip.number, task.result);
    };
    assert!(!text.cut);
    assert_eq!(text.bytes.as_ref(), expected);
    task.chip.details.ended_at.expect("the report's ending time")
}

#[test]
fn one_person_answer_releases_both_latched_reports() {
    let mut world = start();
    let goal = task(&mut world, GOAL);
    assert_eq!(goal.chip.phase, documents::PhaseLine::Active(tasks::Active::Idle));
    let delegates: Vec<_> = goal.delegates.iter().map(|delegate| delegate.chip.number).collect();
    assert_eq!(delegates, [LATCH, REPORTERS[0].0, REPORTERS[1].0]);
    for (number, cue, _) in REPORTERS {
        let before = task(&mut world, number);
        assert_eq!(before.words.bytes.as_ref(), cue.as_bytes());
        let documents::PhaseLine::Waiting { dependencies, more: false, .. } = &before.chip.phase else {
            panic!("reporter {number} waiting: {:?}", before.chip.phase);
        };
        assert_eq!(dependencies.as_slice(), &[LATCH]);
        dependency(&before, true);
        assert_eq!(before.chip.details.run_since, None);
        assert_eq!(before.chip.details.attempt, None);
    }
    let latch = task(&mut world, LATCH);
    assert_eq!(latch.words.bytes.as_ref(), LATCH_WORDS.as_bytes());
    let owner = world.party.person.expect("authenticated owner");
    assert_eq!(latch.executor, tasks::Executor::Person(tasks::PersonAddress::Person(owner)));
    assert_eq!(latch.contract, documents::ContractLine::Report { words: 64 });
    assert_eq!(latch.person_revision, Some(0));
    assert_eq!(world.observed.assignments, 1, "only the parked goal was assigned before the answer");

    // Prepare the parked goal's next completion, but let the last result wake it.
    world.notes_follow_up(report(b"@turn-latch ", GOAL_REPORT));
    assert_eq!(
        world.notes_act(
            people::Ask::AnswerPerson {
                project: 1,
                task: LATCH,
                revision: latch.person_revision.expect("the revision read by the owner"),
                result: people::PersonResult::Report { words: ANSWER.into() },
            },
            [42; 16],
        ),
        people::Reply::Outcome(people::Outcome::PersonAnswered { task: LATCH }),
    );
    let goal_ended = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(GOAL)));
    for _ in 0..super::STEPS {
        if world.store.rows.contains_key(&goal_ended) {
            break;
        }
        world.tick();
    }
    assert!(world.store.rows.contains_key(&goal_ended), "the last result wakes the goal to finish");
    let released = ended_report(&task(&mut world, LATCH), ANSWER);
    let mut last_report = released;
    for (number, _, expected) in REPORTERS {
        let done = task(&mut world, number);
        let ended = ended_report(&done, expected.as_bytes());
        assert!(ended >= released, "reporter {number} ended before release");
        last_report = last_report.max(ended);
        dependency(&done, false);
        assert!(
            done.chip.details.run_since.expect("report run start") >= released,
            "reporter {number} started before release"
        );
    }
    let ended = ended_report(&task(&mut world, GOAL), GOAL_REPORT.as_bytes());
    assert!(ended >= last_report, "the goal reports after both reporters end");
    assert_eq!(world.observed.assignments, 4, "goal, two reporters, then resumed goal");
    assert_only_creation_and_answer(&world);
}

fn assert_only_creation_and_answer(world: &World) {
    let mut outcomes = world.store.rows.values().filter_map(|row| match row {
        engine::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. })) => Some(outcome),
        _ => None,
    });
    assert!(matches!(outcomes.next(), Some(people::Outcome::Created { first: GOAL, last: GOAL, .. })));
    assert_eq!(outcomes.next(), Some(&people::Outcome::PersonAnswered { task: LATCH }));
    assert_eq!(outcomes.next(), None, "no other release, amendment or act");
}
