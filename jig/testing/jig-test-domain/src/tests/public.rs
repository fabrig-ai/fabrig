//! Public admission and the journal callback (jig/docs/design/domain/root.md, 3 and 10).
#![expect(
    clippy::disallowed_macros,
    reason = "ordinary Rust fixtures collect store rows and compare complete output records"
)]
use super::fixtures;
use crate::{Delivery, Domain, Event, Limits, Record, Request, Write};
use jig_domain_core::{self as core, PublicAnswer, PublicRead};
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

fn reply(number: u64) -> ReplyTo {
    ReplyTo::new(Token::new(number))
}

fn projects(number: u64) -> Event {
    Event::Core(core::Event::PublicRead {
        reply: reply(number),
        read: PublicRead { owner: 0, kind: 1, version: 1, project: None, task: None },
    })
}

fn answer(number: u64, answer: PublicAnswer) -> Request {
    Request::Deliver(Delivery::Core(core::Held::PublicRead { reply: reply(number), answer }))
}

fn outputs(mut out: Queue<Request>, expected: &[Request]) {
    assert_eq!(usize::try_from(out.len()).unwrap(), expected.len(), "complete output count: {out:?}");
    for request in expected {
        assert_eq!(format!("{:?}", out.pop().unwrap()), format!("{request:?}"));
    }
}

fn released(domain: &mut Domain, env: &Env<Limits>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::release(domain, env, &mut out);
    out
}

fn expected_projects(at: Time) -> PublicAnswer {
    PublicAnswer::Document {
        decided: at,
        document: core::PublicDocument::Projects(core::PublicProjects {
            projects: Box::new([core::PublicProject { project: 1, name: Box::from("Public project") }]),
        }),
    }
}

fn fixture() -> (Domain, Env<Limits>) {
    let mut env = fixtures::env();
    env.limits.public_reads = 2;
    env.limits.public_watermark = 16;
    env.limits.watermark = 32;
    env.limits.core.public = core::PublicIndexLimits { goals: 2, ended: 1, lines: 2, depth: 2 };
    let mut rows = super::start::seeded_rows();
    for row in &mut rows {
        if let Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) = row {
            header.projects = 1;
            header.people = 1;
            header.sign_ins = 1;
        }
    }
    rows.extend([
        Record::Core(core::Record::People(people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                login: Box::new([]),
                name: Box::new([]),
            },
        })),
        Record::Core(core::Record::People(people::Stored::SignIn {
            number: 1,
            person: 1,
            expires: Wall::from_nanos(100_000_000_000),
        })),
        Record::Core(core::Record::People(people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        })),
        Record::Core(core::Record::People(people::Stored::Policy {
            project: 1,
            name: Box::from("Public project"),
            value: people::PolicyValue {
                ceiling: people::Authority {
                    tools: 1023,
                    models: None,
                    grants: Box::new([]),
                    delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
                    spend: 100,
                    deadline: None,
                    notes: 0,
                    note_resources: Box::new([]),
                },
                escalation_role: None,
                period_spend: 100,
                projections: Box::new([]),
                roles: Box::new([]),
                requirements: Box::new([]),
                permissions: Box::new([]),
                templates: Box::new([]),
            },
        })),
        Record::Core(core::Record::People(people::Stored::Public { project: 1 })),
    ]);
    let mut host = super::start::Host::new(rows);
    host.env = env;
    host.begin();
    host.drive();
    assert!(host.domain.ready());
    (host.domain, host.env)
}

#[test]
fn a_public_read_at_the_public_watermark_is_busy_and_a_signed_in_read_is_admitted() {
    let (mut domain, mut env) = fixture();
    env.limits.public_watermark = 1;
    crate::try_step(&mut domain, &env, projects(10)).unwrap();
    assert_eq!(crate::admission::backlog(&domain), 1);
    crate::try_step(&mut domain, &env, projects(11)).unwrap();
    assert_eq!(domain.public_reads, 1, "the busy read is never admitted");
    crate::try_step(
        &mut domain,
        &env,
        Event::Core(core::Event::Read {
            reply: reply(12),
            sign_in: 1,
            read: core::reads::Read::Project { project: 999 },
        }),
    )
    .unwrap();
    outputs(
        released(&mut domain, &env),
        &[
            Request::Deliver(Delivery::Core(core::Held::Read {
                reply: reply(12),
                answer: core::reads::Answer::None(core::reads::Why::Unknown),
            })),
            answer(11, PublicAnswer::Busy),
        ],
    );
    outputs(released(&mut domain, &env), &[answer(10, expected_projects(env.now))]);
    assert_eq!(domain.public_reads, 0);
}

#[test]
fn the_core_holds_at_most_public_reads() {
    let (mut domain, env) = fixture();
    for number in [20, 21] {
        crate::try_step(&mut domain, &env, projects(number)).unwrap();
    }
    assert_eq!(domain.public_reads, env.limits.public_reads);
    crate::try_step(&mut domain, &env, projects(22)).unwrap();
    assert_eq!(domain.public_reads, 2);
    outputs(released(&mut domain, &env), &[answer(22, PublicAnswer::Busy)]);
    assert_eq!(domain.public_reads, 2, "callbacks still retain their admission slots");
    outputs(
        released(&mut domain, &env),
        &[answer(20, expected_projects(env.now)), answer(21, expected_projects(env.now))],
    );
    assert_eq!(domain.public_reads, 0);
}

#[test]
fn the_count_falls_as_answers_are_released() {
    let (mut domain, mut env) = fixture();
    env.limits.public_reads = 1;
    crate::try_step(&mut domain, &env, projects(30)).unwrap();
    outputs(released(&mut domain, &env), &[]);
    assert_eq!(domain.public_reads, 1);
    crate::try_step(&mut domain, &env, projects(31)).unwrap();
    outputs(released(&mut domain, &env), &[answer(30, expected_projects(env.now)), answer(31, PublicAnswer::Busy)]);
    assert_eq!(domain.public_reads, 0);
    crate::try_step(&mut domain, &env, projects(32)).unwrap();
    outputs(released(&mut domain, &env), &[]);
    outputs(released(&mut domain, &env), &[answer(32, expected_projects(env.now))]);
    assert_eq!(domain.public_reads, 0);
}

#[test]
fn an_application_kind_reaches_the_test_connector_and_back() {
    let (mut domain, env) = fixture();
    for owner in [2, 3, 99] {
        crate::try_step(
            &mut domain,
            &env,
            Event::Core(core::Event::PublicRead {
                reply: reply(40),
                read: PublicRead { owner, kind: 14, version: 7, project: Some(1), task: None },
            }),
        )
        .unwrap();
        assert_eq!(domain.public_reads, 1);
        if owner != 99 {
            let mut held = Queue::uncapped();
            let _released = domain.journal.release(&mut held);
            let Some(Delivery::Core(core::Held::PublicWait { read })) = held.pop() else {
                panic!("the connector answered, and its answer waits for durability")
            };
            assert!(held.is_empty());
            crate::step(&mut domain, &env, Event::Core(core::Event::PublicDurable { read }));
        }
        outputs(released(&mut domain, &env), &[answer(40, PublicAnswer::Unknown)]);
        assert_eq!(domain.public_reads, 0);
    }
}

#[test]
fn a_read_waits_for_newer_commits_before_its_completing_step() {
    let (mut domain, mut env) = fixture();
    crate::try_step(&mut domain, &env, projects(50)).unwrap();
    let core::Write::Save(header) = domain.engine.core_mut().counters.commit_header() else {
        panic!("one deployment header")
    };
    let number = domain.engine.core().counters.deployment().commits;
    let mut decision = domain.journal.decide();
    let saved = Write::Save(Record::Core(header));
    decision.push_write(saved.clone());
    domain.journal.accept(decision);
    let mut writes = Queue::uncapped();
    writes.push(saved);
    outputs(released(&mut domain, &env), &[Request::Commit { number, writes }]);
    assert_eq!(domain.public_reads, 1, "the callback follows the newer commit");
    outputs(released(&mut domain, &env), &[]);
    env.now = Time::from_nanos(200);
    crate::try_step(&mut domain, &env, Event::Committed { number, fill: 0 }).unwrap();
    outputs(released(&mut domain, &env), &[]);
    outputs(released(&mut domain, &env), &[answer(50, expected_projects(env.now))]);
    assert_eq!(domain.public_reads, 0);
}
