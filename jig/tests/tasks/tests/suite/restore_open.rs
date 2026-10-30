use jig_domain_tasks::{self as tasks, Event, Executor, Party, PersonAddress, Refusal, Stored};
use jig_tasks_world::{LIMITS, World, task};
use skein_lib::Queue;

const PERSON: u64 = 9;
const OWNER: u64 = 1;
const MEMBER: u64 = 2;
const PROPOSAL: u64 = 100;
const PROJECT: u32 = 1;
const SEED: u64 = 860;

fn proposals() -> [(Stored, Vec<Stored>); 2] {
    let mut world = World::new(SEED, LIMITS);
    world.make(Party::Person(PERSON), vec![task(OWNER, &[])]);
    let mut member = task(MEMBER, &[]);
    member.funder = tasks::Funder::Task(OWNER);
    let reply_to = world.to();
    world.send(Event::Propose {
        reply_to,
        proposal: tasks::Proposal {
            number: PROPOSAL,
            proposer: OWNER,
            project: PROJECT,
            action: tasks::ProposalAction::Batch(Box::new([member.clone()])),
            reason: b"proposed work".as_slice().into(),
            as_holder: false,
            state: tasks::ProposalState::Pending {
                holder: tasks::ProposalHolder::Person(PERSON),
                since: world.env.wall,
            },
        },
    });
    let task_proposal = Stored::Live(Box::new(world.record(OWNER).clone()));
    let supporting = world
        .records
        .values()
        .filter(|row| row.key().live() && row.key() != tasks::Key::Live(OWNER))
        .cloned()
        .collect();
    member.funder = tasks::Funder::Pool { project: PROJECT, person: PERSON, period: 0 };
    let person_proposal = Stored::PersonProposal(Box::new(tasks::PersonProposal {
        number: PROPOSAL,
        proposer: PERSON,
        project: PROJECT,
        action: tasks::PersonAction::Batch(Box::new([member])),
        reason: b"proposed work".as_slice().into(),
        state: tasks::PersonProposalState::Pending { since: world.env.wall },
    }));
    [(person_proposal, Vec::new()), (task_proposal, supporting)]
}

fn batch(record: &mut Stored) -> &mut [tasks::New] {
    if let Stored::PersonProposal(row) = record {
        let tasks::PersonAction::Batch(batch) = &mut row.action else { panic!("batch fixture") };
        batch
    } else if let Stored::Live(row) = record {
        let proposal = row.proposal.as_mut().expect("task proposal fixture");
        let tasks::ProposalAction::Batch(batch) = &mut proposal.action else { panic!("batch fixture") };
        batch
    } else {
        panic!("proposal fixture")
    }
}

/// Core refuses this shape live (`Malformed::EndedBy`); restore refuses it because only a restored store can hold it.
#[test]
fn restoring_a_proposal_requires_its_open_members_shape() {
    let rows = [
        ("person", Executor::Person(PersonAddress::Person(PERSON)), true, false),
        ("procedure", Executor::Procedure { connector: 1, code: 1 }, true, false),
        ("no conversation", Executor::Agent { charter: 1 }, false, false),
        ("agent conversation", Executor::Agent { charter: 1 }, true, true),
    ];
    for (original, supporting) in proposals() {
        for (name, executor, conversation, accepted) in rows {
            let mut record = original.clone();
            for member in batch(&mut record) {
                member.executor = executor;
                member.flags.ended_by = tasks::EndedBy::Requester;
                member.flags.conversation = conversation;
            }
            let mut domain = jig_tasks_world::domain(&LIMITS, SEED);
            let env = skein_lib::Env { limits: LIMITS, now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH };
            let mut out = Queue::uncapped();
            tasks::step(&mut domain, &env, Event::Restore { record: record.clone() }, &mut out);
            for record in &supporting {
                tasks::step(&mut domain, &env, Event::Restore { record: record.clone() }, &mut out);
            }
            tasks::step(&mut domain, &env, Event::Check, &mut out);
            tasks::step(&mut domain, &env, Event::Restored, &mut out);
            let mut refusals = Vec::new();
            let mut saves = Vec::new();
            while let Some(request) = out.pop() {
                if let tasks::Request::RestoreRefused { problem } = request {
                    refusals.push(problem.why);
                } else if let tasks::Request::Save { record } = request {
                    saves.push(record);
                }
            }
            if accepted {
                assert!(refusals.is_empty(), "{name}: {refusals:?}");
                if let Stored::PersonProposal(row) = record {
                    assert_eq!(domain.person_proposals().len(), 1, "{name}");
                    assert_eq!(domain.person_proposal(PERSON, PROPOSAL), Some(*row), "{name}");
                    assert!(domain.task(OWNER).is_none(), "{name}");
                } else if let Stored::Live(row) = record {
                    assert_eq!(domain.person_proposals().len(), 0, "{name}");
                    assert_eq!(domain.task(OWNER).expect("restored owner").proposal, row.proposal, "{name}");
                    assert!(domain.task(MEMBER).is_none(), "a proposal creates no member");
                } else {
                    panic!("proposal fixture")
                }
            } else {
                assert_eq!(refusals, [Refusal::Restore], "{name}");
                assert!(saves.is_empty(), "{name}");
                assert!(domain.task(OWNER).is_none(), "{name}");
                assert!(domain.task(MEMBER).is_none(), "{name}");
                assert_eq!(domain.person_proposals().len(), 0, "{name}");
                assert_eq!(domain.funding_records().len(), 0, "{name}");
            }
        }
    }
}

#[test]
fn restoring_a_proposal_keeps_members_without_repeating_admission() {
    let rows = [
        ("person", Executor::Person(PersonAddress::Person(PERSON)), true),
        ("procedure", Executor::Procedure { connector: 1, code: 1 }, true),
        ("no conversation", Executor::Agent { charter: 1 }, false),
        ("agent conversation", Executor::Agent { charter: 1 }, true),
    ];
    for (original, supporting) in proposals() {
        for (name, executor, conversation) in rows {
            let mut record = original.clone();
            for member in batch(&mut record) {
                member.executor = executor;
                member.flags.ended_by = tasks::EndedBy::Requester;
                member.flags.conversation = conversation;
            }
            let mut domain = jig_tasks_world::domain(&LIMITS, SEED);
            domain.start_restore();
            let env = skein_lib::Env { limits: LIMITS, now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH };
            let mut out = Queue::uncapped();
            tasks::step(&mut domain, &env, Event::Restore { record: record.clone() }, &mut out);
            for record in &supporting {
                tasks::step(&mut domain, &env, Event::Restore { record: record.clone() }, &mut out);
            }
            tasks::step(&mut domain, &env, Event::Check, &mut out);
            tasks::step(&mut domain, &env, Event::Restored, &mut out);
            let mut refusals = Vec::new();
            let mut saves = Vec::new();
            while let Some(request) = out.pop() {
                if let tasks::Request::RestoreRefused { problem } = request {
                    refusals.push(problem.why);
                } else if let tasks::Request::Save { record } = request {
                    saves.push(record);
                }
            }
            {
                assert!(refusals.is_empty(), "{name}: {refusals:?}");
                if let Stored::PersonProposal(row) = record {
                    assert_eq!(domain.person_proposals().len(), 1, "{name}");
                    assert_eq!(domain.person_proposal(PERSON, PROPOSAL), Some(*row), "{name}");
                    assert!(domain.task(OWNER).is_none(), "{name}");
                } else if let Stored::Live(row) = record {
                    assert_eq!(domain.person_proposals().len(), 0, "{name}");
                    assert_eq!(domain.task(OWNER).expect("restored owner").proposal, row.proposal, "{name}");
                    assert!(domain.task(MEMBER).is_none(), "a proposal creates no member");
                } else {
                    panic!("proposal fixture")
                }
            }
        }
    }
}

#[test]
fn nested_proposal_fit_reports_the_outer_key_and_exact_bytes() {
    for (original, _) in proposals() {
        for size in [64, 65] {
            let mut record = original.clone();
            let key = record.key();
            for member in batch(&mut record) {
                member.spec.words = vec![0; size].into_boxed_slice();
            }
            let mut domain = jig_tasks_world::domain(&LIMITS, SEED);
            domain.start_restore();
            let env = skein_lib::Env { limits: LIMITS, now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH };
            let mut out = Queue::uncapped();
            tasks::step(&mut domain, &env, Event::Restore { record }, &mut out);
            let expected = if size == 64 {
                None
            } else {
                Some(tasks::RestoreFailure {
                    key,
                    cause: tasks::RestoreCause::Bound { which: tasks::RestoreBound::SpecBytes, held: 65, limit: 64 },
                })
            };
            assert_eq!(domain.restore_failure(), expected);
            if size == 65 {
                assert!(matches!(out.pop(), Some(tasks::Request::RestoreRefused { .. })));
                assert!(out.is_empty());
                tasks::step(&mut domain, &env, Event::Check, &mut out);
                tasks::step(&mut domain, &env, Event::Restored, &mut out);
                assert_eq!(domain.restore_failure(), expected);
                assert!(out.is_empty());
                assert!(!domain.checked());
            } else {
                assert!(out.is_empty());
            }
        }
    }
}
