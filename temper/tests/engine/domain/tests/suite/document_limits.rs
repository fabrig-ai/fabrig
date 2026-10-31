use jig_domain_core::reads::{Answer, Read};
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{Delivery, Event, PartyInput};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};
const DOCUMENT_READ: Token = Token::new(7);
const ROWS: &[(u32, u32)] = &[(1, 1), (2, 2)];

#[test]
fn configured_document_rows_reach_the_cores_limits() {
    for &(rows, expected) in ROWS {
        let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
        jig.core.documents.rows = rows;
        let own = temper_service::limits::profile(temper_service::limits::Profile::Small);
        let configured = temper_service::limits::engine(&jig, &own).expect("configured root bounds");
        let mut limits = walking::limits();
        limits.documents = configured.documents;
        let mut driver = Driver::configured(Store::new(), [walking::config(91), walking::config(91)], &limits);
        advance(&mut driver);
        driver.sign_in();
        advance(&mut driver);
        for key in [31, 32] {
            driver.send(Event::Party(PartyInput::Ask {
                models: Box::new([]),
                reply_to: ReplyTo::new(Token::new(u64::from(key))),
                sign_in: driver.session(),
                key: [key; 16],
                ask: chat(),
            }));
            advance(&mut driver);
        }
        driver.send(Event::Party(PartyInput::Read {
            reply_to: ReplyTo::new(DOCUMENT_READ),
            sign_in: driver.session(),
            read: Read::Tasks { query: jig_core_world::fixtures::OWN_TASKS, live: None, ended: None },
        }));
        advance(&mut driver);
        let document = driver.delivered.iter().find_map(|delivery| {
            let Delivery::Read { reply, answer } = delivery else { return None };
            (*reply == ReplyTo::new(DOCUMENT_READ)).then_some(answer)
        });
        let Some(Answer::Tasks(page)) = document else { panic!("rows {rows}: conversation page") };
        assert_eq!(page.live.len(), expected, "rows {rows}: core builds the configured number of document rows");
    }
}

fn chat() -> jig_domain_people::Ask {
    let jig_domain_people::Ask::Create { project, mut batch } = walking::chat(1, b"hello") else { unreachable!() };
    batch[0].budget = jig_domain_people::Budget::AsAllowed { cap: Some(200) };
    jig_domain_people::Ask::Create { project, batch }
}

fn advance(driver: &mut Driver) {
    for _ in 0..40 {
        driver.advance(true);
    }
}
