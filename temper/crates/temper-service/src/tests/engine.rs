use super::fixtures;
use skein_lib::{Duration, Time};
use temper_engine_domain as root;

#[test]
fn the_root_reports_the_earliest_credentials_expiry() {
    let (mut domain, mut env) = fixtures::root();
    env.now = Time::from_nanos(5_000_000_000);
    assert_eq!(domain.next_deadline(&env), None);
    for (number, valid, expected) in [(1, 7, 12_u64), (2, 2, 7)] {
        root::try_step(
            &mut domain,
            &env,
            root::Event::Credential { number, generation: 1, valid: Duration::from_secs(valid) },
        )
        .expect("credential admitted");
        assert_eq!(domain.next_deadline(&env), Some(Time::from_nanos(expected.checked_mul(1_000_000_000).unwrap())));
    }
}

#[test]
fn the_forge_reports_the_earliest_procedure_or_projection_wait() {
    let own = crate::limits::profile(crate::limits::Profile::Small);
    let mut domain = temper_engine_domain_forge::Domain::new(
        &own.forge,
        7,
        skein_forge_client::Config {
            namespace: Box::new([7; 16]),
            writers: Box::new([skein_forge_client::Writer { forge: 1, author: 42 }]),
        },
        &[temper_engine_domain_forge::GitIdentity { forge: 1, credential: 1 }],
        &[],
        false,
    )
    .unwrap();
    let env = skein_lib::Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: own.forge };
    assert_eq!(domain.next_wall_deadline(), None);
    domain.wait_change(&env, 1, Some(skein_lib::Wall::from_nanos(7)));
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(7)));
    domain.projection_after(2, skein_lib::Wall::from_nanos(3));
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(3)));
    domain.projection_settled(2);
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(7)));
    domain.offered_change(1);
    assert_eq!(domain.next_wall_deadline(), None);
}

#[test]
fn the_restart_asks_one_load_at_a_time() {
    let mut service = fixtures::started();
    for _ in 0_u32..16 {
        fixtures::turn(&mut service);
        if store_answered(&service) {
            break;
        }
    }
    let event = service.engine.as_mut().expect("engine").store.above.pop();
    let Some(jig_protocol_store::Event::Restored { owner, rows, next }) = event else {
        panic!("first restart load: {event:?}")
    };
    assert!(rows.is_empty());
    assert_eq!(next, None);
    assert!(service.engine.as_ref().expect("engine").store.above.is_empty());
    fixtures::turn(&mut service);
    assert!(service.engine.as_ref().expect("engine").store.above.is_empty(), "no next load before the first answer");
    service.engine.as_mut().expect("engine").store.above.push(jig_protocol_store::Event::Restored {
        owner,
        rows,
        next,
    });
    for _ in 0_u32..16 {
        fixtures::turn(&mut service);
        if !service.engine.as_mut().expect("engine").store.above.is_empty() {
            break;
        }
    }
    assert_eq!(service.end(), None);
    assert_eq!(service.engine.as_ref().expect("engine").store.above.len(), 1, "one next store operation");
    fixtures::restart(&mut service);
}
/// Whether the store has answered the engine. The accounts' answers, as
/// their handed-in record kept (jig's protocol/store.md, 9), are taken on
/// the next turn.
fn store_answered(service: &crate::Service) -> bool {
    for event in &service.engine.as_ref().expect("engine").store.above {
        let jig_protocol_store::Event::Kept { owner } = event else { return true };
        if !crate::accounts::owns(*owner) {
            return true;
        }
    }
    false
}

#[test]
fn a_corrupt_pending_restart_load_keeps_its_restore_kind() {
    let mut service = fixtures::started();
    for _ in 0_u32..16 {
        fixtures::turn(&mut service);
        if store_answered(&service) {
            break;
        }
    }
    let event = service.engine.as_mut().expect("engine").store.above.pop();
    let Some(jig_protocol_store::Event::Restored { owner, .. }) = event else {
        panic!("first restart load: {event:?}")
    };
    service.engine.as_mut().expect("engine").store.above.push(jig_protocol_store::Event::Corrupt { owner });
    for _ in 0_u32..256 {
        fixtures::turn(&mut service);
        if service.end().is_some() {
            break;
        }
    }
    assert_eq!(
        service.end(),
        Some(&crate::End::Stopped(crate::Stop::RestoreLoad(crate::RestoreFailure {
            range: crate::RestoreRangeKind::Deployment,
            failure: crate::LoadFailure::Store,
        })))
    );
}

#[test]
fn a_store_answer_is_taken_at_the_backlog_watermark() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    let mut number = 2;
    for _ in 0_u32..engine.env.limits.watermark.checked_add(1).unwrap() {
        if root::try_step(&mut engine.domain, &engine.env, fixtures::period(number)).is_err() {
            break;
        }
        number = number.checked_add(1).unwrap();
    }
    let refused = root::try_step(&mut engine.domain, &engine.env, fixtures::period(number))
        .expect_err("backlog is at the watermark");
    let root::Event::Party(root::PartyInput::Period { period, .. }) = &refused else {
        panic!("the original period is handed back")
    };
    assert_eq!(*period, number, "the original period is handed back");
    root::release(&mut engine.domain, &engine.env, &mut engine.requests);
    let Some(root::Request::Store(root::StoreRequest::Commit { number, writes })) = engine.requests.pop() else {
        panic!("commit")
    };
    assert!(engine.store.down(root::StoreRequest::Commit { number, writes }));
    crate::engine::iterate(engine, Time::from_nanos(1), skein_lib::Wall::from_nanos(1));
    for event in &engine.store.above {
        if let jig_protocol_store::Event::Committed { number: found, .. } = event {
            assert_ne!(*found, number, "original terminal taken while admission was blocked");
        }
    }
    assert!(root::try_step(&mut engine.domain, &engine.env, refused).is_ok(), "answer lowered the backlog");
}
#[test]
fn a_completion_passes_new_work_held_at_the_watermark() {
    use jig_domain_host as host;
    use skein_lib::{Token, Wall};
    let (mut service, run) = fixtures::claimed_chat();
    let engine = service.engine.as_mut().expect("engine");
    assert_eq!(engine.env.limits.journal.commits, 3);
    for number in 2..=u64::from(engine.env.limits.watermark) + 2 {
        if root::try_step(&mut engine.domain, &engine.env, fixtures::period(number)).is_err() {
            break;
        }
    }
    engine.step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up {
            up: host::LinkUp::Call {
                run: Token::new(run.task),
                attempt: Token::new(run.attempt),
                call: b"held-call".as_slice().into(),
                tool: b"unknown".as_slice().into(),
                writes: false,
                input: Box::new([]),
                deadline: Duration::from_secs(10),
            },
            parts: root::Parts::none(),
        },
    }));
    assert!(engine.blocked && engine.pending.is_some(), "the worker call waits");
    engine.events.push(root::Event::Worker(root::WorkerInput::ResultRead {
        payload: Token::new(9999),
        end: jig_domain_tasks::End::Finished {
            result: jig_domain_tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
    }));
    crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
    // No store answer has yet relieved the initial pressure. The completion
    // must already have passed the held call, rather than waiting for it.
    assert!(engine.events.is_empty(), "the later completion passes the held call");
    let Some(root::Event::Worker(root::WorkerInput::Link {
        link: root::Link::Up { up: host::LinkUp::Call { .. }, .. },
        ..
    })) = engine.pending.as_deref()
    else {
        panic!("the new call still waits");
    };
    for _ in 0..STARTUP_PASSES {
        crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
        if engine.pending.is_none() {
            break;
        }
    }
    assert!(engine.pending.is_none(), "the call is taken after the backlog drains");
    // The chat fixture plays the host directly, so capture its reply before
    // the service tries sending it to an unconnected transport channel.
    let mut delivered = skein_lib::Queue::with_capacity(32);
    fixtures::settle_root(&mut service, &mut delivered);
    fixtures::settle_root(&mut service, &mut delivered);
    let mut replies = 0_u32;
    for delivery in &delivered {
        if let root::Delivery::Link { down: host::LinkDown::Relayed { call, .. }, .. } = delivery
            && call.as_ref() == b"held-call"
        {
            replies += 1;
        }
    }
    assert_eq!(replies, 1, "the held call is consumed and answered once");
    assert!(service.engine.as_ref().expect("engine").domain.quiescent());
    assert_eq!(service.end(), None);
}

#[test]
fn a_failed_commit_releases_nothing_after_it() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    let first = engine.domain.deployment().commits.checked_add(1).unwrap();
    root::try_step(&mut engine.domain, &engine.env, fixtures::period(2)).unwrap();
    root::try_step(&mut engine.domain, &engine.env, fixtures::period(3)).unwrap();
    engine.store.above.push(jig_protocol_store::Event::Failed { number: first });
    engine.store.above.push(jig_protocol_store::Event::Committed { number: first.checked_add(1).unwrap(), fill: 0 });
    fixtures::turn(&mut service);
    assert_eq!(service.end(), Some(&crate::End::Stopped(crate::Stop::CommitFailed { number: first })));
    assert!(service.engine.as_ref().expect("engine").requests.is_empty(), "later outputs stayed behind the barrier");
    assert!(service.engine.as_ref().expect("engine").store.above.is_empty());
}

#[test]
fn a_frame_refusal_stops_with_its_details_and_releases_no_later_output() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    let number = engine.domain.deployment().commits.checked_add(1).unwrap();
    root::try_step(&mut engine.domain, &engine.env, fixtures::period(2)).unwrap();
    root::try_step(&mut engine.domain, &engine.env, fixtures::period(3)).unwrap();
    engine.store.above.push(jig_protocol_store::Event::FrameTooLarge { number, task: 11, bytes: 79, limit: 78 });
    engine.store.above.push(jig_protocol_store::Event::Committed { number: number.checked_add(1).unwrap(), fill: 0 });
    fixtures::turn(&mut service);
    assert_eq!(
        service.end(),
        Some(&crate::End::Stopped(crate::Stop::FrameTooLarge { number, task: 11, bytes: 79, limit: 78 }))
    );
    let engine = service.engine.as_ref().expect("engine");
    assert!(engine.requests.is_empty(), "later outputs stayed behind the existing Uncommitted barrier");
    assert!(engine.store.above.is_empty());
}

#[test]
fn kernel_operations_are_routed_back_to_their_owner() {
    use skein_io::kernel::{Complete, Done, Error, Fd, Op, Submit};
    use skein_lib::{Queue, Token};
    let mut kernel = crate::accounts::Kernel::new(4);
    let mut sockets = Queue::with_capacity(2);
    sockets.push(Submit { op: Token::new(1), kind: Op::Close { fd: Fd::new(6) } });
    let mut output = Queue::with_capacity(4);
    kernel.publish(&mut sockets, &mut output);
    let request = output.pop().expect("socket operation");
    sockets.push(Submit { op: Token::new(2), kind: Op::Cancel { target: Token::new(1) } });
    let complete =
        kernel.complete(Complete { op: request.op, kind: request.kind, result: Ok(Done::Nothing) }, &sockets);
    assert_eq!(complete.op, Token::new(1));
    kernel.publish(&mut sockets, &mut output);
    let cancel = output.pop().expect("cancel after target completion");
    assert_eq!(cancel.kind, Op::Cancel { target: request.op });
    let complete =
        kernel.complete(Complete { op: cancel.op, kind: cancel.kind, result: Err(Error::TooLate) }, &sockets);
    assert_eq!(complete.op, Token::new(2));
    assert_eq!(complete.kind, Op::Cancel { target: Token::new(1) });
}
#[test]
fn an_io_listener_failure_stops_the_engine() {
    let mut service = fixtures::started();
    service.told.push(skein_io::Event::Closed { owner: skein_lib::Token::new(crate::tokens::WORKER_LISTENERS) });
    fixtures::turn(&mut service);
    assert_eq!(
        service.end(),
        Some(&crate::End::Stopped(crate::Stop::Listener {
            listener: crate::Listening::Workers,
            address: Some(skein_io::Addr::InProcess(
                skein_io::Name::new(b"workers").expect("configured worker listener")
            )),
            error: None
        }))
    );
}

// Both note scopes and their store replies must fit before the held input is retried.
const OWNER_PASSES: u32 = 16;
const WATCHER: skein_lib::Token = skein_lib::Token::new(17);
// A notes page leaves its Core::Notes::Loaded continuation for the next Resume.
fn a_notes_page_waiting_for_its_owner() -> crate::Service {
    use skein_lib::{Queue, ReplyTo, Token, Wall};
    let mut service = fixtures::signed_worker();
    service.engine.as_mut().expect("engine").step(root::Event::Party(root::PartyInput::Watch {
        watcher: WATCHER,
        sign_in: 1,
        subject: jig_domain_core::documents::Watched::Frame,
        window: jig_domain_core::documents::Window { rows: 1, after: None },
    }));
    fixtures::settle_root(&mut service, &mut Queue::with_capacity(32));
    service.engine.as_mut().expect("engine").step(root::Event::Party(root::PartyInput::Delivered { watcher: WATCHER }));
    fixtures::settle_root(&mut service, &mut Queue::with_capacity(32));
    service.engine.as_mut().expect("engine").step(root::Event::Party(root::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(101)),
        sign_in: 1,
        key: [31; 16],
        ask: fixtures::chat(1, b"question"),
    }));
    // Startup is settled; the created task's preparation now reads its notes.
    for _ in 0..OWNER_PASSES {
        crate::engine::iterate(service.engine.as_mut().expect("engine"), Time::from_nanos(1), Wall::from_nanos(1));
        if let Some(jig_protocol_store::Event::Loaded { .. }) =
            service.engine.as_mut().expect("engine").store.above.iter().next()
        {
            let event = service.engine.as_mut().expect("engine").store.above.pop().expect("notes page");
            let engine = service.engine.as_mut().expect("engine");
            crate::store::route::up(&mut engine.store.routes, event, &mut engine.store.routed);
            let Some(crate::store::route::Up::Root(event)) =
                service.engine.as_mut().expect("engine").store.routed.pop()
            else {
                panic!("notes page routed to the root")
            };
            service.engine.as_mut().expect("engine").step(*event);
            return service;
        }
    }
    panic!("task preparation asks for notes");
}

#[test]
fn an_unwatch_refused_behind_owner_work_is_admitted() {
    use skein_lib::Wall;
    let mut service = a_notes_page_waiting_for_its_owner();
    let engine = service.engine.as_mut().expect("engine");
    engine.step(root::Event::Party(root::PartyInput::Unwatch { watcher: WATCHER }));
    let refused = engine.blocked && engine.pending.is_some();
    for _ in 0..OWNER_PASSES {
        crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
    }
    assert_eq!((refused, engine.pending.is_none(), engine.blocked), (true, true, false));
}

#[test]
fn a_refused_completion_precedes_later_inputs_and_new_work_keeps_its_order() {
    use skein_lib::Wall;
    let mut service = a_notes_page_waiting_for_its_owner();
    let engine = service.engine.as_mut().expect("engine");
    engine.step(fixtures::period(90));
    assert!(engine.pending.is_some(), "new work waits for the earlier owner hand-offs");
    engine.events.push(root::Event::Party(root::PartyInput::Unwatch { watcher: WATCHER }));
    engine.events.push(fixtures::period(91));
    engine.events.push(root::Event::Party(root::PartyInput::Delivered { watcher: WATCHER }));
    crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
    assert_held_period(engine.events.iter().next(), 91);
    crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
    assert_held_period(engine.events.iter().next(), 91);
    for _ in 0..OWNER_PASSES {
        if engine.pending.is_none() {
            break;
        }
        assert_held_period(engine.pending.as_deref(), 90);
        crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
    }
    assert!(engine.pending.is_none(), "the earlier new input is eventually taken");
    assert_held_period(engine.events.iter().next(), 91);
    for _ in 0..OWNER_PASSES {
        crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
    }
    assert!(engine.pending.is_none() && engine.events.is_empty());
    assert!(engine.end.is_none());
}

fn assert_held_period(event: Option<&root::Event>, expected: u64) {
    let Some(root::Event::Party(root::PartyInput::Period { period, .. })) = event else {
        panic!("the next new input retains its period");
    };
    assert_eq!(*period, expected, "new inputs keep their arrival order");
}

// The service's existing restart fixture drains within 256 iterations.
const STARTUP_PASSES: u32 = 256;

#[test]
fn startup_admission_does_not_promise_a_connected_forge() {
    use skein_lib::{Queue, ReplyTo, Token};
    use temper_engine_domain_forge as forge;
    let mut config = fixtures::engine();
    // An ordinary temper/ branch prefix needs eight authority segments.
    config.jig.limits.core.authority.segments = 8;
    fixtures::fill_files(&mut config);
    let limits = crate::limits::engine(&config.jig.limits, &config.limits).expect("root limits");
    let models = jig_protocol_config::core(&config.jig, [7; 16], 7).settings.models;
    let store = fixtures::memory_store_for(&limits, &config.jig.limits.store, &models);
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None, fixtures::forge_runtime()).unwrap();
    service.engine.as_mut().expect("engine").store = store;
    service.engine.as_mut().expect("engine").store.hold_mark = true;
    for _ in 0..STARTUP_PASSES {
        fixtures::turn(&mut service);
    }
    let engine = service.engine.as_mut().expect("engine");
    assert!(!engine.store.component.mark_durable());
    assert!(engine.clients.starting.is_some());
    let mark = engine.store.held_mark.take().expect("held startup mark");
    engine.store.hold_mark = false;
    assert!(engine.store.down(mark));
    // The fixture kernel holds Connect completions, independently of the mark.
    for _ in 0..STARTUP_PASSES {
        fixtures::turn(&mut service);
    }
    let engine = service.engine.as_mut().expect("engine");
    assert!(engine.store.component.mark_durable());
    assert!(engine.domain.ready());
    assert!(engine.clients.starting.is_none());
    assert_eq!(engine.domain.forge_connection(1), forge::Connection::NotConnected);
    engine.step(root::Event::Party(root::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(100)),
        identity: jig_domain_people::Identity {
            key: jig_domain_people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    fixtures::settle_root(&mut service, &mut Queue::with_capacity(32));
    service.engine.as_mut().expect("engine").step(startup_adopt(88));
    fixtures::settle_root(&mut service, &mut Queue::with_capacity(32));
    let rows = fixtures::load(&mut service.engine.as_mut().expect("engine").store, root::Range::Forge);
    let [root::Record::Forge { row }] = rows.as_ref() else { panic!("one durable application row: {rows:?}") };
    let forge::Stored::Application { token, outcome, .. } = row.as_ref() else { panic!("application row: {row:?}") };
    assert_eq!(*token, 1);
    assert_eq!(outcome, &forge::ApplicationOutcome::ForgeRefused(forge::AdoptionRefusal::Unreachable));
    let engine = service.engine.as_mut().expect("engine");
    engine.step(root::Event::Forge(Box::new(forge::Event::Connection {
        forge: 1,
        state: forge::Connection::Connected { id: 42, login: b"temper".as_slice().into() },
    })));
    fixtures::settle_root(&mut service, &mut Queue::with_capacity(32));
    let engine = service.engine.as_mut().expect("engine");
    engine.step(startup_adopt(89));
    let (repository, op) = startup_locate(engine);
    assert_eq!(repository.forge, 1);
    assert_eq!(
        op,
        skein_forge::api::Op::Read(skein_forge::api::Read::Locate {
            owner: b"acme".as_slice().into(),
            name: b"widget".as_slice().into(),
        })
    );
}

fn startup_locate(engine: &mut crate::engine::Engine) -> (skein_forge::api::Repository, skein_forge::api::Op) {
    for _ in 0..STARTUP_PASSES {
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        if engine.requests.is_empty() {
            engine.step(root::Event::Resume);
            root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        }
        while let Some(request) = engine.requests.pop() {
            match request {
                root::Request::Forge { repository, op, .. } => {
                    return (repository, op);
                }
                root::Request::Party(root::PartyRequest::Deliver(root::Delivery::Application { outcome, .. })) => {
                    panic!("no terminal answer before Locate: {outcome:?}");
                }
                root::Request::Party(_) => {}
                other @ (root::Request::Unsettled(_)
                | root::Request::Bound { .. }
                | root::Request::StopGroup { .. }
                | root::Request::SetUpGroup { .. }
                | root::Request::Return { .. }
                | root::Request::SetUp { .. }
                | root::Request::Connect { .. }
                | root::Request::Manifest { .. }
                | root::Request::Reachable { .. }
                | root::Request::Accept { .. }
                | root::Request::Install { .. }
                | root::Request::EraseCarried { .. }
                | root::Request::Setup { .. }
                | root::Request::Store(_)
                | root::Request::Worker(_)
                | root::Request::Account(_)
                | root::Request::ToChild(_)
                | root::Request::Busy(_)
                | root::Request::Drained
                | root::Request::Stop) => crate::route::request(engine, other),
            }
        }
        crate::engine::answer(engine);
        engine.domain.reclaim();
    }
    panic!("Connected releases adoption's Locate");
}

fn startup_adopt(key: u8) -> root::Event {
    use skein_lib::{ReplyTo, Token};
    use temper_engine_domain_forge as forge;
    root::Event::Party(root::PartyInput::Application {
        scope: jig_domain_people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(101)),
        sign_in: 1,
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request: root::Application::Forge(forge::Ask::Adopt(forge::Adoption {
            project: 1,
            home: true,
            forge: 1,
            host: b"forge.example".as_slice().into(),
            owner: b"acme".as_slice().into(),
            name: b"widget".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        })),
    })
}

#[test]
fn nothing_leaves_before_the_mark() {
    let previous = fixtures::ready().engine.expect("previous engine");
    let mut service = fixtures::restarting_from(previous);
    service.engine.as_mut().expect("engine").store.hold_mark = true;
    for _ in 0..STARTUP_PASSES {
        fixtures::turn(&mut service);
    }
    assert!(service.links().is_empty(), "no sign-in link is published before startup opens");
    let engine = service.engine.as_mut().expect("engine");
    assert!(engine.store.held_mark.is_some(), "public restore reaches its mark without secret bootstrap");
    assert!(!engine.store.component.mark_durable());
    assert!(!engine.accounts.as_ref().expect("accounts").startup_state().0);
    assert!(engine.accounts.as_ref().expect("accounts").startup_state().1 > 0, "early account requests retained");
    assert_eq!(engine.store.secret_requests, 0);
    assert!(engine.store.forge_loads.is_empty());
    assert!(!engine.forge_records.done());
    assert!(engine.forge.starts.is_empty());
    assert!(!engine.forge.startup_begun());
    assert!(!engine.setup.startup_restored());
    assert!(!engine.work_pending(Time::from_nanos(1)), "a held mark waits for its storage answer");
    let mark = engine.store.held_mark.take().expect("held mark");
    engine.store.hold_mark = false;
    assert!(engine.store.down(mark));
    fixtures::restart(&mut service);
    let engine = service.engine.as_ref().expect("engine");
    assert!(engine.store.component.mark_durable());
    assert_eq!(engine.accounts.as_ref().expect("accounts").startup_state(), (true, 0));
    assert_eq!(engine.store.forge_loads.as_slice(), [None], "one application-secrets pass");
    assert!(engine.forge_records.done());
    assert!(engine.forge.startup_begun());
    assert!(engine.setup.startup_restored());
    let secrets = engine.store.secret_requests;
    for _ in 0..STARTUP_PASSES {
        fixtures::turn(&mut service);
    }
    let engine = service.engine.as_ref().expect("engine");
    assert_eq!(engine.accounts.as_ref().expect("accounts").startup_state(), (true, 0));
    assert_eq!(engine.store.secret_requests, secrets, "bootstrap is not repeated");
    assert_eq!(engine.store.forge_loads.as_slice(), [None]);
}

#[test]
fn a_zero_byte_budget_refuses_direct_engine_startup() {
    let mut settings = fixtures::engine();
    settings.jig.memory = 0;
    fixtures::fill_files(&mut settings);
    let problems = crate::Service::engine(settings, &fixtures::seeds(), None, fixtures::forge_runtime())
        .err()
        .expect("a zero byte budget refuses direct startup");
    assert_eq!(problems.problems.len(), 1);
    let problem = problems.problems.get(0).unwrap();
    assert_eq!(problem.path, jig_protocol_config::Path::root().member(b"engine").member(b"memory_bytes"));
    assert_eq!(problem.what, jig_protocol_config::What::Bound);
}

#[test]
fn a_started_engines_root_uses_the_documents_engine_memory_bytes() {
    let mut settings = fixtures::engine();
    // A distinct engine value: worker memory has a separate parsed document.
    settings.jig.memory += 1;
    let memory = settings.jig.memory;
    fixtures::fill_files(&mut settings);
    let service = crate::Service::engine(settings, &fixtures::seeds(), None, fixtures::forge_runtime()).unwrap();
    assert_eq!(service.engine.as_ref().expect("started engine").env.limits.memory_bytes, memory);
}

#[test]
fn a_cancellation_waiting_unpublished_keeps_its_completed_targets_route() {
    use skein_io::kernel::{Complete, Done, Error, Fd, Op, Submit};
    use skein_lib::{Token, Wall};
    let mut service = fixtures::ready();
    let fd = Fd::new(93);
    service.adopt_signals(fd).unwrap();
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let mut target = None;
    for _ in 0..service.submissions.len() {
        let submit = service.submissions.pop().unwrap();
        if let Op::ReadSignal { fd: found } = submit.kind
            && found == fd
        {
            target = Some(submit);
        } else {
            service.submissions.push(submit);
        }
    }
    let target = target.expect("the signal source's read reached the kernel");
    while service.submissions.room() > 0 {
        service.submissions.push(Submit { op: Token::new(9000), kind: Op::Close { fd: Fd::new(94) } });
    }
    service.engine.as_mut().unwrap().end = Some(crate::End::Aborted);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.submissions.room(), 0);
    assert!(service.pending(Time::ZERO).signals_closing);
    assert!(service.pending(Time::ZERO).raw_submissions > 0, "the cancellation waits unpublished");
    let external = target.op;
    service.completions.push(Complete { op: external, kind: target.kind, result: Err(Error::Cancelled) });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.completions.is_empty(), "the target completion reached its owner while its cancellation waited");
    assert!(service.pending(Time::ZERO).signals, "the source still waits for its cancellation terminal");
    while service.submissions.pop().is_some() {}
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let mut cancel = None;
    for _ in 0..service.submissions.len() {
        let submit = service.submissions.pop().unwrap();
        if let Op::Cancel { target } = submit.kind
            && target == external
        {
            cancel = Some(submit);
        } else {
            if let Op::Close { fd: found } = submit.kind {
                assert_ne!(found, fd, "close waits for the cancel owner");
            }
            service.submissions.push(submit);
        }
    }
    let cancel = cancel.expect("the delayed cancellation retains its target's external route");
    service.completions.push(Complete { op: cancel.op, kind: cancel.kind, result: Err(Error::TooLate) });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let mut close = None;
    for _ in 0..service.submissions.len() {
        let submit = service.submissions.pop().unwrap();
        if let Op::Close { fd: found } = submit.kind
            && found == fd
        {
            close = Some(submit);
        } else {
            service.submissions.push(submit);
        }
    }
    let close = close.expect("the cancellation maps back to its source, which can now close");
    service.completions.push(Complete { op: close.op, kind: close.kind, result: Ok(Done::Nothing) });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(!service.pending(Time::ZERO).signals, "the source's owner observes its final close");
}

#[test]
fn a_released_batch_is_routed_before_the_next_release() {
    use crate::engine::{ReleaseObservation, ReleaseTrace};
    use skein_lib::{Queue, Wall};
    let mut service = fixtures::ready();
    let now = Time::from_nanos(1);
    let wall = Wall::from_nanos(1);
    let engine = service.engine.as_mut().expect("engine");
    engine.release_trace = Some(ReleaseTrace::new());
    // An unsigned watch stages an immediate refusal. Period decisions queue
    // commits beside it, so a journal release can return an output and a commit.
    engine.step(root::Event::Party(root::PartyInput::Watch {
        watcher: skein_lib::Token::new(701),
        sign_in: u64::MAX,
        subject: jig_domain_core::documents::Watched::Frame,
        window: jig_domain_core::documents::Window { rows: 1, after: None },
    }));
    engine.step(fixtures::period(2));
    assert!(engine.pending.is_none(), "the watch and first period are admitted");
    engine.events.push(fixtures::period(3));
    engine.events.push(fixtures::period(4));
    crate::iterate(&mut service, now, wall);
    let engine = service.engine.as_ref().expect("engine");
    assert_held_period(engine.events.iter().next(), 4);
    let mut batch_while_waiting = false;
    for observation in &engine.release_trace.as_ref().expect("enabled release recording").observations {
        if let ReleaseObservation::Released { requests, .. } = observation
            && requests.len() > 1
        {
            batch_while_waiting = true;
        }
    }
    assert!(batch_while_waiting, "a multi-request release runs while the later input waits");
    crate::iterate(&mut service, now, wall);
    let engine = service.engine.as_mut().expect("engine");
    assert!(engine.events.is_empty(), "the later admitted input is eventually taken");
    let trace = engine.release_trace.as_mut().expect("enabled release recording");
    let mut expected = Queue::uncapped();
    let mut multi_request_release = None;
    let mut later_release = false;
    while let Some(observation) = trace.observations.pop() {
        match observation {
            ReleaseObservation::Released { ordinal, requests, .. } => {
                assert!(
                    expected.is_empty(),
                    "every request is dispatched before the next release, including empty releases"
                );
                if !requests.is_empty() && multi_request_release.is_some() {
                    later_release = true;
                }
                if requests.len() > 1 {
                    multi_request_release = Some(ordinal);
                }
                for request in requests.into_boxed() {
                    expected.push((ordinal, request));
                }
            }
            ReleaseObservation::Dispatched { ordinal, request } => {
                let (released, original) = expected.pop().expect("dispatch names a recorded release");
                assert_eq!(ordinal, Some(released), "dispatch keeps its release ordinal");
                assert_eq!(request, original, "requests reach routing in release order");
            }
        }
    }
    assert!(expected.is_empty(), "the last released batch is dispatched completely");
    assert!(multi_request_release.is_some(), "the scenario releases more than one request in a batch");
    assert!(later_release, "a later nonempty release exercises the ordering boundary");
    assert_eq!(service.end(), None);
}

#[test]
fn a_root_productive_past_one_quantum_lets_a_waiting_answer_in() {
    use crate::engine::{ReleaseObservation, ReleaseTrace};
    use skein_lib::{Token, Wall};
    // The shipped small profile's scheduling quantum, pinned independently of output limits.
    const QUANTUM: u32 = 22;
    const INPUTS: u32 = 13;
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    // The client staging queues leave the scheduling quantum as the yield point.
    let settings = fixtures::engine();
    engine.clients = crate::clients::Clients::new(
        &settings.jig,
        &settings.clients,
        &fixtures::seeds(),
        &settings.limits.root.journal,
        None,
        &[],
    )
    .unwrap();
    let mut trace = ReleaseTrace::new();
    // Each routed refusal lets the scripted neighbour ask again. Every cycle
    // uses the real root's admission, journal, release and request routing.
    for watcher in 0..INPUTS {
        trace.inputs.push(root::Event::Party(root::PartyInput::Watch {
            watcher: Token::new(1000 + u64::from(watcher)),
            sign_in: u64::MAX,
            subject: jig_domain_core::documents::Watched::Frame,
            window: jig_domain_core::documents::Window { rows: 1, after: None },
        }));
    }
    engine.step(trace.inputs.pop().expect("first productive input"));
    engine.release_trace = Some(trace);
    engine.store.above.push(jig_protocol_store::Event::Patched {
        owner: Token::new(999),
        of: jig_protocol_store::Owner::Application,
        patch: 999,
        counts: jig_protocol_store::Counts { records: 1, commits: 1 },
    });
    // Enter the release stage with an answer already waiting; the next service
    // pass must take that answer before another quantum of productive work.
    crate::engine::release(engine);
    let trace = engine.release_trace.as_ref().expect("release trace");
    let mut dispatched = 0;
    for observation in &trace.observations {
        if let ReleaseObservation::Dispatched { .. } = observation {
            dispatched += 1;
        }
    }
    assert_eq!(trace.cycles, QUANTUM, "a productive root yields at its scheduling quantum");
    assert!(dispatched > 0 && dispatched < INPUTS, "root output spans the quantum boundary");
    assert!(!trace.inputs.is_empty(), "more root work remains");
    assert!(engine.ready, "quantum exhaustion leaves the root ready");
    assert_eq!(engine.store.above.len(), 1, "the answer waits throughout the first quantum");
    assert!(engine.work_pending(Time::from_nanos(1)), "the yielded root keeps the loop awake");
    crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
    assert!(engine.store.above.is_empty(), "the waiting answer is taken between quanta");
    let mut answered = false;
    for note in &engine.notes {
        if let crate::Note::Patched { patch: 999, .. } = note {
            answered = true;
        }
    }
    assert!(answered, "the waiting answer reaches its owner");
    let trace = engine.release_trace.as_ref().expect("release trace");
    assert!(trace.inputs.is_empty(), "the root resumes on the next pass");
    let mut released = 0;
    for observation in &trace.observations {
        if let ReleaseObservation::Released { requests, waiting_answers, .. } = observation
            && !requests.is_empty()
        {
            assert_eq!(*waiting_answers, u32::from(released < dispatched), "answer precedes the next quantum");
            released += 1;
        }
    }
    assert_eq!(released, INPUTS, "all productive inputs are eventually routed");
    assert_eq!(engine.end, None);
}

#[test]
fn an_oversized_startup_page_keeps_its_load_range_and_failed_step_before_switch() {
    let mut service = fixtures::started();
    for _ in 0_u32..16 {
        fixtures::turn(&mut service);
        if store_answered(&service) {
            break;
        }
    }
    let engine = service.engine.as_mut().unwrap();
    let Some(jig_protocol_store::Event::Restored { owner, .. }) = engine.store.above.pop() else {
        panic!("startup page")
    };
    let header = jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
        temper_engine_domain_world::store::HEADER,
    ));
    let mut rows = skein_lib::List::with_capacity(engine.env.limits.loads.rows + 1);
    for _ in 0..=engine.env.limits.loads.rows {
        rows.push(jig_protocol_store::Row::Jig(header.clone())).expect("one row over demand");
    }
    let rows = rows.into_boxed();
    engine.store.above.push(jig_protocol_store::Event::Restored { owner, rows, next: None });
    for _ in 0_u32..256 {
        fixtures::turn(&mut service);
        if service.end().is_some() {
            break;
        }
    }
    assert_eq!(
        service.end(),
        Some(&crate::End::Stopped(crate::Stop::RestoreLoad(crate::RestoreFailure {
            range: crate::RestoreRangeKind::Deployment,
            failure: crate::LoadFailure::Store
        })))
    );
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.domain.restart_failure(), Some(jig_domain_core::RestartStep::LoadCore));
    assert!(!engine.domain.ready());
    assert!(engine.store.routes.idle());
}
