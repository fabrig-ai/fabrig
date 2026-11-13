use super as fixtures;
use jig_client::contract::*;
use skein_lib::{Env, List, Queue, Time, Token, Wall};
use temper_client_domain::{ChatsFilter, Domain, Request};

pub const TASK: u64 = 9;

pub const TEXT: u32 = 32;

pub const SPENT: u64 = 7;

pub const BELOW: u64 = 2;

pub const BUDGET: u64 = 100;

pub const RESERVED: u64 = 0;

pub const LEFT: u64 = 91;

pub const LABEL: u32 = 1;

pub const DELEGATES: u32 = 3;

pub const EMPTY: u32 = 0;

pub const ACTIVITY: u64 = 1;

#[must_use]
pub fn line() -> TaskLine {
    let budget = Budget {
        numbers: Numbers { budget: BUDGET, spent: SPENT, spent_below: BELOW, reserved: RESERVED },
        left: LEFT,
        funder: Funder::Task(TASK),
        deadline: None,
    };
    let details = ChipDetails {
        dependencies: List::with_capacity(EMPTY),
        project: fixtures::PROJECT,
        executor: Executor::Person(PersonAddress::Party(73)),
        phase_since: Wall::EPOCH,
        run_since: None,
        attempt: None,
        retry: None,
        wake: WakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        person_wait: None,
        hold: None,
        budget: budget.clone(),
        activity: ACTIVITY,
        resources: List::with_capacity(EMPTY),
        resources_cut: false,
        resources_observed: false,
        procedure: None,
        live_delegates: DELEGATES,
        closing_connectors: None,
        ending: None,
        ended_at: None,
        observation: None,
    };
    TaskLine {
        chip: Chip {
            task: TASK,
            title: DocumentText::complete(Text::new(Box::from(*b"Hello"), TEXT).expect("title")),
            executor: ChipExecutor::Party { party: 73 },
            phase: Phase::Idle,
            details: Box::new(ChipPhaseFacts { phase: ChipPhase::Active(RunStage::Idle), details: Box::new(details) }),
        },
        project: fixtures::PROJECT,
        requester: Requester::Person(73),
        tracked: None,
        label: LABEL,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: true, listed: false, retention: Retention::Kept },
        changed: Wall::EPOCH,
        last_activity: Wall::EPOCH,
        ended_at: None,
        budget,
        waits_on: WaitsOn::Nothing,
        phases: PhaseCounts { running: DELEGATES, ..PhaseCounts::ZERO },
        delegates_done: 0,
        delegates_total: 0,
        held_below: EMPTY,
        conversation: None,
    }
}

#[must_use]
pub fn row_status_styles(token: skein_ui::Style) -> Box<[skein_ui::Style]> {
    if token == crate::look::STATUS_YOURS || token == crate::look::STATUS_WAITING {
        Box::from([token, crate::look::STATE_NEEDS])
    } else {
        Box::from([token])
    }
}

pub const HELD_STATUSES: [(HoldCategory, &[u8]); 3] = [
    (
        HoldCategory::Exceeded(Exceeded { bound: Bound::Reasoning, value: 2048 }),
        b"Held: the model's reasoning went past its configured allowance of 2048 bytes",
    ),
    (
        HoldCategory::Exceeded(Exceeded { bound: Bound::History, value: 2048 }),
        b"Held: the run's history no longer fits its configured allowance, even compacted",
    ),
    (HoldCategory::Failures { class: HoldFailure::Run }, b"Held"),
];

#[must_use]
pub fn status_rows() -> [(TaskLine, skein_ui::Style); 11] {
    use crate::look;
    let mut rows = [
        (line(), look::STATUS_YOURS),
        (line(), look::STATUS_HELD),
        (line(), look::STATUS_RETRYING),
        (line(), look::STATUS_ACTIVE),
        (line(), look::STATUS_ACTIVE),
        (line(), look::STATUS_ACTIVE),
        (line(), look::STATUS_ACTIVE),
        (line(), look::STATUS_WAITING),
        (line(), look::STATUS_QUIET),
        (line(), look::STATUS_QUIET),
        (line(), look::STATUS_QUIET),
    ];
    for (index, (line, _)) in rows.iter_mut().enumerate() {
        let ChipPhaseFacts { phase, details } = line.chip.details.as_mut();
        details.ended_at = Some(Wall::EPOCH);
        details.retry =
            Some(RetryNotice::OwnerFailed { class: HoldFailure::Lost, count: 2, allowance: 3, reason: None });
        *phase = match index {
            1 => ChipPhase::Held { reason: HoldCategory::Stopped, revision: 1 },
            2 => ChipPhase::Active(RunStage::BackingOff { until: Wall::EPOCH }),
            3 => ChipPhase::Active(RunStage::Due),
            4..=6 => ChipPhase::Active(RunStage::Running { attempt: 1 }),
            9 => ChipPhase::Closing(ClosingStage::Delegates),
            10 => ChipPhase::Ended,
            _ => ChipPhase::Active(RunStage::Idle),
        };
        if index == 0 || index == 7 {
            line.waits_on = WaitsOn::Party {
                whom: Whom::Person(if index == 0 { 73 } else { 74 }),
                entries: 1,
                task: TASK,
                kind: WaitedFor::Question,
            };
        }
        if (4..=6).contains(&index) {
            details.observation = Some(ChipObservation {
                channel: Token::new(1),
                work: match index {
                    4 => ChipWork::Writing,
                    5 => ChipWork::Tool { call: Box::from(*b"call"), tool: Box::from(*b"search") },
                    _ => ChipWork::Active,
                },
                since: Wall::EPOCH,
                reported_at: Wall::EPOCH,
                started: None,
            });
        }
    }
    rows
}

pub fn open_chats(
    root: &mut Domain,
    env: &Env<temper_client_domain::Limits>,
    out: &mut Queue<Request>,
) -> (Token, Token) {
    use temper_client_domain::{Address, Event};
    temper_client_domain::step(
        root,
        env,
        Event::Went { place: temper_client_domain::Place::Address(Address::Chats { filter: ChatsFilter::All }) },
        out,
    );
    let mut watch = None;
    let mut read = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open { watch: token, subject: jig_client::Watchable::Jig(Subject::Tasks { .. }) },
                },
        } = request
        {
            watch = Some(token);
        } else if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read: token, what: jig_client::Readable::Jig(Read::Tasks { .. }) },
                },
        } = request
        {
            read = Some(token);
        }
    }
    (watch.expect("live watch"), read.expect("closed read"))
}

pub fn deliver_live(
    root: &mut Domain,
    env: &Env<temper_client_domain::Limits>,
    out: &mut Queue<Request>,
    watch: Token,
    rows: List<TaskLine>,
) {
    use temper_client_domain::Event;
    temper_client_domain::step(
        root,
        env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows, more: false }))),
                },
            },
        },
        out,
    );
}

#[must_use]
pub fn listed_root() -> Domain {
    let mut ended = line();
    ended.chip.task = 8;
    ended.ended_at = Some(Wall::EPOCH);
    listed(&[line()], &[ended], false)
}

#[must_use]
pub fn listed(live: &[TaskLine], closed: &[TaskLine], more: bool) -> Domain {
    use temper_client_domain::{Address, Event};
    let limits = fixtures::domain_limits();
    let (mut root, me_watch, _) =
        fixtures::root_at(Address::Inbox { filter: temper_client_domain::address::Filter::ALL }, &limits);
    let template = fixtures::chat::messaging_root();
    let me = template.pages().me(template.client()).expect("me").clone();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: WatchEvent::Delivered(Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    let (watch, read) = open_chats(&mut root, &env, &mut out);

    let mut live_rows = List::with_capacity(u32::try_from(live.len()).unwrap());
    for line in live {
        live_rows.push(line.clone()).expect("live row");
    }
    deliver_live(&mut root, &env, &mut out, watch, live_rows);
    let mut closed_rows = List::with_capacity(u32::try_from(closed.len()).unwrap());
    for line in closed {
        closed_rows.push(line.clone()).expect("closed row");
    }
    // A live update refreshes the first ended page; answer the latest read.
    let mut read = read;
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read: token, what: jig_client::Readable::Jig(Read::Tasks { .. }) },
                },
        } = request
        {
            read = token;
        }
    }
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: EndedGroup::Rows {
                            rows: closed_rows,
                            after: if more { Some(Cursor::new(Box::from(*b"next"), 8).unwrap()) } else { None },
                            cut: false,
                        },
                    })))),
                },
            },
        },
        &mut out,
    );
    root
}

#[must_use]
pub fn deployment_chats_root(welcome: Option<bool>, me: Option<Me>) -> Domain {
    use temper_client_domain::{Address, Event};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let limits = fixtures::domain_limits();
    let env = Env { now: Time::ZERO, wall: fixtures::NOW, limits };
    let mut root = temper_client_domain::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::uncapped();
    for event in [
        Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        Event::Start {
            address: Address::Chats { filter: ChatsFilter::All },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } } },
    ] {
        temper_client_domain::step(&mut root, &env, event, &mut out);
    }
    while let Some(request) = out.pop() {
        match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(Subject::Me) },
                    },
            } => {
                if let Some(me) = &me {
                    temper_client_domain::step(
                        &mut root,
                        &env,
                        Event::Client {
                            event: jig_client::Event::Up {
                                up: jig_client::Up::Watched {
                                    watch,
                                    event: WatchEvent::Delivered(Delivery::Me(Box::new(me.clone()))),
                                },
                            },
                        },
                        &mut out,
                    );
                }
            }
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } if wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read() == &w::Read::Deployment => {
                if let Some(welcome) = welcome {
                    fixtures::deliver_deployment(&mut root, read, welcome, false);
                }
            }
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Copy { .. }
            | Request::Push(_)
            | Request::Fact { .. } => {}
        }
    }
    root
}

#[must_use]
pub fn no_project_root(welcome: Option<bool>) -> Domain {
    let template = fixtures::root_in_project(None);
    let mut me = template.pages().me(template.client()).unwrap().clone();
    me.projects.clear();
    deployment_chats_root(welcome, Some(me))
}
