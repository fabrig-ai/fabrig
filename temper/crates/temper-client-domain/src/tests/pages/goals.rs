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
        // The cross-page goal fact owns this first ended read; page tests answer it once here.
        let mut fact_read = None;
        while let Some(request) = out.pop() {
            if let crate::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Read { read, what: Readable::Jig(c::Read::Tasks { .. }) },
                    },
            } = request
            {
                assert!(fact_read.replace(read).is_none(), "one goal fact read");
            }
        }
        crate::step(
            &mut root,
            &fixtures::env(&limits),
            crate::Event::Client {
                event: jig_client::Event::Up {
                    up: Up::ReadDone {
                        read: fact_read.expect("goal fact read"),
                        result: Readout::Jig(c::ReadResult::Value(page(&[], false))),
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
            State::Closed,
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
            State::Landed | State::Closed | State::Failed => [true, false, false, true],
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
        let unwatch =
            Request::Client(jig_client::Event::Unwatch { handle: jig_client::Handle { asker: handle.asker, number } });
        let mut released = false;
        for request in &out {
            released |= request == &unwatch;
        }
        assert!(released, "goal read released");
    }
    assert!(pages.goals().is_none());
    out.clear();
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
    out.clear();
    pages::step(
        &mut pages,
        &fixtures::env(&limits),
        root.client(),
        Event::Went { address: Address::Goals { project: OTHER.try_into().unwrap(), filter: GoalsFilter::All } },
        &mut out,
    );
    for handle in previous {
        let unwatch = Request::Client(jig_client::Event::Unwatch { handle });
        let mut released = false;
        for request in &out {
            released |= request == &unwatch;
        }
        assert!(released, "goal read released");
    }
    assert!(pages.goals().unwrap().live.number > previous[3].number, "another project gets new handles");
    assert_eq!((pages.goals().unwrap().ended()).len(), 0);
}

fn fact_limits() -> crate::Limits {
    let mut limits = fixtures::limits();
    // A project switch holds the old Home and Goals watches until their closes.
    limits.client.domain.watches = 20;
    // Home also holds the Goals page's four handles beside its inbox cards.
    limits.client.domain.handles = 20;
    limits
}

struct FactHarness {
    root: crate::Domain,
    out: Queue<crate::Request>,
    me: Token,
}
impl FactHarness {
    fn new(identity: c::Me) -> Self {
        let mut root = crate::unlinked(&fact_limits(), fixtures::SEED);
        let me = fixtures::start(&mut root, List::with_capacity(0));
        let mut h = Self { root, out: Queue::uncapped(), me };
        h.go(Address::Home { project: fixtures::PROJECT });
        h.deliver_me(identity);
        h
    }
    fn event(&mut self, event: crate::Event) {
        crate::step(&mut self.root, &fixtures::env(&fact_limits()), event, &mut self.out);
    }
    fn up(&mut self, up: Up) {
        self.event(crate::Event::Client { event: jig_client::Event::Up { up } });
    }
    fn deliver_me(&mut self, identity: c::Me) {
        self.up(Up::Watched { watch: self.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(identity))) });
    }
    fn go(&mut self, address: Address) {
        self.event(crate::Event::Went { place: crate::Place::Address(address) });
    }
    fn live(&mut self, watch: Token, lines: &[c::TaskLine]) {
        self.up(Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows: rows(lines), more: false }))),
        });
    }
    fn answer(&mut self, read: Token, lines: &[c::TaskLine]) {
        self.up(Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(page(lines, false))) });
    }
    fn take(&mut self) -> Box<[Down]> {
        let mut down = List::uncapped();
        while let Some(request) = self.out.pop() {
            if let crate::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: request } } = request {
                down.push(request).expect("fixture requests fit");
            }
        }
        down.into_boxed()
    }
}

fn fact_requests(down: &[Down], project: u32) -> (Token, Token) {
    let query = c::TaskQuery { project: Some(project), ..goals::query(&me(), fixtures::PROJECT).unwrap() };
    let mut watch = None;
    let mut read = None;
    for request in down {
        match request {
            Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Tasks { query: got, window: 1 }) } => {
                assert_eq!(*got, query);
                assert!(watch.replace(*token).is_none(), "one fact watch");
            }
            Down::Read { read: token, what: Readable::Jig(c::Read::Tasks { query: got, live, ended }) } => {
                assert_eq!(
                    *got,
                    c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query }
                );
                assert_eq!(*live, None);
                assert_eq!(*ended, None);
                // The fact is configured before Home's Goals page, whose query is the same.
                if read.is_none() {
                    read = Some(*token);
                }
            }
            Down::Save { .. }
            | Down::Erase { .. }
            | Down::Send { .. }
            | Down::Open { .. }
            | Down::Continue { .. }
            | Down::Close { .. }
            | Down::Read { .. } => {}
        }
    }
    assert!(watch.is_some(), "goal fact watch: {down:?}");
    assert!(read.is_some(), "goal fact read: {down:?}");
    (watch.unwrap(), read.unwrap())
}

fn reserved_handle(offset: u64) -> jig_client::Handle {
    jig_client::Handle {
        asker: crate::owner::Owner::Pages.asker(),
        number: u64::MAX.checked_sub(offset).expect("reserved handle"),
    }
}

#[test]
fn goals_opens_on_home_once_the_project_has_one() {
    for ended in [false, true] {
        let mut h = FactHarness::new(me());
        let (watch, read) = fact_requests(&h.take(), fixtures::PROJECT);
        assert!(h.root.pages().goals().is_none(), "the fact is not delivered yet");
        if ended {
            h.answer(read, &[line(GOAL, Wall::EPOCH)]);
        } else {
            h.live(watch, &[line(GOAL, Wall::EPOCH)]);
        }
        let goals = h.root.pages().goals().expect("Home opens Goals");
        assert_eq!(goals.filter, GoalsFilter::All);
        let handles = [goals.live, goals.ended, goals.needs, goals.report];
        let inbox = h.root.pages().inbox().unwrap();
        assert!(handles[0].number > inbox.entries.number);
        assert!(inbox.next > handles[3].number, "later inbox cards cannot reuse goal handles");
        let requests = h.take();
        let mut watching = false;
        let mut reads = 0_usize;
        for request in &requests {
            if let Down::Open { subject: Watchable::Jig(c::Subject::Tasks { window: WINDOW, .. }), .. } = request {
                watching = true;
            }
            if let Down::Read { .. } = request {
                reads = reads.checked_add(1).unwrap();
            }
        }
        assert!(watching, "Home opens the Goals live window");
        assert!(h.root.client().document(handles[0]).is_some(), "Goals watch admitted");
        assert_eq!(reads, 2, "Goals reads its ended page and needs");
        h.deliver_me(me());
        assert_eq!(h.root.pages().goals().unwrap().live, handles[0], "one Goals per Home visit");
        assert!(h.take().is_empty(), "unchanged facts keep the Goals reads");
    }
}

#[test]
fn home_opens_no_goals_with_none() {
    let mut h = FactHarness::new(me());
    let (watch, read) = fact_requests(&h.take(), fixtures::PROJECT);
    h.answer(read, &[]);
    h.live(watch, &[]);
    assert!(!h.root.has_goals(fixtures::PROJECT));
    assert!(h.root.pages().goals().is_none());
    assert!(h.take().is_empty(), "empty goal facts open no page reads");
}

#[test]
fn leaving_home_leaves_its_goals() {
    for left in [false, true] {
        let mut h = FactHarness::new(me());
        let (watch, _) = fact_requests(&h.take(), fixtures::PROJECT);
        h.live(watch, &[line(GOAL, Wall::EPOCH)]);
        let live = h.root.pages().goals().unwrap().live;
        let mut goal_watch = None;
        for request in h.take() {
            if let Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { window: WINDOW, .. }) } = request {
                goal_watch = Some(watch);
            }
        }
        let goal_watch = goal_watch.expect("Goals watch");
        h.live(goal_watch, &[line(GOAL, Wall::EPOCH)]);
        h.take();
        if left {
            let mut out = Queue::uncapped();
            pages::step(&mut h.root.pages, &fixtures::env(&fact_limits()), &h.root.client, Event::Left, &mut out);
            assert!(h.root.pages().goals().is_none());
            let mut unwatched = false;
            for request in &out {
                unwatched |= *request == Request::Client(jig_client::Event::Unwatch { handle: live });
            }
            assert!(unwatched);
        } else {
            h.go(Address::Settings { project: fixtures::PROJECT });
            assert!(h.root.pages().goals().is_none());
            assert!(h.take().contains(&Down::Close { watch: goal_watch }));
        }
    }
}

#[test]
fn has_goals_stands_across_pages_on_one_watch() {
    let mut h = FactHarness::new(me());
    assert!(!h.root.has_goals(fixtures::PROJECT), "undelivered");
    let (watch, read) = fact_requests(&h.take(), fixtures::PROJECT);
    assert!(h.root.client().document(reserved_handle(1)).is_some(), "fixed session watch handle");
    h.answer(read, &[]);
    h.live(watch, &[line(GOAL, Wall::EPOCH)]);
    assert!(h.root.has_goals(fixtures::PROJECT));
    assert!(!h.root.has_goals(OTHER.try_into().unwrap()), "only the open project");
    h.take();
    for address in [
        Address::Goals { project: fixtures::PROJECT, filter: GoalsFilter::All },
        Address::Chat { task: GOAL },
        Address::Home { project: fixtures::PROJECT },
    ] {
        h.go(address);
        assert!(h.root.has_goals(fixtures::PROJECT), "the fact stays through navigation");
        for request in h.take() {
            assert_ne!(request, Down::Close { watch }, "navigation keeps the fact watch");
            if let Down::Open { subject: Watchable::Jig(c::Subject::Tasks { window, .. }), .. } = request {
                assert_ne!(window, 1, "navigation opens no second fact watch");
            }
        }
        assert!(h.root.client().document(reserved_handle(1)).is_some());
    }
    h.live(watch, &[]);
    assert!(!h.root.has_goals(fixtures::PROJECT), "both empty");
    assert!(h.take().is_empty(), "the fact reads once, independently of the page");
}

#[test]
fn an_ended_goal_counts() {
    let mut h = FactHarness::new(me());
    let (watch, read) = fact_requests(&h.take(), fixtures::PROJECT);
    h.live(watch, &[]);
    assert!(!h.root.has_goals(fixtures::PROJECT));
    h.answer(read, &[line(GOAL, Wall::EPOCH)]);
    assert!(h.root.has_goals(fixtures::PROJECT));
    h.go(Address::Chat { task: GOAL });
    assert!(h.root.has_goals(fixtures::PROJECT));
    // The ended read uses the other fixed session handle.
    pages::step(
        &mut h.root.pages,
        &fixtures::env(&fixtures::limits()),
        &h.root.client,
        Event::Told { told: Told::Page { handle: reserved_handle(2), page: Readout::Limit } },
        &mut Queue::uncapped(),
    );
    assert!(h.root.has_goals(fixtures::PROJECT), "a valueless read keeps the known goal");
}

#[test]
fn no_template_sends_nothing() {
    let mut h = FactHarness::new(fixtures::chats::me());
    assert!(!h.root.has_goals(fixtures::PROJECT));
    for request in h.take() {
        if let Down::Open { subject: Watchable::Jig(c::Subject::Tasks { query, .. }), .. }
        | Down::Read { what: Readable::Jig(c::Read::Tasks { query, .. }), .. } = request
        {
            assert_ne!(query.tracked, Some(true), "no goal fact request without a template");
        }
    }
    assert!(h.root.client().document(reserved_handle(1)).is_none());
    h.deliver_me(me());
    let (watch, read) = fact_requests(&h.take(), fixtures::PROJECT);
    h.live(watch, &[]);
    h.answer(read, &[line(GOAL, Wall::EPOCH)]);
    h.deliver_me(fixtures::chats::me());
    assert!(h.take().contains(&Down::Close { watch }), "removing the template releases the fact");
    assert!(!h.root.has_goals(fixtures::PROJECT));
}

#[test]
fn another_project_starts_again() {
    let mut identity = me();
    let mut other = identity.projects.get(0).unwrap().clone();
    other.project = OTHER.try_into().unwrap();
    identity.projects = List::from_boxed(Box::new([identity.projects.get(0).unwrap().clone(), other]));
    let mut h = FactHarness::new(identity.clone());
    let (watch, read) = fact_requests(&h.take(), fixtures::PROJECT);
    h.live(watch, &[]);
    h.answer(read, &[line(GOAL, Wall::EPOCH)]);
    h.take();
    h.go(Address::Home { project: OTHER.try_into().unwrap() });
    let requests = h.take();
    assert!(requests.contains(&Down::Close { watch }));
    let (next_watch, next_read) = fact_requests(&requests, OTHER.try_into().unwrap());
    assert_ne!(watch, next_watch);
    assert_ne!(read, next_read);
    assert!(!h.root.has_goals(fixtures::PROJECT));
    assert!(!h.root.has_goals(OTHER.try_into().unwrap()), "new project has not delivered");
    h.answer(read, &[line(GOAL, Wall::EPOCH)]);
    h.live(watch, &[line(GOAL, Wall::EPOCH)]);
    assert!(!h.root.has_goals(OTHER.try_into().unwrap()), "old deliveries cannot cross projects");
    h.up(Up::WatchEnded { watch, end: c::WatchEnd::Closed });
    h.live(next_watch, &[]);
    h.answer(next_read, &[line(OTHER, Wall::EPOCH)]);
    assert!(h.root.has_goals(OTHER.try_into().unwrap()));
    identity.projects = List::with_capacity(0);
    h.deliver_me(identity);
    assert_eq!(h.root.pages().project(), None);
    assert!(h.take().contains(&Down::Close { watch: next_watch }));
    h.up(Up::WatchEnded { watch: next_watch, end: c::WatchEnd::Closed });
    assert!(!h.root.has_goals(OTHER.try_into().unwrap()));
    assert!(h.root.client().document(reserved_handle(1)).is_none());
    h.deliver_me(me());
    fact_requests(&h.take(), fixtures::PROJECT);
}

#[test]
fn a_sign_in_asks_the_read_again() {
    for delivered in [false, true] {
        let mut h = FactHarness::new(me());
        let (_, read) = fact_requests(&h.take(), fixtures::PROJECT);
        if delivered {
            h.answer(read, &[line(GOAL, Wall::EPOCH)]);
            assert!(h.root.has_goals(fixtures::PROJECT));
        }
        h.event(crate::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } });
        assert!(!h.root.has_goals(fixtures::PROJECT), "sign-in releases the old fact");
        let mut own = None;
        for request in h.take() {
            if let Down::Open { watch, subject: Watchable::Jig(c::Subject::Me) } = request {
                own = Some(watch);
            }
        }
        h.me = own.expect("sign-in reopens me");
        h.deliver_me(me());
        let (_, next_read) = fact_requests(&h.take(), fixtures::PROJECT);
        assert_ne!(read, next_read, "reads are asked again, never replayed");
        h.answer(next_read, &[line(GOAL, Wall::EPOCH)]);
        assert!(h.root.has_goals(fixtures::PROJECT));
    }
}

#[test]
fn signing_out_and_out_of_date_release_the_fact() {
    for out_of_date in [false, true] {
        let mut h = FactHarness::new(me());
        let (_, read) = fact_requests(&h.take(), fixtures::PROJECT);
        h.answer(read, &[line(GOAL, Wall::EPOCH)]);
        assert!(h.root.has_goals(fixtures::PROJECT));
        if out_of_date {
            h.up(Up::Served { served: false, versions: None });
        } else {
            h.event(crate::Event::SignedOut);
        }
        assert!(!h.root.has_goals(fixtures::PROJECT));
        assert_eq!(h.root.pages().project(), None);
        assert!(h.root.client().document(reserved_handle(1)).is_none());
        h.answer(read, &[line(GOAL, Wall::EPOCH)]);
        assert!(!h.root.has_goals(fixtures::PROJECT));
    }
}

#[test]
fn a_lost_read_keeps_the_page_and_the_count() {
    let mut h = Harness::new();
    let first = ended_read(&h.reads());
    h.answer(first, page(&[line(GOAL, Wall::EPOCH), line(OTHER, Wall::EPOCH)], true));
    h.reads();
    h.act(Act::MoreGoals);
    let next = ended_read(&h.reads());
    let before = rows(h.goals().ended());
    let count = h.goals().count(h.root.client());
    h.result(next, Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)));
    assert_eq!(h.goals().ended(), before.as_slice());
    assert_eq!(h.goals().count(h.root.client()), count, "lost second page counts no retained row twice");
    h.act(Act::MoreGoals);
    assert!(!h.reads().is_empty(), "lost read clears reading_page");
}
#[test]
fn a_lost_read_keeps_has_goals() {
    let mut h = FactHarness::new(me());
    let (watch, read) = fact_requests(&h.take(), fixtures::PROJECT);
    h.live(watch, &[]);
    h.answer(read, &[line(GOAL, Wall::EPOCH)]);
    pages::step(
        &mut h.root.pages,
        &fixtures::env(&fact_limits()),
        &h.root.client,
        Event::Told {
            told: Told::Page {
                handle: reserved_handle(2),
                page: Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)),
            },
        },
        &mut Queue::uncapped(),
    );
    assert!(h.root.has_goals(fixtures::PROJECT));
}
