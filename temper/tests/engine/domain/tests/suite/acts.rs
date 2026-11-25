//! Logical moments are read only from committed rows (`temper/docs/design/testing/workloads.md`, 4.1).
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use temper_engine_domain::Record;
use temper_engine_domain_world::{acts::Watch, store::Store};
use temper_workload::{
    draw::Party,
    names::Names,
    parties::{Act, Action, When},
};

fn record(number: u64, phase: tasks::Phase) -> Record {
    let mut row = jig_protocol_store_kit::task_record();
    row.number = number;
    row.phase = phase;
    Record::Core(core::Record::Tasks(if matches!(row.phase, tasks::Phase::Ended(_)) {
        tasks::Stored::Ended(Box::new(row))
    } else {
        tasks::Stored::Live(Box::new(row))
    }))
}

fn moment(store: &mut Store, when: When) {
    let phase = if when == When::DeadlinePassed || when == When::Held {
        tasks::Phase::Held {
            was: tasks::Was::Waiting,
            why: if when == When::Held { tasks::Hold::Stopped } else { tasks::Hold::Deadline },
            source: None,
        }
    } else {
        tasks::Phase::Active(tasks::Active::Idle)
    };
    let row = record(7, phase);
    store.rows.insert(row.key(), row);
    if when == When::Landing || when == When::Held {
        let mut change = temper_engine_forge_world::changes::change_row();
        change.task = 7;
        change.pull = Some(1);
        let landing = temper_engine_forge_world::landings::landing_row(&change);
        let task = record(landing.landing.task, tasks::Phase::Active(tasks::Active::Idle));
        store.rows.insert(task.key(), task);
        let row = Record::Forge { row: Box::new(temper_engine_domain_forge::Stored::Landing(landing)) };
        store.rows.insert(row.key(), row);
    }
    if when == When::Asked || when == When::ProducerEnded {
        let mut producer = jig_protocol_store_kit::task_record();
        producer.number = 8;
        producer.requester = tasks::Party::Task(7);
        producer.executor = tasks::Executor::Agent { charter: 2 };
        producer.contract = tasks::Contract::Change { connector: 1, kind: 2, words: 128, choices: Box::new([]) };
        producer.phase = tasks::Phase::Active(tasks::Active::Idle);
        if when == When::Asked {
            producer.questions = Box::new([tasks::QuestionCredit {
                number: 1,
                answerer: tasks::Party::Task(7),
                asked: tasks::Party::Task(7),
                line: 1,
                offer: None,
            }]);
        }
        let row = if when == When::ProducerEnded {
            producer.phase =
                tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::NoChange { words: Box::new([]) }));
            tasks::Stored::Ended(Box::new(producer))
        } else {
            tasks::Stored::Live(Box::new(producer))
        };
        let row = Record::Core(core::Record::Tasks(row));
        store.rows.insert(row.key(), row);
    }
}

#[test]
fn an_act_waits_for_its_committed_moment_and_steps_and_falls_once() {
    for when in
        [When::Started, When::Opened, When::Asked, When::Landing, When::Held, When::DeadlinePassed, When::ProducerEnded]
    {
        let action = Action { party: Party::First, act: Act::CancelGoal, target: "target".into(), when, after: 3 };
        let mut watch = Watch::new(vec![action.clone()]);
        let mut names = Names::new();
        let mut store = Store::new();
        moment(&mut store, when);
        assert_eq!(watch.due(&store, &names, 1), [], "an unnamed target has no moment");
        names.name("target", 7).expect("logical target");
        let rows = std::mem::take(&mut store.rows);
        assert_eq!(watch.due(&store, &names, 2), [], "no committed moment");
        store.rows = rows;
        assert_eq!(watch.due(&store, &names, 5), [], "first observation starts the steps");
        assert_eq!(watch.due(&store, &names, 7), [], "one step early");
        store.rows.clear();
        assert_eq!(watch.due(&store, &names, 8), [action], "a past moment is latched");
        moment(&mut store, when);
        assert_eq!(watch.due(&store, &names, 100), [], "one act, once");
    }
}
