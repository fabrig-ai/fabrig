use jig_client_world::harness::{App, Duty, Front};
use skein_lib::{Env, Queue, Secret, Wall};
use skein_ui::Role;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{
    self as d, Address, Place, Step,
    pages::{Act, connect_model::Act as A},
};
use temper_client_world::{app::Temper, fixtures, front::account_acts as accounts};

#[test]
fn a_first_run_welcome_without_a_demo() {
    let (mut front, mut root) = accounts::welcome_fixture(fixtures::SEED);
    let person = root.person();
    assert!(person.demo_answered());
    assert!(person.forge().playground().is_none());
    assert!(person.pages().welcome().expect("welcome opened").decided);
    let document = d::pages::accounts::document(person.client()).expect("accounts delivered");
    assert!(!document.accounts.iter().any(|line| d::forge::playground::usable_account(line, None)));
    let limits = fixtures::domain_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(&mut view, person, fixtures::NOW, &mut Queue::with_capacity(bounds.tree.patches));
    assert!(
        view.tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Region && node.name.as_deref() == Some(b"Set up temper"))
    );
    assert!(!view.tree().nodes().iter().any(|node| node.name.as_deref() == Some(b"Try the playground")));

    accounts::event(
        &mut front,
        &mut root,
        d::Event::Went { place: Place::Address(Address::Welcome { step: Step::Model { way: None } }) }.into(),
    );
    accounts::event(
        &mut front,
        &mut root,
        d::Event::Went {
            place: Place::Address(Address::Welcome {
                step: Step::Model {
                    way: Some(d::ModelWay { source: None, provider: accounts::PROVIDER, kind: d::WayKind::Key }),
                },
            }),
        }
        .into(),
    );
    accounts::event(
        &mut front,
        &mut root,
        d::Event::AccountKey { value: Secret::new(b"test-private-key".as_slice().into()) }.into(),
    );
    accounts::event(
        &mut front,
        &mut root,
        d::Event::Act { action: d::Action::Pages(Act::ConnectModel(A::Submit { name: [41; 16] })) }.into(),
    );
    accounts::kept(&mut front);
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
        forge: 1,
        state: temper_engine_domain_forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
    })));
    accounts::settle(&mut front, &mut root);
    assert!(
        d::pages::accounts::document(root.person().client())
            .expect("current account facts")
            .accounts
            .iter()
            .any(|line| d::forge::playground::usable_account(line, None)),
        "key became usable"
    );
    let mut now = skein_lib::Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1));
    let mut out = Queue::uncapped();
    Temper::step(
        &mut root,
        &Env { now, wall: Wall::EPOCH, limits },
        d::Event::Went { place: Place::Address(Address::Welcome { step: Step::Welcome }) }.into(),
        &mut out,
    );
    // Reopening the welcome decides from the session's current forges rows.
    // The root-only pump discards shell navigation, so retain it here through its actual callbacks.
    let mut moved = false;
    for _ in 0..32 {
        while let Some(request) = out.pop() {
            moved |= request
                == (d::Request::Show {
                    address: Address::Chats { filter: ChatsFilter::All, start: false },
                    replace: true,
                });
            if let Duty::Wire(request) = Temper::duty(request) {
                front.wire(jig_client_world::harness::ClientId(0), now, Wall::EPOCH, request);
            }
        }
        now = now.saturating_add(skein_lib::Duration::from_millis(3));
        let callbacks = front.poll(now, Wall::EPOCH);
        if callbacks.is_empty() {
            break;
        }
        for (_, event) in callbacks {
            Temper::step(&mut root, &Env { now, wall: Wall::EPOCH, limits }, event, &mut out);
        }
    }
    assert!(
        moved,
        "the second open replaces welcome with chats: {:?}, role {:?}, decision {:?}",
        root.person().pages().welcome(),
        d::pages::accounts::current_me(root.person().client()).expect("current account facts").deployment_role,
        d::pages::welcome::decision(root.person())
    );
}

#[test]
fn a_configured_model_hides_the_account_banner() {
    let (mut front, mut root) = accounts::welcome_fixture(fixtures::SEED);
    let person = root.person();
    assert!(person.demo_answered());
    assert!(person.forge().playground().is_none());
    let document = d::pages::accounts::document(person.client()).expect("accounts delivered");
    assert!(document.usable, "the configured model is usable");
    assert!(!document.accounts.iter().any(|line| d::forge::playground::usable_account(line, None)));
    assert_eq!(
        d::pages::welcome::decision(person),
        Some(d::pages::welcome::FirstRun::Unfinished { step: d::Step::Forge { forge: None } }),
        "the forge is not connected"
    );
    accounts::event(
        &mut front,
        &mut root,
        d::Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) }.into(),
    );
    let limits = fixtures::domain_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(&mut view, root.person(), fixtures::NOW, &mut Queue::with_capacity(bounds.tree.patches));
    let nodes = view.tree().nodes();
    assert!(!nodes.iter().any(|node| node.name.as_deref() == Some(b"Agents need an LLM account to run.")));
    assert!(!nodes.iter().any(|node| node.name.as_deref() == Some(b"temper has no model yet.")));
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
        forge: 1,
        state: temper_engine_domain_forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
    })));
    accounts::settle(&mut front, &mut root);
    assert_eq!(d::pages::welcome::decision(root.person()), Some(d::pages::welcome::FirstRun::Whole));
}


