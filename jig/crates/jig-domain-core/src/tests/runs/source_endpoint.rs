//! One model, either source, with the same check at every observation and claim seam.
#![expect(clippy::disallowed_methods, reason = "stories build fixtures and inspect prepared charters")]
use super::run_choice::{account, inventory, login, placement_fixture, placement_fleet};
use crate::tests::*;
use jig_domain_link::{EndpointBound, Form, Grant, Hello, LoginState, Scheme, SourceName};

const TASK: u64 = 1;
const FIRST: Token = Token::new(7);
const SECOND: Token = Token::new(11);
const KEY: u32 = 1;
const LOGIN: u32 = 3;
const PROGRAM: u32 = 1;
const DIRECT: u32 = 3;

fn fixture(program: bool, key: bool) -> (Core, Limits) {
    fixture_with_other(program, key, false)
}
fn fixture_with_other(program: bool, key: bool, other: bool) -> (Core, Limits) {
    let (mut core, limits) = placement_fixture(accounts::Holder::Party(1), accounts::Holder::Party(1), true);
    core.settings.endpoints = Box::new([
        EndpointFacts {
            providers: Box::new([1]),
            direct: Box::new([]),
            number: PROGRAM,
            dialect: 3,
            program: Some(b"claude_code".as_slice().into()),
        },
        EndpointFacts { providers: Box::new([]), direct: Box::new([1]), number: DIRECT, dialect: 3, program: None },
    ]);
    core.settings.models[1].program = core.settings.models[0].program.clone();
    core.current.test_charter_mut(1).policy.alternatives = Box::new([core.settings.models[1].clone()]);
    let mut providers = core.settings.llm_providers.providers.to_vec();
    providers[0].offers.key = true;
    providers[0].offers.programs = Box::new([b"claude_code".as_slice().into()]);
    providers[1].offers.key = true;
    providers[1].offers.programs = Box::new([b"claude_code".as_slice().into()]);
    core.settings.llm_providers.providers = providers.into();
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    account(
        &mut core,
        &limits,
        accounts::Event::Restore {
            record: accounts::Stored::Account {
                list: None,
                number: 0,
                provider: 1,
                holder: accounts::Holder::Party(1),
                place: 0,
                uses: Some(Box::new([Use::Project(1)])),
                subject: Box::new([]),
                display: Box::new([]),
            },
        },
    );
    if program {
        account(
            &mut core,
            &limits,
            accounts::Event::Restore {
                record: accounts::Stored::Source {
                    account: 0,
                    number: LOGIN,
                    place: 0,
                    kind: accounts::Kind::Program,
                    name: b"personal".as_slice().into(),
                    held: None,
                },
            },
        );
        account(
            &mut core,
            &limits,
            accounts::Event::Restore {
                record: accounts::Stored::Issued {
                    number: LOGIN,
                    holder: accounts::Holder::Party(1),
                    name: b"personal".as_slice().into(),
                    signed_out: false,
                },
            },
        );
    }
    if key {
        restore_key_source(&mut core, &limits, 0, KEY);
    }
    if other {
        account(
            &mut core,
            &limits,
            accounts::Event::Restore {
                record: accounts::Stored::Account {
                    list: None,
                    number: 5,
                    provider: 2,
                    holder: accounts::Holder::Party(1),
                    place: 1,
                    uses: Some(Box::new([Use::Project(1)])),
                    subject: Box::new([]),
                    display: Box::new([]),
                },
            },
        );
        restore_key_source(&mut core, &limits, 5, 2);
    }
    account(&mut core, &limits, accounts::Event::Want);
    if other {
        key_grant(&mut core, &limits, 2);
    }
    if key {
        key_grant(&mut core, &limits, KEY);
    }
    core.fleet = placement_fleet(&core, &limits);
    fleet_event(
        &mut core,
        &limits,
        fleet::Event::Sources {
            sources: if program { Box::new([SourceName::new(b"personal").unwrap()]) } else { Box::new([]) },
        },
    );
    connect(&mut core, &limits, FIRST, &[PROGRAM, DIRECT], 8192, LoginState::Usable);
    connect(&mut core, &limits, SECOND, &[PROGRAM, DIRECT], 8192, LoginState::NotSignedIn);
    (core, limits)
}
fn restore_key_source(core: &mut Core, limits: &Limits, account_number: u32, number: u32) {
    account(
        core,
        limits,
        accounts::Event::Restore {
            record: accounts::Stored::Source {
                account: account_number,
                number,
                place: 1,
                kind: accounts::Kind::Key,
                name: Box::new([]),
                held: None,
            },
        },
    );
}
fn key_grant(core: &mut Core, limits: &Limits, source: u32) {
    account(
        core,
        limits,
        accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Key),
            account: source,
            generation: 1,
            valid: Duration::from_secs(60),
        }),
    );
}
fn fleet_event(core: &mut Core, limits: &Limits, event: fleet::Event) {
    let mut out = Queue::uncapped();
    fleet::step(&mut core.fleet, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet }, event, &mut out);
}
fn connect(core: &mut Core, limits: &Limits, channel: Token, endpoints: &[u32], window: u32, state: LoginState) {
    let mut bounds = fixtures::bounds();
    bounds.endpoints =
        endpoints.iter().map(|endpoint| EndpointBound { endpoint: *endpoint, window, output: 1024 }).collect();
    fleet_event(
        core,
        limits,
        fleet::Event::Hello {
            worker: u32::from(channel == SECOND),
            channel,
            hello: Hello {
                jobs: Box::new([]),
                bounds,
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
    );
    if channel != Token::new(0) {
        let logins: Box<[jig_domain_link::Login]> =
            if core.accounts.source(LOGIN).is_some() { Box::new([login(b"personal", state)]) } else { Box::new([]) };
        inventory(core, limits, channel, logins);
    }
}
fn prepare(core: &mut Core, limits: &Limits) -> (Box<RunCharter>, Box<[Grant]>) {
    send_task(core, limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: TASK });
    core.contexts.insert(TASK, core.tasks.run_context(TASK).unwrap()).unwrap();
    core.begin_transcript(TASK, 0, None);
    watch_outputs(core, limits, Event::BriefAssembled { task: TASK, failure: None });
    let attempt = core.workspace_flight(TASK).unwrap().1;
    let out = watch_outputs(
        core,
        limits,
        Event::WorkspacePrepared {
            task: TASK,
            attempt,
            holds: Box::new([]),
            writes: Some(Box::new([])),
            credentials: Box::new([]),
            workspace: None,
        },
    );
    for request in out {
        if let Request::Now(now) = request
            && let Now::RunPrepared { run, grants, .. } = *now
        {
            assert!(core.claim_bindings_usable(TASK, attempt));
            return (run, grants);
        }
    }
    panic!("run prepared");
}

#[test]
fn a_sources_endpoint_agrees_from_readiness_through_assignment() {
    for program in [false, true] {
        for channel in [FIRST, SECOND] {
            let (mut core, limits) = fixture(program, true);
            if channel == SECOND {
                fleet_event(&mut core, &limits, fleet::Event::Draining { channel: FIRST });
            }
            let source = if program && channel == FIRST { LOGIN } else { KEY };
            let endpoint = if source == LOGIN { PROGRAM } else { DIRECT };
            let coverage = core.account_coverage(tasks::Payer::Project(1), Some(1));
            assert_eq!(
                core.model_readiness(tasks::Payer::Project(1), coverage.as_slice(), &core.settings.models[0]).0,
                Readiness::Usable
            );
            let offer = core.offer(tasks::Payer::Project(1), 1, tasks::Models::ALL);
            assert!(offer.as_slice().iter().any(|row| row.model == 0 && row.accounts.as_slice() == [0]));
            assert_eq!(core.preview_llm(1, 1, tasks::Models::ALL, tasks::Payer::Project(1), 1).unwrap().main.model, 0);
            assert_eq!(core.account_llm(1, tasks::Models::ALL, tasks::Payer::Project(1), 1, 0).unwrap().main.model, 0);
            let choice = core.run_choice(TASK).unwrap();
            assert_eq!((choice.placement.channel, choice.accounts[0].source), (channel, source));
            let (run, grants) = prepare(&mut core, &limits);
            assert_eq!(run.policy.model.endpoint, endpoint);
            assert_eq!(run.policy.alternatives[0].endpoint, endpoint);
            assert_eq!(run.sources.as_ref(), [Some(source), Some(source)]);
            assert_eq!(grants.len(), 1);
            assert_eq!(grants[0].account, source);
            assert_eq!(
                grants[0].form,
                if source == LOGIN {
                    Form::Program(SourceName::new(b"personal").unwrap())
                } else {
                    Form::Credential(Scheme::Key)
                }
            );
            assert_eq!(core.settings.models[0].endpoint, PROGRAM);
            if program && channel == SECOND {
                inventory(&mut core, &limits, SECOND, Box::new([login(b"personal", LoginState::Usable)]));
                assert!(!core.claim_bindings_usable(TASK, core.proofs.get(&TASK).unwrap().attempt));
            }
        }
    }
}

#[test]
fn a_login_only_account_keeps_the_program_endpoint() {
    let (mut core, limits) = fixture(true, false);
    let (run, grants) = prepare(&mut core, &limits);
    assert_eq!(run.policy.model.endpoint, PROGRAM);
    assert_eq!(run.policy.alternatives[0].endpoint, PROGRAM);
    assert_eq!(grants[0].form, Form::Program(SourceName::new(b"personal").unwrap()));
    assert_eq!(grants[0].account, LOGIN);
}

#[test]
fn a_globally_limited_login_holds_both_ways() {
    let (mut core, limits) = fixture(true, true);
    account(&mut core, &limits, accounts::Event::Limited { account: LOGIN, reset: None });
    assert_eq!(
        core.run_choice(TASK),
        Err(RunWait::Account {
            account: Some(0),
            source: Some(LOGIN),
            why: accounts::WaitWhy::Source(accounts::State::Limited { until: None })
        })
    );
    assert!(core.offer(tasks::Payer::Project(1), 1, tasks::Models::ALL).is_empty());
}

#[test]
fn host_bounds_follow_the_source_not_the_stored_model() {
    for (endpoints, window, usable) in
        [(&[PROGRAM][..], 8192, false), (&[DIRECT][..], 8192, true), (&[DIRECT][..], 1, false)]
    {
        let (mut core, limits) = fixture(false, true);
        core.fleet = placement_fleet(&core, &limits);
        connect(&mut core, &limits, FIRST, endpoints, window, LoginState::NotSignedIn);
        assert_eq!(core.run_choice(TASK).is_ok(), usable);
        assert_eq!(!core.offer(tasks::Payer::Project(1), 1, tasks::Models::ALL).is_empty(), usable);
        let coverage = core.account_coverage(tasks::Payer::Project(1), Some(1));
        assert_eq!(
            core.model_readiness(tasks::Payer::Project(1), coverage.as_slice(), &core.settings.models[0]).0
                == Readiness::Usable,
            usable
        );
    }
    let (mut core, limits) = fixture(true, true);
    core.fleet = placement_fleet(&core, &limits);
    fleet_event(
        &mut core,
        &limits,
        fleet::Event::Sources { sources: Box::new([SourceName::new(b"personal").unwrap()]) },
    );
    connect(&mut core, &limits, FIRST, &[DIRECT], 8192, LoginState::Usable);
    connect(&mut core, &limits, SECOND, &[DIRECT], 8192, LoginState::NotSignedIn);
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!((choice.placement.channel, choice.accounts[0].source), (SECOND, KEY));
    let (run, _) = prepare(&mut core, &limits);
    assert_eq!(run.policy.model.endpoint, DIRECT);
    core.fleet = placement_fleet(&core, &limits);
    connect(&mut core, &limits, SECOND, &[PROGRAM], 8192, LoginState::NotSignedIn);
    assert!(!core.claim_bindings_usable(TASK, core.proofs.get(&TASK).unwrap().attempt));
    let (mut core, mut limits) = fixture(true, true);
    limits.fleet.engine_slots = 1;
    core.current.test_charter_mut(1).hosts = fleet::Kinds::Engine;
    core.fleet = placement_fleet(&core, &limits);
    connect(&mut core, &limits, Token::new(0), &[DIRECT], 8192, LoginState::Usable);
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!(choice.placement.kind, fleet::HostKind::Engine);
    assert_eq!(choice.accounts[0].source, KEY);
}

#[test]
fn another_providers_program_model_takes_its_own_direct_way() {
    for program_main in [false, true] {
        for direct in [false, true] {
            let (mut core, limits) = fixture_with_other(program_main, true, true);
            core.settings.models[2].program = Some(b"claude_code".as_slice().into());
            core.current.test_charter_mut(1).policy.alternatives =
                Box::new([core.settings.models[1].clone(), core.settings.models[2].clone()]);
            if direct {
                let mut endpoints = core.settings.endpoints.to_vec();
                endpoints.push(EndpointFacts {
                    providers: Box::new([2]),
                    direct: Box::new([]),
                    number: 2,
                    dialect: 3,
                    program: Some(b"claude_code".as_slice().into()),
                });
                endpoints.push(EndpointFacts {
                    providers: Box::new([]),
                    direct: Box::new([2]),
                    number: 4,
                    dialect: 3,
                    program: None,
                });
                core.settings.endpoints = endpoints.into();
            }
            core.fleet = placement_fleet(&core, &limits);
            fleet_event(
                &mut core,
                &limits,
                fleet::Event::Sources {
                    sources: if program_main {
                        Box::new([SourceName::new(b"personal").unwrap()])
                    } else {
                        Box::new([])
                    },
                },
            );
            connect(&mut core, &limits, FIRST, &[PROGRAM, 2, DIRECT, 4], 8192, LoginState::Usable);
            let (run, grants) = prepare(&mut core, &limits);
            assert_eq!(run.policy.alternatives.len(), if direct { 2 } else { 1 });
            assert_eq!(grants.len(), if direct { 2 } else { 1 });
            if direct {
                assert_eq!(run.policy.alternatives[1].endpoint, 4);
                assert_eq!(run.sources[2], Some(2));
                assert_eq!(grants[1].form, Form::Credential(Scheme::Key));
            }
        }
    }
}
