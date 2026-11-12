//! Accepted follow and one queued source landing, read through ctl (domain/forge/follows.md, 7).
use jig_client_kit::harness::{App, ClientId, Settings, Until, World};
use skein_lib::{Duration, ReplyTo, Token};
use temper_client_domain::Address;
use skein_ui::Role;
use temper_engine_domain as engine;
use jig_domain_people as people;
use temper_client_world::{
    app::Temper,
    fixtures,
    front::{
        Joint,
        ways::{self},
    },
};
use temper_engine_domain_forge as forge;
type Clients = World<Temper, Joint>;
fn call(world: &mut Clients, agent: ClientId, words: &[&[u8]]) -> String {
    let number = fixtures::give(world, agent, words);
    let printed = fixtures::answered(world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    String::from_utf8(printed.stdout.into_vec()).expect("follow fixture value")
}
#[test]
#[expect(clippy::too_many_lines, reason = "one story checks ctl and both project pages across a landing")]
fn an_accepted_follow_queues_a_landing_and_ctl_reads_its_lag() {
    let mut front = ways::follows(fixtures::SEED, &fixtures::domain_limits());
    front.engine.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]), reply_to: ReplyTo::new(Token::new(81)), sign_in: front.engine.session(),
        key: [81; 16], ask: people::Ask::MakeProject { name: "Source".into(), policy: 1 },
    }));
    front.engine.settle();
    assert!(front.engine.delivered.iter().any(|delivery| matches!(delivery,
        engine::Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::ProjectMade { project: 2, .. }), .. })));
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), iterations: 4096, ..Settings::default() },
        front,
    );
    world.randomness(0, 0, 0);
    let agent = fixtures::agent(&mut world);
    assert_eq!(call(&mut world, agent, &[b"follow", b"list", b"--project", b"1"]), "no follows\n");
    for (name, role, key) in
        [(b"shop".as_slice(), b"context".as_slice(), b"source".as_slice()), (b"second", b"owned", b"target")]
    {
        call(
            &mut world,
            agent,
            &[
                b"repository",
                b"adopt",
                b"--project",
                b"1",
                b"--owner",
                b"acme",
                b"--name",
                name,
                b"--landing",
                b"main",
                b"--role",
                role,
                b"--key",
                key,
            ],
        );
    }
    call(&mut world, agent, &[b"repository", b"adopt", b"--project", b"2", b"--owner", b"acme", b"--name", b"shop", b"--landing", b"main", b"--role", b"owned", b"--key", b"source-owned"]);
    let caller = world.front().caller;
    let target = world.client(caller, Address::Settings { project: 1 });
    world.run(Until::Time(world.now().saturating_add(fixtures::WITHIN)));
    let number = fixtures::give_with_file(&mut world, agent,
        &[b"settings", b"propose", b"follow.json", b"Follow the source", b"--against", b"1", b"--key", b"follow"],
        br#"{"items":[{"setting":{"kind":"follow","project":1,"number":1},"set":{"source":{"forge":1,"repository":7},"target":{"forge":1,"repository":8},"owner":{"person":1},"check":{"workflow":"tests.yml","artifact":"results","format":"junit"}}}]}"#);
    let proposed = fixtures::answered(&mut world, agent, number);
    assert_eq!(proposed.status, 0, "{proposed:?}");
    let proposed = String::from_utf8(proposed.stdout.into_vec()).expect("follow fixture value");
    let proposal = proposed
        .strip_prefix("proposed: ")
        .expect("follow fixture value")
        .split_once(", ")
        .expect("follow fixture value")
        .0;
    call(
        &mut world,
        agent,
        &[b"proposal", b"decide", proposal.as_bytes(), b"accept", b"--running", b"stays", b"--key", b"accept-follow"],
    );
    // Finish the first watched read: acceptance is a baseline, not a lag entry.
    world.run(Until::Time(world.now().saturating_add(Duration::from_secs(2))));
    assert_eq!(
        call(&mut world, agent, &[b"follow", b"list", b"--project", b"1"]),
        "follow 1: acme/shop main → acme/second main · caught up · no check yet\n"
    );
    assert_follow_page(&world, target, "caught up · no check yet");
    world.front().engine.forge.as_mut().expect("follow fixture value").push_checked(7, b"main", false, true);
    world.run(Until::Time(world.now().saturating_add(Duration::from_secs(40))));
    let saved = world
        .front()
        .engine
        .store
        .rows
        .get(&temper_engine_domain::Key::Forge(forge::Key::Follow { project: 1, follow: 1 }))
        .expect("follow fixture value")
        .clone();
    let temper_engine_domain::Record::Forge { row } = saved else { panic!("forge row") };
    let forge::Stored::Follow(saved) = row.as_ref() else { panic!("follow row") };
    let [entry] = saved.queue.as_ref() else { panic!("one source landing queued: {saved:?}") };
    assert_eq!(entry.state, forge::FollowState::NotMade);
    assert_eq!(
        call(&mut world, agent, &[b"follow", b"list", b"--project", b"1"]),
        "follow 1: acme/shop main → acme/second main · 1 behind · no check yet\n"
    );
    assert_follow_page(&world, target, "1 behind · no check yet");
    world.event(target, Temper::went(Address::Settings { project: 2 }));
    world.run(Until::Time(world.now().saturating_add(Duration::from_secs(2))));
    assert!(!world.tree(target).nodes().iter().any(|node| node.styles.as_slice().contains(&temper_client_view::look::HOW_FOLLOWS)), "source owns no follow");
    let shown = call(&mut world, agent, &[b"follow", b"show", b"--project", b"1", b"1"]);
    let after = temper_client_domain::forge::follows::after12(&entry.after);
    assert!(
        shown.contains(&format!("  {} · not made yet\n", std::str::from_utf8(&after).expect("follow fixture value"))),
        "{shown}"
    );
    assert!(shown.contains("source: acme/shop (1/7)\n"), "{shown}");
    assert!(shown.contains("target: acme/second (1/8)\n"), "{shown}");
    assert!(shown.contains("owner: person 1\nstart: owner\nproducer: default\n"), "{shown}");
    assert!(shown.ends_with("skips: none\nparity: no check yet\n"), "{shown}");
    assert_eq!(call(&mut world, agent, &[b"follow", b"list", b"--project", b"2"]), "no follows\n");
}

fn assert_follow_page(world: &Clients, client: ClientId, status: &str) {
    let nodes = world.tree(client).nodes();
    let reaches = nodes.iter().position(|node| node.role == Role::Heading { level: 2 } && node.name.as_deref() == Some(b"What reaches you")).expect("reaches");
    let follows = nodes.iter().position(|node| node.role == Role::Heading { level: 2 } && node.name.as_deref() == Some(b"What follows a landing")).expect("follows");
    assert!(follows > reaches);
    for (role, words) in [(Role::Heading { level: 3 }, "acme/second follows acme/shop"), (Role::Text, "code triggers, agents judge"), (Role::Paragraph, status)] {
        assert!(nodes.iter().any(|node| node.role == role && node.name.as_deref() == Some(words.as_bytes())), "missing {words}");
    }
}
