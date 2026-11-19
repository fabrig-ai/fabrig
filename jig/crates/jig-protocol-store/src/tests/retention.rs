//! Retention's rows (protocol/store/retention.md, 3–4).
use super::fixtures;
use crate::codec;
use crate::tasks_v13::Record as _;
use jig_domain_core_vocabulary::Record;
use jig_domain_tasks as t;
use skein_lib::Wall;

fn ended() -> Box<t::TaskRecord> {
    let Record::Tasks(t::Stored::Ended(row)) = fixtures::ended_task(12) else { panic!("ended fixture") };
    row
}
fn round_trip(row: &t::Stored) -> usize {
    let limits = codec::tasks::limits(&fixtures::CORE_LIMITS.tasks).unwrap();
    let bytes = codec::tasks::encode(row, &limits, &[]).unwrap();
    assert_eq!(codec::tasks::decode(&bytes, &limits, &[]), Ok(row.clone()));
    bytes.len()
}
#[test]
fn summaries_and_retention_rows_round_trip_with_a_one_kib_summary_bound() {
    let mut summary = t::TaskSummary::of(&ended());
    summary.title = Box::new([b't'; 256]);
    summary.title_cut = true;
    summary.reason = Box::new([b'r'; 256]);
    summary.reason_cut = true;
    summary.ending = t::SummaryEnding::Cancelled { by: t::Party::Person(u64::MAX), at: Wall::from_nanos(u64::MAX) };
    summary.flags.ended_by = t::EndedBy::Closer(t::Closer { connector: u16::MAX, code: u32::MAX });
    summary.tracked = Some(u32::MAX);
    summary.executor = t::Executor::Person(t::PersonAddress::Person(u64::MAX));
    summary.deadline = Some(Wall::from_nanos(u64::MAX));
    summary.funder = t::Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX };
    for kind in [
        t::SummaryResultKind::Job,
        t::SummaryResultKind::Approved,
        t::SummaryResultKind::Report,
        t::SummaryResultKind::Verdict { code: u32::MAX },
        t::SummaryResultKind::Change { connector: u16::MAX, kind: u16::MAX, resource: u64::MAX },
        t::SummaryResultKind::NoChange,
        t::SummaryResultKind::Failed,
    ] {
        summary.result = Some(t::SummaryResult { kind, words: Box::new([b'w'; 256]), words_cut: true });
        let len = round_trip(&t::Stored::Summary(Box::new(summary.clone())));
        assert!(len <= 1024, "summary is {len} bytes");
    }
    let limits = codec::tasks::limits(&fixtures::CORE_LIMITS.tasks).unwrap();
    let bound = crate::tasks_v13::TaskSummary::encoded_bound(&limits).unwrap();
    assert!(bound + 7 <= 1024, "summary bound plus Row envelope: {bound} + 7");
    for stage in [
        t::RetentionStage::Turns,
        t::RetentionStage::Record,
        t::RetentionStage::Summary,
        t::RetentionStage::Forgotten,
        t::RetentionStage::Proposal,
    ] {
        round_trip(&t::Stored::Retention(t::RetentionRow {
            stage,
            since: Wall::from_nanos(u64::MAX),
            number: u64::MAX,
        }));
    }
    for ending in [t::SummaryEnding::Done, t::SummaryEnding::Failed] {
        summary.ending = ending;
        summary.result = None;
        round_trip(&t::Stored::Summary(Box::new(summary.clone())));
    }
}
