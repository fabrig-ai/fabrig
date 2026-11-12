use super::{chats, fixtures};
use crate::{
    frame::Bindings,
    look,
    pages::goals,
    words::{Times, Words},
};
use jig_client::contract as c;
use skein_lib::{List, Queue, Wall};
use skein_ui::{Builder, Role, Style, Tree};
use temper_client_domain::{
    Address, GoalsFilter,
    pages::{goals::State, report::Report},
};

// The mock's row has six landed tasks out of nine (client/pages/README.md).
const DONE: u32 = 6;
const TOTAL: u32 = 9;
const EMPTY: u32 = 0;
const ONE: u32 = 1;
// One link, top, title, metadata, state and headline in the browser contract.
const ROW_NODES: usize = 6;
const TOP_NODES: u32 = 4;
const META_NODES: u32 = 2;
// Seven states, with the two needs words checked separately.
const STATE_CASES: usize = 8;
// The template fixture's bounds and identities, independent of its name.
const TEXT_BYTES: u32 = 64;
const CHARTER: u32 = 1;
const LABEL: u32 = 1;
const DEPTH: u32 = 1;
const TASK_WAY: u32 = 2;
const WHOLE_WAY: u32 = 4;
const UNTRACKED_WAY: u32 = 7;
const OTHER_WAY: u32 = 9;
// Three such names exceed the fixture words' joined-name bound.
const LONG_WAY: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZ";
// Nanoseconds and epoch days for Thursday 22 October 2026 and its Monday.
const MINUTE: u64 = 60_000_000_000;
const HOUR: u64 = 3_600_000_000_000;
const DAY: u64 = 86_400_000_000_000;
const THURSDAY: u64 = 20_748 * DAY;
const MONDAY: u64 = 20_745 * DAY;
const FRIDAY: u64 = 20_749 * DAY;
const NOON: u64 = THURSDAY + 12 * HOUR;
const TODAY_END: u64 = THURSDAY + HOUR + 12 * MINUTE;
const MONDAY_END: u64 = MONDAY + 18 * HOUR + 40 * MINUTE;

fn line() -> c::TaskLine {
    let mut line = chats::line();
    line.delegates_done = DONE;
    line.delegates_total = TOTAL;
    line
}
fn tree(line: &c::TaskLine, state: State, report: Option<&Report>, words: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = Bindings::new(fixtures::tree_limits().bindings);
    goals::row(&mut b, words, &mut bindings, &mut Times::new(fixtures::NOW), line, state, report);
    (b.finish(), bindings)
}
fn states() -> [(State, &'static [u8], Style, bool); STATE_CASES] {
    [
        (State::Needs { funds: true }, b"needs funds", look::GOALS_STATE_NEEDS, false),
        (State::Needs { funds: false }, b"needs you", look::GOALS_STATE_NEEDS, false),
        (State::Paused, b"paused by you", look::GOALS_STATE_PAUSED, false),
        (State::Planning, b"planning", look::GOALS_STATE_PLANNING, false),
        (State::Running, b"running", look::GOALS_STATE_RUNNING, false),
        (State::Landed, b"landed", look::GOALS_STATE_LANDED, true),
        (State::Closed, b"closed", look::GOALS_STATE_CLOSED, true),
        (State::Failed, b"failed", look::GOALS_STATE_CLOSED, true),
    ]
}
fn styled_text(tree: &Tree, style: Style) -> Box<[u8]> {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&style) {
            return fixtures::node_text(tree, index);
        }
    }
    panic!("styled node")
}
#[test]
fn a_row_is_one_link_to_its_task() {
    let (tree, bindings) = tree(&line(), State::Running, None, &fixtures::browser_words());
    let nodes = tree.nodes();
    let mut links = EMPTY;
    for node in nodes {
        if node.role == Role::Link {
            links = links.checked_add(ONE).unwrap();
        }
    }
    assert_eq!(links, ONE);
    let link = &nodes[0];
    assert_eq!(link.role, Role::Link);
    assert_eq!(usize::try_from(link.size).unwrap(), nodes.len());
    assert_eq!(link.styles.as_slice(), [look::GOALS_ROW]);
    let binding = link.binding.expect("row binding");
    assert_eq!(link.link, Some(binding));
    assert_eq!(
        bindings.decode(binding.token, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: Address::Task { task: line().chip.task }
        }))
    );
    // Text roles map to spans in the browser: top(title, count(state)), then line.
    assert_eq!(nodes.len(), ROW_NODES);
    assert_eq!(nodes[1].role, Role::Text);
    assert_eq!(nodes[1].styles.as_slice(), [look::GOALS_TOP]);
    assert_eq!(nodes[1].size, TOP_NODES);
    assert_eq!(nodes[2].role, Role::Text);
    assert_eq!(fixtures::node_text(&tree, 2).as_ref(), b"Hello");
    assert_eq!(nodes[3].role, Role::Text);
    assert_eq!(nodes[3].size, META_NODES);
    assert_eq!(nodes[5].role, Role::Text);
    assert_eq!(nodes[5].styles.as_slice(), [look::GOALS_LINE]);
}
#[test]
fn each_state_says_its_word_and_carries_its_token() {
    for (state, word, token, _) in states() {
        let (tree, _) = tree(&line(), state, None, &fixtures::browser_words());
        assert_eq!(styled_text(&tree, token).as_ref(), word, "{state:?}");
    }
    // 22 October 2026, and the Monday in the same week.
    let now = Wall::from_nanos(NOON);
    for (at, expected) in [(TODAY_END, b"landed 01:12".as_slice()), (MONDAY_END, b"landed Mon 18:40".as_slice())] {
        let mut line = line();
        line.ended_at = Some(Wall::from_nanos(at));
        let mut b = Builder::new(&fixtures::tree_limits());
        let mut times = Times::new(now);
        goals::row(
            &mut b,
            &fixtures::browser_words(),
            &mut Bindings::new(fixtures::tree_limits().bindings),
            &mut times,
            &line,
            State::Landed,
            None,
        );
        assert_eq!(styled_text(&b.finish(), look::GOALS_STATE_LANDED).as_ref(), expected);
        assert_eq!(times.next, Some(Wall::from_nanos(FRIDAY)));
    }
}
#[test]
fn its_count_is_left_out_with_no_delegates() {
    for (total, expected) in [(EMPTY, b"Hellorunning".as_slice()), (TOTAL, "Hello6 / 9 · running".as_bytes())] {
        let mut line = line();
        line.delegates_total = total;
        let (tree, _) = tree(&line, State::Running, None, &fixtures::browser_words());
        assert_eq!(styled_text(&tree, look::GOALS_TOP).as_ref(), expected);
    }
}
#[test]
fn its_headline_is_the_reports_or_its_states() {
    let report = Report {
        headline: Box::from(*b"The store is split in two."),
        paragraphs: List::uncapped(),
        left: None,
        risk: None,
        decided: List::uncapped(),
        at: None,
    };
    for (state, fallback) in [
        (State::Planning, b"Planning from the draft.".as_slice()),
        (State::Running, b"6 of 9 tasks done.".as_slice()),
        (State::Paused, b"6 of 9 tasks done.".as_slice()),
        (State::Needs { funds: true }, b"6 of 9 tasks done.".as_slice()),
        (State::Needs { funds: false }, b"6 of 9 tasks done.".as_slice()),
        (State::Landed, b"Landed.".as_slice()),
        (State::Closed, b"Closed by you.".as_slice()),
        (State::Failed, b"Failed.".as_slice()),
    ] {
        for (given, expected) in [(None, fallback), (Some(&report), report.headline.as_ref())] {
            let (tree, _) = tree(&line(), state, given, &fixtures::browser_words());
            assert_eq!(styled_text(&tree, look::GOALS_LINE).as_ref(), expected, "{state:?}");
        }
    }
    for state in [State::Needs { funds: true }, State::Needs { funds: false }] {
        let mut line = line();
        line.delegates_total = EMPTY;
        let (tree, _) = tree(&line, state, None, &fixtures::browser_words());
        assert_eq!(styled_text(&tree, look::GOALS_LINE).as_ref(), b"Planning from the draft.");
    }
}
#[test]
fn an_ended_row_is_marked() {
    for (state, _, _, ended) in states() {
        let (tree, _) = tree(&line(), state, None, &fixtures::browser_words());
        assert_eq!(tree.nodes()[0].styles.as_slice().contains(&look::GOALS_ROW_ENDED), ended, "{state:?}");
    }
}
#[test]
fn a_row_reads_in_the_terminal() {
    for (total, name, headline) in [
        (EMPTY, "Hello · needs funds", b"Planning from the draft.".as_slice()),
        (TOTAL, "Hello · 6 / 9 · needs funds", b"6 of 9 tasks done.".as_slice()),
    ] {
        let mut line = line();
        line.delegates_total = total;
        let (mut tree, bindings) = tree(&line, State::Needs { funds: true }, None, &fixtures::words());
        let limits = fixtures::tree_limits();
        skein_ui::diff(&Tree::new(limits.nodes), &mut tree, &limits, &mut 0, &mut Queue::with_capacity(limits.patches));
        let name = name.as_bytes();
        let control = fixtures::find(&tree, Role::Link, name).expect("one goal link");
        let (node, act) = fixtures::terminal_command(&tree, &limits, control, name, headline);
        let mut bound = EMPTY;
        let mut binding = None;
        for item in tree.nodes() {
            if item.binding.is_some() {
                bound = bound.checked_add(ONE).unwrap();
            }
            if item.id == node {
                binding = item.binding;
            }
        }
        assert_eq!(bound, ONE);
        let binding = binding.expect("goal link binding");
        assert_eq!(
            bindings.decode(binding.token, act),
            Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                address: Address::Task { task: line.chip.task }
            }))
        );
    }
}
pub(super) fn template(number: u32, name: &[u8], tracked: bool) -> c::Template {
    c::Template {
        resources: List::uncapped(),
        number,
        name: c::DocumentText::complete(c::Text::new(name.into(), TEXT_BYTES).unwrap()),
        executor: c::TemplateExecutor::Charter(CHARTER),
        contract: c::TaskContractDocument::Report { words: TEXT_BYTES },
        shallower: DEPTH,
        budget: c::TemplateBudget::AsAllowed { cap: None },
        deadline: c::TemplateDeadline::None,
        tracked: if tracked { Some(LABEL) } else { None },
        flags: line().flags,
        label: LABEL,
        grace: None,
        llm: None,
    }
}
#[test]
fn ways_name_their_templates_in_number_order() {
    let root = fixtures::root_in_project(Some(fixtures::PROJECT));
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(EMPTY).unwrap();
    project.templates = List::uncapped();
    for template in [
        template(OTHER_WAY, b"other way", true),
        template(WHOLE_WAY, b"goal for a draft", true),
        template(UNTRACKED_WAY, b"untracked", false),
        template(TASK_WAY, b"goal", true),
    ] {
        project.templates.push(template).unwrap();
    }
    for words in [fixtures::words(), fixtures::browser_words()] {
        let ways = goals::ways(&me, fixtures::PROJECT, &words);
        let mut actual = List::uncapped();
        for way in &ways {
            actual.push((way.name.as_ref(), way.line.as_deref())).unwrap();
        }
        assert_eq!(actual.as_slice(), [
            (b"Task by task".as_slice(), Some(b"The same plan; each task lands on its own once checked.".as_slice())),
            (b"Plan, tasks, land whole".as_slice(), Some(b"A plan from the draft, its tasks on a goal branch, landed as one change once all are checked.".as_slice())),
            (b"other way".as_slice(), None),
        ]);
        assert!(goals::ways(&me, u32::MAX, &words).is_empty());
    }
}

// The state table has five live rows (two needs) and three ended rows.
const PAGE_ROWS: u32 = 8;
const NEEDS_TASK: u64 = 101;
const FUNDS_TASK: u64 = 102;
const PAUSED_TASK: u64 = 103;
const PLANNING_TASK: u64 = 104;
const RUNNING_TASK: u64 = 105;
const LANDED_TASK: u64 = 106;
const CLOSED_TASK: u64 = 107;
const FAILED_TASK: u64 = 108;
const CURSOR_BYTES: u32 = 32;

struct Page {
    root: temper_client_domain::Domain,
    out: Queue<temper_client_domain::Request>,
    watch: skein_lib::Token,
    me: skein_lib::Token,
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
        let mut page = Self { root, out: Queue::uncapped(), watch: me, me, limits };
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
    fn populated() -> Self {
        let mut page = Self::new();
        page.live(&live_lines(), false);
        page.ended(
            &[
                page_line(LANDED_TASK, State::Landed),
                page_line(CLOSED_TASK, State::Closed),
                page_line(FAILED_TASK, State::Failed),
            ],
            false,
        );
        let read = page.read(false);
        page.answer(
            read,
            c::ReadValue::Inbox(Box::new(c::InboxPage {
                rows: List::uncapped(),
                before: None,
                after: None,
                cut: false,
                selection: Some(List::from_boxed(Box::new([c::InboxSelection {
                    cursor: c::InboxCursor::Number { number: ONE.into() },
                    goals: List::from_boxed(Box::new([NEEDS_TASK, FUNDS_TASK])),
                }]))),
            })),
        );
        page
    }
    fn tree(&self, words: &Words) -> (Tree, Bindings) {
        let mut b = Builder::new(&fixtures::tree_limits());
        let mut bindings = Bindings::new(fixtures::tree_limits().bindings);
        goals::main(&mut b, &self.root, words, &mut bindings, fixtures::PROJECT, &mut Times::new(fixtures::NOW));
        (b.finish(), bindings)
    }
    fn filter(&mut self, filter: GoalsFilter) {
        self.event(temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(Address::Goals { project: fixtures::PROJECT, filter }),
        });
    }
}
// The needs pair, paused, planning and running rows precede the ended page.
const LIVE_ROWS: usize = 5;
fn live_lines() -> [c::TaskLine; LIVE_ROWS] {
    [
        page_line(NEEDS_TASK, State::Needs { funds: false }),
        page_line(FUNDS_TASK, State::Needs { funds: true }),
        page_line(PAUSED_TASK, State::Paused),
        page_line(PLANNING_TASK, State::Planning),
        page_line(RUNNING_TASK, State::Running),
    ]
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
fn row_tasks(tree: &Tree, bindings: &Bindings) -> Box<[u64]> {
    let mut tasks = List::uncapped();
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&look::GOALS_ROW) {
            let action = bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press);
            if let Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                address: Address::Task { task, .. },
            })) = action
            {
                tasks.push(task).unwrap();
            } else {
                panic!("row action: {action:?}");
            }
        }
    }
    tasks.into_boxed()
}
fn role_nodes(tree: &Tree, role: Role) -> Box<[usize]> {
    let mut nodes = List::uncapped();
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == role {
            nodes.push(index).unwrap();
        }
    }
    nodes.into_boxed()
}
#[expect(clippy::manual_find, reason = "the crate disallows iterator search closures")]
fn styled_node(tree: &Tree, style: Style) -> Option<&skein_ui::Node> {
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&style) {
            return Some(node);
        }
    }
    None
}
fn style_order(tree: &Tree, wanted: &[Style]) -> Box<[Style]> {
    let mut styles = List::uncapped();
    for node in tree.nodes() {
        for style in &node.styles {
            if wanted.contains(style) {
                styles.push(*style).unwrap();
            }
        }
    }
    styles.into_boxed()
}
#[test]
fn the_headline_counts_every_row_in_every_form() {
    for words in [fixtures::browser_words(), fixtures::words()] {
        let mut page = Page::new();
        for (live, more, expected) in [
            ([].as_slice(), false, "No goals yet."),
            ([page_line(RUNNING_TASK, State::Running)].as_slice(), false, "1 goal, 1 under way."),
            ([page_line(RUNNING_TASK, State::Running)].as_slice(), true, "More than 1 goal, 1 under way."),
        ] {
            page.live(live, more);
            page.ended(&[], false);
            let (tree, _) = page.tree(&words);
            assert_eq!(styled_text(&tree, look::GOALS_HEADLINE).as_ref(), expected.as_bytes());
        }
        let mut page = Page::populated();
        for (more, expected) in [(false, "8 goals, 5 under way."), (true, "More than 8 goals, 5 under way.")] {
            page.live(&live_lines(), more);
            for filter in [GoalsFilter::All, GoalsFilter::Needs, GoalsFilter::Live, GoalsFilter::Ended] {
                page.filter(filter);
                assert_eq!(styled_text(&page.tree(&words).0, look::GOALS_HEADLINE).as_ref(), expected.as_bytes());
            }
        }
    }
}
#[test]
fn the_lede_names_the_projects_ways() {
    let mut page = Page::new();
    let mut me = page.root.pages().me(page.root.client()).unwrap().clone();
    for (templates, expected) in [
        ([].as_slice(), "A goal turns a draft into landed work."),
        (
            [template(TASK_WAY, b"goal", true)].as_slice(),
            "A goal turns a draft into landed work, its own way: task by task.",
        ),
        (
            [
                template(OTHER_WAY, "ÉTAPES".as_bytes(), true),
                template(WHOLE_WAY, b"goal for a draft", true),
                template(TASK_WAY, b"goal", true),
                template(UNTRACKED_WAY, b"unused", false),
            ]
            .as_slice(),
            "A goal turns a draft into landed work, its own way: task by task, plan, tasks, land whole, étapes.",
        ),
        (
            [
                template(TASK_WAY, LONG_WAY, true),
                template(WHOLE_WAY, LONG_WAY, true),
                template(OTHER_WAY, LONG_WAY, true),
            ]
            .as_slice(),
            "A goal turns a draft into landed work, its own way: abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz, abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz, abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz.",
        ),
    ] {
        me.projects.get_mut(EMPTY).unwrap().templates = List::from_boxed(templates.into());
        fixtures::deliver_me(&mut page.root, page.me, me.clone());
        for words in [fixtures::words(), fixtures::browser_words()] {
            let (tree, _) = page.tree(&words);
            assert_eq!(styled_text(&tree, look::WORK_KICKER).as_ref(), "Shop · Work".as_bytes());
            assert_eq!(styled_text(&tree, look::GOALS_LEDE).as_ref(), expected.as_bytes());
            let mut print = skein_ui_text::Print::new(&fixtures::tree_limits()).unwrap();
            skein_ui_text::render(&tree, b"temper", None, &mut print);
            assert!(skein_lib::bytes::find(print.text.as_slice(), expected.as_bytes()).is_some());
        }
    }
}
#[test]
fn the_intro_waits_for_me_and_the_live_watch() {
    let mut page = Page::new();
    for root in [Box::new(temper_client_domain::unlinked(&page.limits, fixtures::SEED)), Box::new(Page::new().root)] {
        page.root = *root;
        for words in [fixtures::words(), fixtures::browser_words()] {
            let (tree, _) = page.tree(&words);
            assert!(styled_node(&tree, look::WORK_KICKER).is_some());
            assert!(styled_node(&tree, look::GOALS_LEDE).is_some());
            assert!(styled_node(&tree, look::GOALS_HEADLINE).is_none());
            assert!(styled_node(&tree, look::GOALS_SECTION).is_none());
        }
    }
}
#[test]
fn the_filters_press_and_move() {
    let mut page = Page::populated();
    let filters = [
        (GoalsFilter::All, b"All".as_slice()),
        (GoalsFilter::Needs, b"Needs you"),
        (GoalsFilter::Live, b"Under way"),
        (GoalsFilter::Ended, b"Ended"),
    ];
    for (chosen, _) in filters {
        page.filter(chosen);
        for words in [fixtures::words(), fixtures::browser_words()] {
            page.filter(chosen);
            let (tree, bindings) = page.tree(&words);
            let buttons = role_nodes(&tree, Role::Button);
            assert_eq!(buttons.len(), filters.len() + 1);
            for (index, (filter, label)) in buttons.iter().zip(filters) {
                let node = &tree.nodes()[*index];
                assert_eq!(node.name.as_deref(), Some(label));
                assert_eq!(node.states.as_slice().contains(&skein_ui::State::Pressed), chosen == filter);
                let action = bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press).unwrap();
                assert_eq!(
                    action,
                    temper_client_domain::Action::Pages(temper_client_domain::pages::Act::GoalsFilter { filter })
                );
                page.out = Queue::uncapped();
                page.event(temper_client_domain::Event::Act { action });
                let mut moves = List::uncapped();
                for request in &page.out {
                    if let temper_client_domain::Request::Show { address, .. } = request {
                        moves.push(address.clone()).unwrap();
                    }
                }
                assert_eq!(moves.as_slice(), [Address::Goals { project: fixtures::PROJECT, filter }]);
            }
        }
    }
}
#[test]
fn each_filter_shows_its_states() {
    let mut page = Page::populated();
    for (filter, expected) in [
        (
            GoalsFilter::All,
            [NEEDS_TASK, FUNDS_TASK, PAUSED_TASK, PLANNING_TASK, RUNNING_TASK, LANDED_TASK, CLOSED_TASK, FAILED_TASK]
                .as_slice(),
        ),
        (GoalsFilter::Needs, [NEEDS_TASK, FUNDS_TASK].as_slice()),
        (GoalsFilter::Live, [PAUSED_TASK, PLANNING_TASK, RUNNING_TASK].as_slice()),
        (GoalsFilter::Ended, [LANDED_TASK, CLOSED_TASK, FAILED_TASK].as_slice()),
    ] {
        page.filter(filter);
        for words in [fixtures::words(), fixtures::browser_words()] {
            let (tree, bindings) = page.tree(&words);
            assert_eq!(row_tasks(&tree, &bindings).as_ref(), expected);
        }
    }
}
#[test]
fn a_filter_no_goal_matches_says_none() {
    let mut page = Page::new();
    page.live(&[], false);
    page.ended(&[], false);
    for filter in [GoalsFilter::All, GoalsFilter::Needs, GoalsFilter::Live, GoalsFilter::Ended] {
        page.filter(filter);
        for words in [fixtures::words(), fixtures::browser_words()] {
            let (tree, bindings) = page.tree(&words);
            assert_eq!(styled_text(&tree, look::GOALS_EMPTY).as_ref(), b"None.");
            assert!(row_tasks(&tree, &bindings).is_empty());
        }
    }
    page.live(&[page_line(RUNNING_TASK, State::Running)], false);
    page.filter(GoalsFilter::Needs);
    assert_eq!(styled_text(&page.tree(&fixtures::browser_words()).0, look::GOALS_EMPTY).as_ref(), b"None.");
}
#[test]
fn more_goals_offers_and_reads() {
    let mut page = Page::new();
    page.live(&[], false);
    page.ended(&[page_line(LANDED_TASK, State::Landed)], true);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let (tree, bindings) = page.tree(&words);
        let more = styled_node(&tree, look::GOALS_MORE).unwrap();
        assert_eq!(more.role, Role::Button);
        assert_eq!(more.name.as_deref(), Some(b"More goals".as_slice()));
        let action = bindings.decode(more.binding.unwrap().token, skein_ui::Act::Press).unwrap();
        assert_eq!(action, temper_client_domain::Action::Pages(temper_client_domain::pages::Act::MoreGoals));
    }
    page.out = Queue::uncapped();
    page.event(temper_client_domain::Event::Act {
        action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::MoreGoals),
    });
    let mut cursors = List::uncapped();
    for request in &page.out {
        if let temper_client_domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { what: jig_client::Readable::Jig(c::Read::Tasks { ended, .. }), .. },
                },
        } = request
        {
            cursors
                .push(match ended {
                    Some(cursor) => Some(cursor.value.bytes()),
                    None => None,
                })
                .unwrap();
        }
    }
    assert_eq!(cursors.as_slice(), [Some(b"next".as_slice())]);
    page.ended(&[page_line(FAILED_TASK, State::Failed)], false);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let (tree, bindings) = page.tree(&words);
        assert!(styled_node(&tree, look::GOALS_MORE).is_none());
        assert_eq!(row_tasks(&tree, &bindings).as_ref(), [FAILED_TASK]);
        assert_eq!(styled_text(&tree, look::GOALS_HEADLINE).as_ref(), b"2 goals, 0 under way.");
    }
}
#[test]
fn the_page_draws_no_spend() {
    let page = Page::populated();
    for words in [fixtures::words(), fixtures::browser_words()] {
        let (tree, bindings) = page.tree(&words);
        let text = fixtures::text(&tree);
        for forbidden in [b"spent".as_slice(), b"budget"] {
            assert!(skein_lib::bytes::find(&text, forbidden).is_none());
        }
        let mut row_links = 0;
        for node in tree.nodes() {
            if node.role == Role::Link && !node.styles.as_slice().contains(&look::WORK_CHIP) {
                row_links += 1;
            }
        }
        assert_eq!(row_links, row_tasks(&tree, &bindings).len());
        assert_eq!(tree.nodes()[0].styles.as_slice(), [look::MAIN, look::GOALS]);
        let sections = style_order(
            &tree,
            &[look::GOALS_INTRO, look::GOALS_SECTION, look::GOALS_SECTION_HEAD, look::GOALS_FILTERS, look::GOALS_ROWS],
        );
        assert_eq!(
            sections.as_ref(),
            [look::GOALS_INTRO, look::GOALS_SECTION, look::GOALS_SECTION_HEAD, look::GOALS_FILTERS, look::GOALS_ROWS]
        );
    }
}

#[test]
fn the_link_shows_with_a_tracked_template() {
    let page = Page::populated();
    for words in [fixtures::words(), fixtures::browser_words()] {
        let (tree, bindings) = page.tree(&words);
        let mut end = 0;
        let mut found = false;
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&look::GOALS_SECTION_HEAD) {
                end = index + usize::try_from(node.size).unwrap();
            }
            if node.role == Role::Button && node.name.as_deref() == Some(b"Start one from a draft") {
                assert!(index < end);
                assert!(node.styles.as_slice().contains(&look::SHEET_LINK));
                assert_eq!(
                    bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press),
                    Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::OpenStart {
                        draft: None,
                        chat: None
                    }))
                );
                found = true;
            }
        }
        assert!(found);
    }
}
#[test]
fn none_with_none() {
    let mut page = Page::populated();
    let mut me = page.root.pages().me(page.root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().templates = List::from_boxed(Box::new([template(UNTRACKED_WAY, b"chat", false)]));
    fixtures::deliver_me(&mut page.root, page.me, me);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let (tree, _) = page.tree(&words);
        assert!(fixtures::find(&tree, Role::Button, b"Start one from a draft").is_none());
    }
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
