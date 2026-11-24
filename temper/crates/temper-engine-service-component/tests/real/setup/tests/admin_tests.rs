//! The administration record's directory and file modes (temper/docs/design/protocol/credentials.md, 5).
#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "the real store fixture must open, commit, keep and close before teardown"
)]
use jig_protocol_store::{Event as StoreEvent, SecretOwner, SecretValue, Secrets};
use skein_lib::{Queue, Secret, Token};
use temper_engine_domain as root;
use temper_engine_service_component::{limits, settings, store::Store};
use temper_protocol_records::{self as records, Item};

fn new_store() -> Store {
    let jig = jig_protocol_config_fixtures::small();
    let mut own = temper_limits_fixtures::small();
    own.root.journal.run_bytes = 32_768;
    own.root.transcript_memory = 1_152_921_504_606_846_976;
    let root = limits::engine(&jig, &own).unwrap();
    let profile = temper_engine_service_component::store::limits(&root, &root.forge, &jig.store).unwrap();
    Store::new(
        &root,
        &profile,
        &[],
        &settings::Seeds {
            deployment: [7; 16],
            installation: [9; 16],
            seed: 7,
            crypto: Secret::new(Box::new([7; 32])),
            link_secret: None,
            local_secret: None,
        },
    )
}

fn commit_wires(store: &mut Store) {
    let header = jig_domain_core::Deployment {
        id: [19; 16],
        restart_wait_ms: None,
        wait_floors: None,
        projects: 0,
        tasks: 0,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: 0,
        applications: 0,
        commits: 1,
    };
    assert!(store.down(root::StoreRequest::Commit {
        number: 1,
        writes: Box::new([
            root::Write::Save(root::Record::Core(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(header),
            ))),
            root::Write::Wires,
        ]),
    }));
}

fn disk_turn(store: &mut Store, kernel: &mut skein_shell::Kernel) {
    store.iterate();
    let wait = if kernel.in_flight() > 0 || !store.submissions().is_empty() {
        skein_shell::Wait::Forever
    } else {
        skein_shell::Wait::No
    };
    kernel.submit(store.submissions(), wait);
    let mut completions = Queue::with_capacity(64);
    kernel.reap(&mut completions);
    while let Some(completion) = completions.pop() {
        store.complete(completion);
    }
}
#[test]
fn admin_record_files_and_directories_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = skein_scratch::Scratch::new_in(&std::env::current_dir().unwrap(), "admin-record-modes");
    std::fs::set_permissions(scratch.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut store = new_store();
    let fd = skein_shell::open_root(scratch.path()).unwrap();
    store.adopt(fd, skein_shell::effective_user()).unwrap();
    let mut kernel =
        skein_shell::Kernel::open_on(skein_shell::Config { operations: 64 }, skein_shell::Backend::Readiness).unwrap();
    let mut opened = false;
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if let Some(event) = store.events().pop() {
            assert!(
                (match event {
                    StoreEvent::Opened { .. } => true,
                    _ => false,
                }),
                "{event:?}"
            );
            opened = true;
            break;
        }
    }
    assert!(opened);
    // This permissions fixture does not consume the census (jig jig/docs/design/protocol/store/growth.md, 1.3).
    let Some(StoreEvent::Census { .. }) = store.events().pop() else { panic!("opened census") };
    commit_wires(&mut store);
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if let Some(event) = store.events().pop() {
            let StoreEvent::Committed { .. } = event else { panic!("{event:?}") };
            break;
        }
    }
    assert!(store.mark_durable());
    let _queued = store.secrets(
        Token::new(901),
        Secrets::Keep {
            owner: SecretOwner::Application,
            number: records::number(1, Item::AdminToken),
            value: SecretValue::Application(Secret::new(
                records::encode(Item::AdminToken, b"fixture").unwrap().into_boxed_slice(),
            )),
        },
    );
    let mut kept = false;
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if let Some(event) = store.events().pop() {
            assert!(
                (match event {
                    StoreEvent::Kept { .. } => true,
                    _ => false,
                }),
                "{event:?}"
            );
            kept = true;
            break;
        }
    }
    assert!(kept);
    let mut paths = Queue::with_capacity(64);
    paths.push(scratch.path().join("generations/1/secrets"));
    let mut files = 0_u32;
    while let Some(path) = paths.pop() {
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        assert!(!metadata.is_symlink());
        if metadata.is_dir() {
            assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
            for entry in std::fs::read_dir(&path).unwrap() {
                paths.push(entry.unwrap().path());
            }
        } else {
            assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
            files = files.checked_add(1).unwrap();
        }
    }
    assert!(files > 0);
    store.close();
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if store.closed() {
            break;
        }
    }
    assert!(store.closed());
}
