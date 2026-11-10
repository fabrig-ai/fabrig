//! Root admission and connector public reads (temper/docs/design/client/forge/README.md, 9).
use super::{projection_settled, resume_fixture};
use crate::route::public;
use crate::{Decision, Delivery, Domain, Event, Limits, PartyInput, PartyRequest, Request};
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};
use temper_client_protocol_wire::public_v1 as w;
const REPLY: ReplyTo = ReplyTo::new(Token::new(91));
fn root() -> (Domain, Env<Limits>) {
    let mut limits = resume_fixture::limits();
    limits.public_watermark = limits.watermark;
    limits.public_reads = 1;
    resume_fixture::root_with_config_and_limits(resume_fixture::config(7), &limits)
}
fn read(owner: u16) -> Event {
    Event::Party(PartyInput::PublicRead {
        reply: REPLY,
        read: core::PublicRead { owner, kind: 2, version: 1, project: Some(1), task: None },
    })
}
fn release(domain: &mut Domain, env: &Env<Limits>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    // One pass drains the immediate door; the next releases its held terminal.
    crate::release(domain, env, &mut out);
    crate::release(domain, env, &mut out);
    out
}
#[test]
fn a_forge_kind_is_asked_of_the_forge_with_its_public_tasks() {
    let (mut domain, env, task) = projection_settled::live_goal(Some(1));
    let public = public::public_tasks(&domain, &[task, u64::MAX]);
    assert_eq!(
        public.as_ref(),
        &[
            temper_engine_domain_forge::PublicTask { task, goal: Some(task) },
            temper_engine_domain_forge::PublicTask { task: u64::MAX, goal: None }
        ]
    );
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    for (connector, answers) in [(domain.config.forge_connector, true), (u16::MAX, false)] {
        let mut requests = Queue::uncapped();
        requests.push(core::Request::Ask {
            connector,
            ask: core::Ask::Public {
                reply: REPLY,
                kind: w::PUBLIC_LANDINGS,
                version: w::PUBLIC_VERSION,
                project: 1,
                task: None,
                public: Box::new([task]),
            },
        });
        crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
        let Some(crate::boundary::Work::Core(core::Event::PublicAnswered { reply, answer })) = domain.work.pop() else {
            panic!("public answer in the same decision")
        };
        assert_eq!(reply, REPLY);
        assert_eq!(answer.is_some(), answers);
        if let Some(answer) = answer {
            assert!(answer.tasks.is_empty());
            let _record = w::PublicLandings::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&answer.record))
                .expect("public record");
        }
    }
    let mut out = Queue::uncapped();
    crate::domain::close(&mut domain, &env, decision, &mut out);
}
#[test]
fn a_public_read_past_the_public_watermark_is_busy() {
    let (mut domain, env) = root();
    for _ in 0..env.limits.public_watermark {
        crate::fresh(&mut domain.core.counters, crate::Family::Message).expect("counter");
        crate::try_step(&mut domain, &env, Event::Resume).expect("header");
    }
    let before = domain.core.counters.deployment();
    crate::try_step(&mut domain, &env, read(0)).expect("busy is consumed");
    let out = release(&mut domain, &env);
    assert_eq!(public_answer(&out, false), Some(&core::PublicAnswer::Busy), "{out:?}");
    assert_eq!(domain.public_reads, 0);
    assert_eq!(domain.core.counters.deployment(), before);
}
fn public_answer(out: &Queue<Request>, admitted: bool) -> Option<&core::PublicAnswer> {
    for request in out {
        if let Request::Party(PartyRequest::Deliver(Delivery::Public { answer, admitted: held, .. })) = request
            && *held == admitted
        {
            return Some(answer);
        }
    }
    None
}
#[test]
fn public_reads_past_public_reads_are_busy_and_freed_on_every_answer() {
    for answer in [
        core::PublicAnswer::Unknown,
        core::PublicAnswer::Busy,
        core::PublicAnswer::Document {
            decided: skein_lib::Time::ZERO,
            document: core::PublicDocument::Application(core::PublicApplication {
                tasks: Box::new([]),
                record: Box::new([]),
            }),
        },
    ] {
        let (mut domain, env) = root();
        // A connector's admitted read holds its slot until the terminal leaves the journal.
        domain.public_reads = env.limits.public_reads;
        crate::try_step(&mut domain, &env, read(0)).expect("busy is consumed");
        assert_eq!(public_answer(&release(&mut domain, &env), false), Some(&core::PublicAnswer::Busy));
        assert_eq!(domain.public_reads, 1);
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        let mut requests = Queue::uncapped();
        requests.push(core::Request::Held(Box::new(core::Held::PublicRead { reply: REPLY, answer })));
        crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
        let mut out = Queue::uncapped();
        crate::domain::close(&mut domain, &env, decision, &mut out);
        assert_eq!(domain.public_reads, 1);
        let out = release(&mut domain, &env);
        assert!(public_answer(&out, true).is_some(), "{out:?}");
        assert_eq!(domain.public_reads, 0);
    }
    let (mut domain, env) = root();
    crate::try_step(&mut domain, &env, read(u16::MAX)).expect("admitted unknown owner");
    assert_eq!(domain.public_reads, 1);
    assert!(public_answer(&release(&mut domain, &env), true).is_some());
    assert_eq!(domain.public_reads, 0);
}
#[test]
fn a_public_goal_is_the_outermost_tracked_task() {
    let (domain, env, goal) = projection_settled::live_goal(Some(1));
    let mut parent = domain.core.tasks.task(goal).expect("goal").clone();
    parent.number = goal.checked_add(1).expect("fixture number");
    parent.depth = 0;
    let mut child = domain.core.tasks.task(goal).expect("goal").clone();
    child.requester = tasks::Party::Task(parent.number);
    child.depth = 1;
    let funding = *domain.core.tasks.funding(parent.funder).expect("funding");
    let records = [
        crate::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(funding))),
        crate::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(parent.clone())))),
        crate::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(child)))),
    ];
    let (domain, _) = resume_fixture::root_with_records(resume_fixture::config(7), &env.limits, &records);
    assert_eq!(domain.core.goal_of(domain.core.tasks.task(goal).expect("child")), Some(goal));
    assert_eq!(public::public_goal(&domain, goal), Some(parent.number));
    assert_eq!(public::public_goal(&domain, u64::MAX), None);
}
