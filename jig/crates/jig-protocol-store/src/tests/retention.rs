//! Retention's rows and upgrade jobs (protocol/store/retention.md, 3–4 and 8).
#![expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "ordinary Rust retention contract tests")]
use super::fixtures;
use crate::{JobPage, JobRow, JobWrite, codec, jobs, keys};
use jig_domain_core_vocabulary::{self as core, CoreRecord, Record};
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
fn page(records: &[Record], end: bool) -> JobPage {
    let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
    JobPage {
        rows: records
            .iter()
            .map(|record| JobRow {
                key: keys::encode(&record.key(), fixtures::KEY_LIMIT).unwrap(),
                value: codec::encode(record, &limits, &[]).unwrap(),
            })
            .collect(),
        end,
        layout: None,
    }
}
fn answer(job: &str, page: &JobPage) -> Vec<t::Stored> {
    let answer = jobs::page(job, page).unwrap();
    assert_eq!(answer.through, None, "whole page consumed");
    answer
        .writes
        .iter()
        .map(|write| {
            let JobWrite::Put { key, value } = write else { panic!("neither job erases anything") };
            let row = codec::tasks::decode(value, &crate::tasks_v13::CEILINGS, &[]).unwrap();
            assert_eq!(keys::decode(key), Some(core::Key::Tasks(row.key())));
            row
        })
        .collect()
}
#[test]
fn retention_rows_job_puts_each_summary_and_next_stage_for_the_whole_page() {
    let mut turned = ended();
    turned.ended_at = Some(Wall::from_nanos(1000));
    turned.ever_turned = true;
    let mut unturned = turned.clone();
    unturned.number = 13;
    unturned.ended_at = None;
    unturned.changed = Wall::from_nanos(1100);
    unturned.ever_turned = false;
    let records = [Record::Tasks(t::Stored::Ended(turned.clone())), Record::Tasks(t::Stored::Ended(unturned.clone()))];
    let expected = [
        t::Stored::Summary(Box::new(t::TaskSummary::of(&turned))),
        t::Stored::Retention(t::RetentionRow {
            stage: t::RetentionStage::Turns,
            since: Wall::from_nanos(1000),
            number: 12,
        }),
        t::Stored::Summary(Box::new(t::TaskSummary::of(&unturned))),
        t::Stored::Retention(t::RetentionRow {
            stage: t::RetentionStage::Record,
            since: Wall::from_nanos(1100),
            number: 13,
        }),
    ];
    assert_eq!(answer("retention-rows", &page(&records, false)), expected);
    assert_eq!(answer("retention-rows", &page(&[], true)), []);
    let job = crate::JIG_STORE_PATCH_SET.jobs.iter().find(|job| job.name == "retention-rows").unwrap();
    assert_eq!(job.keys[0].from, &[1, 1, 6]);
    assert_eq!(job.keys[0].to, &[1, 1, 7]);
    assert!(job.converts.is_none());
}
#[test]
fn retention_proposals_job_puts_each_decisions_time_for_the_whole_page() {
    use jig_domain_people::{ProposalChoice, Proposer};
    let mut records = Vec::new();
    let mut expected = Vec::new();
    for (number, choice) in
        [(21, ProposalChoice::Accepted), (22, ProposalChoice::Rejected), (23, ProposalChoice::Withdrawn)]
    {
        let mut row = fixtures::core_rows::proposal();
        row.proposer = Proposer::Party(9);
        row.proposal = number;
        row.choice = choice;
        row.at = Wall::from_nanos(number * 100);
        expected.push(t::Stored::Retention(t::RetentionRow {
            stage: t::RetentionStage::Proposal,
            since: row.at,
            number,
        }));
        records.push(Record::Core(CoreRecord::ProposalDecision(row)));
    }
    assert_eq!(answer("retention-proposals", &page(&records, false)), expected);
    assert_eq!(answer("retention-proposals", &page(&[], true)), []);
    let job = crate::JIG_STORE_PATCH_SET.jobs.iter().find(|job| job.name == "retention-proposals").unwrap();
    assert_eq!(job.keys[0].from, &[1, 0, 6, 1]);
    assert_eq!(job.keys[0].to, &[1, 0, 6, 2]);
    assert!(job.converts.is_none());
}
