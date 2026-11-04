//! Playground journeys (temper/docs/design/client/README.md, 10).
use jig_client_world::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use skein_ui::Role;
use temper_client_domain::{self as d, Address, Place, Step};
use temper_client_world::{
    app::{Root, Temper},
    fixtures as f,
    front::{Joint, account_acts as accounts},
};

fn pump(front: &mut Joint, root: &mut Root, event: d::Event) -> usize {
    let limits = f::domain_limits();
    let mut now = Time::ZERO.saturating_add(Duration::from_secs(1));
    let mut out = Queue::uncapped();
    Temper::step(root, &Env { now, wall: Wall::EPOCH, limits }, event.into(), &mut out);
    let mut replaced = 0;
    for _ in 0..64 {
        Temper::take(root, &Env { now, wall: Wall::EPOCH, limits }, &mut out);
        while let Some(request) = out.pop() {
            if let d::Request::Show { address, replace } = &request {
                replaced += usize::from(*replace);
                Temper::step(
                    root,
                    &Env { now, wall: Wall::EPOCH, limits },
                    d::Event::Went { place: Place::Address(address.clone()) }.into(),
                    &mut out,
                );
            }
            if let Duty::Wire(request) = Temper::duty(request) {
                front.wire(ClientId(0), now, Wall::EPOCH, request);
            }
        }
        now = now.saturating_add(Duration::from_millis(3));
        let callbacks = front.poll(now, Wall::EPOCH);
        if callbacks.is_empty() {
            return replaced;
        }
        for (_, event) in callbacks {
            front.received(ClientId(0), &event);
            Temper::step(root, &Env { now, wall: Wall::EPOCH, limits }, event, &mut out);
        }
    }
    panic!("playground story did not settle");
}
fn go(front: &mut Joint, root: &mut Root, address: Address) -> usize {
    pump(front, root, d::Event::Went { place: Place::Address(address) })
}
fn view(root: &Root) -> temper_client_view::View {
    let limits = f::domain_limits();
    let words = f::phrasebooks(&limits);
    let bounds = f::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, f::FORGE);
    temper_client_view::build(&mut view, root.person(), Wall::EPOCH, &mut Queue::with_capacity(bounds.tree.patches));
    view
}
fn has(root: &Root, role: Role, name: &[u8]) -> bool {
    view(root).tree().nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}
fn press(front: &mut Joint, root: &mut Root, name: &[u8]) {
    let mut view = view(root);
    let id = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(name))
        .expect("button")
        .id;
    let action = temper_client_view::act(&mut view, id, skein_ui::Act::Press).expect("bound start");
    pump(front, root, d::Event::Act { action });
}
#[test]
fn try_run_and_leave_the_playground_through_its_banner() {
    let (mut front, mut root) = accounts::playground_fixture(f::SEED, false);
    assert!(has(&root, Role::Button, b"Try the playground"));
    press(&mut front, &mut root, b"Try the playground");
    assert_eq!(front.playground_starts.len(), 1);
    assert_eq!(root.person().pages().project(), Some(1));
    assert!(has(&root, Role::Region, b"Playground"));
    for address in [Address::Forges, Address::Accounts] {
        go(&mut front, &mut root, address);
        assert!(!has(&root, Role::Button, b"Connect a forge"));
        assert!(!has(&root, Role::Button, b"Add an account"));
    }
    go(&mut front, &mut root, f::chats_page());
    front.hold_playground_end = true;
    let mut rendered = view(&root);
    let id = rendered
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Link && node.name.as_deref() == Some(b"set up temper"))
        .expect("banner reset link")
        .id;
    let action = temper_client_view::act(&mut rendered, id, skein_ui::Act::Press).expect("reset action");
    pump(&mut front, &mut root, d::Event::Act { action });
    assert_eq!(front.playground_ends.len(), 1);
    assert!(
        view(&root)
            .tree()
            .nodes()
            .iter()
            .flat_map(|node| &node.text)
            .any(|run| run.text.as_ref() == "Leaving the playground…".as_bytes())
    );
    front.finish_playground_reset();
    accounts::settle(&mut front, &mut root);
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Forge { forge: None } });
    assert!(root.person().forge().playground().is_none());
    assert!(matches!(root.person().forges(), d::forge::forges::State::Rows(rows) if rows.is_empty()));
}

#[test]
fn a_running_playground_read_does_not_start_it_again() {
    let (mut front, mut root) = accounts::playground_fixture(f::SEED, true);
    assert!(front.playground_starts.is_empty());
    go(&mut front, &mut root, f::chats_page());
    go(&mut front, &mut root, f::chats_page());
    assert!(has(&root, Role::Region, b"Playground"));
    assert!(front.playground_starts.is_empty());
}

#[test]
fn running_playground_pages_list_the_scripted_facts_without_offering_to_add_anything() {
    let (mut front, mut root) = accounts::playground_fixture(f::SEED, true);
    for address in [Address::Forges, Address::Accounts, f::chats_page()] {
        go(&mut front, &mut root, address);
        assert!(has(&root, Role::Region, b"Playground"));
        for name in [b"Connect a forge".as_slice(), b"Add a forge", b"Connect GitHub", b"Add key", b"Add an account"] {
            for role in [Role::Button, Role::Link] {
                assert!(!has(&root, role, name), "playground offers {}", String::from_utf8_lossy(name));
            }
        }
    }
    go(&mut front, &mut root, Address::Accounts);
    assert!(has(&root, Role::Disclosure, "Scripted model · the playground’s · held by the deployment".as_bytes()));
    go(&mut front, &mut root, Address::Forges);
    assert!(d::forge::listed::playground(root.person()).is_some());
}
