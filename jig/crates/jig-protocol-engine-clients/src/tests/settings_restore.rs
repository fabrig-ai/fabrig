//! Project policies, replaced values and restores cross the client boundary (domain/engine/settings.md, 10).
use super::fixtures as f;
use crate::translate::{self, settings};
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
use jig_domain_core::{documents as d, reads, settings as s};
use jig_domain_people as p;
use skein_lib::{List, Reader, Wall, Writer};
fn head() -> s::Head {
    s::Head { number: 6, against: 5, at: Wall::EPOCH, by: s::Decider::Release, proposal: None, moved: false }
}
fn authority() -> jig_domain_authority::Authority {
    use jig_domain_authority as a;
    a::Authority {
        tools: a::Tools(1),
        models: a::Models::ALL,
        grants: Box::new([]),
        delegation: a::Delegation { kinds: Box::new([a::Executor::Charter(1)]), tasks: 2, depth: 1 },
        budget: a::Budget { spend: 100, deadline: None },
        notes: a::Scopes(0),
        note_resources: Box::new([]),
    }
}
fn charter() -> jig_domain_core::Charter {
    use jig_domain_core as core;
    use skein_lib::Duration;
    core::Charter {
        parts: Box::new([]),
        rest: false,
        wake: jig_domain_tasks::WakePolicy::DEFAULT,
        documents: Box::new([
            core::Document {
                connector: 2,
                name: jig_domain_authority::Name {
                    segments: Box::new([Box::from(b"resource".as_slice()), Box::from(b"part".as_slice())]),
                },
                path: Box::from(b"guide.md".as_slice()),
                heading: None,
            },
            core::Document {
                connector: 3,
                name: jig_domain_authority::Name {
                    segments: Box::new([Box::from(b"another".as_slice()), Box::from(b"part".as_slice())]),
                },
                path: Box::from(b"rules.md".as_slice()),
                heading: Some(Box::from("Rules λ".as_bytes())),
            },
        ]),
        number: 1,
        hosts: jig_domain_fleet::Kinds::Both,
        spend: 100,
        policy: core::RunPolicy {
            instructions: Box::from("instructions λ".as_bytes()),
            waiting: Duration::from_secs(2),
            resume: true,
            turns: 4,
            time: Duration::from_secs(30),
            reserve_turns: 1,
            reserve_time: Duration::from_secs(5),
            model: core::Model {
                provider: 1,
                dialect: 3,
                endpoint: 4,
                program: Some(Box::from(b"program".as_slice())),
                name: Box::from(b"careful".as_slice()),
                cache_life_ms: 0,
                window: 8192,
                output: 1024,
                input_price: 1,
                cached_price: 2,
                cache_write_price: Some(3),
                output_price: 4,
                price_unit: 1000,
                efforts: Box::new([Box::from(b"high".as_slice())]),
                effort: Some(Box::from(b"high".as_slice())),
            },
            alternatives: Box::new([]),
            inspect: true,
            modify: false,
            shell: true,
            agents: false,
            call_timeout: Duration::from_secs(1),
            tools: core::RunPolicy::EVERY_FAMILY,
        },
    }
}
fn values() -> [s::Value; 4] {
    use jig_domain_authority as a;
    [
        s::Value::Charter(Box::new(charter())),
        s::Value::Template(Box::new(f::template())),
        s::Value::Policy(Box::new(jig_domain_core::ConfiguredPolicy {
            policy: a::Policy {
                escalation_role: Some(1),
                ceiling: authority(),
                projections: Box::new([]),
                period_spend: 100,
                roles: Box::new([a::Role {
                    number: 1,
                    authority: authority(),
                    period_spend: 100,
                    requests: a::Requests::ALL,
                    decides: a::Proposals::ALL,
                }]),
                requirements: Box::new([]),
            },
            permissions: Box::new([jig_domain_people::PermissionRole { connector: 1, permission: 2, role: 1 }]),
            templates: Box::new([]),
        })),
        s::Value::Rules(Box::new(a::Rules {
            ceiling: authority(),
            period_spend: 100,
            minimum_run_spend: 1,
            maximum_run_spend: 100,
            implies: a::Implies::new(Box::new([]), 0).unwrap(),
            requirements: Box::new([]),
        })),
    ]
}

#[test]
fn a_project_policy_item_enters_the_core_from_a_change_and_a_proposal() {
    let limits = f::wire();
    let policy = values().into_iter().nth(2).unwrap();
    let expected = s::Item { setting: s::Setting::ProjectPolicy { project: 7 }, edit: s::Edit::Set(policy.clone()) };
    let row = w::SettingItem::new(
        &limits,
        w::SettingItemParts {
            setting: settings::name(expected.setting, &limits).unwrap(),
            edit: w::SettingEdit::Set(
                w::SettingSet::new(
                    &limits,
                    w::SettingSetParts { value: settings::value(&policy, &f::models(), &limits).unwrap() },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap();
    let mut bytes = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut bytes).unwrap();
    let items = f::pieces(&[bytes.finish()]);
    let change = w::ChangeSettings::new(
        &limits,
        w::ChangeSettingsParts { against: 5, items: items.clone(), running: w::Running::Stays },
    )
    .unwrap();
    assert_eq!(
        settings::change(change, &f::models(), &limits).unwrap().items.as_ref(),
        ::core::slice::from_ref(&expected)
    );
    let proposal = w::Propose::new(
        &limits,
        w::ProposeParts {
            project: 0,
            action: w::ProposeAction::Settings(
                w::ProposeSettings::new(&limits, w::ProposeSettingsParts { against: 5, items }).unwrap(),
            ),
            reason: Box::from(b"review".as_slice()),
        },
    )
    .unwrap();
    assert_eq!(settings::proposed(proposal, &f::models(), &limits).unwrap().items.as_ref(), &[expected]);
}
#[test]
fn a_project_policy_review_item_carries_the_policy_now_and_as_it_would_be() {
    let limits = f::wire();
    let now = values().into_iter().nth(2).unwrap();
    let mut would_be = now.clone();
    let s::Value::Policy(policy) = &mut would_be else { panic!("policy") };
    policy.policy.period_spend += 1;
    let now_wire = settings::value(&now, &f::models(), &limits).unwrap();
    let would_be_wire = settings::value(&would_be, &f::models(), &limits).unwrap();
    let bytes = u64::from(now_wire.measure().max(would_be_wire.measure()));
    assert_eq!(now_wire.measure(), would_be_wire.measure(), "both sides share the exact bound");
    for (bound, owned, large) in [
        (bytes, limits.owned_record, false),
        (bytes - 1, limits.owned_record, true),
        (u64::MAX, u32::try_from(bytes).unwrap(), false),
        (u64::MAX, u32::try_from(bytes - 1).unwrap(), true),
    ] {
        let limits = w::Limits { owned_record: owned, ..limits };
        let mut items = List::with_capacity(1);
        items
            .push(d::ReviewItem {
                setting: s::Setting::ProjectPolicy { project: 7 },
                edit: reads::RevisionEdit::Set,
                now: Some(now.clone()),
                origin: None,
                would_be: Some(would_be.clone()),
                application: None,
            })
            .unwrap();
        let review = d::SettingsReview { key: None, newest: 5, items, value_bytes: bound };
        let wire = settings::review(&review, &f::models(), &limits).unwrap();
        let piece = skein_codec::piece::split(&mut Reader::new(wire.items())).unwrap().unwrap();
        let row = w::ReviewItem::decode(&limits, &mut Reader::new(&piece)).unwrap();
        assert_eq!(settings::setting(row.setting()), s::Setting::ProjectPolicy { project: 7 });
        assert_eq!(row.origin(), &w::SettingOrigin::NotOffered);
        assert!(row.application().is_none());
        for (shown, expected) in [(row.now(), &now_wire), (row.would_be(), &would_be_wire)] {
            if large {
                let w::ReviewValue::TooLarge(size) = shown else { panic!("size") };
                assert_eq!(size.bytes(), u64::from(expected.measure()));
            } else {
                let w::ReviewValue::Value(value) = shown else { panic!("whole policy") };
                assert_eq!(value.value(), expected);
            }
        }
    }
}
#[test]
fn revision_items_carry_what_each_replaced() {
    let limits = w::CEILINGS;
    let value = s::Value::Charter(Box::new(charter()));
    let bytes = u64::from(settings::value(&value, &f::models(), &limits).unwrap().measure());
    let revision = s::Revision {
        head: head(),
        items: Box::new([
            s::RevisionItem {
                setting: s::Setting::Charter(1),
                edit: s::Edit::Set(value.clone()),
                replaced: s::Replaced::Value(value),
            },
            s::RevisionItem {
                setting: s::Setting::Template(2),
                edit: s::Edit::Removed,
                replaced: s::Replaced::Default,
            },
            s::RevisionItem { setting: s::Setting::Policy(3), edit: s::Edit::Reset, replaced: s::Replaced::Removed },
            s::RevisionItem { setting: s::Setting::Rules, edit: s::Edit::Reset, replaced: s::Replaced::NotOffered },
        ]),
        applications: applications(),
    };
    for (bound, large) in [(bytes, false), (bytes - 1, true)] {
        let answer = reads::RevisionAnswer::Revision { revision: revision.clone(), value_bytes: bound };
        let wire = settings::revision(&answer, &f::models(), &limits).unwrap();
        let c::RevisionAnswer::Revision(document) =
            jig_client_protocol_link::translate::settings::restore_revision_answer(&wire, &f::client_limits()).unwrap()
        else {
            panic!("revision")
        };
        assert_eq!(document.items.len(), 8);
        let first = document.items.get(0).unwrap();
        if large {
            assert_eq!(first.replaced, c::RevisionReplaced::TooLarge(c::Size { bytes }));
            assert_eq!(first.edit, c::RevisionEdit::TooLarge(c::Size { bytes }));
        } else {
            let c::RevisionReplaced::Value(value) = &first.replaced else { panic!("whole replaced") };
            assert_eq!(first.edit, c::RevisionEdit::Set(value.clone()));
        }
        for (index, expected) in [
            c::RevisionReplaced::Default,
            c::RevisionReplaced::Removed,
            c::RevisionReplaced::NotOffered,
            c::RevisionReplaced::Application { size: 91 },
            c::RevisionReplaced::Default,
            c::RevisionReplaced::Removed,
            c::RevisionReplaced::NotOffered,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(document.items.get(u32::try_from(index + 1).unwrap()).unwrap().replaced, expected);
        }
    }
}
#[test]
fn a_restore_ask_enters_the_people_vocabulary() {
    let limits = f::wire();
    let ask = w::Ask::Restore(
        w::Restore::new(&limits, w::RestoreParts { revision: 3, reason: Box::from("restore λ".as_bytes()) }).unwrap(),
    );
    let bytes = f::encode_ask(&ask);
    let ask = w::Ask::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    let named = translate::asks::named(ask, &f::models(), &limits).unwrap();
    assert!(named.models.is_empty());
    assert_eq!(named.ask, p::Ask::Restore { revision: 3, reason: Box::from("restore λ".as_bytes()) });
}
#[test]
fn a_restored_outcome_and_a_forgotten_refusal_reach_the_wire() {
    let left_out =
        Box::new([p::Setting::ProjectPolicy { project: 7 }, p::Setting::Application { kind: 2, scope: 7, number: 9 }]);
    let reply = p::Reply::Outcome(p::Outcome::Restored { proposal: 41, to: 3, left_out });
    let wire = translate::answers::answer(&reply, &f::models(), &f::wire()).unwrap();
    let answer = jig_client_protocol_link::translate::answers::answer(wire, &f::client_limits()).unwrap();
    let c::Answer::Done(c::Outcome::Restored { proposal, to, left_out }) = answer else { panic!("restored") };
    assert_eq!((proposal, to), (41, 3));
    assert_eq!(
        left_out.as_slice(),
        &[c::Setting::ProjectPolicy { project: 7 }, c::Setting::Application { kind: 2, scope: 7, number: 9 }]
    );
    let reply = p::Reply::Refused(p::Refusal::Forgotten { oldest: 4 });
    let wire = translate::answers::answer(&reply, &f::models(), &f::wire()).unwrap();
    assert_eq!(
        jig_client_protocol_link::translate::answers::answer(wire, &f::client_limits()).unwrap(),
        c::Answer::Refused { refusal: c::Refusal::Forgotten { oldest: 4 }, at: None }
    );
}

fn applications() -> Box<[s::application::RevisionRecorded]> {
    let mut items = List::with_capacity(4);
    for (number, replaced) in [
        s::application::Replaced::Changed { size: 91 },
        s::application::Replaced::Default,
        s::application::Replaced::Removed,
        s::application::Replaced::NotOffered,
    ]
    .into_iter()
    .enumerate()
    {
        items
            .push(s::application::RevisionRecorded {
                kind: 1,
                scope: 2,
                number: u32::try_from(number).unwrap(),
                edit: s::application::SizedEdit::Set { size: 99 },
                replaced,
            })
            .unwrap();
    }
    items.into_boxed()
}
