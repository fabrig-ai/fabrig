//! Seeded person facts and responses (jig/docs/design/client/domain/testing.md, 4).
use super::Scripted;
use crate::{stories as fixture, test::list};
use jig_client_domain::contract::{
    ActKind, ActivityLine, Admission, Answer, AnswerOn, Ask, Beyond, Block, Call, DecideOn, Decision, DocumentText,
    Ending, Executor, HeldRole, HoldCategory, InboxGroup, InboxReadGroup, Message, ObjectKey, Offered, Outcome,
    Activity, ActivityLine, ChipEnding, ChipPhase, Ended, EndedValue,
    HistoryHolder, MessageKind, MessageProposalKind, DocumentOutcome, ProposalAction,
    ProposalChoice, ProposalHistoryState, Requester, RunEnd, RunWhy, PersonAddress,
    PersonAnswer, Phase, ProposeAction, Proposer, Read, ReadValue, Recipient, Result, ResultValue, RunStage, Runtime,
    Speaker, Task, Turn, TurnName, TurnRead, Usage,
};
use jig_client_scripted_engine::{Change, Rows, Rule, Source};
use skein_lib::{Time, Wall};

pub(super) fn profile(front: &mut Scripted) {
    let mut frame = fixture::frame(1);
    frame.deployment_role = Some(HeldRole::Owner);
    frame.projects.get_mut(0).expect("project").name = DocumentText::complete(fixture::text(b"Project 1"));
    let mut project = fixture::project();
    project.policy.period_spend = 1000;
    project.policy.escalation_role = Some(0);
    project.policy.roles.get_mut(0).expect("policy role").period_spend = 500;
    project.configured_period = 1;
    let mut deployment = fixture::deployment();
    let account = deployment.accounts.get_mut(0).expect("account");
    account.number = 0;
    account.generation = Some(1);
    for change in [Change::Me(frame), Change::Project(Box::new(project)), Change::Deployment(deployment)] {
        front.engine.act(Time::ZERO, Wall::EPOCH, change).expect("seed owner profile fits");
    }
}

pub(super) fn ended(task: Task, report: &[u8]) -> ReadValue {
    let mut document = fixture::reported(task, report);
    document.acts.clear();
    let result = Result {
        task: document.chip.clone(),
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText::complete(fixture::text(report)))),
        at: Some(Wall::EPOCH),
        acts: list([]),
        details: None,
    };
    ReadValue::Ended(Box::new(Ended {
        key: ObjectKey::Task(document.chip.task),
        value: EndedValue::Task { document: Box::new(document), result: Box::new(result) },
        at: Some(Wall::EPOCH),
        by: None,
    }))
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the seed adapter handles only operation variants selected at its entrance"
)]
pub(super) fn request(front: &mut Scripted, party: u64, ask: &Ask, wall: Wall) {
    role_request(front, party, ask);
    if proposal_request(front, party, ask) {
        return;
    }
    let (Ask::Stop { project, task } | Ask::Message { project, task, .. } | Ask::Release { project, task, .. }) = ask
    else {
        return;
    };
    if !front.chat_projects.contains(&(party, *project)) {
        return;
    }
    let Some(ReadValue::Task(document)) = front.engine.truth().objects.get(&ObjectKey::Task(*task)) else { return };
    let mut document = document.as_ref().clone();
    let (answer, changes) = match ask {
        Ask::Stop { .. } => {
            let mut held = fixture::held_chat(0).front().engine.truth().objects[&ObjectKey::Task(9)].clone();
            let ReadValue::Task(value) = &mut held else { unreachable!("held task") };
            value.chip.task = *task;
            value.chip.title = document.chip.title;
            value.project = *project;
            value.requester = Requester::Person(party);
            value.hold.as_mut().expect("hold").reason = HoldCategory::Placed { hold: 1 };
            value.chip.details.phase = ChipPhase::Held { reason: HoldCategory::Placed { hold: 1 }, revision: 4 };
            value.chip.details.details.hold.as_mut().expect("chip hold").reason = HoldCategory::Placed { hold: 1 };
            (
                DocumentOutcome::Stopped { task: *task },
                vec![Change::Object { project: *project, key: ObjectKey::Task(*task), value: held }],
            )
        }
        Ask::Message { .. } | Ask::Release { .. } => {
            let released = matches!(ask, Ask::Release { .. });
            let words = match ask {
                Ask::Message { words, .. } => words.clone(),
                Ask::Release { .. } => fixture::text(b"Released"),
                _ => unreachable!("message or release"),
            };
            if released {
                document.hold = None;
                document.chip.phase = Phase::Due;
                document.chip.details.phase = ChipPhase::Active(RunStage::Due);
                document.chip.details.details.hold = None;
                document.context.as_mut().expect("chat context").runtime = Runtime::Starting;
            }
            document.activity += 1;
            let mut rows = front
                .engine
                .truth()
                .sources
                .iter()
                .find_map(|source| match &source.rows {
                    Rows::Activity(rows)
                        if source.party == party && source.read == (Read::Activity { task: *task, before: None }) =>
                    {
                        Some(rows.clone())
                    }
                    _ => None,
                })
                .unwrap_or_default();
            rows.push(ActivityLine {
                source: Box::new(ActivityLine {
                    task: *task,
                    position: document.activity,
                    at: wall,
                    line: Activity::Words {
                        message: document.activity,
                        by: Requester::Person(party),
                        kind: MessageKind::Words,
                        words: words.clone(),
                    },
                }),
                decoded: None,
            });
            (
                if released {
                    DocumentOutcome::Released { task: *task }
                } else {
                    DocumentOutcome::Said { task: *task, message: document.activity }
                },
                vec![
                    Change::Object {
                        project: *project,
                        key: ObjectKey::Task(*task),
                        value: ReadValue::Task(Box::new(document)),
                    },
                    Change::Source(Source {
                        party,
                        read: Read::Activity { task: *task, before: None },
                        rows: Rows::Activity(rows),
                    }),
                ],
            )
        }
        _ => unreachable!("message or stop"),
    };
    front
        .engine
        .script(Rule { party: Some(party), ask: ask.clone(), answer: Answer::Done(Outcome::Document(answer)), changes })
        .expect("seeded response fits");
}

pub(super) fn proposal(front: &mut Scripted, words: &[u8], budget: u64) {
    proposal_for(front, 1, &[1], words, budget);
}

fn proposal_for(front: &mut Scripted, proposer: u64, holders: &[u64], words: &[u8], budget: u64) {
    let mut task = front.seed_chat(proposer, 1, b"Hello");
    let mut proposal = fixture::proposal();
    proposal.key.proposer = Proposer::Task(task.chip.task);
    proposal.reason = DocumentText::complete(fixture::text(b"this goal needs a holder"));
    proposal.waiting.holder = if holders.len() == 1 {
        HistoryHolder::Person(1)
    } else {
        HistoryHolder::Policy { project: 1, kind: MessageProposalKind::Batch }
    };
    let mut goal =
        fixture::project().recurring.get(0).expect("goal specification").batch.get(0).expect("one task").clone();
    goal.spec = task.spec.clone();
    goal.spec.specification.words = DocumentText::complete(fixture::text(words));
    goal.budget = budget;
    goal.authority.authority.spend = budget;
    proposal.action = ProposalAction::Batch(list([goal]));
    proposal.funding.clear();
    task.proposals = list([proposal.key]);
    task.activity = 3;
    let key = proposal.key;
    let turn = proposal_turn(words, key);
    for change in [
        Change::Object { project: 1, key: ObjectKey::Task(task.chip.task), value: ReadValue::Task(Box::new(task)) },
        Change::Object {
            project: 1,
            key: ObjectKey::Proposal(key),
            value: ReadValue::Proposal(Box::new(proposal)),
        },
        Change::Source(Source {
            party: 1,
            read: Read::Inbox { group: InboxReadGroup::Needs, project: None, kind: None, goal: None, after: None },
            rows: Rows::Inbox(vec![fixture::inbox_entry(
                ObjectKey::Proposal(key),
                1,
                InboxGroup::Proposals,
                1,
                Wall::EPOCH,
                None,
            )]),
        }),
        Change::Source(Source {
            party: proposer,
            read: Read::Activity { task: 1, before: None },
            rows: Rows::Activity(vec![
                ActivityLine {
                    source: Box::new(ActivityLine {
                        task: 1,
                        position: 3,
                        at: Wall::EPOCH,
                        line: Activity::RunEnded { attempt: 1, end: RunEnd::Parked, spent: 1 },
                    }),
                    decoded: None,
                },
                ActivityLine {
                    source: Box::new(ActivityLine {
                        task: 1,
                        position: 2,
                        at: Wall::EPOCH,
                        line: Activity::RunBegan { attempt: 1, why: RunWhy::Made, resumed: false },
                    }),
                    decoded: None,
                },
                fixture::turn_event(1, 1, TurnName { attempt: 1, number: 1 }, TurnRead::Decoded(turn)),
            ]),
        }),
    ] {
        front.engine.act(Time::ZERO, Wall::EPOCH, change).expect("pending goal seed fits");
    }
    for party in holders.iter().copied().filter(|party| *party != 1) {
        front
            .engine
            .act(
                Time::ZERO,
                Wall::EPOCH,
                Change::Source(Source {
                    party,
                    read: Read::Inbox {
                        group: InboxReadGroup::Needs,
                        project: None,
                        kind: None,
                        goal: None,
                        after: None,
                    },
                    rows: Rows::Inbox(vec![fixture::inbox_entry(
                        ObjectKey::Proposal(key),
                        1,
                        InboxGroup::Proposals,
                        1,
                        Wall::EPOCH,
                        None,
                    )]),
                }),
            )
            .expect("holder inbox fits");
    }
}

fn proposal_request(front: &mut Scripted, party: u64, ask: &Ask) -> bool {
    let Ask::Decide { project, on: DecideOn::Proposal { proposal: key, decision: Decision::Accepted, .. } } = ask
    else {
        return false;
    };
    if !front.chat_projects.contains(&(party, *project)) {
        return false;
    }
    let Some(ReadValue::Proposal(proposal)) = front.engine.truth().objects.get(&ObjectKey::Proposal(*key)) else {
        return false;
    };
    let mut decided =
        fixture::decided_proposal(proposal, ProposalHistoryState::Accepted { by: Requester::Person(party) });
    decided.by = Some(Requester::Person(party));
    let Proposer::Task(task) = key.proposer else { return false };
    let mut changes = vec![
        Change::Object {
            project: *project,
            key: ObjectKey::Proposal(*key),
            value: ReadValue::Ended(Box::new(decided)),
        },
        Change::Source(Source {
            party,
            read: Read::Inbox { group: InboxReadGroup::Needs, project: None, kind: None, goal: None, after: None },
            rows: Rows::Inbox(vec![]),
        }),
    ];
    if let Some(ReadValue::Task(document)) = front.engine.truth().objects.get(&ObjectKey::Task(task)) {
        let mut document = document.as_ref().clone();
        document.activity += 1;
        document.proposals.clear();
        changes.push(Change::Object {
            project: *project,
            key: ObjectKey::Task(task),
            value: ReadValue::Task(Box::new(document)),
        });
    }
    let mut source = front
        .engine
        .truth()
        .sources
        .iter()
        .find(|source| source.read == (Read::Activity { task, before: None }))
        .expect("proposal activity")
        .clone();
    let Rows::Activity(rows) = &mut source.rows else { unreachable!("activity source") };
    rows.push(ActivityLine {
        source: Box::new(ActivityLine {
            task,
            position: 4,
            at: Wall::EPOCH,
            line: Activity::Words {
                message: 2,
                by: Requester::Person(party),
                kind: MessageKind::ProposalDecision { proposal: key.number, accepted: true },
                words: fixture::text(b"Accepted"),
            },
        }),
        decoded: None,
    });
    changes.push(Change::Source(source));
    front
        .engine
        .script(Rule {
            party: Some(party),
            ask: ask.clone(),
            answer: Answer::Done(Outcome::Document(DocumentOutcome::ProposalDecided {
                proposal: *key,
                by: party,
                at: Wall::EPOCH,
                choice: ProposalChoice::Accepted,
                repeated: false,
            })),
            changes,
        })
        .expect("proposal decision fits");
    true
}

pub(super) fn roles(front: &mut Scripted, story: crate::harness::seed::RoleStory) {
    use crate::harness::seed::RoleStory;
    for party in 1..=4 {
        let mut frame = fixture::frame(party);
        let member = frame.projects.get_mut(0).expect("project");
        member.role = if party == 1 {
            HeldRole::Owner
        } else {
            HeldRole::Policy { role: u32::try_from(party - 1).expect("role") }
        };
        let budget = if party <= 2 { 500 } else { 50 };
        let creating = member.creating.as_mut().expect("creating");
        creating.within.pool_left = 500;
        creating.within.budget = budget;
        creating.within.maximum_budget = 1000;
        creating.within.authority.authority.spend = budget;
        creating.beyond = Beyond::Proposed { to: Recipient::Role { project: 1, role: 0 } };
        if party == 4 {
            member.creating = None;
        }
        front.engine.act(Time::ZERO, Wall::EPOCH, Change::Me(frame)).expect("role frame fits");
        front.engine.act(Time::ZERO, Wall::EPOCH, Change::SignedOut { sign_in: party }).expect("signed-out role");
        front.chat_projects.insert((party, 1));
    }
    front
        .engine
        .act(Time::ZERO, Wall::EPOCH, Change::Project(Box::new(fixture::project())))
        .expect("role project fits");
    match story {
        RoleStory::Proposal => proposal_for(front, 3, &[1, 2], b"decide this goal", 150),
        RoleStory::Choice => choice(front),
        RoleStory::Goal | RoleStory::Observer => {}
    }
}

fn choice(front: &mut Scripted) {
    let mut source = fixture::person_confirmation::world(0, fixture::person_confirmation::Form::Choice);
    let ReadValue::PersonTask(document) = &source.front().engine.truth().objects[&ObjectKey::PersonTask(9)] else {
        unreachable!("person task fixture")
    };
    let mut document = document.as_ref().clone();
    document.task.task = 2;
    document.task.executor = Executor::Role { project: 1, role: 1 };
    document.address = PersonAddress::Role(1);
    document.taken_by = None;
    document.spec.specification.words = DocumentText::complete(fixture::text(b"first report or second report"));
    document.acts = list([Offered { effect: None, act: ActKind::Take, admission: Admission::Allowed }]);
    for change in [
        Change::Object {
            project: 1,
            key: ObjectKey::PersonTask(2),
            value: ReadValue::PersonTask(Box::new(document)),
        },
        Change::Source(Source {
            party: 2,
            read: Read::Inbox { group: InboxReadGroup::Needs, project: None, kind: None, goal: None, after: None },
            rows: Rows::Inbox(vec![fixture::inbox_entry(
                ObjectKey::PersonTask(2),
                1,
                InboxGroup::PersonTasks,
                1,
                Wall::EPOCH,
                None,
            )]),
        }),
    ] {
        front.engine.act(Time::ZERO, Wall::EPOCH, change).expect("person task seed fits");
    }
}

#[expect(clippy::wildcard_enum_match_arm, reason = "the person seed adapter handles only take and answer operations")]
fn role_request(front: &mut Scripted, party: u64, ask: &Ask) {
    if goal_proposal_request(front, party, ask) {
        return;
    }
    let (Ask::Take { project, task } | Ask::Answer { project, on: AnswerOn::Person { task, .. } }) = ask else {
        return;
    };
    if !front.chat_projects.contains(&(party, *project)) {
        return;
    }
    let Some(ReadValue::PersonTask(document)) = front.engine.truth().objects.get(&ObjectKey::PersonTask(*task))
    else {
        return;
    };
    let mut document = document.as_ref().clone();
    let (answer, changes) = match ask {
        Ask::Take { .. } => {
            document.taken_by = Some(party);
            document.acts = list([Offered { effect: None, act: ActKind::Answer, admission: Admission::Allowed }]);
            (
                DocumentOutcome::PersonTaken { task: *task },
                vec![Change::Object {
                    project: *project,
                    key: ObjectKey::PersonTask(*task),
                    value: ReadValue::PersonTask(Box::new(document)),
                }],
            )
        }
        Ask::Answer { on: AnswerOn::Person { answer, .. }, .. } => {
            let value = match answer {
                PersonAnswer::Choice { option, reason } => ResultValue::OwnerVerdict {
                    verdict: *option,
                    words: DocumentText::complete(reason.clone()),
                    fields: list([]),
                },
                PersonAnswer::Words(words) => ResultValue::Report(DocumentText::complete(words.clone())),
                PersonAnswer::Approved { state, words } => ResultValue::Approved {
                    state: state.clone(),
                    words: words.as_ref().map(|words| DocumentText::complete(words.clone())),
                },
                PersonAnswer::Cannot { reason } => ResultValue::Failure(DocumentText::complete(reason.clone())),
            };
            document.acts.clear();
            document.task.phase = Phase::Ended;
            document.task.details.phase = ChipPhase::Ended;
            document.task.details.details.ended_at = Some(Wall::EPOCH);
            document.task.details.details.ending = Some(ChipEnding::Done(value.clone()));
            let result = Result {
                task: document.task.clone(),
                ending: Ending::Done,
                value: Some(value),
                at: Some(Wall::EPOCH),
                acts: list([]),
                details: None,
            };
            (
                DocumentOutcome::PersonAnswered { task: *task },
                vec![
                    Change::Object {
                        project: *project,
                        key: ObjectKey::PersonTask(*task),
                        value: ReadValue::Ended(Box::new(Ended {
                            key: ObjectKey::PersonTask(*task),
                            value: EndedValue::PersonTask {
                                document: Box::new(document),
                                result: Box::new(result),
                            },
                            at: Some(Wall::EPOCH),
                            by: Some(Requester::Person(party)),
                        })),
                    },
                    Change::Source(Source {
                        party,
                        read: Read::Inbox {
                            group: InboxReadGroup::Needs,
                            project: None,
                            kind: None,
                            goal: None,
                            after: None,
                        },
                        rows: Rows::Inbox(vec![]),
                    }),
                ],
            )
        }
        _ => unreachable!("take or answer"),
    };
    front
        .engine
        .script(Rule { party: Some(party), ask: ask.clone(), answer: Answer::Done(Outcome::Document(answer)), changes })
        .expect("person response fits");
}

fn proposal_turn(words: &[u8], key: jig_client_domain::contract::ProposalKey) -> Turn {
    Turn {
        messages: list([Message {
            speaker: Speaker::Assistant,
            blocks: list([Block::Call(Call {
                number: 1,
                tool: fixture::text(b"propose_goal"),
                input: fixture::text(words),
                outcome: Some(fixture::text(b"Proposed")),
                failed: false,
                objects: list([ObjectKey::Proposal(key)]),
                parts: list([]),
            })]),
        }]),
        used: Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
        spent: 1,
        read_through: None,
        cut: false,
    }
}

fn goal_proposal_request(front: &mut Scripted, party: u64, ask: &Ask) -> bool {
    if let Ask::Propose { project, action: ProposeAction::Batch { .. }, .. } = ask {
        if front.chat_projects.contains(&(party, *project)) {
            front
                .engine
                .script(Rule {
                    party: Some(party),
                    ask: ask.clone(),
                    answer: Answer::Done(Outcome::Document(DocumentOutcome::Proposed { proposal: 1, to: 0 })),
                    changes: vec![],
                })
                .expect("proposed goal response fits");
        }
        return true;
    }
    false
}
