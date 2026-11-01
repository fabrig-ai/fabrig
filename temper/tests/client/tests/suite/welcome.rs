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
    let (mut front, mut root) = accounts::seeded_forge_fixture(fixtures::SEED);
    let person = root.person();
    assert!(person.demo_answered());
    assert!(person.demo().is_none());
    assert!(person.pages().welcome().expect("welcome opened").decided);
    let document = d::pages::accounts::document(person.client()).expect("accounts delivered");
    assert!(!document.accounts.iter().any(|line| d::forge::demo::usable_account(line, None)));
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
            .any(|line| d::forge::demo::usable_account(line, None)),
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
                    address: Address::Welcome { step: Step::Forge { forge: None } },
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
        "the second open resumes Forge until a repository is adopted: {:?}, role {:?}, decision {:?}",
        root.person().pages().welcome(),
        d::pages::accounts::current_me(root.person().client()).expect("current account facts").deployment_role,
        d::pages::welcome::decision(root.person())
    );
}

#[test]
fn a_configured_model_hides_the_accounts_banner() {
    let (mut front, mut root) = accounts::seeded_forge_fixture(fixtures::SEED);
    let person = root.person();
    assert!(person.demo_answered());
    assert!(person.demo().is_none());
    let document = d::pages::accounts::document(person.client()).expect("accounts delivered");
    assert!(document.usable, "the configured model is usable");
    assert!(!document.accounts.iter().any(|line| d::forge::demo::usable_account(line, None)));
    assert_eq!(d::pages::welcome::decision(person), Some(false), "the forge is not connected");
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
    assert!(!nodes.iter().any(|node| node.name.as_deref() == Some(b"Setup")));
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
        forge: 1,
        state: temper_engine_domain_forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
    })));
    accounts::settle(&mut front, &mut root);
    assert_eq!(
        d::pages::welcome::state(root.person()),
        Some(d::pages::welcome::State::Unfinished(Step::Forge { forge: None }))
    );
}

#[test]
fn a_set_up_cut_short_is_unfinished_after_a_reload_and_starts_over() {
    use super::connect_forge::{accept_owned, node, owned_welcome, press, run_owned, view};
    use temper_engine_domain_forge as forge;
    let (mut front, mut root, mut now) = owned_welcome();
    accept_owned(&mut front, &mut root, &mut now);
    front.fail_first_repository(1);
    assert_eq!(
        front.records()[0].first.as_ref().expect("durable intent").state,
        forge::FirstState::Failed { step: forge::CreateStep::Repository }
    );
    accounts::reopen(&mut front, &mut root, Address::Welcome { step: Step::Welcome }, false);
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Forge { forge: None } });
    node(&view(&root), Role::Heading { level: 2 }, "Forgejo on this machine · not finished".as_bytes());
    press(&mut front, &mut root, &mut now, b"Start over");
    assert!(front.removals.is_empty(), "start over confirms first");
    node(&view(&root), Role::Paragraph, b"Start over removes this forge and its records; the choices open again.");
    press(&mut front, &mut root, &mut now, b"Start over");
    assert_eq!(front.removals, [1]);
    assert!(front.records().is_empty(), "the removal's serve acts have finished");
    node(&view(&root), Role::Button, b"Run on this machine");
    run_owned(&mut front, &mut root, &mut now);
    assert_eq!(front.makes, [forge::ForgeChoice::Owned, forge::ForgeChoice::Owned]);
    assert_eq!(front.records().len(), 1);
    assert_eq!(front.records()[0].number, 2, "a removed forge's number is never reused");
    assert_eq!(front.set_ups().iter().map(|(forge, _)| *forge).collect::<Vec<_>>(), [1, 2]);
}

#[test]
fn github_without_an_installation_goes_on_after_a_reload() {
    use super::connect_forge::{node, press, view};
    let (mut front, mut root) = temper_client_world::front::Joint::github_fixture_with(fixtures::SEED, false, false);
    accounts::reopen(&mut front, &mut root, Address::Welcome { step: Step::Welcome }, false);
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Forge { forge: None } });
    let page = view(&root);
    node(&page, Role::Heading { level: 2 }, "GitHub · not finished".as_bytes());
    let go_on = node(&page, Role::Button, b"Go on");
    let start_over = node(&page, Role::Button, b"Start over");
    assert!(
        page.tree().nodes().iter().position(|n| n.id == go_on)
            < page.tree().nodes().iter().position(|n| n.id == start_over)
    );
    assert!(front.installs.is_empty());
    let mut now = skein_lib::Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1));
    press(&mut front, &mut root, &mut now, b"Go on");
    assert_eq!(front.installs.len(), 1);
    assert_eq!(front.installs[0].0, 1);
    assert!(front.removals.is_empty());
    assert!(front.makes.is_empty(), "Go on uses the record kept before the reload");
    assert_eq!(front.records()[0].number, 1);
}

#[test]
fn an_account_added_before_stays_ready_at_model_after_a_start_over() {
    use super::connect_forge::{accept_owned, go, owned_welcome, press, run_owned, view};
    let (mut front, mut root, mut now) = owned_welcome();
    accept_owned(&mut front, &mut root, &mut now);
    front.fail_first_repository(1);
    let way = d::ModelWay { source: None, provider: accounts::PROVIDER, kind: d::WayKind::Key };
    go(&mut front, &mut root, &mut now, Address::Welcome { step: Step::Model { way: Some(way) } });
    accounts::event(
        &mut front,
        &mut root,
        d::Event::AccountKey { value: Secret::new(b"test-private-key".as_slice().into()) }.into(),
    );
    accounts::event(
        &mut front,
        &mut root,
        d::Event::Act { action: d::Action::Pages(Act::ConnectModel(A::Submit { name: [45; 16] })) }.into(),
    );
    accounts::kept(&mut front);
    accounts::settle(&mut front, &mut root);
    let account = d::pages::accounts::document(root.person().client())
        .expect("account facts")
        .accounts
        .iter()
        .find(|line| d::forge::demo::usable_account(line, None))
        .expect("added usable account")
        .clone();
    accounts::reopen(&mut front, &mut root, Address::Welcome { step: Step::Welcome }, false);
    press(&mut front, &mut root, &mut now, b"Start over");
    press(&mut front, &mut root, &mut now, b"Start over");
    assert!(front.records().is_empty());
    run_owned(&mut front, &mut root, &mut now);
    assert_eq!(front.records()[0].number, 2);
    assert!(
        accept_owned(&mut front, &mut root, &mut now),
        "the second setup reaches Model: setup={:?}, address={:?}, calls={:?}",
        root.person().forge().set_up(),
        root.person().pages().address(),
        front.set_ups()
    );
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Model { way: None } });
    let retained = d::pages::accounts::document(root.person().client())
        .expect("retained account facts")
        .accounts
        .iter()
        .find(|line| line.number == account.number)
        .expect("same account");
    assert_eq!(retained, &account);
    assert_eq!(d::pages::welcome::model_usable(root.person()), Some(true));
    let model = view(&root);
    assert!(model.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&temper_client_view::look::FLOW_DONE)));
    assert!(
        model
            .tree()
            .nodes()
            .iter()
            .any(|n| n.role == Role::Paragraph && n.name.as_deref().is_some_and(|name| name.starts_with(b"Model: ")))
    );
}
