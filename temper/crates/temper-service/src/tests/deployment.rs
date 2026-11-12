use super::fixtures;
use crate::family;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use temper_client_protocol_wire::{self as wire, v1 as w};
const REPLY: skein_lib::ReplyTo = skein_lib::ReplyTo::new(skein_lib::Token::new(19));



#[test]
fn the_deployment_command_is_published_only_by_a_standalone_start() {
    for (standalone, under_box) in [(true, false), (true, true), (false, false)] {
        let document = if standalone { fixtures::document() } else { fixtures::engine_alone_document() };
        let line = b"'/box/temper' program-login --config '/box/config' --data '/box/data' --cache '/box/cache'";
        let agent = fixtures::startup_agent();
        let service = crate::startup::with_setup_given(
            &document,
            &jig_protocol_config::Directories::working(),
            &fixtures::files(),
            if standalone { Some(&agent) } else { None },
            &fixtures::seeds(),
            None,
            if standalone { Some(fixtures::worker_runtime()) } else { None },
            fixtures::forge_runtime(),
            None,
            if under_box { Some(fixtures::system::BOX_BYTES) } else { None },
            Some(line.as_slice().into()),
            &crate::stock::Given::Stock,
            Box::new([]),
        )
        .unwrap();
        let deployment = service.engine.as_ref().unwrap().deployment();
        let mut input = read_input(people::ApplicationScope::Deployment);
        if let kit::ToApplication::Read { record, .. } = &mut input {
            *record =
                wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Deployment }).unwrap());
        }
        let Err(answer) = family::to_root(
            &jig_protocol_config::Directories::working(),
            input,
            &fixtures::forge_table(&[]),
            None,
            deployment,
            None,
            &w::CEILINGS,
        ) else {
            panic!("deployment read")
        };
        let kit::Event::Application(kit::FromApplication::Read { record, .. }) = *answer else {
            panic!("deployment answer")
        };
        let answer = wire::decode_deployment_answer(&record.unwrap(), &w::CEILINGS).unwrap();
        assert_eq!(answer.program_login().as_deref(), if standalone { Some(line.as_slice()) } else { None });
    }
}

fn read_input(scope: people::ApplicationScope) -> kit::ToApplication {
    kit::ToApplication::Read {
        reply: REPLY,
        sign_in: 3,
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        record: wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Playground }).unwrap()),
        scope,
        window: jig_domain_core::documents::Window { rows: 2, after: None },
    }
}