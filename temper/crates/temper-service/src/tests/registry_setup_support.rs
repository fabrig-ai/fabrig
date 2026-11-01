//! Setup table assertions (temper/docs/design/protocol/setup.md, 5.1).
use super::*;
use crate::tests::fixtures;

pub(crate) fn given(service: &crate::Service, forge: u16) -> Option<&host::Kind> {
    service.engine.as_ref().unwrap().setup.served.get(&forge)
}
pub(crate) fn local(service: &crate::Service, forge: u16) -> &lifecycle::Local {
    service.engine.as_ref().unwrap().setup.lifecycle.local(forge).unwrap()
}
pub(crate) fn unserved(service: &crate::Service, forge: u16) -> Option<&'static [u8]> {
    service.engine.as_ref().unwrap().setup.unserved.get(&forge).copied()
}
pub(crate) fn runs(service: &crate::Service) -> u32 {
    service.engine.as_ref().unwrap().setup.runs.len()
}
pub(crate) fn reconcile(service: &crate::Service, forge: u16) {
    let layer = &service.engine.as_ref().unwrap().setup;
    let (_, run) = layer.runs.first().expect("live reconcile");
    assert_eq!(run.forge, forge);
    assert_eq!(run.kind, Kind::LocalReconcile);
}
pub(crate) fn bound(service: &mut crate::Service, address: core::net::SocketAddr) {
    service.engine.as_mut().unwrap().setup.webhooks_listening(address);
}
pub(crate) fn pending(service: &crate::Service) -> bool {
    service.engine.as_ref().unwrap().setup.pending.is_some()
}
pub(crate) fn seed_tables(service: &mut crate::Service, forge: u16) {
    let layer = &mut service.engine.as_mut().unwrap().setup;
    for number in [forge, 2] {
        let token = Token::new(u64::from(number).checked_add(100).unwrap());
        layer.staged.insert(token, host::Plan::Install { forge: number }).unwrap();
        layer.public.insert(token, public::Caller::Install { forge: number, request: token, back: None }).unwrap();
        layer.file_api.insert(number, Secret::new(b"fixture-api".as_slice().into())).unwrap();
        layer.pastes.insert(u64::from(number), number).unwrap();
        layer.erases.insert(token, number).unwrap();
        layer.backs.push(public::ReturnTo {
            forge: number,
            state: b"state".as_slice().into(),
            back: None,
            until: Time::from_nanos(100),
        });
        layer.hold(number, b"state".as_slice().into(), b"page".as_slice().into());
        layer.held.push(Held { run: token, forge: number, until: Time::from_nanos(100) });
        layer.waiting.push(Waiting {
            forge: number,
            call: token,
            installation: core::num::NonZeroU64::new(1).unwrap(),
            state: b"state".as_slice().into(),
            back: None,
            until: Time::from_nanos(100),
        });
        layer.lifecycle.reconcile.insert(number, lifecycle::Reconcile::GitHub).unwrap();
    }
}
pub(crate) fn removed(service: &crate::Service, forge: u16) {
    let layer = &service.engine.as_ref().unwrap().setup;
    assert!(!layer.served.contains_key(&forge));
    assert!(!layer.unserved.contains_key(&forge));
    assert!(!layer.file_api.contains_key(&forge));
    assert!(!layer.lifecycle.reconcile.contains_key(&forge));
    assert!(layer.lifecycle.reconcile.contains_key(&2));
    assert!(layer.file_api.contains_key(&2));
    for (_, run) in &layer.runs {
        assert_ne!(run.forge, forge);
    }
    for (_, plan) in &layer.staged {
        assert_ne!(super::forge(plan), forge);
    }
    for (_, caller) in &layer.public {
        assert_ne!(caller.forge(), forge);
    }
    for (_, target) in &layer.pastes {
        assert_ne!(*target, forge);
    }
    for (_, target) in &layer.erases {
        assert_ne!(*target, forge);
    }
    for back in &layer.backs {
        assert_ne!(back.forge, forge);
    }
    for page in &layer.pages {
        assert_ne!(page.forge, forge);
    }
    for held in &layer.held {
        assert_ne!(held.forge, forge);
    }
    for waiting in &layer.waiting {
        assert_ne!(waiting.forge, forge);
    }
    assert_eq!(layer.staged.len(), 1);
    assert_eq!(layer.public.len(), 1);
    assert_eq!(layer.pastes.len(), 1);
    assert_eq!(layer.erases.len(), 1);
    assert_eq!(layer.backs.len(), 1);
    assert_eq!(layer.pages.len(), 1);
    assert_eq!(layer.held.len(), 1);
    assert_eq!(layer.waiting.len(), 1);
}
pub(crate) fn late_write(service: &mut crate::Service) {
    let engine = service.engine.as_mut().unwrap();
    let Some(StoreEvent::Kept { owner }) = engine.store.above.pop() else { panic!("pending host write") };
    answer(engine, StoreEvent::Kept { owner });
}
pub(crate) fn handover(service: &mut crate::Service) -> crate::HandoverRead {
    tests::box_started(service)
}
pub(crate) fn setup_service(config: crate::settings::Engine, store: crate::store::Store) -> crate::Service {
    let io = config.jig.limits.io;
    let forges = crate::forge::prepare(&config, skein_lib::List::with_capacity(0)).unwrap();
    let mut service = crate::Service::prepared(config, &fixtures::seeds(), None, forges, None, io, Some(128)).unwrap();
    service.engine.as_mut().unwrap().store = store;
    service
}

pub(crate) fn host_capacity(service: &mut crate::Service, forges: u32) {
    service.engine.as_mut().unwrap().setup.env.limits.forges = forges;
}
