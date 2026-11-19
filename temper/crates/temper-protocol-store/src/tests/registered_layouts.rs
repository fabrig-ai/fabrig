//! Registered layouts derive field defaults (jig protocol/store/layouts.md, 4 and 5).
#![expect(clippy::disallowed_methods, reason = "ordinary Rust schema contract tests")]

#[test]
fn a_registered_layout_defaults_an_added_list() {
    let old = skein_codec::schema::parse(b"family fixture 1\nrecord Item { taken: u32 }\n").unwrap();
    let new = skein_codec::schema::parse(b"family fixture 1\nrecord Item { taken: u32 waiting: list 16 u64 = [] }\n")
        .unwrap();
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"Item", None).unwrap(),
        &[],
        &new,
        new.root(b"Item", None).unwrap(),
        &[],
    )
    .expect("the registry derives the added field's default at open");
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1024);
    plan.convert(&4_u32.to_be_bytes(), &mut out, &mut work).unwrap();
    // The retained count, followed by an empty waiting list.
    assert_eq!(out.as_slice(), &[0, 0, 0, 4, 0, 0, 0, 0]);
}

#[test]
fn a_landing_layout_without_wait_defaults_it_to_cleared() {
    use super::fixtures;
    use crate::records;
    use skein_codec::{
        layout::{Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary::{Stored, settings as d};
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replacen(" wait: SettingsLandingWait = cleared", "", 1);
    assert_ne!(before, family.current, "the earlier landing omits wait");
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"setting")).unwrap();
    let new_root = new.root(b"Row", Some(b"setting")).unwrap();
    let write_old = Plan::derive_dropping(&new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let stored = Stored::Setting(d::SettingRow {
        key: d::SettingKey { kind: 4, scope: 7, number: 1, slot: d::SettingSlot::Change },
        edit: d::Edit::Set(d::SizedValue {
            value: d::Value::Landing(d::ProjectLanding { batch: 3, ..d::ProjectLanding::default() }),
            size: 123,
        }),
    });
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    write_old.convert(&records::encode(&stored), &mut earlier, &mut work).unwrap();
    let read_old =
        Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("an added wait defaults stored kind 4 values");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(4096);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(current.as_slice(), &bounds), Ok(stored));
}

#[test]
fn a_change_layout_without_source_or_kept_defaults_both_to_none() {
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let mut before = family.current.to_owned();
    for field in ["source: option ChangeSource = none ", "kept: option fixed 32 = none "] {
        before = before.replacen(field, "", 1);
    }
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    for (root, bytes) in [
        (b"ChangeCold".as_slice(), golden(family.current, "record_change_cold_smallest.bin")),
        (b"ChangeHot".as_slice(), golden(family.current, "record_change_hot_smallest.bin")),
    ] {
        let plan = skein_codec::layout::Plan::derive(
            &old,
            old.root(root, None).unwrap(),
            &[],
            &new,
            new.root(root, None).unwrap(),
            &[],
        )
        .unwrap();
        let mut work = skein_codec::layout::Workspace::new();
        assert!(work.fit(&plan));
        let mut out = skein_lib::Buffer::with_capacity(1024);
        // The added option's zero byte is absent from the earlier smallest record.
        plan.convert(&bytes[1..], &mut out, &mut work).unwrap();
        if root == b"ChangeCold" {
            let row =
                crate::v25::ChangeCold::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice()))
                    .unwrap();
            assert!(row.source().is_none());
        } else {
            let row = crate::v25::ChangeHot::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice()))
                .unwrap();
            assert!(row.kept().is_none());
        }
    }
}

#[test]
fn the_current_layout_has_no_whole_change_value_type() {
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let current = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    assert!(current.root(b"Row", Some(b"change")).is_none());
    assert!(!family.types.iter().any(|ty| ty.number == 12));
}

#[test]
fn a_change_layout_without_regate_defaults_it_to_none() {
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replacen("regate: option Regate = none ", "", 1);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"ChangeHot", None).unwrap(),
        &[],
        &new,
        new.root(b"ChangeHot", None).unwrap(),
        &[],
    )
    .expect("an added regate has a default for stored change rows");
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1024);
    let current = golden(family.current, "record_change_hot_smallest.bin");
    // The earlier row lacks the first option's absent byte.
    plan.convert(&current[1..], &mut out, &mut work).unwrap();
    let row =
        crate::v25::ChangeHot::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice())).unwrap();
    assert!(row.regate().is_none());
}

#[test]
fn a_repository_layout_without_landing_defaults_it_to_its_default_branch() {
    use super::fixtures;
    use crate::records;
    use skein_codec::{
        layout::{Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary::{Protection, Stored};

    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replacen(" landing: bytes 4096 = 0x", "", 1);
    assert_ne!(before, family.current, "the old layout omits landing");
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"repository")).unwrap();
    let new_root = new.root(b"Row", Some(b"repository")).unwrap();
    let write_old = Plan::derive_dropping(&new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let stored = fixtures::adopted(Protection::Absent);
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    write_old.convert(&records::encode(&stored), &mut earlier, &mut work).unwrap();

    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[])
        .expect("an added landing has a default for stored repositories");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(4096);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let decoded = records::decode(current.as_slice(), &bounds).unwrap();
    let Stored::Repository(row) = &decoded else { panic!("repository") };
    assert_eq!(row.settings.default_branch.as_ref(), b"main");
    assert_eq!(row.landing.as_ref(), b"main");
    assert_eq!(decoded, stored);
}

#[test]
fn an_application_adopted_layout_without_landing_defaults_it_to_empty() {
    use super::{fixtures, jobs};
    use crate::records;
    use skein_codec::{
        layout::{Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary::{ApplicationOutcome, Stored};

    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replacen(
        "protection: RepositoryProtection landing: bytes 4096 = 0x }",
        "protection: RepositoryProtection }",
        1,
    );
    assert_ne!(before, family.current, "the old outcome layout omits landing");
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"application")).unwrap();
    let new_root = new.root(b"Row", Some(b"application")).unwrap();
    let write_old = Plan::derive_dropping(&new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let mut stored = jobs::application(fixtures::REPO);
    let Stored::Application { outcome: ApplicationOutcome::Adopted(outcome), .. } = &mut stored else {
        panic!("adopted application")
    };
    outcome.landing = b"staging".as_slice().into();
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    write_old.convert(&records::encode(&stored), &mut earlier, &mut work).unwrap();

    let read_old =
        Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("an added outcome landing has an empty default");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(4096);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let decoded = records::decode(current.as_slice(), &bounds).unwrap();
    let Stored::Application { outcome: ApplicationOutcome::Adopted(outcome), .. } = &mut stored else {
        panic!("adopted application")
    };
    outcome.landing = Box::new([]);
    assert_eq!(decoded, stored, "an old outcome keeps its facts but names no landing");
}

fn golden(text: &'static str, name: &str) -> Box<[u8]> {
    skein_codegen::values(text)
        .goldens
        .clone()
        .into_iter()
        .find(|value| value.name == name)
        .expect("named fixed-rule Change row")
        .bytes
        .into_boxed_slice()
}

#[test]
fn an_earlier_landing_red_defaults_test_ids_and_many_names() {
    use super::fixtures;
    use crate::records;
    use skein_codec::{
        layout::{Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary as d;
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replace(" tests: list 64 FailingTest = [] many: option LoggedMany", "");
    assert_ne!(before, family.current);
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"landing_log")).unwrap();
    let new_root = new.root(b"Row", Some(b"landing_log")).unwrap();
    let write_old = Plan::derive_dropping(&new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let stored = d::Stored::LandingLog(d::LandingLog {
        repository: skein_forge::api::Repository { forge: 1, repository: 2 },
        landings: Box::new([d::LoggedLanding {
            commit: [7; 32],
            branch: b"main".as_slice().into(),
            at: skein_lib::Wall::EPOCH,
            changes: Box::new([]),
            tiers: Box::new([
                d::LoggedTier {
                    tier: b"main".as_slice().into(),
                    verdict: d::LoggedVerdict::Red {
                        run: 7,
                        failing: Some(2),
                        tests: Box::new([b"suite::a".as_slice().into()]),
                        many: None,
                    },
                },
                d::LoggedTier {
                    tier: b"fuzzy".as_slice().into(),
                    verdict: d::LoggedVerdict::Red {
                        run: 9,
                        failing: None,
                        tests: Box::new([]),
                        many: Some(temper_engine_domain_forge_tiers::ManyTests {
                            tests: Box::new([b"suite::t00".as_slice().into()]),
                            more: 21,
                            cut: true,
                        }),
                    },
                },
                d::LoggedTier { tier: b"removed".as_slice().into(), verdict: d::LoggedVerdict::Cleared { run: 6 } },
            ]),
        }]),
    });
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    write_old.convert(&records::encode(&stored), &mut earlier, &mut work).unwrap();
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("red ids and names have defaults");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(4096);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let d::Stored::LandingLog(row) = records::decode(current.as_slice(), &bounds).unwrap() else {
        panic!("landing log")
    };
    assert_eq!(
        row.landings[0].tiers.as_ref(),
        &[
            d::LoggedTier {
                tier: b"main".as_slice().into(),
                verdict: d::LoggedVerdict::Red { run: 7, failing: Some(2), tests: Box::new([]), many: None }
            },
            d::LoggedTier {
                tier: b"fuzzy".as_slice().into(),
                verdict: d::LoggedVerdict::Red { run: 9, failing: None, tests: Box::new([]), many: None }
            },
            d::LoggedTier { tier: b"removed".as_slice().into(), verdict: d::LoggedVerdict::Cleared { run: 6 } },
        ]
    );
}

#[test]
fn an_earlier_landing_log_defaults_member_goals_and_tier_entries() {
    use skein_codec::{
        layout::{Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary as d;
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let mut before = family.current.to_owned();
    for field in [" goal: option u64 = none", " milestone: option u64 = none", " tiers: list 32 LoggedTier = []"] {
        before = before.replacen(field, "", 1);
    }
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"landing_log")).unwrap();
    let new_root = new.root(b"Row", Some(b"landing_log")).unwrap();
    let write_old = Plan::derive_dropping(&new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let stored = d::Stored::LandingLog(d::LandingLog {
        repository: skein_forge::api::Repository { forge: 1, repository: 2 },
        landings: Box::new([d::LoggedLanding {
            commit: [7; 32],
            branch: b"main".as_slice().into(),
            at: skein_lib::Wall::from_nanos(123),
            changes: Box::new([d::LoggedChange {
                signed: Box::new([]),
                task: 51,
                pull: Some(2),
                title: b"member".as_slice().into(),
                goal: Some(42),
                milestone: Some(40),
            }]),
            tiers: Box::new([d::LoggedTier { tier: b"slow".as_slice().into(), verdict: d::LoggedVerdict::Verifying }]),
        }]),
    });
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    write_old.convert(&crate::records::encode(&stored), &mut earlier, &mut work).unwrap();
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("added landing fields have defaults");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(4096);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&super::fixtures::LIMITS, super::fixtures::LIMITS.client.op_bytes).unwrap();
    let decoded = crate::records::decode(current.as_slice(), &bounds).unwrap();
    let d::Stored::LandingLog(row) = decoded else { panic!("landing log") };
    assert_eq!(row.landings.len(), 1);
    assert_eq!(row.landings[0].commit, [7; 32]);
    assert!(row.landings[0].tiers.is_empty());
    for member in &row.landings[0].changes {
        assert_eq!(member.task, 51);
        assert_eq!(member.pull, Some(2));
        assert_eq!(member.title.as_ref(), b"member");
        assert_eq!((member.goal, member.milestone), (None, None));
    }
}
