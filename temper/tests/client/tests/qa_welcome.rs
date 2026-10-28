//! Adversarial Welcome QA from client/README.md, 3.5, 4.8, 5.1, 5.6,
//! and client/browser.md, 2.3–2.4. Only the world's public surface is used.
use jig_client_world::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step as PersonStep};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{Address, Place, Step, address};
use temper_client_world::{app::Temper, fixtures as f, front::Joint};

type Pages = World<Temper, Joint>;

fn welcome(step: Step) -> Address {
    Address::Welcome { step }
}

fn see(role: Role, name: &[u8]) -> PersonStep<Address> {
    PersonStep::See { find: Find::exactly_named(role, name), within: f::WITHIN }
}

fn gone(role: Role, name: &[u8]) -> PersonStep<Address> {
    PersonStep::Gone { find: Find::exactly_named(role, name), within: f::WITHIN }
}

fn follow(world: &mut Pages, client: ClientId, steps: Vec<PersonStep<Address>>) {
    world.person(client, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

// Welcome deliberately has no link-state node (client/README.md, 3.5).
// Establish the world's visible Online expectation at Chats before navigating.
fn open_from_chats(world: &mut Pages, step: Step) -> ClientId {
    let client = world.client(f::PARTY, Address::Chats { filter: ChatsFilter::All, start: false });
    follow(world, client, vec![see(Role::Heading { level: 1 }, b"Chats")]);
    follow(world, client, vec![PersonStep::Go { address: welcome(step) }]);
    client
}

#[test]
fn every_welcome_step_has_its_canonical_address() {
    for (path, step) in [
        (b"/welcome".as_slice(), Step::Welcome),
        (b"/welcome?at=forge", Step::Forge { forge: None }),
        (b"/welcome?at=model", Step::Model { way: None }),
        (b"/welcome?at=ready", Step::Ready),
    ] {
        assert_eq!(address::parse(path), Place::Address(welcome(step)), "{path:?}");
        assert_eq!(address::print(&welcome(step)).as_ref(), path);
    }
}

#[test]
fn unknown_and_unrelated_fields_drop_from_welcome() {
    for (path, step, printed) in [
        ("/welcome?at=done", Step::Welcome, "/welcome"),
        ("/welcome?at=", Step::Welcome, "/welcome"),
        ("/welcome?at=Welcome", Step::Welcome, "/welcome"),
        ("/welcome?at=FORGE", Step::Welcome, "/welcome"),
        ("/welcome?project=1&kind=question", Step::Welcome, "/welcome"),
        ("/welcome?other=ready&at=model&goal=9", Step::Model { way: None }, "/welcome?at=model"),
        ("/welcome?at&at=ready&&x=1", Step::Ready, "/welcome?at=ready"),
        ("/welcome?at=model=ready", Step::Welcome, "/welcome"),
    ] {
        let place = address::parse(path.as_bytes());
        assert_eq!(place, Place::Address(welcome(step)), "{path}");
        assert_eq!(address::print(&welcome(step)).as_ref(), printed.as_bytes(), "{path}");
    }
}

#[test]
fn repeated_step_keeps_first_readable_value() {
    // browser.md, 2.3: as the inbox query, repeats after the first that read are dropped.
    for (path, step) in [
        ("/welcome?at=forge&at=model", Step::Forge { forge: None }),
        ("/welcome?at=model&at=ready", Step::Model { way: None }),
        ("/welcome?at=ready&at=done", Step::Ready),
        ("/welcome?at=done&at=forge", Step::Forge { forge: None }),
        ("/welcome?at=&at=ready", Step::Ready),
    ] {
        assert_eq!(address::parse(path.as_bytes()), Place::Address(welcome(step)), "{path}");
    }
}

#[test]
fn null_demo_owner_stays_and_offers_only_setup() {
    let mut world = f::accounts_world(f::SEED);
    let client = open_from_chats(&mut world, Step::Welcome);
    follow(
        &mut world,
        client,
        vec![see(Role::Heading { level: 1 }, b"Welcome to temper"), see(Role::Region, b"Set up temper")],
    );
    follow(&mut world, client, vec![
        gone(Role::Button, b"Try the playground"),
            gone(Role::Link, b"Try the playground"),
            gone(Role::Region, b"Try the playground"),
        gone(Role::Region, b"Playground"),
        gone(Role::Paragraph, b"Started in the playground? Connect your own forge and model whenever you like; the playground then steps aside."),
        gone(Role::Link, b"Inbox"),
            gone(Role::Link, b"Accounts"),
            gone(Role::Link, b"Project 1"),
            gone(Role::Button, b"Menu"),
        gone(Role::Status, b"Live"),
    ]);
}

#[test]
fn null_demo_later_steps_never_offer_the_playground() {
    for (step, heading) in [
        (Step::Forge { forge: None }, b"Connect a forge".as_slice()),
        (Step::Model { way: None }, b"Connect a model"),
        (Step::Ready, b"Almost ready"),
    ] {
        let mut world = f::accounts_world(f::SEED);
        let client = open_from_chats(&mut world, step);
        follow(&mut world, client, vec![see(Role::Heading { level: 1 }, heading)]);
        if step == Step::Ready {
            for role in [Role::Link, Role::Button] {
                follow(&mut world, client, vec![gone(role, b"Later"), gone(role, "‹ Back".as_bytes())]);
            }
        } else {
            follow(&mut world, client, vec![see(Role::Link, b"Later")]);
        }
        follow(
            &mut world,
            client,
            vec![
                gone(Role::Link, b"Try the playground instead"),
                gone(Role::Button, b"Try the playground instead"),
                gone(Role::Region, b"Playground"),
            ],
        );
    }
}

#[test]
fn non_deployment_owner_without_account_can_set_up() {
    // The ordinary world has project membership but no deployment role.
    // Its legacy model configuration does not establish a usable accounts row.
    let mut world = f::world(f::SEED);
    let client = open_from_chats(&mut world, Step::Welcome);
    follow(
        &mut world,
        client,
        vec![see(Role::Heading { level: 1 }, b"Welcome to temper"), gone(Role::Region, b"Playground")],
    );
}

#[test]
fn later_welcome_steps_do_not_redirect_on_their_own() {
    for (step, heading) in [
        (Step::Forge { forge: None }, b"Connect a forge".as_slice()),
        (Step::Model { way: None }, b"Connect a model"),
        (Step::Ready, b"Almost ready"),
    ] {
        let mut world = f::world(f::SEED);
        let client = open_from_chats(&mut world, step);
        follow(&mut world, client, vec![see(Role::Heading { level: 1 }, heading)]);
        follow(
            &mut world,
            client,
            vec![if step == Step::Ready { gone(Role::Link, b"Later") } else { see(Role::Link, b"Later") }],
        );
    }
}

#[test]
fn welcome_setup_card_offers_its_link_to_the_forge_step() {
    let mut world = f::accounts_world(f::SEED);
    let client = open_from_chats(&mut world, Step::Welcome);
    follow(&mut world, client, vec![see(Role::Heading { level: 1 }, b"Welcome to temper")]);
    let nodes = world.tree(client).nodes();
    assert!(
        // "Set up temper" moves to the Forge step: a move is a link, drawn as the card's button
        // (client/README.md, 5.6; browser.md, 2.3).
        nodes.iter().any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Set up temper")),
        "client/README.md, 5.6 promises each card its button, the set up card's a link; drawing: {:?}",
        nodes.iter().map(|node| (node.role, node.name.as_deref().map(String::from_utf8_lossy))).collect::<Vec<_>>()
    );
}

#[test]
#[ignore = "needs: non-deployment-owner identity with a usable accounts document; Joint binds only its configured owner identity"]
fn non_deployment_owner_with_account_ignores_forge_requirement() {
    let mut world = f::world(f::SEED);
    let client = open_from_chats(&mut world, Step::Welcome);
    follow(&mut world, client, vec![see(Role::Heading { level: 1 }, b"Chats")]);
}

#[test]
#[ignore = "needs: Online observation independent of the hidden link-state node when starting at Welcome"]
fn printed_link_home_opens_welcome_without_link_state() {
    let mut world = f::accounts_world(f::SEED);
    let client = world.client(f::PARTY, welcome(Step::Welcome));
    follow(
        &mut world,
        client,
        vec![see(Role::Heading { level: 1 }, b"Welcome to temper"), gone(Role::Status, b"Live")],
    );
}

#[test]
#[ignore = "needs: per-subject delivery hold/release for demo, accounts, forges and me; run all 24 arrival orders"]
fn landing_waits_for_every_fact_in_each_order() {
    let mut world = f::accounts_world(f::SEED);
    let client = world.client(f::PARTY, welcome(Step::Welcome));
    // Hold three subjects, deliver the fourth, and assert only the heading;
    // release each prefix, then expect the cards or a replacement to Chats.
    follow(
        &mut world,
        client,
        vec![see(Role::Heading { level: 1 }, b"Welcome to temper"), gone(Role::Button, b"Set up temper")],
    );
}

#[test]
#[ignore = "needs: per-subject refused/failed/ended/not-served replies for demo, accounts, forges and me"]
fn terminal_missing_facts_stop_waiting_without_inventing_setup() {
    let mut world = f::accounts_world(f::SEED);
    let client = world.client(f::PARTY, welcome(Step::Welcome));
    follow(&mut world, client, vec![see(Role::Button, b"Set up temper"), gone(Role::Region, b"Playground")]);
}

#[test]
#[ignore = "needs: browser raw-address and history observer; the world starts and moves using typed Address only"]
fn noncanonical_address_is_replaced_without_a_history_entry() {
    assert_eq!(address::parse(b"/welcome?at=done"), Place::Address(welcome(Step::Welcome)));
    // Open the raw URL, then observe replace('/welcome') and Back.
}

#[test]
#[ignore = "needs: non-null demo fixture and demo model/forge half controls; Joint answers every family read with none"]
fn playground_banner_follows_accounts_and_precedes_every_chatbar() {
    let mut world = f::accounts_world(f::SEED);
    let task = f::chat(world.front(), b"Playground banner QA");
    let client = world.client(f::PARTY, f::chat_page(task));
    // With the demo installed, compare semantic tree order on Chat,
    // demo-project Chats (New chat), and ordinary Chats; repeat each half.
    follow(&mut world, client, vec![see(Role::Region, b"Playground")]);
}

#[test]
#[ignore = "needs: non-null running demo fixture and Show history observer"]
fn running_demo_redirects_once_and_welcome_never_draws_banner() {
    let mut world = f::accounts_world(f::SEED);
    let client = world.client(f::PARTY, welcome(Step::Welcome));
    follow(&mut world, client, vec![see(Role::Heading { level: 1 }, b"Chats")]);
    // Observe one replacement; reopening Welcome at Forge must stay there,
    // with neither Playground nor the accounts banner nor setup strip.
}
