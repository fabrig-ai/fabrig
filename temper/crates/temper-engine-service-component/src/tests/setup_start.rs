//! Handover reads happen at startup or started, not a new owned ask (protocol/setup/owned.md, 5).
use super::*;

fn service_with_group(fake: bool) -> crate::Service {
    let mut config = runtime_config();
    if fake {
        config.setups = Box::new([(
            2,
            temper_forge_setup::setup_file::chosen(
                temper_forge_setup::machine::BoxKind::Fake,
                9,
                b"owner",
                false,
                &config.directories,
            ).unwrap(),
        )]);
    }
    local_service(config, local_store(false))
}
fn add(service: &mut crate::Service, mode: domain::Mode) {
    let engine = service.engine.as_mut().unwrap();
    let record = runtime_record(domain::Kind::Forgejo, mode);
    insert_runtime_forge(engine, &record);
    keep(engine, &record);
}
fn take_over(service: &mut crate::Service, read: crate::HandoverRead) {
    assert_eq!(read.forge, 2);
    assert!(read.path.ends_with(b"/admin_token"));
    service.handover_read(2, read.run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
    let engine = service.engine.as_mut().unwrap();
    advance(engine);
    let mut takeover = false;
    for (_, run) in &engine.setup.runs {
        takeover |= run.forge == 2 && run.kind == Kind::LocalTakeOver;
    }
    assert!(takeover, "the present handover begins TakeOver");
}

#[test]
fn a_new_owned_group_reads_no_present_handover_until_started() {
    let mut service = service_with_group(false);
    restore_local_with_file(&mut service, false);
    add(&mut service, domain::Mode::Owned);
    let engine = service.engine.as_mut().unwrap();
    let mut out = Queue::with_capacity(3);
    engine.box_link = crate::route::box_link::Link::new(256, true);
    engine.box_link.group(2, domain::BoxKind::Owned, Token::new(456), &mut out);
    crate::route::box_link::drain(engine, &mut out);
    advance(engine);
    assert!(service.take_handover_read().is_none(), "even a present file waits for started");
    let engine = service.engine.as_mut().unwrap();
    for (_, run) in &engine.setup.runs {
        assert_ne!(run.forge, 2, "no early TakeOver");
    }
    for event in &engine.events {
        if let root::Event::Forge(event) = event
            && let domain::Event::Connection { forge: 2, state } = event.as_ref()
        {
            assert_eq!(*state, domain::Connection::SettingUp { step: None }, "the row stays setting up");
        }
    }
    engine.box_link.write = None;
    engine.box_link.read(b"started 2\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    let read = service.take_handover_read().expect("started reads the present handover");
    take_over(&mut service, read);
}

#[test]
fn a_new_fake_group_reads_its_handover_without_started() {
    let mut service = service_with_group(true);
    restore_local_with_file(&mut service, false);
    add(&mut service, domain::Mode::Fake);
    advance(service.engine.as_mut().unwrap());
    let read = service.take_handover_read().expect("the playground reads without a box answer");
    take_over(&mut service, read);
}

#[test]
fn a_restored_owned_group_reads_its_present_handover_at_startup() {
    let mut service = service_with_group(false);
    add(&mut service, domain::Mode::Owned);
    restore_local_with_file(&mut service, false);
    let read = service.take_handover_read().expect("restored owned group reads at startup");
    take_over(&mut service, read);
}
