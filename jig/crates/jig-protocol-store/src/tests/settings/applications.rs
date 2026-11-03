//! Application metadata and settings addresses (jig/docs/design/protocol/store.md, 4.4 and 5.1).
use super::{codec, core, fixtures, people, s};
use crate::{Event, Request, keys};
use s::application::{Follows, Recorded, SizedEdit, State};
use skein_lib::{List, Token, Wall};

fn head(number: u32) -> s::Head {
    s::Head { number, against: 0, at: Wall::EPOCH, by: s::Decider::FirstStart, proposal: None, moved: false }
}
fn applications(count: u32, number: u32) -> Box<[Recorded]> {
    let mut items = List::with_capacity(count);
    for at in 0..count {
        let edit = match at % 3 {
            0 => SizedEdit::Set { size: number },
            1 => SizedEdit::Remove,
            _ => SizedEdit::Reset,
        };
        items.push(Recorded { kind: number, scope: number, number, edit }).expect("fixture item count");
    }
    items.into_boxed()
}
fn round_trip(row: s::Record) {
    let limits = codec::settings::limits(&fixtures::CORE_LIMITS).unwrap();
    let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
    assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits, &[]), Ok(row.clone()));
    let (_, truncated) = bytes.split_last().expect("encoded row has a version");
    codec::settings::decode(&row.key(), truncated, &limits, &[]).expect_err("truncated row");
}

#[test]
fn application_states_round_trip_at_their_bounds() {
    for number in [0, u32::MAX] {
        for follows in [Follows::Default, Follows::Changed { revision: number }, Follows::Removed { revision: number }]
        {
            for count in [0, u32::from(u8::MAX)] {
                let mut charters = List::with_capacity(count);
                for _ in 0..count {
                    charters.push(number).unwrap();
                }
                round_trip(s::Record::Application {
                    setting: s::Setting::Application { kind: number, scope: number, number },
                    state: State { follows: follows.clone(), size: number, charters: charters.into_boxed() },
                });
            }
        }
    }
}

#[test]
fn application_revisions_proposals_and_setting_names_round_trip_at_their_bounds() {
    for number in [0, u32::MAX] {
        for count in [0, jig_domain_tasks::SETTINGS_ITEMS] {
            let items = Box::new([s::Item {
                setting: s::Setting::Application { kind: number, scope: number, number },
                edit: s::Edit::Reset,
            }]);
            for history in [false, true] {
                round_trip(s::Record::Revision {
                    history,
                    revision: s::Revision {
                        head: head(number),
                        items: items.clone(),
                        applications: applications(count, number),
                    },
                });
            }
            round_trip(s::Record::Proposal {
                proposer: people::Proposer::Party(u64::MAX),
                proposal: number,
                value: s::Proposal { key: None, against: number, items, applications: applications(count, number) },
            });
        }
    }
}

#[test]
fn application_metadata_past_each_count_bound_is_refused() {
    let limits = codec::settings::limits(&fixtures::CORE_LIMITS).unwrap();
    let row = s::Record::Proposal {
        proposer: people::Proposer::Party(1),
        proposal: 1,
        value: s::Proposal {
            key: None,
            against: 0,
            items: Box::new([]),
            applications: applications(jig_domain_tasks::SETTINGS_ITEMS + 1, 0),
        },
    };
    assert_eq!(codec::settings::encode(&row, &limits, &[]).unwrap_err().reason, skein_codec::Reason::Bound);
    let mut charters = List::with_capacity(256);
    for _ in 0_u32..256 {
        charters.push(0).unwrap();
    }
    let row = s::Record::Application {
        setting: s::Setting::Application { kind: 1, scope: 0, number: 1 },
        state: State { follows: Follows::Default, size: 0, charters: charters.into_boxed() },
    };
    assert_eq!(codec::settings::encode(&row, &limits, &[]).unwrap_err().reason, skein_codec::Reason::Bound);
}

#[test]
fn application_keys_sort_after_rules_by_kind_scope_and_number_in_both_subfamilies() {
    let settings = [
        s::Setting::Charter(u32::MAX),
        s::Setting::Template(u32::MAX),
        s::Setting::Policy(u32::MAX),
        s::Setting::Rules,
        s::Setting::Application { kind: 0, scope: 0, number: 0 },
        s::Setting::Application { kind: 0, scope: 0, number: 255 },
        s::Setting::Application { kind: 0, scope: 0, number: 256 },
        s::Setting::Application { kind: 0, scope: 0, number: u32::MAX },
        s::Setting::Application { kind: 0, scope: 1, number: 0 },
        s::Setting::Application { kind: 0, scope: u32::MAX, number: u32::MAX },
        s::Setting::Application { kind: 1, scope: 0, number: 0 },
        s::Setting::Application { kind: u32::MAX, scope: u32::MAX, number: u32::MAX },
    ];
    let mut previous = None;
    for changed in [false, true] {
        for setting in settings {
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
fn revision_reads_keep_application_items_beside_jig_items() {
    let mut store = fixtures::opened();
    let mut writes = List::with_capacity(3);
    writes.push(fixtures::header(1)).unwrap();
    for number in 1..=2 {
        writes
            .push(fixtures::save(core::Record::Core(core::CoreRecord::Settings(s::Record::Revision {
                history: number == 1,
                revision: s::Revision {
                    head: head(number),
                    items: Box::new([s::Item { setting: s::Setting::Template(1), edit: s::Edit::Removed }]),
                    applications: applications(3, number),
                },
            }))))
            .unwrap();
    }
    fixtures::committed(&mut store, writes.into_boxed());
    let range = core::reads::Range {
        party: fixtures::ALICE,
        scope: core::reads::Scope::SettingsHistory { first: 1, last: 2, newest: 2, whole: true },
        after: None,
        backwards: false,
        rows: 2,
        bytes: core::reads::settings::load_bytes(&fixtures::CORE_LIMITS, 2).unwrap(),
    };
    let events = fixtures::run(&mut store, Request::Read { owner: Token::new(7), range });
    let [Event::Read { result: core::reads::Loaded::Page { rows, next: None }, .. }] = events.as_ref() else {
        panic!("revision page: {events:?}")
    };
    assert_eq!(rows.len(), 2);
    for (at, row) in rows.iter().enumerate() {
        let revision = core::reads::settings::record_revision(&row.value).unwrap();
        let number = u32::try_from(at + 1).unwrap();
        assert_eq!(revision.head.number, number);
        assert_eq!(revision.applications, applications(3, number));
        assert_eq!(revision.items.as_ref(), &[s::Item { setting: s::Setting::Template(1), edit: s::Edit::Removed }]);
    }
}
