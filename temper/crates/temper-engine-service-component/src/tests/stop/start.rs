use super::{End, fixtures};

fn unstarted() -> crate::Service {
    let mut settings = fixtures::engine();
    fixtures::fill_files(&mut settings);
    crate::Service::engine(settings, &fixtures::seeds(), None, fixtures::forge_runtime()).expect("unopened engine")
}

fn unmade(engine: &crate::Engine) {
    assert_eq!(engine.accounts.as_ref().expect("accounts").component().startup(0), (false, 0, 0));
    assert_eq!(engine.store.secret_requests, 0, "the secrets gate never opened");
    assert!(engine.store.forge_loads.is_empty(), "no forge records read");
    assert!(engine.forge.starts.is_empty(), "no forge connection begun");
    assert!(engine.workers.forges.is_empty());
    assert!(!engine.setup.startup_restored());
}

fn drain(service: &mut crate::Service) {
    for _ in 0_u32..256 {
        let engine = service.engine.as_ref().expect("engine");
        unmade(engine);
        if engine.store.host.closing() {
            assert!(engine.accounts.as_ref().expect("accounts").idle());
            assert!(engine.setup.closed && engine.setup.io.is_empty());
            assert!(engine.forge.closed && engine.forge.io.is_empty());
            assert!(engine.workers.host.closed());
            assert!(engine.clients.closed && engine.clients.io.is_empty());
        }
        if service.end().is_some() {
            break;
        }
        fixtures::turn(service);
    }
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.drain.as_ref().unwrap().closing, "root must have drained");
    assert!(engine.accounts.as_ref().unwrap().idle(), "accounts");
    assert!(engine.setup.closed, "setup");
    assert!(engine.forge.closed, "forge");
    assert!(engine.workers.host.closed(), "workers");
    assert!(engine.clients.closed, "clients");
    assert!(engine.store.idle(), "store idle");
    assert_eq!(
        service.end(),
        Some(&End::Drained),
        "{:?}",
        service.engine.as_ref().unwrap().pending(skein_lib::Time::from_nanos(1))
    );
    assert!(service.engine.as_ref().expect("engine").store.closed());
}

#[test]
fn a_signal_during_wires_drains_without_opening_mark_gated_components() {
    let previous = fixtures::ready().engine.expect("previous engine");
    let mut service = fixtures::restarting_from(previous);
    service.engine.as_mut().expect("engine").store.hold_mark = true;
    for _ in 0_u32..256 {
        fixtures::turn(&mut service);
        if service.engine.as_ref().unwrap().store.held_mark.is_some() {
            break;
        }
    }
    let engine = service.engine.as_mut().expect("engine");
    assert_eq!(engine.domain.start_phase(), Some(jig_domain_core::start::Phase::Wires));
    let mark = engine.store.held_mark.take().expect("wires in flight");
    service.shutdown();
    assert!(crate::stop::requested(service.engine.as_ref().unwrap()), "the unopened root heard the stop");
    for _ in 0_u32..4 {
        fixtures::turn(&mut service);
    }
    assert_eq!(service.end(), None, "the start's commit is still owed");
    let engine = service.engine.as_mut().unwrap();
    unmade(engine);
    engine.store.hold_mark = false;
    assert!(engine.store.down(mark));
    drain(&mut service);
    assert!(service.engine.as_ref().unwrap().store.host.mark_durable(), "the stopped start's wires became durable");
}

#[test]
fn a_signal_before_the_mark_drains_without_asking_another_page() {
    let mut service = fixtures::started();
    service.engine.as_mut().unwrap().store.hold_load = true;
    for _ in 0_u32..256 {
        fixtures::turn(&mut service);
        if service.engine.as_ref().unwrap().store.held_load.is_some() {
            break;
        }
    }
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.domain.start_phase(), Some(jig_domain_core::start::Phase::LoadCore));
    assert!(engine.store.held_load.is_some());
    service.shutdown();
    drain(&mut service);
    assert_eq!(service.engine.as_ref().unwrap().domain.start_phase(), Some(jig_domain_core::start::Phase::LoadCore));
    assert!(!service.engine.as_ref().unwrap().store.host.mark_durable());
}

#[test]
fn a_signal_while_the_store_opens_closes_it_without_starting_the_root() {
    let mut service = unstarted();
    let (limits, _, store_limits) = fixtures::store_limits();
    let models = jig_protocol_config::core(&fixtures::engine().jig, [7; 16], 7).settings.models;
    let mut store = fixtures::opening_memory_store_for(&limits, &store_limits, &models);
    let opened = store.host.above.pop().expect("held open answer");
    let jig_protocol_store::Event::Opened { .. } = opened else { panic!("open answer") };
    let census = store.host.above.pop().expect("opening census");
    let jig_protocol_store::Event::Census { .. } = census else { panic!("opening census") };
    service.engine.as_mut().unwrap().store = store;
    service.shutdown();
    drain(&mut service);
    assert!(!service.engine.as_ref().unwrap().store.opened, "the hosted store closes midway, without its open answer");
    assert_eq!(service.engine.as_ref().unwrap().domain.start_phase(), None);
}

#[test]
fn a_signal_before_the_root_starts_closes_without_starting_it() {
    let mut service = unstarted();
    service.engine.as_mut().unwrap().store = fixtures::memory_store();
    assert_eq!(service.engine.as_ref().unwrap().domain.start_phase(), None);
    service.shutdown();
    drain(&mut service);
    assert_eq!(service.engine.as_ref().unwrap().domain.start_phase(), None);
    assert!(!service.engine.as_ref().unwrap().store.host.mark_durable());
}

#[test]
fn a_second_signal_before_the_mark_aborts() {
    let mut service = fixtures::started();
    service.shutdown();
    service.shutdown();
    assert_eq!(service.end(), Some(&End::Aborted));
    unmade(service.engine.as_ref().unwrap());
}
