use super as f;
use jig_client::contract as c;
use temper_client_domain::{
    self as d, Action, Address, Event,
    pages::{Act, terminal},
};

#[must_use]
pub fn state_root(state: u8) -> d::Domain {
    let mut task = super::chat_helpers::asking_task();
    task.chip.phase = c::Phase::Active;
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
    task.waits_on = c::WaitsOn::News;
    task.chip.details.details.live_delegates = 0;
    task.delegates.clear();
    task.run_wait = None;
    task.requester = c::Requester::Person(f::chat::PARTY);
    task.wake.words = c::WakeRule::Immediate;
    match state {
        0 => {}
        1 => {
            task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
            task.waits_on = c::WaitsOn::Nothing;
        }
        2 => {
            task.waits_on = c::WaitsOn::Delegates;
            task.chip.details.details.live_delegates = 1;
        }
        3 => super::chat_helpers::pause_chip(&mut task.chip),
        4 => {
            task.acts.clear();
            task.chip.phase = c::Phase::Ended;
            task.chip.details.phase = c::ChipPhase::Ended;
        }
        5 => {
            task.run_wait = Some(c::TaskRunWait::Account { account: None, source: None, why: c::TaskWaitWhy::Empty });
        }
        _ => unreachable!(),
    }
    task.chip.phase = task.chip.details.phase.compact();
    super::chat_helpers::asking_task_root(task, true, 1)
}

#[must_use]
pub fn command_root(state: u8) -> d::Domain {
    command_root_with_watch(state).0
}

#[must_use]
pub fn command_root_with_watch(state: u8) -> (d::Domain, Option<skein_lib::Token>) {
    let source = state_root(state);
    let mut me = source.pages().me(source.client()).unwrap().clone();
    let model = me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap();
    model.efforts = skein_lib::List::from_boxed(Box::new([
        c::Text::new(b"high".as_slice().into(), 32).unwrap(),
        c::Text::new(b"low".as_slice().into(), 32).unwrap(),
    ]));
    model.effort = Some(model.efforts.get(0).unwrap().clone());
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.llm = Some(c::LlmChoice {
        account: Some(1),
        main: c::ModelEffort { model: model.name.clone(), effort: model.effort.clone() },
        subagents: skein_lib::List::with_capacity(0),
        later: None,
    });
    if state != 4 {
        let mut acts = skein_lib::List::with_capacity(4);
        for act in task.acts.as_slice() {
            acts.push(act.clone()).unwrap();
        }
        acts.push(c::Offered { act: c::ActKind::Amend, admission: c::Admission::Allowed, effect: None }).unwrap();
        task.acts = acts;
    }
    let (mut root, me_watch, task_watch) = f::root_at(Address::Chat { task: f::chat::TASK }, &f::domain_limits());
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    let mut chats_watch = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open {
                            watch,
                            subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, .. }),
                        },
                },
        } = request
            && query.requester.is_some()
            && query.tracked.is_none()
        {
            chats_watch = Some(watch);
        }
    }
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: task_watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    (root, chats_watch)
}

#[must_use]
pub fn cards_limits() -> d::Limits {
    let mut limits = f::domain_limits();
    limits.client.domain.text = 512;
    limits.client.domain.long_text = 4096;
    limits.client.domain.turn_text = 512;
    limits.client.domain.rows = 4;
    limits
}

pub fn cards_step(root: &mut d::Domain, event: Event, out: &mut skein_lib::Queue<d::Request>) {
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: cards_limits() }, event, out);
}

pub fn cards_up(root: &mut d::Domain, up: jig_client::Up, out: &mut skein_lib::Queue<d::Request>) {
    cards_step(root, Event::Client { event: jig_client::Event::Up { up } }, out);
}

pub const QUESTION_OBJECT: c::ObjectKey = c::ObjectKey::Question { task: f::chat::DELEGATE, message: 1 };

#[expect(
    clippy::too_many_lines,
    reason = "one fixture answers the chat, subtree inbox and question through their requests"
)]
#[must_use]
pub fn terminal_question_root(choices: bool) -> d::Domain {
    use skein_lib::{List, Queue, Wall};
    let source = f::chat::messaging_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
    task.chip.phase = c::Phase::Active;
    task.run_wait = None;
    task.waits_on = c::WaitsOn::Party {
        whom: c::Whom::Person(f::chat::PARTY),
        entries: 1,
        task: f::chat::DELEGATE,
        kind: c::WaitedFor::Question,
    };
    let (mut root, me, watch, needs_watch) =
        f::root_at_with_needs(Address::Chat { task: f::chat::TASK }, &cards_limits());
    f::deliver_me(&mut root, me, source.pages().me(source.client()).unwrap().clone());
    let mut out = Queue::uncapped();
    cards_up(
        &mut root,
        jig_client::Up::Watched {
            watch: watch.unwrap(),
            event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))),
        },
        &mut out,
    );
    cards_step(&mut root, Event::Act { action: Action::Pages(Act::Terminal(terminal::Act::Enter)) }, &mut out);
    let entries = List::from_boxed(Box::new([temper_fixtures::client_documents::entry(
        QUESTION_OBJECT,
        f::PROJECT,
        1,
        Wall::EPOCH,
        None,
    )]));
    cards_up(
        &mut root,
        jig_client::Up::Watched {
            watch: needs_watch,
            event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(temper_fixtures::client_documents::inbox(
                entries.clone(),
                1,
                c::Count { value: 0, limited: false },
                0,
                false,
            )))),
        },
        &mut out,
    );
    let words: &[u8] = if choices {
        b"Which parser?\n\nChoice: Keep it\nChoice: Replace it (recommended)"
    } else {
        b"What should the parser do?"
    };
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request {
            match down {
                jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) } => {
                    cards_up(
                        &mut root,
                        jig_client::Up::Watched {
                            watch,
                            event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(
                                temper_fixtures::client_documents::inbox(
                                    entries.clone(),
                                    1,
                                    c::Count { value: 0, limited: false },
                                    0,
                                    false,
                                ),
                            ))),
                        },
                        &mut out,
                    );
                }
                jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { .. }) } => {
                    cards_up(
                        &mut root,
                        jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(
                                c::InboxPage {
                                    selection: None,
                                    before: None,
                                    rows: entries.clone(),
                                    after: None,
                                    cut: false,
                                },
                            )))),
                        },
                        &mut out,
                    );
                }
                jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { .. }) } => {
                    let mut chip = task.chip.clone();
                    chip.task = f::chat::DELEGATE;
                    cards_up(
                        &mut root,
                        jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Question(Box::new(
                                c::Question {
                                    task: chip,
                                    from: None,
                                    message: 1,
                                    words: c::DocumentText::complete(c::Text::new(words.into(), 4096).unwrap()),
                                    since: Wall::EPOCH,
                                    acts: List::from_boxed(Box::new([c::Offered {
                                        act: c::ActKind::Answer,
                                        admission: c::Admission::Allowed,
                                        effect: None,
                                    }])),
                                    ending: None,
                                },
                            )))),
                        },
                        &mut out,
                    );
                }
                jig_client::Down::Save { .. }
                | jig_client::Down::Erase { .. }
                | jig_client::Down::Send { .. }
                | jig_client::Down::Open { .. }
                | jig_client::Down::Continue { .. }
                | jig_client::Down::Close { .. }
                | jig_client::Down::Read { .. } => {}
            }
        }
    }
    assert_eq!(root.pages().inbox().unwrap().cards.len(), 1);
    root
}
