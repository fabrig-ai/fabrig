//! Revisions preserve replaced values and restore addresses (domain/engine/settings.md, 10).
use super::{a, catalogue_name, charter, codec, core, fixtures, people, policy, rules, s, template};
use crate::{keys, settings_v2 as wire};
use s::application;
use skein_lib::{List, Reader, Wall, Writer};

fn head() -> s::Head {
    s::Head { number: u32::MAX, against: 0, at: Wall::EPOCH, by: s::Decider::FirstStart, proposal: None, moved: false }
}
fn limits() -> wire::Limits {
    let mut domain = fixtures::CORE_LIMITS;
    domain.authority.executors = 3;
    codec::settings::limits(&domain).unwrap()
}
fn round_trip(row: s::Record) {
    let limits = limits();
    let names = [catalogue_name(b"one"), catalogue_name(b"two")];
    let bytes = codec::settings::encode(&row, &limits, &names).unwrap();
    assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits, &names), Ok(row.clone()));
    let (_, truncated) = bytes.split_last().unwrap();
    codec::settings::decode(&row.key(), truncated, &limits, &names).expect_err("truncated row");
}
fn revision(items: Box<[s::RevisionItem]>) -> s::Record {
    s::Record::Revision { history: false, revision: s::Revision { head: head(), items, applications: Box::new([]) } }
}

#[test]
fn revision_items_round_trip_every_replaced_state() {
    for (setting, value) in [
        (s::Setting::Charter(u32::MAX), s::Value::Charter(Box::new(charter()))),
        (s::Setting::Template(u32::MAX), s::Value::Template(Box::new(template()))),
        (s::Setting::Policy(u32::MAX), s::Value::Policy(Box::new(policy()))),
        (s::Setting::Rules, s::Value::Rules(Box::new(rules()))),
    ] {
        for replaced in
            [s::Replaced::Value(value.clone()), s::Replaced::Default, s::Replaced::Removed, s::Replaced::NotOffered]
        {
            for history in [false, true] {
                round_trip(s::Record::Revision {
                    history,
                    revision: s::Revision {
                        head: head(),
                        applications: Box::new([]),
                        items: Box::new([s::RevisionItem {
                            setting,
                            edit: s::Edit::Set(value.clone()),
                            replaced: replaced.clone(),
                        }]),
                    },
                });
            }
        }
    }
    revision_item_sum_and_count_bounds();
}

fn revision_item_sum_and_count_bounds() {
    let mut domain = fixtures::CORE_LIMITS;
    domain.authority.executors = 3;
    domain.documents.text_bytes = 65536;
    domain.templates = 0;
    let limits = codec::settings::limits(&domain).unwrap();
    let count = s::capacity(&domain).unwrap().checked_mul(2).unwrap().min(jig_domain_tasks::SETTINGS_ITEMS);
    assert_eq!(limits.proposal_items, count);
    let mut value = template();
    value.name = Box::new([]);
    let value = s::Value::Template(Box::new(value));
    let item = s::RevisionItem {
        setting: s::Setting::Template(1),
        edit: s::Edit::Set(value.clone()),
        replaced: s::Replaced::Value(value),
    };
    let row = revision(repeated_items(&item, count));
    let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
    let encoded = wire::Revision::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    let mut remaining = usize::try_from(limits.revision_items).unwrap().checked_sub(encoded.items().len()).unwrap();
    let s::Record::Revision { mut revision, .. } = row else { panic!("revision") };
    for item in &mut revision.items {
        let (s::Edit::Set(s::Value::Template(edited)), s::Replaced::Value(s::Value::Template(replaced))) =
            (&mut item.edit, &mut item.replaced)
        else {
            panic!("template values")
        };
        for value in [edited, replaced] {
            let length = remaining.min(skein_lib::length::usize_of(limits.template_name));
            value.name = skein_lib::bytes::zeroed(length);
            remaining = remaining.checked_sub(length).unwrap();
        }
    }
    assert_eq!(remaining, 0, "fixture fills the derived byte sum within the count");
    let row = s::Record::Revision { history: false, revision: revision.clone() };
    let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
    let encoded = wire::Revision::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    assert_eq!(encoded.items().len(), usize::try_from(limits.revision_items).unwrap());
    // Empty catalogues discard named masks and choices; compare against the same catalogue on both sides.
    let names = [catalogue_name(b"one"), catalogue_name(b"two")];
    let bytes = codec::settings::encode(&row, &limits, &names).unwrap();
    assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits, &names), Ok(row));
    let last = revision.items.last_mut().unwrap();
    let s::Edit::Set(s::Value::Template(value)) = &mut last.edit else { panic!("template") };
    value.name = skein_lib::bytes::zeroed(value.name.len().checked_add(1).unwrap());
    let row = s::Record::Revision { history: false, revision };
    assert_eq!(codec::settings::encode(&row, &limits, &names).unwrap_err().reason, skein_codec::Reason::Bound);
    let wider = wire::Limits { revision_items: limits.revision_items.checked_add(1).unwrap(), ..limits };
    let bytes = codec::settings::encode(&row, &wider, &names).unwrap();
    assert_eq!(
        codec::settings::decode(&row.key(), &bytes, &limits, &names).unwrap_err().reason,
        skein_codec::Reason::Bound
    );
    let row = revision_row_past_count(item, count);
    assert_eq!(codec::settings::encode(&row, &limits, &names).unwrap_err().reason, skein_codec::Reason::Bound);
    let wider = wire::Limits { proposal_items: count.checked_add(1).unwrap(), ..limits };
    let bytes = codec::settings::encode(&row, &wider, &names).unwrap();
    assert_eq!(
        codec::settings::decode(&row.key(), &bytes, &limits, &names).unwrap_err().reason,
        skein_codec::Reason::Bound
    );
}
fn repeated_items(item: &s::RevisionItem, count: u32) -> Box<[s::RevisionItem]> {
    let mut items = List::with_capacity(count);
    for _ in 0..count {
        items.push(item.clone()).unwrap();
    }
    items.into_boxed()
}
fn revision_row_past_count(mut item: s::RevisionItem, count: u32) -> s::Record {
    item.edit = s::Edit::Reset;
    revision(repeated_items(&item, count.checked_add(1).unwrap()))
}

#[test]
fn a_project_policy_item_round_trips_in_a_revision_and_a_proposal() {
    let setting = s::Setting::ProjectPolicy { project: u32::MAX };
    let value = s::Value::Policy(Box::new(policy()));
    let edit = s::Edit::Set(value.clone());
    for history in [false, true] {
        round_trip(s::Record::Revision {
            history,
            revision: s::Revision {
                head: head(),
                applications: Box::new([]),
                items: Box::new([s::RevisionItem {
                    setting,
                    edit: edit.clone(),
                    replaced: s::Replaced::Value(value.clone()),
                }]),
            },
        });
    }
    round_trip(s::Record::Proposal {
        proposer: people::Proposer::Party(u64::MAX),
        proposal: u32::MAX,
        value: s::Proposal {
            key: None,
            against: u32::MAX,
            items: Box::new([s::Item { setting, edit }]),
            applications: Box::new([]),
        },
    });
    round_trip(s::Record::Changed { setting, changed: s::Changed { value: Some(value.clone()), revision: u32::MAX } });
    round_trip(s::Record::Default { setting, default: s::Default { value, revision: u32::MAX } });
    let row = s::Record::Application {
        setting,
        state: application::State { follows: application::Follows::Default, size: 0, charters: Box::new([]) },
    };
    assert_eq!(codec::settings::encode(&row, &limits(), &[]).unwrap_err().reason, skein_codec::Reason::Bound);
}

#[test]
fn revision_application_items_round_trip_every_replaced_state() {
    for replaced in [
        application::Replaced::Changed { size: 0 },
        application::Replaced::Changed { size: u32::MAX },
        application::Replaced::Default,
        application::Replaced::Removed,
        application::Replaced::NotOffered,
    ] {
        for count in [0, jig_domain_tasks::SETTINGS_ITEMS, jig_domain_tasks::SETTINGS_ITEMS + 1] {
            let mut applications = List::with_capacity(count);
            for at in 0..count {
                let edit = match at % 3 {
                    0 => application::SizedEdit::Set { size: u32::MAX },
                    1 => application::SizedEdit::Remove,
                    _ => application::SizedEdit::Reset,
                };
                applications
                    .push(application::RevisionRecorded {
                        kind: u32::MAX,
                        scope: u32::MAX,
                        number: u32::MAX,
                        edit,
                        replaced: replaced.clone(),
                    })
                    .unwrap();
            }
            for history in [false, true] {
                let row = s::Record::Revision {
                    history,
                    revision: s::Revision {
                        head: head(),
                        items: Box::new([]),
                        applications: applications.as_slice().into(),
                    },
                };
                if count > jig_domain_tasks::SETTINGS_ITEMS {
                    assert_eq!(
                        codec::settings::encode(&row, &limits(), &[]).unwrap_err().reason,
                        skein_codec::Reason::Bound
                    );
                } else {
                    round_trip(row);
                }
            }
        }
    }
}

#[test]
fn replaced_values_follow_model_names_after_a_catalogue_reorder() {
    let before = [catalogue_name(b"one"), catalogue_name(b"two")];
    let after = [before[1].clone(), before[0].clone()];
    let mut value = rules();
    value.ceiling.models = a::Models::Named(1);
    let row = revision(Box::new([s::RevisionItem {
        setting: s::Setting::Rules,
        edit: s::Edit::Reset,
        replaced: s::Replaced::Value(s::Value::Rules(Box::new(value))),
    }]));
    let limits = limits();
    let bytes = codec::settings::encode(&row, &limits, &before).unwrap();
    let s::Record::Revision { revision, .. } = codec::settings::decode(&row.key(), &bytes, &limits, &after).unwrap()
    else {
        panic!("revision")
    };
    let s::Replaced::Value(s::Value::Rules(value)) = &revision.items[0].replaced else { panic!("replaced rules") };
    assert_eq!(value.ceiling.models, a::Models::Named(2));
}

#[test]
fn a_project_policy_key_round_trips_and_sorts_after_application_settings() {
    for changed in [false, true] {
        let mut previous = None;
        for setting in [
            s::Setting::Application { kind: u32::MAX, scope: u32::MAX, number: u32::MAX },
            s::Setting::ProjectPolicy { project: 0 },
            s::Setting::ProjectPolicy { project: 1 },
            s::Setting::ProjectPolicy { project: u32::MAX },
        ] {
            let key = if changed { s::Key::Changed(setting) } else { s::Key::Default(setting) };
            let key = core::Key::Core(core::CoreKey::Settings(key));
            let bytes = keys::encode(&key, fixtures::KEY_LIMIT).unwrap();
            assert_eq!(keys::decode(&bytes), Some(key));
            assert_eq!(&bytes[..4], &[1, 0, 8, u8::from(changed)]);
            if let Some(before) = previous {
                assert!(before < bytes);
            }
            previous = Some(bytes);
        }
    }
}

#[test]
fn revision_pieces_must_end_at_their_declared_lengths() {
    let row = revision(Box::new([s::RevisionItem {
        setting: s::Setting::Rules,
        edit: s::Edit::Reset,
        replaced: s::Replaced::Default,
    }]));
    let limits = limits();
    let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
    let value = wire::Revision::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    let mut parts = value.into_parts();
    let mut reader = Reader::new(&parts.items);
    let item = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    let mut padded = Writer::new(item.len() + 1);
    padded.put(&item).unwrap();
    padded.put(&[0]).unwrap();
    let padded = padded.finish();
    let mut writer = Writer::new(padded.len() + 4);
    skein_codec::piece::write(&mut writer, &padded).unwrap();
    parts.items = writer.finish();
    let value = wire::Revision::new(&limits, parts).unwrap();
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    assert_eq!(
        codec::settings::decode(&row.key(), &writer.finish(), &limits, &[]).unwrap_err().reason,
        skein_codec::Reason::Trailing
    );
}
