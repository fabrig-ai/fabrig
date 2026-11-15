//! Ended work goes in stages, through the real store (protocol/store/retention.md, 6).
use crate::read_support::request;
use jig_core_world::{
    fixtures::PARTY,
    world::{Setup, World, fixture},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Time, Wall};

pub(super) const DAY: u64 = 86_400_000_000_000;
pub(super) fn world(seed: u64, forgotten: bool) -> World {
    let (mut config, mut limits) = fixture(seed, false);
    limits.core.documents.rows = 1;
    limits.core.people.sign_in_lifetime = Duration::from_secs(800 * 86_400);
    limits.core.people.requests = 16;
    config.core.grants = Box::new([people::ActGrant { connector: 1, may: Box::new([people::ActKind::RemoveProject]) }]);
    let initial = forgotten.then(jig_test_profile::fixtures::forgettable_conversation);
    World::new(Setup { seed, configuration: Some(config), limits, initial, ..Setup::default() })
}
pub(super) fn key(key: tasks::Key) -> root::Key {
    root::Key::Core(core::Key::Tasks(key))
}
pub(super) fn finish(world: &mut World) -> tasks::TaskRecord {
    let task = world.task();
    world.turn(1, 1, None, b"retained reply");
    jig_core_world::fixtures::finish_chat(world, 1, b"retained result");
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(record))) =
        &world.store.rows[&key(tasks::Key::Ended(task))]
    else {
        panic!("ended task")
    };
    *record.clone()
}
pub(super) fn read(world: &mut World, record: &tasks::TaskRecord) {
    let mark = 90 + u8::try_from(record.number).expect("fixture tasks fit a key") * 2;
    request(
        world,
        mark,
        people::Ask::MarkSeen { seen: people::Seen::Conversation { task: record.number, through: record.last_reply } },
    );
    request(world, mark + 1, people::Ask::MarkSeen { seen: people::Seen::Results { through: record.result_position } });
}
pub(super) fn at(world: &mut World, record: &tasks::TaskRecord, days: u64) {
    let wall = record.ended_at.expect("ended").as_nanos() + days * DAY;
    let now = world.clocks().0;
    world.set_clock(now, Wall::from_nanos(wall));
}
pub(super) fn pass(world: &mut World) {
    world.send(root::Event::Timer(core::Timer::Retention));
}
pub(super) fn has(world: &World, task: u64) -> bool {
    world.store.rows.contains_key(&key(tasks::Key::Ended(task)))
}
pub(super) fn summary(world: &World, task: u64) -> bool {
    world.store.rows.contains_key(&key(tasks::Key::Summary(task)))
}
pub(super) fn places(world: &mut World, task: u64) -> bool {
    let key = jig_protocol_store::keys::encode_own(&jig_protocol_store::keys::Own::Transcript { task }, u32::MAX)
        .expect("transcript key");
    world.store.retained().expect("quiescent store").contains_key(&key)
}
pub(super) fn indexes(world: &World, task: u64) -> usize {
    world
        .store
        .rows
        .keys()
        .filter(|key| {
            matches!(key, root::Key::Core(core::Key::Tasks(
        tasks::Key::EndedByParty { task: n, .. } | tasks::Key::EndedByProject { task: n, .. }
        | tasks::Key::EndedDelegate { task: n, .. })) if *n == task)
        })
        .count()
}
pub(super) fn own(world: &World, task: u64) -> Vec<root::Key> {
    world.store.rows.keys().filter(|key| matches!(key,
        root::Key::Core(core::Key::Tasks(tasks::Key::Activity { task: n, .. } | tasks::Key::History { task: n, .. }
            | tasks::Key::Milestone { task: n, .. })
        | core::Key::Core(core::CoreKey::Terminal { task: n, .. }
            | core::CoreKey::ProposalDecision { proposer: people::Proposer::Task(n), .. }
            | core::CoreKey::EscalationDecision { task: n, .. })
        | core::Key::People(people::Key::EndedRead { task: n, .. })) if *n == task
    )).cloned().collect()
}

#[test]
fn turns_stage_drops_only_the_transcript_at_day_fifteen() {
    let mut w = world(4001, false);
    let record = finish(&mut w);
    let rows = own(&w, record.number);
    assert!(places(&mut w, record.number));
    at(&mut w, &record, 13);
    pass(&mut w);
    assert!(places(&mut w, record.number), "not before its horizon");
    at(&mut w, &record, 15);
    pass(&mut w);
    assert!(!places(&mut w, record.number), "turns stage drops places");
    assert!(has(&w, record.number));
    assert_eq!(own(&w, record.number), rows, "record and activity remain");
}

#[test]
fn record_stage_keeps_only_summary_and_ended_indexes_at_day_thirty_one() {
    let mut w = world(4002, false);
    let record = finish(&mut w);
    read(&mut w, &record);
    let decided_at = record.ended_at.expect("ended");
    let task = record.number;
    w.seed(Box::new([
        root::Record::Core(core::Record::Core(core::CoreRecord::ProposalDecision(core::ProposalDecisionRecord {
            project: 1,
            proposer: people::Proposer::Task(task),
            proposal: 1,
            kind: tasks::ProposalKind::Amend,
            by: PARTY,
            at: decided_at,
            choice: people::ProposalChoice::Rejected,
            created: None,
        }))),
        root::Record::Core(core::Record::Core(core::CoreRecord::EscalationDecision(core::EscalationDecisionRecord {
            project: 1,
            requester: PARTY,
            task,
            revision: 1,
            by: PARTY,
            at: decided_at,
            decision: people::EscalationDecision::Release,
        }))),
        root::Record::Connector {
            number: 1,
            record: jig_test_connector::Record::Task { task, project: 1, resources: Box::new([]) },
        },
        root::Record::Connector {
            number: 2,
            record: jig_test_connector::Record::Task { task, project: 1, resources: Box::new([]) },
        },
    ]));
    w.restart(4002, false);
    let rows = own(&w, record.number);
    assert_ne!(rows, []);
    let kept = indexes(&w, record.number);
    assert!(kept > 0);
    at(&mut w, &record, 29);
    pass(&mut w);
    assert!(has(&w, record.number));
    at(&mut w, &record, 31);
    pass(&mut w);
    assert!(!has(&w, record.number), "record stage erases owner");
    assert!(summary(&w, record.number));
    assert_eq!(indexes(&w, record.number), kept);
    assert_eq!(own(&w, record.number), []);
    for number in [1, 2] {
        assert!(
            !w.store.rows.contains_key(&root::Key::Connector {
                number,
                key: jig_test_connector::RecordKey::Task(record.number)
            }),
            "every connector hears Unname"
        );
    }
    assert!(!places(&mut w, record.number));
    for row in w.store.rows.values() {
        assert!(
            !matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Update(update))) if update.task == record.number)
        );
    }
    assert!(!w.store.rows.contains_key(&key(tasks::Key::Result { party: PARTY, position: record.result_position })));
    assert!(!w.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::ReadConversation {
        party: PARTY,
        task: record.number
    }))));
    assert!(w.commits.iter().any(|commit| commit.contains(&root::Write::Erase(key(tasks::Key::Ended(record.number))))));
}

#[test]
fn summary_stage_leaves_nothing_at_day_three_hundred_sixty_six() {
    let mut w = world(4003, false);
    let record = finish(&mut w);
    read(&mut w, &record);
    at(&mut w, &record, 31);
    pass(&mut w);
    assert!(summary(&w, record.number));
    at(&mut w, &record, 364);
    pass(&mut w);
    assert!(summary(&w, record.number), "summary stays before horizon");
    at(&mut w, &record, 366);
    pass(&mut w);
    assert!(!summary(&w, record.number), "summary stage erases owner");
    assert_eq!(indexes(&w, record.number), 0);
    assert_eq!(own(&w, record.number), []);
    assert!(!has(&w, record.number));
}

#[test]
fn record_stage_defers_an_unread_result_and_counts_again_from_deferral() {
    let mut w = world(4004, false);
    let record = finish(&mut w);
    at(&mut w, &record, 31);
    pass(&mut w);
    assert!(has(&w, record.number), "unread result defers record erasure");
    read(&mut w, &record);
    at(&mut w, &record, 32);
    pass(&mut w);
    assert!(has(&w, record.number), "deferral starts a fresh horizon");
    at(&mut w, &record, 62);
    pass(&mut w);
    assert!(!has(&w, record.number));
    assert!(summary(&w, record.number));
}

#[test]
fn live_names_defer_whole_tasks_and_a_read_forgotten_owner_goes_whole_afterwards() {
    for forgotten in [false, true] {
        let mut w = world(4010, forgotten);
        let record = finish(&mut w);
        let mut input = jig_test_profile::fixtures::people_member(b"names ended work", 10);
        input.inputs = Box::new([record.number]);
        request(&mut w, 72, people::Ask::Create { project: 1, batch: Box::new([input]) });
        assert!(w.store.rows.contains_key(&key(tasks::Key::Stub(record.number))));
        read(&mut w, &record);
        at(&mut w, &record, 31);
        pass(&mut w);
        assert!(has(&w, record.number), "live work keeps full owner");
        assert!(places(&mut w, record.number), "live work keeps turns too");
        let dependent = finish(&mut w);
        read(&mut w, &dependent);
        at(&mut w, &record, 32);
        pass(&mut w);
        assert!(has(&w, record.number), "fresh horizon after deferral");
        at(&mut w, &record, 62);
        pass(&mut w);
        assert!(!has(&w, record.number));
        assert_eq!(summary(&w, record.number), !forgotten);
        if forgotten {
            assert_eq!(indexes(&w, record.number), 0);
        }
    }
}

#[test]
fn a_live_requester_defers_its_ended_delegate_record() {
    let mut w = world(4011, false);
    w.delegate();
    w.observed(13);
    let task = w.procedure.expect("ended delegate");
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(record))) =
        &w.store.rows[&key(tasks::Key::Ended(task))]
    else {
        panic!("ended delegate")
    };
    let record = *record.clone();
    assert!(!w.store.rows.contains_key(&key(tasks::Key::Stub(task))), "requester test is independent of a named stub");
    at(&mut w, &record, 31);
    pass(&mut w);
    assert!(has(&w, task), "live requester defers record");
    let requester = finish(&mut w);
    read(&mut w, &requester);
    at(&mut w, &record, 32);
    pass(&mut w);
    assert!(has(&w, task));
    at(&mut w, &record, 62);
    pass(&mut w);
    assert!(!has(&w, task));
    assert!(summary(&w, task));
}

#[test]
fn proposal_stage_erases_the_decided_proposal_decision_and_update() {
    let mut w = world(4012, false);
    let people::Ask::Create { project, mut batch } =
        jig_core_world::fixtures::chat(&w, 1, b"proposed chat".as_slice().into())
    else {
        panic!("chat")
    };
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 20;
    request(
        &mut w,
        72,
        people::Ask::Propose { project, action: people::Action::Batch(batch), reason: b"review".as_slice().into() },
    );
    let Some(people::Reply::Outcome(people::Outcome::Proposed { proposal, .. })) = w.people_answers.last() else {
        panic!("proposed: {:?}", w.people_answers)
    };
    let proposal = *proposal;
    request(&mut w, 73, people::Ask::Withdraw { project, proposal });
    let decided = key(tasks::Key::DecidedProposal(proposal));
    assert!(w.store.rows.contains_key(&decided));
    let decision = root::Key::Core(core::Key::Core(core::CoreKey::ProposalDecision {
        proposer: people::Proposer::Party(PARTY),
        proposal,
    }));
    assert!(w.store.rows.contains_key(&decision));
    let decision_row = w.store.rows[&decision].clone();
    let update = w.store.rows.values().find(|row| matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Update(update))) if update.object == tasks::UpdateObject::Proposal { proposal: tasks::Newest::of(proposal) })).expect("proposal update").key();
    let update_row = w.store.rows[&update].clone();
    let row = w.store.rows.values().find(|value| matches!(value, root::Record::Core(core::Record::Tasks(tasks::Stored::Retention(row))) if row.stage == tasks::RetentionStage::Proposal && row.number == proposal)).expect("proposal schedule").clone();
    let wall = w.clocks().1;
    let now = w.clocks().0;
    w.set_clock(now, Wall::from_nanos(wall.as_nanos() + 29 * DAY));
    pass(&mut w);
    assert!(w.store.rows.contains_key(&decided));
    w.set_clock(now, Wall::from_nanos(wall.as_nanos() + 31 * DAY));
    pass(&mut w);
    assert!(!w.store.rows.contains_key(&decided), "proposal stage erases decided proposal");
    assert!(!w.store.rows.contains_key(&decision));
    assert!(!w.store.rows.contains_key(&update));
    // A different eraser may already have taken the proposal itself.
    w.seed(Box::new([decision_row, update_row, row]));
    pass(&mut w);
    assert!(!w.store.rows.contains_key(&decision), "orphan decision is not left by a missing proposal");
    assert!(!w.store.rows.contains_key(&update));
}

#[test]
fn forgotten_stage_erases_the_remains_on_the_next_pass() {
    let mut w = world(4005, true);
    let record = finish(&mut w);
    read(&mut w, &record);
    assert!(!has(&w, record.number));
    assert!(summary(&w, record.number));
    assert_ne!(own(&w, record.number), []);
    pass(&mut w);
    assert!(!summary(&w, record.number), "forgotten stage takes summary");
    assert_eq!(own(&w, record.number), []);
    assert!(!places(&mut w, record.number));
    assert_eq!(indexes(&w, record.number), 0);
}

#[test]
fn restart_makes_retention_due_at_once_then_arms_an_hour_after_completion() {
    let mut w = world(4006, false);
    let record = finish(&mut w);
    pass(&mut w);
    let (now, wall) = w.clocks();
    assert_ne!(w.domain.core().due(now, wall), Some(core::Timer::Retention));
    at(&mut w, &record, 15);
    w.restart(4006, false);
    let (now, wall) = w.clocks();
    assert_eq!(w.domain.core().due(now, wall), Some(core::Timer::Retention));
    pass(&mut w);
    assert!(!places(&mut w, record.number));
    assert_ne!(w.domain.core().due(now, wall), Some(core::Timer::Retention));
    let later = now.saturating_add(Duration::from_secs(3600));
    assert_eq!(w.domain.core().due(later, wall), Some(core::Timer::Retention));
}

#[test]
fn leftover_retention_rows_are_erased_alone() {
    let mut w = world(4007, false);
    let before = w.store.rows.clone();
    let row = tasks::RetentionRow { stage: tasks::RetentionStage::Record, since: Wall::EPOCH, number: 999 };
    w.seed(Box::new([root::Record::Core(core::Record::Tasks(tasks::Stored::Retention(row)))]));
    w.set_clock(Time::from_nanos(w.clocks().0.as_nanos()), Wall::from_nanos(31 * DAY));
    pass(&mut w);
    assert!(!w.store.rows.contains_key(&key(row.key())));
    // Only the deployment's commit counter changed beside the leftover row.
    for (key, value) in before {
        if key != root::Key::Core(core::Key::Core(core::CoreKey::Deployment)) {
            assert_eq!(w.store.rows.get(&key), Some(&value));
        }
    }
}

#[test]
fn deletion_takes_shrunk_summaries_and_indexes_before_a_queued_pass() {
    let mut w = world(4008, false);
    let record = finish(&mut w);
    read(&mut w, &record);
    at(&mut w, &record, 31);
    pass(&mut w);
    assert!(!has(&w, record.number));
    assert!(summary(&w, record.number));
    w.send_all(vec![
        root::Event::Core(core::Event::Act {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(72)),
            connector: 1,
            key: [72; 16],
            act: Box::new(people::Act::RemoveProject { project: 1, grace: None }),
            carried: None,
        }),
        root::Event::Timer(core::Timer::Retention),
    ]);
    assert!(matches!(w.act_answers.last(), Some((_, people::Outcome::ProjectRemoved { project: 1 }))));
    assert!(!summary(&w, record.number), "deletion takes already shrunk task");
    assert_eq!(indexes(&w, record.number), 0);
}
