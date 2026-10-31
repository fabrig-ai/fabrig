//! Temper's routing at the clients' component boundary (jig/docs/design/client/domain/testing.md, 5).
use temper_client_domain::Event;

/// Route the component's core request through Temper's party boundary.
#[must_use]
pub fn party(event: jig_domain_core::Event) -> temper_engine_domain::Event {
    temper_engine_domain::Event::Party(
        temper_service::clients::translation::to_root(event).expect("client component party input"),
    )
}

/// The kit's shared packet carrier has a test application owner; only jig belongs here.
#[must_use]
pub fn callback(event: jig_client_test_domain::Event) -> Event {
    match event {
        jig_client_test_domain::Event::Client { event } => Event::Client { event },
        jig_client_test_domain::Event::Jig(_) => unreachable!("joint transport delivers client events"),
        jig_client_test_domain::Event::SignedOut => {
            Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } }
        }
        jig_client_test_domain::Event::Start { .. }
        | jig_client_test_domain::Event::Went { .. }
        | jig_client_test_domain::Event::Act { .. }
        | jig_client_test_domain::Event::First(_) => panic!("no test application event belongs to temper"),
    }
}
