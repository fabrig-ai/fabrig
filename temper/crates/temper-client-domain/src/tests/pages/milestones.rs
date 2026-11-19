#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "test fixtures collect and compare the bounded request stream"
)]
use super::{
    fixtures as f,
    goals::{line, me as goals_me},
};
use crate::{
    Address, GoalsFilter, WorkFilter,
    pages::{
        Request,
        milestones::{self, Count, Headline, Mark, MilestoneRead, Milestones, Segment, State},
    },
};
use jig_client::{Down, Readable, Readout, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};

const PROJECT: u32 = f::PROJECT;
const LABEL: u32 = 4;
const M: u64 = 100;
const DAY: u64 = 86_400_000_000_000;
const WALL: Wall = Wall::from_nanos(10 * DAY);
const WINDOW: u32 = 32;
fn me() -> c::Me {
    let mut me = goals_me();
    let templates = &mut me.projects.get_mut(0).unwrap().templates;
    let mut milestone = templates.get(0).unwrap().clone();
    milestone.number = temper_client_domain_forge::milestones::TEMPLATE;
    milestone.label = LABEL;
    let goal = templates.get(0).unwrap().clone();
    *templates = List::from_boxed(Box::new([goal, milestone]));
    me
}
fn limits() -> crate::Limits {
    let mut limits = f::limits();
    limits.client.domain.handles = 256;
    limits.client.domain.watches = 256;
    limits.client.domain.rows = WINDOW;
    limits.client.domain.reads = 256;
    limits.client.domain.requests.requests = 256;
    limits.client.domain.requests.bytes = 16_777_216;
    limits.client.domain.requests.out = 512;
    limits
}
fn in_milestone(mut line: c::TaskLine, task: u64) -> c::TaskLine {
    line.parts = List::from_boxed(Box::new([temper_fixtures::client_documents::goal_part(None, Some(task))]));
    line
}
fn row_part(mut line: c::TaskLine, goals: &[c::TaskLine], more: u32) -> c::TaskLine {
    let goals: Vec<_> = goals.iter().cloned().map(|line| (line, Some(b"Part report headline.".as_slice()))).collect();
    line.parts =
        List::from_boxed(Box::new([temper_fixtures::milestones::part(Some(WALL), totals(2, 0, 17), &goals, more)]));
    line
}
fn no_drawn_reads(down: &[Down]) {
    assert!(task_reads(down).is_empty(), "no task document or requester reads");
    for request in down {
        match request {
            Down::Read { what: Readable::Family { record, .. }, .. } => {
                if let Some(reading) = wire::decode_reading(&record.bytes, &w::CEILINGS)
                    && let w::Read::Milestone(_) = reading.read()
                {
                    panic!("no milestone totals read");
                }
            }
            Down::Open { subject: Watchable::Jig(c::Subject::Tasks { query, .. }), .. }
            | Down::Read { what: Readable::Jig(c::Read::Tasks { query, .. }), .. } => {
                assert!(query.parent.is_none(), "no drawn goal query");
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
}
fn phase(line: &mut c::TaskLine, phase: c::ChipPhase) {
    line.chip.phase = phase.compact();
    line.chip.details.phase = phase;
}
fn ending(task: u64, ending: c::ChipEnding, at: Wall) -> c::TaskLine {
    let mut row = line(task, at);
    phase(&mut row, c::ChipPhase::Ended);
    row.chip.details.details.ending = Some(ending);
    row.ended_at = Some(at);
    row
}
fn done(task: u64) -> c::TaskLine {
    ending(task, c::ChipEnding::Done(c::ResultValue::Report(f::chat::document_text(b"done"))), WALL)
}
fn failed(task: u64) -> c::TaskLine {
    ending(task, c::ChipEnding::Failed(f::chat::document_text(b"failed")), WALL)
}
fn closed(task: u64) -> c::TaskLine {
    ending(
        task,
        c::ChipEnding::Cancelled {
            by: c::Requester::Person(1),
            reason: f::chat::document_text(b"closed"),
            result: None,
            at: WALL,
        },
        WALL,
    )
}
fn totals(done: u32, failed: u32, live: u32) -> w::MilestoneTotals {
    w::MilestoneTotals::new(
        &w::CEILINGS,
        w::MilestoneTotalsParts {
            task: M,
            done,
            failed,
            live,
            closed: 50,
            ended: None,
            last_landing: None,
            spent: 0,
            reds: 0,
        },
    )
    .unwrap()
}
fn page(rows: &[c::TaskLine], more: bool) -> Readout {
    page_at(rows, more, 0)
}
fn page_at(rows: &[c::TaskLine], more: bool, ended_mark: u64) -> Readout {
    Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(c::TaskPage {
        ended_mark,
        live: List::with_capacity(0),
        live_after: None,
        ended: c::EndedGroup::Rows {
            rows: List::from_boxed(rows.into()),
            after: if more { Some(c::Cursor::new(b"next".as_slice().into(), 32).unwrap()) } else { None },
            cut: false,
        },
    }))))
}
struct Harness {
    root: crate::Domain,
    limits: crate::Limits,
    out: Queue<crate::Request>,
}
impl Harness {
    fn new(me: c::Me) -> Self {
        Self::at(me, None)
    }
    fn at(me: c::Me, address: Option<Address>) -> Self {
        let limits = limits();
        let mut root = crate::unlinked(&limits, f::SEED);
        let watch = f::start(&mut root, List::with_capacity(0));
        let mut h = Self { root, limits, out: Queue::uncapped() };
        if let Some(address) = address {
            h.went(address);
        }
        h.event(crate::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        });
        h
    }
    fn event(&mut self, event: crate::Event) {
        let mut env = f::env(&self.limits);
        env.wall = WALL;
        crate::step(&mut self.root, &env, event, &mut self.out);
    }
    fn went(&mut self, address: Address) {
        self.event(crate::Event::Went { place: crate::Place::Address(address) });
    }
    fn down(&mut self) -> Vec<Down> {
        let mut down = Vec::new();
        while let Some(request) = self.out.pop() {
            if let crate::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: request } } = request {
                down.push(request);
            }
        }
        down
    }
    fn live(&mut self, watch: Token, rows: &[c::TaskLine]) {
        self.live_at(watch, rows, 0);
    }
    fn live_at(&mut self, watch: Token, rows: &[c::TaskLine], ended_mark: u64) {
        self.event(crate::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                        ended_mark,
                        rows: List::from_boxed(rows.into()),
                        more: false,
                    }))),
                },
            },
        });
    }
    fn answer(&mut self, read: Token, result: Readout) {
        self.event(crate::Event::Client { event: jig_client::Event::Up { up: Up::ReadDone { read, result } } });
    }
    fn reads(&self) -> &Milestones {
        self.root.pages().milestones().unwrap()
    }
    fn open(&mut self) -> Vec<Down> {
        self.went(Address::Goals { project: PROJECT, filter: GoalsFilter::All });
        self.down()
    }
}
fn watch(down: &[Down], label: Option<u32>, parent: Option<u64>, window: u32) -> Token {
    for request in down {
        if let Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, window: size }) } = request
            && query.label == label
            && query.parent == parent
            && *size == window
        {
            return *watch;
        }
    }
    panic!("task watch label {label:?}, parent {parent:?}")
}
fn ended_read(down: &[Down], label: Option<u32>, parent: Option<u64>) -> Token {
    for request in down {
        if let Down::Read { read, what: Readable::Jig(c::Read::Tasks { query, .. }) } = request
            && query.label == label
            && query.parent == parent
        {
            return *read;
        }
    }
    panic!("ended read label {label:?}, parent {parent:?}")
}
fn task_reads(down: &[Down]) -> Vec<(Token, u64)> {
    down.iter()
        .filter_map(|request| match request {
            Down::Read { read, what: Readable::Jig(c::Read::Task { task }) } => Some((*read, *task)),
            Down::Save { .. }
            | Down::Erase { .. }
            | Down::Send { .. }
            | Down::Open { .. }
            | Down::Continue { .. }
            | Down::Close { .. }
            | Down::Read { .. } => None,
        })
        .collect()
}
#[test]
fn home_rows_group_before_sorting_and_limiting_and_filters_keep_first_position() {
    use crate::pages::goals::{
        GroupRow::{Goal, Milestone},
        groups, home_rows,
    };
    let mut h = Harness::new(me());
    h.down();
    let opened = h.open();
    let listed = watch(&opened, Some(LABEL), None, WINDOW);
    h.live(listed, &[line(M, Wall::from_nanos(9))]);
    no_drawn_reads(&h.down());
    let mut g3 = line(3, Wall::from_nanos(11));
    g3.requester = c::Requester::Task(M);
    let mut g4 = done(4);
    g4.last_activity = Wall::from_nanos(8);
    g4.requester = c::Requester::Task(M);
    let g3 = in_milestone(g3, M);
    let g4 = in_milestone(g4, M);
    // The shared read can know a goal before the milestone's ended read answers.
    let g1 = line(1, Wall::from_nanos(10));
    let g2 = done(2);
    let live = [g1.clone(), g3.clone()];
    let ended = [g2.clone(), g4.clone()];
    assert_eq!(
        home_rows(&live, &ended, Some(h.reads()), h.root.pages().goals().unwrap(), WALL).as_slice(),
        [Milestone(M), Goal(1), Goal(2)]
    );
    assert_eq!(
        groups(&[g1.clone(), g4, g2.clone(), g3], Some(h.reads()), h.root.pages().goals().unwrap()).as_slice(),
        [Goal(1), Milestone(M), Goal(2)]
    );
    // Equal activity takes the lower task, even when its kind differs.
    let tie = line(1, Wall::from_nanos(11));
    assert_eq!(
        home_rows(&[tie], &[], Some(h.reads()), h.root.pages().goals().unwrap(), WALL).as_slice(),
        [Goal(1), Milestone(M)]
    );
    let live: Vec<_> = (1..=6).map(|task| line(task, Wall::from_nanos(task))).collect();
    let mut older = done(8);
    older.ended_at = Some(Wall::from_nanos(1));
    assert_eq!(home_rows(&live, core::slice::from_ref(&g2), None, h.root.pages().goals().unwrap(), WALL).len(), 6);
    assert_eq!(
        home_rows(&live[..2], &[older, done(7)], None, h.root.pages().goals().unwrap(), WALL).as_slice(),
        [Goal(2), Goal(1), Goal(7), Goal(8)]
    );
    assert_eq!(
        home_rows(&live[..2], core::slice::from_ref(&g1), None, h.root.pages().goals().unwrap(), WALL).as_slice(),
        [Goal(2), Goal(1)],
        "a goal appears once"
    );
    let children: Vec<_> = (10..18)
        .map(|task| {
            let mut goal = line(task, Wall::from_nanos(task));
            goal.requester = c::Requester::Task(M);
            in_milestone(goal, M)
        })
        .collect();
    let mut live = children;
    live.push(g1);
    let ended: Vec<_> = (20..26).map(done).collect();
    assert_eq!(
        home_rows(&live, &ended, Some(h.reads()), h.root.pages().goals().unwrap(), WALL).as_slice(),
        [Milestone(M), Goal(1), Goal(20), Goal(21), Goal(22)],
        "five rows after grouping, not ten live goals"
    );
}

#[test]
fn home_rows_hide_old_milestones_and_keep_unknown_and_undrawn_goals() {
    use crate::pages::goals::{
        GroupRow::{Goal, Milestone},
        home_rows,
    };
    let mut h = Harness::new(me());
    h.down();
    let opened = h.open();
    let listed = watch(&opened, Some(LABEL), None, WINDOW);
    let ended_read = ended_read(&opened, Some(LABEL), None);
    let mut recent = done(100);
    recent.ended_at = Some(Wall::from_nanos(7 * DAY));
    let mut old = done(101);
    old.ended_at = Some(Wall::from_nanos(2 * DAY));
    h.answer(ended_read, page(&[old, recent], false));
    let mut unknown = line(1, WALL);
    unknown.requester = c::Requester::Task(999);
    let mut recent_goal = in_milestone(done(2), 100);
    recent_goal.requester = c::Requester::Task(100);
    let mut old_goal = in_milestone(done(3), 101);
    old_goal.requester = c::Requester::Task(101);
    assert_eq!(
        home_rows(&[unknown], &[recent_goal, old_goal], Some(h.reads()), h.root.pages().goals().unwrap(), WALL)
            .as_slice(),
        [Goal(1), Milestone(100)]
    );
    let many: Vec<_> = (200..217).map(|task| line(task, WALL)).collect();
    h.live(listed, &many);
    let mut undrawn = in_milestone(line(4, WALL), 216);
    undrawn.requester = c::Requester::Task(216);
    assert!(h.reads().milestone(216).is_none());
    let rows = home_rows(&[undrawn], &[], Some(h.reads()), h.root.pages().goals().unwrap(), WALL);
    assert!(rows.as_slice().contains(&Goal(4)));
    assert_eq!(rows.len(), 17, "live rows are counted after grouping");
}

#[test]
fn milestones_read_the_query_and_keep_it_across_the_four_pages() {
    let root = crate::unlinked(&limits(), f::SEED);
    let mut next = 31;
    let mut reads = Milestones::new(&mut next);
    let mut out = Queue::uncapped();
    reads.configure(Some(&goals_me()), PROJECT, root.client(), &limits(), WALL, &mut out);
    assert!(out.is_empty(), "no template 8, no reads");
    reads.configure(Some(&me()), PROJECT, root.client(), &limits(), WALL, &mut out);
    let query = c::TaskQuery {
        parts: List::from_boxed(Box::new([temper_client_domain_forge::parts::asks(wire::MILESTONE_PART)])),
        project: Some(PROJECT),
        label: Some(LABEL),
        tracked: Some(true),
        order: c::QueryOrder::Priority,
        requester: None,
        root: None,
        parent: None,
        phase: None,
        executor: None,
    };
    assert_eq!(milestones::query(&me(), PROJECT), Some(query.clone()));
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Watch {
            handle: reads.live,
            subject: Watchable::Jig(c::Subject::Tasks { query: query.clone(), window: WINDOW })
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Read {
            handle: reads.ended,
            read: Readable::Jig(c::Read::Tasks {
                query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query.clone() },
                live: None,
                ended: None
            })
        }))
    );
    assert!(out.is_empty());
    reads.configure(Some(&me()), PROJECT, root.client(), &limits(), WALL, &mut out);
    assert!(out.is_empty());
    reads.leave(&mut out);
    for handle in [reads.live, reads.ended] {
        assert_eq!(out.pop(), Some(Request::Client(jig_client::Event::Unwatch { handle })));
    }
    let mut h = Harness::new(me());
    h.down();
    let opened = h.open();
    let live = watch(&opened, Some(LABEL), None, WINDOW);
    h.answer(ended_read(&opened, Some(LABEL), None), page_at(&[], false, 5));
    h.down();
    h.live_at(live, &[line(M, WALL)], 5);
    let delivered = h.down();
    assert!(marked_ended_reads(&delivered).is_empty(), "the first live delivery does not reread ended milestones");
    no_drawn_reads(&delivered);
    let handles = (h.reads().live, h.reads().ended);
    for address in [
        Address::Home { project: PROJECT },
        Address::Work { project: PROJECT, filter: WorkFilter::All },
        Address::Changes { project: PROJECT },
        Address::Goals { project: PROJECT, filter: GoalsFilter::Live },
    ] {
        h.went(address.clone());
        h.down();
        assert_eq!((h.reads().live, h.reads().ended), handles);
        assert!(h.reads().milestone(M).is_some(), "drawn read kept on {address:?}");
    }
    h.went(Address::Task { task: 9 });
    assert!(h.root.pages().milestones().is_none());
    let mut closed = Vec::new();
    for down in h.down() {
        if let Down::Close { watch } = down {
            closed.push(watch);
        }
    }
    assert!(closed.contains(&live));
}

#[test]
fn drawn_milestones_take_their_facts_totals_and_sixteen_goals_only_from_the_row() {
    let mut h = Harness::new(me());
    h.down();
    let down = h.open();
    let listed = watch(&down, Some(LABEL), None, WINDOW);
    let own = line(M, WALL);
    h.live(listed, core::slice::from_ref(&own));
    no_drawn_reads(&h.down());
    let drawn = h.reads().milestone(M).unwrap();
    assert!(drawn.facts.is_none());
    assert!(drawn.totals.is_none());
    assert!(drawn.goals.is_empty());
    let mut goals: Vec<_> = (1..=16).rev().map(|task| line(task, Wall::from_nanos(task))).collect();
    goals[0] = done(16);
    goals[1] = done(15);
    let own = row_part(own, &goals, 3);
    h.live(listed, core::slice::from_ref(&own));
    no_drawn_reads(&h.down());
    let drawn = h.reads().milestone(M).unwrap();
    assert_eq!(drawn.facts.as_ref().unwrap().whole, Some(WALL));
    assert_eq!(drawn.facts.as_ref().unwrap().maker, own.requester);
    assert_eq!(drawn.facts.as_ref().unwrap().name.as_ref(), own.chip.title.bytes());
    assert_eq!(drawn.totals.as_ref().unwrap().done(), 2);
    assert_eq!(drawn.more, 3);
    assert_eq!(drawn.goals.iter().map(|goal| goal.chip.task).collect::<Vec<_>>(), (1..=16).collect::<Vec<_>>());
    assert_eq!(drawn.headline(16), Some(b"Part report headline.".as_slice()));
    // A delivery can lose its part, and never reads a replacement.
    let mut lost = own;
    lost.parts = List::from_boxed(Box::new([c::RowPart::Failed(c::PartFailure::Busy)]));
    h.live(listed, &[lost]);
    no_drawn_reads(&h.down());
    let drawn = h.reads().milestone(M).unwrap();
    assert_eq!(drawn.totals.as_ref().unwrap().done(), 2);
    assert_eq!(drawn.more, 3);
    assert_eq!(drawn.goals.len(), 16);
    assert_eq!(drawn.headline(16), Some(b"Part report headline.".as_slice()));
    assert_eq!(drawn.facts.as_ref().unwrap().whole, Some(WALL));
}

#[test]
#[expect(clippy::too_many_lines, reason = "one sequence holds four pages and marks coalesced in flight")]
fn a_milestone_pages_ended_goals_follow_marks_and_keep_four_merged_pages() {
    let mut h = Harness::new(me());
    h.down();
    let mut next = 90;
    let mut read = MilestoneRead::new(PROJECT, M, &mut next);
    let mut out = Queue::uncapped();
    read.configure_goals(h.root.client(), &h.limits, &mut out);
    let mut live_handle = None;
    let mut ended_handle = None;
    while let Some(request) = out.pop() {
        match request {
            Request::Client(jig_client::Event::Watch { handle, subject }) => {
                let Watchable::Jig(c::Subject::Tasks { query, .. }) = &subject else { panic!("goals watch") };
                assert_eq!(query.parts.as_slice(), [temper_client_domain_forge::parts::asks(wire::GOAL_PART)]);
                live_handle = Some(handle);
                h.event(crate::Event::Client { event: jig_client::Event::Watch { handle, subject } });
            }
            Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(c::Read::Tasks { query, .. }) }) => {
                assert_eq!(query.parts.as_slice(), [temper_client_domain_forge::parts::asks(wire::GOAL_PART)]);
                ended_handle = Some(handle);
            }
            Request::Push(_)
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::OpenTab { .. }
            | Request::Client(_)
            | Request::Show { .. }
            | Request::Replace { .. } => panic!("only the goal groups open"),
        }
    }
    let live_handle = live_handle.unwrap();
    let ended_handle = ended_handle.unwrap();
    let watch = watch(&h.down(), None, Some(M), WINDOW);
    let mut live = line(202, WALL);
    h.live_at(watch, core::slice::from_ref(&live), 5);
    read.told(&Told::Changed { handle: live_handle }, h.root.client(), &h.limits, &mut out);
    assert!(out.is_empty(), "opening read is in flight");
    read.told(
        &Told::Page { handle: ended_handle, page: page_at(&[done(201)], false, 5) },
        h.root.client(),
        &h.limits,
        &mut out,
    );
    assert!(out.is_empty(), "first delivery keeps the opening answer");
    live.changed = Wall::from_nanos(17);
    h.live_at(watch, core::slice::from_ref(&live), 5);
    read.told(&Told::Changed { handle: live_handle }, h.root.client(), &h.limits, &mut out);
    assert!(out.is_empty(), "a changed live row is no ended change");
    h.live_at(watch, core::slice::from_ref(&live), 6);
    read.told(&Told::Changed { handle: live_handle }, h.root.client(), &h.limits, &mut out);
    assert_eq!(out.len(), 1, "a greater mark rereads the first page");
    out.clear();
    for index in 0..4 {
        read.told(
            &Told::Page { handle: ended_handle, page: page_at(&[done(201 + index), done(202)], true, 6) },
            h.root.client(),
            &h.limits,
            &mut out,
        );
        if index < 3 {
            let Some(Request::Client(jig_client::Event::Read {
                read: Readable::Jig(c::Read::Tasks { ended: Some(_), .. }),
                ..
            })) = out.pop()
            else {
                panic!("follow an older page")
            };
        }
        assert!(out.is_empty(), "no fifth page");
    }
    read.configure_goals(h.root.client(), &h.limits, &mut out);
    assert!(read.more);
    assert_eq!(read.goals.iter().map(|g| g.chip.task).collect::<Vec<_>>(), [201, 202, 203, 204]);
    h.live_at(watch, core::slice::from_ref(&live), 7);
    read.told(&Told::Changed { handle: live_handle }, h.root.client(), &h.limits, &mut out);
    assert_eq!(out.len(), 1);
    out.clear();
    h.live_at(watch, core::slice::from_ref(&live), 8);
    read.told(&Told::Changed { handle: live_handle }, h.root.client(), &h.limits, &mut out);
    assert!(out.is_empty(), "marks coalesce during a read");
    read.told(
        &Told::Page { handle: ended_handle, page: page_at(&[done(201)], true, 7) },
        h.root.client(),
        &h.limits,
        &mut out,
    );
    let Some(Request::Client(jig_client::Event::Read {
        read: Readable::Jig(c::Read::Tasks { ended: None, .. }), ..
    })) = out.pop()
    else {
        panic!("an uncovered mark refreshes the first page as soon as the read answers")
    };
    assert!(out.is_empty());
    read.told(
        &Told::Page { handle: ended_handle, page: page_at(&[done(201), done(202)], false, 8) },
        h.root.client(),
        &h.limits,
        &mut out,
    );
    assert!(out.is_empty());
    read.configure_goals(h.root.client(), &h.limits, &mut out);
    assert_eq!(read.goals.iter().map(|g| g.chip.task).collect::<Vec<_>>(), [201, 202]);
    assert_eq!(read.goals.get(1).unwrap(), &live, "live wins the duplicate number");
    assert!(!read.more);
}

#[test]
fn drawn_selection_is_recent_live_first_and_bounded() {
    let mut h = Harness::new(me());
    h.down();
    let down = h.open();
    let listed = watch(&down, Some(LABEL), None, WINDOW);
    let ended = ended_read(&down, Some(LABEL), None);
    let recent = done(500);
    let mut boundary = done(501);
    boundary.ended_at = Some(Wall::from_nanos(3 * DAY));
    let mut old = done(502);
    old.ended_at = Some(Wall::from_nanos(3 * DAY - 1));
    h.answer(ended, page(&[old, boundary, recent], false));
    h.down();
    assert!(h.reads().milestone(500).is_some());
    assert!(h.reads().milestone(501).is_some());
    assert!(h.reads().milestone(502).is_none());
    let rows: Vec<_> = (0..17).map(|i| line(M + i, Wall::from_nanos(i))).collect();
    h.live(listed, &rows);
    h.down();
    assert!(h.reads().milestone(M).is_none(), "oldest activity loses the sixteenth place");
    assert!(h.reads().milestone(M + 1).is_some());
    assert!(h.reads().milestone(M + 16).is_some());
    assert_eq!(h.reads().drawn().len(), 16);
    assert_eq!(h.reads().drawn()[0].task, M + 16, "drawn live order is newest activity first");
    assert!(h.reads().milestone(500).is_none(), "live rows precede ended rows");
    assert_eq!(h.reads().lines().len(), 20, "listed lines include old ended rows, live then ended");
}

#[test]
fn a_goals_part_names_its_milestone_without_reading_any_requester() {
    use crate::pages::goals::{GroupRow, groups};
    let mut h = Harness::new(me());
    h.down();
    let down = h.open();
    let goals = watch(&down, Some(f::chat::LABEL), None, WINDOW);
    let listed = watch(&down, Some(LABEL), None, WINDOW);
    h.live(listed, &[line(M, WALL)]);
    h.down();
    let mut goal = line(200, WALL);
    goal.requester = c::Requester::Task(M);
    h.live(goals, core::slice::from_ref(&goal));
    no_drawn_reads(&h.down());
    assert_eq!(h.reads().of(&goal, h.root.pages().goals().unwrap()), None, "a requester alone does not group");
    assert_eq!(
        groups(core::slice::from_ref(&goal), Some(h.reads()), h.root.pages().goals().unwrap()).as_slice(),
        [GroupRow::Goal(200)]
    );
    goal = in_milestone(goal, M);
    goal.requester = c::Requester::Task(999);
    h.live(goals, core::slice::from_ref(&goal));
    no_drawn_reads(&h.down());
    assert_eq!(h.reads().of(&goal, h.root.pages().goals().unwrap()), Some(M), "the part, not the requester, groups");
    assert_eq!(
        groups(core::slice::from_ref(&goal), Some(h.reads()), h.root.pages().goals().unwrap()).as_slice(),
        [GroupRow::Milestone(M)]
    );
    goal.parts = List::from_boxed(Box::new([c::RowPart::Unknown]));
    h.live(goals, core::slice::from_ref(&goal));
    no_drawn_reads(&h.down());
    assert_eq!(h.reads().of(&goal, h.root.pages().goals().unwrap()), Some(M));
}

fn marked_milestones(fact: bool) -> (Harness, Token, Token) {
    let mut identity = me();
    if fact {
        let templates = &mut identity.projects.get_mut(0).unwrap().templates;
        *templates = List::from_boxed(Box::new([templates.get(1).unwrap().clone()]));
    }
    let mut h = Harness::new(identity);
    let down = if fact {
        h.down()
    } else {
        h.down();
        h.open()
    };
    let live = watch(&down, Some(LABEL), None, if fact { 1 } else { WINDOW });
    let reads = marked_ended_reads(&down);
    assert_eq!(reads.len(), 1, "one opening ended page");
    (h, live, reads[0])
}
fn marked_ended_reads(down: &[Down]) -> Vec<Token> {
    let mut reads = Vec::new();
    for request in down {
        if let Down::Read { read, what: Readable::Jig(c::Read::Tasks { query, live, ended }) } = request
            && query.label == Some(LABEL)
            && query.parent.is_none()
        {
            assert_eq!(
                (query.phase, query.order, live, ended),
                (Some(c::PhaseClass::Ended), c::QueryOrder::Ended, &None, &None)
            );
            reads.push(*read);
        }
    }
    assert!(reads.len() <= 1, "one milestone ended read at a time");
    reads
}

#[test]
fn ended_milestones_and_their_fact_read_again_only_when_the_mark_grows() {
    for fact in [false, true] {
        let (mut h, live, first) = marked_milestones(fact);
        h.answer(first, page_at(&[], false, 5));
        assert!(marked_ended_reads(&h.down()).is_empty(), "the opening answer needs no follow-up");
        h.live_at(live, &[line(M, WALL)], 5);
        assert!(marked_ended_reads(&h.down()).is_empty(), "first delivery keeps the opening answer");
        let changed = line(M, Wall::from_nanos(WALL.as_nanos() + 1));
        h.live_at(live, core::slice::from_ref(&changed), 5);
        assert!(marked_ended_reads(&h.down()).is_empty(), "a live row change is not an ended change");
        h.live_at(live, &[changed], 6);
        let reads = marked_ended_reads(&h.down());
        assert_eq!(reads.len(), 1, "equal live rows can hide a task that started and ended");
        h.answer(reads[0], page_at(&[done(M + 1)], false, 6));
        assert!(marked_ended_reads(&h.down()).is_empty(), "one refresh suffices");
        if fact {
            h.live_at(live, &[], 6);
            assert!(h.root.has_goals(PROJECT), "the ended milestone fact is held");
        } else {
            assert!(h.reads().lines().iter().any(|line| line.chip.task == M + 1), "the ended row is held");
        }
        assert!(marked_ended_reads(&h.down()).is_empty(), "the held ended row keeps its mark");
    }
}

#[test]
fn ended_milestones_and_their_fact_coalesce_marks_while_reading() {
    for fact in [false, true] {
        for answered_mark in [5, 6, 7] {
            let (mut h, live, first) = marked_milestones(fact);
            h.live_at(live, &[], 6);
            assert!(marked_ended_reads(&h.down()).is_empty(), "the opening read is in flight");
            h.live_at(live, &[], 5);
            assert!(marked_ended_reads(&h.down()).is_empty(), "keep the greatest delivery mark");
            h.answer(first, page_at(&[], false, answered_mark));
            let reads = marked_ended_reads(&h.down());
            if answered_mark < 6 {
                assert_eq!(reads.len(), 1, "an older answer needs one follow-up");
                h.live_at(live, &[], 7);
                h.live_at(live, &[], 6);
                assert!(marked_ended_reads(&h.down()).is_empty(), "the follow-up stays in flight");
                h.answer(reads[0], page_at(&[], false, 6));
                let next = marked_ended_reads(&h.down());
                assert_eq!(next.len(), 1, "one follow-up for a mark grown during a refresh");
                h.answer(next[0], page_at(&[], false, 7));
                assert!(marked_ended_reads(&h.down()).is_empty(), "one follow-up per uncovered mark suffices");
            } else {
                assert!(reads.is_empty(), "the answer covers the greatest delivery mark");
            }
        }
    }
}

#[test]
fn a_project_with_only_a_milestone_has_goals() {
    let mut me = me();
    let templates = &mut me.projects.get_mut(0).unwrap().templates;
    *templates = List::from_boxed(Box::new([templates.get(1).unwrap().clone()]));
    let mut h = Harness::new(me);
    assert!(!h.root.has_goals(PROJECT));
    let opened = h.down();
    let fact = watch(&opened, Some(LABEL), None, 1);
    let first = ended_read(&opened, Some(LABEL), None);
    h.answer(first, page_at(&[], false, 5));
    h.down();
    h.live_at(fact, &[line(M, WALL)], 5);
    assert!(h.root.has_goals(PROJECT));
    assert!(h.down().is_empty(), "the first delivery keeps the ended answer");
    h.live_at(fact, &[], 6);
    assert!(!h.root.has_goals(PROJECT));
    let read = ended_read(&h.down(), Some(LABEL), None);
    h.answer(read, page_at(&[done(M)], false, 6));
    assert!(h.root.has_goals(PROJECT), "an ended milestone keeps the project fact");
}

#[test]
fn milestone_facts_take_each_pages_list_and_keep_it_until_their_own_delivery() {
    let mut identity = me();
    let templates = &mut identity.projects.get_mut(0).unwrap().templates;
    *templates = List::from_boxed(Box::new([templates.get(1).unwrap().clone()]));
    for address in [
        Address::Home { project: PROJECT },
        Address::Work { project: PROJECT, filter: WorkFilter::All },
        Address::Goals { project: PROJECT, filter: GoalsFilter::All },
        Address::Changes { project: PROJECT },
    ] {
        let mut h = Harness::at(identity.clone(), Some(address));
        let opened = h.down();
        let listed = watch(&opened, Some(LABEL), None, WINDOW);
        let read = ended_read(&opened, Some(LABEL), None);
        assert_eq!(
            opened
                .iter()
                .filter(|request| {
                    if let Down::Open { subject: Watchable::Jig(c::Subject::Tasks { query, .. }), .. } = request {
                        query.label == Some(LABEL)
                    } else {
                        false
                    }
                })
                .count(),
            1,
            "no narrower milestone twin"
        );
        assert_eq!(
            opened
                .iter()
                .filter(|request| {
                    if let Down::Read { what: Readable::Jig(c::Read::Tasks { query, .. }), .. } = request {
                        query.label == Some(LABEL)
                    } else {
                        false
                    }
                })
                .count(),
            1,
            "the fact opens no ended read"
        );
        assert!(!h.root.has_goals(PROJECT));
        h.answer(read, page(&[], false));
        assert!(h.down().is_empty(), "no twin on the following step");
        h.live(listed, &[line(M, WALL)]);
        assert!(h.root.has_goals(PROJECT), "the list's live row supplies the fact");
        h.down();
        h.live_at(listed, &[], 1);
        assert!(!h.root.has_goals(PROJECT));
        let read = ended_read(&h.down(), Some(LABEL), None);
        // An old ended milestone counts even when it is outside the drawn week.
        let mut ended = done(M);
        ended.ended_at = Some(Wall::EPOCH);
        h.answer(read, page_at(&[ended], false, 1));
        assert!(h.root.has_goals(PROJECT), "the list's ended page supplies the fact");
        h.down();
        h.went(Address::Settings { project: PROJECT });
        assert!(h.root.has_goals(PROJECT), "leaving a list cannot blink the entry off");
        let opened = h.down();
        let own = watch(&opened, Some(LABEL), None, 1);
        assert!(opened.contains(&Down::Close { watch: listed }));
        let read = ended_read(&opened, Some(LABEL), None);
        h.answer(read, page(&[], false));
        assert!(h.root.has_goals(PROJECT), "kept until the first own delivery");
        h.live(own, &[]);
        assert!(!h.root.has_goals(PROJECT));
    }
}

#[test]
fn milestone_words_count_state_strip_and_headline_use_the_first_matching_rule() {
    let mut milestone = line(M, WALL);
    let active = line(8, WALL);
    let mut waiting = line(7, WALL);
    phase(&mut waiting, c::ChipPhase::Waiting { dependencies: List::with_capacity(0), more: false, holds: false });
    let goals = [done(1), done(2), failed(5), closed(6), waiting.clone(), active.clone()];
    assert_eq!(milestones::count(&goals, None), Count { done: 2, total: 5 });
    let counts = totals(7, 1, 2);
    assert_eq!(milestones::count(&goals, Some(&counts)), Count { done: 7, total: 10 });
    assert_eq!(
        milestones::strip(&goals, &[7, 8]).as_slice(),
        [
            Segment { task: 1, mark: Mark::Done },
            Segment { task: 2, mark: Mark::Done },
            Segment { task: 5, mark: Mark::Failed },
            Segment { task: 7, mark: Mark::Needs },
            Segment { task: 8, mark: Mark::Needs }
        ]
    );
    assert_eq!(
        milestones::strip(&[active.clone(), waiting.clone()], &[]).as_slice(),
        [Segment { task: 7, mark: Mark::Waits }, Segment { task: 8, mark: Mark::Live }]
    );
    for (goals, need, expected) in [
        (&goals[..], true, State::Needs),
        (&goals[..], false, State::Running),
        (core::slice::from_ref(&waiting), false, State::Waits),
        (&[][..], false, State::Open),
    ] {
        assert_eq!(milestones::state(&milestone, goals, None, need), expected);
    }
    // The placed Stop fixture pins the same own-subtree pause as a goal.
    let c::ReadValue::Escalation(escalation) = f::acts::held(c::ActKind::Release) else { panic!("hold") };
    let mut hold = *escalation.task.hold.unwrap();
    hold.placed = Some(c::PlacedHold {
        number: 1,
        owner: c::HoldOwner::Party { person: f::PARTY },
        mode: c::HoldMode::Stop,
        scope: c::HoldScope::Subtree { task: M },
        reason: Box::default(),
        placed_at: WALL,
        revision: c::Revision { value: 1 },
        lifted: false,
        may_lift: true,
    });
    phase(&mut milestone, c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 });
    milestone.chip.details.details.hold = Some(hold);
    assert_eq!(milestones::state(&milestone, &goals, None, false), State::Paused);
    assert_eq!(milestones::state(&milestone, &goals, None, true), State::Needs);
    milestone.chip.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().lifted = true;
    assert_eq!(milestones::state(&milestone, &goals, None, false), State::Running);
    // Ended wins even when a need remains below it.
    milestone = done(M);
    assert_eq!(milestones::state(&milestone, &goals, None, true), State::Done);
    assert_eq!(milestones::state(&milestone, &goals, Some(&counts), true), State::Ended);
    assert_eq!(
        milestones::headline(&milestone, &goals, Some(&counts), true),
        Headline::Ended { count: Count { done: 7, total: 10 }, failed: 1, first: Some(5) }
    );
    let all = totals(8, 0, 0);
    assert_eq!(milestones::state(&milestone, &goals, Some(&all), false), State::Complete);
    milestone = closed(M);
    assert_eq!(milestones::state(&milestone, &goals, Some(&all), true), State::Closed);
}

#[test]
fn goal_membership_stands_through_busy_and_falls_back_only_before_decoding() {
    use crate::pages::goals::{GroupRow, groups, home_rows};
    let mut h = Harness::new(me());
    h.down();
    let opened = h.open();
    let goals = watch(&opened, Some(f::chat::LABEL), None, WINDOW);
    let listed = watch(&opened, Some(LABEL), None, WINDOW);
    let mut child = in_milestone(line(200, WALL), M);
    h.live(listed, &[row_part(line(M, WALL), &[child.clone()], 0)]);
    let unread = line(200, WALL);
    assert_eq!(h.reads().of(&unread, h.root.pages().goals().unwrap()), Some(M));
    h.live(goals, core::slice::from_ref(&child));
    h.live(listed, &[row_part(line(M, WALL), &[], 0)]);
    child.parts = List::from_boxed(Box::new([c::RowPart::Failed(c::PartFailure::Busy)]));
    h.live(goals, core::slice::from_ref(&child));
    let held = h.root.pages().goals().unwrap();
    assert_eq!(h.reads().of(&child, held), Some(M));
    assert_eq!(
        home_rows(core::slice::from_ref(&child), &[], Some(h.reads()), held, WALL).as_slice(),
        [GroupRow::Milestone(M)]
    );
    assert_eq!(groups(core::slice::from_ref(&child), Some(h.reads()), held).as_slice(), [GroupRow::Milestone(M)]);
    h.live(listed, &[row_part(line(M, WALL), &[unread], 0)]);
    child.parts = List::from_boxed(Box::new([temper_fixtures::client_documents::goal_part(None, None)]));
    h.live(goals, core::slice::from_ref(&child));
    child.parts = List::from_boxed(Box::new([c::RowPart::Unknown]));
    h.live(goals, core::slice::from_ref(&child));
    assert_eq!(
        h.reads().of(&child, h.root.pages().goals().unwrap()),
        None,
        "decoded absence beats the milestone's list"
    );
    no_drawn_reads(&h.down());
}

#[test]
fn ended_milestone_parts_stand_through_a_mark_refresh_and_leave_with_the_list() {
    let mut h = Harness::new(me());
    h.down();
    let opened = h.open();
    let listed = watch(&opened, Some(LABEL), None, WINDOW);
    let first = ended_read(&opened, Some(LABEL), None);
    let own = row_part(done(M), &[done(200)], 2);
    h.answer(first, page_at(core::slice::from_ref(&own), false, 5));
    h.live_at(listed, &[], 5);
    h.down();
    h.live_at(listed, &[], 6);
    let refresh = ended_read(&h.down(), Some(LABEL), None);
    let mut lost = own;
    lost.parts = List::from_boxed(Box::new([c::RowPart::Failed(c::PartFailure::Busy)]));
    h.answer(refresh, page_at(core::slice::from_ref(&lost), false, 6));
    let drawn = h.reads().milestone(M).unwrap();
    assert_eq!(drawn.totals.as_ref().unwrap().done(), 2);
    assert_eq!(drawn.goals.len(), 1);
    assert_eq!(drawn.headline(200), Some(b"Part report headline.".as_slice()));
    assert_eq!(drawn.more, 2);
    h.live_at(listed, &[], 7);
    let refresh = ended_read(&h.down(), Some(LABEL), None);
    h.answer(refresh, page_at(&[], false, 7));
    h.live(listed, core::slice::from_ref(&lost));
    assert!(h.reads().milestone(M).unwrap().totals.is_none(), "a returning row has no kept part");
    h.live(listed, &[row_part(line(M, WALL), &[done(200)], 0)]);
    h.went(Address::Settings { project: PROJECT });
    assert!(h.root.pages().milestones().is_none());
    let opened = h.open();
    let listed = watch(&opened, Some(LABEL), None, WINDOW);
    h.live(listed, &[lost]);
    assert!(h.reads().milestone(M).unwrap().totals.is_none(), "leaving the list drops its parts");
}

pub(super) fn opened_project() -> (crate::Domain, crate::Limits, Token, Token) {
    let mut h = Harness::new(me());
    h.down();
    let opened = h.open();
    let goals = watch(&opened, Some(f::chat::LABEL), None, WINDOW);
    let milestones = watch(&opened, Some(LABEL), None, WINDOW);
    (h.root, h.limits, goals, milestones)
}

fn own_reads(out: &Queue<Request>) -> (jig_client::Handle, jig_client::Handle, jig_client::Handle) {
    let mut document = None;
    let mut own_line = None;
    let mut totals = None;
    for request in out {
        if let Request::Client(jig_client::Event::Read { handle, read }) = request {
            match read {
                Readable::Jig(c::Read::Task { .. }) => document = Some(*handle),
                Readable::Jig(c::Read::Tasks { query, .. }) if query.parent.is_none() => own_line = Some(*handle),
                Readable::Family { record, .. } => {
                    if let Some(reading) = wire::decode_reading(&record.bytes, &w::CEILINGS)
                        && let w::Read::Milestone(_) = reading.read()
                    {
                        totals = Some(*handle);
                    }
                }
                Readable::Jig(_) => {}
            }
        }
    }
    (document.expect("document read"), own_line.expect("own line read"), totals.expect("totals read"))
}

#[test]
#[expect(clippy::too_many_lines, reason = "one sequence holds the milestone's independent reads through refreshes")]
fn a_milestone_page_keeps_its_document_totals_ended_rows_and_line_until_answers() {
    let mut h = Harness::new(me());
    h.down();
    let mut out = Queue::uncapped();
    let mut task_page = crate::pages::task::Task::new(f::chat::TASK, 90_000, &h.limits, &mut out);
    h.event(crate::Event::Client {
        event: jig_client::Event::Watch {
            handle: task_page.document,
            subject: Watchable::Jig(c::Subject::Task { task: f::chat::TASK }),
        },
    });
    let task_watch = h
        .down()
        .iter()
        .find_map(|request| match request {
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Task { task }) } if *task == f::chat::TASK => {
                Some(*watch)
            }
            Down::Save { .. }
            | Down::Erase { .. }
            | Down::Send { .. }
            | Down::Open { .. }
            | Down::Continue { .. }
            | Down::Close { .. }
            | Down::Read { .. } => None,
        })
        .unwrap();
    let mut task = f::chat::task();
    task.chip.executor =
        c::ChipExecutor::Procedure { connector: jig_client::shapes::RECURRING, kind: jig_client::shapes::GATHERING };
    task.chip.details.details.executor =
        c::Executor::Procedure { connector: jig_client::shapes::RECURRING, code: jig_client::shapes::GATHERING };
    f::chat::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
    let deliver_task = |h: &mut Harness, task: &c::Task| {
        h.event(crate::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched {
                    watch: task_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))),
                },
            },
        });
    };
    deliver_task(&mut h, &task);
    out.clear();
    task_page.told(skein_lib::Time::ZERO, h.root.client(), Told::Changed { handle: task_page.document }, &mut out);
    let (document, own_line, totals_read) = own_reads(&out);
    let mut live_handle = None;
    let mut ended_handle = None;
    for request in &out {
        if let Request::Client(jig_client::Event::Watch { handle, subject }) = request {
            live_handle = Some(*handle);
            h.event(crate::Event::Client {
                event: jig_client::Event::Watch { handle: *handle, subject: subject.clone() },
            });
        } else if let Request::Client(jig_client::Event::Read {
            handle,
            read: Readable::Jig(c::Read::Tasks { query, .. }),
        }) = request
            && query.parent == Some(f::chat::TASK)
        {
            ended_handle = Some(*handle);
        }
    }
    let live_handle = live_handle.unwrap();
    let ended_handle = ended_handle.unwrap();
    let live = watch(&h.down(), None, Some(f::chat::TASK), WINDOW);
    out.clear();
    let answer = |page: &mut crate::pages::task::Task, h: &Harness, handle, value, out: &mut Queue<Request>| {
        assert!(page.told(skein_lib::Time::ZERO, h.root.client(), Told::Page { handle, page: value }, out));
    };
    answer(
        &mut task_page,
        &h,
        document,
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(task.clone())))),
        &mut out,
    );
    let totals = w::MilestoneTotals::new(
        &w::CEILINGS,
        w::MilestoneTotalsParts {
            task: f::chat::TASK,
            done: 2,
            failed: 0,
            closed: 0,
            live: 4,
            ended: None,
            last_landing: None,
            spent: 0,
            reds: 0,
        },
    )
    .unwrap();
    let totals =
        w::MilestoneAnswer::new(&w::CEILINGS, w::MilestoneAnswerParts { result: w::MilestoneResult::Totals(totals) })
            .unwrap();
    answer(
        &mut task_page,
        &h,
        totals_read,
        Readout::Family { family: wire::FORGE, record: Some(wire::milestone_answer(&totals)) },
        &mut out,
    );
    let mut own = line(f::chat::TASK, WALL);
    own.chip = task.chip.clone();
    let own_page = c::TaskPage {
        ended_mark: 0,
        live: List::from_boxed(Box::new([own.clone()])),
        live_after: None,
        ended: c::EndedGroup::None,
    };
    answer(
        &mut task_page,
        &h,
        own_line,
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(own_page)))),
        &mut out,
    );
    let child = super::goals::with_part(200, super::goals::report_part(Some(b"Ended goal headline")));
    answer(&mut task_page, &h, ended_handle, page_at(&[child], false, 5), &mut out);
    h.live_at(live, &[], 5);
    task_page.told(skein_lib::Time::ZERO, h.root.client(), Told::Changed { handle: live_handle }, &mut out);
    out.clear();
    h.live_at(live, &[], 6);
    task_page.told(skein_lib::Time::ZERO, h.root.client(), Told::Changed { handle: live_handle }, &mut out);
    let mut refreshing = false;
    for request in &out {
        if let Request::Client(jig_client::Event::Read { handle, .. }) = request {
            refreshing |= *handle == ended_handle;
        }
    }
    assert!(refreshing);
    out.clear();
    answer(&mut task_page, &h, ended_handle, Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)), &mut out);
    let held = &task_page.milestone.as_ref().unwrap().milestone;
    assert_eq!(held.goals.len(), 1, "a first ended read without its value keeps the old page");
    assert_eq!(held.goals.get(0).unwrap().chip.task, 200);
    let child = super::goals::with_part(200, c::RowPart::Failed(c::PartFailure::Busy));
    answer(&mut task_page, &h, ended_handle, page_at(&[child], false, 6), &mut out);
    assert_eq!(task_page.milestone.as_ref().unwrap().headline(200).as_deref(), Some(b"Ended goal headline".as_slice()));
    task.changed = Wall::from_nanos(task.changed.as_nanos().checked_add(1).unwrap());
    deliver_task(&mut h, &task);
    out.clear();
    task_page.told(skein_lib::Time::ZERO, h.root.client(), Told::Changed { handle: task_page.document }, &mut out);
    let (document, own_line, totals_read) = own_reads(&out);
    assert_eq!(
        task_page.milestone.as_ref().unwrap().line.as_ref().unwrap(),
        &own,
        "starting another search keeps the last line"
    );
    out.clear();
    for read in [document, own_line, totals_read] {
        answer(&mut task_page, &h, read, Readout::Limit, &mut out);
    }
    let page = task_page.milestone.as_ref().unwrap();
    assert_eq!(
        page.milestone.document.as_ref().unwrap().changed.as_nanos().checked_add(1),
        Some(task.changed.as_nanos())
    );
    assert!(page.milestone.facts.is_some());
    assert_eq!(page.milestone.totals.as_ref().unwrap().done(), 2);
    assert_eq!(page.line.as_ref().unwrap(), &own);
}

#[test]
fn undrawn_milestones_keep_their_parts_only_while_listed() {
    let mut h = Harness::new(me());
    h.down();
    let opened = h.open();
    let listed = watch(&opened, Some(LABEL), None, WINDOW);
    h.live(listed, &[row_part(line(M, Wall::EPOCH), &[done(200)], 0)]);
    let many: Vec<_> = (0..17).map(|i| line(M + i, Wall::from_nanos(i))).collect();
    h.live(listed, &many);
    assert!(h.reads().milestone(M).is_none(), "the seventeenth row is not drawn");
    h.live(listed, &many[..16]);
    assert_eq!(h.reads().milestone(M).unwrap().headline(200), Some(b"Part report headline.".as_slice()));
    h.live(listed, &many[1..16]);
    h.live(listed, &many[..16]);
    assert!(h.reads().milestone(M).unwrap().totals.is_none(), "leaving the list drops even undrawn parts");
}
