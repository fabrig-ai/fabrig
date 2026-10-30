use super::fixtures;
use crate::{Undecodable, application, keys, records};
use temper_engine_domain_forge as forge;

#[test]
fn the_application_sizes_are_the_largest_key_and_row() {
    // Change's eight remarks hold 4096 bytes each; eight failure counts add 100 bytes.
    // Authority causes add 42 bytes to each of eight holds, drift and released.
    // Way and landing add ten bytes.
    // Terminal CI adds a presence byte, a 32-byte head and a one-byte status.
    // The effect account adds 1 presence + 8 entry + 4 length + 512 words = 525 bytes.
    // A producer failure adds 1 presence + 4 length + 512 words = 517 bytes.
    // Result evidence adds four 4096-byte prefixes, cut counts, a head and repair count.
    assert_eq!(
        application(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes),
        Some(jig_protocol_store::sizes::Application { version: 10, key: 511, value: 53_492 })
    );
}
#[test]
fn limits_past_a_ceiling_have_no_sizes() {
    for (name, field, value) in [("name", 0_u8, 4097_u32), ("answer", 1, 1_048_577)] {
        let mut limits = fixtures::LIMITS;
        match field {
            0 => limits.name_bytes = value,
            1 => limits.client.answer_bytes = value,
            _ => panic!("table field"),
        }
        assert_eq!(application(&limits, limits.client.op_bytes), None, "{name}");
    }
}
#[test]
fn a_row_whose_key_disagrees_with_its_record_is_undecodable() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let row = forge::Stored::Landed { commit: [1; 32], task: 2 };
    let value = records::encode(&row);
    for (name, other) in [
        ("right key", forge::Key::Landed([1; 32])),
        ("wrong commit", forge::Key::Landed([9; 32])),
        ("wrong family", forge::Key::Change(2)),
    ] {
        let key = keys::encode(&other, u32::MAX).expect("a fixed key fits");
        if name == "right key" {
            assert_eq!(records::row(&key, &value, &bounds), Ok(row.clone()));
        } else {
            assert_eq!(records::row(&key, &value, &bounds), Err(Undecodable), "{name}");
        }
    }
    assert_eq!(records::row(&[], &value, &bounds), Err(Undecodable), "malformed key");
    let key = keys::encode(&forge::Key::Landed([1; 32]), u32::MAX).expect("a fixed key fits");
    assert_eq!(records::row(&key, &[], &bounds), Err(Undecodable), "malformed value");
}
#[test]
fn the_store_version_is_ten() {
    assert_eq!(crate::STORE_VERSION, 10);
}

#[test]
fn demo_row_sizes_fit_the_existing_application_ceiling() {
    use crate::v10 as w;
    assert_eq!(w::Demo::encoded_bound(&w::CEILINGS), Some(3));
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for state in [forge::DemoState::Running, forge::DemoState::Ended] {
        let row = forge::Stored::Demo(forge::Demo { forge: u16::MAX, state });
        let value = records::encode(&row);
        let key = keys::encode(&forge::stored_key(&row), u32::MAX).expect("key");
        assert_eq!(key.len(), 9);
        assert_eq!(value.len(), 6);
        assert_eq!(records::row(&key, &value, &bounds), Ok(row));
    }
}

#[test]
fn plan_text_uses_the_body_bound_without_changing_the_format() {
    // Eight items grow from 256 bytes of text each to 512 bytes each.
    const EXTRA_PLAN_BYTES: u64 = 2048;
    // Decode peak 5,888 bytes, measured 2026-10-29; plus a quarter, rounded up to two significant digits.
    const ISSUE_ROW_BEFORE_HEAP_CEILING: u64 = 7_400;
    // Decode peak 7,936 bytes, measured 2026-10-29; plus a quarter, rounded up to two significant digits.
    const ISSUE_ROW_AFTER_HEAP_CEILING: u64 = 10_000;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let after = *bounds.limits();
    let before = crate::v10::Limits { plan_item_text: fixtures::LIMITS.issue_policy.comment_bytes, ..after };
    let old_view = crate::v10::GoalView::encoded_bound(&before).expect("old view size");
    let new_view = crate::v10::GoalView::encoded_bound(&after).expect("new view size");
    assert_eq!(new_view - old_view, EXTRA_PLAN_BYTES);
    // The schema's fixed framing plus the measured view; ChangeRow still dominates Row.
    assert_eq!(old_view, 5238);
    assert_eq!(new_view, 7286);
    assert_eq!(crate::v10::IssueRow::encoded_bound(&before), Some(5657));
    assert_eq!(crate::v10::IssueRow::encoded_bound(&after), Some(7705));
    assert_eq!(crate::v10::IssueRow::decoded_bound(&before), Some(5888));
    assert_eq!(crate::v10::IssueRow::decoded_bound(&after), Some(7936));
    for (name, limits, ceiling) in
        [("before", before, ISSUE_ROW_BEFORE_HEAP_CEILING), ("after", after, ISSUE_ROW_AFTER_HEAP_CEILING)]
    {
        let row = full_issue_row(&limits);
        let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).expect("row length"));
        row.encode(&mut writer).expect("measured room");
        let encoded = writer.finish();
        assert_eq!(
            Some(u64::try_from(encoded.len()).expect("encoded length")),
            crate::v10::IssueRow::encoded_bound(&limits)
        );
        let meter = skein_world::domain::heap::Meter::new();
        meter.start();
        let decoded =
            crate::v10::IssueRow::decode(&limits, &mut skein_lib::Reader::new(&encoded)).expect("full issue row");
        let peak = meter.end().peak();
        assert!(peak <= ceiling, "{name}: {peak} bytes exceeds {ceiling}");
        assert_eq!(decoded, row);
    }
    assert_eq!(crate::v10::Row::encoded_bound(&before), Some(53_492));
    assert_eq!(crate::v10::Row::encoded_bound(&after), Some(53_492));
}

fn full_issue_row(limits: &crate::v10::Limits) -> crate::v10::IssueRow {
    use crate::v10 as w;
    use skein_lib::{List, bytes};
    let key = w::MilestoneKey::Lifecycle(
        w::Lifecycle::new(limits, w::LifecycleParts { task: u64::MAX, position: u64::MAX }).expect("fixed key"),
    );
    let mut projected = List::with_capacity(limits.projected_milestones);
    for _ in 0..limits.projected_milestones {
        projected.push(key.clone()).expect("milestone room");
    }
    let state = w::Projected::new(
        limits,
        w::ProjectedParts {
            opened: true,
            digest: Some([u8::MAX; 32]),
            last_body: Some(u64::MAX),
            body_revision: u64::MAX,
            milestones: projected,
            closed: true,
        },
    )
    .expect("full projection");
    let mut plan = List::with_capacity(limits.goal_view_plan);
    for _ in 0..limits.goal_view_plan {
        plan.push(
            w::PlanItem::new(
                limits,
                w::PlanItemParts {
                    text: bytes::zeroed(usize::try_from(limits.plan_item_text).expect("u32 length")),
                    phase: w::Phase::Done,
                },
            )
            .expect("full plan item"),
        )
        .expect("plan room");
    }
    let mut milestones = List::with_capacity(limits.goal_view_milestones);
    for _ in 0..limits.goal_view_milestones {
        milestones
            .push(
                w::Milestone::new(
                    limits,
                    w::MilestoneParts {
                        key: key.clone(),
                        text: bytes::zeroed(usize::try_from(limits.milestone_text).expect("u32 length")),
                    },
                )
                .expect("full milestone"),
            )
            .expect("milestone room");
    }
    let desired = w::GoalView::new(
        limits,
        w::GoalViewParts {
            phase: w::Phase::Done,
            goal: u64::MAX,
            repository: u64::MAX,
            title: bytes::zeroed(usize::try_from(limits.goal_view_title).expect("u32 length")),
            goal_text: bytes::zeroed(usize::try_from(limits.goal_view_goal_text).expect("u32 length")),
            plan,
            milestones,
            finished: Some(bytes::zeroed(usize::try_from(limits.goal_view_finished_some).expect("u32 length"))),
        },
    )
    .expect("full goal view");
    w::IssueRow::new(
        limits,
        w::IssueRowParts {
            goal: u64::MAX,
            repository: w::Repository::new(limits, w::RepositoryParts { forge: u16::MAX, repository: u32::MAX })
                .expect("fixed repository"),
            state: state.clone(),
            number: Some(u64::MAX),
            pending: Some(u64::MAX),
            before: Some(state),
            failed: true,
            desired: Some(desired),
        },
    )
    .expect("full issue row")
}
