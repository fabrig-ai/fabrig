//! Charter document bounds at start, change and restart (domain/engine.md, 17.9).
use crate::settings::{Changed, Current, Edit, Failure, Fault, Item, Record, Setting, Value};
use crate::tests::*;

fn bytes(len: u32) -> Box<[u8]> {
    let mut bytes = skein_lib::bytes::zeroed(skein_lib::length::usize_of(len));
    bytes.fill(b'n');
    bytes
}

fn name(segments: u32, segment_bytes: u32) -> authority::Name {
    let mut name = List::with_capacity(segments);
    for _ in 0..segments {
        name.push(bytes(segment_bytes)).unwrap();
    }
    authority::Name { segments: name.into_boxed() }
}

fn bounded_document(limits: &Limits) -> crate::Document {
    crate::Document {
        connector: 1,
        name: name(limits.authority.segments, limits.authority.segment_bytes),
        path: bytes(DOCUMENT_PATH_BYTES),
        heading: Some(bytes(DOCUMENT_HEADING_BYTES)),
    }
}

fn charter_with_documents(limits: &Limits, count: u32) -> Charter {
    let mut charter = config(1).defaults.charters[0].clone();
    let mut documents = List::with_capacity(count);
    for _ in 0..count {
        documents.push(bounded_document(limits)).unwrap();
    }
    charter.documents = documents.into_boxed();
    charter
}

fn bounded_charter(limits: &Limits) -> Charter {
    charter_with_documents(limits, limits.tasks.charter_documents)
}

fn check_at_start_and_change(charter: Charter, expected: Result<(), Failure>) {
    let limits = limits();
    let mut configured = config(1);
    let current = Current::new(configured.defaults, &limits, &[]);
    let items = [Item { setting: Setting::Charter(1), edit: Edit::Set(Value::Charter(Box::new(charter.clone()))) }];
    assert_eq!(
        crate::settings::check_change(&current, &items, &configured.settings, &configured.connectors, &limits),
        expected
    );
    configured.defaults = config(1).defaults;
    configured.defaults.charters[0] = charter;
    let mut current = Current::new(configured.defaults, &limits, &[]);
    assert_eq!(crate::settings::check(&current, &configured.settings, &configured.connectors, &limits), expected);
    // The current overlay is checked separately from the binary's defaults.
    current.defaults.get_mut(&Setting::Charter(1)).unwrap().value =
        Value::Charter(Box::new(config(1).defaults.charters[0].clone()));
    let Edit::Set(value) = &items[0].edit else { panic!("set fixture") };
    current.changes.insert(Setting::Charter(1), Changed { value: Some(value.clone()), revision: 2 }).unwrap();
    assert_eq!(crate::settings::check(&current, &configured.settings, &configured.connectors, &limits), expected);
}

#[test]
fn documents_at_their_bounds_are_accepted_at_start_and_change() {
    check_at_start_and_change(bounded_charter(&limits()), Ok(()));
}

#[test]
fn documents_past_each_bound_are_shape_failures_naming_the_charter() {
    let limits = limits();
    let bounded = bounded_charter(&limits);
    let mut cases = List::with_capacity(7);
    cases.push(charter_with_documents(&limits, limits.tasks.charter_documents.checked_add(1).unwrap())).unwrap();
    for name in [
        name(limits.authority.segments.checked_add(1).unwrap(), 1),
        name(1, limits.authority.segment_bytes.checked_add(1).unwrap()),
    ] {
        let mut charter = bounded.clone();
        charter.documents[0].name = name;
        cases.push(charter).unwrap();
    }
    for len in [0, DOCUMENT_PATH_BYTES.checked_add(1).unwrap()] {
        let mut charter = bounded.clone();
        charter.documents[0].path = bytes(len);
        cases.push(charter).unwrap();
    }
    for len in [0, DOCUMENT_HEADING_BYTES.checked_add(1).unwrap()] {
        let mut charter = bounded.clone();
        charter.documents[0].heading = Some(bytes(len));
        cases.push(charter).unwrap();
    }
    assert_eq!(cases.len(), 7);
    for charter in cases.into_boxed() {
        let failure = Failure { setting: Setting::Charter(1), fault: Fault::Shape };
        assert_eq!(
            crate::settings::check::restore_fit(
                Setting::Charter(1),
                &Value::Charter(Box::new(charter.clone())),
                &limits
            ),
            Err(failure)
        );
        check_at_start_and_change(charter, Err(failure));
    }
}

#[test]
fn documents_require_a_registered_connector() {
    let mut charter = bounded_charter(&limits());
    charter.documents[0].connector = u16::MAX;
    check_at_start_and_change(charter, Err(Failure { setting: Setting::Charter(1), fault: Fault::Shape }));
}

#[test]
fn stored_documents_past_a_bound_are_refused_at_restart() {
    let limits = limits();
    let mut core = Core::new(config(1), &limits).unwrap();
    let _begin = core.start_begin(false, Wall::EPOCH);
    let mut charter = bounded_charter(&limits);
    charter.documents[0].path = bytes(DOCUMENT_PATH_BYTES.checked_add(1).unwrap());
    assert!(!core.restore_setting(
        Record::Changed {
            setting: Setting::Charter(1),
            changed: Changed { value: Some(Value::Charter(Box::new(charter))), revision: 1 },
        },
        &limits
    ));
    assert_eq!(
        core.start_failure(),
        Some(crate::start::Failure::Settings {
            failure: Failure { setting: Setting::Charter(1), fault: Fault::Shape },
            changed: true,
        })
    );
}

#[test]
fn documents_accept_one_byte_paths_and_optional_one_byte_headings() {
    for heading in [None, Some(Box::from(b"h".as_slice()))] {
        let mut charter = bounded_charter(&limits());
        charter.documents[0].path = b"p".as_slice().into();
        charter.documents[0].heading = heading;
        check_at_start_and_change(charter, Ok(()));
    }
}

#[test]
fn documents_at_their_bounds_are_priced_within_each_charters_setting() {
    let mut limits = limits();
    let charter = bounded_charter(&limits);
    let documents =
        charter_owned_bytes(&charter).unwrap().checked_sub(policy_owned_bytes(&charter.policy).unwrap()).unwrap();
    let with_documents = crate::settings::set_bound(&limits).unwrap();
    limits.tasks.charter_documents = 0;
    assert_eq!(
        with_documents.checked_sub(crate::settings::set_bound(&limits).unwrap()).unwrap(),
        u64::from(limits.tasks.charters).checked_mul(documents).unwrap()
    );
}
