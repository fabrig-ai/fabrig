use jig_client_world::{harness::App, observe};
use skein_lib::Token;
use temper_client_world::app::Temper;
const TASK: u64 = 9;
const PROJECT: u32 = 1;
const REVISION: jig_client::contract::Revision = jig_client::contract::Revision { value: 7 };
const ATTEMPT: Token = Token::new(1);
const REQUEST: [u8; 16] = [3; 16];
#[test]
fn a_release_is_observed_against_the_inbox_escalation_the_person_saw() {
    let request = temper_client_domain::Request::Client {
        request: jig_client::Request::Down {
            down: jig_client::Down::Send {
                attempt: ATTEMPT,
                key: REQUEST,
                request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Release {
                    project: PROJECT,
                    task: TASK,
                    revision: REVISION,
                    again: None,
                })),
                value: None,
            },
        },
    };
    let sent = Temper::sent(&request).expect("keyed request");
    assert_eq!(sent.card, Some(observe::key(jig_client::contract::ObjectKey::Escalation(TASK))));
    assert_eq!(sent.revision, Some((2, REVISION.value)));
}

#[test]
fn confirming_preserves_the_inbox_cards_visible_revision() {
    let event = temper_client_world::app::Event::Root(temper_client_domain::Event::Act {
        action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Confirm),
    });
    assert!(!Temper::window_changed(&event));
}

#[test]
fn a_temper_hold_card_reports_the_revision_it_prints() {
    use jig_client_world::harness::{Seen, Until};
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    use temper_client_world::fixtures;
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, held) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world
        .person(person, vec![Step::See { find: Find::named(Role::Button, b"Release task"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    let visible = Temper::seen(world.tree(person));
    assert!(visible.iter().any(|value| matches!(value,
        Seen::Card { key, revision: Some(_), .. } if *key == observe::key(jig_client::contract::ObjectKey::Escalation(held))
    )));
}
