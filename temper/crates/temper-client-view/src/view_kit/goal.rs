use super::chats;
use super::{self as fixtures, task as f};
use jig_client::{Down, Event as ClientEvent, Readable, Readout, UnlinkedRequest, Up, Watchable, contract as c};
use skein_lib::{Env, List, Queue, Time, Wall};
use temper_client_domain::{Address, Domain, Event, Request, pages::Act};
use temper_fixtures::client_documents as documents;

pub const TASK: u64 = 9;

pub const DONE: u32 = 6;

pub const TOTAL: u32 = 9;

pub const MINUTE: u64 = 60_000_000_000;

pub const MADE: Wall = Wall::from_nanos(340 * MINUTE);

pub const NOW: Wall = Wall::from_nanos(720 * MINUTE);

pub const DRAFT_WORDS: &[u8] = b"Build the draft.\nDraft: docs/design/draft/away-mode.md\nChat: T4";

pub fn document() -> c::Task {
    let mut task = f::document();
    task.chip.task = TASK;
    task.priority = Some(1);
    task.project = fixtures::PROJECT;
    task.chip.details.details.project = fixtures::PROJECT;
    task.chip.title = text(b"Away mode");
    task.requester = c::Requester::Person(73);
    task.made = MADE;
    task.budget.numbers.spent = 31;
    task.budget.numbers.budget = 40;
    // These supplied fields are independent of the header's spent number.
    task.budget.numbers.spent_below = 13;
    task.budget.numbers.reserved = 5;
    task.budget.left = 2;
    task.spec.specification.words = text(DRAFT_WORDS);
    task.result = None;
    task.chip.details.details.ending = None;
    task
}

pub fn text(value: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(value.into(), 1024).unwrap())
}

pub fn root(task: c::Task, done: u32, total: u32, note: Option<c::Note>, needs: bool) -> Domain {
    root_with_side(task, done, total, note, needs, Side::empty())
}

pub struct Side {
    pub activity: Option<c::ActivityPage>,
    pub specification: Option<Box<[u8]>>,
    pub rows: List<c::TreeRow>,
    pub ended: c::TaskPage,
    pub need: Option<u64>,
    pub cards: List<(c::ObjectKey, c::Read, c::ReadValue)>,
}

impl Side {
    pub fn empty() -> Self {
        Self {
            activity: None,
            specification: None,
            rows: List::uncapped(),
            ended: c::TaskPage {
                live: List::uncapped(),
                live_after: None,
                ended: c::EndedGroup::Rows { rows: List::uncapped(), after: None, cut: false },
            },
            need: None,
            cards: List::uncapped(),
        }
    }
}

#[expect(clippy::too_many_lines, reason = "one fixture settles the goal and its side reads together")]
pub fn root_with_side(task: c::Task, done: u32, total: u32, note: Option<c::Note>, needs: bool, side: Side) -> Domain {
    let mut limits = fixtures::domain_limits();
    if let Some(whole) = &side.specification {
        limits.client.domain.long_text = limits.client.domain.long_text.max(u32::try_from(whole.len()).unwrap());
    }
    let (mut root, _, watch, tree_watch, _, _, inbox_watch) = fixtures::root_at_with_dismissals_and_needs(
        Address::Task { task: TASK },
        &limits,
        b"Shop",
        None,
        side.activity.clone(),
        None,
        Some(&[]),
    );
    let mut out = Queue::uncapped();
    let env = Env { now: Time::ZERO, wall: NOW, limits };
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: ClientEvent::Up {
                up: Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))),
                },
            },
        },
        &mut out,
    );
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: ClientEvent::Up {
                up: Up::Watched {
                    watch: inbox_watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        List::uncapped(),
                        0,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                },
            },
        },
        &mut out,
    );
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: ClientEvent::Up {
                up: Up::Watched {
                    watch: tree_watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Tree(Box::new(c::Tree {
                        root: task.chip.clone(),
                        rows: side.rows.clone(),
                        folded: List::uncapped(),
                        held_below: 0,
                        next: None,
                    }))),
                },
            },
        },
    );
    // Drain only the finite reads and watches opened by this document and its replies.
    while let Some(request) = out.pop() {
        let up = match request {
            Request::Unlinked {
                request: UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
            } => match read_value(what, &task, done, total, note.as_ref(), needs, &side) {
                Some(value) => Some(Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) }),
                None => None,
            },
            Request::Unlinked {
                request:
                    UnlinkedRequest::Down { down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) } },
            } => Some(Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                    List::uncapped(),
                    0,
                    c::Count { value: 0, limited: false },
                    0,
                    false,
                )))),
            }),
            Request::Unlinked {
                request:
                    UnlinkedRequest::Down { down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Tree { .. }) } },
            } => Some(Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Tree(Box::new(c::Tree {
                    root: task.chip.clone(),
                    rows: side.rows.clone(),
                    folded: List::uncapped(),
                    held_below: 0,
                    next: None,
                }))),
            }),
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
            | Request::Fact { .. } => None,
        };
        if let Some(up) = up {
            temper_client_domain::step(&mut root, &env, Event::Client { event: ClientEvent::Up { up } }, &mut out);
        }
    }
    assert_eq!(root.pages().inbox().unwrap().needs().next().is_some(), needs, "goal inbox: {:?}", root.pages().inbox());
    assert!(root.pages().task().unwrap().goal.as_ref().unwrap().line.is_some(), "own line delivered");
    root
}

pub fn read_value(
    what: c::Read,
    task: &c::Task,
    done: u32,
    total: u32,
    note: Option<&c::Note>,
    needs: bool,
    side: &Side,
) -> Option<c::ReadValue> {
    for (_, wanted, page) in &side.cards {
        if *wanted == what {
            return Some(page.clone());
        }
    }
    match what {
        c::Read::ActivityBeside { .. } => Some(c::ReadValue::Activity(Box::new(c::ActivityPage {
            rows: List::with_capacity(0),
            before: None,
            after: None,
            cut: false,
        }))),
        c::Read::Activity { .. } => match &side.activity {
            Some(page) => Some(c::ReadValue::Activity(Box::new(page.clone()))),
            None => None,
        },
        c::Read::Question { task, message } if needs && side.cards.is_empty() => {
            Some(question(task, message, b"Where?"))
        }
        c::Read::Notes { .. } => {
            let mut rows = List::uncapped();
            if let Some(note) = note {
                rows.push(note.clone()).unwrap();
            }
            Some(c::ReadValue::Notes(Box::new(c::NotePage { rows, before: None, after: None, cut: false })))
        }
        c::Read::TaskText { task, field: c::TaskTextField::Specification, from: 0 } if side.specification.is_some() => {
            let words = side.specification.as_deref().unwrap();
            Some(c::ReadValue::TaskText(Box::new(c::TaskText {
                task,
                field: c::TaskTextField::Specification,
                from: 0,
                total: skein_lib::len_u64(words.len()),
                next: None,
                text: c::Text::new(words.into(), u32::try_from(words.len()).unwrap()).unwrap(),
            })))
        }
        c::Read::Tasks { query, .. } if query.tracked == Some(true) => {
            let mut line = chats::line();
            line.chip = task.chip.clone();
            line.delegates_done = done;
            line.delegates_total = total;
            let mut rows = List::uncapped();
            rows.push(line).unwrap();
            Some(c::ReadValue::Tasks(Box::new(c::TaskPage {
                live: rows,
                live_after: None,
                ended: c::EndedGroup::None,
            })))
        }
        c::Read::Inbox { group: c::InboxReadGroup::Needs, goal: Some(TASK), .. } => {
            let mut rows = List::uncapped();
            for (object, _, _) in &side.cards {
                rows.push(documents::entry(*object, fixtures::PROJECT, 1, MADE, None)).unwrap();
            }
            if needs && side.cards.is_empty() {
                rows.push(documents::entry(
                    c::ObjectKey::Question { task: side.need.unwrap_or(TASK), message: 1 },
                    fixtures::PROJECT,
                    1,
                    MADE,
                    None,
                ))
                .unwrap();
            }
            Some(c::ReadValue::Inbox(Box::new(c::InboxPage {
                rows,
                before: None,
                after: None,
                selection: None,
                cut: false,
            })))
        }
        c::Read::Inbox { .. } => Some(c::ReadValue::Inbox(Box::new(c::InboxPage {
            rows: List::uncapped(),
            before: None,
            after: None,
            selection: None,
            cut: false,
        }))),
        c::Read::Tasks { .. } => Some(c::ReadValue::Tasks(Box::new(side.ended.clone()))),
        c::Read::Settings
        | c::Read::Setting(_)
        | c::Read::Revision(_)
        | c::Read::Revisions(_)
        | c::Read::Me
        | c::Read::Turn { .. }
        | c::Read::TaskText { .. }
        | c::Read::Task { .. }
        | c::Read::Proposal { .. }
        | c::Read::Escalation { .. }
        | c::Read::Question { .. }
        | c::Read::PersonTask { .. }
        | c::Read::Result { .. }
        | c::Read::Ended { .. }
        | c::Read::History { .. }
        | c::Read::Spend { .. }
        | c::Read::Project { .. }
        | c::Read::Deployment => None,
    }
}

pub fn offers(kinds: &[c::ActKind]) -> List<c::Offered> {
    let mut offers = List::uncapped();
    for act in kinds {
        offers.push(c::Offered { act: *act, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    offers
}

pub fn question(task: u64, message: u64, words: &[u8]) -> c::ReadValue {
    let mut chip = document().chip;
    chip.task = task;
    c::ReadValue::Question(Box::new(c::Question {
        task: chip,
        from: None,
        message,
        words: text(words),
        since: MADE,
        acts: offers(&[c::ActKind::Answer]),
        ending: None,
    }))
}

pub fn paused_document() -> c::Task {
    let mut task = document();
    f::phase(&mut task, c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 });
    task.chip.details.details.hold = Some(c::ChipHold {
        placed: Some(c::PlacedHold {
            number: 1,
            owner: c::HoldOwner::Party { person: 73 },
            mode: c::HoldMode::Stop,
            scope: c::HoldScope::Subtree { task: TASK },
            reason: Box::default(),
            placed_at: MADE,
            revision: c::Revision { value: 1 },
            lifted: false,
            may_lift: true,
        }),
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Stopped,
        revision: 1,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    });
    task
}

pub fn press(root: &mut Domain, act: Act) -> Queue<Request> {
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &Env { now: Time::ZERO, wall: NOW, limits: fixtures::domain_limits() },
        Event::Act { action: temper_client_domain::Action::Pages(act) },
        &mut out,
    );
    out
}

pub fn with_goal_answer(act: Act, answer: c::Answer) -> Domain {
    let mut task = if act == Act::Resume { paused_document() } else { document() };
    task.acts = offers(&[c::ActKind::Stop, c::ActKind::Release]);
    let mut root = root(task, DONE, TOTAL, None, false);
    let attempt = sent_attempt(press(&mut root, act));
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: ClientEvent::Up {
                up: Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(answer)),
                    why: None,
                },
            },
        },
    );
    root
}

pub fn sent_attempt(mut out: Queue<Request>) -> skein_lib::Token {
    let mut attempt = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Send { attempt: sent, .. } } } = request
        {
            attempt = Some(sent);
        }
    }
    attempt.unwrap()
}
