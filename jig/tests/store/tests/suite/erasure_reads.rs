//! Core erasure pages on codec-written store rows (playground-reset.md, 5).
use jig_domain_core::{self as core, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_protocol_store::{Event, Request, Write};
use jig_store_world::{World, root};
use skein_lib::{Token, Wall};

const TASK: u64 = 10;
const PAGE_BYTES: u64 = 1_048_576;

fn ended(number: u64, project: u32) -> core::Record {
    let core::Record::Tasks(tasks::Stored::Live(mut task)) = root::live_task(number) else { panic!("live fixture") };
    task.project = project;
    task.run_since = None;
    task.phase = tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) });
    task.ended_at = Some(Wall::EPOCH);
    core::Record::Tasks(tasks::Stored::Ended(task))
}

fn page(
    world: &mut World,
    scope: reads::Scope,
    after: Option<reads::Position>,
    rows: u32,
    bytes: u64,
) -> (Box<[reads::Row]>, Option<reads::Position>) {
    let mut events = world
        .request(Request::Read {
            owner: Token::new(1),
            range: reads::Range { party: u64::MAX, scope, after, backwards: false, rows, bytes },
        })
        .into_iter();
    let Some(Event::Read { result: reads::Loaded::Page { rows, next }, .. }) = events.next() else {
        panic!("one erasure page terminal")
    };
    assert!(events.next().is_none());
    for row in &rows {
        assert!(row.owner.is_none());
        assert!(row.ancestors.is_empty());
        assert!(row.related.is_none());
        assert!(row.unjoined.is_none());
        assert!(row.ended_read.is_none());
        assert!(row.linked.is_empty());
        assert!(row.inbox.is_none());
    }
    (rows, next)
}

#[test]
fn ended_tasks_page_every_ended_task_in_task_order() {
    let mut world = World::new(18);
    let expected = [ended(2, 2), ended(5, 1), ended(8, 2)];
    world
        .commit(Box::new([
            Write::Save(root::header(1)),
            Write::Save(expected[2].clone()),
            Write::Save(root::live_task(3)),
            Write::Save(expected[0].clone()),
            Write::Save(expected[1].clone()),
        ]))
        .expect("ended tasks of two projects and a live task");
    let mut after = None;
    for (index, record) in expected.iter().enumerate() {
        let (rows, next) = page(&mut world, reads::Scope::EndedTasks, after, 1, PAGE_BYTES);
        assert_eq!(rows.len(), 1);
        assert_eq!(&rows[0].value, record);
        if index + 1 < expected.len() {
            assert_eq!(next, Some(reads::Position::Owner(record.key())));
        } else {
            assert!(next.is_none());
        }
        after = next;
    }
}

fn person_proposal(number: u64, project: u32, state: tasks::PersonProposalState) -> tasks::PersonProposal {
    tasks::PersonProposal {
        number,
        proposer: 1,
        project,
        action: tasks::PersonAction::Amend {
            task: TASK,
            revision: 1,
            amendment: Box::new(tasks::Amendment {
                tracked: Some(tasks::Tracking::Tracked { priority: 1 }),
                addressed: None,
                instructions: None,
                release: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: None,
                llm: None,
                reason: Box::from(b"track task".as_slice()),
            }),
        },
        reason: Box::from(b"erase proposal".as_slice()),
        state,
    }
}

#[test]
fn decided_proposals_page_every_decided_proposal_in_order() {
    let mut world = World::with_joins(18, 0);
    let expected = [
        core::Record::Tasks(tasks::Stored::DecidedProposal(Box::new(person_proposal(
            2,
            2,
            tasks::PersonProposalState::Accepted { by: tasks::Party::Person(1) },
        )))),
        core::Record::Tasks(tasks::Stored::DecidedProposal(Box::new(person_proposal(
            5,
            1,
            tasks::PersonProposalState::Rejected {
                by: tasks::Party::Person(1),
                reason: Box::from(b"declined".as_slice()),
                message: 1,
                at: Wall::EPOCH,
            },
        )))),
        core::Record::Tasks(tasks::Stored::DecidedProposal(Box::new(person_proposal(
            8,
            2,
            tasks::PersonProposalState::Withdrawn { at: Wall::EPOCH },
        )))),
    ];
    world
        .commit(Box::new([
            Write::Save(root::header(1)),
            Write::Save(expected[2].clone()),
            Write::Save(core::Record::Tasks(tasks::Stored::PersonProposal(Box::new(person_proposal(
                3,
                1,
                tasks::PersonProposalState::Pending { since: Wall::EPOCH },
            ))))),
            Write::Save(expected[0].clone()),
            Write::Save(expected[1].clone()),
        ]))
        .expect("decided proposals of two projects and a pending proposal");
    let mut after = None;
    for (index, record) in expected.iter().enumerate() {
        let (rows, next) = page(&mut world, reads::Scope::DecidedProposals, after, 1, PAGE_BYTES);
        assert_eq!(rows.len(), 1);
        assert_eq!(&rows[0].value, record);
        if index + 1 < expected.len() {
            assert_eq!(next, Some(reads::Position::Owner(record.key())));
        } else {
            assert!(next.is_none());
        }
        after = next;
    }
}

fn task_rows(task: u64) -> Vec<core::Record> {
    let mut records = vec![core::Record::Tasks(tasks::Stored::Stub(tasks::Stub {
        task,
        phase: tasks::Status::Failed,
        result: Token::new(task),
    }))];
    for revision in 1..=2 {
        records.push(core::Record::Tasks(tasks::Stored::History(tasks::History {
            task,
            revision,
            at: Wall::EPOCH,
            by: tasks::Party::Person(1),
            change: tasks::Change::Amended,
            reason: Box::new([]),
            proposal: None,
            amendment: None,
        })));
    }
    for position in 1..=2 {
        records.push(core::Record::Tasks(tasks::Stored::Milestone(tasks::Milestone {
            task,
            position,
            phase: tasks::Phase::Waiting,
            at: Wall::EPOCH,
        })));
    }
    for position in 1..=2 {
        records.push(core::Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
            task,
            position,
            at: Wall::EPOCH,
            line: tasks::Line::Decided { revision: position },
        })));
    }
    for delegate in [15, 16] {
        records
            .push(core::Record::Tasks(tasks::Stored::EndedDelegate(tasks::Edge { requester: task, task: delegate })));
    }
    records.push(core::Record::Tasks(tasks::Stored::Delegate(tasks::Edge { requester: task, task: 17 })));
    records.push(root::run_proof(task));
    for position in 1..=2 {
        records.push(core::Record::Core(core::CoreRecord::Call(core::CallRecord {
            key: core::CallKey { task, attempt: 1, completion: 1, position },
            part: core::CallPart::Controlled,
            settled: Some(core::SettledCall {
                serial: u64::from(position),
                tool: Box::from(b"control".as_slice()),
                answer: core::SettledAnswer::Host { error: false, body: Box::from(b"done".as_slice()) },
            }),
        })));
    }
    records.push(core::Record::Core(core::CoreRecord::Terminal(core::TerminalRecord {
        task,
        attempt: 1,
        cumulative: 0,
        end: tasks::End::Parked,
    })));
    records.push(core::Record::Core(core::CoreRecord::EscalationDecision(core::EscalationDecisionRecord {
        project: 1,
        requester: 1,
        task,
        revision: 1,
        by: 1,
        at: Wall::EPOCH,
        decision: people::EscalationDecision::Reject { reason: Box::from(b"ended".as_slice()) },
    })));
    records.push(core::Record::Core(core::CoreRecord::ProposalDecision(core::ProposalDecisionRecord {
        project: 1,
        proposer: people::Proposer::Task(task),
        proposal: 1,
        kind: tasks::ProposalKind::Settings,
        by: 1,
        at: Wall::EPOCH,
        choice: people::ProposalChoice::Accepted,
        created: None,
    })));
    records.push(core::Record::Core(core::CoreRecord::Settings(core::settings::Record::Proposal {
        proposer: people::Proposer::Task(task),
        proposal: 1,
        value: core::settings::Proposal {
            against: 1,
            items: Box::new([core::settings::Item {
                setting: core::settings::Setting::Template(1),
                edit: core::settings::Edit::Reset,
            }]),
        },
    })));
    records.push(core::Record::People(people::Stored::EndedRead { task, party: 1, through: 2 }));
    records
}

fn fixture() -> (World, Vec<core::Record>) {
    let mut world = World::with_joins(18, 0);
    let expected = task_rows(TASK);
    let mut writes = vec![Write::Save(root::header(1)), Write::Save(ended(TASK, 1))];
    for task in [TASK - 1, TASK, TASK + 1] {
        writes.extend(task_rows(task).into_iter().map(Write::Save));
    }
    // These rows belong to the caller's task record or a party-first range.
    writes.push(Write::Save(core::Record::People(people::Stored::ReadConversation {
        party: 1,
        task: TASK,
        through: 2,
    })));
    world.commit(writes.into_boxed_slice()).expect("task and neighbouring rows");
    (world, expected)
}

fn collect(world: &mut World, cap: u32) -> Vec<core::Record> {
    let mut found = Vec::new();
    let mut after = None;
    for _ in 0..100 {
        let (rows, next) = page(world, reads::Scope::TaskRows { task: TASK }, after, cap, PAGE_BYTES);
        assert!(rows.len() <= usize::try_from(cap).expect("page cap"));
        assert!(next.is_none() || !rows.is_empty(), "continuation makes progress");
        if next.is_some() {
            assert_eq!(next, rows.last().map(|row| reads::Position::Owner(row.value.key())));
        }
        for row in rows {
            assert!(!found.iter().any(|record: &core::Record| record.key() == row.value.key()), "each key once");
            found.push(row.value);
        }
        after = next;
        if after.is_none() {
            return found;
        }
    }
    panic!("bounded fixture walk terminates");
}

#[test]
fn task_rows_page_every_row_keyed_by_the_task() {
    let (mut world, expected) = fixture();
    for cap in [1, 3, 32] {
        assert_eq!(collect(&mut world, cap), expected, "family order and exact task prefix");
    }
}

#[test]
fn task_rows_are_empty_once_erased() {
    let (mut world, expected) = fixture();
    let mut after = None;
    for (index, _) in expected.iter().enumerate() {
        let commit = u64::try_from(index).expect("fixture rows fit") + 2;
        let (rows, next) = page(&mut world, reads::Scope::TaskRows { task: TASK }, after, 1, PAGE_BYTES);
        assert_eq!(rows.len(), 1);
        world
            .commit(Box::new([Write::Save(root::header(commit)), Write::Erase(rows[0].value.key())]))
            .expect("erase discovered key");
        after = next;
        if after.is_none() {
            break;
        }
    }
    let (rows, next) = page(&mut world, reads::Scope::TaskRows { task: TASK }, None, 1, PAGE_BYTES);
    assert!(rows.is_empty());
    assert!(next.is_none());
    assert_eq!(world.get(core::Key::Tasks(tasks::Key::Ended(TASK))), ended(TASK, 1));
    for task in [TASK - 1, TASK + 1] {
        for record in task_rows(task) {
            assert_eq!(world.get(record.key()), record, "neighbours remain");
        }
    }
}

#[test]
fn erasure_pages_stop_at_the_decoded_byte_cap_and_resume_across_families() {
    let (mut world, expected) = fixture();
    let (rows, _) = page(&mut world, reads::Scope::TaskRows { task: TASK }, None, 32, PAGE_BYTES);
    let first_bytes = rows[0].owned_bytes().expect("stub ownership");
    let second_bytes = rows[1].owned_bytes().expect("history ownership");
    for bytes in [first_bytes, first_bytes + second_bytes - 1] {
        let (rows, next) = page(&mut world, reads::Scope::TaskRows { task: TASK }, None, 32, bytes);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].value, expected[0]);
        assert_eq!(next, Some(reads::Position::Owner(expected[0].key())));
        let (rows, _) = page(&mut world, reads::Scope::TaskRows { task: TASK }, next, 1, PAGE_BYTES);
        assert_eq!(rows[0].value, expected[1]);
    }
    let (rows, next) = page(&mut world, reads::Scope::TaskRows { task: TASK }, None, 32, first_bytes + second_bytes);
    assert_eq!(rows.len(), 2);
    assert_eq!(next, Some(reads::Position::Owner(expected[1].key())));
}
