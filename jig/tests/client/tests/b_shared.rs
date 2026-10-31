#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;
// Person stories on the scripted and joint boundaries (client/domain/testing.md, 4–5).
use jig_client_world::{Settings, World, joint::Joint, stories};
fn scripted(
    _facts: stories::shared::b_seed::Facts,
) -> jig_client_world::World<jig_client_world::Test, jig_client_world::Scripted> {
    let mut fixture = stories::world(8510);
    let engine = &fixture.front().engine;
    let mut bounds = engine.bounds();
    bounds.rows = jig_client_world::test::limits().client.domain.rows;
    let script = jig_client_scripted_engine::Script {
        versions: engine.versions(),
        bounds,
        truth: engine.truth().clone(),
        rules: vec![],
    };
    World::new(
        Settings { seed: 8510, ..Settings::default() },
        jig_client_world::Scripted::new(script, 8510, jig_client_world::scripted::Family::default()),
    )
}
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
        smith: smith_transcript::v6::CEILINGS,
        bytes: 1_048_576,
        turn: jig_client_domain::contract::TurnLimits { messages: 2, blocks: 2, text: 32 },
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
    front.version_fault(jig_client_domain::contract::VersionRange { first: 8, last: 8 });
    World::new(Settings { seed: 8510, ..Settings::default() }, front)
}
mod scripted {
    jig_client_world::person_story_suite!(super::scripted;
        release: [ignore = "scripted stopped holds release without the Released notice emitted for placed holds"],
        compound_release: [],
        inbox_chat: [ignore = "scripted proposal decisions do not append the decision to their chat activity"],
        close: [], family: []);
}
mod joint {
    jig_client_world::person_story_suite!(super::joint;
        release: [],
        compound_release: [ignore = "compound release confirmation omits the release effect facts required by jig's check"],
        inbox_chat: [],
        close: [ignore = "close confirmation omits delegate cancellation facts required by jig's check"],
        family: [ignore = "joint family observer reports a different revision from the connector object shown"]);
}
