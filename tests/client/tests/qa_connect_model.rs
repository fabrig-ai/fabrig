//! Design-led accounts flow QA with the existing joined world's fakes.
mod qa_model;
use qa_model::*;
use skein_fake_person::Step;
use skein_ui::Role;
use temper_client_domain::{Address, address};
fn way(kind: &[u8], source: Option<u32>) -> Vec<u8> {
    let config = temper_engine_domain_world::walking::config(temper_client_world::fixtures::SEED);
    let provider = config.core.settings.llm_providers.providers[0].number;
    let mut path = format!("/accounts/connect?provider={provider}&way={}", String::from_utf8_lossy(kind));
    if let Some(source) = source {
        use std::fmt::Write;
        write!(&mut path, "&source={source}").expect("write address");
    }
    path.into_bytes()
}

fn gone(role: Role, name: &[u8]) -> Step<Address> {
    Step::Gone {
        find: skein_fake_person::Find::exactly_named(role, name),
        within: temper_client_world::fixtures::WITHIN,
    }
}
// client/browser.md 2.3: each way and a sign-in's source have canonical addresses.
#[test]
fn every_way_round_trips_its_address() {
    for path in [
        b"/accounts/connect".as_slice(),
        b"/accounts/connect?provider=17&way=key",
        b"/accounts/connect?provider=17&way=program",
        b"/accounts/connect?provider=17&way=sign-in",
        b"/accounts/connect?provider=17&way=sign-in&source=7",
        b"/welcome?at=model&provider=17&way=key",
        b"/welcome?at=model&provider=17&way=program",
        b"/welcome?at=model&provider=17&way=sign-in&source=7",
    ] {
        assert_eq!(address::print(&at(path)).as_ref(), path);
    }
}

// client/browser.md 2.3: incomplete ways and sources on non-sign-in ways drop.
#[test]
fn malformed_way_fields_drop_without_refusing_the_page() {
    for (input, printed) in [
        ("/accounts/connect?provider=17", "/accounts/connect"),
        ("/accounts/connect?way=key", "/accounts/connect"),
        ("/accounts/connect?provider=017&way=key", "/accounts/connect"),
        ("/accounts/connect?provider=17&way=unknown", "/accounts/connect"),
        ("/accounts/connect?provider=17&way=key&source=7", "/accounts/connect?provider=17&way=key"),
        ("/accounts/connect?provider=17&way=program&source=7", "/accounts/connect?provider=17&way=program"),
        ("/accounts/connect?provider=17&way=sign-in&source=07", "/accounts/connect?provider=17&way=sign-in"),
    ] {
        assert_eq!(address::print(&at(input.as_bytes())).as_ref(), printed.as_bytes(), "{input}");
    }
}

// client/README.md 3.5 ConnectModel: a way the providers do not offer shows Choose.
#[test]
fn an_unknown_provider_shows_choose() {
    let mut world = world();
    let client = open(&mut world, b"/accounts/connect?provider=999&way=key");
    follow(&mut world, client, vec![see(Role::Link, b"Use an API key"), gone(Role::Button, b"Add key")]);
}

// client/README.md 3.5 ConnectModel: deployment choice offers keys alone.
#[test]
fn deployment_choice_hides_the_personal_sign_in_way() {
    let mut world = world();
    let client = open(&mut world, b"/accounts/connect");
    follow(
        &mut world,
        client,
        vec![
            press(Role::Button, b"Use deployment accounts"),
            see(Role::Link, b"Use an API key"),
            gone(Role::Button, b"Sign in to Scripted"),
        ],
    );
}

// client/README.md 3.5 ConnectModel: every step after Choose offers All ways.
#[test]
fn all_ways_returns_key_details_to_choose() {
    let mut world = world();
    let client = open(&mut world, way(b"key", None).as_slice());
    follow(
        &mut world,
        client,
        vec![
            see(Role::Button, b"Add key"),
            press(Role::Link, "‹ All ways".as_bytes()),
            see(Role::Link, b"Use an API key"),
            gone(Role::Button, b"Add key"),
        ],
    );
}

// client/README.md 3.5 ConnectModel: a direct sign-in link offers its button without starting.
#[test]
fn direct_sign_in_link_waits_for_a_press() {
    let mut world = world();
    let client = open(&mut world, way(b"sign-in", None).as_slice());
    follow(&mut world, client, vec![see(Role::Button, b"Sign in to Scripted")]);
    assert!(
        !world
            .front()
            .engine
            .protocol
            .accounts
            .iter()
            .any(|r| matches!(r, jig_domain_accounts::Request::Signing(jig_domain_accounts::Signing::SignIn { .. })))
    );
}

// client/README.md 3.5 Come back: a source absent from the watch offers the way again.
#[test]
fn unknown_return_source_offers_sign_in_again() {
    let mut world = world();
    let client = open(&mut world, way(b"sign-in", Some(999)).as_slice());
    follow(&mut world, client, vec![see(Role::Button, b"Sign in to Scripted")]);
}

// client/README.md 3.5 The welcome: Model uses the flow body inside the welcome frame.
#[test]
fn welcome_model_keeps_its_frame_when_opening_key_details() {
    let mut world = world();
    let client = open(&mut world, b"/welcome?at=model");
    follow(
        &mut world,
        client,
        vec![
            press(Role::Link, b"Use an API key"),
            see(Role::Button, b"Add key"),
            see(Role::Link, b"Later"),
            gone(Role::Link, b"Accounts"),
            gone(Role::Status, b"Live"),
            press(Role::Link, "‹ All ways".as_bytes()),
            see(Role::Link, b"Use an API key"),
            press(Role::Link, b"Use an API key"),
            press(Role::Link, "‹ Back".as_bytes()),
            see(Role::Link, b"Use an API key"),
        ],
    );
}

// client/README.md 3.5 Its state: leaving the flow drops a pasted key first.
#[test]
fn leaving_the_flow_drops_a_pasted_key() {
    let mut world = world();
    let client = open(&mut world, way(b"key", None).as_slice());
    follow(
        &mut world,
        client,
        vec![
            type_key(),
            Step::Go { address: at(b"/accounts") },
            Step::Go { address: at(way(b"key", None).as_slice()) },
        ],
    );
    follow(&mut world, client, vec![press(Role::Button, b"Add key")]);
    assert!(
        !world
            .front()
            .engine
            .protocol
            .accounts
            .iter()
            .any(|r| matches!(r, jig_domain_accounts::Request::Signing(jig_domain_accounts::Signing::Keep { .. })))
    );
}

// client/README.md 3.5 Ready: adding a key makes an account and offers Start a chat.
#[test]
fn adding_a_key_reaches_ready() {
    let mut world = world();
    let client = open(&mut world, way(b"key", None).as_slice());
    follow(&mut world, client, vec![type_key(), press(Role::Button, b"Add key")]);
    temper_client_world::front::account_acts::kept(world.front());
    follow(
        &mut world,
        client,
        vec![
            Step::See {
                find: skein_fake_person::Find::text(b"Added your Scripted API key"),
                within: temper_client_world::fixtures::WITHIN,
            },
            see(Role::Link, b"Start a chat"),
        ],
    );
}

// client/README.md 3.5 Choose and Details: a deployment key's card opens Details.
#[test]
fn deployment_key_choice_opens_details() {
    let mut world = world();
    let client = open(&mut world, b"/accounts/connect");
    follow(
        &mut world,
        client,
        vec![
            press(Role::Button, b"Use deployment accounts"),
            press(Role::Link, b"Use an API key"),
            see(Role::Button, b"Add key"),
        ],
    );
}

// client/README.md 3.6: a double press joins the pending request, adding one key.
#[test]
fn double_pressing_add_key_makes_one_source() {
    // One source write for the one submitted key (client/README.md 3.6).
    const ONE_KEY: usize = 1;
    let mut world = world();
    let client = open(&mut world, way(b"key", None).as_slice());
    follow(&mut world, client, vec![type_key()]);
    let add = world
        .tree(client)
        .nodes()
        .iter()
        .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Add key"))
        .expect("Add key button")
        .id;
    world.act(client, add, skein_ui::Act::Press);
    world.act(client, add, skein_ui::Act::Press);
    follow(&mut world, client, vec![see(Role::Link, b"Start a chat")]);
    assert_eq!(
        world
            .front()
            .engine
            .protocol
            .accounts
            .iter()
            .filter(|r| matches!(r, jig_domain_accounts::Request::Signing(jig_domain_accounts::Signing::Keep { .. })))
            .count(),
        ONE_KEY
    );
}

// client/README.md 3.5 Choose: the holder choice is kept across the flow's steps.
#[test]
fn deployment_choice_is_kept_across_details_and_choose() {
    let mut world = world();
    let client = open(&mut world, b"/accounts/connect");
    follow(
        &mut world,
        client,
        vec![
            press(Role::Button, b"Use deployment accounts"),
            press(Role::Link, b"Use an API key"),
            see(Role::Button, b"Add key"),
            press(Role::Link, "‹ All ways".as_bytes()),
            see(Role::Link, b"Use an API key"),
        ],
    );
    let choice = world
        .tree(client)
        .nodes()
        .iter()
        .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Use deployment accounts"))
        .expect("holder choice");
    assert!(choice.states.as_slice().contains(&skein_ui::State::Pressed));
}

// client/README.md 3.5 Its state: moving between the flow's steps keeps its key's value.
#[test]
fn all_ways_keeps_the_key_until_the_flow_is_left() {
    let mut world = world();
    let client = open(&mut world, way(b"key", None).as_slice());
    follow(
        &mut world,
        client,
        vec![
            type_key(),
            press(Role::Link, "‹ All ways".as_bytes()),
            see(Role::Link, b"Use an API key"),
            press(Role::Link, b"Use an API key"),
            see(Role::Button, b"Add key"),
            press(Role::Button, b"Add key"),
            see(Role::Link, b"Start a chat"),
        ],
    );
    temper_client_world::front::account_acts::kept(world.front());
}
