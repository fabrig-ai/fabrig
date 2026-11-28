//! A description promises room until its flight ends (domain/forge/room.md, 2 and 5–6).
use super::{adopted, change_row};
use crate::{
    CallAnswer, CallKey, Decision, Domain, Limits, Work,
    route::{forge_route as route, host_route},
};
use jig_domain_authority as authority;
use jig_domain_contract::effects::{Ask, EffectRefusal};
use skein_forge::api::{self, Answer, Error, Op};
use skein_lib::{Env, Queue, ReplyTo, Token};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;

const CHANGE: u64 = 51;
const ENTRY: u64 = 101;
const HEAD: [u8; 32] = [3; 32];
const BASE: [u8; 32] = [4; 32];
const RIGHT: Token = Token::new(501);

fn step(domain: &mut Domain, env: &Env<Limits>, event: forge::Event) -> Queue<forge::Request> {
    let mut out = Queue::uncapped();
    forge::step(&mut domain.forge, &env.with(env.limits.forge), event, &mut out);
    out
}

fn fixture(full: bool) -> (Domain, Env<Limits>, CallKey) {
    fixture_with_proposal(full, false)
}

fn fixture_with_proposal(full: bool, proposal: bool) -> (Domain, Env<Limits>, CallKey) {
    use jig_domain_tasks as tasks;
    let (source, _, task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
    let mut caller = source.engine.core().tasks.task(task).unwrap().clone();
    let funding = *source.engine.core().tasks.funding(caller.funder).unwrap();
    let mut change_task = caller.clone();
    change_task.number = CHANGE;
    change_task.root = CHANGE;
    change_task.executor =
        tasks::Executor::Procedure { connector: source.config.forge_connector, code: route::CHANGE_CODE };
    change_task.phase = tasks::Phase::Active(tasks::Active::Idle);
    if proposal {
        caller.proposal = Some(Box::new(tasks::Proposal {
            number: 71,
            proposer: task,
            project: 1,
            action: tasks::ProposalAction::Effect {
                connector: source.config.forge_connector,
                authority: caller.authority.clone(),
                attempt: 1,
                completion: 1,
                position: 0,
                purpose: 1,
            },
            reason: b"please post".as_slice().into(),
            as_holder: false,
            state: tasks::ProposalState::Pending {
                holder: tasks::ProposalHolder::Policy { project: 1, kind: tasks::ProposalKind::Effect },
                since: skein_lib::Wall::from_nanos(0),
            },
        }));
    }
    let mut limits = crate::tests::resume_fixture::limits();
    limits.forge.entries = 1;
    limits.authority.segments = 8;
    let (mut domain, env) = crate::tests::resume_fixture::root_with_records(
        crate::tests::resume_fixture::config(7),
        &limits,
        &[
            crate::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(caller)))),
            crate::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(change_task)))),
            crate::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(funding))),
        ],
    );
    let mut repository = adopted(1);
    repository.project = 1;
    domain.forge.restore_begin(env.wall);
    step(&mut domain, &env, forge::Event::Restore { record: forge::Stored::Repository(repository) });
    step(
        &mut domain,
        &env,
        forge::Event::Connection {
            forge: 1,
            state: forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
        },
    );
    step(
        &mut domain,
        &env,
        forge::Event::Restore {
            record: forge::Stored::BranchHead(forge::BranchHead {
                name: forge::Name {
                    forge: 1,
                    repository: 1,
                    what: forge::What::Branch(Box::new([b"main".as_slice().into()])),
                },
                commit: BASE,
                owned: false,
            }),
        },
    );
    step(&mut domain, &env, forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    step(&mut domain, &env, forge::Event::CatchUp);

    let key = CallKey { task, attempt: 1, completion: 1, position: 0 };
    domain
        .call_renders
        .insert(
            RIGHT,
            host_route::Flight {
                task: key.task,
                attempt: key.attempt,
                key: Some(key),
                note_decoding: false,
                render: host_route::FlightRender::Plain,
                name: b"call".as_slice().into(),
                tool: b"effect_forge".as_slice().into(),
                job: None,
                page: None,
            },
        )
        .unwrap();
    if full {
        step(&mut domain, &env, forge::Event::Enqueue { entry: entry(1, key.task) });
    }
    (domain, env, key)
}

fn write() -> api::Write {
    api::Write::Post { number: 1, key: b"call".as_slice().into(), body: b"words".as_slice().into() }
}

fn entry(number: u64, task: u64) -> skein_forge_client::Entry {
    skein_forge_client::Entry {
        number,
        task,
        repository: adopted(1).provider,
        effect: skein_forge_client::Effect { write: write(), condition: skein_forge_client::Condition::None },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    }
}

fn call(domain: &mut Domain, env: &Env<Limits>, key: CallKey, decision: &mut Decision) {
    route::effect_call(
        domain,
        env,
        decision,
        ReplyTo::new(RIGHT),
        key,
        adopted(1).provider,
        forge::What::Issue(1),
        write(),
        None,
    );
}

fn owner(domain: &mut Domain) -> Token {
    let Some(Work::Core(jig_domain_core::Event::EffectStart { owner, .. })) = domain.work.pop() else {
        panic!("call starts a core flight")
    };
    assert!(domain.work.is_empty());
    owner
}

fn ask(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, ask: Ask) {
    let connector = domain.config.forge_connector;
    crate::route::asks::ask(domain, env, decision, connector, jig_domain_contract::Ask::Effects(ask));
}

fn keep(
    domain: &mut Domain,
    env: &Env<Limits>,
    key: CallKey,
    owner: Token,
    decision: &mut Decision,
) -> Queue<forge::Request> {
    ask(
        domain,
        env,
        decision,
        Ask::Keep {
            owner,
            entry: ENTRY,
            task: key.task,
            project: None,
            key: jig_domain_core::EffectKey {
                deployment: domain.engine.core().counters.deployment().id,
                task: key.task,
                origin: jig_domain_core::EffectPurpose::Call {
                    attempt: key.attempt,
                    completion: key.completion,
                    position: key.position,
                },
                purpose: key.completion.into(),
            },
        },
    );
    let Some(Work::Forge(event @ forge::Event::EnqueueCall { .. })) = domain.work.pop() else {
        panic!("call's reserved enqueue")
    };
    step(domain, env, event)
}

fn open(domain: &mut Domain, env: &Env<Limits>) -> Queue<forge::Request> {
    let mut row = change_row(None);
    row.task = CHANGE;
    row.change.task = CHANGE;
    row.repository = adopted(1).provider;
    row.branch = b"temper/51".as_slice().into();
    row.change.state = change::State::Opening { requested: false };
    row.change.last_head = Some(HEAD);
    step(domain, env, forge::Event::Change { row });
    let event = route::change_step(domain, env, CHANGE, 102, adopted(1).provider, b"main", false);
    step(domain, env, event);
    let mut last = Queue::uncapped();
    for result in [
        Ok(Answer::Commit(HEAD)),
        Ok(Answer::Commit(BASE)),
        Err(Error::Missing),
        Ok(Answer::Checks(Box::new([]))),
        Ok(Answer::Checks(Box::new([]))),
        Ok(Answer::Commits { commits: Box::new([]), more: false }),
    ] {
        let mut sent = Queue::uncapped();
        let mut reading = None;
        for _ in 0..env.limits.forge.client.pending {
            forge::resume(&mut domain.forge, &env.with(env.limits.forge), &mut sent);
            while let Some(request) = sent.pop() {
                if let forge::Request::Call { call, op, .. } = request {
                    let Op::Read(_) = op else { panic!("change reads, never writes: {op:?}") };
                    reading = Some(call);
                    break;
                }
            }
            if reading.is_some() {
                break;
            }
        }
        last = step(
            domain,
            env,
            forge::Event::Client(skein_forge_client::Event::Answered {
                free: 0,
                rate: None,
                call: reading.expect("change read"),
                cost: 1,
                result,
            }),
        );
        domain.forge.reclaim();
    }
    last
}

fn assert_wait(out: &Queue<forge::Request>) {
    let mut waited = false;
    for request in out {
        if let forge::Request::ChangeDecision {
            task: CHANGE,
            decision: change::Decision::Wait { until: None },
            entry: None,
            ..
        } = request
        {
            waited = true;
        }
        if let forge::Request::ResourceRefused { .. }
        | forge::Request::Refused { .. }
        | forge::Request::ChangeDecision { decision: change::Decision::Hold(_), .. } = request
        {
            panic!("room waits without refusal or hold: {out:?}");
        }
    }
    assert!(waited, "{out:?}");
}

fn assert_unheld(domain: &Domain, key: CallKey) {
    if let jig_domain_tasks::Phase::Held { .. } = domain.engine.core().tasks.task(key.task).unwrap().phase {
        panic!("a call's lack of room never holds its task");
    }
}

fn rendered(domain: &mut Domain, env: &Env<Limits>, decision: Decision) -> CallAnswer {
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        false,
    )
    .unwrap();
    if let Some(crate::JournalOutput::Commit { number, .. }) =
        crate::decision::commit(&mut domain.journal, &env.limits.journal)
    {
        crate::decision::committed(&mut domain.journal, number);
    }
    let mut out = Queue::uncapped();
    crate::decision::resume(&mut domain.journal, &mut out);
    let Some(crate::JournalOutput::Deliver(crate::Delivery::Host(host))) = out.pop() else {
        panic!("call answer: {out:?}")
    };
    let host_route::HostDelivery::Render { answer, .. } = *host else { panic!("rendered call") };
    answer
}

#[test]
fn a_described_call_keeps_its_room_while_a_change_tries_to_open() {
    let (mut domain, env, key) = fixture(false);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    call(&mut domain, &env, key, &mut decision);
    let owner = owner(&mut domain);
    let second = CallKey { position: 1, ..key };
    assert_ne!(key, second);
    call(&mut domain, &env, second, &mut decision);
    assert_eq!(rendered(&mut domain, &env, decision), CallAnswer::Unavailable);
    assert!(domain.work.is_empty());
    assert_wait(&open(&mut domain, &env));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let kept = keep(&mut domain, &env, key, owner, &mut decision);
    let mut saved = false;
    for request in &kept {
        if let forge::Request::Save { record: forge::Stored::Entry { entry, .. }, .. } = request {
            saved |= entry.number == ENTRY;
        }
        if let forge::Request::ResourceRefused { .. } | forge::Request::Refused { .. } = request {
            panic!("reserved entry is kept, not refused: {kept:?}");
        }
    }
    assert!(saved, "{kept:?}");
    assert_eq!(domain.forge.change(CHANGE).unwrap().room, Some(1));
    assert_unheld(&domain, key);
    assert!(domain.forge_effects.is_empty());
}

#[test]
fn a_call_answers_unavailable_with_no_room_or_room_promised_to_a_waiter() {
    for freed in [false, true] {
        let (mut domain, env, key) = fixture(true);
        assert_wait(&open(&mut domain, &env));
        if freed {
            let asked = step(&mut domain, &env, forge::Event::Withdraw { entry: 1 });
            let mut woke = false;
            for request in &asked {
                if let forge::Request::WakeProcedure { task: CHANGE } = request {
                    woke = true;
                }
            }
            assert!(woke, "{asked:?}");
        }
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        call(&mut domain, &env, key, &mut decision);
        assert_eq!(rendered(&mut domain, &env, decision), CallAnswer::Unavailable);
        assert!(domain.forge_effects.is_empty());
        assert!(domain.work.is_empty());
        assert_unheld(&domain, key);
    }
}

#[test]
fn a_call_ending_without_an_entry_returns_room_and_asks_the_first_waiter() {
    for ending in [authority::Answer::Wait, authority::Answer::Refuse, authority::Answer::Propose] {
        let (mut domain, env, key) = fixture(false);
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        call(&mut domain, &env, key, &mut decision);
        let owner = owner(&mut domain);
        assert_wait(&open(&mut domain, &env));
        if ending == authority::Answer::Propose {
            ask(
                &mut domain,
                &env,
                &mut decision,
                Ask::KeepProposal { owner, proposal: 71, task: key.task, project: 1 },
            );
        } else {
            ask(
                &mut domain,
                &env,
                &mut decision,
                Ask::Drop { owner, answer: ending, findings: Box::new([]), why: EffectRefusal::Authority },
            );
        }
        assert!(domain.forge_effects.is_empty());
        assert!(domain.forge.room_asked(CHANGE));
        assert_eq!(domain.forge.inspection_reservations().as_ref(), &[CHANGE]);
        let mut asked = false;
        while let Some(work) = domain.work.pop() {
            if let Work::Forge(forge::Event::StepChange { task: CHANGE, .. })
            | Work::Tasks(jig_domain_tasks::Event::WakeProcedure { task: CHANGE }) = work
            {
                asked = true;
            }
            if let Work::Forge(forge::Event::KeepProposedEffect { row }) = work {
                step(&mut domain, &env, forge::Event::KeepProposedEffect { row });
            } else if let Work::Tasks(jig_domain_tasks::Event::Hold { .. } | jig_domain_tasks::Event::HoldFrom { .. }) =
                work
            {
                panic!("returning a call's room never holds a task");
            }
        }
        assert!(asked, "returned room asks the first waiter");
        assert_eq!(domain.forge.proposed_effect(71).is_some(), ending == authority::Answer::Propose);
        assert_unheld(&domain, key);
    }
}

#[test]
fn a_proposal_description_waits_for_room_and_reserves_its_acceptance() {
    for full in [true, false] {
        let (mut domain, env, key) = fixture_with_proposal(full, true);
        step(
            &mut domain,
            &env,
            forge::Event::KeepProposedEffect {
                row: forge::ProposedEffect { number: 71, resource: forge::What::Issue(1), entry: entry(0, key.task) },
            },
        );
        let owner = domain.engine.core_mut().effect_owner();
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        ask(&mut domain, &env, &mut decision, Ask::DescribeProposal { owner, proposal: 71 });
        if full {
            let Some(Work::Connector(
                _,
                jig_domain_contract::Up::Effects(jig_domain_contract::effects::Up::Busy { owner: received }),
            )) = domain.work.pop()
            else {
                panic!("jig answers a busy acceptance Wait, retaining its proposal")
            };
            assert_eq!(received, owner);
        } else {
            let Some(Work::Connector(
                _,
                jig_domain_contract::Up::Effects(jig_domain_contract::effects::Up::Described { .. }),
            )) = domain.work.pop()
            else {
                panic!("proposal is described into reserved room")
            };
            assert!(!domain.forge.effect_room(&env.limits.forge));
            let kept = keep(&mut domain, &env, key, owner, &mut decision);
            let mut saved = false;
            for request in &kept {
                if let forge::Request::Save { record: forge::Stored::Entry { entry, .. }, .. } = request {
                    saved |= entry.number == ENTRY;
                }
            }
            assert!(saved);
            step(&mut domain, &env, forge::Event::Withdraw { entry: ENTRY });
            assert!(domain.forge.effect_room(&env.limits.forge), "entry consumed, not leaked, its reservation");
        }
        assert!(domain.forge.proposed_effect(71).is_some());
        assert!(domain.engine.core().tasks.proposal(key.task, 71).is_some());
        assert!(domain.work.is_empty());
        assert_unheld(&domain, key);
    }
}

#[test]
fn a_full_outbox_does_not_hide_a_calls_permanent_refusal() {
    for (repository, too_large, expected) in [
        (api::Repository { forge: 1, repository: 99 }, false, Error::Missing),
        (api::Repository { forge: 3, repository: 1 }, false, Error::Forbidden),
        (adopted(1).provider, true, Error::Forbidden),
    ] {
        let (mut domain, mut env, key) = fixture(true);
        let mut other = adopted(3);
        other.project = 99;
        step(&mut domain, &env, forge::Event::Restore { record: forge::Stored::Repository(other) });
        if too_large {
            env.limits.forge.client.op_bytes = 1;
        }
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        route::effect_call(
            &mut domain,
            &env,
            &mut decision,
            ReplyTo::new(RIGHT),
            key,
            repository,
            forge::What::Issue(1),
            write(),
            None,
        );
        let answer = rendered(&mut domain, &env, decision);
        assert_eq!(answer, CallAnswer::ForgeEffectRefused(expected));
        assert!(domain.forge_effects.is_empty());
        assert!(domain.work.is_empty());
    }
}
