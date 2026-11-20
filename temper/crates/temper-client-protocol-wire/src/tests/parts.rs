//! Parts and row additions (client/forge/parts.md, 3; records.md, A project's changes; domain/trial.md, 6).
use super::{fixtures, v1};
use crate::v1::Record;
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};

fn bytes(count: u32) -> Box<[u8]> {
    let mut bytes = List::with_capacity(count);
    for _ in 0..count {
        bytes.push(b'x').expect("counted room");
    }
    bytes.into_boxed()
}

fn bound(path: v1::Path) -> v1::Problem {
    v1::Problem { path, reason: skein_codec::Reason::Bound }
}

#[test]
fn goal_part_keeps_a_report_at_its_headline_bound() {
    let limits = fixtures::limits();
    let report = v1::GoalReport { headline: bytes(512), at: 37 };
    let part = v1::GoalPart { report: Some(report), milestone: Some(12) };
    let mut writer =
        Writer::new(usize::try_from(part.measure(&v1::CEILINGS).expect("within ceilings")).expect("size fits"));
    part.write(&v1::CEILINGS, &mut writer).expect("encode goal");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::GoalPart::decode(&limits, &mut reader).expect("decode goal"), part);
    assert!(reader.is_empty());
    assert_eq!(
        (v1::GoalReport { headline: bytes(513), at: 37 }).encode(&limits).expect_err("headline past bound"),
        bound(v1::paths::GOAL_REPORT_HEADLINE)
    );
}

#[test]
fn chat_part_keeps_a_document_at_its_bound() {
    let limits = fixtures::limits();
    let part = v1::ChatPart { document: Some(bytes(512)) };
    let mut writer =
        Writer::new(usize::try_from(part.measure(&v1::CEILINGS).expect("within ceilings")).expect("size fits"));
    part.write(&v1::CEILINGS, &mut writer).expect("encode chat");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::ChatPart::decode(&limits, &mut reader).expect("decode chat"), part);
    assert!(reader.is_empty());
    assert_eq!(
        (v1::ChatPart { document: Some(bytes(513)) }).encode(&limits).expect_err("document past bound"),
        bound(v1::paths::CHAT_PART_DOCUMENT)
    );
}

fn milestone(goals: List<v1::MilestoneGoal>) -> v1::MilestonePart {
    let totals = v1::MilestoneTotals {
        task: 12,
        done: 1,
        failed: 2,
        closed: 3,
        live: 16,
        ended: None,
        last_landing: None,
        spent: 700,
    };
    v1::MilestonePart { whole: Some(29), totals, goals, more: 6 }
}

#[test]
fn milestone_part_keeps_sixteen_goals_at_their_byte_bounds() {
    let limits = fixtures::limits();
    let goal = v1::MilestoneGoal { line: bytes(16384), headline: Some(bytes(512)) };
    let mut goals = List::with_capacity(16);
    for _ in 0_u32..16 {
        goals.push(goal.clone()).expect("sixteen slots");
    }
    let part = milestone(goals);
    let mut writer =
        Writer::new(usize::try_from(part.measure(&v1::CEILINGS).expect("within ceilings")).expect("size fits"));
    part.write(&v1::CEILINGS, &mut writer).expect("encode milestone");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::MilestonePart::decode(&limits, &mut reader).expect("decode milestone"), part);
    assert!(reader.is_empty());
    let mut goals = List::with_capacity(17);
    for _ in 0_u32..17 {
        goals.push(goal.clone()).expect("seventeen slots");
    }
    assert_eq!(
        (milestone(goals)).encode(&limits).expect_err("seventeen goals"),
        bound(v1::paths::MILESTONE_PART_GOALS)
    );
    assert_eq!(
        (v1::MilestoneGoal { line: bytes(16385), headline: None }).encode(&limits).expect_err("line past bound"),
        bound(v1::paths::MILESTONE_GOAL_LINE)
    );
    assert_eq!(
        (v1::MilestoneGoal { line: bytes(1), headline: Some(bytes(513)) })
            .encode(&limits)
            .expect_err("headline past bound"),
        bound(v1::paths::MILESTONE_GOAL_HEADLINE)
    );
}

#[test]
fn trial_row_keeps_run_wait_at_its_bound() {
    let limits = fixtures::limits();
    let parts = v1::TrialRow {
        account: 1,
        task: Some(3),
        took: None,
        step: v1::TrialStep::Answers,
        state: v1::TrialState::Running,
        run_wait: Some(bytes(1024)),
    };
    let row = parts.clone();
    let mut writer =
        Writer::new(usize::try_from(row.measure(&v1::CEILINGS).expect("within ceilings")).expect("size fits"));
    row.write(&v1::CEILINGS, &mut writer).expect("encode trial");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::TrialRow::decode(&limits, &mut reader).expect("decode trial"), row);
    assert!(reader.is_empty());
    assert_eq!(
        (v1::TrialRow { run_wait: Some(bytes(1025)), ..parts }).encode(&limits).expect_err("run wait past bound"),
        bound(v1::paths::TRIAL_ROW_RUN_WAIT)
    );
}

#[test]
fn landed_rows_keep_provenance_and_signer_names_at_their_bounds() {
    let limits = fixtures::limits();
    let from = v1::ChangeFrom { task: 41, title: bytes(256) };
    let sign_parts = v1::ChangesSigned { party: 7, head: [1; 32], at: 5, name: Some(bytes(256)) };
    let sign = sign_parts.clone();
    let mut signed = List::with_capacity(1);
    signed.push(sign).expect("one sign-off");
    let row = v1::ChangesLanded {
        task: 6,
        title: bytes(256),
        at: 11,
        commit: [2; 32],
        default_branch: true,
        signed,
        goal: Some(from.clone()),
        chat: Some(from.clone()),
        milestone: Some(from),
    };
    let mut writer =
        Writer::new(usize::try_from(row.measure(&v1::CEILINGS).expect("within ceilings")).expect("size fits"));
    row.write(&v1::CEILINGS, &mut writer).expect("encode landed");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::ChangesLanded::decode(&limits, &mut reader).expect("decode landed"), row);
    assert!(reader.is_empty());
    assert_eq!(
        (v1::ChangeFrom { task: 41, title: bytes(257) }).encode(&limits).expect_err("provenance title past bound"),
        bound(v1::paths::CHANGE_FROM_TITLE)
    );
    assert_eq!(
        (v1::ChangesSigned { name: Some(bytes(257)), ..sign_parts })
            .encode(&limits)
            .expect_err("signer name past bound"),
        bound(v1::paths::CHANGES_SIGNED_NAME)
    );
}
