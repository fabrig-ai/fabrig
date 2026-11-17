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
        milestones::{self, Count, Headline, Mark, Milestone, Milestones, Segment, State},
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
        w::MilestoneTotalsParts { task: M, done, failed, live, closed: 50, ended: None, last_landing: None, spent: 0 },
    )
    .unwrap()
}
fn total_answer(totals: w::MilestoneTotals) -> Readout {
    let answer =
        w::MilestoneAnswer::new(&w::CEILINGS, w::MilestoneAnswerParts { result: w::MilestoneResult::Totals(totals) })
            .unwrap();
    Readout::Family { family: wire::FORGE, record: Some(wire::milestone_answer(&answer)) }
}
fn page(rows: &[c::TaskLine], more: bool) -> Readout {
    Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(c::TaskPage {
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
        let limits = limits();
        let mut root = crate::unlinked(&limits, f::SEED);
        let watch = f::start(&mut root, List::with_capacity(0));
        let mut h = Self { root, limits, out: Queue::uncapped() };
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
        self.event(crate::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
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
fn totals_read(down: &[Down]) -> Token {
    for request in down {
        if let Down::Read { read, what: Readable::Family { record, .. } } = request
            && let Some(reading) = wire::decode_reading(&record.bytes, &w::CEILINGS)
            && let w::Read::Milestone(_) = reading.read()
        {
            return *read;
        }
    }
    panic!("totals read")
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
    let children = watch(&h.down(), None, Some(M), WINDOW);
    let mut g3 = line(3, Wall::from_nanos(11));
    g3.requester = c::Requester::Task(M);
    let mut g4 = done(4);
    g4.last_activity = Wall::from_nanos(8);
    g4.requester = c::Requester::Task(M);
    h.live(children, core::slice::from_ref(&g3));
    // The shared read can know a goal before the milestone's ended read answers.
    let g1 = line(1, Wall::from_nanos(10));
    let g2 = done(2);
    let live = [g1.clone(), g3.clone()];
    let ended = [g2.clone(), g4.clone()];
    assert_eq!(home_rows(&live, &ended, Some(h.reads()), WALL).as_slice(), [Milestone(M), Goal(1), Goal(2)]);
    assert_eq!(groups(&[g1.clone(), g4, g2.clone(), g3], Some(h.reads())).as_slice(), [Goal(1), Milestone(M), Goal(2)]);
    // Equal activity takes the lower task, even when its kind differs.
    let tie = line(1, Wall::from_nanos(11));
    assert_eq!(home_rows(&[tie], &[], Some(h.reads()), WALL).as_slice(), [Goal(1), Milestone(M)]);
    let live: Vec<_> = (1..=6).map(|task| line(task, Wall::from_nanos(task))).collect();
    let mut older = done(8);
    older.ended_at = Some(Wall::from_nanos(1));
    assert_eq!(home_rows(&live, core::slice::from_ref(&g2), None, WALL).len(), 6);
    assert_eq!(home_rows(&live[..2], &[older, done(7)], None, WALL).as_slice(), [Goal(2), Goal(1), Goal(7), Goal(8)]);
    assert_eq!(
        home_rows(&live[..2], core::slice::from_ref(&g1), None, WALL).as_slice(),
        [Goal(2), Goal(1)],
        "a goal appears once"
    );
    let children: Vec<_> = (10..18)
        .map(|task| {
            let mut goal = line(task, Wall::from_nanos(task));
            goal.requester = c::Requester::Task(M);
            goal
        })
        .collect();
    let mut live = children;
    live.push(g1);
    let ended: Vec<_> = (20..26).map(done).collect();
    assert_eq!(
        home_rows(&live, &ended, Some(h.reads()), WALL).as_slice(),
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
    let mut recent_goal = done(2);
    recent_goal.requester = c::Requester::Task(100);
    let mut old_goal = done(3);
    old_goal.requester = c::Requester::Task(101);
    assert_eq!(
        home_rows(&[unknown], &[recent_goal, old_goal], Some(h.reads()), WALL).as_slice(),
        [Goal(1), Milestone(100)]
    );
    let many: Vec<_> = (200..217).map(|task| line(task, WALL)).collect();
    h.live(listed, &many);
    let mut undrawn = line(4, WALL);
    undrawn.requester = c::Requester::Task(216);
    assert!(h.reads().milestone(216).is_none());
    let rows = home_rows(&[undrawn], &[], Some(h.reads()), WALL);
    assert!(rows.as_slice().contains(&Goal(4)));
    assert_eq!(rows.len(), 17, "live rows are counted after grouping");
}

#[test]
fn milestones_read_the_query_and_keep_it_across_the_four_pages() {
    let root = crate::unlinked(&limits(), f::SEED);
    let mut next = 31;
    let mut reads = Milestones::new(&mut next);
    let mut out = Queue::uncapped();
    reads.configure(Some(&goals_me()), PROJECT, root.client(), &limits(), WALL, &mut next, &mut out);
    assert!(out.is_empty(), "no template 8, no reads");
    reads.configure(Some(&me()), PROJECT, root.client(), &limits(), WALL, &mut next, &mut out);
    let query = c::TaskQuery {
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
    assert_eq!(milestones::query(&me(), PROJECT), Some(query));
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Watch {
            handle: reads.live,
            subject: Watchable::Jig(c::Subject::Tasks { query, window: WINDOW })
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(jig_client::Event::Read {
            handle: reads.ended,
            read: Readable::Jig(c::Read::Tasks {
                query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query },
                live: None,
                ended: None
            })
        }))
    );
    assert!(out.is_empty());
    reads.configure(Some(&me()), PROJECT, root.client(), &limits(), WALL, &mut next, &mut out);
    assert!(out.is_empty());
    reads.leave(&mut out);
    for handle in [reads.live, reads.ended] {
        assert_eq!(out.pop(), Some(Request::Client(jig_client::Event::Unwatch { handle })));
    }
    let mut h = Harness::new(me());
    h.down();
    let opened = h.open();
    let live = watch(&opened, Some(LABEL), None, WINDOW);
    h.live(live, &[line(M, WALL)]);
    let drawn_watch = watch(&h.down(), None, Some(M), WINDOW);
    h.live(drawn_watch, &[]);
    h.down();
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
    assert!(closed.contains(&drawn_watch), "closed {closed:?}, drawn {drawn_watch:?}");
}

#[test]
fn drawn_reads_refresh_documents_totals_and_four_pages_of_merged_goals() {
    let mut h = Harness::new(me());
    h.down();
    let down = h.open();
    let listed = watch(&down, Some(LABEL), None, WINDOW);
    let mut milestone = line(M, WALL);
    h.live(listed, &[milestone.clone()]);
    let down = h.down();
    let tasks = task_reads(&down);
    assert_eq!(tasks.len(), 1, "{down:?}; milestones {:?}", h.reads());
    assert_eq!(tasks[0].1, M);
    let goal_watch = watch(&down, None, Some(M), WINDOW);
    let first = ended_read(&down, None, Some(M));
    let total = totals_read(&down);
    let mut document = f::chat::task();
    document.chip.task = M;
    h.answer(tasks[0].0, Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(document)))));
    h.answer(total, total_answer(totals(7, 0, 1)));
    h.down();
    let read = h.reads().milestone(M).unwrap();
    assert!(read.facts.is_some());
    assert_eq!(read.totals.as_ref().unwrap().done(), 7);
    let live = line(202, WALL);
    h.live(goal_watch, core::slice::from_ref(&live));
    h.down();
    // A live update during an ended read queues one fresh first page.
    h.answer(first, page(&[done(202)], true));
    let mut next = ended_read(&h.down(), None, Some(M));
    for index in 0..4 {
        h.answer(next, page(&[done(201 + index), done(202)], true));
        let down = h.down();
        if index < 3 {
            next = ended_read(&down, None, Some(M));
        } else {
            for request in &down {
                if let Down::Read { what: Readable::Jig(c::Read::Tasks { query, .. }), .. } = request {
                    assert_ne!(query.parent, Some(M));
                }
            }
        }
    }
    let read = h.reads().milestone(M).unwrap();
    assert!(read.more);
    assert_eq!(read.goals.as_slice().iter().map(|g| g.chip.task).collect::<Vec<_>>(), [201, 202, 203, 204]);
    assert_eq!(read.goals.get(1).unwrap(), &live, "live wins a duplicate number");
    h.live(listed, &[milestone.clone()]);
    assert!(task_reads(&h.down()).is_empty(), "same changed wall keeps document and totals");
    milestone.changed = Wall::from_nanos(3);
    h.live(listed, &[milestone.clone()]);
    let down = h.down();
    assert_eq!(task_reads(&down).len(), 1);
    let total = totals_read(&down);
    h.answer(total, Readout::Limit);
    h.down();
    assert!(h.reads().milestone(M).unwrap().totals.is_none(), "a failed totals read holds none");
    // A standalone read makes the end transition even without a moved changed wall.
    let root = crate::unlinked(&limits(), f::SEED);
    let mut next = 70;
    let mut read = Milestone::new(PROJECT, M, &mut next);
    let mut out = Queue::uncapped();
    read.configure(&milestone, root.client(), &limits(), &mut out);
    let mut handles = Vec::new();
    while let Some(request) = out.pop() {
        if let Request::Client(jig_client::Event::Read { handle, .. }) = request {
            handles.push(handle);
        }
    }
    for handle in handles {
        assert!(read.told(&Told::Page { handle, page: Readout::Limit }, &limits(), &mut out));
    }
    let mut ended = done(M);
    ended.changed = milestone.changed;
    read.configure(&ended, root.client(), &limits(), &mut out);
    let mut rereads = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Client(jig_client::Event::Read { .. }) = request {
            rereads += 1;
        }
    }
    assert_eq!(rereads, 3, "ending rereads document, totals and ended goals once");
    read.configure(&ended, root.client(), &limits(), &mut out);
    assert!(out.is_empty());
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
fn requesters_are_read_once_and_the_oldest_of_sixteen_is_dropped() {
    let mut h = Harness::new(me());
    h.down();
    let down = h.open();
    let goals = watch(&down, Some(f::chat::LABEL), None, WINDOW);
    let listed = watch(&down, Some(LABEL), None, WINDOW);
    h.live(listed, &[line(M, WALL)]);
    h.down();
    let mut goal = line(200, WALL);
    goal.requester = c::Requester::Task(M);
    h.live(goals, &[goal.clone()]);
    assert_eq!(task_reads(&h.down()), []);
    assert_eq!(h.reads().of(&goal), Some(M));
    goal.requester = c::Requester::Task(300);
    h.live(goals, &[goal.clone()]);
    let reads = task_reads(&h.down());
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].1, 300);
    assert_eq!(h.reads().of(&goal), None);
    h.live(goals, &[goal.clone()]);
    assert!(task_reads(&h.down()).is_empty(), "in flight is kept");
    let mut task = f::chat::task();
    task.chip.task = 300;
    task.chip.details.details.executor =
        c::Executor::Procedure { connector: jig_client::shapes::RECURRING, code: jig_client::shapes::GATHERING };
    h.answer(reads[0].0, Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(task)))));
    h.down();
    assert_eq!(h.reads().of(&goal), Some(300));
    let mut rows: Vec<_> = (0..17)
        .map(|i| {
            let mut line = line(200 + i, WALL);
            line.requester = c::Requester::Task(300 + i);
            line
        })
        .collect();
    h.live(goals, &rows);
    let reads = task_reads(&h.down());
    assert_eq!(reads.len(), 16);
    assert_eq!(h.reads().of(&goal), None, "oldest requester was evicted");
    h.live(goals, &rows);
    assert!(task_reads(&h.down()).is_empty(), "unchanged rows do not loop on evicted requesters");
    rows[0].changed = Wall::from_nanos(7);
    h.live(goals, &rows);
    assert_eq!(task_reads(&h.down())[0].1, 300);
    goal.requester = c::Requester::Person(300);
    assert_eq!(h.reads().of(&goal), None);
    rows[0].requester = c::Requester::Task(400);
    h.live(goals, &rows);
    let reads = task_reads(&h.down());
    assert_eq!(reads.len(), 1);
    let mut chat = f::chat::task();
    chat.chip.task = 400;
    h.answer(reads[0].0, Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(chat)))));
    h.down();
    assert_eq!(h.reads().of(&rows[0]), None, "a held chat is not a milestone");
    h.live(goals, &rows);
    assert!(task_reads(&h.down()).is_empty(), "a held chat is not read again");
}

#[test]
fn a_project_with_only_a_milestone_has_goals() {
    let mut me = me();
    let templates = &mut me.projects.get_mut(0).unwrap().templates;
    *templates = List::from_boxed(Box::new([templates.get(1).unwrap().clone()]));
    let mut h = Harness::new(me);
    assert!(!h.root.has_goals(PROJECT));
    let fact = watch(&h.down(), Some(LABEL), None, 1);
    h.live(fact, &[line(M, WALL)]);
    assert!(h.root.has_goals(PROJECT));
    h.down();
    h.live(fact, &[]);
    assert!(!h.root.has_goals(PROJECT));
    let read = ended_read(&h.down(), Some(LABEL), None);
    h.answer(read, page(&[done(M)], false));
    assert!(h.root.has_goals(PROJECT), "an ended milestone keeps the project fact");
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
