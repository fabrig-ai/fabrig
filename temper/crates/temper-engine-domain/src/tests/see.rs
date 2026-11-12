//! Address grammar in the chat's project (domain/agent/ask-temper.md, 1).
use crate::route::see::{Address, address};
use jig_domain_core::settings::Setting;
use jig_domain_people::Proposer;

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks each address grammar production")]
fn addresses_parse_in_the_chats_project() {
    for (line, expected) in [
        ("settings", Address::Inventory),
        ("settings charter 1", Address::Setting(Setting::Charter(1))),
        ("settings template 5", Address::Setting(Setting::Template(5))),
        ("settings policy 1", Address::Setting(Setting::Policy(1))),
        ("settings model 1", Address::Setting(Setting::Model(1))),
        ("settings rules", Address::Setting(Setting::Rules)),
        ("settings landing-rule 1", Address::Setting(Setting::Application { kind: 1, scope: 0, number: 1 })),
        ("settings landing-rule 3 project", Address::Setting(Setting::Application { kind: 2, scope: 7, number: 3 })),
        ("settings forge-charter 1", Address::Setting(Setting::Application { kind: 3, scope: 0, number: 1 })),
        ("settings project-areas", Address::Setting(Setting::Application { kind: 12, scope: 7, number: 1 })),
        ("settings project-triager", Address::Setting(Setting::Application { kind: 13, scope: 7, number: 1 })),
        ("settings template-goal 4", Address::Setting(Setting::Application { kind: 11, scope: 0, number: 4 })),
        ("settings project-ways", Address::Setting(Setting::Application { kind: 7, scope: 7, number: 1 })),
        ("settings project-questions", Address::Setting(Setting::Application { kind: 6, scope: 7, number: 1 })),
        ("settings landing", Address::Landing),
        ("settings gates qa/pilot", Address::Gates { repository: b"qa/pilot".as_slice().into() }),
        ("revisions", Address::Revisions),
        ("revision 2", Address::Revision { number: 2 }),
        ("proposal t4/12", Address::Proposal { proposer: Proposer::Task(4), number: 12 }),
        ("proposal p2/9", Address::Proposal { proposer: Proposer::Party(2), number: 9 }),
        ("help", Address::Page { name: b"index".as_slice().into() }),
        ("help settings", Address::Page { name: b"settings".as_slice().into() }),
        ("help search landing  rule", Address::Search { words: b"landing rule".as_slice().into() }),
        (
            "file qa/pilot README.md",
            Address::File {
                repository: b"qa/pilot".as_slice().into(),
                path: b"README.md".as_slice().into(),
                branch: None,
            },
        ),
        (
            "file qa/pilot README.md main",
            Address::File {
                repository: b"qa/pilot".as_slice().into(),
                path: b"README.md".as_slice().into(),
                branch: Some(b"main".as_slice().into()),
            },
        ),
        (" \tsettings  charter 1\n", Address::Setting(Setting::Charter(1))),
        ("me", Address::Me),
        ("project", Address::Project),
        ("goals", Address::Goals { ended: false }),
        ("goals ended", Address::Goals { ended: true }),
        ("tasks", Address::Tasks),
        ("task 41", Address::Task { number: 41 }),
        ("task 41 tree", Address::Tree { number: 41 }),
        ("task 41 activity", Address::Activity { number: 41 }),
        ("task 41 turn 1.2", Address::Turn { number: 41, attempt: 1, turn: 2 }),
        ("change 1", Address::Change { number: 1 }),
        ("changes", Address::Changes),
        ("inbox", Address::Inbox),
        ("ways", Address::Ways),
        ("Settings", Address::Unknown),
        ("settings charter", Address::Unknown),
        ("settings charter 0", Address::Unknown),
        ("settings charter x", Address::Unknown),
        ("settings charter 4294967296", Address::Unknown),
        ("revision", Address::Unknown),
        ("revision 0", Address::Unknown),
        ("help Bad_Name", Address::Unknown),
        ("help search", Address::Unknown),
        ("file qa/pilot", Address::Unknown),
        ("file qapilot README.md", Address::Unknown),
        ("file a/b/c x", Address::Unknown),
        ("proposal x4/12", Address::Unknown),
        ("proposal t0/12", Address::Unknown),
        ("proposal p2/0", Address::Unknown),
        ("more a@1", Address::Unknown),
        ("task 0", Address::Unknown),
        ("task x", Address::Unknown),
        ("task 41 turn 1", Address::Unknown),
        ("task 41 turn 0.2", Address::Unknown),
        ("task 41 turn 1.0", Address::Unknown),
        ("task 41 turn 1.2.3", Address::Unknown),
        ("task 41 tree now", Address::Unknown),
        ("goals done", Address::Unknown),
        ("tasks 3", Address::Unknown),
        ("me now", Address::Unknown),
        ("change 18446744073709551616", Address::Unknown),
        ("inbox all", Address::Unknown),
        ("task 18446744073709551616", Address::Unknown),
        ("task 41 turn 18446744073709551616.1", Address::Unknown),
        ("task 41 turn 1.4294967296", Address::Unknown),
        ("change 0", Address::Unknown),
        ("change", Address::Unknown),
        ("task", Address::Unknown),
        ("task 41 turn", Address::Unknown),
        ("task 41 turn 1.x", Address::Unknown),
        ("", Address::Unknown),
        (" \t\n", Address::Unknown),
        ("change 4294967296", Address::Change { number: 4_294_967_296 }),
        ("change 18446744073709551615", Address::Change { number: u64::MAX }),
        ("task 18446744073709551615", Address::Task { number: u64::MAX }),
        (
            "task 41 turn 18446744073709551615.4294967295",
            Address::Turn { number: 41, attempt: u64::MAX, turn: u32::MAX },
        ),
    ] {
        assert_eq!(address(line.as_bytes(), 7), expected, "{line}");
    }
}

use crate::route::see::person::{self, Gathered, Part, Step};
use jig_domain_core::{
    documents as d,
    reads::{Answer, Read, Why},
};
use skein_lib::{List, Token};

fn task_query(order: d::QueryOrder, phase: Option<d::PhaseClass>, parent: Option<u64>, tracked: Option<bool>) -> Read {
    Read::Tasks {
        query: d::TaskQuery {
            project: Some(7),
            requester: None,
            root: None,
            parent,
            label: None,
            phase,
            executor: None,
            tracked,
            order,
        },
        live: None,
        ended: None,
    }
}

fn needs_read() -> Read {
    Read::Inbox {
        group: jig_domain_core::reads::Group::Needs,
        filter: jig_domain_core::reads::Filter { project: Some(7), goal: None },
        from: None,
    }
}

fn application_read(subject: u64) -> Read {
    Read::Application {
        scope: jig_domain_people::ApplicationScope::Project(7),
        connector: 3,
        read: Token::new(subject),
        window: d::Window { rows: 0, after: None },
    }
}

/// A task document from the existing step fixture, without a world or a sign-in.
fn task_answer(tracked: bool) -> Answer {
    let (mut domain, env, number) = super::live_task(
        if tracked { Some(1) } else { None },
        jig_domain_tasks::Executor::Procedure { connector: 0, code: 1 },
    );
    domain.core.people = jig_domain_people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
    domain.core.people.start_restore(env.wall);
    let mut out = skein_lib::Queue::uncapped();
    for record in [
        jig_domain_people::Stored::Person {
            number: 1,
            identity: jig_domain_people::Identity {
                key: jig_domain_people::IdentityKey { provider: 0, subject: b"reader".as_slice().into() },
                login: b"reader".as_slice().into(),
                name: Box::new([]),
            },
        },
        jig_domain_people::Stored::Roles {
            project: 1,
            holdings: Box::new([jig_domain_people::Holding { person: 1, role: jig_domain_people::Role::Owner }]),
        },
    ] {
        jig_domain_people::step(
            &mut domain.core.people,
            &env.with(env.limits.people),
            jig_domain_people::Event::Restore { record },
            &mut out,
        );
    }
    assert!(domain.core.people.start_link());
    domain.core.people.start_sweep();
    assert!(domain.core.people.start_complete(&env.with(env.limits.people)));
    let document = d::task_from_row(
        &domain.core,
        &crate::limits::core_limits(&env.limits),
        1,
        domain.core.tasks.task(number).unwrap(),
        env.wall,
    )
    .unwrap();
    Answer::Document(Box::new(d::Document::Task(Box::new(document))))
}

fn read_answer(read: &Read, tracked: bool) -> Part {
    if let Read::Task { .. } = read {
        Part::Jig(task_answer(tracked))
    } else if let Read::Inbox { .. } = read {
        Part::Jig(question_page())
    } else if let Read::Question { .. } = read {
        Part::Jig(Answer::None(Why::Unknown))
    } else if let Read::Application { .. } = read {
        Part::Forge(Some(Box::new(temper_engine_domain_forge::Document::Landings(
            temper_engine_domain_forge::LandingsView { project: 7, turn: None, landings: Box::new([]) },
        ))))
    } else {
        Part::Jig(Answer::Document(Box::new(d::Document::Chips(List::with_capacity(0)))))
    }
}

fn question_page() -> Answer {
    let mut rows = List::uncapped();
    for (task, object) in [
        (41, d::InboxObject::Question { message: 9 }),
        (41, d::InboxObject::Proposal { proposer: Proposer::Task(41), number: 1 }),
        (42, d::InboxObject::Question { message: 3 }),
    ] {
        rows.push(jig_domain_core::reads::PageRow::Inbox(d::InboxEntry {
            reply: None,
            task: d::TaskLink::Stored { number: task, status: None },
            project: Some(7),
            object,
            activity: None,
            at: Some(skein_lib::Wall::EPOCH),
            unread: None,
            cursor: d::Cursor::Inbox { task, object, at: Some(skein_lib::Wall::EPOCH) },
            projects: List::uncapped(),
        }))
        .unwrap();
    }
    Answer::Page(Box::new(jig_domain_core::reads::Page {
        rows,
        before: None,
        next: None,
        cut: false,
        inbox_selection: None,
    }))
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "one ordinary test table owns each address's read list and pins its keys, order and refusal"
)]
fn person_addresses_choose_each_read_in_order_and_keep_later_refusals() {
    use crate::SeeNotRead;
    use d::{PhaseClass as P, QueryOrder as O};
    for (line, tracked, reads) in [
        (
            "me",
            false,
            vec![
                Read::Project { project: 7 },
                needs_read(),
                Read::Question { task: 41, message: 9 },
                Read::Question { task: 42, message: 3 },
            ],
        ),
        ("project", false, vec![Read::Project { project: 7 }, task_query(O::Number, Some(P::Held), None, None)]),
        ("goals", false, vec![task_query(O::Priority, None, None, Some(true)), Read::Notes { project: 7, from: None }]),
        (
            "goals ended",
            false,
            vec![task_query(O::Ended, Some(P::Ended), None, Some(true)), Read::Notes { project: 7, from: None }],
        ),
        ("tasks", false, vec![task_query(O::Activity, None, None, None)]),
        ("task 41", false, vec![Read::Task { task: 41 }]),
        (
            "task 41",
            true,
            vec![
                Read::Task { task: 41 },
                Read::ScopedNotes {
                    scope: jig_domain_notes::Scope::Goal { project: 7, goal: 41 },
                    description: None,
                    from: None,
                },
            ],
        ),
        (
            "task 41 tree",
            false,
            vec![task_query(O::Number, None, Some(41), None), task_query(O::Number, Some(P::Ended), Some(41), None)],
        ),
        ("task 41 activity", false, vec![Read::Activity { task: 41, before: None }]),
        ("task 41 turn 2.3", false, vec![Read::Turn { task: 41, attempt: 2, turn: 3 }]),
        ("change 41", false, vec![application_read(41)]),
        (
            "changes",
            false,
            vec![
                application_read(temper_engine_domain_forge::changes_subject(7)),
                application_read(temper_engine_domain_forge::landings_subject(7, false)),
            ],
        ),
        (
            "inbox",
            false,
            vec![needs_read(), Read::Question { task: 41, message: 9 }, Read::Question { task: 42, message: 3 }],
        ),
    ] {
        let address = address(line.as_bytes(), 7);
        let mut gathered = Gathered::new(3);
        for (index, read) in reads.iter().enumerate() {
            assert_eq!(person::next(&address, 7, &gathered), Step::Read(read.clone()), "{line}, read {index}");
            let answer = read_answer(read, tracked);
            gathered.push(&address, answer);
        }
        assert_eq!(person::next(&address, 7, &gathered), Step::Deliver(None), "{line}");
        for (why, expected) in [(Why::Unknown, SeeNotRead::Unknown), (Why::Busy, SeeNotRead::Busy)] {
            let refused = Gathered {
                connector: 3,
                first: Some(Part::Jig(Answer::None(why))),
                second: None,
                words: List::uncapped(),
            };
            assert_eq!(person::next(&address, 7, &refused), Step::Deliver(Some(expected)), "{line}");
            if gathered.second.is_some() {
                gathered.second = Some(Part::Jig(Answer::None(why)));
                assert_eq!(person::next(&address, 7, &gathered), Step::Deliver(None), "{line}, later {why:?}");
            }
        }
        if line == "task 41 turn 2.3" {
            let sized = Gathered {
                connector: 3,
                first: Some(Part::Jig(Answer::None(Why::TooLarge { bytes: 8192 }))),
                second: None,
                words: List::uncapped(),
            };
            assert_eq!(person::next(&address, 7, &sized), Step::Deliver(None), "a turn keeps its size refusal");
        }
        if let Some(Read::Application { .. }) = reads.first() {
            let missing =
                Gathered { connector: 3, first: Some(Part::Forge(None)), second: None, words: List::uncapped() };
            assert_eq!(person::next(&address, 7, &missing), Step::Deliver(Some(SeeNotRead::Unknown)), "{line}");
        }
    }
}

fn chat(requester: jig_domain_tasks::Party) -> (crate::Domain, skein_lib::Env<crate::Limits>, crate::CallKey) {
    use jig_domain_core::Record as CoreRecord;
    use jig_domain_people as people;
    use jig_domain_tasks as tasks;
    let (domain, env, number) = super::live_task(None, tasks::Executor::Procedure { connector: 0, code: 1 });
    let mut row = domain.core.tasks.task(number).unwrap().clone();
    row.requester = requester;
    let ledger = *domain.core.tasks.funding(row.funder).unwrap();
    let parent = if let tasks::Party::Task(number) = requester {
        let mut parent = row.clone();
        parent.number = number;
        parent.root = number;
        parent.requester = tasks::Party::Deployment { project: 1 };
        parent.delegates = Box::new([row.number]);
        parent.made = 2;
        row.root = number;
        row.depth = 1;
        Some(parent)
    } else {
        None
    };
    let mut records = List::uncapped();
    for record in [
        crate::Record::Core(CoreRecord::Tasks(tasks::Stored::Live(Box::new(row)))),
        crate::Record::Core(CoreRecord::Tasks(tasks::Stored::Ledger(ledger))),
        crate::Record::Core(CoreRecord::People(people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: b"reader".as_slice().into() },
                login: b"reader".as_slice().into(),
                name: b"Reader".as_slice().into(),
            },
        })),
        crate::Record::Core(CoreRecord::People(people::Stored::Person {
            number: 2,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: b"other".as_slice().into() },
                login: b"other".as_slice().into(),
                name: Box::new([]),
            },
        })),
        crate::Record::Core(CoreRecord::People(people::Stored::Roles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person: 2, role: people::Role::Observer },
            ]),
        })),
    ] {
        records.push(record).unwrap();
    }
    if let Some(parent) = parent {
        records.push(crate::Record::Core(CoreRecord::Tasks(tasks::Stored::Live(Box::new(parent))))).unwrap();
    }
    let (mut domain, env) =
        super::resume_fixture::root_with_records(super::resume_fixture::config(7), &env.limits, records.as_slice());
    let key = crate::CallKey { task: number, attempt: 1, completion: 2, position: 3 };
    domain
        .call_renders
        .insert(
            Token::new(90),
            crate::route::host_route::Flight {
                task: number,
                attempt: 1,
                key: Some(key),
                note_decoding: false,
                job: None,
                page: None,
                render: crate::route::host_route::FlightRender::Plain,
                name: b"read".as_slice().into(),
                tool: b"see".as_slice().into(),
            },
        )
        .unwrap();
    (domain, env, key)
}

fn take_read(domain: &mut crate::Domain, key: crate::CallKey) -> Read {
    let Some(crate::Work::Core(jig_domain_core::Event::NamedAction { to, key: actual, action })) = domain.work.pop()
    else {
        panic!("the flight asks one named person read");
    };
    assert_eq!(to, skein_lib::ReplyTo::new(Token::new(90)));
    assert_eq!(actual, key);
    let jig_domain_core::NamedAction::PersonRead(read) = action else { panic!("read as the person") };
    assert!(domain.work.is_empty());
    read
}

fn person_now(
    domain: &mut crate::Domain,
    env: &skein_lib::Env<crate::Limits>,
    decision: &mut crate::Decision,
    key: crate::CallKey,
    answer: Answer,
) {
    let mut out = skein_lib::Queue::uncapped();
    out.push(jig_domain_core::Request::Now(Box::new(jig_domain_core::Now::PersonAnswer {
        to: skein_lib::ReplyTo::new(Token::new(90)),
        key,
        answer: Box::new(answer),
    })));
    crate::route::route_core_requests(domain, env, decision, out);
}

fn delivered(
    domain: &mut crate::Domain,
    env: &skein_lib::Env<crate::Limits>,
    decision: crate::Decision,
    line: &[u8],
) -> crate::SeeAnswer {
    let mut out = skein_lib::Queue::uncapped();
    crate::domain::close(domain, env, decision, &mut out);
    crate::release(domain, env, &mut out);
    crate::release(domain, env, &mut out);
    let mut delivered = None;
    for _ in 0..out.len() {
        let request = out.pop().unwrap();
        if let crate::Request::Worker(crate::WorkerRequest::Deliver(crate::Delivery::Host(host))) = request {
            let crate::HostDelivery::See { to, line: actual, from, answer } = *host else { panic!("see delivery") };
            assert_eq!(to, skein_lib::ReplyTo::new(Token::new(90)));
            assert_eq!(actual.as_ref(), line);
            assert_eq!(from, 17);
            assert!(delivered.is_none(), "one terminal delivery");
            delivered = Some(*answer);
        }
    }
    delivered.expect("one terminal delivery")
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table checks the flight's core and application handoffs through delivery"
)]
fn person_flights_route_core_and_forge_answers_with_names_and_refusals() {
    use crate::{SeeAnswer, SeeName, SeeNames, SeeNotRead, SeePerson, SeeRead};
    use jig_domain_tasks::Party;
    use skein_lib::{Queue, ReplyTo};
    for (line, requester, why) in [
        ("me", Party::Task(9), None),
        ("me", Party::Deployment { project: 1 }, None),
        ("me", Party::Person(1), Some(Why::Unknown)),
        ("project", Party::Person(1), Some(Why::Busy)),
        ("me", Party::Person(1), None),
        ("inbox", Party::Person(1), None),
        ("changes", Party::Person(1), None),
        ("change 99", Party::Person(1), None),
    ] {
        let (mut domain, env, key) = chat(requester);
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::route::see::call(
            &mut domain,
            &env,
            &mut decision,
            ReplyTo::new(Token::new(90)),
            key,
            line.as_bytes().into(),
            17,
        );
        if requester != Party::Person(1) {
            assert!(domain.work.is_empty(), "no read without a person");
            assert_eq!(
                delivered(&mut domain, &env, decision, line.as_bytes()),
                SeeAnswer::NotRead(SeeNotRead::NoPerson)
            );
            continue;
        }
        let read = take_read(&mut domain, key);
        if let Some(why) = why {
            person_now(&mut domain, &env, &mut decision, key, Answer::None(why));
            assert!(domain.work.is_empty(), "a refused first read stops");
            assert_eq!(
                delivered(&mut domain, &env, decision, line.as_bytes()),
                SeeAnswer::NotRead(if why == Why::Busy { SeeNotRead::Busy } else { SeeNotRead::Unknown })
            );
            continue;
        }
        let expected = if line == "me" {
            assert_eq!(read, Read::Project { project: 1 });
            let mut document = d::make(
                &domain.core,
                &crate::limits::core_limits(&env.limits),
                1,
                &d::Subject::Project { project: 1 },
                &d::Window { rows: 0, after: None },
                env.wall,
            )
            .unwrap();
            let d::Document::Project(project) = &mut document else { panic!("project document") };
            project.name.bytes = b"Project from read".as_slice().into();
            let project = Answer::Document(Box::new(document));
            person_now(&mut domain, &env, &mut decision, key, project.clone());
            assert_eq!(
                take_read(&mut domain, key),
                Read::Inbox {
                    group: jig_domain_core::reads::Group::Needs,
                    filter: jig_domain_core::reads::Filter { project: Some(1), goal: None },
                    from: None,
                }
            );
            person_now(&mut domain, &env, &mut decision, key, Answer::None(Why::Busy));
            SeeAnswer::Person(Box::new(SeePerson {
                read: SeeRead::Me { person: 1, project, needs: Answer::None(Why::Busy), words: Box::new([]) },
                names: SeeNames {
                    project: SeeName { number: 1, name: b"Project from read".as_slice().into() },
                    people: Box::new([
                        SeeName { number: 1, name: b"Reader".as_slice().into() },
                        SeeName { number: 2, name: b"other".as_slice().into() },
                    ]),
                    charters: Box::new([SeeName { number: 1, name: Box::new([]) }]),
                },
            }))
        } else if line == "inbox" {
            use crate::SeeWords;
            assert_eq!(
                read,
                Read::Inbox {
                    group: jig_domain_core::reads::Group::Needs,
                    filter: jig_domain_core::reads::Filter { project: Some(1), goal: None },
                    from: None,
                }
            );
            let needs = question_page();
            person_now(&mut domain, &env, &mut decision, key, needs.clone());
            for (task, message) in [(41, 9), (42, 3)] {
                assert_eq!(take_read(&mut domain, key), Read::Question { task, message });
                person_now(&mut domain, &env, &mut decision, key, Answer::None(Why::Unknown));
            }
            SeeAnswer::Person(Box::new(SeePerson {
                read: SeeRead::Inbox {
                    needs,
                    words: Box::new([
                        SeeWords { task: 41, message: 9, words: Answer::None(Why::Unknown) },
                        SeeWords { task: 42, message: 3, words: Answer::None(Why::Unknown) },
                    ]),
                },
                names: SeeNames {
                    project: SeeName { number: 1, name: b"Project 1".as_slice().into() },
                    people: Box::new([
                        SeeName { number: 1, name: b"Reader".as_slice().into() },
                        SeeName { number: 2, name: b"other".as_slice().into() },
                    ]),
                    charters: Box::new([SeeName { number: 1, name: Box::new([]) }]),
                },
            }))
        } else {
            let Read::Application { scope, connector, read, window } = read else { panic!("application read") };
            let expected_document = domain.forge.document(read.raw(), scope, 1, env.wall, &env.limits.forge);
            let mut requests = Queue::uncapped();
            requests.push(jig_domain_core::Request::Ask {
                connector,
                ask: jig_domain_core::Ask::Read { reply: ReplyTo::new(Token::new(90)), scope, read, party: 1, window },
            });
            crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
            if line == "change 99" {
                assert!(expected_document.is_none());
                SeeAnswer::NotRead(SeeNotRead::Unknown)
            } else {
                let Read::Application { read, .. } = take_read(&mut domain, key) else {
                    panic!("second application read")
                };
                assert_eq!(read, Token::new(temper_engine_domain_forge::landings_subject(1, false)));
                person_now(&mut domain, &env, &mut decision, key, Answer::None(Why::Busy));
                SeeAnswer::Person(Box::new(SeePerson {
                    read: SeeRead::Changes {
                        changes: match expected_document {
                            Some(document) => Some(Box::new(document)),
                            None => None,
                        },
                        landings: None,
                    },
                    names: SeeNames {
                        project: SeeName { number: 1, name: b"Project 1".as_slice().into() },
                        people: Box::new([
                            SeeName { number: 1, name: b"Reader".as_slice().into() },
                            SeeName { number: 2, name: b"other".as_slice().into() },
                        ]),
                        charters: Box::new([SeeName { number: 1, name: Box::new([]) }]),
                    },
                }))
            }
        };
        assert_eq!(delivered(&mut domain, &env, decision, line.as_bytes()), expected, "{line}");
        assert!(domain.work.is_empty(), "the final read makes no further work");
    }
}
