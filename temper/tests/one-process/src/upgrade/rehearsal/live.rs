//! The stopped engine's opening, catch-up and worker verdict on the settled image.
use crate::{
    fixtures, golden,
    testing::{Disk, USER},
};
use jig_domain_core::{CoreKey, CoreRecord, Key, Record};
use jig_protocol_store::{Event, Get, Request, Row, Write};
use skein_fake_machine::Machine;
use skein_io::kernel::Op;
use skein_lib::{List, Queue, Token};
use temper_engine_domain_forge as forge;
use temper_engine_service::{self as service, startup};
use temper_playground::programs::prepared_schemas;
use temper_shell::directories::Directories;

pub(super) const OWNER: Token = Token::new(77);

fn seed(disk: &mut Disk, inputs: &startup::LimitInputs) -> Box<[u8]> {
    let (mut jig, own) = inputs.stock().expect("fixture stock limits");
    let root = startup::engine_limits(&mut jig, &own, None).expect("fixture engine limits");
    let limits = startup::checked_store(&root, &jig.store, None).expect("fixture store limits");
    let mut store = service::store::Store::new(&root, &limits, &[], &inputs.seeds);
    store.prepared_schemas(prepared_schemas());
    store.adopt(disk.root(0), USER).expect("source adopted");
    let Event::Opened { last, pending, .. } = disk.event(&mut store) else { panic!("source opened") };
    if pending {
        store.ask(Request::Switch { owner: OWNER });
        assert!(matches!(disk.event(&mut store), Event::Switched { .. }));
    }
    store.ask(Request::Get { owner: OWNER, key: Get::Jig(Key::Core(CoreKey::Deployment)) });
    let Event::Got { row: Some(Row::Jig(Record::Core(CoreRecord::Deployment(mut header)))), .. } =
        disk.event(&mut store)
    else {
        panic!("settled deployment row");
    };
    let key = temper_protocol_store::keys::encode(&forge::Key::Forge(1), u32::MAX).expect("forge key");
    let forge = forge::Stored::Forge(forge::ForgeRecord {
        number: 1,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Connected,
        state: forge::ForgeState::Kept,
        place: forge::Place {
            api: forge::Address { host: b"127.0.0.1".as_slice().into(), port: 1, tls: None },
            git: b"http://127.0.0.1:1".as_slice().into(),
            web: None,
        },
        git_program: None,
        read_only: false,
        bot: forge::Bot {
            login: b"bot".as_slice().into(),
            name: b"Bot".as_slice().into(),
            email: b"bot@example.invalid".as_slice().into(),
        },
        asked: false,
        first: None,
        person: None,
    });
    let playground = temper_protocol_store::keys::encode(&forge::Key::Playground, u32::MAX).expect("playground key");
    header.commits = last + 1;
    store.ask(Request::Commit {
        number: header.commits,
        writes: Box::new([
            Write::Remove { key: playground },
            Write::Put { key: key.clone(), value: temper_protocol_store::records::encode(&forge) },
            Write::Save(Record::Core(CoreRecord::Deployment(header))),
        ]),
    });
    assert!(matches!(disk.event(&mut store), Event::Committed { number, .. } if number == last + 1));
    store.ask(Request::Get { owner: OWNER, key: Get::Application(key.clone()) });
    assert!(
        matches!(disk.event(&mut store), Event::Got { row: Some(Row::Application { value, .. }), .. } if !value.is_empty())
    );
    disk.close(store);
    key
}

fn open_copy(disk: &mut Disk, store: &mut service::store::Store) {
    store.adopt_rehearsal(disk.root(0), disk.root(1), USER).expect("rehearsal roots adopted");
    let mut keep = Queue::uncapped();
    for _ in 0..10_000 {
        disk.tick(store);
        while let Some(event) = store.events().pop() {
            match event {
                Event::Opened { installation, pending, .. } => store.opened_limits(installation, pending),
                event @ Event::Job { .. } => assert!(store.opening_job(event)),
                event @ (Event::OpenRefused { .. } | Event::Undecodable { .. } | Event::Corrupt { .. }) => {
                    panic!("copy failed to open: {event:?}")
                }
                event => keep.push(event),
            }
        }
        if store.opened() {
            while let Some(event) = keep.pop() {
                store.events().push(event);
            }
            return;
        }
    }
    panic!("copy opens");
}

pub(super) fn settle(disk: &mut Disk, service: &mut service::Service) {
    for _ in 0..10_000 {
        let now = disk.sim.now();
        service::iterate(service, now, disk.sim.wall());
        while service.notes().pop().is_some() {}
        while service.listening().pop().is_some() {}
        if service.upgrade_report().is_some_and(|report| report.restored) {
            assert!(
                service.engine_domain().expect("rehearsal engine").ready(),
                "the rehearsal reached decisions opening"
            );
        }
        for submit in service.submissions().iter() {
            assert!(
                !matches!(submit.kind, Op::Listen { .. } | Op::Connect { .. } | Op::Spawn { .. }),
                "rehearsal effect: {:?}",
                submit.kind
            );
        }
        if service.is_empty() {
            disk.closed();
            return;
        }
        service.check_wait_boundary(now);
        let pending = service.work_pending(now);
        disk.pass(service.submissions(), pending);
        while let Some(complete) = disk.completions().pop() {
            service.completions().push(complete);
        }
    }
    panic!("rehearsal drains");
}

pub(super) fn accounts(engine: &jig_protocol_config::Engine) -> service::AccountsRuntime {
    use skein_oauth_accounts::{Endpoint, Origin, Transport};

    let mut runtime = service::AccountsRuntime { providers: List::with_capacity(2), trust: None };
    let endpoint = || Endpoint { address: ([127, 0, 0, 1], 7104).into(), transport: Transport::Plaintext };
    for provider in engine.llm.providers.iter().filter(|provider| provider.sign_in.is_some()) {
        let mut origins = List::with_capacity(1);
        origins
            .push(Origin { uri: b"http://127.0.0.1:7104".as_slice().into(), endpoint: endpoint() })
            .expect("sign-in origin");
        runtime.providers.push((provider.number, endpoint(), origins)).expect("two providers");
    }
    runtime
}

/// Rehearse the settled deployment with stock figures, keeping the source whole.
pub fn a_live_change_and_uncertain_write_reach_the_opening_without_network(
    machine: Machine,
    document: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
) {
    let directories_jig = temper_service_shell::jig_directories(directories);
    let engine = temper_engine_shell::serve::settings(document, directories, given);
    let accounts = accounts(&engine);
    let seeds = fixtures::seeds();
    let models = jig_protocol_config::core(&engine, seeds.deployment, seeds.seed).settings.models;
    let files = service::settings::Files { entries: List::with_capacity(0) };
    let inputs = startup::LimitInputs::new(document, &directories_jig, &files, None, &seeds, given, false, None);
    let mut disk = Disk::new(machine);
    let key = seed(&mut disk, &inputs);
    let before = golden::files::image_root(&mut disk.machine, 0);
    let mut store = service::store::Store::for_limits(&inputs, &models).expect("rehearsal store limits");
    store.prepared_schemas(prepared_schemas());
    store.build_name("test-build");
    let began = disk.sim.now();
    open_copy(&mut disk, &mut store);
    let limits = store.resolve_settings(inputs, None).expect("resolve the copy's limits");
    let startup::CheckedStart { engine, io, forges, .. } = startup::checked_start_with_resolved(
        document,
        &directories_jig,
        &files,
        None,
        &seeds,
        List::with_capacity(0),
        None,
        given,
        &limits.in_force,
    )
    .expect("checked stopped engine");
    let mut service = startup::restoring_prepared(engine, &seeds, Some(accounts), forges, io).expect("stopped service");
    service.engine_component().rehearse_opened(store, Some(limits), began);
    let store = service.engine_component().store();
    store.ask(Request::Get { owner: OWNER, key: Get::Application(key) });
    let mut saved = Queue::uncapped();
    let mut found = false;
    while let Some(event) = store.events().pop() {
        if let Event::Got { owner, row } = event {
            assert_eq!(owner, OWNER);
            assert!(row.is_some(), "forge in the copy");
            found = true;
        } else {
            saved.push(event);
        }
    }
    assert!(found);
    while let Some(event) = saved.pop() {
        store.events().push(event);
    }
    settle(&mut disk, &mut service);
    let report = service.upgrade_report().expect("check report");
    assert!(report.done);
    assert_eq!(golden::files::image_root(&mut disk.machine, 1).len(), 1, "scratch emptied");
    let (text, code) = temper_engine_shell::upgrade::lines("temper-engine", report, directories, b"/data/state");
    assert_eq!(code, 0, "{text}");
    let lines: Vec<_> = text.lines().collect();
    let opening = lines
        .iter()
        .position(|line| line.contains("registry commit, jobs and restores pass; opened in "))
        .expect(&text);
    let millis = lines[opening].split("opened in ").nth(1).expect("opening time").strip_suffix(" ms").expect("unit");
    assert!(millis.parse::<u64>().is_ok(), "{text}");
    assert!(lines[opening + 1].starts_with("temper-engine upgrade: forge 1 would catch up: "), "{text}");
    assert!(lines[opening + 1].contains("1 uncertain write"), "{text}");
    assert!(lines[opening + 1].contains("fresh reads"), "CatchUp was routed before the snapshot: {text}");
    assert!(text.contains("worker local may not stay: limits:"), "{text}");
    assert!(text.ends_with("workers may stay: no\n"), "{text}");
    golden::files::unchanged(&before, &golden::files::image_root(&mut disk.machine, 0));
}
