//! Decisions on a chat's settings card use its proposal's project
//! (temper/docs/design/client/README.md, 5.13 and 3.6).
use jig_client::contract as c;
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step, Target};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::{Action, Address, Event, pages::Act};
use temper_client_world::{app::Temper, fixtures, front::Joint};

type ClientWorld = World<Temper, Joint>;
const PROJECT: u32 = 1;
const ACCEPTED: &[u8] = "✓ Accepted · new work follows it".as_bytes();
const REASON: &[u8] = b"Keep the current ways of working.";

fn proposed() -> (ClientWorld, ClientId, u64, ClientId) {
    let value = temper_engine_domain_forge::settings::Value::Ways(temper_engine_domain_forge::settings::ProjectWays {
        drafts: true,
        goals: true,
        changes: false,
        away: true,
        way: 0,
    });
    let item = temper_engine_domain_world::kinds::item(7, PROJECT, 1, &value);
    let (mut world, agent, task, _) = super::configuring_chat::propose(false, Some(Box::new([item])));
    let person = world.client(fixtures::PARTY, Address::Chat { task });
    steps(&mut world, person, vec![see(Find::exactly_named(Role::Button, b"Accept"))]);
    assert_eq!(rail(&world, person), 1);
    (world, agent, task, person)
}

fn see(find: Find) -> Step<Address> {
    Step::See { find, within: fixtures::WITHIN }
}
fn steps(world: &mut ClientWorld, person: ClientId, steps: Vec<Step<Address>>) {
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn rail(world: &ClientWorld, person: ClientId) -> u32 {
    let root = world.root(person).person();
    root.pages().needs().count(root.client(), PROJECT).count
}
fn newest(world: &mut ClientWorld, agent: ClientId, revision: u64) {
    let number = fixtures::give(world, agent, &[b"settings", b"show"]);
    let printed = fixtures::answered(world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    let settings = String::from_utf8(printed.stdout.into_vec()).expect("settings UTF-8");
    assert!(settings.starts_with(&format!("newest revision {revision}\n")), "{settings}");
}

#[test]
fn a_configuring_chat_accepts_its_own_proposal() {
    let (mut world, agent, task, person) = proposed();
    steps(
        &mut world,
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Accept") },
            see(Find::exactly_named(Role::Dialog, b"Accept this proposal?")),
            Step::Press {
                find: Find::exactly_named(Role::Button, b"Accept").within(Box::new([Target::ExactRole {
                    role: Role::Dialog,
                    name: b"Accept this proposal?".as_slice().into(),
                }])),
            },
            Step::Gone { find: Find::exactly_named(Role::Dialog, b"Accept this proposal?"), within: fixtures::WITHIN },
            see(Find::exactly_named(Role::Paragraph, ACCEPTED)),
            see(Find::exactly_named(Role::Link, b"Goals")),
        ],
    );
    assert!(world.root(person).person().pages().confirmation().is_none());
    assert_eq!(rail(&world, person), 0);
    newest(&mut world, agent, 2);
    steps(
        &mut world,
        person,
        vec![
            see(Find::exactly_named(Role::TextArea, b"Message")),
            Step::Type {
                find: Find::exactly_named(Role::TextArea, b"Message"),
                words: b"What comes next?".as_slice().into(),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Send") },
            see(Find::text(b"What comes next?")),
        ],
    );
    let chat = world.root(person).person().pages().chat().expect("chat");
    assert!(
        matches!(chat.outcome, Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
            c::Outcome::Document(c::DocumentOutcome::Said { task: said, .. })
        )))) if said == task),
        "the engine admitted the message: {:?}",
        chat.outcome
    );
}

#[test]
fn a_configuring_chat_rejects_its_own_proposal() {
    let (mut world, agent, task, person) = proposed();
    let object = world.root(person).person().pages().chat().expect("chat").cards.get(0).expect("proposal").object;
    for act in [
        Act::CardWords { object, words: REASON.into() },
        Act::Decide { object, revision: None, act: c::ActKind::Reject },
    ] {
        world.event(person, Event::Act { action: Action::Pages(act) }.into());
    }
    steps(&mut world, person, vec![see(Find::exactly_named(Role::Dialog, b"Reject this proposal?"))]);
    world.event(person, Event::Act { action: Action::Pages(Act::Confirm) }.into());
    steps(
        &mut world,
        person,
        vec![Step::Gone {
            find: Find::exactly_named(Role::Dialog, b"Reject this proposal?"),
            within: fixtures::WITHIN,
        }],
    );
    assert!(world.root(person).person().pages().confirmation().is_none());
    assert_eq!(rail(&world, person), 0);
    newest(&mut world, agent, 1);
    assert!(
        world.front().engine.task(task).expect("configuring chat").inbox.iter().any(|word| {
            matches!(
                word.kind,
                jig_domain_tasks::MessageKind::SettingsDecision {
                    outcome: jig_domain_tasks::SettingsOutcome::Rejected,
                    ..
                }
            ) && word.words.as_ref() == REASON
        }),
        "the run hears the rejection and its reason"
    );
}
