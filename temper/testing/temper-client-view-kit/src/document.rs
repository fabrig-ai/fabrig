use super as f;
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time, Wall};
use temper_client_domain::pages::Act;
use temper_client_domain::{self as d, pages::chat::document::Mode};
use temper_fixtures::client_documents as documents;

pub const TITLE: &[u8] = b"How Shop works";

pub const TASK: u64 = f::chat::TASK;

pub const LONG: u32 = 65_536;

pub fn limits() -> d::Limits {
    let mut limits = f::domain_limits();
    limits.client.domain.long_text = LONG;
    limits.client.domain.ask_bytes = 131_072;
    limits.client.domain.requests.bytes = 524_288;
    limits.client.domain.text = 512;
    limits.client.domain.turn_text = 2048;
    limits.client.domain.rows = 4;
    limits
}

pub fn activity(outcomes: &[&str]) -> c::ActivityPage {
    let mut rows = List::with_capacity(1);
    if !outcomes.is_empty() {
        let mut blocks = List::with_capacity(u32::try_from(outcomes.len()).unwrap());
        for (index, outcome) in outcomes.iter().enumerate() {
            blocks
                .push(c::Block::Call(c::Call {
                    number: u64::try_from(index).unwrap().checked_add(1).unwrap(),
                    tool: c::Text::new(Box::from(b"note".as_slice()), 512).unwrap(),
                    input: c::Text::new(Box::default(), 512).unwrap(),
                    outcome: Some(c::Text::new(Box::from(outcome.as_bytes()), 512).unwrap()),
                    failed: false,
                    objects: List::with_capacity(0),
                    parts: List::with_capacity(0),
                }))
                .unwrap();
        }
        let mut messages = List::with_capacity(1);
        messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).unwrap();
        rows.push(documents::turn(
            1,
            Wall::EPOCH,
            c::TurnName { attempt: 1, number: 1 },
            c::TurnRead::Decoded(c::Turn {
                messages,
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                llm: None,
                cut: false,
            }),
        ))
        .unwrap();
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProposalState {
    Ordinary,
    DesignSession,
    DraftOffered,
    EndedDraftOffered,
    UnsendableDraftOffered,
    NotProposed,
    Waiting,
    Accepted,
    Unknown,
}

pub const PROPOSAL: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(TASK), number: 12 };

pub const ACCEPTED_AT: Wall = Wall::from_nanos(50_580_000_000_000);

pub fn document_root(body: Option<&[u8]>, mode: Mode, outcomes: &[&str], state: ProposalState) -> d::Domain {
    activity_root(body, mode, activity(outcomes), state)
}

#[expect(
    clippy::too_many_lines,
    reason = "the fixture pumps the task, document and departed proposal through their public events"
)]
pub fn activity_root(body: Option<&[u8]>, mode: Mode, activity: c::ActivityPage, state: ProposalState) -> d::Domain {
    let limits = limits();
    let drafts = state == ProposalState::DraftOffered
        || state == ProposalState::EndedDraftOffered
        || state == ProposalState::UnsendableDraftOffered;
    let ways = d::forge::ways::Ways { drafts, ..d::forge::ways::Ways::STOCK };
    let (mut root, me_watch, watch, _, _, _, _, _, _) = f::root_at_with_values(
        d::Address::Chat { task: TASK },
        &limits,
        b"Shop",
        None,
        Some(activity),
        None,
        Some(&[]),
        Some(ways),
        false,
        true,
    );
    let source = f::chat::messaging_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    if state != ProposalState::Ordinary {
        let mut me = asking_me(&source);
        if state == ProposalState::DesignSession {
            let template = me.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap();
            template.number = 6;
            template.executor = c::TemplateExecutor::Charter(12);
        }
        if state == ProposalState::DraftOffered
            || state == ProposalState::EndedDraftOffered
            || state == ProposalState::UnsendableDraftOffered
        {
            let template = me.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap();
            template.tracked = Some(1);
            template.flags.conversation = false;
        }
        d::step(
            &mut root,
            &env,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: me_watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                    },
                },
            },
            &mut out,
        );
        if state != ProposalState::DraftOffered
            && state != ProposalState::EndedDraftOffered
            && state != ProposalState::UnsendableDraftOffered
        {
            let charter = if state == ProposalState::DesignSession { 12 } else { 1 };
            task.chip.executor = c::ChipExecutor::Agent { charter };
        }
        if state == ProposalState::UnsendableDraftOffered {
            task.acts.clear();
        }
        if state == ProposalState::EndedDraftOffered {
            task.chip.phase = c::Phase::Ended;
        }
    }
    if state == ProposalState::Waiting || state == ProposalState::Accepted || state == ProposalState::Unknown {
        task.proposals = List::with_capacity(1);
        task.proposals.push(PROPOSAL).unwrap();
    }
    let mut without_proposal = task.clone();
    without_proposal.proposals.clear();
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    let mut departed = false;
    for _ in 0_u32..64 {
        let Some(request) = out.pop() else {
            if !departed && (state == ProposalState::Accepted || state == ProposalState::Unknown) {
                departed = true;
                d::step(
                    &mut root,
                    &env,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch: watch.unwrap(),
                                event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(without_proposal.clone()))),
                            },
                        },
                    },
                    &mut out,
                );
                continue;
            }
            break;
        };
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Proposal { .. }) },
                },
        } = &request
        {
            let result = proposal_answer(state, departed);
            d::step(
                &mut root,
                &env,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone { read: *read, result: jig_client::Readout::Jig(result) },
                    },
                },
                &mut out,
            );
        }
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }) },
                },
        } = request
        {
            let mut rows = List::with_capacity(1);
            if let Some(body) = body {
                rows.push(c::Note {
                    number: 1,
                    revision: c::Revision { value: 2 },
                    scope,
                    description: c::DocumentText::complete(c::Text::new(Box::from(TITLE), 512).unwrap()),
                    body: c::DocumentText::complete(c::Text::new(Box::from(body), LONG).unwrap()),
                    writer: c::NoteAuthor::Task { task: TASK, attempt: 1 },
                    tasks: List::with_capacity(0),
                    changed: Some(Wall::EPOCH),
                    acts: List::with_capacity(0),
                    cut: false,
                })
                .unwrap();
            }
            d::step(
                &mut root,
                &env,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(
                                c::NotePage { rows, before: None, after: None, cut: false },
                            )))),
                        },
                    },
                },
                &mut out,
            );
        }
    }
    assert_eq!(root.pages().chat().unwrap().draft.is_some(), body.is_some());
    d::step(&mut root, &env, d::Event::Act { action: d::Action::Pages(Act::Mode { mode }) }, &mut out);
    root
}

pub fn asking_me(source: &d::Domain) -> c::Me {
    let mut me = source.pages().me(source.client()).unwrap().clone();
    let member = me.projects.get_mut(0).unwrap();
    let mut template = member.templates.get(0).unwrap().clone();
    template.number = 5;
    let mut templates = List::with_capacity(2);
    templates.push(member.templates.get(0).unwrap().clone()).unwrap();
    templates.push(template).unwrap();
    member.templates = templates;

    me
}

pub fn proposal_answer(state: ProposalState, departed: bool) -> c::ReadResult {
    if departed {
        if state == ProposalState::Unknown {
            c::ReadResult::None(c::ReadNone::Unknown)
        } else {
            c::ReadResult::Value(c::ReadValue::Ended(Box::new(c::Ended {
                key: c::ObjectKey::Proposal(PROPOSAL),
                value: c::EndedValue::Proposal(c::ProposalHistory {
                    reason: c::DocumentText::complete(
                        c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
                    ),
                    action: c::RetainedAction::Batch(List::with_capacity(0)),
                    number: PROPOSAL.number,
                    proposer: TASK,
                    as_holder: true,
                    state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(f::chat::PARTY) },
                }),
                at: Some(ACCEPTED_AT),
                by: None,
            })))
        }
    } else {
        c::ReadResult::Value(c::ReadValue::Proposal(Box::new(c::Proposal {
            key: PROPOSAL,
            project: 1,
            projects: List::from_boxed(Box::new([1])),
            reason: c::DocumentText::complete(c::Text::new(Box::from(b"Work this way".as_slice()), 512).unwrap()),
            action: c::ProposalAction::Batch(List::with_capacity(0)),
            as_holder: true,
            waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(f::chat::PARTY), since: Wall::EPOCH },
            funding: List::with_capacity(0),
            acts: List::with_capacity(0),
        })))
    }
}

pub fn offered(body: Option<&[u8]>, ended: bool) -> d::Domain {
    document_root(
        body,
        Mode::Chat,
        &[],
        if ended { ProposalState::EndedDraftOffered } else { ProposalState::DraftOffered },
    )
}
