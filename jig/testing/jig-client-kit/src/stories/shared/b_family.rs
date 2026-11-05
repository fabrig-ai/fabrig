//! A person changes a connector's object (jig/docs/design/client/domain/testing.md, 4–5).
use super::{
    b_conversations::{WAIT, finish, sign_in},
    b_seed::Facts,
};
use crate::{Front, Scripted, Test, World, harness::Seed, joint::engine::Driver};
use jig_client_test_domain::{connector_values as c, family as f};
use skein_fake_person::{Find, Step};
use skein_lib::{Time, Wall};
use skein_ui::Role;

pub fn scripted(front: &mut Scripted) {
    let subject = f::Subject { scope: f::Scope::Project(1), kind: f::Kind::Item(1) };
    let mut family = front.engine.application().clone();
    family.documents.insert(
        subject,
        f::Document::Connector(c::ConnectorDocument {
            subject,
            value: Box::new(c::ClientDocument::Object {
                object: 1,
                now: c::ApplicationObject { revision: 0, value: 0 },
            }),
        }),
    );
    front
        .engine
        .act_application(Time::ZERO, Wall::EPOCH, jig_client_test_domain::FIRST, 1, family, vec![])
        .expect("family fact fits");
}
pub fn core(driver: &mut Driver, now: Time, wall: Wall) {
    let configuration = jig_test_profile::config(8510);
    let resource = configuration.first.resources[0].path.clone();
    for event in [
        jig_test_connector::Event::Adopt {
            project: 1,
            resource: resource.clone(),
            role: jig_test_connector::ResourceRole::Owned,
        },
        jig_test_connector::Event::System(jig_test_connector::SystemEvent::Fact {
            resource,
            fact: jig_test_connector::Fact { state: Some(0), observed: wall, pending: false },
            origin: jig_test_connector::Origin::Other,
        }),
    ] {
        driver.send(now, wall, jig_test_domain::Event::Connector { number: 1, event });
    }
}
pub fn a_person_s_part_request_is_answered_from_the_connector_s_outcome<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Project(Facts::FamilyResource));
    let client = world.client(1, crate::stories::conversation());
    world.person(
        client,
        sign_in(vec![
            Step::See { find: Find::text(b"Signed in as"), within: WAIT },
            Step::Go {
                address: jig_client_test_domain::Address::First(jig_client_test_domain::first::Address {
                    project: 1,
                    item: 1,
                }),
            },
            Step::See { find: Find::named(Role::TextField, b"Parameters"), within: WAIT },
            Step::Type { find: Find::named(Role::TextField, b"Parameters"), words: b"42".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Apply parameters") },
            Step::See { find: Find::text(b"Object changed"), within: WAIT },
            Step::See { find: Find::text(b"Value: 42"), within: WAIT },
        ]),
    );
    finish(world);
}
