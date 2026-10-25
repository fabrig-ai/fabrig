//! GitHub QA on the joint engine/client world, with scripted setup and picker boundaries.
use temper_client_world::fixtures;

#[test]
#[ignore = "gap: Joint does not bridge App paste requests"]
fn the_web_app_form_masks_and_erases_a_rejected_paste() {
    let world = fixtures::world(fixtures::SEED);
    drop(world);
    // client/forge.md 5.4: visit own App Details, fill App id and secret key;
    // submit a refused paste; id stays, key is empty, no key in saved page state.
    panic!("gap: the joint world does not bridge App paste requests");
}

use jig_client_world::harness::{App, ClientId, Front};
use skein_lib::{Queue, Time, Wall};
use skein_ui::Role;
use temper_client_domain::{self as d, Address, Choice};
use temper_client_world::{
    app::Root,
    front::{Joint, account_acts},
};
use temper_engine_domain_forge as forge;

const CLIENT: ClientId = ClientId(0);

fn view(root: &Root) -> temper_client_view::View {
    let limits = fixtures::domain_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(&mut view, root.person(), Wall::EPOCH, &mut Queue::with_capacity(bounds.tree.patches));
    view
}
fn node(view: &temper_client_view::View, role: Role, name: &[u8]) -> skein_ui::NodeId {
    view.tree()
        .nodes()
        .iter()
        .find(|n| n.role == role && n.name.as_deref() == Some(name))
        .unwrap_or_else(|| {
            panic!(
                "missing {role:?}: {}; page: {:?}",
                String::from_utf8_lossy(name),
                view.tree()
                    .nodes()
                    .iter()
                    .filter_map(|n| n.name.as_ref().map(|name| (n.role, String::from_utf8_lossy(name))))
                    .collect::<Vec<_>>()
            )
        })
        .id
}
fn revisit(front: &mut Joint, root: &mut Root) {
    account_acts::event(
        front,
        root,
        d::Event::Went { place: d::Place::Address(Address::ConnectForge { forge: Some(Choice::Forge(1)) }) }.into(),
    );
}
fn install(front: &mut Joint, root: &mut Root, url: &[u8]) {
    let mut page = view(root);
    let button = node(&page, Role::Button, b"Install on GitHub");
    let action = temper_client_view::act(&mut page, button, skein_ui::Act::Press).expect("Install action");
    account_acts::event(front, root, d::Event::Act { action }.into());
    let (forge, request) = *front.installs.last().expect("Install reaches the setup boundary");
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::InstallAnswered {
        forge,
        request,
        answer: forge::InstallAnswer::Link(url.into()),
        stopped: None,
    })));
    front.engine.settle();
    revisit(front, root);
}

#[test]
fn qa_an_abandoned_install_stays_connecting_and_install_again_can_accept() {
    // F-m3-2: no fake cancellation response exists. The first install tab is abandoned.
    let (mut front, mut root) = Joint::github_fixture_with(fixtures::SEED, false, false);
    node(&view(&root), Role::Heading { level: 2 }, "Install temper’s App".as_bytes());
    install(&mut front, &mut root, b"https://github.test/apps/app/installations/new?state=first");
    assert_eq!(front.installs.len(), 1);
    assert_eq!(
        front.leaves.last().expect("first tab departure").as_ref(),
        b"https://github.test/apps/app/installations/new?state=first"
    );
    let d::forge::forges::State::Rows(rows) = root.person().forges() else { panic!("forge rows") };
    assert!(rows[0].installations.is_empty());
    assert!(d::forge::connect::underway(&rows[0]));
    assert!(front.reachable_reads.is_empty(), "an abandoned install gives no repository rows");
    node(&view(&root), Role::Paragraph,
        b"app is connected to temper, and installed nowhere yet. Install it, and choose the repositories temper may use.");
    // Returning to temper is a fresh page/client, as after leaving a browser tab.
    front.reload(CLIENT, Time::ZERO);
    root = Root::Person(Box::new(d::unlinked(&fixtures::domain_limits(), fixtures::SEED + 1)));
    account_acts::event(
        &mut front,
        &mut root,
        temper_client_world::app::Temper::start(Address::ConnectForge { forge: Some(Choice::Forge(1)) }, Vec::new(), 0),
    );
    revisit(&mut front, &mut root);
    node(&view(&root), Role::Heading { level: 2 }, "Install temper’s App".as_bytes());
    install(&mut front, &mut root, b"https://github.test/apps/app/installations/new?state=second");
    assert_eq!(front.installs.len(), 2);
    assert_ne!(front.installs[0].1, front.installs[1].1, "Install again is a new request");
    assert_eq!(front.leaves.len(), 2, "one departure per answered link");
    assert_ne!(front.leaves[0], front.leaves[1], "the new install shows its fresh state");
    front.engine.send(temper_engine_domain::Event::Forge(Box::new(forge::Event::Installation {
        forge: 1,
        installation: 2,
        state: Some(forge::Installation {
            account: Some((1, b"acme".as_slice().into())),
            suspended: false,
            stray: false,
        }),
    })));
    front.engine.settle();
    revisit(&mut front, &mut root);
    let ready = view(&root);
    node(&ready, Role::Heading { level: 2 }, b"Choose a repository to start in");
    node(&ready, Role::Choice, b"acme/shop");
    node(&ready, Role::Choice, b"acme/second");
    let d::forge::forges::State::Rows(rows) = root.person().forges() else { panic!("ready rows") };
    assert_eq!(rows[0].installations.len(), 1);
    assert!(!d::forge::connect::underway(&rows[0]));
}

#[test]
fn qa_ready_for_an_account_without_repositories_says_none_reachable() {
    let (front, root) = Joint::github_fixture_with(fixtures::SEED, true, true);
    assert_eq!(front.reachable_reads, [(1, Some(1), 1)]);
    let ready = view(&root);
    node(&ready, Role::Heading { level: 2 }, b"Choose a repository to start in");
    node(&ready, Role::Status, b"None reachable.");
    assert!(!ready.tree().nodes().iter().any(|n| n.role == Role::Choice));
    assert!(!ready.tree().nodes().iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"More")));
}
