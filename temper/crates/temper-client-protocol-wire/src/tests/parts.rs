//! Parts and row additions (client/forge/parts.md, 3; records.md, A project's changes; domain/trial.md, 6).
use super::{fixtures, v1};
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
    let report =
        v1::GoalReport::new(&limits, v1::GoalReportParts { headline: bytes(512), at: 37 }).expect("headline at bound");
    let part =
        v1::GoalPart::new(&limits, v1::GoalPartParts { report: Some(report), milestone: Some(12) }).expect("goal part");
    let mut writer = Writer::new(usize::try_from(part.measure()).expect("size fits"));
    part.encode(&mut writer).expect("encode goal");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::GoalPart::decode(&limits, &mut reader).expect("decode goal"), part);
    assert!(reader.is_empty());
    assert_eq!(
        v1::GoalReport::new(&limits, v1::GoalReportParts { headline: bytes(513), at: 37 })
            .expect_err("headline past bound"),
        bound(v1::Path::GoalReportHeadline)
    );
}

#[test]
fn chat_part_keeps_a_document_at_its_bound() {
    let limits = fixtures::limits();
    let part = v1::ChatPart::new(&limits, v1::ChatPartParts { document: Some(bytes(512)) }).expect("document at bound");
    let mut writer = Writer::new(usize::try_from(part.measure()).expect("size fits"));
    part.encode(&mut writer).expect("encode chat");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::ChatPart::decode(&limits, &mut reader).expect("decode chat"), part);
    assert!(reader.is_empty());
    assert_eq!(
        v1::ChatPart::new(&limits, v1::ChatPartParts { document: Some(bytes(513)) }).expect_err("document past bound"),
        bound(v1::Path::ChatPartDocument)
    );
}

fn milestone(goals: List<v1::MilestoneGoal>) -> v1::MilestonePartParts {
    let totals = v1::MilestoneTotals::new(
        &fixtures::limits(),
        v1::MilestoneTotalsParts {
            task: 12,
            done: 1,
            failed: 2,
            closed: 3,
            live: 16,
            ended: None,
            last_landing: None,
            spent: 700,
            reds: 3,
        },
    )
    .expect("totals");
    v1::MilestonePartParts { whole: Some(29), totals, goals, more: 6 }
}

#[test]
fn milestone_part_keeps_sixteen_goals_at_their_byte_bounds() {
    let limits = fixtures::limits();
    let goal =
        v1::MilestoneGoal::new(&limits, v1::MilestoneGoalParts { line: bytes(16384), headline: Some(bytes(512)) })
            .expect("goal at both bounds");
    let mut goals = List::with_capacity(16);
    for _ in 0_u32..16 {
        goals.push(goal.clone()).expect("sixteen slots");
    }
    let part = v1::MilestonePart::new(&limits, milestone(goals)).expect("sixteen goals");
    let answer = v1::MilestoneAnswer::new(
        &limits,
        v1::MilestoneAnswerParts { result: v1::MilestoneResult::Totals(part.totals().clone()) },
    )
    .unwrap();
    assert_eq!(crate::decode_milestone_answer(&crate::milestone_answer(&answer), &limits), Some(answer));
    let mut writer = Writer::new(usize::try_from(part.measure()).expect("size fits"));
    part.encode(&mut writer).expect("encode milestone");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::MilestonePart::decode(&limits, &mut reader).expect("decode milestone"), part);
    assert!(reader.is_empty());
    let mut goals = List::with_capacity(17);
    for _ in 0_u32..17 {
        goals.push(goal.clone()).expect("seventeen slots");
    }
    assert_eq!(
        v1::MilestonePart::new(&limits, milestone(goals)).expect_err("seventeen goals"),
        bound(v1::Path::MilestonePartGoals)
    );
    assert_eq!(
        v1::MilestoneGoal::new(&limits, v1::MilestoneGoalParts { line: bytes(16385), headline: None })
            .expect_err("line past bound"),
        bound(v1::Path::MilestoneGoalLine)
    );
    assert_eq!(
        v1::MilestoneGoal::new(&limits, v1::MilestoneGoalParts { line: bytes(1), headline: Some(bytes(513)) })
            .expect_err("headline past bound"),
        bound(v1::Path::MilestoneGoalHeadline)
    );
}

#[test]
fn trial_row_keeps_run_wait_at_its_bound() {
    let limits = fixtures::limits();
    let parts = v1::TrialRowParts {
        account: 1,
        task: Some(3),
        took: None,
        step: v1::TrialStep::Answers,
        state: v1::TrialState::Running,
        run_wait: Some(bytes(1024)),
    };
    let row = v1::TrialRow::new(&limits, parts.clone()).expect("run wait at bound");
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("size fits"));
    row.encode(&mut writer).expect("encode trial");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::TrialRow::decode(&limits, &mut reader).expect("decode trial"), row);
    assert!(reader.is_empty());
    assert_eq!(
        v1::TrialRow::new(&limits, v1::TrialRowParts { run_wait: Some(bytes(1025)), ..parts })
            .expect_err("run wait past bound"),
        bound(v1::Path::TrialRowRunWait)
    );
}

#[test]
fn landed_rows_keep_provenance_and_signer_names_at_their_bounds() {
    let limits = fixtures::limits();
    let from =
        v1::ChangeFrom::new(&limits, v1::ChangeFromParts { task: 41, title: bytes(256) }).expect("provenance at bound");
    let sign_parts = v1::ChangesSignedParts { party: 7, head: [1; 32], at: 5, name: Some(bytes(256)) };
    let sign = v1::ChangesSigned::new(&limits, sign_parts.clone()).expect("signer name at bound");
    let mut signed = List::with_capacity(1);
    signed.push(sign).expect("one sign-off");
    let row = v1::ChangesLanded::new(
        &limits,
        v1::ChangesLandedParts {
            task: 6,
            title: bytes(256),
            at: 11,
            commit: [2; 32],
            default_branch: true,
            signed,
            goal: Some(from.clone()),
            chat: Some(from.clone()),
            milestone: Some(from),
        },
    )
    .expect("all provenance");
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("size fits"));
    row.encode(&mut writer).expect("encode landed");
    let encoded = writer.finish();
    let mut reader = Reader::new(&encoded);
    assert_eq!(v1::ChangesLanded::decode(&limits, &mut reader).expect("decode landed"), row);
    assert!(reader.is_empty());
    assert_eq!(
        v1::ChangeFrom::new(&limits, v1::ChangeFromParts { task: 41, title: bytes(257) })
            .expect_err("provenance title past bound"),
        bound(v1::Path::ChangeFromTitle)
    );
    assert_eq!(
        v1::ChangesSigned::new(&limits, v1::ChangesSignedParts { name: Some(bytes(257)), ..sign_parts })
            .expect_err("signer name past bound"),
        bound(v1::Path::ChangesSignedName)
    );
}
