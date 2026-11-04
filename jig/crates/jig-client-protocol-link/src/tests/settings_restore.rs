//! Restore facts survive the wire and the kept request table (engine.md, 17.10).
use super::*;
use crate::translate::{answers, asks, settings as t, settings_proposals};
use jig_client_test_records::settings as values;
use skein_lib::{List, Writer};

fn limits() -> Limits {
    Limits::new(jig_client_domain::Limits { rows: 16, ..*fixtures::limits().client() }, 4).unwrap()
}
fn policy() -> c::SettingSet {
    c::SettingSet { value: values::settings_values().get(2).unwrap().clone() }
}
#[test]
fn a_project_policy_setting_round_trips_in_items_reviews_and_refusals() {
    let limits = limits();
    let setting = c::Setting::ProjectPolicy { project: 7 };
    assert_eq!(t::restore_setting(&t::wire_setting(&setting, limits.wire()).unwrap()).unwrap(), setting);
    let item = c::SettingItem { setting, edit: c::SettingEdit::Set(policy()) };
    assert_eq!(t::restore_setting_item(&t::wire_setting_item(&item, limits.wire()).unwrap(), &limits).unwrap(), item);
    let mut would_be = policy();
    let c::SettingValue::Policy(value) = &mut would_be.value else { panic!("policy") };
    value.period_spend += 1;
    let review = c::ReviewItem {
        setting,
        edit: c::RevisionItemEdit::Set,
        now: c::ReviewValue::Value(policy()),
        origin: c::SettingOrigin::NotOffered,
        would_be: c::ReviewValue::Value(would_be),
        application: None,
    };
    assert_eq!(
        settings_proposals::restore_review_item(
            &settings_proposals::wire_review_item(&review, limits.wire()).unwrap(),
            &limits
        )
        .unwrap(),
        review
    );
    let refusal = w::Refusal::Malformed(
        w::RefusalMalformed::new(
            limits.wire(),
            w::RefusalMalformedParts {
                at: 2,
                why: w::Malformed::Settings(
                    w::MalformedSettings::new(
                        limits.wire(),
                        w::MalformedSettingsParts {
                            setting: t::wire_setting(&setting, limits.wire()).unwrap(),
                            fault: w::SettingFault::Shape,
                        },
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap(),
    );
    assert_eq!(
        answers::refusal(&refusal, &limits).unwrap(),
        c::Refusal::Malformed {
            at: 2,
            why: c::Malformed::Settings(c::MalformedSettings { setting, fault: c::SettingFault::Shape })
        }
    );
}
#[test]
fn revision_items_round_trip_every_replaced_arm() {
    let limits = limits();
    for replaced in [
        c::RevisionReplaced::Value(policy()),
        c::RevisionReplaced::TooLarge(c::Size { bytes: 999 }),
        c::RevisionReplaced::Default,
        c::RevisionReplaced::Removed,
        c::RevisionReplaced::NotOffered,
        c::RevisionReplaced::Application { size: 91 },
    ] {
        let item = c::RevisionItem {
            setting: c::Setting::ProjectPolicy { project: 7 },
            edit: c::RevisionEdit::Set(policy()),
            replaced,
        };
        let wire = t::wire_revision_item(&item, limits.wire()).unwrap();
        let mut bytes = Writer::new(usize::try_from(wire.measure()).unwrap());
        wire.encode(&mut bytes).unwrap();
        let wire = w::RevisionItem::decode(limits.wire(), &mut Reader::new(&bytes.finish())).unwrap();
        assert_eq!(t::restore_revision_item(&wire, &limits).unwrap(), item);
    }
    let revision = values::settings_revision();
    assert_eq!(
        t::restore_revision_document(&t::wire_revision_document(&revision, limits.wire()).unwrap(), &limits).unwrap(),
        revision
    );
}
#[test]
fn a_restore_ask_round_trips_through_the_wire_and_the_kept_table() {
    let limits = limits();
    for reason in ["restore λ".as_bytes(), b"".as_slice()] {
        let ask = c::Ask::Restore { revision: 3, reason: c::Text::new(reason.into(), 65536).unwrap() };
        let wire = asks::ask(&ask, limits.wire()).unwrap();
        let mut bytes = Writer::new(usize::try_from(wire.measure()).unwrap());
        wire.encode(&mut bytes).unwrap();
        let wire = w::Ask::decode(limits.wire(), &mut Reader::new(&bytes.finish())).unwrap();
        assert_eq!(translate::store::restore_ask(wire, &limits).unwrap(), ask);
        let saved = fixtures::saved_table(ask);
        let mut harness = fixtures::Harness::new(&fixtures::limits());
        harness.start_store(fixtures::one_store_read(fixtures::store_read(saved.clone())));
        assert!(harness.restored(&saved));
    }
    let long = w::Restore::new(limits.wire(), w::RestoreParts { revision: 3, reason: skein_lib::bytes::zeroed(65536) })
        .unwrap();
    asks::restore_ask_restore(&long, &limits).expect_err("the client text bound applies to restored reasons");
    w::Restore::new(limits.wire(), w::RestoreParts { revision: 3, reason: skein_lib::bytes::zeroed(65537) })
        .expect_err("the wire reason ceiling applies");
}
#[test]
fn restored_outcomes_and_forgotten_refusals_reach_the_contract() {
    let limits = limits();
    let mut left_out = List::with_capacity(2);
    left_out.push(c::Setting::ProjectPolicy { project: 7 }).unwrap();
    left_out.push(c::Setting::Application { kind: 2, scope: 7, number: 9 }).unwrap();
    let wire = answers::wire_outcome_restored(41, 3, &left_out, limits.wire()).unwrap();
    let wire = w::Outcome::Restored(wire);
    let mut bytes = Writer::new(usize::try_from(wire.measure()).unwrap());
    wire.encode(&mut bytes).unwrap();
    let wire = w::Outcome::decode(limits.wire(), &mut Reader::new(&bytes.finish())).unwrap();
    assert_eq!(answers::outcome(&wire, &limits).unwrap(), c::Outcome::Restored { proposal: 41, to: 3, left_out });
    let wire = answers::wire_refusal_forgotten(4, limits.wire()).unwrap();
    assert_eq!(answers::restore_refusal_forgotten(&wire), c::Refusal::Forgotten { oldest: 4 });
    assert_eq!(answers::refusal(&w::Refusal::Forgotten(wire), &limits).unwrap(), c::Refusal::Forgotten { oldest: 4 });
}
