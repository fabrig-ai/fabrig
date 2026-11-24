//! Scalar meanings at generated task translation boundaries (skein/docs/design/data/codec/maps.md, 5).
use crate::{generated::tasks as mapped, tasks_v13 as wire};
use jig_domain_tasks as domain;
use skein_lib::Wall;

#[test]
fn newest_fields_store_values_not_ordering_complements() {
    let value = domain::Update {
        party: 2,
        at: Wall::from_nanos(3),
        task: 5,
        object: domain::UpdateObject::Reply { message: domain::Newest::of(7), activity: domain::Newest::of(11) },
    };
    let expected = wire::Update {
        party: 2,
        at: 3,
        task: 5,
        object: wire::UpdateObject::Reply(wire::ReplyObject { message: 7, activity: 11 }),
    };
    assert_eq!(mapped::update_to_record(&value), expected.clone());
    assert_eq!(mapped::update_to_domain(expected), value);
}

#[test]
fn a_generated_session_refuses_unknown_reason_bits_at_the_field() {
    let valid = wire::Session { first: 3, reasons: domain::Reasons::ALL.0, last: u64::MAX };
    let value = domain::Session { first: 3, reasons: domain::Reasons::ALL, last: Wall::from_nanos(u64::MAX) };
    assert_eq!(mapped::session_to_record(&value), valid.clone());
    assert_eq!(mapped::session_to_domain(valid), Ok(value));
    let unknown = wire::Session { first: 3, reasons: 1 << 14, last: 5 };
    assert_eq!(
        mapped::session_to_domain(unknown),
        Err(wire::paths::SESSION_REASONS.problem(skein_codec::Reason::Bound)),
    );
}
