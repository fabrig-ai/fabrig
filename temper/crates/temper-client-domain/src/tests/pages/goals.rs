use super::{chats::chat_line, fixtures};
use crate::{
    Address, GoalsFilter,
    pages::{
        self, Act, Event, Pages, Request,
        goals::{self, Goals, State, Wait},
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
// Two rows of the first ended page, one of the next, and one live row.
const ENDED_READ: u32 = 3;
const ALL_READ: u32 = 4;
const CURRENT_ENDED: usize = 1;

pub(super) fn me() -> c::Me {
    let mut me = fixtures::chats::me();
    let template = me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap();
    template.number = jig_client::shapes::GOAL;
    template.tracked = Some(1);
    me
}
pub(super) fn line(task: u64, activity: Wall) -> c::TaskLine {
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
    page_at(lines, more, 0)
}
fn page_at(lines: &[c::TaskLine], more: bool, ended_mark: u64) -> c::ReadValue {
    c::ReadValue::Tasks(Box::new(c::TaskPage {
        ended_mark,
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
        self.live_at(lines, more, 0);
    }
    fn live_at(&mut self, lines: &[c::TaskLine], more: bool, ended_mark: u64) {
        self.event(crate::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched {
                    watch: self.watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                        rows: rows(lines),
                        more,
                        ended_mark,
                    }))),
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
    let mut ended = None;
    for (token, read) in reads {
        if let c::Read::Tasks { .. } = read {
            assert!(ended.replace(*token).is_none(), "one ended read at a time");
        }
    }
    ended
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
pub(super) fn report_part(headline: Option<&[u8]>) -> c::RowPart {
    let report = match headline {
        Some(headline) => Some((headline, LATER)),
        None => None,
    };
    temper_fixtures::client_documents::goal_part(report, Some(42))
}
pub(super) fn with_part(task: u64, part: c::RowPart) -> c::TaskLine {
    let mut line = line(task, Wall::EPOCH);
    line.parts = List::from_boxed(Box::new([part]));
    line
}

#[test]
fn goals_read_its_query_and_release_it() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let mut goals = Goals::new(FIRST);
    let mut out = Queue::uncapped();
    goals.configure(Some(&me()), fixtures::PROJECT, root.client(), &limits, 0, &mut out);
    let query = c::TaskQuery {
        parts: List::from_boxed(Box::new([temper_client_domain_forge::parts::asks(
            temper_client_protocol_wire::GOAL_PART,
        )])),
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
            subject: Watchable::Jig(c::Subject::Tasks { query: query.clone(), window: limits.client.domain.rows })
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Read {
            handle: goals.ended,
            read: Readable::Jig(c::Read::Tasks {
                query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query.clone() },
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
    goals.configure(Some(&me()), fixtures::PROJECT, root.client(), &limits, 0, &mut out);
    assert!(out.is_empty(), "the same query keeps its reads");
    let mut changed = me();
    changed.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().label = LABEL + 1;
    goals.configure(Some(&changed), fixtures::PROJECT, root.client(), &limits, 0, &mut out);
    for handle in [goals.live, goals.ended, goals.needs] {
        assert_eq!(out.pop(), Some(Request::Client(jig_client::Event::Unwatch { handle })));
    }
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Watch {
            handle: goals.live,
            subject: Watchable::Jig(c::Subject::Tasks {
                query: c::TaskQuery { label: Some(LABEL + 1), ..query.clone() },
                window: limits.client.domain.rows
            })
        }))
    );
    while out.pop().is_some() {}
    let next = goals.leave(&mut out);
    for handle in [goals.live, goals.ended, goals.needs] {
        assert_eq!(out.pop(), Some(Request::Client(jig_client::Event::Unwatch { handle })));
    }
    assert!(Goals::new(next).live.number > goals.needs.number);
    assert!(goals::query(&fixtures::chats::me(), fixtures::PROJECT).is_none());
    assert!(goals::query(&me(), OTHER.try_into().unwrap()).is_none());
    assert!(out.is_empty());
}

fn phase(chip: &mut c::Chip, phase: c::ChipPhase) {
    chip.phase = phase.compact();
    chip.details.phase = phase;
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
    for (dependencies, more, holds, expected) in [
        (&[12][..], false, false, State::Waits(Wait { first: 12, second: None, count: 1, more: false })),
        (&[12, 13, 14][..], false, true, State::Waits(Wait { first: 12, second: Some(13), count: 3, more: false })),
        (&[][..], true, true, State::Waits(Wait { first: 0, second: None, count: 0, more: true })),
        (&[][..], false, true, State::WaitsForRoom),
        (&[][..], false, false, State::Planning),
    ] {
        let mut chip = base.clone();
        phase(&mut chip, c::ChipPhase::Waiting { dependencies: List::from_boxed(dependencies.into()), more, holds });
        cases.push((chip.clone(), false, false, 0, expected)).unwrap();
        if expected != State::Planning {
            cases.push((chip.clone(), false, true, 1, expected)).unwrap();
        }
        cases.push((chip, true, false, 0, State::Needs { funds: false })).unwrap();
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
            State::Planning | State::Paused | State::Waits(_) | State::WaitsForRoom | State::Running => {
                [true, false, true, false]
            }
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
fn a_goals_id_is_its_leading_capitals_and_digits() {
    for (title, expected) in [
        ("S7 history in task logs", Some("S7")),
        ("K10 · prepare reconnect statements once", Some("K10")),
        ("H1, Home's parts", Some("H1")),
        ("ABC123", Some("ABC123")),
        ("A0", Some("A0")),
        ("Fix 3 bugs", None),
        ("s7 history", None),
        ("SS7a", None),
        ("ABCD1", None),
        ("A1234", None),
        ("A", None),
        ("1", None),
        ("", None),
    ] {
        let expected = match expected {
            Some(id) => Some(id.as_bytes()),
            None => None,
        };
        assert_eq!(goals::id(title.as_bytes()), expected, "{title}");
    }
}

#[test]
fn reports_arrive_with_live_and_ended_rows_without_notes() {
    let mut h = Harness::new();
    let read = ended_read(&h.reads());
    h.live(&[with_part(GOAL, report_part(Some(b"Two of three landed")))], false);
    h.answer(read, page(&[with_part(OTHER, report_part(Some(b"Ended report")))], false));
    let report = h.goals().report(GOAL).unwrap();
    assert_eq!(report.headline.as_ref(), b"Two of three landed");
    assert_eq!(report.at, Some(LATER));
    assert!(report.paragraphs.is_empty());
    assert_eq!(h.goals().report(OTHER).unwrap().headline.as_ref(), b"Ended report");
    assert!(reports(&h.reads()).is_empty());
    h.live(&[with_part(GOAL, report_part(Some(b"Next delivery")))], false);
    assert_eq!(h.goals().report(GOAL).unwrap().headline.as_ref(), b"Next delivery");
    h.live(&[with_part(GOAL, c::RowPart::Unknown)], false);
    assert_eq!(h.goals().report(GOAL).unwrap().headline.as_ref(), b"Next delivery");
    assert!(reports(&h.reads()).is_empty());
}

#[test]
fn unavailable_parts_read_nothing_and_the_next_report_wins() {
    for part in [
        c::RowPart::Unknown,
        c::RowPart::NotServed(c::NotServed {
            owner: c::Owner::Family(temper_client_protocol_wire::FORGE),
            served: List::with_capacity(0),
        }),
        c::RowPart::Failed(c::PartFailure::Busy),
        report_part(None),
    ] {
        let mut h = Harness::new();
        h.reads();
        h.live(&[with_part(GOAL, part)], false);
        assert!(h.goals().report(GOAL).is_none());
        assert!(reports(&h.reads()).is_empty());
        h.live(&[with_part(GOAL, report_part(Some(b"Now written")))], false);
        assert_eq!(h.goals().report(GOAL).unwrap().headline.as_ref(), b"Now written");
        assert!(reports(&h.reads()).is_empty());
    }
}

#[test]
fn goal_parts_require_the_forge_version_and_a_complete_record() {
    use temper_client_domain_forge::parts;
    let part = report_part(Some(b"Written"));
    let c::RowPart::Value(value) = &part else { unreachable!() };
    let decoded = parts::goal(&with_part(GOAL, part.clone())).unwrap();
    assert_eq!(decoded.milestone, Some(42));
    assert_eq!(decoded.report.unwrap().headline.as_ref(), b"Written");
    let mut trailing = List::uncapped();
    for byte in value.bytes.iter().copied() {
        trailing.push(byte).unwrap();
    }
    trailing.push(0).unwrap();
    for value in [
        jig_client::Family { family: 99, ..value.clone() },
        jig_client::Family { version: 0, ..value.clone() },
        jig_client::Family { bytes: Box::new([]), ..value.clone() },
        jig_client::Family { bytes: trailing.into_boxed(), ..value.clone() },
    ] {
        assert!(parts::goal(&with_part(GOAL, c::RowPart::Value(value))).is_none());
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
    assert!(h.goals().needs_own(GOAL));
    assert!(!h.goals().needs_own(OTHER));
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
    assert!(!h.goals().needs_own(GOAL));
    h.event(crate::Event::Client {
        event: jig_client::Event::Up {
            up: Up::Watched { watch: h.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(changed))) },
        },
    });
    assert!(h.reads().is_empty(), "unchanged total keeps the read");
    h.live(&[], false);
    for (_, read) in h.reads() {
        if let c::Read::Inbox { .. } = read {
            panic!("a goal delivery is no inbox mark");
        }
    }
}

#[test]
fn ended_goals_read_again_only_when_the_mark_grows() {
    let mut h = Harness::new();
    let first = ended_read(&h.reads());
    h.answer(first, page_at(&[], false, 5));
    h.reads();
    h.live_at(&[line(GOAL, Wall::EPOCH)], false, 5);
    assert_eq!(maybe_ended_read(&h.reads()), None, "the first delivery keeps the opening read");
    h.live_at(&[line(GOAL, LATER)], false, 5);
    assert_eq!(maybe_ended_read(&h.reads()), None, "a changed live row is no ended change");
    h.live_at(&[line(GOAL, LATER)], false, 6);
    let reads = h.reads();
    let refresh = ended_read(&reads);
    let c::Read::Tasks { ended, .. } = &reads[0].1 else { panic!("ended read") };
    assert_eq!(*ended, None, "even equal live rows can hide a task that started and ended");
    h.answer(refresh, page_at(&[line(OTHER, LATER)], false, 6));
    assert_eq!(maybe_ended_read(&h.reads()), None);
    assert_eq!(h.goals().ended()[0].chip.task, OTHER);
}

#[test]
fn ended_goals_coalesce_marks_while_the_read_is_under_way() {
    for answered_mark in [5, 6, 7] {
        let mut h = Harness::new();
        let first = ended_read(&h.reads());
        h.live_at(&[], false, 6);
        assert_eq!(maybe_ended_read(&h.reads()), None, "opening read stays in flight");
        h.live_at(&[], false, 5);
        assert_eq!(maybe_ended_read(&h.reads()), None, "keep the greatest delivery mark");
        h.answer(first, page_at(&[], false, answered_mark));
        let reads = h.reads();
        if answered_mark < 6 {
            let refresh = ended_read(&reads);
            h.live_at(&[], false, 7);
            h.live_at(&[], false, 6);
            assert_eq!(maybe_ended_read(&h.reads()), None, "one follow-up stays in flight");
            h.answer(refresh, page_at(&[], false, 6));
            let next = ended_read(&h.reads());
            h.answer(next, page_at(&[], false, 7));
            assert_eq!(maybe_ended_read(&h.reads()), None, "one follow-up per uncovered mark suffices");
        } else {
            assert_eq!(maybe_ended_read(&reads), None, "the answer already covers the delivery");
        }
    }
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
    h.live_at(&[], false, 1);
    let refresh = h.reads();
    let c::Read::Tasks { ended, .. } = &refresh[0].1 else { panic!("first page refresh") };
    assert_eq!(*ended, None, "a grown mark refreshes the first page");
    h.answer(ended_read(&refresh), page_at(&[line(GOAL, Wall::EPOCH), line(OTHER, Wall::EPOCH)], true, 1));
    h.reads();
    assert_eq!(h.goals().count(h.root.client()), WINDOW, "refreshing the first page counts its rows once");
    h.act(Act::MoreGoals);
    let reads = h.reads();
    let c::Read::Tasks { ended, .. } = &reads[0].1 else { panic!("older page") };
    assert!(ended.is_some());
    h.answer(ended_read(&reads), page_at(&[line(OTHER, LATER)], false, 1));
    h.reads();
    assert_eq!(h.goals().ended().len(), CURRENT_ENDED);
    assert_eq!(h.goals().count(h.root.client()), ENDED_READ, "two first rows and one next row");
    assert!(!h.goals().more());
    h.live_at(&[], false, 2);
    let reads = h.reads();
    assert_eq!(maybe_ended_read(&reads), None, "older page stays put even as the mark grows");
    h.live_at(&[line(GOAL, LATER)], true, 2);
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
    let handles = [handle.number - 1, handle.number, handle.number + 1];
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
    let previous = [old.live, old.ended, old.needs];
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
    assert!(pages.goals().unwrap().live.number > previous[2].number, "another project gets new handles");
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
        Self::at(identity, Address::Home { project: fixtures::PROJECT })
    }
    fn at(identity: c::Me, address: Address) -> Self {
        let mut root = crate::unlinked(&fact_limits(), fixtures::SEED);
        let me = fixtures::start(&mut root, List::with_capacity(0));
        let mut h = Self { root, out: Queue::uncapped(), me };
        h.go(address);
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
        self.live_at(watch, lines, 0);
    }
    fn live_at(&mut self, watch: Token, lines: &[c::TaskLine], ended_mark: u64) {
        self.up(Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                ended_mark,
                rows: rows(lines),
                more: false,
            }))),
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
                    c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query.clone() }
                );
                assert_eq!(*live, None);
                assert_eq!(*ended, None);
                assert!(read.replace(*token).is_none(), "one fact ended read");
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

fn list_requests(down: &[Down]) -> (Token, Token) {
    let query = goals::query(&me(), fixtures::PROJECT).unwrap();
    let ended_query = c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query.clone() };
    let mut watch = None;
    let mut read = None;
    for request in down {
        match request {
            Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Tasks { query: got, window }) }
                if *got == query =>
            {
                assert_eq!(*window, WINDOW, "only the list window, never a narrower twin");
                assert!(watch.replace(*token).is_none(), "one list watch");
            }
            Down::Read { read: token, what: Readable::Jig(c::Read::Tasks { query: got, live: None, ended: None }) }
                if *got == ended_query =>
            {
                assert!(read.replace(*token).is_none(), "one ended read, from the list");
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
    (watch.expect("list watch"), read.expect("list ended read"))
}

#[test]
fn pages_with_goals_ask_the_list_before_the_fact() {
    for address in [
        Address::Work { project: fixtures::PROJECT, filter: crate::WorkFilter::All },
        Address::Goals { project: fixtures::PROJECT, filter: GoalsFilter::All },
        Address::Changes { project: fixtures::PROJECT },
    ] {
        let mut h = FactHarness::at(me(), address);
        let (watch, read) = list_requests(&h.take());
        h.answer(read, &[]);
        assert!(!h.root.has_goals(fixtures::PROJECT));
        h.deliver_me(me());
        assert!(h.take().is_empty(), "no second-step twin to open or close");
        h.live(watch, &[line(GOAL, Wall::EPOCH)]);
        assert!(h.root.has_goals(fixtures::PROJECT), "a live list row supplies the fact");
        h.take();
        h.live_at(watch, &[], 1);
        assert!(!h.root.has_goals(fixtures::PROJECT));
        let mut read = None;
        for request in h.take() {
            if let Down::Read { read: token, what: Readable::Jig(c::Read::Tasks { query, .. }) } = request
                && query.label == Some(LABEL)
            {
                read = Some(token);
            }
        }
        h.up(Up::ReadDone {
            read: read.expect("list's refreshed ended read"),
            result: Readout::Jig(c::ReadResult::Value(page_at(&[line(GOAL, Wall::EPOCH)], false, 1))),
        });
        assert!(h.root.has_goals(fixtures::PROJECT), "the list's ended row supplies the fact");
    }
}

fn inbox_watch(down: &[Down]) -> Token {
    for request in down {
        if let Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) } = request {
            return *watch;
        }
    }
    panic!("standing inbox watch");
}
fn needs_reads(down: &[Down]) -> usize {
    let mut count = 0_usize;
    for request in down {
        if let Down::Read {
            what:
                Readable::Jig(c::Read::Inbox { group: c::InboxReadGroup::Needs, project: Some(fixtures::PROJECT), .. }),
            ..
        } = request
        {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}

#[test]
fn needs_read_again_on_inbox_moves_including_the_first_after_reopening() {
    let address = Address::Work { project: fixtures::PROJECT, filter: crate::WorkFilter::All };
    let mut h = FactHarness::at(me(), address.clone());
    let opened = h.take();
    assert_eq!(needs_reads(&opened), 1, "once on opening");
    let inbox = inbox_watch(&opened);
    let deliver = |h: &mut FactHarness, watch| {
        h.up(Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(temper_fixtures::client_documents::inbox(
                List::uncapped(),
                0,
                c::Count { value: 0, limited: false },
                0,
                false,
            )))),
        });
    };
    deliver(&mut h, inbox);
    assert_eq!(needs_reads(&h.take()), 0, "the first delivery is no move");
    deliver(&mut h, inbox);
    assert_eq!(needs_reads(&h.take()), 1, "a later delivery is a move, even with the same rows");
    deliver(&mut h, inbox);
    assert_eq!(needs_reads(&h.take()), 1, "each later delivery moves the mark");
    let goals_watch = list_requests(&opened).0;
    h.live(goals_watch, &[]);
    assert_eq!(needs_reads(&h.take()), 0, "goals do not mark the inbox");
    h.go(Address::Welcome { step: crate::Step::Ready });
    assert!(h.take().contains(&Down::Close { watch: inbox }));
    h.go(address);
    let reopened = h.take();
    assert_eq!(needs_reads(&reopened), 1, "the new list reads once");
    let inbox = inbox_watch(&reopened);
    deliver(&mut h, inbox);
    assert_eq!(needs_reads(&h.take()), 1, "the first delivery after reopening is a move");
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
        let handles = [goals.live, goals.ended, goals.needs];
        let inbox = h.root.pages().inbox().unwrap();
        assert!(handles[0].number > inbox.entries.number);
        assert!(inbox.next > handles[2].number, "later inbox cards cannot reuse goal handles");
        let requests = h.take();
        if ended {
            // An opening watch's close waits for its first delivery.
            h.live(watch, &[]);
            assert!(h.take().contains(&Down::Close { watch }));
        } else {
            assert!(requests.contains(&Down::Close { watch }), "Home releases the fact watch");
        }
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
fn has_goals_takes_the_list_then_stands_until_its_own_first_delivery() {
    for address in [Address::Settings { project: fixtures::PROJECT }, Address::Chat { task: GOAL }] {
        let mut h = FactHarness::at(me(), Address::Goals { project: fixtures::PROJECT, filter: GoalsFilter::All });
        let opened = h.take();
        let watch = list_requests(&opened).0;
        h.live(watch, &[line(GOAL, Wall::EPOCH)]);
        assert!(h.root.has_goals(fixtures::PROJECT));
        assert!(!h.root.has_goals(OTHER.try_into().unwrap()), "only the open project");
        h.take();
        h.go(address);
        assert!(h.root.has_goals(fixtures::PROJECT), "leaving the list keeps its last value");
        let requests = h.take();
        assert!(requests.contains(&Down::Close { watch }));
        let (own, ended) = fact_requests(&requests, fixtures::PROJECT);
        h.answer(ended, &[]);
        assert!(h.root.has_goals(fixtures::PROJECT), "an empty ended read is not the watch's first delivery");
        h.live(own, &[]);
        assert!(!h.root.has_goals(fixtures::PROJECT), "the own watch now answers");
        assert!(h.take().is_empty());
    }
}

#[test]
fn an_ended_goal_counts() {
    let mut h = FactHarness::at(me(), Address::Settings { project: fixtures::PROJECT });
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
    let mut h = FactHarness::at(identity.clone(), Address::Settings { project: fixtures::PROJECT });
    let (watch, read) = fact_requests(&h.take(), fixtures::PROJECT);
    h.live(watch, &[]);
    h.answer(read, &[line(GOAL, Wall::EPOCH)]);
    h.take();
    h.go(Address::Settings { project: OTHER.try_into().unwrap() });
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
        let mut h = FactHarness::at(me(), Address::Settings { project: fixtures::PROJECT });
        let (_, read) = fact_requests(&h.take(), fixtures::PROJECT);
        if delivered {
            h.answer(read, &[line(GOAL, Wall::EPOCH)]);
            assert!(h.root.has_goals(fixtures::PROJECT));
        }
        h.event(crate::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } });
        assert!(!h.root.has_goals(fixtures::PROJECT), "sign-in releases the old fact");
        h.take();
        h.me = Token::new(0);
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
    assert!(!h.reads().is_empty(), "lost read releases the page read");
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

#[test]
fn missing_goal_parts_keep_the_last_report_until_a_decoded_absence() {
    for missing in [
        c::RowPart::Failed(c::PartFailure::Busy),
        c::RowPart::Unknown,
        c::RowPart::NotServed(c::NotServed {
            owner: c::Owner::Family(temper_client_protocol_wire::FORGE),
            served: List::uncapped(),
        }),
        c::RowPart::Failed(c::PartFailure::Unreadable),
    ] {
        let mut h = Harness::new();
        h.reads();
        h.live(&[with_part(GOAL, report_part(Some(b"K3 lands the codec's shape")))], false);
        h.live(&[with_part(GOAL, missing)], false);
        assert_eq!(h.goals().report(GOAL).unwrap().headline.as_ref(), b"K3 lands the codec's shape");
        assert!(reports(&h.reads()).is_empty(), "no read replaces a missing part");
        h.live(&[with_part(GOAL, report_part(None))], false);
        assert!(h.goals().report(GOAL).is_none(), "a decoded absence replaces the report");
    }
}

#[test]
fn ended_goal_parts_stand_through_a_mark_refresh_and_leave_with_the_row() {
    let mut h = Harness::new();
    let first = ended_read(&h.reads());
    h.answer(first, page_at(&[with_part(GOAL, report_part(Some(b"Ended headline")))], false, 5));
    h.live_at(&[], false, 5);
    h.live_at(&[], false, 6);
    let refresh = ended_read(&h.reads());
    h.answer(refresh, page_at(&[with_part(GOAL, c::RowPart::Failed(c::PartFailure::Busy))], false, 6));
    assert_eq!(h.goals().report(GOAL).unwrap().headline.as_ref(), b"Ended headline");
    h.live_at(&[], false, 7);
    let refresh = ended_read(&h.reads());
    h.answer(refresh, page_at(&[], false, 7));
    assert!(h.goals().report(GOAL).is_none());
    assert_eq!(h.goals().membership(GOAL), None, "leaving the list drops every kept field");
    h.live_at(&[with_part(GOAL, c::RowPart::Unknown)], false, 7);
    assert!(h.goals().report(GOAL).is_none(), "a returning row has no kept report");
}

#[test]
fn live_goal_parts_leave_with_the_row_and_the_shared_list() {
    let mut h = Harness::new();
    h.reads();
    h.live(&[with_part(GOAL, report_part(Some(b"K3 lands")))], false);
    h.live(&[], false);
    assert_eq!(h.goals().membership(GOAL), None);
    h.live(&[with_part(GOAL, c::RowPart::Unknown)], false);
    assert!(h.goals().report(GOAL).is_none());
    h.live(&[with_part(GOAL, report_part(Some(b"K3 lands")))], false);
    h.event(crate::Event::Went { place: crate::Place::Address(Address::Settings { project: fixtures::PROJECT }) });
    assert!(h.root.pages().goals().is_none(), "leaving the shared list drops its parts");
}

#[test]
fn unanswered_needs_reads_keep_the_selection() {
    for result in [
        Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)),
        Readout::Limit,
        Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)),
    ] {
        let mut h = Harness::new();
        let needs = needs_read(&h.reads());
        h.answer(
            needs,
            c::ReadValue::Inbox(Box::new(c::InboxPage {
                selection: Some(List::from_boxed(Box::new([c::InboxSelection {
                    cursor: c::InboxCursor::Number { number: 1 },
                    goals: List::from_boxed(Box::new([GOAL, OTHER])),
                }]))),
                rows: List::uncapped(),
                before: None,
                after: None,
                cut: false,
            })),
        );
        let mut changed = me();
        changed.needs_total = 1;
        h.event(crate::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched { watch: h.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(changed))) },
            },
        });
        let needs = needs_read(&h.reads());
        h.result(needs, result);
        assert!(h.goals().needs_own(GOAL));
        assert!(h.goals().needs_person(GOAL));
        assert!(h.goals().needs_person(OTHER));
        assert!(!h.goals().needs_own(OTHER));
    }
}
