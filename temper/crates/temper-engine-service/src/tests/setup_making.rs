//! Kept repository rows meet the setup host (temper/docs/design/protocol/setup/owned.md, 9).
use super::*;
use crate::setup::making;
use git::api::Fill;

const FORGE: u16 = 1;
const ORG: &[u8] = b"work";
const NAME: &[u8] = b"copied";
const FULL: &[u8] = b"work/copied";
const ADDRESS: &[u8] = b"https://source.invalid/upstream/starter";
const SOURCE: &[u8] = b"upstream/starter";
// The catalogue source is git-only; the protocol assigns the first destination id one.
const SOURCE_ID: u32 = 2;
const DESTINATION_ID: u64 = 1;
// The shared host settles within 20,000 millisecond turns; no real time passes.
const TURNS: u32 = 20_000;
// The shared host advances four root turns per IO turn and bounds journal settlement at 256 passes.
const ROOT_TURNS: u32 = 4;
const ROOT_PASSES: u32 = 256;
// protocol/setup/owned.md, 9: one writing run per forge.
const WRITERS: u32 = 1;

fn host() -> (Forge, crate::Service) {
    host_with_local(false)
}
fn host_with_local(keep_local: bool) -> (Forge, crate::Service) {
    let mut forge = Forge::with_sources(
        false,
        Box::new([fake::Source {
            address: ADDRESS.into(),
            repository: SOURCE.into(),
            private: false,
            fill: Fill::Whole,
        }]),
    );
    git::repository(
        &mut forge.git,
        &forge.config,
        git::api::Setup {
            id: SOURCE_ID,
            name: SOURCE.into(),
            default: b"trunk".as_slice().into(),
            tree: Box::new([git::api::File {
                path: b"README.md".as_slice().into(),
                content: b"# starter\n".as_slice().into(),
            }]),
            labels: Box::new([]),
            checks: git::api::Checks {
                cues: Box::new([]),
                contexts: Box::new([]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 0,
                reruns: 0,
                cue: None,
            },
            workflows: Box::new([]),
            protection: None,
            hooked: false,
        },
    );
    git::grant(&mut forge.git, SOURCE, 1, git::api::Permission::Read);
    let mut service = configured(&forge);
    let engine = service.engine.as_mut().unwrap();
    // Connection sequencing has its own tests; making starts at its connected seam.
    if !keep_local {
        engine.setup.lifecycle.reconcile.remove(&FORGE);
    }
    root::step(
        &mut engine.domain,
        &engine.env,
        root::Event::Forge(Box::new(domain::Event::Connection {
            forge: FORGE,
            state: domain::Connection::Connected { id: 42, login: b"temper".as_slice().into() },
        })),
    );
    (forge, service)
}
fn turn(forge: &mut Forge, service: &mut crate::Service) {
    let engine = service.engine.as_mut().unwrap();
    while engine.notes.pop().is_some() {}
    forge.env.now = forge.env.now.checked_add(Duration::from_millis(1)).unwrap();
    engine.env.now = forge.env.now;
    advance(engine);
    // This host owns setup traffic; ordinary forge traffic is covered by its own world.
    while engine.forge.io.pop().is_some() {}
    forge.turn_io(engine);
    forge.turn_server(engine);
    for _ in 0..ROOT_TURNS {
        root_turn(engine);
    }
    forge.service.reclaim();
}
fn row(name: &[u8], from: domain::MakeFrom) -> domain::RepositoryMaking {
    domain::RepositoryMaking {
        key: domain::MakingKey { forge: FORGE, org: ORG.into(), name: name.into() },
        from,
        into: None,
        asked: Wall::EPOCH,
        copying: false,
        state: domain::MakingState::Waiting,
    }
}
fn copy() -> domain::RepositoryMaking {
    row(NAME, domain::MakeFrom::Copy { address: ADDRESS.into() })
}
fn restore(service: &mut crate::Service, row: domain::RepositoryMaking) {
    let engine = service.engine.as_mut().unwrap();
    root::step(
        &mut engine.domain,
        &engine.env,
        root::Event::Forge(Box::new(domain::Event::Restore { record: domain::Stored::Making(row) })),
    );
}
fn finish(forge: &mut Forge, service: &mut crate::Service, key: &domain::MakingKey) {
    for _ in 0..TURNS {
        turn(forge, service);
        let engine = service.engine.as_ref().unwrap();
        if engine.domain.making(key).is_none() && engine.setup.runs.is_empty() && forge.socket.is_none() {
            forge.record_requests();
            return;
        }
        if let Some(row) = engine.domain.making(key)
            && let domain::MakingState::Failed(failed) = &row.state
        {
            panic!("making stopped: {failed:?}");
        }
    }
    panic!("making did not finish: {:?}", service.engine.as_ref().unwrap().domain.making(key));
}
fn starter(forge: &Forge, full: &[u8]) -> Box<[u8]> {
    let branch = forge.git.metadata(full).unwrap().default_branch;
    let head = forge.git.branch(full, &branch).unwrap();
    forge.git.object(head).unwrap().tree.get(temper_forge_setup::STARTER_PATH).unwrap().clone()
}
#[test]
fn a_copy_runs_on_a_connected_forge_and_is_made() {
    let (mut forge, mut service) = host();
    let row = copy();
    let key = row.key.clone();
    restore(&mut service, row);
    finish(&mut forge, &mut service, &key);
    assert!(forge.service.setup().is_private(DESTINATION_ID));
    assert_eq!(starter(&forge, FULL), temper_forge_setup::starter_workflow(b"temper-host").unwrap());
    let head = forge.git.branch(FULL, b"trunk").unwrap();
    assert_eq!(forge.git.object(head).unwrap().tree.get(b"README.md".as_slice()).unwrap().as_ref(), b"# starter\n");
    assert!(migrates(&forge));
    assert!(service.engine.as_ref().unwrap().domain.making(&key).is_none());
}
#[test]
fn an_empty_making_runs_create_with_the_starter() {
    let (mut forge, mut service) = host();
    let row = row(NAME, domain::MakeFrom::Empty);
    let key = row.key.clone();
    restore(&mut service, row);
    finish(&mut forge, &mut service, &key);
    assert_eq!(starter(&forge, FULL), temper_forge_setup::starter_workflow(b"temper-host").unwrap());
    let head = forge.git.branch(FULL, b"main").unwrap();
    assert_eq!(forge.git.object(head).unwrap().tree.get(b"README.md".as_slice()).unwrap().as_ref(), b"# copied\n");
    assert!(!migrates(&forge));
}

fn begin_row(service: &mut crate::Service, row: domain::RepositoryMaking) -> Token {
    let key = row.key.clone();
    restore(service, row);
    lifecycle::advance(service.engine.as_mut().unwrap());
    run_for(service.engine.as_ref().unwrap(), &key).unwrap()
}
fn held_row(service: &mut crate::Service) -> (Token, domain::MakingKey) {
    let row = copy();
    let key = row.key.clone();
    restore(service, row);
    let engine = service.engine.as_mut().unwrap();
    let token = lifecycle::next_run(&mut engine.setup.lifecycle);
    engine
        .setup
        .runs
        .insert(
            token,
            Run {
                making: Some(key.clone()),
                words: None,
                first: None,
                previous: engine.domain.forge_connection(FORGE),
                forge: FORGE,
                kind: Kind::Making,
                internal: true,
            },
        )
        .unwrap();
    (token, key)
}
fn need(engine: &mut crate::engine::Engine, run: Token, need: host::Need) {
    engine.setup.above.push(host::Event::Need { run, need });
    advance(engine);
}
fn writing_run(service: &mut crate::Service) -> Token {
    let engine = service.engine.as_mut().unwrap();
    let token = lifecycle::next_run(&mut engine.setup.lifecycle);
    start(service, token.raw(), host::Plan::Reconcile { forge: FORGE, adopted: Box::new([]) });
    token
}
#[test]
fn a_row_waits_for_the_forges_writing_run() {
    let (_forge, mut service) = host();
    let token = writing_run(&mut service);
    let row = copy();
    let key = row.key.clone();
    restore(&mut service, row);
    let engine = service.engine.as_mut().unwrap();
    lifecycle::advance(engine);
    assert_eq!(engine.setup.runs.len(), WRITERS);
    assert_eq!(engine.domain.making(&key).unwrap().state, domain::MakingState::Waiting);
    assert_not_busy(engine);
    engine.setup.runs.remove(&token);
    engine.setup.down(host::Request::Remove { forge: FORGE });
    let config = engine.setup.served.get(&FORGE).unwrap().clone();
    engine.setup.down(host::Request::Add(config));
    lifecycle::advance(engine);
    assert!(run_for(engine, &key).is_some());
}
#[test]
fn one_row_runs_at_a_time() {
    let (_forge, mut service) = host();
    let first = begin_row(&mut service, copy());
    let next = row(b"second", domain::MakeFrom::Empty);
    let key = next.key.clone();
    restore(&mut service, next);
    let engine = service.engine.as_mut().unwrap();
    lifecycle::advance(engine);
    assert_eq!(engine.setup.runs.len(), WRITERS);
    assert!(engine.setup.runs.contains_key(&first));
    assert_eq!(engine.domain.making(&key).unwrap().state, domain::MakingState::Waiting);
    assert_not_busy(engine);
}
fn connection(service: &mut crate::Service, state: domain::Connection) {
    let engine = service.engine.as_mut().unwrap();
    root::step(
        &mut engine.domain,
        &engine.env,
        root::Event::Forge(Box::new(domain::Event::Connection { forge: FORGE, state })),
    );
}
fn connect(service: &mut crate::Service) {
    connection(service, domain::Connection::Connected { id: 42, login: b"temper".as_slice().into() });
}
#[test]
fn a_row_waits_for_its_forge_to_connect() {
    let (_forge, mut service) = host();
    connection(&mut service, domain::Connection::NotConnected);
    let row = copy();
    let key = row.key.clone();
    restore(&mut service, row);
    let engine = service.engine.as_mut().unwrap();
    lifecycle::advance(engine);
    assert!(engine.setup.runs.is_empty());
    assert_eq!(engine.domain.making(&key).unwrap().state, domain::MakingState::Waiting);
    connect(&mut service);
    let engine = service.engine.as_mut().unwrap();
    assert!(making::ready(engine));
    lifecycle::advance(engine);
    assert_eq!(engine.setup.runs.len(), WRITERS);
}
#[test]
fn an_excluded_forge_keeps_its_rows() {
    let (_forge, mut service) = host();
    let row = copy();
    let key = row.key.clone();
    restore(&mut service, row);
    let engine = service.engine.as_mut().unwrap();
    engine.setup.served.remove(&FORGE);
    lifecycle::advance(engine);
    assert!(!making::ready(engine));
    assert!(engine.setup.runs.is_empty());
    assert_eq!(engine.domain.making(&key).unwrap().state, domain::MakingState::Waiting);
}
#[test]
fn copying_is_answered_from_the_row() {
    let (_forge, mut service) = host();
    let (run, key) = held_row(&mut service);
    let engine = service.engine.as_mut().unwrap();
    need(engine, run, host::Need::Read(skein_forge_setup::item::Item::Copying));
    assert_eq!(engine.setup.answers.pop(), Some((run, host::Answer::Read(None))));
    need(
        engine,
        run,
        host::Need::Write { item: skein_forge_setup::item::Item::Copying, bytes: Secret::new(Box::from([1])) },
    );
    assert!(engine.setup.answers.is_empty(), "Written waits for the connector's kept mark");
    let mut wrong = key.clone();
    wrong.name = b"another".as_slice().into();
    crate::route::request(engine, root::Request::MakingCopied { key: wrong });
    assert!(engine.setup.answers.is_empty(), "acknowledgements are keyed by the row");
    for _ in 0..ROOT_PASSES {
        root_turn(engine);
        if !engine.setup.answers.is_empty() {
            break;
        }
    }
    assert!(engine.domain.making(&key).unwrap().copying);
    assert_eq!(engine.setup.answers.pop(), Some((run, host::Answer::Written)));
    need(engine, run, host::Need::Read(skein_forge_setup::item::Item::Copying));
    let Some((read_run, host::Answer::Read(Some(value)))) = engine.setup.answers.pop() else { panic!("kept mark") };
    assert_eq!(read_run, run);
    assert_eq!(value.expose(), &[1]);
}
#[test]
fn copying_never_reaches_the_records() {
    let (_forge, mut service) = host();
    let (run, _key) = held_row(&mut service);
    let engine = service.engine.as_mut().unwrap();
    need(
        engine,
        run,
        host::Need::Write { item: skein_forge_setup::item::Item::Copying, bytes: Secret::new(Box::from([1])) },
    );
    assert!(engine.setup.writes.is_empty(), "no credential write is staged for Copying");
    assert_eq!(item_record(skein_forge_setup::item::Item::Copying), None);
    let mut found = false;
    for event in &engine.events {
        if let root::Event::Forge(event) = event
            && let domain::Event::MakingCopying { .. } = event.as_ref()
        {
            found = true;
        }
    }
    assert!(found, "Copying is sent to the connector");
}
fn failure(why: reconcile::Why) -> reconcile::Failure {
    reconcile::Failure { forge: FORGE, step: reconcile::Stage::Repository, subject: FULL.into(), why }
}
fn stop_event(engine: &mut crate::engine::Engine) -> domain::MakingFailed {
    while let Some(event) = engine.events.pop() {
        if let root::Event::Forge(event) = event
            && let domain::Event::MakingStopped { failed, .. } = *event
        {
            return failed;
        }
    }
    panic!("making stop");
}
#[test]
fn said_words_reach_the_row_on_the_stop() {
    let (_forge, mut service) = host();
    let (run, key) = held_row(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let words = b"Forgejo refused this source";
    engine.setup.above.push(host::Event::Said { run, words: words.as_slice().into() });
    engine.setup.above.push(host::Event::Stop { run, failure: failure(reconcile::Why::Refused(422)) });
    advance(engine);
    for _ in 0..ROOT_PASSES {
        root_turn(engine);
        if let domain::MakingState::Failed(_) = engine.domain.making(&key).unwrap().state {
            break;
        }
    }
    assert_eq!(
        engine.domain.making(&key).unwrap().state,
        domain::MakingState::Failed(domain::MakingFailed {
            step: domain::MakeStep::Repository,
            reason: domain::MakeFailure::Refused { status: 422 },
            words: words.as_slice().into(),
        })
    );
}
#[test]
fn taken_empty_and_timed_out_are_typed() {
    let (_forge, mut service) = host();
    for (why, reason, words) in [
        (reconcile::Why::Taken, domain::MakeFailure::Taken, b"setup Repository work/copied: Taken".as_slice()),
        (reconcile::Why::Empty, domain::MakeFailure::Empty, b"setup Repository work/copied: Empty".as_slice()),
        (reconcile::Why::TimedOut, domain::MakeFailure::TimedOut, b"setup Repository work/copied: TimedOut".as_slice()),
        (
            reconcile::Why::Refused(403),
            domain::MakeFailure::Refused { status: 403 },
            b"setup Repository work/copied: Refused(403)".as_slice(),
        ),
        (reconcile::Why::Down, domain::MakeFailure::Other, b"setup Repository work/copied: Down".as_slice()),
    ] {
        let (run, _key) = held_row(&mut service);
        let engine = service.engine.as_mut().unwrap();
        let failure = failure(why);
        lifecycle::stopped(engine, run, failure);
        assert_eq!(
            stop_event(engine),
            domain::MakingFailed { step: domain::MakeStep::Repository, reason, words: words.into() }
        );
    }
}
#[test]
fn a_busy_root_does_not_lose_an_end() {
    let (_forge, mut service) = host();
    let engine = service.engine.as_mut().unwrap();
    let key = copy().key;
    for event in [
        domain::Event::MakingStarted { key: key.clone() },
        domain::Event::MakingCopying { key: key.clone() },
        domain::Event::MakingMade { key: key.clone(), id: DESTINATION_ID, branch: b"main".as_slice().into() },
        domain::Event::MakingStopped {
            key: key.clone(),
            failed: domain::MakingFailed {
                step: domain::MakeStep::Repository,
                reason: domain::MakeFailure::Taken,
                words: Box::new([]),
            },
        },
        domain::Event::MakingAdopt { key },
    ] {
        while engine.events.pop().is_some() {}
        let expected = core::mem::discriminant(&event);
        crate::route::request(engine, root::Request::Busy(Box::new(event)));
        let Some(root::Event::Forge(returned)) = engine.events.pop() else { panic!("busy input is retained") };
        assert_eq!(core::mem::discriminant(returned.as_ref()), expected);
        let returned_key = if let domain::Event::MakingStarted { key } = returned.as_ref() {
            key
        } else if let domain::Event::MakingCopying { key } = returned.as_ref() {
            key
        } else if let domain::Event::MakingAdopt { key } = returned.as_ref() {
            key
        } else if let domain::Event::MakingMade { key, id, branch } = returned.as_ref() {
            assert_eq!((*id, branch.as_ref()), (DESTINATION_ID, b"main".as_slice()));
            key
        } else if let domain::Event::MakingStopped { key, failed } = returned.as_ref() {
            assert_eq!(
                failed,
                &domain::MakingFailed {
                    step: domain::MakeStep::Repository,
                    reason: domain::MakeFailure::Taken,
                    words: Box::new([]),
                }
            );
            key
        } else {
            panic!("making input")
        };
        assert_eq!(returned_key, &copy().key);
    }
}
#[test]
fn a_restart_runs_the_row_again() {
    let (mut forge, mut service) = host();
    let mut row = copy();
    row.state = domain::MakingState::Making;
    row.copying = true;
    let key = row.key.clone();
    // A new host receives the connector's row as stored at an interrupted run.
    restore(&mut service, row);
    finish(&mut forge, &mut service, &key);
    assert_eq!(starter(&forge, FULL), temper_forge_setup::starter_workflow(b"temper-host").unwrap());
    assert!(service.engine.as_ref().unwrap().domain.making(&key).is_none());
}
#[test]
fn a_made_row_asks_its_adoption_once_connected() {
    let (_forge, mut service) = host();
    let mut row = copy();
    row.state = domain::MakingState::Made { id: DESTINATION_ID, branch: b"main".as_slice().into() };
    row.into = Some(domain::MakeInto { project: 1, home: false, host: b"forge.invalid".as_slice().into() });
    let key = row.key.clone();
    restore(&mut service, row);
    connection(&mut service, domain::Connection::NotConnected);
    let engine = service.engine.as_mut().unwrap();
    while engine.events.pop().is_some() {}
    lifecycle::advance(engine);
    assert!(engine.events.is_empty());
    connect(&mut service);
    let engine = service.engine.as_mut().unwrap();
    lifecycle::advance(engine);
    let expected = root::Event::Forge(Box::new(domain::Event::MakingAdopt { key: key.clone() }));
    let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("adoption pull") };
    let domain::Event::MakingAdopt { key: returned } = *event else { panic!("adoption event") };
    assert_eq!(returned, key);
    root::step(&mut engine.domain, &engine.env, expected);
    assert!(engine.domain.making_due(FORGE).is_none());
    lifecycle::advance(engine);
    assert!(engine.events.is_empty(), "one adoption act stays in flight");
}
#[test]
fn the_label_is_the_records_or_the_modes() {
    for (mode, first, label) in [
        (domain::Mode::Owned, None, b"temper-host".as_slice()),
        (domain::Mode::Managed, None, b"docker".as_slice()),
        (domain::Mode::Managed, Some(b"my-runner".as_slice()), b"my-runner".as_slice()),
    ] {
        let (mut forge, mut service) = host();
        let mut record = runtime_record(domain::Kind::Forgejo, mode);
        record.number = FORGE;
        record.first = match first {
            Some(label) => Some(domain::FirstIntent::Empty(domain::EmptyIntent {
                playground: false,
                org: b"other".as_slice().into(),
                repository: b"first".as_slice().into(),
                label: label.into(),
            })),
            None => None,
        };
        let engine = service.engine.as_mut().unwrap();
        root::step(
            &mut engine.domain,
            &engine.env,
            root::Event::Forge(Box::new(domain::Event::Restore { record: domain::Stored::Forge(record) })),
        );
        let row = copy();
        let key = row.key.clone();
        restore(&mut service, row);
        finish(&mut forge, &mut service, &key);
        assert_eq!(starter(&forge, FULL), temper_forge_setup::starter_workflow(label).unwrap());
    }
}

#[test]
fn a_long_copy_holds_a_restart_and_adoptions() {
    let (_forge, mut service) = host_with_local(true);
    let token = begin_row(&mut service, copy());
    let engine = service.engine.as_mut().unwrap();
    engine
        .setup
        .lifecycle
        .adoptions
        .insert(
            FORGE,
            lifecycle::Adoption { next: core::num::NonZeroU64::new(1).unwrap(), repositories: 1, files: None },
        )
        .unwrap();
    lifecycle::started(engine, FORGE);
    lifecycle::advance(engine);
    assert_eq!(engine.setup.runs.len(), WRITERS);
    assert_eq!(engine.setup.runs.get(&token).unwrap().kind, Kind::Making);
    assert!(engine.setup.lifecycle.local(FORGE).unwrap().pending, "restart and reconcile stay pending");
    assert!(engine.setup.lifecycle.adoptions.contains_key(&FORGE));
    assert!(lifecycle::take_read(engine).is_none(), "the restart cannot read its handover beside a copy");
    assert!(!engine.setup.lifecycle_ready(engine), "pending work sleeps until the copy ends");
}

#[test]
fn a_busy_making_waits_without_failing_the_row() {
    let (_forge, mut service) = host();
    let (run, key) = held_row(&mut service);
    let engine = service.engine.as_mut().unwrap();
    lifecycle::stopped(engine, run, failure(reconcile::Why::Busy));
    assert!(engine.setup.staged.contains_key(&run));
    assert!(engine.setup.runs.contains_key(&run));
    assert_eq!(engine.domain.making(&key).unwrap().state, domain::MakingState::Waiting);
    for event in &engine.events {
        if let root::Event::Forge(event) = event
            && let domain::Event::MakingStopped { .. } = event.as_ref()
        {
            panic!("a Busy stop must wait");
        }
    }
    assert!(making::ready(engine));
    lifecycle::advance(engine);
    assert!(!engine.setup.staged.contains_key(&run));
    let mut found = false;
    for event in &engine.setup.above {
        if let host::Event::Need { run: token, need: host::Need::Read(_) } = event
            && *token == run
        {
            found = true;
        }
    }
    assert!(found, "the waiting plan is retried");
}

#[test]
fn a_making_end_keeps_the_repository_and_default_branch() {
    let (_forge, mut service) = host();
    for (outcome, id, branch) in [
        (
            reconcile::Outcome::Created {
                id: core::num::NonZeroU64::new(7).unwrap(),
                branch: b"main".as_slice().into(),
            },
            7,
            b"main".as_slice(),
        ),
        (
            reconcile::Outcome::Copied {
                id: core::num::NonZeroU64::new(9).unwrap(),
                branch: b"trunk".as_slice().into(),
            },
            9,
            b"trunk".as_slice(),
        ),
    ] {
        let (run, key) = held_row(&mut service);
        let engine = service.engine.as_mut().unwrap();
        while engine.events.pop().is_some() {}
        lifecycle::done(engine, run, outcome);
        let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("making end") };
        let domain::Event::MakingMade { key: returned, id: returned_id, branch: returned_branch } = *event else {
            panic!("repository made")
        };
        assert_eq!((returned, returned_id, returned_branch.as_ref()), (key, id, branch));
        assert!(!engine.setup.runs.contains_key(&run));
    }
}

fn migrates(forge: &Forge) -> bool {
    for line in forge.requests.as_slice() {
        if line.starts_with(b"POST /api/v1/repos/migrate ") {
            return true;
        }
    }
    false
}
fn run_for(engine: &crate::engine::Engine, key: &domain::MakingKey) -> Option<Token> {
    for (token, run) in &engine.setup.runs {
        if run.making.as_ref() == Some(key) {
            return Some(*token);
        }
    }
    None
}
fn assert_not_busy(engine: &crate::engine::Engine) {
    for event in &engine.setup.above {
        if let host::Event::Stop { failure, .. } = event {
            assert_ne!(failure.why, reconcile::Why::Busy, "the pull must never start Busy");
        }
    }
}

#[test]
fn a_handover_answer_waits_while_a_row_runs() {
    let (_forge, mut service) = host_with_local(true);
    let engine = service.engine.as_mut().unwrap();
    // The box's connection event is still waiting for the root when the row is pulled.
    lifecycle::started(engine, FORGE);
    let read = lifecycle::take_read(engine).unwrap();
    let token = begin_row(&mut service, copy());
    let engine = service.engine.as_mut().unwrap();
    lifecycle::read(engine, FORGE, read.run, Ok(None));
    lifecycle::advance(engine);
    assert_eq!(engine.setup.runs.get(&token).unwrap().kind, Kind::Making);
    let lifecycle::FileStep::Read(Ok(None)) = engine.setup.lifecycle.local(FORGE).unwrap().file else {
        panic!("the handover answer waits for the copy");
    };
    assert!(!engine.setup.lifecycle_ready(engine), "the handover answer sleeps until the copy ends");
    assert_not_busy(engine);
}
