use crate::decision::{
    Counters, Decision, Delivery, Journal as SkeinJournal, JournalOutput as Output, accept_routed,
    committed as mark_committed, resume as release_ready,
};
use crate::{Deployment, Family, Key, Record, Write};
use alloc::boxed::Box;
use jig_domain_link::{Form, Grant, Scheme};
use skein_lib::{List, Queue, Token, Wall};
use step_journal_fixture::{Journal, accept, ack, acknowledged, commit, decision, fresh, resume as resume_journal};
use temper_engine_step_kit::journal::{DEPLOYMENT, LIMITS, bytes, turn};

temper_engine_step_kit::journal_fixture!(crate);
temper_engine_step_kit::start_fixture!(crate);
temper_engine_step_kit::project_charter_fixture!(crate, super::step_start_fixture);
pub(crate) use temper_engine_step_kit::project_charters::owned_repository;

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod admission;
mod budget;
mod change_facts;
pub(crate) mod change_row;
mod chat_document;
mod checkpoints;
mod dismiss;
mod documents;
mod forge_route;
mod guides;
mod history;
mod looked;
mod project_charters;
mod projection_room;
mod projection_settled;
mod public;
mod remember;
pub(crate) use projection_settled::live_task;
mod kinds;
mod release;
mod removal;
mod restore_fit;
mod result;
mod resume;
pub(super) mod resume_fixture;
mod setup;
mod start_stop;
mod start_wait;
mod validity;

pub(crate) fn accept_step(domain: &mut crate::Domain, env: &skein_lib::Env<crate::Limits>, decision: Decision) {
    accept_routed(&mut domain.journal, &mut domain.engine.core_mut().counters, &env.limits.journal, decision, false)
        .unwrap();
}

#[test]
fn a_refused_commit_stops_the_root_once() {
    for (durable, sent, number) in [(0, 0, 0), (0, 0, 1), (0, 0, u64::MAX), (8, 9, 7), (8, 9, 10)] {
        let (mut domain, env) = resume_fixture::root();
        domain.engine.core_mut().counters = Counters::new(Deployment { commits: durable, ..domain.deployment() });
        domain.journal = SkeinJournal::uncapped(1, durable);
        domain.sent_commit = durable;
        domain.durable_commit = durable;
        let mut setup = Queue::uncapped();
        for _ in durable..sent {
            let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
            crate::domain::save(&mut decision, &env.limits, turn(1, 0));
            crate::domain::close(&mut domain, &env, decision, &mut setup);
            crate::domain::send_commit(&mut domain, &mut setup);
        }
        let mut out = Queue::uncapped();
        crate::step(&mut domain, &env, crate::Event::Store(crate::StoreInput::Committed { number, fill: 0 }));
        crate::release(&mut domain, &env, &mut out);
        assert_eq!(out.pop(), Some(crate::Request::Stop), "refused commit {number}");
        assert!(out.is_empty());
        assert_eq!(domain.journal_stop(), Some(crate::JournalStop::Refused { commit: number, sent, durable }));
        crate::step(&mut domain, &env, crate::Event::Store(crate::StoreInput::Committed { number, fill: 0 }));
        crate::release(&mut domain, &env, &mut out);
        assert!(out.is_empty(), "the stopped root emits Stop once");
    }
}

#[test]
fn a_root_store_edge_overrun_releases_only_stop_and_discards_immediate_outputs() {
    let (mut domain, env) = resume_fixture::root();
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    for number in 1..=env.limits.journal.writes {
        crate::domain::save(&mut decision, &env.limits, turn(number, 1));
    }
    let mut out = Queue::uncapped();
    out.push(crate::Request::Drained);
    crate::domain::close(&mut domain, &env, decision, &mut out);
    crate::domain::stage_now(&mut domain, &mut out);
    crate::release(&mut domain, &env, &mut out);
    assert_eq!(out.pop(), Some(crate::Request::Stop));
    assert!(out.is_empty());
    crate::try_step(&mut domain, &env, crate::Event::Resume).unwrap();
    crate::release(&mut domain, &env, &mut out);
    assert!(out.is_empty(), "nothing from the failed decision or after it leaves");
    assert!(domain.journal.stopped());
    assert_eq!(domain.journal_stop(), Some(crate::JournalStop::PastEdge { writes: env.limits.journal.writes }));
}

#[test]
fn a_sign_in_cap_routes_from_the_party_input_to_the_core() {
    let (mut domain, env) = resume_fixture::root();
    let cap = Wall::from_nanos(1200);
    crate::step(
        &mut domain,
        &env,
        crate::Event::Party(crate::PartyInput::SignedIn {
            reply_to: skein_lib::ReplyTo::new(Token::new(73)),
            identity: jig_domain_people::Identity {
                key: jig_domain_people::IdentityKey { provider: 0, subject: Box::new([1]) },
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            },
            held: None,
            until: Some(cap),
        }),
    );
    assert_eq!(domain.engine.core().people.next_wall_expiry(), Some(cap));
}

#[test]
fn new_work_waits_at_the_watermark_and_admitted_work_is_always_taken() {
    use crate::{Event, PartyInput, StoreInput};
    use skein_lib::{ReplyTo, Token};
    fn call() -> Event {
        Event::Party(PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(72)),
            sign_in: 99,
            key: [7; 16],
            ask: jig_domain_people::Ask::SetRoles { project: 1, holdings: Box::new([]) },
        })
    }
    let mut limits = resume_fixture::limits();
    limits.watermark = limits.journal.commits + 2;
    assert_ne!(limits.watermark, crate::default_watermark(&limits.journal));
    let (mut domain, env) = resume_fixture::root_with_config_and_limits(resume_fixture::config(1), &limits);
    // Header-only commits give an exact backlog without held-output entries.
    for _ in 0..u64::from(env.limits.watermark) - 1 {
        crate::fresh(&mut domain.engine.core_mut().counters, Family::Message).unwrap();
        crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    }
    assert_eq!(crate::limits::backlog(&domain), u64::from(env.limits.watermark) - 1);
    assert!(crate::limits::backlog(&domain) > u64::from(env.limits.journal.commits));
    crate::try_step(&mut domain, &env, call()).expect("past commits but below the watermark the call is taken");
    // This unauthenticated call owns one held refusal, bringing backlog to the mark.
    assert_eq!(crate::limits::backlog(&domain), u64::from(env.limits.watermark));
    let returned = crate::try_step(&mut domain, &env, call()).expect_err("at the mark new work waits");
    let Event::Party(PartyInput::Ask { reply_to, sign_in, key, ask, models }) = &returned else {
        panic!("the original call is returned");
    };
    assert_eq!(reply_to, &ReplyTo::new(Token::new(72)));
    assert_eq!((*sign_in, *key), (99, [7; 16]));
    assert!(models.is_empty());
    assert_eq!(ask, &jig_domain_people::Ask::SetRoles { project: 1, holdings: Box::new([]) });
    let released = Event::Released(crate::Released(Delivery::Fleet(jig_domain_fleet::Event::TurnKept {
        run: Token::new(1),
        attempt: Token::new(1),
        turn: 1,
    })));
    crate::try_step(&mut domain, &env, released).expect("released work advances at the mark");
    let mut last = 0;
    while let Some(Output::Commit { number, .. }) = crate::commit(&mut domain.journal, &env.limits.journal) {
        last = number;
    }
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number: last, fill: 0 }))
        .expect("store completion is always taken");
    assert!(crate::limits::backlog(&domain) < u64::from(env.limits.watermark));
    crate::try_step(&mut domain, &env, returned).expect("after completion the returned call is taken");
    assert!(!domain.journal.stopped());
}

#[test]
fn counter_and_commit_overflow_cannot_reuse_names() {
    let mut j = Journal::new(Deployment { tasks: u64::MAX, commits: u64::MAX, ..DEPLOYMENT }, &LIMITS);
    assert_eq!(fresh(&mut j, Family::Task), None);
    let mut out = Queue::with_capacity(1);
    let d = decision(&mut j, 1);
    accept(&mut j, &LIMITS, d, &mut out).unwrap();
    assert!(j.stopped());
    assert!(out.is_empty());
    assert_eq!(j.deployment().tasks, u64::MAX);

    let (mut domain, env) = resume_fixture::root();
    domain.engine.core_mut().counters = Counters::new(Deployment { commits: u64::MAX, ..domain.deployment() });
    domain.journal = SkeinJournal::uncapped(1, u64::MAX);
    domain.sent_commit = u64::MAX;
    domain.durable_commit = u64::MAX;
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::domain::save(&mut decision, &env.limits, turn(1, 1));
    let mut out = Queue::uncapped();
    crate::domain::close(&mut domain, &env, decision, &mut out);
    assert_eq!(out.pop(), Some(crate::Request::Stop));
    assert!(out.is_empty());
    assert_eq!(domain.journal_stop(), Some(crate::JournalStop::Counter));
    crate::try_step(&mut domain, &env, crate::Event::Resume).unwrap();
    crate::release(&mut domain, &env, &mut out);
    assert!(out.is_empty(), "the counter stop is emitted once");
}

#[test]
fn held_assignment_checks_owned_bytes_and_section_backing_before_acceptance() {
    let limits = crate::JournalLimits {
        commits: 1,
        held: 1,
        writes: 1,
        deliveries: 1,
        transcript_bytes: 4,
        result_bytes: 4,
        run_bytes: 512,
    };
    let mut journal = SkeinJournal::uncapped(1, 0);
    let mut decision = Decision::decide(&mut journal, &limits);
    let mut assignment = held_assignment();
    assert!(
        decision
            .deliver(&limits, Delivery::Assigned { channel: Token::new(1), assignment: assignment.clone() })
            .is_err()
    );
    assignment.sections = Box::new([]);
    let bytes = crate::limits::run_charter_bytes(&assignment.run).expect("fixture charter bytes");
    let exact = crate::JournalLimits { run_bytes: u32::try_from(bytes).expect("tiny charter"), ..limits };
    for waiting in [skein_lib::Duration::from_secs(1), skein_lib::Duration::ZERO] {
        assignment.run.policy.waiting = waiting;
        let mut journal = SkeinJournal::uncapped(1, 0);
        let mut decision = Decision::decide(&mut journal, &exact);
        decision
            .deliver(&exact, Delivery::Assigned { channel: Token::new(1), assignment: assignment.clone() })
            .expect("the exact run byte envelope accepts its binding arrays, including zero waiting");
    }
    assignment.run.sources = Box::new([Some(1), Some(2)]);
    assignment.run.accounts = Box::new([
        jig_domain_authority::AccountBinding { provider: 1, account: 1, source: 1 },
        jig_domain_authority::AccountBinding { provider: 2, account: 2, source: 2 },
    ]);
    let mut journal = SkeinJournal::uncapped(1, 0);
    let mut decision = Decision::decide(&mut journal, &exact);
    assert!(decision.deliver(&exact, Delivery::Assigned { channel: Token::new(1), assignment }).is_err());
}

fn held_assignment() -> crate::Assignment {
    use jig_domain_brief as brief;
    crate::Assignment {
        check_timeout: None,
        task: 1,
        attempt: 1,
        conversation: 1,
        live: jig_domain_host::LiveAsk::NONE,
        charter: 1,
        run: Box::new(crate::RunCharter {
            revision: 1,
            ended_by: jig_domain_tasks::EndedBy::Result,
            sources: Box::new([Some(1)]),
            accounts: Box::new([jig_domain_authority::AccountBinding { provider: 1, account: 1, source: 1 }]),
            placement: jig_domain_fleet::PlacementConstraint {
                channel: Token::new(1),
                kind: jig_domain_fleet::HostKind::Worker,
            },
            policy: crate::RunPolicy {
                instructions: Box::new([]),
                waiting: skein_lib::Duration::from_secs(1),
                resume: true,
                turns: 2,
                time: skein_lib::Duration::from_secs(2),
                reserve_turns: 1,
                reserve_time: skein_lib::Duration::from_secs(1),
                model: crate::Model {
                    provider: 1,
                    program: None,
                    dialect: 1,
                    endpoint: 1,
                    name: b"m".as_slice().into(),
                    cache_life_ms: 0,
                    window: 1,
                    output: 1,
                    input_price: 1,
                    cached_price: 1,
                    cache_write_price: None,
                    output_price: 1,
                    price_unit: 1,
                    efforts: Box::new([]),
                    effort: None,
                    speeds: Box::default(),
                    speed: None,
                },
                alternatives: Box::new([]),
                inspect: false,
                modify: false,
                shell: false,
                agents: false,
                call_timeout: skein_lib::Duration::from_secs(1),
                tools: crate::RunPolicy::EVERY_FAMILY,
            },
            contract: jig_domain_tasks::Contract::Report { words: 1 },
            authority: jig_domain_tasks::Authority {
                tools: jig_domain_tasks::Tools(0),
                models: jig_domain_tasks::Models::NONE,
                grants: Box::new([]),
                delegation: jig_domain_tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
                budget: jig_domain_tasks::Budget { spend: 1, deadline: None },
                notes: jig_domain_tasks::Scopes(0),
                note_resources: Box::new([]),
            },
            budget: 1,
        }),
        sections: Box::new([crate::BriefSection {
            kind: crate::BriefKind::Core(brief::Core::Task),
            body: crate::BriefBody::Text(b"12345".as_slice().into()),
        }]),
        messages: Box::new([]),
        saved: Box::new([]),
        workspace: crate::ForgeWorkspace::default(),
        workstream: 1,
        transcript: Box::new([]),
        answered: Box::new([]),
        settled: Box::new([]),
        parts: Box::new([]),
        grants: Box::new([Grant {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: skein_lib::Duration::from_secs(1),
        }]),
    }
}

#[test]
fn a_conversations_prepared_text_ask_reaches_its_assignment() {
    use crate::boundary::{PreparedWorkspace, WorkspacePreparation};
    use crate::route::forge_route::RunWorkspace;
    use jig_domain_core as core;

    let (mut domain, env) = resume_fixture::root();
    let fixture = held_assignment();
    domain
        .run_workspaces
        .insert(
            fixture.task,
            WorkspacePreparation::Ready(PreparedWorkspace {
                workspace: RunWorkspace {
                    workspace: crate::ForgeWorkspace::default(),
                    workstream: fixture.task,
                    writes: Box::new([]),
                    names: Box::new([]),
                    designs: Box::new([]),
                    own_holds: Box::new([]),
                },
                claim_names: Box::new([]),
            }),
        )
        .unwrap();
    let mut requests = Queue::with_capacity(1);
    requests.push(core::Request::Now(Box::new(core::Now::RunPrepared {
        parts: Box::new([jig_domain_link::KeptPart {
            attempt: 1,
            turn: 2,
            thread: 0,
            call: 0,
            kind: jig_domain_link::PartKind::Answer,
            body: b"parts".as_slice().into(),
        }]),
        task: fixture.task,
        attempt: fixture.attempt,
        conversation: fixture.conversation,
        live: core::LiveAsk { text: true, reasoning: false, input: false },
        charter: fixture.charter,
        run: fixture.run,
        messages: Box::new([]),
        saved: Box::new([]),
        transcript: Box::new([]),
        answered: Box::new([]),
        grants: Box::new([]),
        brief: Box::new([]),
    })));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
    let assignment = domain.hub_link.assignment(fixture.task).expect("prepared assignment");
    assert!(assignment.live.text);
    assert!(!assignment.live.reasoning);
    assert!(!assignment.live.input);
    assert_eq!(assignment.parts.len(), 1);
    assert_eq!(domain.held_bytes(), 5, "prepared parts hold transcript room");
}

#[test]
fn a_refused_settlement_uses_cores_returned_reply_address() {
    let (mut domain, _) = resume_fixture::root();
    let payload = domain.hub_link.retain(crate::boundary::Payload::Busy);
    domain.refused_calls.insert(payload, b"returned opaque address".as_slice().into()).unwrap();
    let delivery = crate::route::host_route::relayed(
        &mut domain,
        Token::new(7),
        Token::new(3),
        Token::new(9),
        b"fleet address".as_slice().into(),
        payload,
    );
    assert_eq!(
        delivery,
        Delivery::Link {
            channel: Token::new(7),
            down: jig_domain_host::LinkDown::Relayed {
                run: Token::new(3),
                attempt: Token::new(9),
                call: b"returned opaque address".as_slice().into(),
                answer: jig_domain_host::Relayed::Answer { error: true, text: b"busy".as_slice().into() },
            },
        }
    );
}

#[test]
fn zero_watermark_refuses_startup_and_the_default_saturates() {
    let mut limits = resume_fixture::limits();
    limits.watermark = 0;
    assert!(!crate::valid(&limits));
    let journal = crate::JournalLimits { commits: u32::MAX, ..limits.journal };
    assert_eq!(crate::default_watermark(&journal), u32::MAX);
}

#[test]
fn bulk_task_drops_cross_the_journal_without_replacing_record_erases() {
    use jig_domain_core as core;
    let (mut domain, env) = resume_fixture::root();
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let erased = [
        core::Key::Tasks(jig_domain_tasks::Key::Ended(7)),
        core::Key::Core(core::CoreKey::EscalationDecision { task: 7, revision: 2 }),
        core::Key::Core(core::CoreKey::ProposalDecision {
            proposer: jig_domain_people::Proposer::Party(3),
            proposal: 9,
        }),
    ];
    let mut requests = Queue::uncapped();
    requests.push(core::Request::Write(core::Write::DropTranscript { task: 7 }));
    requests.push(core::Request::Write(core::Write::DropHistory { task: 7 }));
    for key in &erased {
        requests.push(core::Request::Write(core::Write::Erase(key.clone())));
    }
    crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
    accept_routed(&mut domain.journal, &mut domain.engine.core_mut().counters, &env.limits.journal, decision, true)
        .unwrap();
    let Some(Output::Commit { writes, .. }) = crate::decision::commit(&mut domain.journal, &env.limits.journal) else {
        panic!("the erasure reaches the store through its journal commit");
    };
    assert!(writes.contains(&Write::DropTranscript { task: 7 }));
    assert!(writes.contains(&Write::DropHistory { task: 7 }));
    assert_eq!(crate::store::owned_bytes(&Write::DropHistory { task: 7 }), Some(8));
    assert_eq!(crate::store::owned_bytes(&Write::DropTranscript { task: 7 }), Some(8));
    for key in erased {
        assert!(writes.contains(&Write::Erase(Key::Core(key))));
    }
    assert_ne!(
        Write::DropTranscript { task: 7 }.key(),
        Key::Core(core::Key::Core(core::CoreKey::Turn { task: 7, attempt: 1, turn: 1 }))
    );
}

pub(crate) mod room_checks;

mod see;

mod worker_pause;

#[test]
fn rejected_payloads_are_returned_without_replacing_admitted_ownership() {
    let mut j = Journal::new(DEPLOYMENT, &LIMITS);
    let mut d = Decision::decide(&mut j.inner, &LIMITS);
    d.write(&LIMITS, turn(1, 3)).unwrap();
    let refused = d.write(&LIMITS, turn(1, 65)).unwrap_err();
    assert_eq!(refused, turn(1, 65));
    let refused = d.deliver(&LIMITS, Delivery::Result { person: 1, task: 1, words: bytes(33) }).unwrap_err();
    let Delivery::Result { words, .. } = refused else {
        panic!("result");
    };
    assert_eq!(words.len(), 33);
    assert!(
        d.write(&LIMITS, Write::Erase(Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment))))
            .is_err()
    );
    let mut out = Queue::with_capacity(1);
    accept(&mut j, &LIMITS, d, &mut out).unwrap();
    let (_, writes) = commit(&mut out);
    assert_eq!(writes.len(), 2);
    assert_eq!(writes[1], turn(1, 3));
    step_journal_fixture::committed(&mut j, 1);
    resume_journal(&mut j, &mut out);
    assert!(out.is_empty());
}

#[test]
fn a_decision_past_the_store_write_edge_stops_the_root_journal() {
    for writes in [LIMITS.writes] {
        let mut journal = Journal::new(DEPLOYMENT, &LIMITS);
        let mut out = Queue::with_capacity(1);
        let mut decision = Decision::decide(&mut journal.inner, &LIMITS);
        for number in 1..writes {
            decision.write(&LIMITS, turn(number, 1)).expect("reserved write room");
        }
        decision.deliver(&LIMITS, ack(1)).unwrap();
        let refused = turn(writes, 1);
        let Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(row)))) = &refused
        else {
            panic!("turn");
        };
        let address = row.transcript.as_ptr();
        let refused = decision.carry_write(&LIMITS, refused).unwrap_err();
        let Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(row)))) = refused
        else {
            panic!("original refused turn");
        };
        assert_eq!(row.transcript.as_ptr(), address, "refusal moves the original payload back");
        assert_eq!(row.turn, writes);
        accept(&mut journal, &LIMITS, decision, &mut out).expect("overrun is accepted to stop");
        assert!(journal.stopped());
        resume_journal(&mut journal, &mut out);
        assert!(out.is_empty(), "no partial commit or held delivery escapes an overrun");
        assert!(journal.inner.stopped());
    }
}

#[test]
fn decisions_keep_the_header_and_release_exactly_the_admitted_deliveries() {
    for (write_count, deliveries, persist_header) in [
        (0, 0, false),
        (0, 2, false),
        (1, 0, true),
        (1, 2, true),
        (1, 2, false),
        (3, 0, true),
        (3, 2, true),
        (3, 2, false),
    ] {
        let mut journal = Journal::bootstrap(DEPLOYMENT.id, &LIMITS);
        let mut decision = Decision::decide(&mut journal.inner, &LIMITS);
        for number in 1..write_count {
            decision.carry_write(&LIMITS, turn(number, 1)).unwrap();
        }
        for number in 1..=deliveries {
            decision.carry_delivery(&LIMITS, ack(number));
        }
        let header = Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
            DEPLOYMENT,
        ))));
        assert_eq!(decision.write(&LIMITS, header.clone()), Err(header), "caller cannot own the header slot");
        accept_routed(&mut journal.inner, &mut journal.counters, &LIMITS, decision, persist_header).unwrap();
        let mut out = Queue::with_capacity(1);
        if write_count <= 1 && !persist_header {
            assert!(crate::decision::commit(&mut journal.inner, &LIMITS).is_none());
            assert!(journal.counters.dirty(), "read-only store terminals never persist the dirty header");
        } else {
            out.push(crate::decision::commit(&mut journal.inner, &LIMITS).unwrap());
            let (number, writes) = commit(&mut out);
            assert_eq!((number, writes.len()), (1, usize::try_from(write_count).unwrap()));
            assert_eq!(
                writes[0],
                Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
                    Deployment { commits: 1, ..DEPLOYMENT }
                ))))
            );
            for number in 1..write_count {
                assert_eq!(writes[usize::try_from(number).unwrap()], turn(number, 1));
            }
            resume_journal(&mut journal, &mut out);
            assert!(out.is_empty(), "held effects wait for the header's commit");
            step_journal_fixture::committed(&mut journal, number);
        }
        for number in 1..=deliveries {
            resume_journal(&mut journal, &mut out);
            assert_eq!(acknowledged(&mut out), number, "each admitted delivery releases once, in order");
        }
        resume_journal(&mut journal, &mut out);
        assert!(out.is_empty());
        assert!(!journal.stopped());
    }
}
