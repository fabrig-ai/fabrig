//! Registered layouts derive field defaults (jig protocol/store/layouts.md, 4 and 5).
#![expect(clippy::disallowed_methods, reason = "ordinary Rust schema contract tests")]

use skein_codec::Record;
#[test]
fn an_earlier_finding_reply_keeps_its_result_and_defaults_repair_and_blame_context() {
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary as d;
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let reply = family.current.lines().find(|line| line.starts_with("record FindingReply {")).unwrap();
    let before = family
        .current
        .replace(reply, "record FindingReply { project: u32 number: u32 area: option bytes 32 result: FindingResult }")
        .replace(", repaired: Number, blamed", "");
    assert_ne!(before, family.current);
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"call")).unwrap();
    let new_root = new.root(b"Row", Some(b"call")).unwrap();
    let stored = d::Stored::Call {
        key: d::NamedCall { task: 40, attempt: 1, completion: 2, position: 3 },
        answer: d::CallPayload::Finding(Box::new(d::FindingReply {
            project: 3,
            number: 12,
            area: Some(b"tests".as_slice().into()),
            result: Ok(d::FindingDone::Routed(99)),
            more: Box::new([]),
            landing: None,
        })),
    };
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    write_old.convert(&crate::records::encode(&stored), &mut earlier, &mut work).unwrap();
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("reply additions default");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(4096);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&super::fixtures::LIMITS, super::fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(crate::records::decode(current.as_slice(), &bounds), Ok(stored));
}

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
fn an_earlier_closer_defaults_to_an_unread_report() {
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_closer::{Closer, ReportCheck, relay::Relay};
    use temper_engine_domain_forge_vocabulary::{CloserRow, Stored};
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before =
        family.current.replace("report_check: ClosingReportCheck = unread goal_left: option bytes 1048576 = none ", "");
    assert_ne!(before, family.current);
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"closer")).unwrap();
    let new_root = new.root(b"Row", Some(b"closer")).unwrap();
    let mut row = CloserRow {
        task: 8,
        goal: 7,
        closer: Closer::new(),
        making: None,
        close_changes: Box::new([]),
        superseded: Box::new([]),
        landing_changes: Box::new([]),
        relay: Relay::new(),
        done: false,
    };
    let expected = Stored::Closer(row.clone());
    assert_eq!(row.closer.report_check, ReportCheck::Unread);
    assert_eq!(row.closer.goal_left, None);
    row.closer.report_check = ReportCheck::Held;
    row.closer.goal_left = Some(b"an earlier layout cannot retain this".as_slice().into());
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    write_old.convert(&crate::records::encode(&Stored::Closer(row)), &mut earlier, &mut work).unwrap();
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("closer additions default");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(4096);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let limits = &super::fixtures::LIMITS;
    let bounds = crate::bounds(limits, limits.client.op_bytes).unwrap();
    assert_eq!(crate::records::decode(current.as_slice(), &bounds), Ok(expected));
}

#[test]
fn a_landing_layout_without_wait_defaults_it_to_cleared() {
    landing_layout_defaults(" wait: SettingsLandingWait = cleared");
}
#[test]
fn a_landing_layout_without_repairs_defaults_it_to_none() {
    landing_layout_defaults(" repairs: option RepairSettings = none");
}
fn landing_layout_defaults(field: &str) {
    use super::fixtures;
    use crate::records;
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary::{Stored, settings as d};
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replacen(field, "", 1);
    assert_ne!(before, family.current, "the earlier landing omits the added field");
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"setting")).unwrap();
    let new_root = new.root(b"Row", Some(b"setting")).unwrap();
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
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
        Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("an added field defaults stored kind 4 values");
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
        (b"ChangeCold".as_slice(), fixed_value(family.current, "record_change_cold_smallest.bin")),
        (b"ChangeHot".as_slice(), fixed_value(family.current, "record_change_hot_smallest.bin")),
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
            assert!(row.source.is_none());
        } else {
            let row = crate::v25::ChangeHot::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice()))
                .unwrap();
            assert!(row.kept.is_none());
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
fn a_change_layout_without_measured_tips_defaults_to_none() {
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = super::dev_layouts::DEV;
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
    .expect("measured tips have a default in the registered layout");
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1024);
    let earlier = fixed_value(before, "record_change_hot_smallest.bin");
    plan.convert(&earlier, &mut out, &mut work).unwrap();
    let row =
        crate::v25::ChangeHot::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice())).unwrap();
    assert!(row.close_at.is_none());
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
    let current = fixed_value(family.current, "record_change_hot_smallest.bin");
    // The earlier row lacks the first option's absent byte.
    plan.convert(&current[1..], &mut out, &mut work).unwrap();
    let row =
        crate::v25::ChangeHot::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice())).unwrap();
    assert!(row.regate.is_none());
}

#[test]
fn a_repository_layout_without_landing_defaults_it_to_its_default_branch() {
    use super::fixtures;
    use crate::records;
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
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
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
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
        layout::{Mode, Plan, Workspace},
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
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
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
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary as d;
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let mut before = family.current.to_owned();
    let facts = "person: option u64 = none goal_title: option bytes 256 = none draft: option bytes 256 = none ";
    assert_eq!(before.matches(facts).count(), 2, "both logged members and cold changes carry goal facts");
    before = before.replace(facts, "");
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
                fixes: Box::new([]),
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
        let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
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
        layout::{Mode, Plan, Workspace},
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
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
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

#[test]
fn an_earlier_tier_defaults_its_last_judged_run_number_to_none() {
    use super::fixtures;
    use crate::records;
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary::Stored;
    let family = crate::OWNER_SET.families.first().expect("store family");
    let before = family.current.replace(" last_run: option u64 = none", "");
    assert_ne!(before, family.current);
    let old = schema::parse(before.as_bytes()).expect("earlier tier schema");
    let new = schema::parse(family.current.as_bytes()).expect("current tier schema");
    let old_root = old.root(b"Row", Some(b"tier")).expect("earlier tier root");
    let new_root = new.root(b"Row", Some(b"tier")).expect("current tier root");
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).expect("drop run number");
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let mut row = fixtures::tier_row();
    row.record.last_run = Some(7);
    let mut earlier = skein_lib::Buffer::with_capacity(1_048_576);
    write_old.convert(&records::encode(&Stored::Tier(row.clone())), &mut earlier, &mut work).expect("earlier bytes");
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("run number defaults");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(1_048_576);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).expect("default run number");
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("store bounds");
    row.record.last_run = None;
    assert!(records::decode(current.as_slice(), &bounds) == Ok(Stored::Tier(row)), "earlier tiers have no run number");
}

#[test]
fn an_earlier_finding_defaults_repair_and_blame_move() {
    use super::fixtures;
    use crate::records;
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary::{BlameStrength, Stored};

    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").expect("store family");
    let mut before = family.current.to_owned();
    for field in [" repair: option FindingRepair = none", " moved: option BlameMove = none"] {
        assert!(before.contains(field));
        before = before.replacen(field, "", 1);
    }
    before = before.replace(
        "enum BlameStrength { owned, suspected, none, moved }",
        "enum BlameStrength { owned, suspected, none }",
    );
    let old = schema::parse(before.as_bytes()).expect("earlier finding schema");
    let new = schema::parse(family.current.as_bytes()).expect("current schema");
    let old_root = old.root(b"Row", Some(b"finding")).expect("earlier finding root");
    let new_root = new.root(b"Row", Some(b"finding")).expect("current finding root");
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).expect("drop additions");
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let mut row = fixtures::finding();
    row.blame.as_mut().expect("blame").strength = BlameStrength::Owned;
    let mut earlier = skein_lib::Buffer::with_capacity(1_048_576);
    write_old.convert(&records::encode(&Stored::Finding(row.clone())), &mut earlier, &mut work).expect("earlier bytes");
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("repair and move default to none");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(1_048_576);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).expect("default additions");
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("store bounds");
    row.repair = None;
    row.blame.as_mut().expect("blame").moved = None;
    assert!(
        records::decode(current.as_slice(), &bounds) == Ok(Stored::Finding(row)),
        "an earlier finding reads with no repair or move"
    );
}

fn fixed_value(text: &'static str, name: &str) -> Box<[u8]> {
    skein_codegen::values(text)
        .fixed
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
        layout::{Mode, Plan, Workspace},
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
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
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
        layout::{Mode, Plan, Workspace},
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
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let stored = d::Stored::LandingLog(d::LandingLog {
        repository: skein_forge::api::Repository { forge: 1, repository: 2 },
        landings: Box::new([d::LoggedLanding {
            commit: [7; 32],
            branch: b"main".as_slice().into(),
            at: skein_lib::Wall::from_nanos(123),
            changes: Box::new([d::LoggedChange {
                fixes: Box::new([]),
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

#[test]
fn an_earlier_landing_member_defaults_fixes() {
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary as d;
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replacen(" fixes: list 8 u32 = []", "", 1);
    assert_ne!(before, family.current);
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"landing_log")).unwrap();
    let new_root = new.root(b"Row", Some(b"landing_log")).unwrap();
    let mut stored = d::Stored::LandingLog(d::LandingLog {
        repository: skein_forge::api::Repository { forge: 1, repository: 2 },
        landings: Box::new([d::LoggedLanding {
            commit: [7; 32],
            branch: b"main".as_slice().into(),
            at: skein_lib::Wall::from_nanos(123),
            tiers: Box::new([]),
            changes: Box::new([d::LoggedChange {
                fixes: Box::new([7, 12]),
                signed: Box::new([]),
                task: 51,
                pull: Some(101),
                title: b"member".as_slice().into(),
                goal: Some(42),
                milestone: Some(40),
                person: Some(7),
                goal_title: Some(b"Goal".as_slice().into()),
                draft: None,
            }]),
        }]),
    });
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    write_old.convert(&crate::records::encode(&stored), &mut earlier, &mut work).unwrap();
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("fixes default to empty");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(4096);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&super::fixtures::LIMITS, super::fixtures::LIMITS.client.op_bytes).unwrap();
    let decoded = crate::records::decode(current.as_slice(), &bounds).unwrap();
    let d::Stored::LandingLog(log) = &mut stored else { panic!("landing log") };
    log.landings[0].changes[0].fixes = Box::new([]);
    assert_eq!(decoded, stored, "the previous member layout retains all other facts but names no fixes");
}

#[test]
fn an_earlier_tier_defaults_waiting_and_unkept() {
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_vocabulary::Stored;
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replace(" waiting: list 8 VerdictFiling = [] unkept: u32 = 0", "");
    assert_ne!(before, family.current);
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"tier")).unwrap();
    let new_root = new.root(b"Row", Some(b"tier")).unwrap();
    let write_old = Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&write_old));
    let mut row = super::fixtures::tier_row();
    row.unkept = 12;
    let mut earlier = skein_lib::Buffer::with_capacity(1_048_576);
    write_old.convert(&crate::records::encode(&Stored::Tier(row.clone())), &mut earlier, &mut work).unwrap();
    let read_old = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("tier additions have defaults");
    assert!(work.fit(&read_old));
    let mut current = skein_lib::Buffer::with_capacity(1_048_576);
    read_old.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&super::fixtures::LIMITS, super::fixtures::LIMITS.client.op_bytes).unwrap();
    row.unkept = 0;
    let decoded = crate::records::decode(current.as_slice(), &bounds).unwrap();
    let Stored::Tier(tier) = &decoded else { panic!("tier layout") };
    assert_eq!(tier.waiting.len(), 0);
    assert_eq!(tier.unkept, 0);
    assert_eq!(decoded, Stored::Tier(row));
}

#[test]
fn a_held_answer_in_an_earlier_layout_defaults_to_no_question_or_blocker() {
    use super::fixtures;
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use temper_engine_domain_forge_change::{Blocked, Open, Route, Stands, State};
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let fields = " question: u64 = 0 dependency: option u64 = none since: option u64 = none";
    let before = family.current.replacen(fields, "", 1);
    assert_ne!(before, family.current);
    let old = schema::parse(before.as_bytes()).unwrap();
    let current = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"change_hot")).unwrap();
    let current_root = current.root(b"Row", Some(b"change_hot")).unwrap();
    let mut row = fixtures::change_row(State::Producing { requested: true });
    row.questions.route = Route::Up;
    row.questions.open = Box::new([Open {
        asker: 9,
        number: 21,
        line: b"Which path?".as_slice().into(),
        words: b"Which path?".as_slice().into(),
        stands: Stands::Held {
            answer: b"Use the path.".as_slice().into(),
            question: 50,
            blocked: Some(Blocked { dependency: Some(7), since: skein_lib::Wall::from_nanos(9) }),
        },
    }]);
    let downgrade = Plan::between(Mode::Read, &current, current_root, &[], &old, old_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&downgrade));
    let mut earlier = skein_lib::Buffer::with_capacity(4096);
    downgrade.convert(&crate::records::encode(&fixtures::change_hot(row.clone())), &mut earlier, &mut work).unwrap();
    let upgrade = Plan::derive(&old, old_root, &[], &current, current_root, &[]).expect("held fields default");
    assert!(work.fit(&upgrade));
    let mut now = skein_lib::Buffer::with_capacity(4096);
    upgrade.convert(earlier.as_slice(), &mut now, &mut work).unwrap();
    row.questions.open[0].stands =
        Stands::Held { answer: b"Use the path.".as_slice().into(), question: 0, blocked: None };
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(crate::records::decode(now.as_slice(), &bounds), Ok(fixtures::change_hot(row)));
}
