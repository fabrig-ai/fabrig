use super::fixtures;

use jig_client::contract as c;

use skein_lib::{List, Queue, Wall};

use skein_ui::Role;

use temper_client_domain::{Address, GoalsFilter, pages::goals::State};

// The mock's row has six landed tasks out of nine (client/pages/README.md).

pub(super) use super::fixtures::goals::{TEXT_BYTES, line, template};

const EMPTY: u32 = 0;

const ONE: u32 = 1;

// The state table has five live rows (two needs) and three ended rows.
const PAGE_ROWS: u32 = 8;

const RUNNING_TASK: u64 = 105;

const CURSOR_BYTES: u32 = 32;

struct Page {
    root: temper_client_domain::Domain,
    out: Queue<temper_client_domain::Request>,
    watch: skein_lib::Token,
    limits: temper_client_domain::Limits,
}

impl Page {
    fn new() -> Self {
        let mut limits = fixtures::domain_limits();
        limits.client.domain.rows = PAGE_ROWS;
        // Work keeps its four lists alongside the frame and per-goal reads.
        limits.client.domain.handles = 64;
        limits.client.domain.reads = 32;
        limits.client.exchanges = 32;
        let (mut root, me, _) = fixtures::root_at(
            Address::Inbox {
                filter: temper_client_domain::address::Filter {
                    project: Some(fixtures::PROJECT),
                    ..temper_client_domain::address::Filter::ALL
                },
            },
            &limits,
        );
        let mut frame = root.pages().me(root.client()).unwrap().clone();
        frame.projects.get_mut(EMPTY).unwrap().templates =
            List::from_boxed(Box::new([template(jig_client::shapes::GOAL, b"goal", true)]));
        fixtures::deliver_me(&mut root, me, frame);
        let mut page = Self { root, out: Queue::uncapped(), watch: me, limits };
        page.event(temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(Address::Goals {
                project: fixtures::PROJECT,
                filter: GoalsFilter::All,
            }),
        });
        for request in &page.out {
            if let temper_client_domain::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Open {
                                watch,
                                subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, .. }),
                            },
                    },
            } = request
                && query.tracked == Some(true)
            {
                page.watch = *watch;
            }
        }
        page
    }
    fn event(&mut self, event: temper_client_domain::Event) {
        temper_client_domain::step(
            &mut self.root,
            &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: self.limits },
            event,
            &mut self.out,
        );
    }
    fn up(&mut self, up: jig_client::Up) {
        self.event(temper_client_domain::Event::Client { event: jig_client::Event::Up { up } });
    }
    fn read(&mut self, tasks: bool) -> skein_lib::Token {
        let mut found = None;
        let mut held = Queue::uncapped();
        while let Some(request) = self.out.pop() {
            if let temper_client_domain::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(what) },
                    },
            } = &request
            {
                if tasks && let c::Read::Tasks { .. } = what {
                    found = Some(*read);
                }
                if !tasks && let c::Read::Inbox { .. } = what {
                    found = Some(*read);
                }
            }
            held.push(request);
        }
        self.out = held;
        found.expect("page read")
    }
    fn answer(&mut self, read: skein_lib::Token, value: c::ReadValue) {
        self.up(jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::Value(value)) });
    }
    fn live(&mut self, lines: &[c::TaskLine], more: bool) {
        self.up(jig_client::Up::Watched {
            watch: self.watch,
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                rows: List::from_boxed(lines.into()),
                more,
            }))),
        });
    }
    fn ended(&mut self, lines: &[c::TaskLine], more: bool) {
        let read = self.read(true);
        self.answer(
            read,
            c::ReadValue::Tasks(Box::new(c::TaskPage {
                live: List::uncapped(),
                live_after: None,
                ended: c::EndedGroup::Rows {
                    rows: List::from_boxed(lines.into()),
                    cut: false,
                    after: if more {
                        Some(c::Cursor::new(b"next".as_slice().into(), CURSOR_BYTES).unwrap())
                    } else {
                        None
                    },
                },
            })),
        );
    }
}

fn page_line(task: u64, state: State) -> c::TaskLine {
    let mut line = line();
    line.chip.task = task;
    match state {
        State::Planning => line.delegates_total = EMPTY,
        State::Needs { funds: false } | State::Running => {}
        State::Needs { funds: true } => {
            line.chip.details.phase = c::ChipPhase::Held {
                reason: c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Answer, value: ONE.into() }),
                revision: ONE.into(),
            }
        }
        State::Paused => {
            line.chip.details.phase = c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: ONE.into() };
            line.chip.details.details.hold = Some(c::ChipHold {
                placed: Some(c::PlacedHold {
                    number: ONE.into(),
                    owner: c::HoldOwner::Party { person: ONE.into() },
                    mode: c::HoldMode::Stop,
                    scope: c::HoldScope::Subtree { task },
                    reason: Box::default(),
                    placed_at: Wall::EPOCH,
                    revision: c::Revision { value: ONE.into() },
                    lifted: false,
                    may_lift: true,
                }),
                words: None,
                circle: None,
                account: None,
                source: None,
                reason: c::HoldCategory::Stopped,
                revision: ONE.into(),
                before: c::PriorPhase::Active(c::RunStage::Idle),
                escalation: None,
                tries: c::FailureTries {
                    transient: EMPTY,
                    permanent: EMPTY,
                    run: EMPTY,
                    agent: EMPTY,
                    lost: EMPTY,
                    invalid: EMPTY,
                    infrastructure: 0,
                },
                hold_wait_since: None,
            });
        }
        State::Landed | State::Closed | State::Failed => {
            line.chip.phase = c::Phase::Ended;
            line.chip.details.phase = c::ChipPhase::Ended;
            let text = || c::DocumentText::complete(c::Text::new(b"Done".as_slice().into(), TEXT_BYTES).unwrap());
            line.chip.details.details.ending = Some(match state {
                State::Landed => c::ChipEnding::Done(c::ResultValue::Report(text())),
                State::Closed => {
                    c::ChipEnding::Cancelled { reason: text(), result: None, by: line.requester, at: Wall::EPOCH }
                }
                State::Failed => c::ChipEnding::Failed(text()),
                State::Needs { .. } | State::Paused | State::Planning | State::Running => unreachable!(),
            });
        }
    }
    line
}

#[test]
fn an_unanswered_goals_page_claims_no_empty() {
    let mut page = Page::new();
    page.live(&[], false);
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        let main = fixtures::main_text(view.tree());
        for name in [b"No goals yet.".as_slice(), b"None."] {
            assert!(skein_lib::bytes::find(&main, name).is_none());
        }
        for node in view.tree().nodes() {
            assert_ne!(node.role, Role::Heading { level: 1 });
        }
    }
}

#[test]
fn a_lost_ended_read_keeps_the_ended_rows() {
    let mut page = Page::new();
    let ended = page_line(RUNNING_TASK, State::Closed);
    page.live(&[], false);
    page.ended(core::slice::from_ref(&ended), false);
    page.live(&[], false);
    let read = page.read(true);
    page.up(jig_client::Up::ReadDone {
        read,
        result: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)),
    });
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        assert!(skein_lib::bytes::find(&fixtures::main_text(view.tree()), ended.chip.title.bytes()).is_some());
    }
}
