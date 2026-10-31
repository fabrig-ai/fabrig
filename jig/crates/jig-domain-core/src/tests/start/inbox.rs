//! Census loading before people's durable opening (domain/engine.md, 6, 11.4).
use super::{Action, Core, Env, Failure, Phase, Queue, Time, Wall, config, limits, people, step_phase, tasks};
use crate::tests::fixtures;
use crate::{CoreRecord, Limits, Record, RestartRequest, RestartStep, Restored, start::LiveSet};
use skein_lib::Map;

fn loaded(selected: bool) -> (Core, Env<Limits>) {
    let limits = limits();
    let mut core = Core::new(config(401), &limits).expect("configured core");
    let env = Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits };
    if selected {
        step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    } else {
        assert_eq!(core.restart_begin(false), RestartRequest::Step(RestartStep::LoadCore));
    }
    let mut header = core.counters.deployment();
    header.commits = 1;
    header.tasks = 100;
    header.runs = 100;
    header.people = 100;
    header.messages = 100;
    assert_eq!(core.restore_core(CoreRecord::Deployment(header), &limits), Restored::Deployment { commits: 1 });
    let mut out = Queue::uncapped();
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        let _out = crate::step(&mut core, &env, crate::Event::People(people::Event::Restore { record }));
    }
    if !selected {
        people::step(
            &mut core.people,
            &Env { now: env.now, wall: env.wall, limits: limits.people },
            people::Event::Restored,
            &mut out,
        );
    }
    (core, env)
}

fn inbox(core: &mut Core, env: &Env<Limits>, selected: bool) {
    if selected {
        step_phase(core.start_done(env, Phase::LoadCore, &mut Queue::uncapped()), Phase::LoadInbox);
        assert_eq!(core.restart_step(), None);
        assert!(!core.people.ready());
    } else {
        assert_eq!(core.restart_done(RestartStep::LoadCore), RestartRequest::Step(RestartStep::LoadInbox));
    }
    let census = core.census(&env.limits);
    assert_eq!(census.results.as_ref(), [crate::ReadResults { party: 1, after: 0 }]);
}

fn ended() -> tasks::TaskRecord {
    let mut row = fixtures::ended_report(2, tasks::Party::Person(1));
    row.last_reply = 0;
    row.replies = Box::new([]);
    row
}

#[test]
fn selected_census_uses_its_phase_and_clears_progress_when_done() {
    let (mut core, env) = loaded(true);
    inbox(&mut core, &env, true);
    assert!(core.restore_inbox_row(&env.limits, &Record::Tasks(tasks::Stored::Ended(Box::new(ended())))));
    assert_eq!(core.people.cached_updates(&env.limits.people, 1).unwrap().len(), 1);
    let _next = core.start_done(&env, Phase::LoadInbox, &mut Queue::uncapped());
    assert!(core.census_progress.is_empty());
}

#[test]
fn row_rules_do_not_refuse_selected_loading() {
    let mut past_counter = ended();
    past_counter.result_position = 101;
    let mut header = ended();
    header.number = 101;
    let mut phase = ended();
    phase.phase = tasks::Phase::Waiting;
    let mut unknown = ended();
    unknown.requester = tasks::Party::Person(99);
    let mut no_time = ended();
    no_time.ended_at = None;
    let mut zero_position = ended();
    zero_position.result_position = 0;
    for row in [past_counter, header, phase, unknown, no_time, zero_position] {
        for selected in [false, true] {
            let (mut core, env) = loaded(selected);
            inbox(&mut core, &env, selected);
            assert_eq!(
                core.restore_inbox_row(&env.limits, &Record::Tasks(tasks::Stored::Ended(Box::new(row.clone())))),
                selected
            );
            assert_eq!(core.start_failure(), None);
        }
    }
}

#[test]
fn question_storage_full_names_the_activity_key() {
    let (mut core, env) = loaded(true);
    inbox(&mut core, &env, true);
    core.question_activity = Map::with_capacity(0);
    let line = tasks::ActivityLine {
        task: 1,
        position: 3,
        at: Wall::EPOCH,
        line: tasks::Line::Words {
            message: 2,
            by: tasks::Party::Person(1),
            kind: tasks::MessageKind::Question,
            words: Box::new([]),
        },
    };
    let record = Record::Tasks(tasks::Stored::Activity(line));
    assert!(!core.restore_inbox_row(&env.limits, &record));
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet { set: LiveSet::Census, key: record.key(), held: 1, limit: 0 })
    );
}

#[test]
#[should_panic(expected = "People(EndedRead { task: 3, party: 1 }) links mismatched Tasks(Ended(2))")]
fn ended_read_pair_mismatch_panics_with_both_keys() {
    let (mut core, env) = loaded(true);
    inbox(&mut core, &env, true);
    assert!(
        core.restore_inbox_row(
            &env.limits,
            &Record::People(people::Stored::EndedRead { task: 3, party: 1, through: 0 })
        )
    );
    let _accepted = core.restore_inbox_row(&env.limits, &Record::Tasks(tasks::Stored::Ended(Box::new(ended()))));
}

#[test]
fn ended_conversation_witness_is_retired_at_link() {
    let (mut core, env) = loaded(true);
    let _out = crate::step(
        &mut core,
        &env,
        crate::Event::People(people::Event::Restore {
            record: people::Stored::ReadConversation { party: 1, task: 2, through: 0 },
        }),
    );
    inbox(&mut core, &env, true);
    assert!(
        core.restore_inbox_row(
            &env.limits,
            &Record::People(people::Stored::EndedRead { task: 2, party: 1, through: 0 })
        )
    );
    assert!(core.restore_inbox_row(&env.limits, &Record::Tasks(tasks::Stored::Ended(Box::new(ended())))));
    assert_eq!(core.people.read_conversations().len(), 1);
    assert!(core.people.start_link(&Env { now: env.now, wall: env.wall, limits: env.limits.people }));
    assert!(core.people.read_conversations().is_empty());
}

#[test]
#[should_panic(expected = "ReadConversation { party: 1, task: 99 }) links missing Tasks(Live(99))")]
fn missing_conversation_panics_at_link_with_the_read_key() {
    let (mut core, env) = loaded(true);
    let _out = crate::step(
        &mut core,
        &env,
        crate::Event::People(people::Event::Restore {
            record: people::Stored::ReadConversation { party: 1, task: 99, through: 0 },
        }),
    );
    inbox(&mut core, &env, true);
    let mut action = core.start_done(&env, Phase::LoadInbox, &mut Queue::uncapped());
    while let Action::Step(phase) = action {
        action = core.start_done(&env, phase, &mut Queue::uncapped());
    }
}

#[test]
fn census_party_and_read_result_sets_name_the_person_record() {
    for census_table in [true, false] {
        let set = if census_table { LiveSet::Census } else { LiveSet::ReadResults };
        let (mut core, mut env) = loaded(true);
        step_phase(core.start_done(&env, Phase::LoadCore, &mut Queue::uncapped()), Phase::LoadInbox);
        if census_table {
            core.census_progress = Map::with_capacity(0);
        } else {
            env.limits.people.people = 0;
        }
        let census = core.census(&env.limits);
        assert!(census.results.is_empty());
        assert_eq!(
            core.start_failure(),
            Some(Failure::LiveSet { set, key: crate::Key::People(people::Key::Person(1)), held: 1, limit: 0 })
        );
    }
}

#[test]
fn archive_capacity_refuses_with_the_ended_record_key() {
    let (mut core, env) = loaded(true);
    inbox(&mut core, &env, true);
    core.archived_chats = Map::with_capacity(0);
    let record = Record::Tasks(tasks::Stored::Ended(Box::new(ended())));
    assert!(!core.restore_inbox_row(&env.limits, &record));
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet { set: LiveSet::Conversations, key: record.key(), held: 1, limit: 0 })
    );
}

#[test]
fn unknown_requester_does_not_take_a_census_party_slot() {
    let (mut core, env) = loaded(true);
    inbox(&mut core, &env, true);
    let occupied = core.census_progress.len();
    let mut row = ended();
    row.requester = tasks::Party::Person(99);
    assert!(core.restore_inbox_row(&env.limits, &Record::Tasks(tasks::Stored::Ended(Box::new(row)))));
    assert_eq!(core.census_progress.len(), occupied);
    assert_eq!(core.start_failure(), None);
}

#[test]
fn question_membership_is_supplied_by_the_selected_census_index() {
    for selected in [false, true] {
        let (mut core, env) = loaded(selected);
        let (source, _) = crate::tests::watch_core();
        let row = source.tasks.task(1).unwrap().clone();
        let mut out = Queue::uncapped();
        tasks::step(
            &mut core.tasks,
            &Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) },
            &mut out,
        );
        assert!(out.is_empty());
        inbox(&mut core, &env, selected);
        let record = Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
            task: 1,
            position: 3,
            at: Wall::EPOCH,
            line: tasks::Line::Words {
                message: 2,
                by: tasks::Party::Person(1),
                kind: tasks::MessageKind::Words,
                words: Box::new([]),
            },
        }));
        assert_eq!(core.restore_inbox_row(&env.limits, &record), selected);
        assert_eq!(core.start_failure(), None);
    }
}

#[test]
fn a_full_question_list_names_its_live_task_record() {
    let (mut core, mut env) = loaded(true);
    let (source, _) = crate::tests::watch_core();
    let mut row = source.tasks.task(1).unwrap().clone();
    row.question_lines = Box::new([tasks::QuestionLine { message: 2, line: 3, answer: None }]);
    let mut out = Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
        tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) },
        &mut out,
    );
    assert!(out.is_empty());
    step_phase(core.start_done(&env, Phase::LoadCore, &mut Queue::uncapped()), Phase::LoadInbox);
    env.limits.tasks.tasks = 0;
    let census = core.census(&env.limits);
    assert!(census.questions.is_empty());
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet { set: LiveSet::Census, key: crate::Key::Tasks(tasks::Key::Live(1)), held: 1, limit: 0 })
    );
}
