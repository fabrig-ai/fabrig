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

#[test]
fn earlier_landing_members_and_change_rows_default_goal_facts() {
    use skein_codec::{
        layout::{Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary as d;
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let mut before = family.current.to_owned();
    before = before.replacen(
        "record LoggedChange { person: option u64 = none goal_title: option bytes 256 = none draft: option bytes 256 = none",
        "record LoggedChange {", 1,
    );
    before = before.replacen(
        "record ChangeCold { person: option u64 = none goal_title: option bytes 256 = none draft: option bytes 256 = none",
        "record ChangeCold {", 1,
    );
    assert_ne!(before, family.current);
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let person = Some(7);
    let goal_title = Some(Box::from(b"R2 Reds blamed".as_slice()));
    let draft = Some(Box::from(b"temper/docs/design/draft/after-landing-repairs.md".as_slice()));
    let (mut cold, _) = super::fixtures::change_row(temper_engine_domain_forge_change::State::Ended).split();
    cold.goal = Some(42);
    cold.milestone = Some(40);
    cold.person = person;
    cold.goal_title = goal_title.clone();
    cold.draft = draft.clone();
    let log = d::LandingLog {
        repository: cold.repository,
        landings: Box::new([d::LoggedLanding {
            commit: [7; 32],
            branch: b"main".as_slice().into(),
            at: skein_lib::Wall::from_nanos(123),
            tiers: Box::new([]),
            changes: Box::new([d::LoggedChange {
                task: cold.task,
                pull: Some(2),
                title: b"member".as_slice().into(),
                signed: Box::new([]),
                goal: cold.goal,
                milestone: cold.milestone,
                person,
                goal_title,
                draft,
            }]),
        }]),
    };
    let mut old_cold = cold.clone();
    old_cold.person = None;
    old_cold.goal_title = None;
    old_cold.draft = None;
    let mut old_log = log.clone();
    let member = &mut old_log.landings[0].changes[0];
    member.person = None;
    member.goal_title = None;
    member.draft = None;
    for (variant, stored, expected) in [
        (b"landing_log".as_slice(), d::Stored::LandingLog(log), d::Stored::LandingLog(old_log)),
        (b"change_cold".as_slice(), d::Stored::ChangeCold(Box::new(cold)), d::Stored::ChangeCold(Box::new(old_cold))),
    ] {
        let old_root = old.root(b"Row", Some(variant)).unwrap();
        let new_root = new.root(b"Row", Some(variant)).unwrap();
        let write_old = Plan::derive_dropping(&new, new_root, &[], &old, old_root, &[]).unwrap();
        let mut work = Workspace::new();
        assert!(work.fit(&write_old));
        let mut earlier = skein_lib::Buffer::with_capacity(4096);
        write_old.convert(&crate::records::encode(&stored), &mut earlier, &mut work).unwrap();
        let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("goal facts default to none");
        assert!(work.fit(&read_old));
        let mut current = skein_lib::Buffer::with_capacity(4096);
        read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
        let bounds = crate::bounds(&super::fixtures::LIMITS, super::fixtures::LIMITS.client.op_bytes).unwrap();
        let decoded = crate::records::decode(current.as_slice(), &bounds).unwrap();
        assert_eq!(decoded, expected, "{variant:?} preserves all other facts");
    }
}

#[test]
fn an_earlier_finding_defaults_blame_clearing_and_many() {
    use super::fixtures;
    use crate::records;
    use skein_codec::{
        layout::{Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary::Stored;

    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let mut before = family.current.to_owned();
    for field in [" blame: option FindingBlame = none", " cleared: option FindingCleared = none", " many: bool = false"]
    {
        assert!(before.contains(field));
        before = before.replacen(field, "", 1);
    }
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"finding")).unwrap();
    let new_root = new.root(b"Row", Some(b"finding")).unwrap();
    let write_old = Plan::derive_dropping(&new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let mut row = fixtures::finding();
    let mut earlier = skein_lib::Buffer::with_capacity(1_048_576);
    write_old.convert(&records::encode(&Stored::Finding(row.clone())), &mut earlier, &mut work).unwrap();
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("finding additions have defaults");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(1_048_576);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let decoded = records::decode(current.as_slice(), &bounds).unwrap();
    row.blame = None;
    row.cleared = None;
    assert!(!row.signature.as_ref().unwrap().many);
    assert_eq!(decoded, Stored::Finding(row), "an old finding keeps its signature, without blame or clearing");
}

fn golden(text: &str, name: &str) -> Box<[u8]> {
    skein_codegen::goldens(&skein_codegen::parse(text).expect("current store schema"))
        .into_iter()
        .find(|value| value.name == name)
        .expect("named fixed-rule Change row")
        .bytes
        .into_boxed_slice()
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
                person: None,
                goal_title: None,
                draft: None,
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
