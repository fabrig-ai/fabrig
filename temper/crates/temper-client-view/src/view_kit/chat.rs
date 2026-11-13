use super as fixtures;
use jig_client::{contract as c, thread::Thread};
use skein_lib::{List, Wall};

pub const TASK: u64 = 9;
pub const ATTEMPT: u64 = 23;
pub const TURN: u32 = 1;
pub const RETRY: u32 = 0;
pub const OFFSET: u32 = 0;
pub const COMMITTED: u32 = 0;
pub const LINES: u32 = 4;
pub const BYTES: u64 = 65536;
pub const TEXT: u32 = 32;
pub const MESSAGE: u64 = 3;
pub const PARTY: u64 = 73;
pub const DELEGATE: u64 = 12;
pub const OTHER_DELEGATE: u64 = 13;
pub const CLOSE_GRACE: skein_lib::Duration = skein_lib::Duration::from_secs(5);
pub const CLOSE_OPERATIONS: u32 = 3;
pub const FRACTIONAL_GRACE: skein_lib::Duration = skein_lib::Duration::from_millis(125);
pub const GRACES: [(Option<skein_lib::Duration>, &[u8]); 3] = [
    (Some(CLOSE_GRACE), b"gets 5 seconds to finish, then stops."),
    (Some(FRACTIONAL_GRACE), b"gets 125000000 nanoseconds to finish, then stops."),
    (None, b"The run under way stops now."),
];
pub const CHAT_LABEL: u32 = 7;
pub const CHAT_TEMPLATE: u32 = 1;
pub const CHAT_CHARTER: u32 = 1;
pub const SINGLE_TEMPLATE: u32 = 1;
pub const EMPTY_PARAMETERS: u32 = 0;
pub const MESSAGING_OFFERS: u32 = 2;
pub const PROPOSAL: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(DELEGATE), number: 3 };
pub const PROPOSAL_OUTPUTS: u32 = 256;
pub const PROPOSAL_NOW: skein_lib::Time = skein_lib::Time::ZERO;
pub const PROPOSAL_OFFERS: u32 = 2;
pub const EMPTY_BATCH: u32 = 0;

pub fn provisional() -> Thread {
    provisional_with_text(b"Hello")
}
pub fn provisional_with_text(words: &[u8]) -> Thread {
    let mut thread =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
    let text = c::RunText {
        writing: None,
        attempt: ATTEMPT,
        began: Some(Wall::EPOCH),
        turn: c::ProvisionalTurnName { attempt: ATTEMPT, number: TURN, retry: RETRY },
        from: OFFSET,
        through: u32::try_from(words.len()).expect("run text size fits"),
        text: c::Text::new(Box::from(words), u32::try_from(BYTES).expect("byte bound fits")).expect("text"),
        cut: false,
        call: None,
    };
    thread.run(
        &c::RunState { task: TASK, attempt: Some(ATTEMPT), committed_turn: COMMITTED, writing: None, call: None },
        Some(&text),
    );
    thread
}
pub fn chat_root() -> temper_client_domain::Domain {
    chat_offering(false)
}
/// `chat_root`'s, with the message act offered beside the close.
pub fn messaging_root() -> temper_client_domain::Domain {
    chat_offering(true)
}
pub fn chat_offering(message: bool) -> temper_client_domain::Domain {
    use temper_client_domain::{Address, Event};
    let (mut root, me_watch, task_watch) = fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
    let mut me = root.pages().me(root.client()).expect("me").clone();
    let member = me.projects.get_mut(EMPTY_PARAMETERS).expect("project");
    member.templates = List::with_capacity(SINGLE_TEMPLATE);
    member
        .templates
        .push(c::Template {
            resources: List::with_capacity(0),
            number: CHAT_TEMPLATE,
            name: c::DocumentText::complete(c::Text::new(Box::from(*b"Chat"), TEXT).expect("name")),
            executor: c::TemplateExecutor::Charter(CHAT_CHARTER),
            contract: c::TaskContractDocument::Report { words: TEXT },
            shallower: SINGLE_TEMPLATE,
            budget: c::TemplateBudget::AsAllowed { cap: None },
            deadline: c::TemplateDeadline::None,
            tracked: None,
            flags: c::TaskFlags {
                ended_by: jig_client::contract::EndedBy::Result,
                conversation: true,
                listed: false,
                retention: c::Retention::Kept,
            },
            label: CHAT_LABEL,
            grace: Some(CLOSE_GRACE),
            llm: None,
        })
        .expect("template");
    let mut task = fixtures::task::document();
    task.chip.task = TASK;
    task.project = fixtures::PROJECT;
    task.acts = List::with_capacity(if message { MESSAGING_OFFERS } else { SINGLE_TEMPLATE });
    if message {
        task.acts
            .push(c::Offered { act: c::ActKind::Message, admission: c::Admission::Allowed, effect: None })
            .expect("message");
    }
    task.acts
        .push(c::Offered {
            act: c::ActKind::Cancel,
            admission: c::Admission::Allowed,
            effect: Some(c::Effect::Cancel(c::Impact {
                tasks: c::ImpactCounts {
                    waiting: 0,
                    running: 1,
                    held: 0,
                    closing: 0,
                    ended: 0,
                    done: 0,
                    failed: 0,
                    cancelled: 0,
                },
                live_runs: 1,
                proposals: 0,
            })),
        })
        .expect("close");
    for (watch, document) in [
        (me_watch, c::Delivery::Me(Box::new(me))),
        (task_watch.expect("chat watch"), c::Delivery::Task(Box::new(task))),
    ] {
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(document) },
                },
            },
        );
    }
    root
}
pub fn proposal_root() -> temper_client_domain::Domain {
    proposal_root_with_activity(None)
}
pub fn proposal_root_with_activity(activity: Option<c::ActivityPage>) -> temper_client_domain::Domain {
    proposal_root_with_document(activity, None)
}
pub fn proposal_root_with_document(
    activity: Option<c::ActivityPage>,
    document: Option<c::ReadValue>,
) -> temper_client_domain::Domain {
    proposal_in_layout(activity, document, false, false)
}
#[expect(
    clippy::too_many_lines,
    reason = "one fixture delivers the task, delegate and proposal through their requests"
)]
pub fn proposal_in_layout(
    activity: Option<c::ActivityPage>,
    document: Option<c::ReadValue>,
    asking: bool,
    delegate: bool,
) -> temper_client_domain::Domain {
    use temper_client_domain::Event;
    let mut limits = fixtures::domain_limits();
    limits.client.domain.rows = 4;
    limits.client.domain.turn_text = 512;
    let (mut root, me_watch, watch, _, _) = fixtures::root_at_with_activity(
        temper_client_domain::Address::Chat { task: TASK },
        &limits,
        b"Shop",
        None,
        activity,
        None,
    );
    let mut task = fixtures::task::document();
    task.chip.task = TASK;
    if asking {
        let source = messaging_root();
        let me = fixtures::asking_me(source.pages().me(source.client()).expect("me").clone());
        fixtures::deliver_me(&mut root, me_watch, me);
        task.chip.executor = c::ChipExecutor::Agent { charter: 37 };
        task.project = fixtures::PROJECT;
    }
    if delegate {
        task.project = fixtures::PROJECT;
        task.delegates = List::with_capacity(1);
        task.delegates.push(c::LiveDelegate { chip: delegated_task().chip, takeover_budget: 0 }).unwrap();
    }
    task.proposals = List::with_capacity(SINGLE_TEMPLATE);
    task.proposals.push(PROPOSAL).expect("one proposal");
    let mut out = skein_lib::Queue::with_capacity(PROPOSAL_OUTPUTS);
    temper_client_domain::step(
        &mut root,
        &skein_lib::Env { now: PROPOSAL_NOW, wall: fixtures::NOW, limits },
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    let mut acts = List::with_capacity(PROPOSAL_OFFERS);
    for act in [c::ActKind::Accept, c::ActKind::Reject] {
        acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).expect("offer");
    }
    let page = match document {
        Some(page) => page,
        None => c::ReadValue::Proposal(Box::new(c::Proposal {
            key: PROPOSAL,
            project: fixtures::PROJECT,
            projects: List::from_boxed(Box::new([fixtures::PROJECT])),
            reason: c::DocumentText::complete(c::Text::new(Box::from(*b"Make a goal"), TEXT).expect("reason")),
            action: c::ProposalAction::Batch(List::with_capacity(EMPTY_BATCH)),
            as_holder: true,
            waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(PARTY), since: Wall::EPOCH },
            funding: List::with_capacity(EMPTY_BATCH),
            acts,
        })),
    };
    let mut answered = false;
    let mut page = Some(page);
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open {
                            watch,
                            subject: jig_client::Watchable::Jig(c::Subject::Task { task: OTHER_DELEGATE }),
                        },
                },
        } = &request
        {
            fixtures::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Watched {
                            watch: *watch,
                            event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(delegated_task()))),
                        },
                    },
                },
            );
        }
        if let temper_client_domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Read {
                            read,
                            what: jig_client::Readable::Jig(c::Read::Proposal { proposal: PROPOSAL }),
                        },
                },
        } = request
        {
            fixtures::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(
                                page.take().expect("one proposal read"),
                            )),
                        },
                    },
                },
            );
            answered = true;
        }
    }
    assert!(answered, "proposal read emitted");
    root
}
pub fn delegated_task() -> c::Task {
    let mut task = fixtures::task::document();
    task.chip.task = OTHER_DELEGATE;
    task.chip.title = c::DocumentText::complete(fixtures::first_run::text(b"Delegated work"));
    task.project = fixtures::PROJECT;
    task.requester = c::Requester::Task(TASK);
    task
}
pub fn rejecting_chat() -> temper_client_domain::Domain {
    use temper_client_domain::{Action, Event, pages::Act};
    let mut root = proposal_root();
    let object = c::ObjectKey::Proposal(PROPOSAL);
    for act in [
        Act::CardWords { object, words: Box::from(*b"Too costly") },
        Act::Decide { object, act: c::ActKind::Reject, revision: None },
    ] {
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(act) });
    }
    root
}
