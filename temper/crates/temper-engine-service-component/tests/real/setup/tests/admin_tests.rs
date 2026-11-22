use crate::{fixtures, store};
use fixtures::ADMIN;
use jig_protocol_store::Event as StoreEvent;
use skein_lib::{Queue, Secret, Token};
use temper_engine_domain as root;
use temper_protocol_records::{self as records, Item};

fn disk_turn(store: &mut store::Store, kernel: &mut skein_shell::Kernel) {
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
#[expect(
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "the store event remains available as an assertion diagnostic"
)]
fn admin_record_files_and_directories_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = skein_scratch::Scratch::new_in(&std::env::current_dir().unwrap(), "admin-record-modes");
    std::fs::set_permissions(scratch.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let (root, _, profile) = fixtures::store_limits();
    let limits = store::limits(&root, &root.forge, &profile).unwrap();
    let mut store = store::Store::new(&root, &limits, &[], &fixtures::seeds());
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
    // This permissions fixture does not consume the census (jig protocol/store/README.md, 6.3).
    let Some(StoreEvent::Census { .. }) = store.events().pop() else { panic!("opened census") };
    fixtures::commit(&mut store, 1, Box::new([root::Write::Wires]));
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if let Some(event) = store.events().pop() {
            let StoreEvent::Committed { .. } = event else { panic!("{event:?}") };
            break;
        }
    }
    assert!(store.mark_durable());
    store.secrets(
        Token::new(901),
        fixtures::application_request(
            records::number(1, Item::AdminToken),
            Secret::new(records::encode(Item::AdminToken, ADMIN).unwrap().into_boxed_slice()),
        ),
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
