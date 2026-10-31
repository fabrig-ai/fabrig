//! The inbox census builds the same projections through either start entrance.
use jig_core_world::{
    effects::{World, fixture},
    fixtures::{ask_as, chat, people_member},
};
use jig_domain_accounts as accounts;
use jig_domain_core::{
    self as core,
    start::{Action, Phase, SweepInput},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use std::collections::BTreeMap;

const SEED: u64 = 2401;

fn limits() -> root::Limits {
    let (_, mut limits) = fixture(SEED, false);
    limits.core.tasks.tasks = 4;
    limits.core.tasks.project_tasks = 4;
    limits.core.tasks.tree_tasks = 4;
    limits.core.tasks.funders = 8;
    limits.core.views.runs = 4;
    limits.core.fleet.slots = 2;
    limits.core.fleet.attempts = 4;
    jig_core_world::peers::journal(&mut limits);
    limits
}

fn configuration() -> root::Config {
    let (mut config, _) = fixture(SEED, false);
    for template in &mut config.core.defaults.templates {
        template.flags.retention = tasks::Retention::Kept;
    }
    config
}

fn finish(world: &mut World) -> u64 {
    let task = world.task();
    world.turn(1, 1, None, b"a committed reply");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: Box::from(b"done".as_slice()) },
            cancel_delegates: false,
        },
        1,
    );
    task
}

fn next_chat(world: &mut World, key: u8) -> u64 {
    let ask = chat(world, 1, Box::from(b"another conversation".as_slice()));
    ask_as(world, world.sign_in.expect("signed in owner"), key, ask);
    world.task()
}

fn stored_world() -> World {
    let limits = limits();
    let mut world = World::configured(SEED, false, configuration(), limits);
    let read = finish(&mut world);
    let sign_in = world.sign_in.expect("signed in owner");
    ask_as(
        &mut world,
        sign_in,
        101,
        people::Ask::MarkSeen { seen: people::Seen::Conversation { task: read, through: 1 } },
    );
    let _unread = next_chat(&mut world, 102);
    finish(&mut world);
    let target = next_chat(&mut world, 103);
    let key = world.call_key(1);
    let mut delegate =
        jig_core_world::fixtures::member(&world, tasks::Executor::Agent { charter: 1 }, b"ask the parent");
    delegate.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(900)),
        key,
        batch: Box::new([delegate]),
        stubs: Box::new([]),
    }));
    assert_ne!(world.task(), target, "delegate is assigned: {:?}", world.answers.last());
    let key = world.call_key(1);
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(901)),
        key,
        action: core::NamedAction::Message {
            target,
            kind: tasks::MessageKind::Question,
            words: Box::from(b"what next?".as_slice()),
        },
    }));
    assert!(
        matches!(world.answers.last(), Some((_, _, core::CallPart::Sent { .. }))),
        "message answer: {:?}",
        world.answers.last()
    );
    ask_as(
        &mut world,
        sign_in,
        105,
        people::Ask::Propose {
            project: 1,
            action: people::Action::Batch(Box::new([people_member(b"proposed work", 150)])),
            reason: Box::from(b"please approve".as_slice()),
        },
    );
    assert!(matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::Proposed { .. }))));
    world
}

fn phase(action: &Action, expected: Phase) {
    assert!(matches!(action, Action::Step(found) if *found == expected));
}

fn load_core(rows: &BTreeMap<root::Key, root::Record>, limits: &root::Limits, selected: bool) -> core::Core {
    let mut domain = core::Core::new(configuration().core, &limits.core).expect("configured direct core");
    let env = core_env(limits);
    if selected {
        phase(&domain.start_begin(false, env.wall), Phase::LoadCore);
    } else {
        assert_eq!(domain.restart_begin(false), core::RestartRequest::Step(core::RestartStep::LoadCore));
    }
    for range in core::RestoreRange::LIVE {
        if !selected && range == core::RestoreRange::Accounts {
            assert!(domain.settings_restored(&env));
        }
        for (key, record) in rows {
            if let root::Key::Core(key) = key
                && range.admits(key)
            {
                let root::Record::Core(record) = record else { panic!("jig live row") };
                load_record(&mut domain, limits, record.clone());
            }
        }
    }
    if selected {
        phase(&domain.start_done(&core_env(limits), Phase::LoadCore, &mut Queue::uncapped()), Phase::LoadInbox);
        assert!(!domain.people.ready());
        assert_eq!(domain.restart_step(), None);
    } else {
        let _out = core::step(&mut domain, &core_env(limits), core::Event::People(people::Event::Restored));
        assert_eq!(
            domain.restart_done(core::RestartStep::LoadCore),
            core::RestartRequest::Step(core::RestartStep::LoadInbox)
        );
    }
    domain
}

fn load_record(domain: &mut core::Core, limits: &root::Limits, record: core::Record) {
    let env = core_env(limits);
    match record {
        core::Record::Core(row) => assert_ne!(domain.restore_core(row, &limits.core), core::Restored::Rejected),
        core::Record::People(people::Stored::Policy { project, name, value }) => {
            assert!(domain.restore_policy(&limits.core, project, name, value));
        }
        core::Record::People(row) => {
            assert!(domain.restore_people_row(&row));
            let mut out = Queue::uncapped();
            people::step(
                &mut domain.people,
                &Env { now: env.now, wall: env.wall, limits: limits.core.people },
                people::Event::Restore { record: row },
                &mut out,
            );
            assert!(out.is_empty());
        }
        core::Record::Tasks(row) => {
            assert!(domain.restore_task_row(&row));
            let mut out = Queue::uncapped();
            tasks::step(
                &mut domain.tasks,
                &Env { now: env.now, wall: env.wall, limits: limits.core.tasks },
                tasks::Event::Restore { record: row },
                &mut out,
            );
            assert!(out.is_empty());
        }
        core::Record::Accounts(row) => {
            let mut out = Queue::uncapped();
            accounts::step(
                &mut domain.accounts,
                &Env { now: env.now, wall: env.wall, limits: limits.core.accounts },
                accounts::Event::Restore { record: row },
                &mut out,
            );
            assert!(out.is_empty());
        }
        core::Record::Notes(_) => panic!("no notes in live core ranges"),
    }
}

fn core_env(limits: &root::Limits) -> Env<core::Limits> {
    Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits: limits.core }
}

fn load_inbox(domain: &mut core::Core, rows: &BTreeMap<root::Key, root::Record>, limits: &root::Limits) {
    let census = domain.census(&limits.core);
    assert!(!census.questions.is_empty());
    assert!(!census.results.is_empty());
    let mut walk = Vec::new();
    for (key, record) in rows {
        if let root::Key::Core(key) = key
            && let Some(interval) = census.interval(key)
        {
            walk.push((interval, key.clone(), record));
        }
    }
    walk.sort_by_key(|(interval, key, _)| (*interval, key.clone()));
    let mut positions = 0;
    for (_, _, record) in walk {
        let root::Record::Core(record) = record else { panic!("jig census row") };
        match record {
            core::Record::People(people::Stored::EndedRead { task, .. }) => {
                positions += 1;
                assert!(domain.restore_inbox_row(&limits.core, record));
                let root::Record::Core(ended) = &rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(*task)))]
                else {
                    panic!("ended owner")
                };
                assert!(domain.restore_inbox_row(&limits.core, ended));
            }
            core::Record::Tasks(tasks::Stored::Result(result)) => {
                let root::Record::Core(ended) =
                    &rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(result.task)))]
                else {
                    panic!("result owner")
                };
                assert!(domain.restore_inbox_row(&limits.core, ended));
            }
            core::Record::Tasks(tasks::Stored::Activity(_)) => {
                assert!(domain.restore_inbox_row(&limits.core, record));
            }
            other @ (core::Record::Core(_)
            | core::Record::Tasks(_)
            | core::Record::People(_)
            | core::Record::Notes(_)
            | core::Record::Accounts(_)) => panic!("unexpected census row: {other:?}"),
        }
    }
    assert_eq!(positions, 2);
}

fn through_wires(domain: &mut core::Core, limits: &root::Limits) {
    let env = core_env(limits);
    let mut out = Queue::uncapped();
    let mut action = domain.start_done(&env, Phase::LoadInbox, &mut out);
    for _ in 0..32 {
        action = match action {
            Action::Step(Phase::AdoptRuns) => return,
            Action::Step(Phase::Sweep) => domain.start_sweep(&env, SweepInput::Finished, &mut out),
            Action::Step(done) => domain.start_done(&env, done, &mut out),
            Action::Commit(token) => domain.start_resume(&env, token, &mut out),
            other @ (Action::Idle | Action::Refused(_)) => panic!("opening projections: {other:?}"),
        };
        while out.pop().is_some() {}
    }
    panic!("bounded start finishes Wires");
}

#[test]
fn selected_census_matches_default_inbox_chats_and_unread_results_from_one_store() {
    let world = stored_world();
    let limits = limits();
    let mut default = load_core(&world.store.rows, &limits, false);
    let mut selected = load_core(&world.store.rows, &limits, true);
    load_inbox(&mut default, &world.store.rows, &limits);
    load_inbox(&mut selected, &world.store.rows, &limits);
    through_wires(&mut selected, &limits);
    assert!(selected.people.ready());
    let person = 1;
    let people_limits = &limits.core.people;
    assert_eq!(
        selected.people.cached_waiting(people_limits, person),
        default.people.cached_waiting(people_limits, person)
    );
    assert_eq!(
        selected.people.cached_updates(people_limits, person),
        default.people.cached_updates(people_limits, person)
    );
    assert_eq!(selected.people.cached_results(person), default.people.cached_results(person));
    assert_eq!(selected.people.counts(people_limits, person), default.people.counts(people_limits, person));
    assert!(selected.people.needs(person).expect("restored party needs") >= 2);
    assert_eq!(selected.people.cached_results(person).expect("restored party results").len(), 2);
    let subject = core::documents::Subject::Inbox { party: person };
    let window = core::documents::Window { rows: limits.core.documents.rows, after: None };
    let expected = core::documents::make(&default, &limits.core, person, &subject, &window, Wall::from_nanos(10))
        .expect("default inbox document");
    assert_eq!(
        core::documents::make(&selected, &limits.core, person, &subject, &window, Wall::from_nanos(10)),
        Ok(expected)
    );
    for record in world.store.rows.values() {
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) = record {
            assert_eq!(selected.people.chat_owner(row.number), default.people.chat_owner(row.number));
            assert_eq!(
                selected.people.conversation_seen(person, row.number),
                default.people.conversation_seen(person, row.number)
            );
        }
    }
}
