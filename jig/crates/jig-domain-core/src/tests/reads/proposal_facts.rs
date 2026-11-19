//! Decided cards read the same reason and scope as their live proposal.
use crate::tests::*;
use crate::{
    reads,
    routing::{Work, settings::dispatch},
    settings as s,
};

const PROJECT: u32 = 1;
const KIND: u32 = 7;
const REASON: &[u8] = b"keep the project settings reason";

#[test]
fn retained_party_actions_keep_their_history_or_single_goal_form() {
    for state in [
        tasks::PersonProposalState::Pending { since: Wall::from_nanos(41) },
        tasks::PersonProposalState::Accepted { by: tasks::Party::Person(2) },
        tasks::PersonProposalState::Rejected {
            by: tasks::Party::Person(2),
            reason: b"no".as_slice().into(),
            message: 59,
            at: Wall::from_nanos(61),
        },
        tasks::PersonProposalState::Withdrawn { at: Wall::from_nanos(67) },
    ] {
        let (core, _) = watch_core();
        let goal = new_row(&core, 37, tasks::Funder::Period { project: 1, period: 1 }, Some(3));
        let change = tasks::Amendment {
            tracked: None,
            addressed: Some(tasks::AddressChange { revision: 47, address: tasks::PersonAddress::Role(5) }),
            instructions: Some(b"new instructions".as_slice().into()),
            release: Some(53),
            spec: Some(goal.spec.clone()),
            wake: Some(goal.wake),
            dependencies: Some(Box::new([43])),
            authority: Some(goal.authority.clone()),
            llm: None,
            reason: b"widen authority".as_slice().into(),
        };
        for action in [
            tasks::PersonAction::Amend { task: 37, revision: 71, amendment: Box::new(change.clone()) },
            tasks::PersonAction::Batch(Box::new([goal.clone()])),
            tasks::PersonAction::Batch(Box::new([goal.clone(), tasks::New { number: 38, ..goal.clone() }])),
        ] {
            let (mut core, limits) = watch_core();
            let proposal = tasks::PersonProposal {
                number: 13,
                proposer: 1,
                project: 1,
                action: action.clone(),
                reason: REASON.into(),
                state: state.clone(),
            };
            let mut row = stored_row(Record::Tasks(tasks::Stored::DecidedProposal(Box::new(proposal.clone()))), None);
            if let tasks::PersonAction::Batch(batch) = &action
                && batch.len() == 1
            {
                row.related = Some(Record::Core(CoreRecord::ProposalDecision(ProposalDecisionRecord {
                    project: 1,
                    proposer: Proposer::Party(1),
                    proposal: 13,
                    kind: tasks::ProposalKind::Batch,
                    by: 2,
                    at: Wall::from_nanos(73),
                    choice: people::ProposalChoice::Accepted,
                    created: None,
                })));
            }
            let Held::ReadLoad { load, .. } = read_start(
                &mut core,
                &limits,
                10,
                reads::Read::Proposal { proposer: Proposer::Party(1), proposal: 13 },
            ) else {
                panic!("retained load")
            };
            let reads::Answer::Ended(ended) =
                loaded_answer(&mut core, &limits, load, reads::Loaded::Page { rows: Box::new([row]), next: None })
                    .unwrap()
            else {
                panic!("retained party proposal")
            };
            match ended.value {
                reads::EndedValue::PartyProposal(history) => {
                    assert_eq!(history.proposer, 1);
                    assert_eq!(history.number, 13);
                    assert_eq!(history.reason.bytes.as_ref(), REASON);
                    assert!(history.as_holder);
                    let kind = match (&action, &history.action) {
                        (tasks::PersonAction::Amend { .. }, RetainedAction::Amend { task, change: retained }) => {
                            assert_eq!(*task, 37);
                            assert_eq!(retained.addressed, change.addressed);
                            assert_eq!(retained.instructions.as_ref().unwrap().bytes.as_ref(), b"new instructions");
                            assert_eq!(retained.release, Some(53));
                            assert_eq!(retained.authority.as_ref().unwrap().ceiling.spend, goal.authority.budget.spend);
                            assert_eq!(retained.spec.as_ref().unwrap().words.bytes, goal.spec.words);
                            assert_eq!(retained.wake, change.wake);
                            assert_eq!(retained.dependencies.as_ref().unwrap().as_slice(), &[43]);
                            assert_eq!(retained.reason.bytes.as_ref(), b"widen authority");
                            assert!(!retained.cut);
                            tasks::ProposalKind::Amend
                        }
                        (tasks::PersonAction::Batch(batch), RetainedAction::Batch(kept)) => {
                            if batch.len() == 1 {
                                let tasks::PersonProposalState::Withdrawn { .. } = state else {
                                    panic!("single goal history is withdrawn")
                                };
                            }
                            assert_eq!(kept.len(), u32::try_from(batch.len()).unwrap());
                            assert_eq!(kept.get(0).unwrap().number, 37);
                            assert_eq!(kept.get(0).unwrap().spec.words.bytes, goal.spec.words);
                            assert_eq!(kept.get(0).unwrap().funder, goal.funder);
                            tasks::ProposalKind::Batch
                        }
                        values => panic!("retained action {values:?}"),
                    };
                    let expected = match &state {
                        tasks::PersonProposalState::Pending { since } => {
                            ProposalHistoryState::Pending(ProposalWaiting {
                                holder: tasks::ProposalHolder::Policy { project: 1, kind },
                                since: *since,
                            })
                        }
                        tasks::PersonProposalState::Accepted { by } => ProposalHistoryState::Accepted { by: *by },
                        tasks::PersonProposalState::Rejected { by, reason, .. } => ProposalHistoryState::Rejected {
                            by: *by,
                            reason: Text { bytes: reason.clone(), cut: false },
                        },
                        tasks::PersonProposalState::Withdrawn { .. } => ProposalHistoryState::Withdrawn,
                        tasks::PersonProposalState::SettingsAccepted { .. } => unreachable!(),
                    };
                    assert_eq!(history.state, expected);
                    assert_eq!(
                        ended.by,
                        match state {
                            tasks::PersonProposalState::Withdrawn { .. } => Some(tasks::Party::Person(1)),
                            tasks::PersonProposalState::Pending { .. } => None,
                            tasks::PersonProposalState::Accepted { by }
                            | tasks::PersonProposalState::Rejected { by, .. }
                            | tasks::PersonProposalState::SettingsAccepted { by, .. } => Some(by),
                        }
                    );
                }
                reads::EndedValue::PersonProposal { proposal: kept, decision } => {
                    let tasks::PersonAction::Batch(batch) = &action else { panic!("single goal") };
                    assert_eq!(batch.len(), 1);
                    match state {
                        tasks::PersonProposalState::Withdrawn { .. } => panic!("withdrawn proposal needs history"),
                        tasks::PersonProposalState::Pending { .. }
                        | tasks::PersonProposalState::Accepted { .. }
                        | tasks::PersonProposalState::Rejected { .. } => {}
                        tasks::PersonProposalState::SettingsAccepted { .. } => unreachable!(),
                    }
                    assert_eq!(*kept, proposal);
                    assert!(decision.is_some());
                }
                value @ (reads::EndedValue::Task { .. }
                | reads::EndedValue::Proposal(_)
                | reads::EndedValue::Escalation { .. }
                | reads::EndedValue::Question { .. }
                | reads::EndedValue::PersonTask { .. }
                | reads::EndedValue::Reply(_)
                | reads::EndedValue::Turn(_)
                | reads::EndedValue::WithheldTurn(_)) => panic!("party form {value:?}"),
            }
        }
    }
}

#[test]
fn a_decided_settings_proposal_keeps_its_reason_and_projects() {
    for state in [
        tasks::ProposalState::Accepted { by: tasks::Party::Person(1) },
        tasks::ProposalState::Rejected { by: tasks::Party::Person(1), reason: b"no".as_slice().into() },
    ] {
        let (mut core, mut limits) = watch_core();
        limits.documents.text_bytes = 8;
        let identity = core.people.identity(1).unwrap().clone();
        core.people =
            people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider, Box::new([]));
        core.people.start_restore(Wall::EPOCH);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
        for record in [
            people::Stored::Person { number: 1, identity },
            people::Stored::SignIn {
                number: 10,
                person: 1,
                expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
            },
            people::Stored::Roles {
                project: 0,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            },
            people::Stored::Roles {
                project: PROJECT,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            },
        ] {
            people::step(&mut core.people, &env, people::Event::Restore { record }, &mut Queue::uncapped());
        }
        finish_people(&mut core.people, &env);
        core.kinds = Box::new([s::Kind {
            number: KIND,
            scope: s::KindScope::Project,
            connector: 1,
            value_bytes: 64,
            per_scope: 4,
            charters: 0,
        }]);
        let mut work = Queue::uncapped();
        let output = dispatch::propose_run(
            &mut core,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            &mut work,
            ReplyTo::new(Token::new(91)),
            CallKey { task: 1, attempt: 7, completion: 1, position: 1 },
            s::Proposed {
                against: 1,
                reason: REASON.into(),
                items: Box::new([s::Item {
                    setting: s::Setting::Application { kind: KIND, scope: PROJECT, number: 1 },
                    edit: s::Edit::Set(s::Value::Application(b"value".as_slice().into())),
                }]),
            },
        );
        let Some(Work::Event(Event::Tasks(tasks::Event::Propose { mut proposal, .. }))) = work.pop() else {
            panic!("task proposal admitted: {output:?}")
        };
        proposal.state = state.clone();
        let number = proposal.number;
        let change = match state {
            tasks::ProposalState::Accepted { .. } => tasks::Change::ProposalAccepted,
            tasks::ProposalState::Rejected { .. } => tasks::Change::ProposalRejected,
            tasks::ProposalState::Pending { .. } | tasks::ProposalState::Withdrawn => unreachable!(),
        };
        let row = stored_row(
            Record::Tasks(tasks::Stored::History(tasks::History {
                task: 1,
                revision: 1,
                by: tasks::Party::Person(1),
                change,
                reason: b"outer history".as_slice().into(),
                proposal: Some(Box::new(proposal)),
                amendment: None,
                at: Wall::EPOCH,
            })),
            Some(core.tasks.task(1).unwrap().clone()),
        );
        let Held::ReadLoad { load, .. } =
            read_start(&mut core, &limits, 10, reads::Read::Proposal { proposer: Proposer::Task(1), proposal: number })
        else {
            panic!("retained proposal load")
        };
        let reads::Answer::Ended(ended) =
            loaded_answer(&mut core, &limits, load, reads::Loaded::Page { rows: Box::new([row]), next: None }).unwrap()
        else {
            panic!("ended proposal")
        };
        let reads::EndedValue::Proposal(proposal) = ended.value else { panic!("proposal facts") };
        assert_eq!(proposal.reason, Text { bytes: REASON.into(), cut: false });
        let RetainedAction::Settings { projects, .. } = proposal.action else { panic!("settings facts") };
        assert_eq!(projects.as_slice(), &[PROJECT]);
    }
}
