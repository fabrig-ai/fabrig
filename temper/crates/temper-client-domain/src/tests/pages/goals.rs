use super::{chats::chat_line, fixtures};
use crate::{
    Address, GoalsFilter,
    pages::{
        self, Act, Event, Pages, Request,
        goals::{self, Goals, State},
    },
};
use jig_client::{Down, Readable, Readout, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token, Wall};

const FIRST: u64 = 31;
const GOAL: u64 = fixtures::chat::TASK;
const OTHER: u64 = 89;
const LATER: Wall = Wall::from_nanos(10);
// Two rows fill the fixture's live window and each ended page.
const WINDOW: u32 = 2;
const LABEL: u32 = 7;
// The report handle admits one read at a time.
const REPORTS_AT_ONCE: usize = 1;
// Two rows of the first ended page, one of the next, and one live row.
const ENDED_READ: u32 = 3;
const ALL_READ: u32 = 4;
const CURRENT_ENDED: usize = 1;

fn me() -> c::Me {
    let mut me = fixtures::chats::me();
    let template = me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap();
    template.number = jig_client::shapes::GOAL;
    template.tracked = Some(1);
    me
}
fn line(task: u64, activity: Wall) -> c::TaskLine {
    let mut line = chat_line();
    line.chip.task = task;
    line.last_activity = activity;
    line.tracked = Some(1);
    line
}
fn rows(lines: &[c::TaskLine]) -> List<c::TaskLine> {
    List::from_boxed(lines.into())
}
fn page(lines: &[c::TaskLine], more: bool) -> c::ReadValue {
    c::ReadValue::Tasks(Box::new(c::TaskPage {
        live: List::with_capacity(0),
        live_after: None,
        ended: c::EndedGroup::Rows {
            rows: rows(lines),
            after: if more { Some(c::Cursor::new(b"next".as_slice().into(), 32).unwrap()) } else { None },
            cut: false,
        },
    }))
}
struct Harness {
    root: crate::Domain,
    out: Queue<crate::Request>,
    watch: Token,
    me: Token,
}
impl Harness {
    fn new() -> Self {
        let limits = fixtures::limits();
        let mut root = crate::unlinked(&limits, fixtures::SEED);
        let me_watch = fixtures::start(&mut root, List::with_capacity(0));
        let mut out = Queue::uncapped();
        crate::step(
            &mut root,
            &fixtures::env(&limits),
            crate::Event::Client {
                event: jig_client::Event::Up {
                    up: Up::Watched {
                        watch: me_watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me()))),
                    },
                },
            },
            &mut out,
        );
        crate::step(
            &mut root,
            &fixtures::env(&limits),
            crate::Event::Went {
                place: crate::Place::Address(Address::Goals { project: fixtures::PROJECT, filter: GoalsFilter::All }),
            },
            &mut out,
        );
        let mut watch = None;
        let mut held = Queue::uncapped();
        while let Some(request) = out.pop() {
            if let crate::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Tasks { .. }) },
                    },
            } = &request
            {
                watch = Some(*token);
            }
            held.push(request);
        }
        Self { root, out: held, watch: watch.expect("goals watch"), me: me_watch }
    }
    fn event(&mut self, event: crate::Event) {
        crate::step(&mut self.root, &fixtures::env(&fixtures::limits()), event, &mut self.out);
    }
    fn goals(&self) -> &Goals {
        self.root.pages().goals().unwrap()
    }
    fn reads(&mut self) -> Box<[(Token, c::Read)]> {
        let mut reads = List::uncapped();
        while let Some(request) = self.out.pop() {
            if let crate::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
            } = request
            {
                reads.push((read, what)).expect("fixture reads fit");
            }
        }
        reads.into_boxed()
    }
    fn live(&mut self, lines: &[c::TaskLine], more: bool) {
        self.event(crate::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched {
                    watch: self.watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows: rows(lines), more }))),
                },
            },
        });
    }
    fn answer(&mut self, read: Token, value: c::ReadValue) {
        self.result(read, Readout::Jig(c::ReadResult::Value(value)));
    }
    fn result(&mut self, read: Token, result: Readout) {
        self.event(crate::Event::Client { event: jig_client::Event::Up { up: Up::ReadDone { read, result } } });
    }
    fn act(&mut self, act: Act) {
        self.event(crate::Event::Act { action: crate::Action::Pages(act) });
    }
}
fn maybe_ended_read(reads: &[(Token, c::Read)]) -> Option<Token> {
    for (token, read) in reads {
        if let c::Read::Tasks { .. } = read {
            return Some(*token);
        }
    }
    None
}
fn ended_read(reads: &[(Token, c::Read)]) -> Token {
    maybe_ended_read(reads).expect("ended read")
}
fn reports(reads: &[(Token, c::Read)]) -> Box<[(Token, u64)]> {
    let mut reports = List::uncapped();
    for (token, read) in reads {
        if let c::Read::Notes { scope: c::NoteScope::Task { task, .. }, .. } = read {
            reports.push((*token, *task)).expect("fixture reads fit");
        }
    }
    reports.into_boxed()
}
fn needs_read(reads: &[(Token, c::Read)]) -> Token {
    for (token, read) in reads {
        if let c::Read::Inbox { .. } = read {
            return *token;
        }
    }
    panic!("needs read")
}
fn notes(headline: &[u8]) -> c::ReadValue {
    let note = c::Note {
        number: 1,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: fixtures::PROJECT, task: GOAL },
        description: fixtures::chat::document_text(headline),
        body: fixtures::chat::document_text(b""),
        writer: c::NoteAuthor::Task { task: GOAL, attempt: 1 },
        tasks: List::with_capacity(0),
        changed: Some(LATER),
        acts: List::with_capacity(0),
        cut: false,
    };
    c::ReadValue::Notes(Box::new(c::NotePage {
        rows: List::from_boxed(Box::new([note])),
        before: None,
        after: None,
        cut: false,
    }))
}

#[test]
fn goals_read_its_query_and_release_it() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let mut goals = Goals::new(FIRST);
    let mut out = Queue::uncapped();
    goals.configure(Some(&me()), fixtures::PROJECT, root.client(), &limits, &mut out);
    let query = c::TaskQuery {
        project: Some(fixtures::PROJECT),
        label: Some(LABEL),
        tracked: Some(true),
        order: c::QueryOrder::Priority,
        requester: None,
        root: None,
        parent: None,
        phase: None,
        executor: None,
    };
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Watch {
            handle: goals.live,
            subject: Watchable::Jig(c::Subject::Tasks { query, window: limits.client.domain.rows })
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Read {
            handle: goals.ended,
            read: Readable::Jig(c::Read::Tasks {
                query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query },
                live: None,
                ended: None
            })
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Read {
            handle: goals.needs,
            read: Readable::Jig(c::Read::Inbox {
                group: c::InboxReadGroup::Needs,
                project: Some(fixtures::PROJECT),
                kind: None,
                goal: None,
                after: None
            })
        }))
    );
    goals.configure(Some(&me()), fixtures::PROJECT, root.client(), &limits, &mut out);
    assert!(out.is_empty(), "the same query keeps its reads");
    let mut changed = me();
    changed.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().label = LABEL + 1;
    goals.configure(Some(&changed), fixtures::PROJECT, root.client(), &limits, &mut out);
    for handle in [goals.live, goals.ended, goals.needs, goals.report] {
        assert_eq!(out.pop(), Some(Request::Client(jig_client::Event::Unwatch { handle })));
    }
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Watch {
            handle: goals.live,
            subject: Watchable::Jig(c::Subject::Tasks {
                query: c::TaskQuery { label: Some(LABEL + 1), ..query },
                window: limits.client.domain.rows
            })
        }))
    );
    while out.pop().is_some() {}
    let next = goals.leave(&mut out);
    for handle in [goals.live, goals.ended, goals.needs, goals.report] {
        assert_eq!(out.pop(), Some(Request::Client(jig_client::Event::Unwatch { handle })));
    }
    assert!(Goals::new(next).live.number > goals.report.number);
    assert!(goals::query(&fixtures::chats::me(), fixtures::PROJECT).is_none());
    assert!(goals::query(&me(), OTHER.try_into().unwrap()).is_none());
    assert!(out.is_empty());
}

#[test]
fn each_state_of_the_table_reads() {
    let base = fixtures::chat::task().chip;
    let mut paused = base.clone();
    let c::ReadValue::Escalation(escalation) = fixtures::acts::held(c::ActKind::Release) else { panic!("hold") };
    let mut hold = *escalation.task.hold.unwrap();
    hold.placed = Some(c::PlacedHold {
        number: 1,
        owner: c::HoldOwner::Party { person: fixtures::PARTY },
        mode: c::HoldMode::Stop,
        scope: c::HoldScope::Subtree { task: GOAL },
        reason: Box::default(),
        placed_at: Wall::EPOCH,
        revision: c::Revision { value: 1 },
        lifted: false,
        may_lift: true,
    });
    paused.details.phase = c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 };
    paused.details.details.hold = Some(hold);
    let mut funds = paused.clone();
    funds.details.phase = c::ChipPhase::Held {
        reason: c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Answer, value: 1 }),
        revision: 1,
    };
    let mut cases = List::uncapped();
    for case in [
        (base.clone(), true, false, 0, State::Needs { funds: false }),
        (funds, true, true, 1, State::Needs { funds: true }),
        (paused.clone(), true, false, 0, State::Needs { funds: false }),
        (paused.clone(), false, false, 0, State::Paused),
        (base.clone(), false, false, 0, State::Planning),
        (base.clone(), false, true, 0, State::Running),
        (base.clone(), false, false, 1, State::Running),
    ] {
        cases.push(case).expect("state cases fit");
    }
    for (ending, expected) in [
        (Some(c::ChipEnding::Done(c::ResultValue::Report(fixtures::chat::document_text(b"done")))), State::Landed),
        (Some(c::ChipEnding::Failed(fixtures::chat::document_text(b"failed"))), State::Failed),
        (
            Some(c::ChipEnding::Cancelled {
                reason: fixtures::chat::document_text(b"stop"),
                result: None,
                by: c::Requester::Person(fixtures::PARTY),
                at: Wall::EPOCH,
            }),
            State::Stopped,
        ),
        (None, State::Failed),
    ] {
        let mut chip = paused.clone();
        chip.details.phase = c::ChipPhase::Ended;
        chip.details.details.ending = ending;
        cases.push((chip.clone(), false, false, 0, expected)).expect("state cases fit");
        cases.push((chip, true, true, 1, State::Needs { funds: false })).expect("state cases fit");
    }
    for (mode, scope, lifted) in [
        (c::HoldMode::Drain, c::HoldScope::Subtree { task: GOAL }, false),
        (c::HoldMode::Stop, c::HoldScope::Project, false),
        (c::HoldMode::Stop, c::HoldScope::Subtree { task: OTHER }, false),
        (c::HoldMode::Stop, c::HoldScope::Subtree { task: GOAL }, true),
    ] {
        let mut chip = paused.clone();
        let placed = chip.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap();
        placed.mode = mode;
        placed.scope = scope;
        placed.lifted = lifted;
        cases.push((chip, false, true, 1, State::Running)).expect("state cases fit");
    }
    for (chip, needs, report, delegates, expected) in &cases {
        assert_eq!(goals::state(chip, *needs, *report, *delegates), *expected);
        let filters = match expected {
            State::Needs { .. } => [true, true, false, false],
            State::Planning | State::Paused | State::Running => [true, false, true, false],
            State::Landed | State::Stopped | State::Failed => [true, false, false, true],
        };
        for (filter, selected) in
            [GoalsFilter::All, GoalsFilter::Needs, GoalsFilter::Live, GoalsFilter::Ended].into_iter().zip(filters)
        {
            assert_eq!(goals::shown(filter, *expected), selected);
        }
    }
}

#[test]
fn reports_are_read_one_at_a_time_and_again_as_activity_moves() {
    let mut h = Harness::new();
    h.reads();
    h.live(&[line(GOAL, Wall::EPOCH), line(OTHER, Wall::EPOCH)], false);
    let reads = reports(&h.reads());
    assert_eq!(reads.len(), REPORTS_AT_ONCE);
    assert_eq!(reads[0].1, GOAL);
    h.live(&[line(GOAL, LATER), line(OTHER, Wall::EPOCH)], false);
    assert!(reports(&h.reads()).is_empty(), "one report in flight");
    h.answer(reads[0].0, notes(b"report: Old."));
    let reads = reports(&h.reads());
    assert_eq!(reads.len(), REPORTS_AT_ONCE);
    assert_eq!(reads[0].1, GOAL, "moved activity is next in row order");
    h.answer(reads[0].0, notes(b"report: New."));
    let reads = reports(&h.reads());
    assert_eq!(reads.len(), REPORTS_AT_ONCE);
    assert_eq!(reads[0].1, OTHER);
    h.answer(reads[0].0, notes(b"report: Other."));
    assert!(reports(&h.reads()).is_empty());
    assert_eq!(h.goals().report(GOAL).unwrap().headline.as_ref(), b"New.");
    h.live(&[line(GOAL, LATER), line(OTHER, Wall::EPOCH)], false);
    assert!(reports(&h.reads()).is_empty(), "same activity keeps reports");
}

#[test]
fn no_report_is_no_error() {
    for value in [
        Readout::Limit,
        Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)),
        Readout::Jig(c::ReadResult::Value(notes(b"words report: not an opening"))),
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
            rows: List::with_capacity(0),
            before: None,
            after: None,
            cut: false,
        })))),
    ] {
        let mut h = Harness::new();
        h.reads();
        h.live(&[line(GOAL, Wall::EPOCH)], false);
        let reads = reports(&h.reads());
        assert_eq!(reads.len(), REPORTS_AT_ONCE);
        h.result(reads[0].0, value);
        assert!(h.goals().report(GOAL).is_none());
        assert!(reports(&h.reads()).is_empty());
        h.live(&[line(GOAL, Wall::EPOCH)], false);
        assert!(reports(&h.reads()).is_empty(), "an absent report is remembered at this activity");
        h.live(&[line(GOAL, LATER)], false);
        assert_eq!(reports(&h.reads()).len(), REPORTS_AT_ONCE, "moved activity reads an absent report again");
    }
}

#[test]
fn needs_come_from_the_selection_and_read_again_as_needs_total_moves() {
    let mut h = Harness::new();
    let read = needs_read(&h.reads());
    let selection = List::from_boxed(Box::new([c::InboxSelection {
        cursor: c::InboxCursor::Number { number: 1 },
        goals: List::from_boxed(Box::new([GOAL, OTHER])),
    }]));
    h.answer(
        read,
        c::ReadValue::Inbox(Box::new(c::InboxPage {
            selection: Some(selection),
            rows: List::with_capacity(0),
            before: None,
            after: None,
            cut: false,
        })),
    );
    assert!(h.goals().needs_person(GOAL));
    assert!(h.goals().needs_person(OTHER));
    let mut changed = me();
    changed.needs_total = 1;
    h.event(crate::Event::Client {
        event: jig_client::Event::Up {
            up: Up::Watched {
                watch: h.me,
                event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(changed.clone()))),
            },
        },
    });
    let read = needs_read(&h.reads());
    h.answer(
        read,
        c::ReadValue::Inbox(Box::new(c::InboxPage {
            selection: None,
            rows: List::with_capacity(0),
            before: None,
            after: None,
            cut: false,
        })),
    );
    assert!(!h.goals().needs_person(GOAL));
    h.event(crate::Event::Client {
        event: jig_client::Event::Up {
            up: Up::Watched { watch: h.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(changed))) },
        },
    });
    assert!(h.reads().is_empty(), "unchanged total keeps the read");
    h.live(&[], false);
    needs_read(&h.reads());
}

#[test]
fn more_goals_reads_the_next_page_and_keeps_the_count() {
    let mut h = Harness::new();
    let first = ended_read(&h.reads());
    h.answer(first, page(&[line(GOAL, Wall::EPOCH), line(OTHER, Wall::EPOCH)], true));
    h.reads();
    assert_eq!(h.goals().count(h.root.client()), WINDOW);
    assert!(h.goals().more());
    assert!(h.goals().unread(h.root.client()));
    h.live(&[], false);
    let refresh = h.reads();
    let c::Read::Tasks { ended, .. } = &refresh[0].1 else { panic!("first page refresh") };
    assert_eq!(*ended, None, "live changes refresh the first page");
    h.answer(ended_read(&refresh), page(&[line(GOAL, Wall::EPOCH), line(OTHER, Wall::EPOCH)], true));
    h.reads();
    assert_eq!(h.goals().count(h.root.client()), WINDOW, "refreshing the first page counts its rows once");
    h.act(Act::MoreGoals);
    let reads = h.reads();
    let c::Read::Tasks { ended, .. } = &reads[0].1 else { panic!("older page") };
    assert!(ended.is_some());
    h.answer(ended_read(&reads), page(&[line(OTHER, LATER)], false));
    h.reads();
    assert_eq!(h.goals().ended().len(), CURRENT_ENDED);
    assert_eq!(h.goals().count(h.root.client()), ENDED_READ, "two first rows and one next row");
    assert!(!h.goals().more());
    h.live(&[], false);
    let reads = h.reads();
    assert_eq!(maybe_ended_read(&reads), None, "older page stays put");
    h.live(&[line(GOAL, LATER)], true);
    h.reads();
    assert_eq!(h.goals().count(h.root.client()), ALL_READ, "one live and three ended read");
    assert!(h.goals().unread(h.root.client()), "live more is reachable");
}

#[test]
fn a_filter_is_a_move_that_keeps_the_reads() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let mut pages = Pages::new();
    let mut out = Queue::uncapped();
    let address = Address::Goals { project: fixtures::PROJECT, filter: GoalsFilter::All };
    pages::step(&mut pages, &fixtures::env(&limits), root.client(), Event::Went { address }, &mut out);
    let handle = pages.goals().unwrap().ended;
    pages::step(
        &mut pages,
        &fixtures::env(&limits),
        root.client(),
        Event::Told {
            told: Told::Page {
                handle,
                page: Readout::Jig(c::ReadResult::Value(page(&[line(GOAL, Wall::EPOCH)], true))),
            },
        },
        &mut out,
    );
    while out.pop().is_some() {}
    for filter in [GoalsFilter::Needs, GoalsFilter::Live, GoalsFilter::Ended, GoalsFilter::All] {
        let address = Address::Goals { project: fixtures::PROJECT, filter };
        pages::step(
            &mut pages,
            &fixtures::env(&limits),
            root.client(),
            Event::Act { act: Act::GoalsFilter { filter } },
            &mut out,
        );
        assert_eq!(out.pop(), Some(Request::Show { address: address.clone() }));
        pages::step(&mut pages, &fixtures::env(&limits), root.client(), Event::Went { address }, &mut out);
        let goals = pages.goals().unwrap();
        assert_eq!(goals.ended, handle);
        assert_eq!(goals.filter, filter);
        assert_eq!(goals.ended().len(), CURRENT_ENDED);
        assert!(goals.more());
        assert!(out.is_empty());
    }
    pages::step(&mut pages, &fixtures::env(&limits), root.client(), Event::Left, &mut out);
    let handles = [handle.number - 1, handle.number, handle.number + 1, handle.number + 2];
    for number in handles {
        assert_eq!(
            out.pop(),
            Some(Request::Client(jig_client::Event::Unwatch {
                handle: jig_client::Handle { asker: handle.asker, number }
            }))
        );
    }
    assert!(pages.goals().is_none());
    pages::step(
        &mut pages,
        &fixtures::env(&limits),
        root.client(),
        Event::Went { address: Address::Goals { project: fixtures::PROJECT, filter: GoalsFilter::All } },
        &mut out,
    );
    assert!(pages.goals().unwrap().live.number > *handles.last().unwrap());
    let old = pages.goals().unwrap();
    let previous = [old.live, old.ended, old.needs, old.report];
    pages::step(
        &mut pages,
        &fixtures::env(&limits),
        root.client(),
        Event::Went { address: Address::Goals { project: OTHER.try_into().unwrap(), filter: GoalsFilter::All } },
        &mut out,
    );
    for handle in previous {
        assert_eq!(out.pop(), Some(Request::Client(jig_client::Event::Unwatch { handle })));
    }
    assert!(pages.goals().unwrap().live.number > previous[3].number, "another project gets new handles");
    assert!(pages.goals().unwrap().ended().is_empty());
}
