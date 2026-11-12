//! Home's Skip crosses the client and engine (temper/docs/design/client/pages/README.md
//! , To begin, and 10; temper/docs/design/testing/README.md, 2.1).
use jig_client_kit::harness::{App, ClientId, Settings, Until, World};
use skein_lib::Duration;
use skein_world::domain::Verdict;
use temper_client_domain::{
    Action, Address, Event,
    pages::{
        Act,
        home::{self, Facts, Skip},
    },
};
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures,
    front::Joint,
};
use temper_engine_domain::{Key, Record};
use temper_engine_domain_forge as forge;

const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
// The owner's identity and sign-in occupy the first two people identifiers.
const OTHER_PARTY: u64 = 3;
const OTHER_SUBJECT: u64 = 8;
// The same bound on immediate protocol turns as the dismissal stories.
const SIGN_IN_TURNS: usize = 32;
const OFFLINE: Duration = Duration::from_millis(30);

type Clients = World<Temper, Joint>;

// Navigation opens a Home before the old page's handle retirements return.
// The session also holds trials and a running trial task.
fn limits() -> temper_client_domain::Limits {
    let mut limits = fixtures::domain_limits();
    limits.client.domain.handles = 18;
    limits.client.domain.watches = 18;
    limits
}

fn root(seed: u64) -> (Root, View) {
    let (_, view) = Temper::new(seed);
    (Root::Person(Box::new(temper_client_domain::unlinked(&limits(), seed))), view)
}

fn client(world: &mut Clients, party: u64, project: u32) -> ClientId {
    world.front().next_client_limits(limits());
    let who = world.client_with(party, Address::Home { project }, limits(), root);
    // To begin uses neither accounts nor status; leave room for both devices' Away and decided watches.
    let status = jig_client::Handle {
        asker: temper_client_domain::owner::Owner::Pages.asker(),
        number: world.root(who).person().pages().chats().expect("Home holds its chats").live.number - 1,
    };
    for handle in [temper_client_domain::pages::accounts::handle(), status] {
        world.event(who, Event::Client { event: jig_client::Event::Unwatch { handle } }.into());
    }
    who
}

fn world() -> Clients {
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() },
        Joint::with_to_begin_people(fixtures::SEED, limits()),
    );
    world.randomness(0, 0, 0);
    world
}

fn settle(world: &mut Clients) {
    let until = world.now().saturating_add(fixtures::WITHIN);
    let outcome = world.run(Until::Time(until));
    assert_eq!(outcome.verdict, Verdict::Passed, "jig's check: {outcome:?}");
}

fn facts(world: &Clients, who: ClientId, project: u32) -> Facts {
    let root = world.root(who).person();
    assert_eq!(root.pages().address(), &Address::Home { project });
    home::facts(root, project, skein_lib::Wall::EPOCH)
}

fn showing(world: &Clients, who: ClientId, project: u32) {
    let facts = facts(world, who, project);
    assert!(facts.to_begin(), "To begin shows for {who:?} on {project}: {facts:?}");
}

fn hidden(world: &Clients, who: ClientId) {
    let facts = facts(world, who, PROJECT);
    assert_eq!(facts.skip, Skip::Skipped, "the project's skip is known for {who:?}");
    assert!(!facts.to_begin(), "To begin stays hidden: {facts:?}");
}

fn press(world: &mut Clients, who: ClientId) {
    world.event(who, Event::Act { action: Action::Pages(Act::SkipToBegin) }.into());
}

fn one_row(world: &mut Clients) {
    let rows: Vec<_> = world
        .front()
        .engine
        .store
        .rows
        .iter()
        .filter_map(|(key, record)| {
            if let Key::Forge(forge::Key::Dismissed { .. }) = key { Some((key.clone(), record.clone())) } else { None }
        })
        .collect();
    assert_eq!(
        rows,
        [(
            Key::Forge(forge::Key::Dismissed {
                party: fixtures::PARTY,
                what: forge::PersonFact::ToBegin { project: PROJECT },
            }),
            Record::Forge {
                row: Box::new(forge::Stored::Dismissed {
                    party: fixtures::PARTY,
                    what: forge::PersonFact::ToBegin { project: PROJECT },
                })
            },
        )]
    );
}

#[test]
fn skip_on_home_hides_to_begin_on_every_device() {
    let mut world = world();
    let first = client(&mut world, fixtures::PARTY, PROJECT);
    let second = client(&mut world, fixtures::PARTY, PROJECT);
    settle(&mut world);
    showing(&world, first, PROJECT);
    showing(&world, second, PROJECT);
    press(&mut world, first);
    hidden(&world, first);
    settle(&mut world);
    hidden(&world, first);
    hidden(&world, second);
    // End the first two sessions before a third Home: settings is session-long,
    // and the engine shares its stream watch room across devices.
    for who in [first, second] {
        world.event(who, Event::Act { action: Action::Pages(Act::SignOut) }.into());
    }
    settle(&mut world);
    let third = client(&mut world, fixtures::PARTY, PROJECT);
    assert!(!facts(&world, third, PROJECT).to_begin(), "no flash before the dismissal arrives");
    // Check every delivery turn during the fresh sign-in, as well as its settled state.
    for _ in 0..SIGN_IN_TURNS {
        let outcome = world.run(Until::Steps(1));
        assert!(!matches!(outcome.verdict, Verdict::Failed(_) | Verdict::Stopped { .. }), "jig's check: {outcome:?}");
        assert!(!facts(&world, third, PROJECT).to_begin(), "no flash during sign-in");
    }
    settle(&mut world);
    hidden(&world, third);
    one_row(&mut world);
}

#[test]
fn skip_holds_across_a_restart() {
    let mut world = world();
    let first = client(&mut world, fixtures::PARTY, PROJECT);
    let second = client(&mut world, fixtures::PARTY, PROJECT);
    settle(&mut world);
    showing(&world, first, PROJECT);
    showing(&world, second, PROJECT);
    press(&mut world, first);
    settle(&mut world);
    hidden(&world, second);
    world.front().restart_dismissal_people(OFFLINE);
    settle(&mut world);
    hidden(&world, first);
    hidden(&world, second);
    // A fresh Home has no page-local acknowledgement from before the restart.
    world.go(second, Address::Home { project: OTHER_PROJECT });
    world.go(second, Address::Home { project: PROJECT });
    settle(&mut world);
    hidden(&world, second);
    one_row(&mut world);
}

#[test]
fn skip_leaves_the_others_to_begin() {
    let mut world = world();
    let first = client(&mut world, fixtures::PARTY, PROJECT);
    settle(&mut world);
    world.go(first, Address::Home { project: OTHER_PROJECT });
    settle(&mut world);
    showing(&world, first, OTHER_PROJECT);
    world.go(first, Address::Home { project: PROJECT });
    settle(&mut world);
    // Establish membership before the new Home opens its watches.
    let member = ClientId(1);
    world.front().bind_dismissal_person(member, OTHER_PARTY, OTHER_SUBJECT);
    world.front().add_dismissal_member(first, OTHER_PARTY, PROJECT);
    assert_eq!(client(&mut world, OTHER_PARTY, PROJECT), member);
    settle(&mut world);
    showing(&world, first, PROJECT);
    showing(&world, member, PROJECT);
    press(&mut world, first);
    settle(&mut world);
    hidden(&world, first);
    showing(&world, member, PROJECT);
    world.go(first, Address::Home { project: OTHER_PROJECT });
    settle(&mut world);
    showing(&world, first, OTHER_PROJECT);
    showing(&world, member, PROJECT);
    one_row(&mut world);
}

#[test]
fn a_double_press_sends_one_request() {
    let mut world = world();
    let first = client(&mut world, fixtures::PARTY, PROJECT);
    settle(&mut world);
    showing(&world, first, PROJECT);
    // Both acts precede the engine's answer to the first request.
    press(&mut world, first);
    press(&mut world, first);
    hidden(&world, first);
    settle(&mut world);
    hidden(&world, first);
    assert_eq!(world.front().dismissal_sends(), 1, "one wire request for both presses");
    one_row(&mut world);
}
