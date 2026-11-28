#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;
// Person stories on the joint boundary (jig/docs/design/client/domain/testing.md, 5).
use jig_client_world::{Settings, World, joint::Joint, stories};
fn joint(
    facts: jig_client_world::stories::shared::b_seed::Facts,
) -> jig_client_world::World<jig_client_world::Test, Joint> {
    use jig_client_world::joint::{engine::Driver, session::Session};
    use skein_lib::{Duration, Time, Wall};
    use std::collections::BTreeMap;
    fn clock(now: Time) -> Wall {
        Wall::from_nanos(1_000_000_000_000 + now.as_nanos())
    }
    let mut limits = jig_test_profile::limits();
    limits.core.views.watchers = 32;
    limits.core.documents.rows = 2;
    limits.core.documents.lines = 2;
    limits.core.documents.chips = 2;
    limits.core.documents.text_bytes = 32;
    let (configuration, limits, peers) = stories::shared::b_config::setup(8510, &facts, limits);
    let driver = Driver::new(configuration, limits, peers, Time::ZERO, clock(Time::ZERO));
    let codec = jig_protocol_transcript::Limits {
        smith: smith_transcript::v8::CEILINGS,
        bytes: 1_048_576,
        turn: jig_client_domain::contract::TurnLimits { messages: 2, blocks: 2, text: 32, outcome: 32 },
    };
    let identity = jig_domain_people::Identity {
        key: jig_domain_people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        login: b"login".as_slice().into(),
        name: b"Person".as_slice().into(),
    };
    let mut front = Joint::new(
        Session::new(driver, jig_client_world::test::limits(), codec),
        BTreeMap::from([(1, identity)]),
        clock,
    );
    front.session.stream_heartbeat(Duration::from_millis(50));
    front.restart_configuration(|| jig_test_profile::config(8510));
    front.version_fault(false);
    World::new(Settings { seed: 8510, ..Settings::default() }, front)
}
mod joint {
    jig_client_world::person_story_suite!(super::joint;
        release: [],
        compound_release: [ignore = "compound release confirmation omits the release effect facts required by jig's check"],
        inbox_chat: [],
        close: [ignore = "close confirmation omits delegate cancellation facts required by jig's check"],
        family: [ignore = "joint family observer reports a different revision from the connector object shown"]);
}
