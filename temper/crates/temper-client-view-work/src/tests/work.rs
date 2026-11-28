//! Work's three read lists, five filters and both shells (client/pages/README.md).
use super::{chats, fixtures as f};

use crate::{
    bindings::Bindings,
    look,
    pages::work,
    words::{Times, Words},
};

use jig_client::{Down, Readable, Readout, Up, Watchable, contract as c};

use skein_lib::{List, Queue, Time, Token, Wall};

use skein_ui::{Builder, Role, Tree};

use temper_client_domain::{
    Address, Domain, Event, GoalsFilter, Place, Request, WorkFilter,
    forge::ways::Ways,
    pages::work::{Card, Item, Landed, LandedDraft, Work},
};

use temper_client_protocol_wire::{self as wire, v1 as w};

const NOW: Wall = Wall::from_nanos(1_792_656_000_000_000_000);

const RECENT: u64 = 1_792_655_640_000_000_000;

const GOAL: u64 = 1;

const CHAT: u64 = 3;

const DRAFT: u64 = 4;

const QUEUED_GOAL: u64 = 11;

const QUEUED_CHAT: u64 = 12;

const INDEPENDENT: u64 = 13;

const LANDED: u64 = 14;

fn list<T: Clone>(rows: &[T]) -> List<T> {
    List::from_boxed(rows.into())
}

fn text(value: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(value.into(), 1024).unwrap())
}

fn line(task: u64, title: &[u8]) -> c::TaskLine {
    let mut line = chats::line();
    line.chip.task = task;
    line.chip.title = text(title);
    line.last_activity = Wall::from_nanos(RECENT);
    line
}

fn limits() -> temper_client_domain::Limits {
    let mut limits = f::domain_limits();
    // The small whole project opens its lists together.
    limits.client.domain.handles = 64;
    limits.client.domain.text = 1024;
    limits.client.domain.long_text = 1024;
    limits.client.domain.turn_text = 1024;
    limits.client.domain.reads = 32;
    limits.client.exchanges = 32;
    limits
}

struct Fixture {
    root: Domain,
    out: Queue<Request>,
    goals: Token,
    chats: Token,
    changes: Token,
    drafts: Token,
    me: Token,
    milestones: Option<Token>,
    answers: List<Up>,
}

impl Fixture {
    fn new(ways: Option<Ways>, refused: bool) -> Self {
        let (mut root, me, ..) = f::root_at_with_values(
            Address::Models { project: f::PROJECT },
            &limits(),
            b"Shop",
            None,
            None,
            None,
            Some(&[]),
            ways,
            refused,
            true,
        );
        let mut frame = root.pages().me(root.client()).unwrap().clone();
        let mut goal = f::goals::named_template(jig_client::shapes::GOAL, b"goal", b"Separate goals", true);
        goal.label = 8;
        let mut chat = f::goals::template(jig_client::shapes::CHAT, b"chat", false);
        chat.label = 7;
        frame.projects.get_mut(0).unwrap().templates = list(&[chat, goal]);
        f::deliver_me(&mut root, me, frame);
        let mut fixture = Self {
            root,
            out: Queue::uncapped(),
            goals: Token::new(0),
            chats: Token::new(0),
            changes: Token::new(0),
            drafts: Token::new(0),
            me,
            milestones: None,
            answers: List::uncapped(),
        };
        fixture.go(Address::Work { project: f::PROJECT, filter: WorkFilter::All });
        for request in &fixture.out {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } },
            } = request
            {
                match subject {
                    Watchable::Jig(c::Subject::Tasks { query, .. }) => {
                        if query.tracked == Some(true) {
                            fixture.goals = *watch;
                        } else {
                            fixture.chats = *watch;
                        }
                    }
                    Watchable::Family { .. } => {
                        if Some(subject.clone())
                            == temper_client_domain::forge::changes::watch(f::PROJECT, &limits().forge)
                        {
                            fixture.changes = *watch;
                        } else if Some(subject.clone())
                            == temper_client_domain::forge::drafts::watch(f::PROJECT, &limits().forge)
                        {
                            fixture.drafts = *watch;
                        }
                    }
                    Watchable::Jig(_) => {}
                }
            }
        }
        assert_ne!(fixture.goals, fixture.chats);
        fixture
    }
    fn event(&mut self, event: Event) {
        temper_client_domain::step(
            &mut self.root,
            &skein_lib::Env { now: Time::ZERO, wall: NOW, limits: limits() },
            event,
            &mut self.out,
        );
    }
    fn up(&mut self, up: Up) {
        self.answers.push(up.clone()).unwrap();
        self.event(Event::Client { event: jig_client::Event::Up { up } });
    }
    fn go(&mut self, address: Address) {
        self.event(Event::Went { place: Place::Address(address) });
    }
    fn answer(&mut self, read: Token, value: c::ReadValue) {
        self.up(Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) });
    }
    fn live(&mut self, watch: Token, rows: &[c::TaskLine]) {
        self.up(Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                ended_mark: 0,
                rows: list(rows),
                more: false,
            }))),
        });
    }
    fn reads(&mut self) -> Box<[(Token, c::Read)]> {
        let mut reads = List::uncapped();
        while let Some(request) = self.out.pop() {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
            } = request
            {
                reads.push((read, what)).unwrap();
            }
        }
        reads.into_boxed()
    }
    fn populate(&mut self, full: bool) {
        self.populate_named(full, None);
    }
    fn populate_named(&mut self, full: bool, title: Option<&[u8]>) {
        self.drafts(&[]);
        let mut goal = line(GOAL, title.unwrap_or(b"membership-handoff"));
        goal.label = 8;
        goal.tracked = Some(1);
        goal.delegates_done = 6;
        goal.delegates_total = 9;
        goal.budget.numbers.spent = 31;
        goal.budget.numbers.budget = 50;
        goal.parts = list(&[temper_fixtures::client_documents::goal_part(
            Some((b"Six of nine tasks landed.", Wall::from_nanos(RECENT))),
            None,
        )]);
        self.live(self.goals, if full { core::slice::from_ref(&goal) } else { &[] });
        let mut draft = line(DRAFT, title.unwrap_or(b"Read barriers"));
        draft.parts = list(&[temper_fixtures::client_documents::chat_part(Some(b"Read barriers"))]);
        let chat_lines = [line(CHAT, title.unwrap_or(b"The comment is stale")), draft];
        self.live(self.chats, if full { &chat_lines } else { &[] });
        for (read, what) in self.reads() {
            if let c::Read::Tasks { .. } = what {
                self.answer(
                    read,
                    c::ReadValue::Tasks(Box::new(c::TaskPage {
                        ended_mark: 0,
                        live: List::uncapped(),
                        live_after: None,
                        ended: c::EndedGroup::None,
                    })),
                );
            } else if let c::Read::Inbox { .. } = what {
                self.up(Up::ReadDone { read, result: Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)) });
            } else if let c::Read::Notes { .. } = what {
                panic!("Work rows read no notes");
            }
        }
        let goal_title = title.unwrap_or(b"membership-handoff");
        let mut queued = List::with_capacity(3);
        if full {
            for (task, title) in [
                (QUEUED_GOAL, title.unwrap_or(b"Witness retention")),
                (QUEUED_CHAT, title.unwrap_or(b"Comment rewritten")),
                (INDEPENDENT, title.unwrap_or(b"Witness rejoin test")),
            ] {
                queued
                    .push(w::ChangesQueued {
                        goal: if task == QUEUED_GOAL { Some(origin(GOAL, goal_title)) } else { None },
                        chat: if task == QUEUED_CHAT { Some(origin(CHAT, b"chat")) } else { None },
                        milestone: None,
                        task,
                        title: title.into(),
                        ready_since: RECENT,
                        checks_since: None,
                    })
                    .unwrap();
            }
        }
        let landed = if full {
            list(&[w::ChangesLanded {
                goal: Some(origin(GOAL, goal_title)),
                chat: None,
                milestone: None,
                signed: List::with_capacity(8),
                commit: [0xab; 32],
                default_branch: true,
                task: LANDED,
                title: title.unwrap_or(b"Handoff states").into(),
                at: RECENT,
            }])
        } else {
            List::with_capacity(0)
        };
        let panel = w::ChangesPanel {
            catching_up: None,
            landed_default: landed.clone(),
            landed_default_more: 0,
            project: f::PROJECT,
            landing: b"main".as_slice().into(),
            queues: list(&[w::ChangesQueue {
                way: w::Way::Queue,
                in_flight: None,
                repository: b"owner/repo".as_slice().into(),
                branch: b"main".as_slice().into(),
                red: full,
                changes: queued,
            }]),
            more: 0,
            landed,
            landed_more: 0,
        };
        self.up(Up::FamilyDelivered {
            watch: self.changes,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::changes_panel(&panel),
            },
        });
    }
    fn drafts(&mut self, rows: &[w::LandedDraft]) {
        let panel = w::DraftsPanel { project: f::PROJECT, drafts: list(rows) };
        self.up(Up::FamilyDelivered {
            watch: self.drafts,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::drafts_panel(&panel),
            },
        });
    }
    fn open_milestones(&mut self) -> Token {
        let mut me = self.root.pages().me(self.root.client()).unwrap().clone();
        let templates = &mut me.projects.get_mut(0).unwrap().templates;
        let mut template = templates.get(1).unwrap().clone();
        template.number = temper_client_domain::milestones::TEMPLATE;
        template.label = 9;
        *templates = list(&[templates.get(0).unwrap().clone(), templates.get(1).unwrap().clone(), template]);
        self.up(Up::Watched { watch: self.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) });
        for request in &self.out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, window }) },
                    },
            } = request
                && query.label == Some(9)
                && *window > 1
            {
                self.milestones = Some(*watch);
                return *watch;
            }
        }
        panic!("milestone list watch");
    }
    fn serve_milestone(
        &mut self,
        own: &c::TaskLine,
        live: &[c::TaskLine],
        ended: &[c::TaskLine],
        whole: bool,
        totals: Option<&w::MilestoneTotals>,
    ) {
        let mut own = own.clone();
        if let Some(totals) = totals {
            let mut goals = List::uncapped();
            for line in live.iter().chain(ended) {
                goals.push((line.clone(), None)).unwrap();
            }
            own.parts = list(&[f::milestones::part(whole.then_some(NOW), totals.clone(), goals.as_slice(), 0)]);
        }
        // The opening ended page is still its own read, never a drawn milestone's.
        let mut requests = List::uncapped();
        while let Some(request) = self.out.pop() {
            if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request {
                requests.push(down).unwrap();
            }
        }
        for request in requests.into_boxed() {
            match request {
                Down::Read { read, what: Readable::Jig(c::Read::Tasks { query, .. }) } => {
                    assert!(query.parent.is_none(), "no drawn goals query");
                    self.answer(
                        read,
                        ended_page(if query.label == Some(9) && own.chip.phase == c::Phase::Ended {
                            core::slice::from_ref(&own)
                        } else {
                            &[]
                        }),
                    );
                }
                Down::Read { what: Readable::Jig(c::Read::Task { .. }) | Readable::Family { .. }, .. } => {
                    panic!("no drawn document or totals read")
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
        let listed = self.milestones.expect("milestone list watch");
        self.live(listed, if own.chip.phase == c::Phase::Ended { &[] } else { core::slice::from_ref(&own) });
    }
    fn tree(&self, words: &Words) -> (Tree, Bindings) {
        let mut b = Builder::new(&f::tree_limits());
        let mut bindings = Bindings::new(f::tree_limits().bindings);
        b.open(Role::Page);
        work::main(&mut b, &self.root, words, &mut bindings, f::PROJECT, &mut Times::new(NOW));
        b.close();
        (b.finish(), bindings)
    }
}

fn has(tree: &Tree, value: &str) -> bool {
    skein_lib::bytes::find(&f::text(tree), value.as_bytes()).is_some()
}

fn styled(tree: &Tree, style: skein_ui::Style) -> Box<[&skein_ui::Node]> {
    let mut nodes = List::uncapped();
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&style) {
            nodes.push(node).unwrap();
        }
    }
    nodes.into_boxed()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the project fixture checks goal, chat and unread landings across successive history deliveries"
)]
fn landed_this_week_names_the_goal_once_at_its_main_commit() {
    let mut page = Fixture::new(None, false);
    page.populate(false);
    let branch = w::ChangesLanded {
        goal: Some(origin(GOAL, b"The whole goal")),
        chat: None,
        milestone: None,
        signed: List::with_capacity(8),
        task: 21,
        title: b"Goal-branch implementation".as_slice().into(),
        at: RECENT,
        commit: [0x12; 32],
        default_branch: false,
    };
    let landing = w::ChangesLanded {
        goal: Some(origin(GOAL, b"The whole goal")),
        chat: None,
        milestone: None,
        signed: List::with_capacity(8),
        task: 22,
        title: b"Landing fallback".as_slice().into(),
        at: RECENT + 1,
        commit: [0xab; 32],
        default_branch: true,
    };
    let panel = w::ChangesPanel {
        catching_up: None,
        project: f::PROJECT,
        landing: b"main".as_slice().into(),
        queues: List::with_capacity(0),
        more: 0,
        landed: list(&[landing.clone(), branch]),
        landed_more: 0,
        landed_default: list(&[landing]),
        landed_default_more: 0,
    };
    page.up(Up::FamilyDelivered {
        watch: page.changes,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::changes_panel(&panel),
        },
    });
    let (tree, bindings) = page.tree(&f::words());
    assert!(has(&tree, "Landed this week · 1"), "counts main landings, not goal-branch changes");
    assert!(has(&tree, "One landed this week."));
    assert_eq!(styled(&tree, look::WORK_ROW).len(), 1);
    assert!(has(&tree, "The whole goal"));
    assert!(!has(&tree, "Goal-branch implementation"));
    assert!(!has(&tree, "Landing fallback"));
    assert!(has(&tree, "abababa"));
    assert!(has(&tree, "Also one change landed into a goal branch."));
    let mut tasks = List::uncapped();
    for node in tree.nodes() {
        if let Some(link) = node.link
            && let Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                address: Address::Task { task },
            })) = bindings.decode(link.token, skein_ui::Act::Press)
        {
            tasks.push(task).unwrap();
        }
    }
    assert_eq!(tasks.as_slice(), &[GOAL]);

    // Rows without a goal stay change rows; a named chat keeps its provenance.
    let mut parts = panel;
    let mut branches = List::with_capacity(2);
    for row in parts.landed.into_boxed() {
        let mut row = row;
        row.default_branch = false;
        branches.push(row).unwrap();
    }
    parts.landed = branches;
    let mut defaults = List::with_capacity(3);
    for (task, title, commit) in [(30, "Chat landing", 0x32), (31, "Unread landing", 0x45), (32, "Goal fallback", 0x67)]
    {
        defaults
            .push(w::ChangesLanded {
                goal: None,
                chat: if task == 30 { Some(origin(CHAT, b"chat")) } else { None },
                milestone: None,
                signed: List::with_capacity(8),
                task,
                title: title.as_bytes().into(),
                at: RECENT,
                commit: [commit; 32],
                default_branch: true,
            })
            .unwrap();
    }
    parts.landed_default = defaults;
    let panel = parts;
    page.up(Up::FamilyDelivered {
        watch: page.changes,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::changes_panel(&panel),
        },
    });
    let (tree, _) = page.tree(&f::words());
    assert!(has(&tree, "Landed this week · 3"));
    assert!(has(&tree, "Chat landing"));
    assert!(has(&tree, "Unread landing"));
    assert!(has(&tree, "Goal fallback"));
    assert!(has(&tree, "a chat with you · 3232323"));
    assert!(has(&tree, "4545454"));
    assert!(has(&tree, "6767676"));
    assert!(has(&tree, "Also two changes landed into goal branches."));
}

fn heading_work(flight: u32, drafts: u32, landed: u32) -> Work {
    let card = Card { goals: List::uncapped(), item: Item::Goal(GOAL), changes: List::uncapped() };
    let mut work = Work::default();
    for _ in 0..flight {
        work.in_flight.push(card.clone()).unwrap();
    }
    for _ in 0..drafts {
        work.drafts.push(card.clone()).unwrap();
    }
    for _ in 0..landed {
        work.landed
            .push(Landed {
                milestone: None,
                signed: None,
                item: Item::Goal(GOAL),
                change: LANDED,
                commit: [0xab; 32],
                at: NOW,
            })
            .unwrap();
    }
    work
}

#[test]
fn the_heading_counts_sections_in_words_then_digits_and_leaves_zero_parts_out() {
    for (flight, drafts, landed, expected) in [
        (0, 0, 0, "Nothing in flight yet."),
        (1, 0, 0, "One in flight."),
        (0, 1, 0, "One draft."),
        (0, 0, 1, "One landed this week."),
        (4, 2, 23, "Four in flight, two drafts, 23 landed this week."),
        (0, 2, 3, "Two drafts, three landed this week."),
        (2, 0, 3, "Two in flight, three landed this week."),
        (2, 3, 0, "Two in flight, three drafts."),
        (9, 0, 0, "Nine in flight."),
        (10, 0, 0, "10 in flight."),
    ] {
        assert_eq!(work::heading(&heading_work(flight, drafts, landed), &f::words()).as_ref(), expected.as_bytes());
    }
}

#[test]
fn filters_count_link_and_keep_current_even_when_empty_or_overridden() {
    for (ways, refused, goals_shown, changes_shown) in [
        (None, false, true, true),
        (None, true, true, true),
        (Some(Ways { goals: true, changes: true, ..Ways::STOCK }), false, true, true),
        (Some(Ways { goals: false, changes: false, ..Ways::STOCK }), false, false, false),
    ] {
        let mut page = Fixture::new(ways, refused);
        page.populate(true);
        let (tree, bindings) = page.tree(&f::browser_words());
        let filters = styled(&tree, look::WORK_CHIP);
        for (label, address, shown) in [
            ("All", Address::Work { project: f::PROJECT, filter: WorkFilter::All }, true),
            ("Drafts · 1", Address::Work { project: f::PROJECT, filter: WorkFilter::Drafts }, true),
            ("Goals · 1", Address::Goals { project: f::PROJECT, filter: GoalsFilter::All }, goals_shown),
            ("Changes · 4", Address::Changes { project: f::PROJECT }, changes_shown),
            ("Waits", Address::Waits { project: f::PROJECT, filter: temper_client_domain::WaitsFilter::ALL }, true),
            ("Chats · 1", Address::Chats { filter: temper_client_domain::ChatsFilter::All }, true),
        ] {
            let mut found = None;
            for node in &filters {
                if node.name.as_deref() == Some(label.as_bytes()) {
                    found = Some(*node);
                }
            }
            assert_eq!(found.is_some(), shown, "{ways:?} {refused}: {label}");
            if let Some(node) = found {
                assert_eq!(
                    bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press),
                    Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go { address }))
                );
            }
        }
        assert!(filters[0].states.as_slice().contains(&skein_ui::State::Current));
        assert_eq!(styled(&tree, look::WORK_CARD).len(), 4, "overrides leave All's cards");
        for (address, current) in [
            (Address::Work { project: f::PROJECT, filter: WorkFilter::Drafts }, "Drafts · 1"),
            (Address::Goals { project: f::PROJECT, filter: GoalsFilter::Ended }, "Goals · 1"),
            (Address::Changes { project: f::PROJECT }, "Changes · 4"),
        ] {
            page.go(address);
            let (tree, _) = page.tree(&f::browser_words());
            let mut selected = List::uncapped();
            for node in styled(&tree, look::WORK_CHIP) {
                if node.states.as_slice().contains(&skein_ui::State::Current) {
                    selected.push(node).unwrap();
                }
            }
            assert_eq!(selected.len(), 1);
            assert_eq!(selected.get(0).unwrap().name.as_deref(), Some(current.as_bytes()));
        }
    }
    let mut empty = Fixture::new(None, false);
    empty.populate(false);
    empty.go(Address::Work { project: f::PROJECT, filter: WorkFilter::Drafts });
    let (tree, _) = empty.tree(&f::browser_words());
    assert_eq!(styled(&tree, look::WORK_CHIP).len(), 3, "All, Drafts, and Waits");
    assert!(has(&tree, "Drafts · 0"));
    assert!(has(&tree, "None."));
}

#[test]
fn the_terminal_numbers_filters_and_cards_with_their_from_headline_and_rows() {
    let mut page = Fixture::new(None, false);
    page.populate(true);
    let (tree, _) = page.tree(&f::words());
    let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
    skein_ui_text::render(&tree, b"temper", None, &mut print);
    let printed = print.text.as_slice();
    assert!(skein_lib::bytes::find(printed, b"6 / 9: 6 / 9").is_none());
    for value in [
        "[1] All",
        "[2] Drafts · 1",
        "[3] Goals · 1",
        "[4] Changes · 4",
        "[5] Waits",
        "[6] Chats · 1",
        "[7] goal membership-handoff",
        "Six of nine tasks landed.",
        "Written by the goal at 07:54",
        "Witness retention",
        "landed 07:54",
    ] {
        assert!(skein_lib::bytes::find(printed, value.as_bytes()).is_some(), "missing {value}");
    }
}

#[test]
fn work_rows_draw_signers_after_the_title_in_the_byline_style() {
    for words in [f::browser_words(), f::words()] {
        for signed in [None, Some(b"signed by you and Sam".as_slice())] {
            let mut b = Builder::new(&f::tree_limits());
            let mut bindings = Bindings::new(f::tree_limits().bindings);
            b.open(Role::Page);
            b.open(Role::List);
            work::work_row(
                &mut b,
                &words,
                &mut bindings,
                LANDED,
                b"12:30",
                b"Change title",
                b"abababa",
                signed,
                look::WORK_ROW,
            );
            b.close();
            b.close();
            let tree = b.finish();
            let mut title = None;
            let mut signer = None;
            for (index, node) in tree.nodes().iter().enumerate() {
                if node.role == Role::Link && node.name.as_deref() == Some(b"Change title".as_slice()) {
                    title = Some(index);
                }
                if node.name.as_deref() == Some(" · signed by you and Sam".as_bytes()) {
                    signer = Some(index);
                }
            }
            let title = title.unwrap();
            let bylines = styled(&tree, look::WORK_BYLINE);
            assert_eq!(bylines.len(), if signed.is_some() { 2 } else { 1 });
            if signed.is_some() {
                let at = signer.expect("signed words after title");
                assert!(at > title);
                assert!(tree.nodes().get(at).unwrap().styles.as_slice().contains(&look::WORK_BYLINE));
                assert_eq!(bylines.last().unwrap().name.as_deref(), Some(b"abababa".as_slice()));
            } else {
                assert!(!has(&tree, "signed by"));
            }
        }
    }
}

const READY: u64 = 1_792_159_800_000_000_000;
// Friday 14:10 in the clock's preceding week.
fn landed_draft(path: &[u8], title: &[u8]) -> w::LandedDraft {
    w::LandedDraft {
        forge: 3,
        owner: b"author".as_slice().into(),
        name: b"NAME".as_slice().into(),
        path: path.into(),
        branch: b"main".as_slice().into(),
        commit: b"abc".as_slice().into(),
        title: title.into(),
        writer: 9,
        at: READY,
        chat: Some(99),
        goal: None,
        ready: true,
    }
}

fn subtree_text(tree: &Tree, node: &skein_ui::Node) -> Box<[u8]> {
    let mut text = List::uncapped();
    let mut taking = false;
    let mut left = node.size;
    for next in tree.nodes() {
        if core::ptr::eq(next, node) {
            taking = true;
        }
        if taking && left > 0 {
            for run in &next.text {
                for byte in &run.text {
                    text.push(*byte).unwrap();
                }
            }
            left = left.checked_sub(1).unwrap();
        }
    }
    text.into_boxed()
}

struct LandedChoices {
    configured: Option<u32>,
    tracked: bool,
    labels: &'static [&'static [u8]],
}

#[test]
fn landed_cards_draw_clock_provenance_and_the_three_choice_cases_in_both_shells() {
    use temper_client_domain::{Action, pages::Act};
    for words in [f::browser_words(), f::words()] {
        for LandedChoices { configured, tracked, labels } in [
            LandedChoices {
                configured: Some(jig_client::shapes::GOAL),
                tracked: true,
                labels: &[b"Start a goal: Task by task", b"Another way"],
            },
            LandedChoices { configured: Some(99), tracked: true, labels: &[b"Start a goal\xe2\x80\xa6"] },
            LandedChoices { configured: None, tracked: true, labels: &[b"Start a goal\xe2\x80\xa6"] },
            LandedChoices { configured: Some(jig_client::shapes::GOAL), tracked: false, labels: &[] },
        ] {
            let ways = match configured {
                Some(way) => Some(Ways { way, ..Ways::STOCK }),
                None => None,
            };
            let mut page = Fixture::new(ways, false);
            if !tracked {
                let mut me = page.root.pages().me(page.root.client()).unwrap().clone();
                for template in me.projects.get_mut(0).unwrap().templates.iter_mut() {
                    template.tracked = None;
                }
                f::deliver_me(&mut page.root, page.me, me);
            }
            page.populate(false);
            let row = landed_draft(b"docs/design/draft/catch-up.md", b"Catch up");
            page.drafts(core::slice::from_ref(&row));
            let (tree, bindings) = page.tree(&words);
            assert_eq!(skein_ui::outline::check(&tree), None);
            assert!(has(&tree, "One draft."));
            assert!(has(&tree, "Drafts · 1"), "only a landed draft shows the filter");
            let from = styled(&tree, look::WORK_FROM);
            assert_eq!(from.len(), 1);
            assert_eq!(
                subtree_text(&tree, from[0]).as_ref(),
                "draft · catch-up.md · landed in docs/design/draft/ · ready since Fri 14:10".as_bytes()
            );
            assert!(has(&tree, "Catch up"));
            let choices = styled(&tree, look::WORK_CHOICE);
            let mut actual = List::uncapped();
            for choice in &choices {
                actual.push(choice.name.as_deref().unwrap()).unwrap();
            }
            assert_eq!(actual.as_slice(), labels);
            for (i, choice) in choices.iter().enumerate() {
                assert_eq!(choice.role, Role::Button);
                assert_eq!(choice.styles.as_slice().contains(&look::WORK_RECOMMENDED), i == 0);
                assert_eq!(
                    bindings.decode(choice.binding.unwrap().token, skein_ui::Act::Press),
                    Some(Action::Pages(Act::OpenLanded {
                        draft: LandedDraft::row(&row),
                        start: labels.len() == 2 && i == 0,
                    }))
                );
            }
            assert_eq!(has(&tree, "Shop’s way"), labels.len() == 2);
            if words.shell == crate::words::Shell::Terminal {
                let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
                skein_ui_text::render(&tree, b"temper", None, &mut print);
                let printed = print.text.as_slice();
                let from = skein_lib::bytes::find(printed, "draft · catch-up.md".as_bytes()).unwrap();
                let title = skein_lib::bytes::find(printed, b"Catch up").unwrap();
                assert!(from < title);
                if let Some(label) = labels.first() {
                    assert!(title < skein_lib::bytes::find(printed, label).unwrap());
                    let numbered = crate::accounts::words::join(&[b"[4] ", label]);
                    assert!(skein_lib::bytes::find(printed, &numbered).is_some(), "{printed:?}");
                }
            }
            page.drafts(&[landed_draft(b"root.md", b"")]);
            let (tree, _) = page.tree(&words);
            assert!(has(&tree, "landed in NAME/"));
            assert!(styled(&tree, look::WORK_QUOTE).is_empty());
        }
    }
}

#[test]
fn landed_and_chat_drafts_share_the_headings_count_and_newest_first_order() {
    let mut page = Fixture::new(None, false);
    page.populate(true);
    let row = landed_draft(b"catch-up.md", b"Catch up");
    let mut parts = row;
    parts.at = RECENT.checked_add(1).unwrap();
    page.drafts(&[parts]);
    page.go(Address::Work { project: f::PROJECT, filter: WorkFilter::Drafts });
    let (tree, _) = page.tree(&f::browser_words());
    assert!(has(&tree, "Three in flight, two drafts, one landed this week."));
    assert_eq!(styled(&tree, look::WORK_CARD).len(), 2);
    let from = styled(&tree, look::WORK_FROM);
    assert!(skein_lib::bytes::find(&subtree_text(&tree, from[0]), b"catch-up.md").is_some());
    assert!(skein_lib::bytes::find(&subtree_text(&tree, from[1]), b"read-barriers.md").is_some());
}

const M: u64 = 100;
fn milestone_line() -> c::TaskLine {
    let mut own = line(M, b"Store shape");
    own.budget.numbers.budget = 400;
    own.chip.details.details.executor =
        c::Executor::Procedure { connector: jig_client::shapes::RECURRING, code: jig_client::shapes::GATHERING };
    own
}
fn ended_page(rows: &[c::TaskLine]) -> c::ReadValue {
    c::ReadValue::Tasks(Box::new(c::TaskPage {
        ended_mark: 0,
        live: List::uncapped(),
        live_after: None,
        ended: c::EndedGroup::Rows { rows: list(rows), after: None, cut: false },
    }))
}
fn milestone_totals(done: u32, failed: u32, live: u32) -> w::MilestoneTotals {
    w::MilestoneTotals { task: M, done, failed, live, closed: 0, spent: 212, ended: None, last_landing: None, reds: 0 }
}
fn milestone_goal(task: u64, title: &[u8]) -> c::TaskLine {
    let mut line = line(task, title);
    line.requester = c::Requester::Task(M);
    line = f::milestones::in_milestone(line, M);
    line.label = 8;
    line.tracked = Some(1);
    line
}
fn ended_goal(task: u64, title: &[u8], ending: c::ChipEnding, at: u64) -> c::TaskLine {
    let mut line = milestone_goal(task, title);
    line.chip.phase = c::Phase::Ended;
    line.chip.details.phase = c::ChipPhase::Ended;
    line.chip.details.details.ending = Some(ending);
    line.ended_at = Some(Wall::from_nanos(at));
    line
}
fn queued_change(task: u64) -> w::ChangesQueued {
    w::ChangesQueued {
        goal: Some(origin(3, b"C4 running")),
        chat: None,
        milestone: Some(origin(M, b"Store shape")),
        task,
        title: b"Move the reader".as_slice().into(),
        ready_since: RECENT,
        checks_since: None,
    }
}
fn send_panel(page: &mut Fixture, queued: &[w::ChangesQueued], landed: &[w::ChangesLanded]) {
    let panel = w::ChangesPanel {
        catching_up: None,
        project: f::PROJECT,
        landing: b"main".as_slice().into(),
        more: 0,
        landed_more: 0,
        landed_default_more: 0,
        queues: list(&[w::ChangesQueue {
            way: w::Way::Queue,
            repository: b"owner/repo".as_slice().into(),
            branch: b"main".as_slice().into(),
            in_flight: None,
            red: false,
            changes: list(queued),
        }]),
        landed: list(landed),
        landed_default: list(landed),
    };
    page.up(Up::FamilyDelivered {
        watch: page.changes,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::changes_panel(&panel),
        },
    });
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one fixture checks the grouped live card and its ended headline in both shells"
)]
fn milestones_hold_their_goals_changes_and_folds_in_both_shells() {
    let mut page = Fixture::new(None, false);
    page.populate(false);
    let mut running = milestone_goal(3, b"C4 running");
    running.delegates_done = 5;
    running.delegates_total = 7;
    let mut waiting = milestone_goal(5, b"C5 waiting");
    waiting.chip.details.phase = c::ChipPhase::Waiting { dependencies: list(&[3]), more: false, holds: false };
    waiting.chip.phase = c::Phase::Waiting;
    let done = ended_goal(4, b"S8 done", c::ChipEnding::Done(c::ResultValue::Report(text(b"done"))), RECENT);
    let failed = ended_goal(6, b"S9 failed", c::ChipEnding::Failed(text(b"failed")), RECENT - 1);
    let closed = ended_goal(
        7,
        b"S10 closed",
        c::ChipEnding::Cancelled { by: c::Requester::Person(1), at: NOW, reason: text(b"closed"), result: None },
        RECENT - 2,
    );
    page.live(page.goals, &[waiting.clone(), running.clone()]);
    let watch = page.open_milestones();
    assert!(
        !temper_client_domain::pages::work::facts(&page.root, f::PROJECT).unwrap().known.in_flight(),
        "waits for milestone list"
    );
    let own = milestone_line();
    page.live(watch, core::slice::from_ref(&own));
    let (unread, _) = page.tree(&f::browser_words());
    assert_eq!(styled(&unread, look::WORK_HEADLINE).len(), 1);
    assert!(styled(&unread, look::WORK_HEADLINE)[0].text.is_empty(), "ended milestone list still unread");
    send_panel(&mut page, &[queued_change(21)], &[]);
    let mut owed = milestone_totals(1, 1, 2);
    owed.reds = 2;
    page.serve_milestone(&own, &[waiting, running], &[failed, closed, done], true, Some(&owed));
    let facts = temper_client_domain::pages::work::facts(&page.root, f::PROJECT).unwrap();
    assert_eq!(facts.milestones.len(), 1);
    assert_eq!(facts.goals.len(), 5, "includes the drawn milestone's ended goals");
    let placed = temper_client_domain::pages::work::place(&facts, NOW);
    assert_eq!(placed.in_flight.len(), 1);
    assert_eq!(placed.in_flight.get(0).unwrap().item, Item::Milestone(M));
    assert_eq!(placed.in_flight.get(0).unwrap().goals.get(0).unwrap().changes.get(0).unwrap().task, 21);
    for words in [f::browser_words(), f::words()] {
        let (tree, bindings) = page.tree(&words);
        assert_eq!(skein_ui::outline::check(&tree), None);
        assert_eq!(styled(&tree, look::WORK_CARD).len(), 1);
        assert_eq!(styled(&tree, look::MS_CARD).len(), 1);
        for value in [
            "One in flight.",
            "In flight · 1",
            "A milestone opens to its goals, and a goal to its tasks and changes;",
            "4 goals · whole",
            "212 spent of 400",
            "1 of 4 goals done; 2 reds after landing owed.",
            "running · 5 / 7",
            "Move the reader",
            "waits on C4",
            "S9 failed",
            "S10 closed",
        ] {
            assert!(has(&tree, value), "missing {value}");
        }
        let cards = styled(&tree, look::MS_CARD);
        let card_text = subtree_text(&tree, cards[0]);
        assert_eq!(skein_lib::bytes::count(&card_text, b"Move the reader", 2), 1, "change on its goal line alone");
        if words.shell == crate::words::Shell::Browser {
            let folds = styled(&tree, look::MS_FOLD);
            assert_eq!(folds.len(), 2);
            assert_eq!(folds[0].name.as_deref(), Some(b"1 waits".as_slice()));
            assert_eq!(folds[1].name.as_deref(), Some("1 done, the last S8 at 07:54 · 1 failed · 1 closed".as_bytes()));
            for fold in folds {
                assert_eq!(fold.role, Role::Disclosure);
                assert!(!fold.states.as_slice().contains(&skein_ui::State::Expanded));
            }
        } else {
            let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
            skein_ui_text::render(&tree, b"temper", None, &mut print);
            let printed = print.text.as_slice();
            assert!(skein_lib::bytes::find(printed, "[5] milestone · Store shape".as_bytes()).is_some());
            assert!(skein_lib::bytes::find(printed, "▒■□× 1 done of 4".as_bytes()).is_some());
            assert!(
                skein_lib::bytes::find(printed, b"1 waits").unwrap()
                    < skein_lib::bytes::find(printed, b"C5 waiting").unwrap()
            );
        }
        let mut links = List::uncapped();
        for node in tree.nodes() {
            if let Some(link) = node.link
                && let Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                    address: Address::Task { task },
                })) = bindings.decode(link.token, skein_ui::Act::Press)
            {
                links.push(task).unwrap();
            }
        }
        assert_eq!(links.as_slice(), &[M, 3, 21, 5, 4, 6, 7]);
    }
    // The renderer also words an ended card; placement only keeps live cards in flight.
    let mut own = own;
    own.chip.phase = c::Phase::Ended;
    own.chip.details.phase = c::ChipPhase::Ended;
    own.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(text(b"done"))));
    let mut totals = milestone_totals(1, 0, 0);
    totals.reds = 2;
    assert_eq!(
        temper_client_domain::pages::milestones::state(&own, &[], Some(&totals), false),
        temper_client_domain::pages::milestones::State::Complete
    );
    own.parts = list(&[f::milestones::part(Some(NOW), totals, &[], 0)]);
    page.live(watch, &[own]);
    for words in [f::browser_words(), f::words()] {
        let mut b = Builder::new(&f::tree_limits());
        let mut bindings = Bindings::new(f::tree_limits().bindings);
        work::card_view(
            &mut b,
            &page.root,
            &words,
            &mut bindings,
            &mut Times::new(NOW),
            f::PROJECT,
            &Card { item: Item::Milestone(M), changes: List::uncapped(), goals: List::uncapped() },
        );
        let tree = b.finish();
        assert!(has(&tree, "All 1 goal done; 2 reds after landing owed."));
    }
}

#[test]
fn a_milestone_part_draws_goals_outside_the_shared_window_and_the_newest_report() {
    let mut page = Fixture::new(None, false);
    page.populate(false);
    let watch = page.open_milestones();
    let mut first = milestone_goal(1, b"C1 first");
    first.last_activity = Wall::from_nanos(1);
    let mut waiting = milestone_goal(2, b"C2 waits");
    waiting.chip.phase = c::Phase::Waiting;
    waiting.chip.details.phase = c::ChipPhase::Waiting { dependencies: list(&[1]), more: false, holds: false };
    let mut newest = milestone_goal(3, b"C3 newest");
    newest.last_activity = Wall::from_nanos(3);
    let done = ended_goal(4, b"C4 done", c::ChipEnding::Done(c::ResultValue::Report(text(b"done"))), RECENT);
    let other = ended_goal(5, b"C5 done", c::ChipEnding::Done(c::ResultValue::Report(text(b"done"))), RECENT - 1);
    let mut own = milestone_line();
    own.parts = list(&[f::milestones::part(
        Some(NOW),
        milestone_totals(2, 0, 3),
        &[
            (newest, Some(b"The newest part report.")),
            (waiting, None),
            (first, Some(b"The older part report.")),
            (other, None),
            (done, None),
        ],
        0,
    )]);
    page.live(watch, core::slice::from_ref(&own));
    page.serve_milestone(&own, &[], &[], false, None);
    let facts = temper_client_domain::pages::work::facts(&page.root, f::PROJECT).unwrap();
    assert_eq!(facts.goals.len(), 5, "no project goal rows are needed");
    for words in [f::browser_words(), f::words()] {
        let (tree, _) = page.tree(&words);
        assert!(has(&tree, "5 goals · whole"));
        assert!(has(&tree, "2 of 5 goals done. C3: The newest part report, and one more running."));
        assert!(!has(&tree, "The older part report"));
        let mut live = List::uncapped();
        for node in tree.nodes() {
            if node.role == Role::Item && node.styles.as_slice().contains(&look::MS_LIVE) {
                live.push(node).unwrap();
            }
        }
        assert_eq!(live.len(), 2);
        assert!(skein_lib::bytes::find(&subtree_text(&tree, live.get(0).unwrap()), b"C1 first").is_some());
        assert!(skein_lib::bytes::find(&subtree_text(&tree, live.get(1).unwrap()), b"C3 newest").is_some());
        if words.shell == crate::words::Shell::Browser {
            let strip = styled(&tree, look::MS_STRIP);
            assert_eq!(strip[0].name.as_deref(), Some(b"2 done of 5".as_slice()));
            assert_eq!(styled(&tree, look::MS_DONE).len(), 2);
            assert_eq!(styled(&tree, look::MS_WAITS).len(), 2, "waiting segment and row");
        } else {
            assert!(has(&tree, "▒□▒■■ 2 done of 5"));
        }
    }
    let mut goals = List::with_capacity(16);
    for task in 1..=16 {
        let row = if task <= 7 {
            ended_goal(task, b"S1 done", c::ChipEnding::Done(c::ResultValue::Report(text(b"done"))), RECENT)
        } else if task == 8 {
            milestone_goal(task, b"S8 running")
        } else {
            ended_goal(
                task,
                b"S9 closed",
                c::ChipEnding::Cancelled {
                    by: c::Requester::Person(1),
                    at: NOW,
                    reason: text(b"closed"),
                    result: None,
                },
                RECENT,
            )
        };
        goals.push((row, None)).unwrap();
    }
    let mut totals = milestone_totals(7, 0, 1);
    totals.closed = 11;
    own.parts = list(&[f::milestones::part(Some(NOW), totals, goals.as_slice(), 3)]);
    page.live(watch, &[own]);
    let drawn = page.root.pages().milestones().unwrap().milestone(M).unwrap();
    assert_eq!(drawn.goals.len(), 16);
    assert_eq!(drawn.more, 3);
    for words in [f::browser_words(), f::words()] {
        let (tree, _) = page.tree(&words);
        assert!(has(&tree, "8 goals · whole"));
        assert!(has(&tree, "7 of 8 goals done."));
    }
}

#[test]
fn landed_goal_rows_name_their_milestone_and_only_its_recent_completing_change() {
    const WEEK: u64 = 604_800_000_000_000;
    for (failed, completed, last_change, read, complete) in [
        (0, RECENT, 21, true, true),
        (0, NOW.as_nanos() - WEEK, 21, true, true),
        (0, NOW.as_nanos() - WEEK - 1, 21, true, false),
        (0, NOW.as_nanos() + 1, 21, true, false),
        (0, RECENT, 22, true, false),
        (1, RECENT, 21, true, false),
        (0, RECENT, 21, false, false),
    ] {
        let mut page = Fixture::new(None, false);
        page.populate(false);
        let watch = page.open_milestones();
        page.live(watch, &[]);
        let mut own = milestone_line();
        own.chip.phase = c::Phase::Ended;
        own.chip.details.phase = c::ChipPhase::Ended;
        own.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(text(b"done"))));
        own.ended_at = Some(Wall::from_nanos(RECENT));
        let mut totals = milestone_totals(1, failed, 0);
        totals.ended = Some(completed);
        totals.last_landing = Some(w::MilestoneLanding { at: RECENT, goal: 3, change: last_change });
        let landing = w::ChangesLanded {
            goal: Some(origin(3, b"C4 running")),
            chat: None,
            milestone: Some(origin(M, b"Store shape")),
            task: 21,
            title: b"The landing".as_slice().into(),
            at: RECENT,
            commit: [0xab; 32],
            default_branch: true,
            signed: List::with_capacity(8),
        };
        send_panel(&mut page, &[], &[landing]);
        page.serve_milestone(&own, &[], &[], true, if read { Some(&totals) } else { None });
        let facts = temper_client_domain::pages::work::facts(&page.root, f::PROJECT).unwrap();
        let placed = temper_client_domain::pages::work::place(&facts, NOW);
        assert!(placed.in_flight.is_empty(), "ended milestone is never a card");
        assert_eq!(placed.landed.len(), 1, "a milestone is never its own landed row");
        let row = placed.landed.get(0).unwrap();
        assert_eq!(row.item, Item::Goal(3));
        assert_eq!(row.milestone.as_ref().unwrap().name.as_ref(), b"Store shape");
        assert_eq!(row.milestone.as_ref().unwrap().completed.is_some(), complete);
        for words in [f::browser_words(), f::words()] {
            let (tree, bindings) = page.tree(&words);
            assert!(has(&tree, "Landed this week · 1"));
            assert!(has(&tree, "C4 running"));
            assert!(has(&tree, " · in Store shape"));
            assert_eq!(has(&tree, ", now complete"), complete, "{failed} {completed} {last_change} {read}");
            let mut links = List::uncapped();
            for node in tree.nodes() {
                if let Some(link) = node.link
                    && let Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                        address: Address::Task { task },
                    })) = bindings.decode(link.token, skein_ui::Act::Press)
                {
                    links.push(task).unwrap();
                }
            }
            assert_eq!(links.as_slice(), &[3]);
        }
    }
}

#[test]
fn a_milestone_card_names_only_the_metadata_its_part_brings() {
    let mut page = Fixture::new(None, false);
    page.populate(false);
    let watch = page.open_milestones();
    let own = milestone_line();
    page.live(watch, core::slice::from_ref(&own));
    page.serve_milestone(&own, &[], &[], false, None);
    for words in [f::browser_words(), f::words()] {
        let (tree, _) = page.tree(&words);
        for absent in ["0 goals", "whole", "so far", "spent", "0 / 0"] {
            assert!(!has(&tree, absent), "no {absent} before its part");
        }
        assert_eq!(styled(&tree, look::MS_STRIP).len(), 0);
    }
    let mut whole = own.clone();
    whole.parts = list(&[f::milestones::part(Some(NOW), milestone_totals(6, 0, 0), &[], 0)]);
    page.live(watch, core::slice::from_ref(&whole));
    let (tree, _) = page.tree(&f::browser_words());
    assert!(has(&tree, "6 goals · whole"));
    assert!(has(&tree, "212 spent of 400"));
    let mut so_far = own;
    so_far.parts = list(&[f::milestones::part(None, milestone_totals(6, 0, 0), &[], 0)]);
    page.live(watch, core::slice::from_ref(&so_far));
    let (tree, _) = page.tree(&f::browser_words());
    assert!(has(&tree, "6 goals so far"));
    so_far.budget.numbers.budget = 0;
    page.live(watch, &[so_far]);
    let (tree, _) = page.tree(&f::browser_words());
    assert!(!has(&tree, "spent"));
}

#[test]
fn a_placed_goal_uses_the_shared_live_row_while_the_milestones_read_catches_up() {
    let mut page = Fixture::new(None, false);
    page.populate(false);
    let mut goal = milestone_goal(3, b"C4 running");
    goal.delegates_total = 7;
    page.live(page.goals, &[goal]);
    let watch = page.open_milestones();
    let own = milestone_line();
    page.live(watch, core::slice::from_ref(&own));
    let old = ended_goal(3, b"Stale ended goal", c::ChipEnding::Done(c::ResultValue::Report(text(b"done"))), RECENT);
    page.serve_milestone(&own, &[], &[old], true, Some(&milestone_totals(1, 0, 0)));
    let (tree, _) = page.tree(&f::browser_words());
    assert_eq!(styled(&tree, look::WORK_CARD).len(), 1);
    assert!(has(&tree, "C4 running"));
    assert!(!has(&tree, "Stale ended goal"));
    assert_eq!(styled(&tree, look::WORK_ROW).len(), 1, "a live goal has one place, never an ended line too");
}

fn origin(task: u64, title: &[u8]) -> w::ChangeFrom {
    w::ChangeFrom { task, title: title.into() }
}

#[test]
fn work_cards_and_rows_cut_only_their_titles_in_both_shells() {
    for (title, cut) in f::row_titles() {
        let mut page = Fixture::new(None, false);
        page.populate_named(true, Some(title));
        page.drafts(&[landed_draft(b"docs/design/draft/another.md", title)]);
        for words in [f::browser_words(), f::words()] {
            let (tree, bindings) = page.tree(&words);
            let mut title_quotes = 0_usize;
            for quote in styled(&tree, look::WORK_QUOTE) {
                if quote.name.as_deref() == Some(cut) {
                    title_quotes += 1;
                }
            }
            assert_eq!(title_quotes, 2, "the chat draft and landed draft titles");
            for task in [GOAL, CHAT, QUEUED_GOAL, QUEUED_CHAT, INDEPENDENT] {
                let link = title_link(
                    &tree,
                    &bindings,
                    if task == CHAT { Address::Chat { task } } else { Address::Task { task } },
                );
                assert!(skein_lib::bytes::find(link.name.as_deref().unwrap(), cut).is_some());
                assert!(skein_lib::bytes::find(&subtree_text(&tree, link), cut).is_some());
            }
            if title != cut {
                assert!(skein_lib::bytes::find(&f::text(&tree), title).is_none());
            }
        }
    }
}

#[test]
fn milestone_groups_cut_their_from_and_goal_titles_in_both_shells() {
    for (title, cut) in f::row_titles() {
        let mut page = Fixture::new(None, false);
        page.populate(false);
        let running = milestone_goal(3, title);
        page.live(page.goals, core::slice::from_ref(&running));
        let watch = page.open_milestones();
        let mut own = milestone_line();
        own.chip.title = text(title);
        page.live(watch, core::slice::from_ref(&own));
        page.serve_milestone(&own, &[running], &[], true, Some(&milestone_totals(0, 0, 1)));
        for words in [f::browser_words(), f::words()] {
            let (tree, bindings) = page.tree(&words);
            for task in [M, 3] {
                let link = title_link(&tree, &bindings, Address::Task { task });
                assert!(skein_lib::bytes::find(link.name.as_deref().unwrap(), cut).is_some());
                assert!(skein_lib::bytes::find(&subtree_text(&tree, link), cut).is_some());
            }
        }
    }
}

#[test]
fn changes_shows_catching_up_below_the_lede_and_removes_it() {
    let mut page = Fixture::new(None, false);
    page.populate(false);
    let temper_client_domain::pages::changes::State::Shown(panel) = page.root.pages().changes().unwrap().state() else {
        panic!("changes shown");
    };
    let mut panel = panel.clone();
    let zero =
        w::CatchingUp { fresh: 0, gathers: 0, ci: 0, goal_files: 0, lost_writers: 0, goal_branches: 0, uncertain: 0 };
    for (state, expected) in [
        (
            Some(w::CatchingUp { fresh: 3, uncertain: 2, ..zero.clone() }),
            Some("Catching up: 3 fresh reads, 2 uncertain writes"),
        ),
        (Some(w::CatchingUp { goal_branches: 1, ..zero.clone() }), Some("Catching up: 1 goal-branch read")),
        (Some(zero), Some("Catching up")),
        (None, None),
    ] {
        panel.catching_up = state;
        page.up(Up::FamilyDelivered {
            watch: page.changes,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::changes_panel(&panel),
            },
        });
        let temper_client_domain::pages::changes::State::Shown(shown) = page.root.pages().changes().unwrap().state()
        else {
            panic!("still shown")
        };
        assert_eq!(shown.catching_up, panel.catching_up, "new delivery reaches page");
        for words in [f::browser_words(), f::words()] {
            let mut b = Builder::new(&f::tree_limits());
            crate::pages::changes::main(
                &mut b,
                &page.root,
                &words,
                &mut Bindings::new(f::tree_limits().bindings),
                f::PROJECT,
                &mut Times::new(NOW),
            );
            let tree = b.finish();
            let mut line = None;
            for (index, node) in tree.nodes().iter().enumerate() {
                let text = f::node_text(&tree, index);
                if node.role == Role::Paragraph && text.starts_with(b"Catching up") {
                    assert!(line.is_none(), "one progress line");
                    assert!(node.styles.as_slice().contains(&look::HOME_QUIET));
                    line = Some(text);
                }
            }
            let expected_bytes = match expected {
                Some(expected) => Some(expected.as_bytes()),
                None => None,
            };
            assert_eq!(line.as_deref(), expected_bytes);
            if let Some(expected) = expected {
                let text = f::text(&tree);
                let lede = skein_lib::bytes::find(&text, b"before it lands.").unwrap();
                assert!(lede < skein_lib::bytes::find(&text, expected.as_bytes()).unwrap());
            }
        }
    }
}

#[test]
fn changes_rows_cut_the_change_and_origin_goal_titles_in_both_shells() {
    for (title, cut) in f::row_titles() {
        let mut page = Fixture::new(None, false);
        page.populate_named(true, Some(title));
        for words in [f::browser_words(), f::words()] {
            let mut b = Builder::new(&f::tree_limits());
            let mut bindings = Bindings::new(f::tree_limits().bindings);
            crate::pages::changes::main(&mut b, &page.root, &words, &mut bindings, f::PROJECT, &mut Times::new(NOW));
            let tree = b.finish();
            let link = title_link(&tree, &bindings, Address::Task { task: QUEUED_GOAL });
            assert_eq!(skein_lib::bytes::count(link.name.as_deref().unwrap(), cut, 3), 2);
            assert_eq!(skein_lib::bytes::count(&subtree_text(&tree, link), cut, 3), 2);
        }
    }
}

fn title_link<'a>(tree: &'a Tree, bindings: &Bindings, address: Address) -> &'a skein_ui::Node {
    for node in tree.nodes() {
        if node.role == Role::Link
            && let Some(binding) = node.binding
            && bindings.decode(binding.token, skein_ui::Act::Press)
                == Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                    address: address.clone(),
                }))
        {
            return node;
        }
    }
    panic!("missing title link {address:?}");
}

// Node and binding numbers change when the head gains filters; the section's tree does not.
fn section_trees(tree: &Tree) -> List<List<skein_ui::Node>> {
    let mut sections = List::uncapped();
    for (index, section) in tree.nodes().iter().enumerate() {
        if !section.styles.as_slice().contains(&look::WORK_SECTION) {
            continue;
        }
        let mut nodes = List::uncapped();
        for node in tree.nodes().iter().skip(index).take(usize::try_from(section.size).unwrap()) {
            let mut node = node.clone();
            node.id.0 = 0;
            node.link = None;
            node.binding = None;
            nodes.push(node).unwrap();
        }
        sections.push(nodes).unwrap();
    }
    sections
}

fn answer_rank(page: &Fixture, reads: &[(Token, c::Read)], up: &Up) -> u8 {
    if let Up::FamilyDelivered { watch, .. } = up {
        if *watch == page.changes {
            return 0;
        }
        if *watch == page.drafts {
            return 5;
        }
    }
    if let Up::Watched { watch, .. } = up {
        if *watch == page.chats {
            return 1;
        }
        if *watch == page.goals {
            return 2;
        }
        if Some(*watch) == page.milestones {
            return 4;
        }
    }
    if let Up::ReadDone { read, .. } = up {
        for (token, what) in reads {
            if token != read {
                continue;
            }
            if let c::Read::Inbox { .. } = what {
                return 3;
            }
            if let c::Read::Tasks { query, .. } = what {
                return match query.label {
                    Some(7) => 1,
                    Some(8) => 2,
                    Some(9) => 4,
                    other => panic!("unexpected list {other:?}"),
                };
            }
        }
    }
    panic!("unexpected answer {up:?}");
}

#[test]
fn sections_draw_top_down_without_changing_earlier_sections_in_both_shells() {
    let mut source = Fixture::new(None, false);
    let milestones = source.open_milestones();
    source.answers.clear();
    let mut reads = List::uncapped();
    for request in &source.out {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
        } = request
        {
            reads.push((*read, what.clone())).unwrap();
        }
    }
    source.populate(true);
    source.live(milestones, &[]);
    let mut answers = List::uncapped();
    for up in &source.answers {
        answers.push((answer_rank(&source, reads.as_slice(), up), up.clone())).unwrap();
    }
    let words = [f::browser_words(), f::words()];
    for last in 0_u8..6 {
        let mut page = Fixture::new(None, false);
        page.open_milestones();
        let mut sorted = skein_lib::Map::uncapped();
        for (index, (rank, up)) in answers.iter().enumerate() {
            sorted.insert((*rank == last, *rank, index), (*rank, up.clone())).unwrap();
        }
        let mut order = List::uncapped();
        for (_, answer) in &sorted {
            order.push(answer.clone()).unwrap();
        }
        let mut before = words.each_ref().map(|words| {
            let (initial, _) = page.tree(words);
            let held = styled(&initial, look::WORK_HEADLINE);
            assert_eq!(held.len(), usize::from(words.shell == crate::words::Shell::Browser));
            if let Some(held) = held.first() {
                assert!(held.name.is_none());
                assert!(held.text.is_empty());
                assert_eq!(held.role, Role::Heading { level: 1 });
            }
            assert!(has(&initial, "Shop · Work"));
            assert!(has(&initial, "Everything on Shop"));
            assert_eq!(styled(&initial, look::WORK_CHIP).len(), 2, "All and Waits");
            section_trees(&initial)
        });
        for (index, (rank, up)) in order.iter().enumerate() {
            page.up(up.clone());
            for (words, before) in words.iter().zip(&mut before) {
                let (tree, _) = page.tree(words);
                let after = section_trees(&tree);
                assert!(after.as_slice().starts_with(before.as_slice()), "last={last}, answer={index}, rank={rank}");
                if last == 5
                    && *rank == 4
                    && let Some((5, _)) = order.as_slice().get(index + 1)
                {
                    assert_eq!(after.len(), 1);
                    assert!(has(&tree, "In flight · 3"));
                    assert!(has(&tree, "Witness retention"));
                    assert!(!has(&tree, "Drafts ·"));
                    assert!(!has(&tree, "Landed this week ·"));
                    assert!(!has(&tree, "Three in flight,"));
                    assert_eq!(styled(&tree, look::WORK_CHIP).len(), 2, "All and Waits");
                }
                *before = after;
            }
        }
        for (words, before) in words.iter().zip(&before) {
            let (tree, _) = page.tree(words);
            assert_eq!(before.len(), 3);
            assert!(has(&tree, "Three in flight, one draft, one landed this week."));
            assert!(has(&tree, "Goals · 1"));
        }
    }
}

#[test]
fn drafts_filter_waits_only_for_chats_and_drafts_before_claiming_none() {
    for words in [f::browser_words(), f::words()] {
        let mut page = Fixture::new(None, false);
        page.go(Address::Work { project: f::PROJECT, filter: WorkFilter::Drafts });
        assert!(!has(&page.tree(&words).0, "None."));
        page.drafts(&[]);
        page.live(page.chats, &[]);
        assert!(!has(&page.tree(&words).0, "None."), "chats' ended read still unanswered");
        for (read, what) in page.reads() {
            if let c::Read::Tasks { query, .. } = what
                && query.label == Some(7)
            {
                page.answer(read, ended_page(&[]));
            }
        }
        let (tree, _) = page.tree(&words);
        assert!(has(&tree, "None."));
        assert!(!has(&tree, "Nothing in flight yet."));
        assert_eq!(styled(&tree, look::WORK_CHIP).len(), 3, "All, current Drafts, and Waits");
    }
}
