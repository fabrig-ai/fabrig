use super::fixtures;
use crate::family;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use skein_lib::{ReplyTo, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain_forge as forge;
const REPLY: ReplyTo = ReplyTo::new(Token::new(19));
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

fn playground_read(row: Option<&forge::Playground>, demo: Option<&crate::settings::Demo>) -> Option<Box<[u8]>> {
    let mut input = read_input(people::ApplicationScope::Deployment);
    if let kit::ToApplication::Read { record, .. } = &mut input {
        *record = wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Playground }).unwrap());
    }
    let Err(answer) = family::to_root(
        &jig_protocol_config::Directories::working(),
        input,
        &fixtures::forge_table(&[]),
        demo,
        &family::Deployment { welcome: false, owned: false, program_login: None, push: None },
        row,
        &w::CEILINGS,
    ) else {
        panic!("playground read")
    };
    let kit::Event::Application(kit::FromApplication::Read { reply, family, record }) = *answer else {
        panic!("playground answer")
    };
    assert_eq!((reply, family), (REPLY, wire::FORGE));
    record
}

#[test]
fn the_playground_read_answers_from_the_row_and_the_settings() {
    let mut service = fixtures::demo_service();
    let demo = service.engine.as_mut().unwrap().demo.as_mut().unwrap();
    demo.owner = b"playground".as_slice().into();
    for (state, project, expected) in [
        (forge::PlaygroundState::Starting, None, w::PlaygroundState::Starting),
        (forge::PlaygroundState::Running, Some(13), w::PlaygroundState::Running),
        (forge::PlaygroundState::Resetting(forge::ResetStep::Project), Some(13), w::PlaygroundState::Resetting),
        (forge::PlaygroundState::Resetting(forge::ResetStep::Forge), Some(13), w::PlaygroundState::Resetting),
        (forge::PlaygroundState::Resetting(forge::ResetStep::Account), Some(13), w::PlaygroundState::Resetting),
    ] {
        let row = forge::Playground { forge: 9, project, account: Some(7), state };
        let record = playground_read(Some(&row), Some(demo)).unwrap();
        let answer = wire::decode_playground_answer(&record, &w::CEILINGS).unwrap();
        assert_eq!((answer.forge(), *answer.project(), answer.provider()), (9, project, demo.provider));
        assert_eq!(answer.state(), &expected);
        assert_eq!(answer.owner(), b"playground");
        assert_eq!(answer.name(), b"widget");
        assert_eq!(answer.cues().as_slice(), demo.cues.as_ref());
    }
}

#[test]
fn no_row_is_the_empty_answer() {
    let service = fixtures::demo_service();
    assert!(playground_read(None, service.engine.as_ref().unwrap().demo.as_ref()).is_none());
    assert!(playground_read(None, None).is_none());
}

#[test]
fn the_deployment_read_stays_answered_before_any_party() {
    let service = fixtures::demo_service();
    let demo = service.engine.as_ref().unwrap().demo.as_ref().unwrap();
    let row =
        forge::Playground { state: forge::PlaygroundState::Running, forge: 1, project: Some(1), account: Some(1) };
    for deployment in [
        &family::Deployment { welcome: false, owned: false, program_login: None, push: None },
        &family::Deployment { welcome: false, owned: true, program_login: None, push: None },
        &family::Deployment { welcome: true, owned: false, program_login: None, push: None },
        &family::Deployment { welcome: true, owned: true, program_login: None, push: Some([4; 65]) },
    ] {
        for (demo, playground) in [(None, None), (Some(demo), None), (None, Some(&row)), (Some(demo), Some(&row))] {
            let mut input = read_input(people::ApplicationScope::Deployment);
            if let kit::ToApplication::Read { record, sign_in, .. } = &mut input {
                *sign_in = 0;
                *record = wire::reading(
                    &w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Deployment }).unwrap(),
                );
            }
            let Err(answer) = family::to_root(
                &jig_protocol_config::Directories::working(),
                input,
                &fixtures::forge_table(&[]),
                demo,
                deployment,
                playground,
                &w::CEILINGS,
            ) else {
                panic!("deployment read answers directly")
            };
            let kit::Event::Application(kit::FromApplication::Read { reply, family, record }) = *answer else {
                panic!("deployment answer")
            };
            assert_eq!((reply, family), (REPLY, wire::FORGE));
            let answer = wire::decode_deployment_answer(&record.expect("every deployment answers"), &w::CEILINGS)
                .expect("deployment record");
            assert_eq!((answer.welcome(), answer.owned()), (deployment.welcome, deployment.owned));
            assert_eq!(*answer.push(), deployment.push);
        }
    }
}

#[test]
fn deployment_owned_needs_the_document_and_the_box() {
    for (document_owned, under_box, expected) in
        [(false, false, false), (false, true, false), (true, false, false), (true, true, true)]
    {
        assert_eq!(crate::system::deployment_owned(document_owned, under_box), expected);
    }
}

#[test]
fn playground_config_needs_the_welcome() {
    let service = fixtures::demo_service();
    let demo = service.engine.as_ref().unwrap().demo.as_ref().unwrap();
    let deadline = skein_lib::Duration::from_secs(10);
    for (demo, welcome, offered) in
        [(None, false, false), (None, true, false), (Some(demo), false, false), (Some(demo), true, true)]
    {
        let config = crate::system::playground_config(demo, welcome, deadline);
        assert_eq!(config.is_some(), offered);
        if let Some(config) = config {
            assert_eq!(config.provider, demo.unwrap().provider);
            assert_eq!(config.deadline, deadline);
        }
    }
}

