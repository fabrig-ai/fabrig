//! History records and the v20 patch (landing-history.md, 2, 4 and 7).
use super::{
    catch_up::{kept_golden, patched},
    fixtures,
};
use crate::{keys, records, v21};
use skein_lib::{Reader, Wall};
use temper_engine_domain_forge_vocabulary::{Stored, history::*};

fn check(row: Stored) {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let value = records::encode(&row);
    let key = keys::encode(&temper_engine_domain_forge_vocabulary::stored_key(&row), u32::MAX).unwrap();
    assert_eq!(records::row(&key, &value, &bounds), Ok(row.clone()));
    assert!(
        value.len()
            <= usize::try_from(crate::application(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap().value)
                .unwrap()
    );
}
fn event(what: What) -> Event {
    Event {
        project: 3,
        number: 7,
        at: Wall::from_nanos(81),
        source_at: Some(Wall::from_nanos(19)),
        change: 9,
        repository: fixtures::REPO,
        landing: Some(InLanding { task: 13, round: 2, unit: 3 }),
        head: Some([1; 32]),
        base: Some([2; 32]),
        operation: 17,
        attempt: 2,
        what,
    }
}
fn kinds() -> Box<[What]> {
    Box::new([
        What::Ready,
        What::Joined { line: Line::Queue, place: Some(2), priority: Some(-23) },
        What::Departed {
            line: Line::Landing,
            cause: Departure::Landing(temper_engine_domain_forge_landing::Leave::Conflict),
        },
        What::Selected,
        What::Update { style: skein_forge::api::UpdateStyle::Merge },
        What::Update { style: skein_forge::api::UpdateStyle::Rebase },
        What::Job { job: Job::Prepare, task: 61 },
        What::Job { job: Job::Merge, task: 62 },
        What::Conflict { with: With::Branch, files: 12 },
        What::Conflict { with: With::Earlier, files: 3 },
        What::Asked { ask: Ask::Judgement, task: 63, gate: Some(4) },
        What::Pushed,
        What::Delivered { checked: true },
        What::Delivered { checked: false },
        What::Paused { why: Pause::BranchRed },
        What::Paused { why: Pause::TierRed },
        What::Resumed,
        What::Formed {
            members: Box::new([
                Member { change: 7, unit: 0, head: [4; 32] },
                Member { change: 8, unit: 1, head: [5; 32] },
            ]),
            place: 2,
        },
        What::Gate { gate: 3, key: [9; 32] },
        What::Proved { proofs: Box::new([Proof { gate: 3, run: Some(31) }, Proof { gate: 4, run: None }]) },
        What::Proved { proofs: Box::new([]) },
        What::Search,
        What::Searched { green: Some([2; 32]), red: [4; 32], blamed: 3 },
        What::Searched { green: None, red: [1; 32], blamed: 0 },
        What::Probe,
        What::ProbeEnded { green: true },
        What::ProbeEnded { green: false },
        What::Turn { members: 2 },
        What::Landed,
        What::Recorded { state: RecordedState::Merged },
        What::Recorded { state: RecordedState::Closed },
        What::Ended { ending: Ending::NoChange },
        What::Ended { ending: Ending::Cancelled },
        What::Ended { ending: Ending::Failed },
        What::LandingEnded { outcome: LandingOutcome::Done },
        What::LandingEnded { outcome: LandingOutcome::Cancelled },
        What::LandingEnded { outcome: LandingOutcome::Held },
        What::Held { on: On::Change, reason: Box::new([b'x'; 256]), by: Some(By::Party(42)) },
        What::Released { on: On::Landing, reason: b"go on".as_slice().into(), by: Some(By::Task(43)) },
        What::Held { on: On::Landing, reason: Box::new([]), by: None },
    ])
}
#[test]
fn history_records_round_trip_every_kind_and_interval() {
    for what in kinds() {
        check(Stored::HistoryEvent(event(what)));
    }
    for why in [
        Unready::Held,
        Unready::HeadMoved,
        Unready::Refused,
        Unready::Dependency,
        Unready::Approval,
        Unready::Conflict,
        Unready::Closed,
    ] {
        check(Stored::HistoryEvent(event(What::Unready { why })));
    }
    for line in [Line::Queue, Line::Landing, Line::ForgeQueue] {
        check(Stored::HistoryEvent(event(What::Joined { line, place: None, priority: None })));
        for cause in [Departure::Unready, Departure::Conflict, Departure::Repair, Departure::Ejected] {
            check(Stored::HistoryEvent(event(What::Departed { line, cause })));
        }
    }
    for outcome in [UpdateOutcome::Updated, UpdateOutcome::Conflict, UpdateOutcome::Failed] {
        check(Stored::HistoryEvent(event(What::Updated { outcome })));
    }
    for job in [Job::Prepare, Job::Merge] {
        for outcome in [JobOutcome::Done, JobOutcome::Conflict, JobOutcome::Failed, JobOutcome::Lost] {
            check(Stored::HistoryEvent(event(What::JobEnded { job, outcome })));
        }
    }
    for ask in [Ask::Resolution, Ask::Repair, Ask::Judgement] {
        check(Stored::HistoryEvent(event(What::Asked { ask, task: 29, gate: None })));
        for outcome in
            [Answer::Pushed, Answer::Passed, Answer::Refused, Answer::Nothing, Answer::Failed, Answer::Cancelled]
        {
            check(Stored::HistoryEvent(event(What::Answered { ask, outcome })));
        }
    }
    for cause in [RoundCause::Turned, RoundCause::Left, RoundCause::Stale, RoundCause::Cancelled, RoundCause::Cut] {
        check(Stored::HistoryEvent(event(What::RoundEnded { cause })));
    }
    for verdict in
        [GateVerdict::Passed, GateVerdict::Failed, GateVerdict::Fault, GateVerdict::Timeout, GateVerdict::Superseded]
    {
        check(Stored::HistoryEvent(event(What::GateEnded { gate: 4, key: [3; 32], verdict })));
    }
    for verdict in [AfterVerdict::Verified, AfterVerdict::Red, AfterVerdict::Cleared, AfterVerdict::Uncleared] {
        check(Stored::HistoryEvent(event(What::After { verdict })));
    }
    check(Stored::HistoryEvent(Event { source_at: None, landing: None, head: None, base: None, ..event(What::Ready) }));
    check(Stored::HistoryProject(Project {
        project: 3,
        next: 41,
        kept: 7,
        whole_since: Some(Wall::from_nanos(23)),
        marks: marks(),
        sweeping: Some(19),
        removing: true,
    }));
    check(Stored::HistoryProject(Project {
        project: 3,
        next: 1,
        kept: 0,
        whole_since: None,
        marks: Box::new([]),
        sweeping: None,
        removing: false,
    }));
    for landing in [None, Some(29)] {
        check(Stored::HistoryChange(ChangeIndex { project: 3, change: 9, number: 7, landing }));
    }
    check(Stored::HistoryLanding(LandingIndex { project: 3, landing: 29, number: 7, change: 9 }));
    check(Stored::HistoryLast(Last { project: 3, number: 7, change: 9, at: Wall::from_nanos(31) }));
    let mut change = fixtures::change_row(temper_engine_domain_forge_change::State::Ended);
    for history in [
        None,
        Some(ChangeHistory { last: 7, ready: None, queued: None, paused: None, held: None, landing_held: None }),
        Some(ChangeHistory {
            last: 29,
            ready: Some(2),
            queued: Some(3),
            paused: Some(4),
            held: Some(5),
            landing_held: Some(6),
        }),
    ] {
        change.history = history;
        check(Stored::Change(Box::new(change.clone())));
    }
    assert_eq!(v21::CEILINGS.history_project_marks, MARKS);
    assert_eq!(v21::CEILINGS.history_hold_reason, REASON_BYTES);
    assert_eq!(
        crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap().limits().history_formed_members,
        fixtures::LIMITS.landing_members
    );
}

#[test]
fn the_v20_change_patch_adds_no_backdated_history() {
    let before = kept_golden(20, "record_change_row_full.bin");
    let bytes = patched(20, 9, b"ChangeRow", before.clone());
    let (job, rest) = bytes.as_ref().split_first().unwrap();
    assert_eq!(*job, 0, "no pending job");
    assert_eq!(rest.split_last(), Some((&0, before.as_ref())), "no backdated history");
    let after = v21::ChangeRow::decode(&v21::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    assert!(after.history().is_none());
    assert!(after.job().is_none());
    let old = crate::v20::ChangeRow::decode(&crate::v20::CEILINGS, &mut Reader::new(&before)).unwrap();
    assert_eq!(after.task(), old.task());
    assert_eq!(after.repository().forge(), old.repository().forge());
    assert_eq!(crate::STORE_VERSION, 21);
    assert_eq!(crate::patches::PATCHES.len(), 10);
}

fn marks() -> Box<[Mark]> {
    let mut marks = skein_lib::List::with_capacity(MARKS);
    for hour in 0..MARKS {
        marks
            .push(Mark {
                hour: Wall::from_nanos(u64::from(hour).checked_mul(HOUR_NS).unwrap()),
                first: u64::from(hour).checked_add(1).unwrap(),
            })
            .unwrap();
    }
    marks.into_boxed()
}
