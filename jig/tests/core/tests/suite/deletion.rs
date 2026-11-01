//! Project erasure through the real store and crash boundary (contracts J4, J6).
use jig_core_world::{
    effects::{Cut, World},
    stories::reads::{request, world},
};
use jig_domain_core::{self as core, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

const SEED: u64 = 911;

fn ended(task: u64) -> root::Key {
    root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))
}

fn finished(turns: u32) -> (World, u64) {
    let mut world = world(SEED);
    let task = world.assigned.as_ref().expect("conversation").task;
    for number in 1..=turns {
        world.turn(number, u64::from(number), None, b"reply");
    }
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        u64::from(turns),
    );
    assert!(world.store.rows.contains_key(&ended(task)));
    (world, task)
}

fn erase(world: &mut World) {
    world.send(root::Event::Core(core::Event::People(people::Event::Deletion {
        project: 1,
        stage: Some(people::Stage::Erasing),
    })));
}

fn releasing(world: &World) {
    assert_eq!(
        world.store.rows.get(&root::Key::Core(core::Key::People(people::Key::Deletion(1)))),
        Some(&root::Record::Core(core::Record::People(people::Stored::Deletion {
            project: 1,
            stage: people::Stage::Releasing,
        })))
    );
}

fn configuration() -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = jig_core_world::faults::plan_fixture(SEED);
    limits.core.fleet.slots = 2;
    limits.core.tasks.executor_kinds = 3;
    limits.core.authority.executors = 3;
    limits.core.documents.executors = 3;
    limits.core.authority.projects = 2;
    limits.core.people.projects = 3;
    limits.core.people.initial_owners = 2;
    limits.core.people.inbox_recipients = 24;
    let kinds: Box<[jig_domain_authority::Executor]> = Box::new([
        jig_domain_authority::Executor::Charter(1),
        jig_domain_authority::Executor::Procedure(1),
        jig_domain_authority::Executor::Role(0),
    ]);
    configuration.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    let policy = &mut configuration.core.seed_projects[0].policy;
    policy.policy.ceiling.delegation.kinds.clone_from(&kinds);
    for role in &mut policy.policy.roles {
        role.authority.delegation.kinds.clone_from(&kinds);
    }
    let second = core::SeedProject { number: 2, name: "Other project".into(), policy: policy.clone() };
    let first =
        core::SeedProject { number: 1, name: configuration.core.seed_projects[0].name.clone(), policy: policy.clone() };
    configuration.core.seed_projects = Box::new([first, second]);
    let owner = configuration.core.owners[0].clone();
    configuration.core.owners =
        Box::new([owner.clone(), people::InitialOwner { project: 2, identity: owner.identity }]);
    jig_core_world::peers::journal(&mut limits);
    (configuration, limits)
}

#[expect(
    clippy::too_many_lines,
    reason = "one project fixture includes delegation, a question, a person, a proposal and the unaffected project"
)]
fn rich_world() -> (World, Vec<u64>, u64, u64) {
    let (configuration, limits) = configuration();
    let mut world =
        World::configured_with_startup(SEED, configuration, limits, |domain| domain.core_mut().person_answers());
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    let task = world.assigned.as_ref().expect("conversation").task;
    let parent_attempt = world.assigned.as_ref().expect("conversation").attempt;
    world.turn(1, 1, None, b"reply with transcript");
    let mut member = jig_core_world::fixtures::member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(9100)),
        key: world.call_key(1),
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    let delegate = world.assigned.as_ref().expect("agent delegate").task;
    assert_ne!(delegate, task, "delegate: {:?}", world.answers);
    world.turn(1, 1, None, b"delegate transcript");
    world.send(root::Event::Core(core::Event::NamedAction {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(9101)),
        key: world.call_key(1),
        action: core::NamedAction::Message {
            target: task,
            kind: tasks::MessageKind::Question,
            words: b"question?".as_slice().into(),
        },
    }));
    let Some((_, _, core::CallPart::Sent { message: question })) = world.answers.last() else {
        panic!("question: {:?}", world.answers)
    };
    let question = *question;
    request(
        &mut world,
        64,
        people::Ask::AnswerQuestion { project: 1, task, question, words: b"answer".as_slice().into() },
    );
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::QuestionAnswered { .. }))
    ));
    world.send(root::Event::Core(core::Event::NamedAction {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(9102)),
        key: core::CallKey { task, attempt: parent_attempt, completion: 2, position: 0 },
        action: core::NamedAction::Message {
            target: delegate,
            kind: tasks::MessageKind::Answer { question },
            words: b"answer".as_slice().into(),
        },
    }));
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"delegated".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    assert!(world.store.rows.contains_key(&ended(delegate)));
    let revision = jig_core_world::fixtures::live_row(&world, task).revisions.spec;
    request(
        &mut world,
        60,
        people::Ask::Propose {
            project: 1,
            action: people::Action::Amend {
                task,
                revision,
                amendment: Box::new(people::Amendment {
                    spec: Some(people::Spec {
                        asks: false,
                        words: b"proposed change".as_slice().into(),
                        parameters: Box::new([]),
                        inputs: Box::new([]),
                    }),
                    ..jig_core_world::fixtures::person_unchanged(b"proposed")
                }),
            },
            reason: b"review this change".as_slice().into(),
        },
    );
    let Some(people::Reply::Outcome(people::Outcome::Proposed { proposal, .. })) = world.people_answers.last() else {
        panic!("person proposal: {:?}", world.people_answers)
    };
    let proposal = *proposal;
    request(
        &mut world,
        61,
        people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Party(1),
            proposal,
            decision: people::ProposalDecision::Reject { reason: b"keep current".as_slice().into() },
            running: None,
        },
    );
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::DecidedProposal(proposal)))));
    let mut member = jig_core_world::fixtures::people_member(b"person report", 10);
    member.executor = people::TaskExecutor::Person(people::PersonAddress::Role(0));
    request(&mut world, 63, people::Ask::Create { project: 1, batch: Box::new([member]) });
    let Some(people::Reply::Outcome(people::Outcome::Created { first: person_task, .. })) = world.people_answers.last()
    else {
        panic!("person task: {:?}", world.people_answers)
    };
    let person_task = *person_task;
    request(&mut world, 67, people::Ask::TakePerson { project: 1, task: person_task });
    let revision = jig_core_world::fixtures::live_row(&world, person_task).revisions.person;
    request(
        &mut world,
        65,
        people::Ask::AnswerPerson {
            project: 1,
            task: person_task,
            revision,
            result: people::PersonResult::Report { words: b"checked".as_slice().into() },
        },
    );
    assert!(world.store.rows.contains_key(&ended(person_task)), "person answer: {:?}", world.people_answers);
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: skein_lib::Token::new(7),
        task,
        attempt: parent_attempt,
        cumulative: 1,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
    });
    assert!(world.store.rows.contains_key(&ended(task)));
    let other_request = jig_core_world::fixtures::chat(&world, 2, b"other".as_slice().into());
    request(&mut world, 66, other_request);
    let Some(people::Reply::Outcome(people::Outcome::Created { first: other, .. })) = world.people_answers.last()
    else {
        panic!("other project conversation: {:?}", world.people_answers)
    };
    let other = *other;
    assert_eq!(world.assigned.as_ref().expect("other assignment").task, other);
    assert_ne!(task, other);
    world.turn(1, 1, None, b"other transcript");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"other done".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    (world, vec![task, delegate, person_task], proposal, other)
}

fn names(row: &root::Record, tasks: &[u64]) -> bool {
    match row {
        root::Record::Core(core::Record::Tasks(row)) => match row {
            tasks::Stored::Live(row) | tasks::Stored::Ended(row) => tasks.contains(&row.number),
            tasks::Stored::History(row) => tasks.contains(&row.task),
            tasks::Stored::Activity(row) => tasks.contains(&row.task),
            tasks::Stored::Milestone(row) => tasks.contains(&row.task),
            tasks::Stored::Stub(row) => tasks.contains(&row.task),
            tasks::Stored::Result(row) => tasks.contains(&row.task),
            tasks::Stored::Update(row) => tasks.contains(&row.task),
            tasks::Stored::EndedByParty(row) => tasks.contains(&row.task),
            tasks::Stored::EndedByProject(row) => tasks.contains(&row.task),
            tasks::Stored::EndedDelegate(row) | tasks::Stored::Delegate(row) => {
                tasks.contains(&row.task) || tasks.contains(&row.requester)
            }
            tasks::Stored::PlacedHold(row) => {
                matches!(row.owner, tasks::Owner::Task { task } if tasks.contains(&task))
                    || matches!(row.scope, tasks::Scope::Subtree { task } if tasks.contains(&task))
            }
            tasks::Stored::Ledger(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::DecidedProposal(_) => false,
        },
        root::Record::Core(core::Record::Core(row)) => match row {
            core::CoreRecord::Terminal(row) => tasks.contains(&row.task),
            core::CoreRecord::Turn(row) => tasks.contains(&row.task),
            core::CoreRecord::Call(row) => tasks.contains(&row.key.task),
            core::CoreRecord::RunProof(row) => tasks.contains(&row.task),
            core::CoreRecord::EscalationDecision(row) => tasks.contains(&row.task),
            core::CoreRecord::ProposalDecision(row) => {
                matches!(row.proposer, people::Proposer::Task(task) if tasks.contains(&task))
            }
            core::CoreRecord::Projection(_) | core::CoreRecord::Deployment(_) | core::CoreRecord::Settings(_) => false,
        },
        root::Record::Core(core::Record::People(
            people::Stored::EndedRead { task, .. } | people::Stored::ReadConversation { task, .. },
        )) => tasks.contains(task),
        root::Record::Core(core::Record::People(_) | core::Record::Notes(_) | core::Record::Accounts(_))
        | root::Record::Connector { .. } => false,
    }
}

fn transcript(world: &mut World, task: u64) -> Vec<Box<[u8]>> {
    let attempt = world.assignments.iter().find(|(owner, _)| *owner == task).expect("task assigned").1;
    world.store.ask(jig_test_store::StoreOut::Now(core::Now::StartPreparation {
        task,
        transcript_waiter: None,
        from: Some((attempt, 1)),
    }));
    let mut turns = Vec::new();
    for answer in world.store.answers() {
        if let jig_test_store::Up::Root(root::Event::TranscriptLoaded { rows, .. }) = answer {
            turns.extend(rows.into_vec().into_iter().map(|(row, _)| row.transcript));
        }
    }
    turns
}

#[test]
fn erasing_leaves_no_row_of_the_projects_ended_tasks() {
    let (mut world, tasks, proposal, other) = rich_world();
    let kept: Vec<_> = world.store.rows.values().filter(|row| names(row, &[other])).cloned().collect();
    assert!(!kept.is_empty());
    assert!(!transcript(&mut world, tasks[0]).is_empty());
    assert!(!transcript(&mut world, other).is_empty());
    erase(&mut world);
    releasing(&world);
    for row in world.store.rows.values() {
        assert!(!names(row, &tasks), "erased task row remains: {row:?}");
        assert!(
            !matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::DecidedProposal(row))) if row.number == proposal)
        );
        assert!(
            !matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Update(tasks::Update { object: tasks::UpdateObject::Proposal { proposal: owner }, .. }))) if owner.value() == proposal)
        );
    }
    assert_eq!(world.store.rows.values().filter(|row| names(row, &[other])).cloned().collect::<Vec<_>>(), kept);
    assert!(transcript(&mut world, tasks[0]).is_empty());
    assert!(!transcript(&mut world, other).is_empty());
}

#[test]
fn an_erased_tasks_inbox_entries_and_positions_are_gone() {
    let (mut world, task) = finished(2);
    request(&mut world, 62, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    assert!(
        world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead { task, party: 1 })))
    );
    erase(&mut world);
    for restart in [false, true] {
        if restart {
            world.restart(SEED, false);
        }
        releasing(&world);
        assert!(
            !world
                .store
                .rows
                .contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead { task, party: 1 })))
        );
        jig_core_world::reads::ask(
            &mut world,
            500,
            reads::Read::Inbox {
                group: reads::Group::Updates,
                filter: reads::Filter { project: None, goal: None },
                from: None,
            },
        );
        let reads::Answer::Page(page) = jig_core_world::reads::latest(&world) else { panic!("inbox page") };
        assert!(page.rows.is_empty());
    }
}

#[test]
fn erasing_pages_through_a_task_larger_than_one_page() {
    let (mut world, task) = finished(40);
    let before = world.commits.len();
    erase(&mut world);
    releasing(&world);
    let write_bound = 74
        + jig_core_world::world::limits().core.tasks.placed_holds
        + 2 * jig_core_world::world::limits().core.people.people;
    assert!(
        world.commits[before..].iter().all(|writes| writes.len() <= usize::try_from(write_bound).expect("write bound"))
    );
    let activity_pages = world.commits[before..].iter().filter(|writes| writes.iter().any(|write| matches!(write,
        root::Write::Erase(root::Key::Core(core::Key::Tasks(tasks::Key::Activity { task: owner, .. }))) if *owner == task
    ))).count();
    assert!(activity_pages >= 2);
    assert!(!world.store.rows.contains_key(&ended(task)));
}

#[test]
fn a_crash_during_erasing_resumes_from_the_start_and_ends_the_same() {
    let (mut uncut, task) = finished(40);
    let first = uncut.store.applied + 1;
    erase(&mut uncut);
    let last = uncut.store.applied;
    let expected: Vec<_> = uncut
        .store
        .rows
        .values()
        .filter(|row| !matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(_)))))
        .cloned()
        .collect();
    assert!(last > first);
    for number in first..=last {
        for durable in [true, false] {
            if !durable && number == first {
                continue; // An uncommitted deletion request has not started Erasing.
            }
            let cut = if durable { Cut::Durable(number) } else { Cut::Submitted(number) };
            let (mut world, _) = finished(40);
            world.cut = Some(cut);
            erase(&mut world);
            assert_eq!(world.reached_cut, Some(cut));
            world.reads.as_mut().expect("read observer").hold_loads = true;
            world.restart(SEED, false);
            let erasing = world.store.rows.get(&root::Key::Core(core::Key::People(people::Key::Deletion(1))))
                == Some(&root::Record::Core(core::Record::People(people::Stored::Deletion {
                    project: 1,
                    stage: people::Stage::Erasing,
                })));
            if erasing {
                let loads = &world.reads.as_ref().expect("read observer").loads;
                assert_eq!(loads.len(), 1);
                assert_eq!(loads[0].1.scope, reads::Scope::ProjectRows { project: 1 });
                assert_eq!(loads[0].1.after, None, "restart scans the project from its beginning");
            }
            world.reads.as_mut().expect("read observer").hold_loads = false;
            for request in world.reads.as_mut().expect("read observer").complete() {
                world.store.ask(request);
            }
            world.drain();
            releasing(&world);
            assert!(!world.store.rows.contains_key(&ended(task)));
            let rows: Vec<_> = world
                .store
                .rows
                .values()
                .filter(|row| !matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(_)))))
                .cloned()
                .collect();
            assert_eq!(rows, expected, "crash at {cut:?}");
        }
    }
}

#[test]
fn erasing_uses_its_own_load_while_the_partys_only_read_slot_is_busy() {
    let (mut world, task) = finished(2);
    world.reads.as_mut().expect("read observer").hold_loads = true;
    jig_core_world::reads::ask(&mut world, 501, reads::Read::Task { task });
    erase(&mut world);
    let loads = &world.reads.as_ref().expect("read observer").loads;
    assert_eq!(loads.len(), 2, "one party load and the independent erasure load");
    assert_ne!(loads[0].0, loads[1].0);
    assert_eq!(loads[1].1.party, 0);
    assert_eq!(loads[1].1.scope, reads::Scope::ProjectRows { project: 1 });
    let first = loads[1].0;
    world.send(root::Event::Core(core::Event::ReadLoaded { load: first, result: reads::Loaded::Busy }));
    let loads = &world.reads.as_ref().expect("read observer").loads;
    assert_eq!(loads.len(), 3);
    assert_ne!(loads[2].0, first, "retry has its own terminal identity");
    world.send(root::Event::Core(core::Event::ReadLoaded {
        load: first,
        result: reads::Loaded::Page { rows: Box::new([]), next: None },
    }));
    assert_eq!(world.reads.as_ref().expect("read observer").loads.len(), 3, "stale load cannot advance the scan");
    world.reads.as_mut().expect("read observer").hold_loads = false;
    for request in world.reads.as_mut().expect("read observer").complete() {
        world.store.ask(request);
    }
    world.drain();
    releasing(&world);
    assert!(!world.store.rows.contains_key(&ended(task)));
}

#[test]
fn erasing_is_independent_of_the_authenticated_read_byte_budget() {
    let (mut world, task) = finished(2);
    world.seed(Box::new([root::Record::Core(core::Record::People(people::Stored::Deletion {
        project: 1,
        stage: people::Stage::Erasing,
    }))]));
    let (configuration, mut limits) = jig_core_world::effects::fixture(SEED, false);
    limits.core.load_bytes = 1;
    world.restart_with_limits(configuration, limits);
    releasing(&world);
    assert!(!world.store.rows.contains_key(&ended(task)));
}

#[test]
fn erasing_serializes_projects_in_project_order() {
    let (mut world, tasks, _, other) = rich_world();
    world.seed(Box::new([2, 1].map(|project| {
        root::Record::Core(core::Record::People(people::Stored::Deletion { project, stage: people::Stage::Erasing }))
    })));
    world.reads.as_mut().expect("read observer").hold_loads = true;
    let before = world.commits.len();
    world.restart_with(configuration().0);
    let loads = &world.reads.as_ref().expect("read observer").loads;
    assert_eq!(loads.len(), 1, "one erasing project at a time");
    assert_eq!(loads[0].1.scope, reads::Scope::ProjectRows { project: 1 });
    world.reads.as_mut().expect("read observer").hold_loads = false;
    for request in world.reads.as_mut().expect("read observer").complete() {
        world.store.ask(request);
    }
    world.drain();
    releasing(&world);
    assert_eq!(
        world.store.rows.get(&root::Key::Core(core::Key::People(people::Key::Deletion(2)))),
        Some(&root::Record::Core(core::Record::People(people::Stored::Deletion {
            project: 2,
            stage: people::Stage::Releasing
        })))
    );
    let dropped: Vec<_> = world.commits[before..]
        .iter()
        .flatten()
        .filter_map(|write| match write {
            root::Write::DropTranscript { task } => Some(*task),
            root::Write::Save(_)
            | root::Write::Erase(_)
            | root::Write::CopySettingsRevision(_)
            | root::Write::Wires => None,
        })
        .collect();
    assert_eq!(dropped.len(), tasks.len() + 1);
    assert_eq!(dropped.last(), Some(&other));
    assert!(world.store.rows.values().all(|row| !names(row, &tasks) && !names(row, &[other])));
}
