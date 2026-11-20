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

use temper_client_domain::{Address, Domain, Event, Place, Request, WorkFilter, forge::ways::Ways};

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

struct Fixture {
    root: Domain,
    out: Queue<Request>,
    goals: Token,
    me: Token,
    inbox: Token,
    chats: Token,
    changes: Token,
    drafts: Token,
}

impl Fixture {
    fn new(ways: Option<Ways>, refused: bool) -> Self {
        let (mut root, me, _, _, _, _, _, inbox, _) = f::root_at_with_values(
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
        let mut milestone = goal.clone();
        milestone.number = 1;
        milestone.name = text(b"milestone");
        milestone.flags.ended_by = c::EndedBy::Requester;
        frame.projects.get_mut(0).unwrap().templates = list(&[chat, milestone, goal]);
        f::deliver_me(&mut root, me, frame);
        let mut fixture = Self {
            root,
            out: Queue::uncapped(),
            goals: Token::new(0),
            me,
            inbox: inbox.expect("standing inbox watch"),
            chats: Token::new(0),
            changes: Token::new(0),
            drafts: Token::new(0),
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
        self.drafts(&[]);
        let mut goal = line(GOAL, b"membership-handoff");
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
        let mut draft = line(DRAFT, b"Read barriers");
        draft.parts = list(&[temper_fixtures::client_documents::chat_part(Some(b"Read barriers"))]);
        let chat_lines = [line(CHAT, b"The comment is stale"), draft];
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
        let mut queued = List::with_capacity(3);
        if full {
            for (task, title) in [
                (QUEUED_GOAL, b"Witness retention".as_slice()),
                (QUEUED_CHAT, b"Comment rewritten"),
                (INDEPENDENT, b"Witness rejoin test"),
            ] {
                queued
                    .push(w::ChangesQueued {
                        goal: if task == QUEUED_GOAL { Some(origin(GOAL)) } else { None },
                        chat: if task == QUEUED_CHAT { Some(origin(CHAT)) } else { None },
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
                goal: Some(origin(GOAL)),
                chat: None,
                milestone: Some(w::ChangeFrom { task: 100, title: b"Store shape".as_slice().into() }),
                signed: list(&[w::ChangesSigned {
                    party: 5,
                    name: Some(b"Sam".as_slice().into()),
                    head: [0; 32],
                    at: RECENT,
                }]),
                commit: [0xab; 32],
                default_branch: true,
                task: LANDED,
                title: b"Handoff states".as_slice().into(),
                at: RECENT,
            }])
        } else {
            List::with_capacity(0)
        };
        let panel = w::ChangesPanel {
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
fn sections_wait_for_answers_omit_empty_and_draw_the_whole_small_project() {
    for words in [f::browser_words(), f::words()] {
        let mut page = Fixture::new(None, false);
        let (unread, _) = page.tree(&words);
        assert_eq!(
            styled(&unread, look::WORK_HEADLINE).len(),
            usize::from(words.shell == crate::words::Shell::Browser)
        );
        assert!(!has(&unread, "Nothing in flight yet."));
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
            "signed by Sam",
            "in Store shape",
        ] {
            assert!(has(&tree, value), "missing {value}");
        }
        assert!(!has(&tree, "milestone"));
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

/// The same store answers again, with independent goal, milestone and needs turns.
struct TurningWork {
    page: Fixture,
    watch: Token,
    goals: List<c::TaskLine>,
    milestone: c::TaskLine,
}

impl TurningWork {
    fn new() -> Self {
        let mut page = Fixture::new(None, false);
        page.populate(false);
        let mut out = f::milestones::offer(&mut page.root, page.me, &limits());
        let watch = f::milestones::watch(&out, Some(f::milestones::LABEL), None, limits().client.domain.rows);
        while let Some(request) = out.pop() {
            page.out.push(request);
        }
        let mut goals = List::uncapped();
        let mut children = List::uncapped();
        for (task, title, headline) in [
            (3, b"K3 codec records".as_slice(), b"K3 lands the codec's shape".as_slice()),
            (4, b"K4 codec reader", b"K4 keeps old records readable"),
            (5, b"K5 codec writer", b"K5 writes the new shape"),
            (6, b"K6 codec checks", b"K6 pins round trips"),
        ] {
            let mut goal = line(task, title);
            goal.label = 8;
            goal.tracked = Some(1);
            goal.requester = c::Requester::Task(f::milestones::TASK);
            goal.parts = list(&[temper_fixtures::client_documents::goal_part(
                Some((headline, Wall::from_nanos(RECENT))),
                Some(f::milestones::TASK),
            )]);
            if task == 3 {
                goal.chip.phase = c::Phase::Active;
                goal.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
                goal.delegates_total = 1;
            }
            children.push((goal.clone(), Some(headline))).unwrap();
            goals.push(goal).unwrap();
        }
        for (task, title, headline, at) in [
            (1, b"Plain reader".as_slice(), b"The reader keeps its witness".as_slice(), Wall::from_nanos(RECENT)),
            (2, b"Plain writer", b"The writer keeps its witness", NOW),
        ] {
            let mut goal = line(task, title);
            goal.label = 8;
            goal.tracked = Some(1);
            goal.parts = list(&[temper_fixtures::client_documents::goal_part(Some((headline, at)), None)]);
            goals.push(goal).unwrap();
        }
        let mut milestone = line(f::milestones::TASK, b"Codec shape");
        milestone.label = f::milestones::LABEL;
        milestone.tracked = Some(1);
        let totals = w::MilestoneTotals {
            task: f::milestones::TASK,
            done: 2,
            failed: 0,
            closed: 0,
            live: 4,
            ended: None,
            last_landing: None,
            spent: 0,
        };
        milestone.parts = list(&[f::milestones::part(None, totals, children.as_slice(), 0)]);
        page.live(watch, core::slice::from_ref(&milestone));
        page.live(page.goals, goals.as_slice());
        let reads = page.reads();
        assert_eq!(reads.len(), 1, "the milestone's ended read opens");
        for (read, what) in reads {
            let c::Read::Tasks { query, .. } = what else { panic!("only the milestone's ended read opens") };
            assert_eq!(query.label, Some(f::milestones::LABEL));
            page.answer(
                read,
                c::ReadValue::Tasks(Box::new(c::TaskPage {
                    ended_mark: 0,
                    live: List::uncapped(),
                    live_after: None,
                    ended: c::EndedGroup::None,
                })),
            );
        }
        let mut story = Self { page, watch, goals, milestone };
        story.inbox();
        assert!(story.page.reads().is_empty(), "the first inbox delivery is no move");
        let read = story.needs_read();
        story.page.up(Up::ReadDone { read, result: Self::needs() });
        assert!(story.page.reads().is_empty(), "every list has answered");
        story
    }

    fn goals(&mut self, part: Option<c::RowPart>) {
        let mut rows = self.goals.clone().into_boxed();
        if let Some(part) = part {
            for row in &mut rows {
                row.parts = list(core::slice::from_ref(&part));
            }
        }
        self.page.live(self.page.goals, &rows);
    }

    fn milestone(&mut self, part: Option<c::RowPart>) {
        let mut row = self.milestone.clone();
        if let Some(part) = part {
            row.parts = list(&[part]);
        }
        self.page.live(self.watch, &[row]);
    }

    fn inbox(&mut self) {
        let inbox = temper_fixtures::client_documents::inbox(
            list(&[Self::need()]),
            1,
            c::Count { value: 0, limited: false },
            0,
            false,
        );
        self.page.up(Up::Watched {
            watch: self.page.inbox,
            event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(inbox))),
        });
    }

    fn needs_read(&mut self) -> Token {
        self.inbox();
        let reads = self.page.reads();
        assert_eq!(reads.len(), 1, "the inbox delivery asks the goals' needs again");
        let (read, what) = reads.first().unwrap();
        let c::Read::Inbox { group: c::InboxReadGroup::Needs, project: Some(f::PROJECT), .. } = what else {
            panic!("the project's goals' needs read")
        };
        *read
    }

    fn need() -> c::InboxEntry {
        temper_fixtures::client_documents::entry(
            c::ObjectKey::PersonTask(6),
            f::PROJECT,
            1,
            Wall::from_nanos(RECENT),
            None,
        )
    }

    fn needs() -> Readout {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(c::InboxPage {
            rows: list(&[Self::need()]),
            cut: false,
            before: None,
            after: None,
            selection: Some(list(&[c::InboxSelection {
                cursor: c::InboxCursor::Number { number: 1 },
                goals: list(&[6, f::milestones::TASK]),
            }])),
        }))))
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "the test compares UTF-8 text byte for byte with readable failure drawings"
    )]
    fn same(&mut self, words: &Words, first: &[u8], turn: &str) {
        let (tree, _) = self.page.tree(words);
        let text = f::text(&tree);
        assert_eq!(
            core::str::from_utf8(&text).unwrap(),
            core::str::from_utf8(first).unwrap(),
            "{turn}, {:?}",
            words.shell
        );
        assert!(self.page.reads().is_empty(), "{turn}: no unanswered follow-up read");
    }
}

#[test]
fn work_draws_the_same_through_partial_answers_in_turns() {
    for words in [f::browser_words(), f::words()] {
        let mut story = TurningWork::new();
        let (tree, _) = story.page.tree(&words);
        for value in [
            "In flight · 3",
            "Three in flight",
            "Codec shape",
            "6 goals so far",
            "2 of 6 goals done.",
            "K3 codec records",
            "K4 codec reader",
            "K5 codec writer",
            "K6 codec checks",
            "The reader keeps its witness",
            "The writer keeps its witness",
            "Written by the goal at 07:54",
            "Written by the goal at 08:00",
            "needs you",
        ] {
            assert!(has(&tree, value), "first drawing missing {value}");
        }
        assert_eq!(styled(&tree, look::WORK_CARD).len(), 3, "no separate K3–K6 cards");
        assert_eq!(styled(&tree, look::MS_CARD).len(), 1);
        assert_eq!(styled(&tree, look::WORK_ROW).len(), 4, "all four goal lines on the milestone");
        assert!(!has(&tree, "In flight · 7"));
        let first = f::text(&tree);
        let busy = c::RowPart::Failed(c::PartFailure::Busy);
        for part in [busy.clone(), c::RowPart::Unknown] {
            story.goals(Some(part));
            story.same(&words, &first, "goals' parts not made");
            story.goals(None);
            story.same(&words, &first, "goals' parts whole again");
        }
        story.milestone(Some(busy.clone()));
        story.same(&words, &first, "milestone's part busy");
        story.milestone(None);
        story.same(&words, &first, "milestone's part whole again");
        for result in [Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)), Readout::Limit, TurningWork::needs()] {
            let read = story.needs_read();
            story.same(&words, &first, "needs read pending");
            story.page.up(Up::ReadDone { read, result });
            story.same(&words, &first, "needs read answered");
        }
        for milestone_first in [true, false] {
            for (part, result) in [
                (busy.clone(), Readout::Jig(c::ReadResult::None(c::ReadNone::Busy))),
                (c::RowPart::Unknown, Readout::Limit),
            ] {
                let read = story.needs_read();
                story.same(&words, &first, "all pending");
                if milestone_first {
                    story.milestone(Some(part.clone()));
                    story.same(&words, &first, "milestone before goals");
                    story.goals(Some(part));
                } else {
                    story.goals(Some(part.clone()));
                    story.same(&words, &first, "goals before milestone");
                    story.milestone(Some(part));
                }
                story.same(&words, &first, "both watches' parts not made");
                story.page.up(Up::ReadDone { read, result });
                story.same(&words, &first, "all answers not made");
                let read = story.needs_read();
                story.same(&words, &first, "needs pending with both parts not made");
                story.page.up(Up::ReadDone { read, result: TurningWork::needs() });
                story.same(&words, &first, "needs whole before watches");
                if milestone_first {
                    story.goals(None);
                    story.same(&words, &first, "whole goals before milestone");
                    story.milestone(None);
                } else {
                    story.milestone(None);
                    story.same(&words, &first, "whole milestone before goals");
                    story.goals(None);
                }
                story.same(&words, &first, "all whole again");
            }
        }
    }
}

fn origin(task: u64) -> w::ChangeFrom {
    w::ChangeFrom { task, title: b"membership-handoff".as_slice().into() }
}
