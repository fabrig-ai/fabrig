//! Work's three read lists, five filters and both shells (client/pages/README.md).
use super::{chats, fixtures as f, goals};
use crate::{
    frame::Bindings,
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
    pages::work::{Card, Item, Landed, Work},
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
    // The small whole project opens three lists, reports and provenance together.
    limits.client.domain.handles = 64;
    limits.client.domain.reads = 32;
    limits.client.exchanges = 32;
    limits
}
fn note(task: u64, description: &[u8]) -> c::Note {
    c::Note {
        number: task,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: f::PROJECT, task },
        description: text(description),
        body: text(b""),
        writer: c::NoteAuthor::Task { task, attempt: 1 },
        tasks: List::uncapped(),
        changed: Some(Wall::from_nanos(RECENT)),
        acts: List::uncapped(),
        cut: false,
    }
}
fn notes(rows: &[c::Note]) -> c::ReadValue {
    c::ReadValue::Notes(Box::new(c::NotePage { rows: list(rows), before: None, after: None, cut: false }))
}
struct Fixture {
    root: Domain,
    out: Queue<Request>,
    goals: Token,
    chats: Token,
    changes: Token,
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
        let mut goal = goals::template(jig_client::shapes::GOAL, b"goal", true);
        goal.label = 8;
        let mut chat = goals::template(jig_client::shapes::CHAT, b"chat", false);
        chat.label = 7;
        frame.projects.get_mut(0).unwrap().templates = list(&[chat, goal]);
        f::deliver_me(&mut root, me, frame);
        let mut fixture =
            Self { root, out: Queue::uncapped(), goals: Token::new(0), chats: Token::new(0), changes: Token::new(0) };
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
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows: list(rows), more: false }))),
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
    #[expect(clippy::too_many_lines, reason = "one fixture answers three lists and their per-row provenance")]
    fn populate(&mut self, full: bool) {
        let mut goal = line(GOAL, b"membership-handoff");
        goal.label = 8;
        goal.tracked = Some(1);
        goal.delegates_done = 6;
        goal.delegates_total = 9;
        goal.budget.numbers.spent = 31;
        goal.budget.numbers.budget = 50;
        self.live(self.goals, if full { core::slice::from_ref(&goal) } else { &[] });
        let chat_lines = [line(CHAT, b"The comment is stale"), line(DRAFT, b"Read barriers")];
        self.live(self.chats, if full { &chat_lines } else { &[] });
        let draft_notes = [note(DRAFT, b"Read barriers")];
        for (read, what) in self.reads() {
            if let c::Read::Tasks { .. } = what {
                self.answer(
                    read,
                    c::ReadValue::Tasks(Box::new(c::TaskPage {
                        live: List::uncapped(),
                        live_after: None,
                        ended: c::EndedGroup::None,
                    })),
                );
            } else if let c::Read::Notes { scope: c::NoteScope::Tasks { .. }, .. } = what {
                self.answer(read, notes(if full { &draft_notes } else { &[] }));
            } else if let c::Read::Notes { scope: c::NoteScope::Task { task: GOAL, .. }, .. } = what {
                self.answer(read, notes(&[note(GOAL, b"report: Six of nine tasks landed.")]));
            }
        }
        let mut queued = List::with_capacity(3);
        if full {
            for (task, title) in [
                (QUEUED_GOAL, b"Witness retention".as_slice()),
                (QUEUED_CHAT, b"Comment rewritten"),
                (INDEPENDENT, b"Witness rejoin test"),
            ] {
                queued
                    .push(
                        w::ChangesQueued::new(
                            &w::CEILINGS,
                            w::ChangesQueuedParts {
                                task,
                                title: title.into(),
                                ready_since: RECENT,
                                checks_since: None,
                            },
                        )
                        .unwrap(),
                    )
                    .unwrap();
            }
        }
        let landed = if full {
            list(&[w::ChangesLanded::new(
                &w::CEILINGS,
                w::ChangesLandedParts {
                    commit: [0xab; 32],
                    default_branch: true,
                    task: LANDED,
                    title: b"Handoff states".as_slice().into(),
                    at: RECENT,
                },
            )
            .unwrap()])
        } else {
            List::with_capacity(0)
        };
        let panel = w::ChangesPanel::new(
            &w::CEILINGS,
            w::ChangesPanelParts {
                landed_default: landed.clone(),
                landed_default_more: 0,
                project: f::PROJECT,
                landing: b"main".as_slice().into(),
                queues: list(&[w::ChangesQueue::new(
                    &w::CEILINGS,
                    w::ChangesQueueParts {
                        way: w::Way::Queue,
                        in_flight: None,
                        repository: b"owner/repo".as_slice().into(),
                        branch: b"main".as_slice().into(),
                        red: full,
                        changes: queued,
                    },
                )
                .unwrap()]),
                more: 0,
                landed,
                landed_more: 0,
            },
        )
        .unwrap();
        self.up(Up::FamilyDelivered {
            watch: self.changes,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::changes_panel(&panel),
            },
        });
        for (read, what) in self.reads() {
            if let c::Read::Task { task } = what {
                let mut document = f::task::document();
                document.chip.task = task;
                document.goal = if task == QUEUED_GOAL || task == LANDED { Some(GOAL) } else { None };
                document.lineage_chips = List::with_capacity(1);
                if task == QUEUED_CHAT {
                    let mut chip = document.chip.clone();
                    chip.task = CHAT;
                    chip.executor = c::ChipExecutor::Agent { charter: 1 };
                    document.lineage_chips.push(chip).unwrap();
                } else if document.goal.is_some() {
                    let mut chip = document.chip.clone();
                    chip.task = GOAL;
                    chip.title = text(b"membership-handoff");
                    document.lineage_chips.push(chip).unwrap();
                }
                self.answer(read, c::ReadValue::Task(Box::new(document)));
            }
        }
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
    let branch = w::ChangesLanded::new(
        &w::CEILINGS,
        w::ChangesLandedParts {
            task: 21,
            title: b"Goal-branch implementation".as_slice().into(),
            at: RECENT,
            commit: [0x12; 32],
            default_branch: false,
        },
    )
    .unwrap();
    let landing = w::ChangesLanded::new(
        &w::CEILINGS,
        w::ChangesLandedParts {
            task: 22,
            title: b"Landing fallback".as_slice().into(),
            at: RECENT + 1,
            commit: [0xab; 32],
            default_branch: true,
        },
    )
    .unwrap();
    let panel = w::ChangesPanel::new(
        &w::CEILINGS,
        w::ChangesPanelParts {
            project: f::PROJECT,
            landing: b"main".as_slice().into(),
            queues: List::with_capacity(0),
            more: 0,
            landed: list(&[landing.clone(), branch]),
            landed_more: 0,
            landed_default: list(&[landing]),
            landed_default_more: 0,
        },
    )
    .unwrap();
    page.up(Up::FamilyDelivered {
        watch: page.changes,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::changes_panel(&panel),
        },
    });
    for (read, what) in page.reads() {
        if let c::Read::Task { task } = what {
            let mut document = f::task::document();
            document.chip.task = task;
            document.goal = Some(GOAL);
            let mut chip = document.chip.clone();
            chip.task = GOAL;
            chip.title = text(b"The whole goal");
            document.lineage_chips = list(&[chip]);
            page.answer(read, c::ReadValue::Task(Box::new(document)));
        }
    }
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

    // Chat and unread tasks stay change rows; a goal without its chip uses the change's title.
    let mut parts = panel.into_parts();
    let mut branches = List::with_capacity(2);
    for row in parts.landed.into_boxed() {
        let mut row = row.into_parts();
        row.default_branch = false;
        branches.push(w::ChangesLanded::new(&w::CEILINGS, row).unwrap()).unwrap();
    }
    parts.landed = branches;
    let mut defaults = List::with_capacity(3);
    for (task, title, commit) in [(30, "Chat landing", 0x32), (31, "Unread landing", 0x45), (32, "Goal fallback", 0x67)]
    {
        defaults
            .push(
                w::ChangesLanded::new(
                    &w::CEILINGS,
                    w::ChangesLandedParts {
                        task,
                        title: title.as_bytes().into(),
                        at: RECENT,
                        commit: [commit; 32],
                        default_branch: true,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    parts.landed_default = defaults;
    let panel = w::ChangesPanel::new(&w::CEILINGS, parts).unwrap();
    page.up(Up::FamilyDelivered {
        watch: page.changes,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::changes_panel(&panel),
        },
    });
    for (read, what) in page.reads() {
        if let c::Read::Task { task } = what {
            if task == 31 {
                continue;
            }
            let mut document = f::task::document();
            document.chip.task = task;
            document.goal = if task == 32 { Some(GOAL) } else { None };
            document.lineage_chips = List::uncapped();
            if task == 30 {
                let mut chip = document.chip.clone();
                chip.task = CHAT;
                chip.executor = c::ChipExecutor::Agent { charter: 1 };
                document.lineage_chips = list(&[chip]);
            }
            page.answer(read, c::ReadValue::Task(Box::new(document)));
        }
    }
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
    let card = Card { item: Item::Goal(GOAL), changes: List::uncapped() };
    let mut work = Work::default();
    for _ in 0..flight {
        work.in_flight.push(card.clone()).unwrap();
    }
    for _ in 0..drafts {
        work.drafts.push(card.clone()).unwrap();
    }
    for _ in 0..landed {
        work.landed.push(Landed { item: Item::Goal(GOAL), change: LANDED, commit: [0xab; 32], at: NOW }).unwrap();
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
    assert_eq!(styled(&tree, look::WORK_CHIP).len(), 2);
    assert!(has(&tree, "Drafts · 0"));
    assert!(has(&tree, "None."));
}
#[test]
fn sections_wait_for_answers_omit_empty_and_draw_the_whole_small_project() {
    for words in [f::browser_words(), f::words()] {
        let mut page = Fixture::new(None, false);
        let (unread, _) = page.tree(&words);
        assert!(styled(&unread, look::WORK_HEADLINE).is_empty());
        assert!(styled(&unread, look::WORK_SECTION).is_empty());
        assert!(!has(&unread, "Nothing in flight yet."));
        page.populate(false);
        let (empty, _) = page.tree(&words);
        assert!(has(&empty, "Nothing in flight yet."));
        assert!(styled(&empty, look::WORK_SECTION).is_empty());
        let mut page = Fixture::new(None, false);
        page.populate(true);
        let (tree, bindings) = page.tree(&words);
        assert_eq!(skein_ui::outline::check(&tree), None);
        for value in [
            "Shop · Work",
            "Three in flight, one draft, one landed this week.",
            "3 queued",
            "main red",
            "In flight · 3",
            "newest activity first",
            "goal",
            "membership-handoff",
            "Task by task",
            "31 of 50 spent",
            "Six of nine tasks landed.",
            "6 / 9",
            "Written by the goal at 07:54",
            "Witness retention",
            "checking",
            "Handoff states",
            "landed 07:54",
            "change",
            "Witness rejoin test",
            "queued 3rd",
            "chat",
            "with you",
            "The comment is stale",
            "Comment rewritten",
            "queued 2nd",
            "Drafts · 1",
            "a goal starts from each",
            "draft",
            "read-barriers.md",
            "a chat with you",
            "Read barriers",
            "Open the draft",
            "Landed this week · 1",
        ] {
            assert!(has(&tree, value), "missing {value}");
        }
        assert_eq!(styled(&tree, look::WORK_SECTION).len(), 3);
        let mut addresses = List::uncapped();
        for node in tree.nodes() {
            if let Some(binding) = node.link {
                addresses.push(bindings.decode(binding.token, skein_ui::Act::Press)).unwrap();
            }
        }
        for address in [
            Address::Task { task: GOAL },
            Address::Task { task: INDEPENDENT },
            Address::Chat { task: CHAT },
            Address::Chat { task: DRAFT },
            Address::Task { task: QUEUED_GOAL },
            Address::Task { task: QUEUED_CHAT },
            Address::Task { task: LANDED },
        ] {
            assert!(addresses.as_slice().contains(&Some(temper_client_domain::Action::Pages(
                temper_client_domain::pages::Act::Go { address }
            ))));
        }
        let view_limits = crate::Limits::of(&limits(), &words).unwrap();
        let mut full_view = crate::View::new(&view_limits, &limits(), words.clone(), f::FORGE);
        crate::build(&mut full_view, &page.root, NOW, &mut Queue::uncapped());
        assert!(has(full_view.tree(), "Three in flight, one draft, one landed this week."), "Work's root dispatch");
        assert_eq!(skein_ui::outline::check(full_view.tree()), None);
        page.go(Address::Work { project: f::PROJECT, filter: WorkFilter::Drafts });
        let (tree, _) = page.tree(&words);
        assert_eq!(styled(&tree, look::WORK_SECTION).len(), 1);
        assert_eq!(styled(&tree, look::WORK_CARD).len(), 1);
        assert!(has(&tree, "Open the draft"));
        assert!(!has(&tree, "In flight · 3"));
    }
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
        "[5] Chats · 1",
        "[6] goal membership-handoff",
        "Six of nine tasks landed.",
        "Witness retention",
        "landed 07:54",
    ] {
        assert!(skein_lib::bytes::find(printed, value.as_bytes()).is_some(), "missing {value}");
    }
}
