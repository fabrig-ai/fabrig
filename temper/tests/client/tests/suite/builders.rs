//! Workers reads and edits the joined engine's deployment (temper/docs/design/client/pages/workers.md, 2).
use super::model::{Pages, at, follow, open, press, see};
use jig_client::contract as c;
use jig_client_kit::harness::{ClientId, FrontFault};
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_lib::Duration;
use skein_ui::{Act, NodeId, Role};
use temper_client_domain::{Address, pages::builders::Read};
use temper_client_world::fixtures as f;

#[test]
fn an_owner_reads_worker_settings_not_the_connected_hosts() {
    let mut world = f::accounts_world(f::SEED);
    world.front().engine.hello(skein_lib::Token::new(7));
    let client = open(&mut world, BUILDERS);
    follow(&mut world, client, vec![see(Role::Heading { level: 2 }, "Workers · 1".as_bytes())]);
    let root = world.root(client).person();
    let Read::Hosts(doc) = root.pages().builders().unwrap().read(root.client()) else {
        panic!("engine deployment read")
    };
    assert_eq!(doc.worker_settings.len(), 1);
    let setting = doc.worker_settings.get(0).expect("default worker");
    assert_eq!(setting.name.as_ref(), b"worker");
    assert!(setting.linked);
    let worker = doc.hosts.get(0).expect("one worker");
    assert_eq!(doc.hosts.len(), 1);
    assert_eq!(worker.channel, 7);
    assert_eq!(worker.name.as_deref(), Some(b"worker".as_slice()));
    assert!(worker.connected);
}
const BUILDERS: &[u8] = b"/projects/1/settings/builders";
const FIRST_PLACES: &[u8] = b"Places for worker";
const W2: &[u8] = b"w2";
const W2_NUMBER: u32 = 2; // The first free number after walking's worker.
const W2_PLACES: &[u8] = b"Places for w2";
const DIGEST_HEX: &[u8; 64] = b"abababababababababababababababababababababababababababababababab";
const DIGEST: [u8; 32] = [0xab; 32]; // The bytes the owner typed as DIGEST_HEX.
const ADDED_PLACES: u32 = 1;
const REVISED_PLACES: u32 = 2;
const REVISED: &[u8] = "2 places · no runs · digest abababab…".as_bytes();

fn type_in(field: &[u8], words: &[u8]) -> Step<Address> {
    Step::Type { find: Find::exactly_named(Role::TextField, field), words: words.into() }
}

fn worker_settings(world: &Pages, client: ClientId) -> &[c::WorkerState] {
    let root = world.root(client).person();
    let Read::Hosts(doc) = root.pages().builders().expect("Workers tab").read(root.client()) else {
        panic!("engine deployment read")
    };
    doc.worker_settings.as_slice()
}

fn assert_change(world: &mut Pages, against: u32, edit: c::SettingEdit) {
    let expected = c::SettingItem { setting: c::Setting::Worker(c::SettingNumber { number: W2_NUMBER }), edit };
    let changes = std::mem::take(&mut world.front().settings_changes);
    assert_eq!(
        changes,
        [c::ChangeSettings {
            against,
            items: skein_lib::List::from_boxed(Box::new([expected])),
            running: c::Running::Stays
        }]
    );
}

fn set_w2(places: u32) -> c::SettingEdit {
    c::SettingEdit::Set(c::SettingSet {
        value: c::SettingValue::Worker(Box::new(c::WorkerSetting {
            number: W2_NUMBER,
            name: W2.into(),
            places,
            digest: Some(DIGEST),
            withdrawn: false,
        })),
    })
}

fn remove_from_w2_row(world: &Pages, client: ClientId) -> NodeId {
    // Both workers have a Remove button. Find the controls that enclose w2's field.
    let tree = world.tree(client);
    let nodes = tree.nodes();
    let field = nodes
        .iter()
        .position(|n| n.role == Role::TextField && n.name.as_deref() == Some(W2_PLACES))
        .expect("w2's places field");
    let (_, controls) = nodes[..field]
        .iter()
        .enumerate()
        .rev()
        .find(|(start, n)| n.role == Role::Slot && usize::try_from(n.size).expect("subtree size fits") > field - start)
        .expect("w2's controls");
    let mut row = TreeFace::subtree(tree, controls.id).expect("w2's controls subtree");
    let Found::Node(remove) = row.find(&Find::exactly_named(Role::Button, b"Remove")) else {
        panic!("w2's Remove button")
    };
    remove
}

#[test]
fn an_owner_adds_revises_reopens_and_removes_a_worker() {
    let mut world = f::worker_settings_world(f::SEED);
    let owner = open(&mut world, BUILDERS);
    follow(
        &mut world,
        owner,
        vec![
            see(Role::Heading { level: 1 }, b"Where agents work."),
            see(Role::Heading { level: 2 }, "Workers · 1".as_bytes()),
            see(Role::TextField, FIRST_PLACES),
            see(Role::Heading { level: 2 }, b"Add a worker"),
        ],
    );
    let [first] = worker_settings(&world, owner) else { panic!("one initial worker setting") };
    let first = first.clone();
    let before_add = world.front().engine.root.settings_revision();
    follow(
        &mut world,
        owner,
        vec![
            type_in("The worker’s name".as_bytes(), W2),
            type_in(b"Places", b"1"),
            type_in("Its secret’s digest".as_bytes(), DIGEST_HEX),
            press(Role::Button, b"Add"),
            see(Role::Heading { level: 2 }, "Workers · 2".as_bytes()),
            see(Role::TextField, W2_PLACES),
        ],
    );
    assert_change(&mut world, before_add, set_w2(ADDED_PLACES));

    let before_revise = world.front().engine.root.settings_revision();
    // Keep the engine callbacks behind the submit so the line is checked before the answer as well.
    world.fault(FrontFault::Slow { span: Duration::from_millis(100) });
    follow(
        &mut world,
        owner,
        vec![
            type_in(W2_PLACES, b"2"),
            Step::Send { find: Find::exactly_named(Role::TextField, W2_PLACES) },
            see(Role::Text, "1 place · no runs · digest abababab…".as_bytes()),
            Step::Gone { find: Find::exactly_named(Role::Text, REVISED), within: Duration::ZERO },
        ],
    );
    follow(&mut world, owner, vec![see(Role::Text, REVISED)]);
    assert_change(&mut world, before_revise, set_w2(REVISED_PLACES));
    let [_, revised] = worker_settings(&world, owner) else { panic!("two settings after revision") };
    assert_eq!((revised.number, revised.digest, revised.places), (W2_NUMBER, Some(DIGEST), REVISED_PLACES));
    let revised = revised.clone();

    let before_reload = world.front().engine.root.settings_revision();
    // This world's restart drops the address; restore the browser's saved tab.
    world.reload(owner);
    follow(
        &mut world,
        owner,
        vec![see(Role::Status, b"Live"), Step::Go { address: at(BUILDERS) }, see(Role::Text, REVISED)],
    );
    assert!(world.front().settings_changes.is_empty(), "reopening must not send ChangeSettings");
    assert_eq!(world.front().engine.root.settings_revision(), before_reload);
    assert_eq!(worker_settings(&world, owner), &[first.clone(), revised.clone()]);

    let remove = remove_from_w2_row(&world, owner);
    world.act(owner, remove, Act::Press);
    follow(
        &mut world,
        owner,
        vec![
            see(Role::Heading { level: 2 }, "Workers · 1".as_bytes()),
            Step::Gone { find: Find::exactly_named(Role::TextField, W2_PLACES), within: f::WITHIN },
        ],
    );
    assert_change(&mut world, before_reload, c::SettingEdit::Removed);
    assert_eq!(worker_settings(&world, owner), &[first, c::WorkerState { withdrawn: true, ..revised }]);
}

#[test]
fn a_person_with_no_role_is_told_who_sees_them() {
    // This fixture gives its second person a project membership and no deployment role.
    let mut world = f::asking_world(f::SEED, true);
    let caller = world.front().caller;
    let client = world.client(caller, temper_client_domain::Address::Builders { project: 1 });
    follow(
        &mut world,
        client,
        vec![
            see(Role::Status, b"Live"),
            see(Role::Paragraph, "Workers are seen by the deployment’s owners.".as_bytes()),
        ],
    );
    let root = world.root(client).person();
    assert_eq!(root.pages().me(root.client()).unwrap().deployment_role, None);
    assert!(matches!(root.pages().builders().unwrap().read(root.client()), Read::NoRole));
    assert!(!world.tree(client).nodes().iter().any(|n| n.role == Role::Heading { level: 2 }));
}
