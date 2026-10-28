//! Accounts disclosures and acts, checked against the design alone.
mod qa_model;
use qa_model::*;
use skein_ui::{Role, State};
const DEPLOYMENT_ROW: &[u8] = "Scripted API key · held by the deployment".as_bytes();

// client/README.md 3.5 Accounts: empty own accounts still offer Connect a model.
#[test]
fn empty_own_accounts_offer_a_model_connection() {
    let mut world = world();
    let client = open(&mut world, b"/accounts");
    follow(
        &mut world,
        client,
        vec![
            see(Role::Heading { level: 2 }, b"Your accounts"),
            see(Role::Paragraph, b"No model of yours yet."),
            see(Role::Link, b"Connect a model"),
        ],
    );
}

// client/README.md 3.5 Accounts and 5.2: deployment accounts start folded.
#[test]
fn deployment_accounts_start_folded() {
    let mut world = world();
    let client = open(&mut world, b"/accounts");
    follow(&mut world, client, vec![see(Role::Heading { level: 2 }, b"Deployment accounts")]);
    let rows: Vec<_> = world
        .tree(client)
        .nodes()
        .iter()
        .filter(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(DEPLOYMENT_ROW))
        .collect();
    assert!(!rows.is_empty(), "the fixture restores a deployment account");
    for row in rows {
        assert!(!row.states.as_slice().contains(&State::Expanded), "an account starts folded");
    }
}

// client/README.md 3.5 Accounts and 5.1: the fake person finds a folded row's content whole.
#[test]
fn account_rows_expose_their_account_and_source_acts() {
    let mut world = world();
    let client = open(&mut world, b"/accounts");
    follow(&mut world, client, vec![see(Role::Button, b"Remove account"), see(Role::Button, b"Replace")]);
}

// client/README.md 3.5 Accounts: Remove account asks to confirm before changing anything.
#[test]
fn removing_an_account_waits_for_confirmation() {
    let mut world = world();
    let client = open(&mut world, b"/accounts");
    follow(&mut world, client, vec![press(Role::Button, b"Remove account")]);
    assert!(
        world
            .tree(client)
            .nodes()
            .iter()
            .any(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(DEPLOYMENT_ROW))
    );
}

// client/README.md 3.5 Accounts: Connect a model reaches the flow's Choose.
#[test]
fn accounts_connect_link_opens_choose() {
    let mut world = world();
    let client = open(&mut world, b"/accounts");
    follow(
        &mut world,
        client,
        vec![
            press(Role::Link, b"+ Connect a model"),
            see(Role::Heading { level: 1 }, b"Connect a model"),
            see(Role::Link, b"Use an API key"),
        ],
    );
}

// client/README.md 5.3: Accounts navigation is current on the connection flow too.
#[test]
fn accounts_navigation_is_current_on_connect_model() {
    let mut world = world();
    let client = open(&mut world, b"/accounts/connect");
    follow(&mut world, client, vec![see(Role::Link, b"Accounts")]);
    let entry = world
        .tree(client)
        .nodes()
        .iter()
        .find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Accounts"))
        .expect("accounts navigation");
    assert!(entry.states.as_slice().contains(&State::Current));
}

// client/README.md 3.5 Accounts: the person's accounts come before the deployment's.
#[test]
fn connected_own_account_precedes_the_deployment_section() {
    let mut world = world();
    let client = open(&mut world, b"/accounts/connect");
    follow(
        &mut world,
        client,
        vec![
            press(Role::Link, b"Use an API key"),
            see(Role::Button, b"Add key"),
            type_key(),
            press(Role::Button, b"Add key"),
            see(Role::Link, b"Start a chat"),
        ],
    );
    temper_client_world::front::account_acts::kept(world.front());
    follow(
        &mut world,
        client,
        vec![
            skein_fake_person::Step::Go { address: at(b"/accounts") },
            see(Role::Disclosure, "Scripted API key · held by Person".as_bytes()),
        ],
    );
    let nodes = world.tree(client).nodes();
    let own = nodes
        .iter()
        .position(|n| {
            n.role == Role::Disclosure && n.name.as_deref() == Some("Scripted API key · held by Person".as_bytes())
        })
        .expect("own account");
    let deployment = nodes
        .iter()
        .position(|n| n.role == Role::Heading { level: 2 } && n.name.as_deref() == Some(b"Deployment accounts"))
        .expect("deployment section");
    assert!(own < deployment);
}
