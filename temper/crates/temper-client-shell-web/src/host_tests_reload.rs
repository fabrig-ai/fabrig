//! The page's one reload is separate from the client's records and the view's words.
use super::*;
use crate::reload::Action;

fn starting() -> Host {
    let mut host = Host::new(limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
    host.root = root::unlinked(&host.env.limits, 11);
    host.event(Event::Start { address: Address::Accounts, kept: List::with_capacity(0), with: None });
    host.client(jig_client::Event::StartUnlinked { kept: List::with_capacity(0) });
    host.client(jig_client::Event::SignIn { with: jig_client::With::Transport });
    host
}

fn refuse_link(host: &mut Host) {
    host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } });
}

fn refuse_forge(host: &mut Host) {
    host.client(jig_client::Event::Up {
        up: jig_client::Up::FamilyServed { family: temper_client_protocol_wire::FORGE, versions: None },
    });
}

fn out_of_date_column(host: &mut Host) {
    build(host);
    assert!(
        host.tree()
            .nodes()
            .iter()
            .any(|node| { node.role == Role::Heading { level: 1 } && node.name.as_deref() == Some(b"Out of date") })
    );
    assert!(
        host.tree()
            .nodes()
            .iter()
            .flat_map(|node| &node.text)
            .any(|run| { run.text.as_ref() == b"Run the temper the engine runs." })
    );
}

#[test]
fn an_out_of_date_link_reloads_once_then_shows_the_column() {
    let mut host = fixture();
    refuse_link(&mut host);
    assert_eq!(host.reload(Some(false)), Action::Reload);

    // A new page has the session marker, but has not heard from the engine yet.
    let mut reloaded = starting();
    assert_eq!(reloaded.reload(Some(true)), Action::None);
    refuse_link(&mut reloaded);
    assert_eq!(reloaded.reload(Some(true)), Action::None);
    out_of_date_column(&mut reloaded);
}

#[test]
fn an_unserved_forge_reloads_once_even_while_the_link_is_live() {
    let mut host = fixture_at(b"/projects/1/adopt");
    refuse_forge(&mut host);
    assert_eq!(host.root.client().link().state, jig_client::Link::Live);
    assert_eq!(host.reload(Some(false)), Action::Reload);

    let mut reloaded = fixture_at(b"/projects/1/adopt");
    refuse_forge(&mut reloaded);
    assert_eq!(reloaded.reload(Some(true)), Action::None);
    build(&mut reloaded);
    assert!(reloaded.tree().nodes().iter().flat_map(|node| &node.text).any(|run| {
        run.text.as_ref() == b"This temper is out of date for adopting repositories: run the temper the engine runs."
    }));
}

#[test]
fn a_page_never_out_of_date_never_reloads() {
    let host = starting();
    assert_eq!(host.reload(Some(false)), Action::None);
    let live = fixture();
    assert_eq!(live.reload(Some(false)), Action::None);
}

#[test]
fn a_live_page_clears_the_marker_for_a_later_redeploy() {
    let mut host = fixture();
    assert_eq!(host.reload(Some(true)), Action::Clear);
    refuse_link(&mut host);
    assert_eq!(host.reload(Some(false)), Action::Reload);
}

#[test]
fn unavailable_storage_shows_the_column_without_reloading() {
    let mut host = fixture();
    refuse_link(&mut host);
    assert_eq!(host.reload(None), Action::None);
    out_of_date_column(&mut host);
    let mut forge = fixture();
    refuse_forge(&mut forge);
    assert_eq!(forge.reload(None), Action::None);
}
