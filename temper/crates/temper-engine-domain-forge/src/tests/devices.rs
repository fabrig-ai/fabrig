//! Device decisions, limits and private documents (attention.md, 7.1).
#![expect(clippy::disallowed_macros, reason = "ordinary tests build bounded device fixtures")]
#![expect(clippy::disallowed_types, reason = "ordinary tests collect the device view")]
#![expect(clippy::disallowed_methods, reason = "ordinary assertions inspect collected device rows")]
use super::*;
use alloc::vec::Vec;
use jig_domain_people::ApplicationScope;
const PARTY: u64 = 7;
const OTHER: u64 = 8;
const REQUEST: Token = Token::new(90);
const ANSWER: Token = Token::new(91);
fn subscription() -> AddDevice {
    AddDevice {
        endpoint: b"http://push.test:4000/push/alice-phone".as_slice().into(),
        p256dh: [4; 65],
        auth: [7; 16],
        label: b"temper ctl".as_slice().into(),
    }
}
fn enabled() -> Domain {
    let mut d = domain();
    d.devices = Some(Box::new(crate::devices::State::new(
        Some(Box::new([b"http://push.test:4000".as_slice().into()])),
        LIMITS.devices(),
    )));
    d
}
fn add(d: &mut Domain, party: u64, call: AddDevice) -> Box<[Request]> {
    outputs(d, Event::AddDevice { request: REQUEST, token: ANSWER, party, person: true, call })
}
fn answer(d: &Domain) -> DeviceAnswer {
    let Some(Answered::Device(answer)) = d.application_answer(ANSWER) else { panic!("device answer") };
    answer
}
fn done() -> Request {
    Request::ApplicationDone { request: REQUEST, token: ANSWER }
}
fn refusal(d: &mut Domain, call: AddDevice, why: DeviceRefusal) {
    assert_eq!(add(d, PARTY, call).as_ref(), [done()]);
    assert_eq!(answer(d), DeviceAnswer::Refused(why));
}
#[test]
fn devices_validate_in_order_replace_in_place_and_transfer_atomically() {
    let mut invalid = subscription();
    invalid.endpoint = Box::new([]);
    invalid.p256dh = [3; 65];
    invalid.label = Box::new([]);
    refusal(&mut domain(), invalid.clone(), DeviceRefusal::Off);
    let mut d = enabled();
    refusal(&mut d, invalid, DeviceRefusal::Service);
    for endpoint in [b"https://elsewhere.test/push".as_slice(), b"http://push.test:4000.evil/push", b"not a URL"] {
        refusal(&mut d, AddDevice { endpoint: endpoint.into(), ..subscription() }, DeviceRefusal::Service);
    }
    refusal(&mut d, AddDevice { p256dh: [3; 65], ..subscription() }, DeviceRefusal::Malformed);
    for label in [vec![], vec![b'l'; 61], vec![255]] {
        refusal(&mut d, AddDevice { label: label.into_boxed_slice(), ..subscription() }, DeviceRefusal::Malformed);
    }
    let outputs = add(&mut d, PARTY, subscription());
    assert_eq!(outputs.len(), 3);
    let Request::Save { record: Stored::Device(first), .. } = &outputs[0] else { panic!("saved device") };
    assert_eq!((first.number, first.party, first.at), (1, PARTY, env().wall));
    assert_eq!(answer(&d), DeviceAnswer::Added { number: 1 });
    assert_eq!(add(&mut d, PARTY, subscription()).as_ref(), [done()], "identical subscription writes nothing");
    let changed = AddDevice { label: b"new label".as_slice().into(), p256dh: [4; 65], auth: [8; 16], ..subscription() };
    let mut environment = env();
    environment.wall = Wall::from_nanos(999);
    let mut out = Queue::uncapped();
    step(
        &mut d,
        &environment,
        Event::AddDevice { request: REQUEST, token: ANSWER, party: PARTY, person: true, call: changed.clone() },
        &mut out,
    );
    let held = d.devices.as_ref().unwrap().rows.get(&(PARTY, 1)).unwrap();
    assert_eq!((held.label.as_ref(), held.auth, held.at), (changed.label.as_ref(), changed.auth, environment.wall));
    let transferred = add(&mut d, OTHER, subscription());
    assert_eq!(transferred[0], Request::Erase { key: Key::Device { party: PARTY, number: 1 } });
    assert_eq!(transferred.len(), 4);
    assert!(d.devices_of(PARTY).next().is_none());
    assert_eq!(d.devices_of(OTHER).next().unwrap().1.number, 1);
    assert_eq!(answer(&d), DeviceAnswer::Added { number: 1 });
    let got = outputs_service(&mut d);
    assert_eq!(got.as_ref(), [done()]);
    assert_eq!(answer(&d), DeviceAnswer::Refused(DeviceRefusal::Person));
}
#[test]
fn pushed_erases_only_the_device_still_held_at_that_endpoint_and_marks_devices() {
    let mut d = enabled();
    add(&mut d, PARTY, subscription());
    let sent_endpoint = subscription().endpoint;
    outputs(&mut d, Event::RemoveDevice { request: REQUEST, token: ANSWER, party: PARTY, number: 1 });
    add(
        &mut d,
        PARTY,
        AddDevice { endpoint: b"http://push.test:4000/replacement".as_slice().into(), ..subscription() },
    );
    let before = d.devices_of(PARTY).next().unwrap().1.clone();
    let mut outcomes = Vec::new();
    for (party, device, endpoint, answer) in [
        (PARTY, 1, before.endpoint.clone(), PushAnswer::Made),
        (PARTY, 1, before.endpoint.clone(), PushAnswer::Failed),
        (OTHER, 1, before.endpoint.clone(), PushAnswer::Gone),
        (PARTY, 2, before.endpoint.clone(), PushAnswer::Gone),
        (PARTY, 1, sent_endpoint, PushAnswer::Gone),
    ] {
        outcomes.extend(outputs(&mut d, Event::Pushed { party, device, endpoint, answer }));
        assert_eq!(d.devices_of(PARTY).next().map(|(_, row)| row), Some(&before));
    }
    assert!(outcomes.is_empty(), "no save, erase or mark for any stale or non-Gone answer");
    assert_eq!(
        outputs(
            &mut d,
            Event::Pushed { party: PARTY, device: 1, endpoint: before.endpoint.clone(), answer: PushAnswer::Gone }
        )
        .as_ref(),
        [
            Request::Erase { key: Key::Device { party: PARTY, number: 1 } },
            Request::Marked { tasks: Box::new([DEVICES_SUBJECT]) },
        ]
    );
    assert!(d.devices_of(PARTY).next().is_none());
    assert!(
        outputs(&mut d, Event::Pushed { party: PARTY, device: 1, endpoint: before.endpoint, answer: PushAnswer::Gone })
            .is_empty(),
        "duplicate Gone is inert"
    );
}

fn outputs_service(d: &mut Domain) -> Box<[Request]> {
    outputs(d, Event::AddDevice { request: REQUEST, token: ANSWER, party: OTHER, person: false, call: subscription() })
}
#[test]
fn devices_are_bounded_private_restored_and_not_a_projects() {
    let mut d = enabled();
    for number in 1..=DEVICES {
        let endpoint = format!("http://push.test:4000/push/{number}").into_bytes().into_boxed_slice();
        add(&mut d, PARTY, AddDevice { endpoint, ..subscription() });
        assert_eq!(answer(&d), DeviceAnswer::Added { number });
    }
    refusal(&mut d, subscription(), DeviceRefusal::TooMany);
    let own = d.document(DEVICES_SUBJECT, ApplicationScope::Own, PARTY, env().wall, &LIMITS).unwrap();
    let Document::Devices(view) = own else { panic!("devices") };
    assert_eq!(view.rows.iter().map(|row| row.number).collect::<Vec<_>>(), (1..=DEVICES).collect::<Vec<_>>());
    assert_eq!(
        d.document(DEVICES_SUBJECT, ApplicationScope::Own, OTHER, env().wall, &LIMITS),
        Some(Document::Devices(DevicesView { rows: Box::new([]) }))
    );
    assert!(d.document(DEVICES_SUBJECT, ApplicationScope::Project(1), PARTY, env().wall, &LIMITS).is_none());
    let rows: Vec<_> = d.devices_of(PARTY).map(|(_, row)| Stored::Device(row.clone())).collect();
    let mut restored = enabled();
    for row in &rows {
        outputs(&mut restored, Event::Restore { record: row.clone() });
    }
    assert_eq!(restored.restore_fit_failure(), None);
    assert_eq!(restored.devices_of(PARTY).collect::<Vec<_>>(), d.devices_of(PARTY).collect::<Vec<_>>());
    let out = outputs(&mut restored, Event::ReleaseProject { project: 1, request: REQUEST });
    assert_eq!(out.as_ref(), [Request::ProjectReleased { request: REQUEST }]);
    assert_eq!(restored.devices_of(PARTY).collect::<Vec<_>>(), d.devices_of(PARTY).collect::<Vec<_>>());
    let removed = outputs(&mut d, Event::RemoveDevice { request: REQUEST, token: ANSWER, party: PARTY, number: 2 });
    assert_eq!(removed[0], Request::Erase { key: Key::Device { party: PARTY, number: 2 } });
    assert_eq!(answer(&d), DeviceAnswer::Removed);
    assert_eq!(
        outputs(&mut d, Event::RemoveDevice { request: REQUEST, token: ANSWER, party: OTHER, number: 1 }).as_ref(),
        [done()]
    );
    assert_eq!(answer(&d), DeviceAnswer::Refused(DeviceRefusal::Unknown));
    add(&mut d, PARTY, subscription());
    assert_eq!(answer(&d), DeviceAnswer::Added { number: 9 });
    let Stored::Device(mut ninth) = rows[0].clone() else { panic!("device") };
    ninth.number = 9;
    outputs(&mut restored, Event::Restore { record: Stored::Device(ninth) });
    assert_eq!(restored.restore_fit_failure().unwrap().family, Some(RecordKind::Device));
    // The family-wide bound is independent of each party's eight.
    let mut small = enabled();
    small.devices.as_mut().unwrap().rows = skein_lib::Map::with_capacity(1);
    add(&mut small, OTHER, subscription());
    let other = AddDevice { endpoint: b"http://push.test:4000/other".as_slice().into(), ..subscription() };
    refusal(&mut small, other, DeviceRefusal::TooMany);
    add(&mut small, PARTY, subscription());
    assert_eq!(answer(&small), DeviceAnswer::Added { number: 1 });
    assert_eq!(small.devices.as_ref().unwrap().rows.len(), 1);
    let mut limits = LIMITS;
    limits.bindings = 1;
    let mut family = domain_with_limits(&limits);
    family.restore_begin(Wall::EPOCH);
    let mut environment = env();
    environment.limits = limits;
    for party in 1..=u64::from(limits.devices()).saturating_add(1) {
        let row = Device {
            party,
            number: 1,
            endpoint: format!("http://push.test/{party}").into_bytes().into_boxed_slice(),
            p256dh: [4; 65],
            auth: [7; 16],
            label: b"phone".as_slice().into(),
            at: Wall::EPOCH,
        };
        let mut out = Queue::uncapped();
        step(&mut family, &environment, Event::Restore { record: Stored::Device(row) }, &mut out);
    }
    assert_eq!(family.devices.as_ref().unwrap().rows.len(), limits.devices());
    assert_eq!(family.restore_fit_failure().unwrap().family, Some(RecordKind::Device));
}
