use jig_domain_people::{Ask, EndedBy, Holding, Lack, Outcome, Refusal, Reply, Role, TaskExecutor};
use jig_people_world::{Settings, World, fixtures::people_member};

#[test]
fn r14_writer_refusal_replays_after_explicit_start_without_routing_or_writing() {
    for malformed in [false, true] {
        let mut settings = Settings::calm(14);
        settings.authority_refuses = malformed;
        let mut world = World::new(settings);
        let signin = world.signin(10, 1);
        world.commit_all();
        let Reply::SignedIn { person, .. } = world.reply(signin).expect("durable sign-in replied") else {
            panic!("sign-in")
        };
        world.roles(1, Box::new([Holding { person, role: Role::Member }]));
        world.commit_all();
        let mut member = people_member(b"request", 1);
        if malformed {
            member.executor = TaskExecutor::Person(jig_domain_people::PersonAddress::Person(999));
            member.flags.ended_by = EndedBy::Requester;
            member.flags.conversation = true;
        }
        let ask = Ask::Create { project: 1, batch: Box::new([member]) };
        let call = world.request(10, [14; 16], ask.clone());
        world.commit_all();
        let reply = world.reply(call).expect("durable request replied");
        if malformed {
            assert_eq!(
                reply,
                Reply::Outcome(Outcome::Refused(Refusal::Authority { proposable: None, lacking: Lack::Shape }))
            );
            assert_eq!(world.tasks(), 0);
        }
        let routes = world.stats().routes;
        let tasks = world.tasks();
        let records = world.records().clone();
        let committed = world.commit_number();
        world.restart_start();
        let retried = world.request(10, [14; 16], ask.clone());
        world.commit_all();
        assert_eq!(world.reply(retried), Some(reply));
        assert_eq!(world.stats().routes, routes);
        assert_eq!(world.tasks(), tasks);
        assert_eq!(world.records(), &records);
        assert_eq!(world.commit_number(), committed, "replay writes no decision");
        let fresh = world.request(10, [15; 16], ask);
        world.commit_all();
        if malformed {
            assert_eq!(
                world.reply(fresh),
                Some(Reply::Outcome(Outcome::Refused(Refusal::Authority { proposable: None, lacking: Lack::Shape })))
            );
            assert_eq!(world.tasks(), 0, "fresh malformed admission still refuses");
        } else {
            assert_eq!(world.tasks(), tasks + 1, "fresh valid admission still creates");
        }
        assert_eq!(world.stats().routes, routes + 1);
        world.assert_settled();
    }
}

#[test]
fn explicit_world_runs_the_existing_scenario() {
    let mut world = World::configured_start(Settings::calm(4), Box::new([]), Box::new([]));
    world.run();
}

#[test]
fn explicit_start_one_row_pages_hold_no_expired_keys_as_the_range_grows() {
    use jig_domain_people::{Act, ActKey, Domain, Event, Request, RequestKey, Stored, step};
    use jig_people_world::LIMITS;
    use skein_lib::{Env, Queue, Time, Wall};
    use skein_world::domain::heap::Meter;
    let mut peaks = Vec::new();
    for count in [10_u64, 1000] {
        let meter = Meter::new();
        meter.start();
        {
            let limits = jig_domain_people::Limits { sign_ins: 1, requests: 1, ..LIMITS };
            let env = Env { limits, now: Time::ZERO, wall: Wall::from_nanos(limits.request_retention.as_nanos()) };
            let mut domain = Domain::new(&limits, Box::new([]), 1, Box::new([]));
            domain.start_restore(env.wall);
            let mut out = Queue::uncapped();
            for walk in 0..2 {
                if walk == 1 {
                    assert!(domain.start_link(&env));
                    domain.start_sweep();
                }
                for number in 0..count {
                    let mut key = [0; 16];
                    key[..8].copy_from_slice(&number.to_be_bytes());
                    for row in [
                        Stored::SignIn { number, person: 999, expires: env.wall },
                        Stored::Answer {
                            key: RequestKey { person: 999, key },
                            models: Box::new([]),
                            ask: Box::new(Ask::Stop { project: 0, task: 0 }),
                            outcome: Outcome::Stopped { task: 0 },
                            at: Wall::EPOCH,
                        },
                        Stored::Acted {
                            key: ActKey { connector: 0, key },
                            act: Act::RemoveAccount { account: 0 },
                            outcome: Outcome::AccountChanged,
                            at: Wall::EPOCH,
                        },
                    ] {
                        let key = row.key();
                        if walk == 0 {
                            step(&mut domain, &env, Event::Restore { record: row }, &mut out);
                            assert!(out.is_empty());
                        } else {
                            domain.start_sweep_row(&env, &row, &mut out);
                            assert_eq!(out.pop(), Some(Request::Erase { key }));
                        }
                    }
                }
            }
            assert!(domain.start_complete(&env));
            assert_eq!(domain.sign_in_person(0), None);
            assert!(!domain.has_project(0));
        }
        let peak = meter.end().peak();
        eprintln!("people_expired_one_row_pages count={count} heap_peak_bytes={peak}");
        peaks.push(peak);
    }
    assert_eq!(peaks[0], peaks[1], "expired rows consume no growing retained state");
}

#[test]
fn explicit_world_sweep_commits_each_page_without_skipping_erased_keys() {
    use jig_domain_people::Key;
    use jig_people_world::LIMITS;
    use skein_lib::Wall;
    let mut world = World::new(Settings::calm(18));
    let signed = world.signin(10, 1);
    world.commit_all();
    let Reply::SignedIn { person, .. } = world.reply(signed).expect("durable sign-in replied") else {
        panic!("sign-in")
    };
    world.roles(1, Box::new([Holding { person, role: Role::Member }]));
    world.commit_all();
    for number in 11..18 {
        world.signin(number, 1);
        world.commit_all();
    }
    for number in 1..=8 {
        world.ask(10, [number; 16], b"work");
        world.commit_all();
    }
    let routes = world.stats().routes;
    let committed = world.commit_number();
    world.set_wall(Wall::from_nanos(LIMITS.request_retention.as_nanos()));
    world.restart_start();
    assert_eq!(world.commit_number(), committed + 16, "eight sign-ins and eight answers, one durable page each");
    assert_eq!(world.stats().routes, routes);
    assert!(world.records().contains_key(&Key::Person(person)));
    for record in world.records().values() {
        assert!(!matches!(
            record,
            jig_domain_people::Stored::SignIn { .. }
                | jig_domain_people::Stored::Answer { .. }
                | jig_domain_people::Stored::Acted { .. }
        ));
    }
    world.assert_settled();
}

#[test]
fn selected_census_projects_needs_results_and_chats_without_opening() {
    use jig_domain_people::{self as p, RestoreProjection as P};
    use jig_people_world::{LIMITS, identity};
    use skein_lib::{Env, Queue, Time, Wall};
    for prefix in [0, 2] {
        let limits = p::Limits { inbox_entries: prefix, ..LIMITS };
        let env = Env { limits, now: Time::ZERO, wall: Wall::EPOCH };
        let mut domain = p::Domain::new(&limits, Box::new([]), 1, Box::new([]));
        assert!(domain.parties().is_empty());
        domain.start_restore(env.wall);
        let mut out = Queue::uncapped();
        for row in [
            p::Stored::Person { number: 1, identity: identity(1) },
            p::Stored::Roles { project: 1, holdings: Box::new([p::Holding { person: 1, role: p::Role::Member }]) },
            p::Stored::ReadResults { party: 1, through: 1 },
            p::Stored::ReadConversation { party: 1, task: 21, through: 1 },
            p::Stored::ReadConversation { party: 1, task: 22, through: 1 },
            p::Stored::SignIn { number: 1, person: 1, expires: Wall::from_nanos(1) },
        ] {
            p::step(&mut domain, &env, p::Event::Restore { record: row }, &mut out);
        }
        assert_eq!(&*domain.parties().into_boxed(), &[1]);
        assert_eq!(domain.results_seen(1), Some(1));
        assert_eq!(domain.results_seen(99), None);
        assert_eq!(domain.persons(), u32::MAX);
        for (task, kind) in [
            (11, p::EntryKind::PersonTask),
            (12, p::EntryKind::Question { message: 7 }),
            (13, p::EntryKind::Proposal { number: 1 }),
        ] {
            assert!(domain.start_project(
                &limits,
                P::Waiting {
                    task,
                    entries: Box::new([p::Entry { task, project: 1, whom: p::Whom::Person(1), kind, at: Wall::EPOCH }])
                }
            ));
        }
        assert_eq!(domain.needs(1), Some(3));
        assert_eq!(domain.question_person(12, 7), Some(1));
        let result = p::ResultRef {
            task: 22,
            project: 1,
            at: Wall::EPOCH,
            position: 2,
            forgettable: Some(p::Forgettable { label: 0, listed: false, turns_read: false, last_reply: Some(3) }),
        };
        assert!(domain.start_project(&limits, P::Result { party: 1, entry: result }));
        for (task, live, seen) in [(21, true, None), (22, false, Some(1))] {
            assert!(domain.start_project(
                &limits,
                P::ChatState {
                    party: 1,
                    project: 1,
                    task,
                    last_reply: 3,
                    live,
                    seen,
                    replies: Box::new([p::ReplyRef {
                        activity: 4,
                        entry: p::Entry {
                            task,
                            project: 1,
                            whom: p::Whom::Person(1),
                            kind: p::EntryKind::ChatReply { message: 3 },
                            at: Wall::EPOCH,
                        }
                    }]),
                }
            ));
        }
        assert_eq!(domain.chat_owner(21), Some(1));
        assert_eq!(domain.chat_owner(22), None);
        assert!(domain.start_project(&limits, P::EndedConversation { task: 22 }));
        assert!(domain.read_conversations().contains_key(&(1, 22)), "Link still needs its person witness");
        assert!(!domain.ready());
        assert_eq!(domain.next_deadline(), None);
        assert!(out.is_empty());
        let counts = domain.counts(&limits, 1).expect("loaded census");
        assert_eq!(counts.needs, 3);
        assert_eq!(counts.unread, prefix);
        assert!(counts.more_unread);
        assert!(domain.start_link(&env));
        assert!(!domain.read_conversations().contains_key(&(1, 22)));
        assert!(domain.read_conversations().contains_key(&(1, 21)));
        assert!(!domain.ready());
        domain.start_sweep();
        assert!(out.is_empty());
        assert_eq!(domain.next_deadline(), None);
        assert!(domain.start_complete(&env));
        assert!(domain.ready());
        assert_eq!(domain.cached_results(1), Some([result].as_slice()), "zero prefix retains result witness");
        assert_eq!(domain.needs(1), Some(3));
        assert_eq!(domain.next_deadline(), Some(Time::from_nanos(1)));
        assert_eq!(domain.restore_failure(), None);
        assert!(out.is_empty());
    }
}

#[test]
fn selected_historical_ended_owners_retain_only_loaded_read_position_witnesses() {
    use jig_domain_people::{Domain, Event, RestoreProjection, Stored, step};
    use jig_people_world::{LIMITS, identity};
    use skein_lib::{Env, Queue, Time, Wall};
    use skein_world::domain::heap::Meter;
    let mut peaks = Vec::new();
    for count in [10, 1000] {
        let meter = Meter::new();
        meter.start();
        {
            let limits = jig_domain_people::Limits { people: 1, inbox_tasks: 1, ..LIMITS };
            let env = Env { limits, now: Time::ZERO, wall: Wall::EPOCH };
            let mut domain = Domain::new(&limits, Box::new([]), 1, Box::new([]));
            domain.start_restore(env.wall);
            let mut out = Queue::uncapped();
            for row in [
                Stored::Person { number: 1, identity: identity(1) },
                Stored::ReadConversation { party: 1, task: 1, through: 0 },
            ] {
                step(&mut domain, &env, Event::Restore { record: row }, &mut out);
            }
            for task in 0..count {
                assert!(domain.start_project(&limits, RestoreProjection::EndedConversation { task }));
            }
            assert_eq!(domain.read_conversations().len(), 1);
            assert!(domain.start_link(&env));
            assert!(domain.read_conversations().is_empty());
            domain.start_sweep();
            assert!(domain.start_complete(&env));
            assert!(out.is_empty());
        }
        let peak = meter.end().peak();
        eprintln!("people_ended_witnesses count={count} heap_peak_bytes={peak}");
        peaks.push(peak);
    }
    assert_eq!(peaks[0], peaks[1], "historical owners retain no growing marker set");
}
