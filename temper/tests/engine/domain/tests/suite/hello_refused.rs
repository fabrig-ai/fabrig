//! Hello refusals retain the live settings' shortfall (protocol/cluster.md, 3).
use jig_domain_core::settings as s;
use jig_domain_fleet::{AgentShortfall, Figures, HelloRefusal, HelloRefused};
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{Delivery, Event, PartyInput};
use temper_engine_domain_world::{direct::Driver, protocol, store::Store, walking};

#[test]
fn a_charter_set_after_start_refuses_with_its_new_figures() {
    let mut driver = Driver::new(Store::new());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let mut charter = walking::charter();
    charter.policy.turns = 9;
    driver.send(Event::Party(PartyInput::ChangeSettings {
        kinds: Box::default(),
        reply_to: ReplyTo::new(Token::new(20)),
        sign_in: driver.session(),
        key: [20; 16],
        digest: [20; 32],
        change: Box::new(s::Change {
            against: 1,
            items: Box::new([s::Item {
                setting: s::Setting::Charter(1),
                edit: s::Edit::Set(s::Value::Charter(Box::new(charter))),
            }]),
            running: s::Running::Stays,
        }),
    }));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::WebReply {
                reply: jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::SettingsChanged { revision: 2 }),
                ..
            }
        )),
        "the new charter was committed after startup"
    );
    driver.send(protocol::hello(Token::new(7), Box::new([])));
    driver.settle();
    let refusal = driver.delivered.iter().find_map(|delivery| match delivery {
        Delivery::Refuse { channel, hello } if *channel == Token::new(7) => Some(*hello),
        _ => None,
    });
    assert_eq!(
        refusal,
        Some(Some(HelloRefused {
            why: HelloRefusal::Agent { why: AgentShortfall::Turns { charter: 1 } },
            figures: Some(Figures { required: 9, offered: 8 }),
        }))
    );
    assert!(!driver.stopped);
}
