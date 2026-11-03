use jig_client_world::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use skein_ui::Role;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{self as d, Address, Choice, Step};
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::{Joint, account_acts as accounts},
};
use temper_engine_domain_forge::{self as forge, Connection};
const OWNED: u16 = 1;
const CLIENT: ClientId = ClientId(0);
const PUMP_STEPS: usize = 64;
fn pump(front: &mut Joint, root: &mut Root, now: &mut Time, event: d::Event) -> bool {
    let limits = fixtures::domain_limits();
    let mut out = Queue::uncapped();
    Temper::step(root, &Env { now: *now, wall: Wall::EPOCH, limits }, event.into(), &mut out);
    let mut model = false;
    for _ in 0..PUMP_STEPS {
        while let Some(request) = out.pop() {
            if let d::Request::Show { address, .. } = &request {
                model |= *address == Address::Welcome { step: Step::Model { way: None } };
                Temper::step(
                    root,
                    &Env { now: *now, wall: Wall::EPOCH, limits },
                    d::Event::Went { place: d::Place::Address(address.clone()) }.into(),
                    &mut out,
                );
            }
            if let Duty::Wire(request) = Temper::duty(request) {
                front.wire(CLIENT, *now, Wall::EPOCH, request);
            }
        }
        *now = now.saturating_add(Duration::from_millis(3));
        let callbacks = front.poll(*now, Wall::EPOCH);
        if callbacks.is_empty() {
            return model;
        }
        for (_, event) in callbacks {
            front.received(CLIENT, &event);
            Temper::step(root, &Env { now: *now, wall: Wall::EPOCH, limits }, event, &mut out);
        }
    }
    panic!("forge story did not settle");
}
pub(super) fn view(root: &Root) -> temper_client_view::View {
    let limits = fixtures::domain_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(&mut view, root.person(), Wall::EPOCH, &mut Queue::with_capacity(bounds.tree.patches));
    view
}
pub(super) fn node(view: &temper_client_view::View, role: Role, name: &[u8]) -> skein_ui::NodeId {
    view.tree()
        .nodes()
        .iter()
        .find(|n| n.role == role && n.name.as_deref() == Some(name))
        .unwrap_or_else(|| {
            panic!(
                "missing {role:?}: {}; nodes: {:?}",
                String::from_utf8_lossy(name),
                view.tree()
                    .nodes()
                    .iter()
                    .map(|node| (node.role, node.name.as_deref().map(String::from_utf8_lossy)))
                    .collect::<Vec<_>>()
            )
        })
        .id
}
fn connection(front: &mut Joint, state: Connection) {
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::Connection { forge: OWNED, state })));
    front.engine.settle();
}
fn run_owned(front: &mut Joint, root: &mut Root, now: &mut Time) {
    pump(
        front,
        root,
        now,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Forge { forge: None } }) },
    );
    let mut choose = view(root);
    let button = node(&choose, Role::Link, b"Forgejo on this machine");
    let action = temper_client_view::act(&mut choose, button, skein_ui::Act::Press).expect("bound choice");
    pump(front, root, now, d::Event::Act { action });
    let mut details = view(root);
    let button = node(&details, Role::Button, b"Set up a Forgejo here");
    let action = temper_client_view::act(&mut details, button, skein_ui::Act::Press).expect("setup new forge");
    pump(front, root, now, d::Event::Act { action });
}
pub(super) fn owned_welcome() -> (Joint, Root, Time) {
    let (mut front, mut root) = accounts::fresh_fixture(fixtures::SEED);
    let mut now = Time::ZERO.saturating_add(Duration::from_secs(1));
    run_owned(&mut front, &mut root, &mut now);
    (front, root, now)
}
pub(super) fn accept_owned(front: &mut Joint, root: &mut Root, now: &mut Time) -> bool {
    let (forge, request) = *front.set_ups().first().expect("engine asked the box");
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::SetUpAnswered {
        forge,
        request,
        answer: forge::SetUpAnswer::SettingUp,
    })));
    front.engine.settle();
    pump(
        front,
        root,
        now,
        d::Event::Went {
            place: d::Place::Address(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(OWNED)) } }),
        },
    )
}
#[test]
fn setting_up_forgejo_here_from_the_welcome() {
    let (mut front, mut root, mut now) = owned_welcome();
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Forge { forge: Some(Choice::Owned) } });
    let setup = root.person().forge().set_up().expect("welcome sent setup");
    assert_eq!((&*setup.org, &*setup.repository, setup.playground), (b"work".as_slice(), b"hello".as_slice(), false));
    assert_eq!(front.set_ups().iter().map(|(forge, _)| *forge).collect::<Vec<_>>(), [OWNED]);
    assert!(accept_owned(&mut front, &mut root, &mut now), "accepted setup moves to Model");
    connection(&mut front, Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) });
    pump(
        &mut front,
        &mut root,
        &mut now,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    let model = view(&root);
    node(&model, Role::Paragraph, "Forgejo on this machine · Setting up: preparing · work/hello".as_bytes());
    node(&model, Role::Disclosure, b"Each step");
    pump(
        &mut front,
        &mut root,
        &mut now,
        d::Event::Went { place: d::Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) },
    );
    let chats = view(&root);
    assert!(
        !chats.tree().nodes().iter().any(|node| node.role == Role::Region && node.name.as_deref() == Some(b"Setup"))
    );
    connection(&mut front, Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    pump(
        &mut front,
        &mut root,
        &mut now,
        d::Event::Went { place: d::Place::Address(Address::ConnectForge { forge: Some(Choice::Forge(OWNED)) }) },
    );
    let making = view(&root);
    node(&making, Role::Heading { level: 2 }, b"Setting up Forgejo on this machine");
    node(&making, Role::Paragraph, b"work/hello");
    let before = front.applications_admitted();
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::FirstMade { forge: OWNED, id: 42 })));
    front.engine.settle();
    // Creation leaves the first repository unadopted on every page.
    pump(
        &mut front,
        &mut root,
        &mut now,
        d::Event::Went { place: d::Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) },
    );
    assert_eq!(front.applications_admitted(), before, "setup sends no adoption");
    assert!(!front.engine_adopted(OWNED, 42));
    for _ in 0_u32..2 {
        pump(
            &mut front,
            &mut root,
            &mut now,
            d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) },
        );
        let ready = view(&root);
        node(&ready, Role::Heading { level: 1 }, b"Almost ready");
        node(&ready, Role::Button, b"Open Default");
        assert_eq!(front.applications_admitted(), before, "revisiting Ready adopts nothing");
    }
}

#[test]
fn model_follows_forgejo_from_setup_to_its_first_repository() {
    let (mut front, mut root, mut now) = owned_welcome();
    accept_owned(&mut front, &mut root, &mut now);
    connection(&mut front, Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) });
    let model_address = Address::Welcome { step: Step::Model { way: None } };
    pump(&mut front, &mut root, &mut now, d::Event::Went { place: d::Place::Address(model_address.clone()) });
    let setting_up = view(&root);
    node(&setting_up, Role::Paragraph, "Forgejo on this machine · Setting up: preparing · work/hello".as_bytes());
    node(&setting_up, Role::Disclosure, b"Each step");
    connection(&mut front, Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::FirstMade { forge: OWNED, id: 42 })));
    front.engine.settle();
    pump(&mut front, &mut root, &mut now, d::Event::Went { place: d::Place::Address(model_address.clone()) });
    assert_eq!(root.person().pages().address(), &model_address);
    assert!(!front.engine_adopted(OWNED, 42));
    let made = view(&root);
    let line = node(&made, Role::Paragraph, "Forgejo on this machine · work/hello".as_bytes());
    let line = made.tree().nodes().iter().find(|n| n.id == line).expect("forge line");
    assert!(line.styles.as_slice().contains(&temper_client_view::look::FLOW_DONE));
    assert!(!made.tree().nodes().iter().any(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Each step")));
}

#[test]
fn github_connects_without_a_picker_and_the_named_welcome_moves_to_model() {
    let (mut front, mut root) = Joint::github_fixture(fixtures::SEED);
    node(&view(&root), Role::Heading { level: 2 }, b"GitHub is connected");
    assert!(front.reachable_reads.is_empty());
    assert!(!front.engine_adopted(1, 8));
    accounts::event(
        &mut front,
        &mut root,
        Temper::went(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(1)) } }),
    );
    node(&view(&root), Role::Heading { level: 1 }, b"Connect a model");
    assert!(front.reachable_reads.is_empty());
}

fn press(front: &mut Joint, root: &mut Root, role: Role, name: &[u8]) {
    let mut rendered = view(root);
    let id = node(&rendered, role, name);
    let action = temper_client_view::act(&mut rendered, id, skein_ui::Act::Press).expect("bound forge action");
    accounts::event(front, root, d::Event::Act { action }.into());
}
#[test]
fn an_unfinished_forgejo_starts_over_by_removing_its_record() {
    let (mut front, mut root) = accounts::forge_fixture(fixtures::SEED);
    node(&view(&root), Role::Button, b"Start over");
    press(&mut front, &mut root, Role::Button, b"Start over");
    assert!(matches!(root.person().forges(), d::forge::forges::State::Rows(rows) if rows.is_empty()));
    assert_eq!(front.leaves.last().map(AsRef::as_ref), Some(b"/welcome?at=forge".as_slice()));
    accounts::event(&mut front, &mut root, Temper::went(Address::Welcome { step: Step::Forge { forge: None } }));
    node(&view(&root), Role::Button, b"Connect GitHub");
}
#[test]
fn an_unfinished_github_app_goes_on_to_its_installation() {
    let (mut front, mut root) = Joint::github_fixture_with(fixtures::SEED, false, false);
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::AppRecorded {
        forge: 1,
        app: forge::AppWay::Pasted,
    })));
    front.engine.settle();
    accounts::event(&mut front, &mut root, Temper::went(Address::Welcome { step: Step::Forge { forge: None } }));
    node(&view(&root), Role::Button, b"Go on");
    assert!(front.installs.is_empty());
    press(&mut front, &mut root, Role::Button, b"Go on");
    assert_eq!(front.installs.len(), 1);
    assert_eq!(front.installs[0].0, 1);
}
