//! The Waits page through the same linked engine double as Work.
use super::{BUDGET, Budget, Engine, NOW, PROJECT, Project, contains, env, fixture, kit, page_requests, take, w};
use jig_client::{Message, Request as WireRequest};
use jig_client_protocol_wire::Record as _;
use jig_client_test_records as records;
use skein_lib::{List, Queue, Reader};
use skein_ui::{Act, Role, State};
use temper_client_domain::{self as domain, Address, Event, Place, Request, WaitsFilter, WorkFilter};
use temper_client_view::{self as view, View, look, words::Words};
use temper_client_view_kit::engine::owned;

const BLOCKER: u64 = 91;
const SIGNING: u64 = 61;
const HEARER: u64 = 812;
const QUESTION: u64 = 3;
const HOUR: u64 = 3_600_000_000_000;
const CURSOR: [u8; 3] = [8, 0, 255];
const HEADING: Role = Role::Heading { level: 1 };
const FIRST_LINE: &str =
    "Blocker · question 3 at you · waiting 3 h · overdue for 1 h · then task 812 hears · Party · Overdue";

struct Page {
    root: domain::Domain,
    view: View,
    engine: Engine,
    frame: Project,
    out: Queue<Request>,
}

impl Page {
    fn start(address: Address, words: Words) -> Self {
        let limits = fixture::limits();
        let view_limits = view::Limits::of(&limits, &words).expect("stock view limits");
        let mut page = Self {
            root: domain::new(&limits, kit::SEED),
            view: View::new(&view_limits, &limits, words, kit::FORGE),
            engine: Engine::new(fixture::me()),
            frame: Project::new(false),
            out: Queue::uncapped(),
        };
        page.event(Event::Start { address, kept: List::with_capacity(0), with: Some(jig_client::With::Transport) });
        page
    }

    fn event(&mut self, event: Event) {
        domain::step(&mut self.root, &env(), event, &mut self.out);
        self.settle();
    }

    fn settle(&mut self) {
        domain::settle(&mut self.root, &env(), &mut self.out);
        domain::take(&mut self.root, &env(), &mut self.out);
        view::build(&mut self.view, &self.root, NOW, &mut Queue::uncapped());
    }

    fn wave(&mut self, sent: &[WireRequest], value: &w::WaitPage) {
        for event in self.engine.wave(sent, |subject, watch| {
            if !watch && let Some(w::Read::Waits(_)) = read(subject) {
                owned(w::ReadValue::Waits(value.clone()).encode(&w::CEILINGS).expect("waits answer"), None)
            } else if watch
                && subject.owner == w::Owner::Jig
                && let w::Subject::Tasks(tasks) =
                    w::Subject::decode(&w::CEILINGS, &mut Reader::new(&subject.record)).expect("subject")
                && tasks.window == 1
            {
                // The rail probes a single row when it cannot borrow Work's lists.
                let value = w::DeliveryValue::Tasks(w::Tasks { rows: Box::new([]), more: false, ended_mark: 0 });
                owned(value.encode(&w::CEILINGS).expect("rail's probe"), None)
            } else {
                self.frame.answer(subject, watch)
            }
        }) {
            domain::step(&mut self.root, &env(), Event::Client { event }, &mut self.out);
        }
        self.settle();
    }

    fn stream(&mut self) -> Vec<WireRequest> {
        let sent = take(&mut self.out);
        self.wave(&sent, &waits(&[], false));
        sent
    }

    fn answer(&mut self, value: &w::WaitPage) -> Vec<WireRequest> {
        let sent = take(&mut self.out);
        self.wave(&sent, value);
        sent
    }

    fn press(&mut self, role: Role, label: &[u8]) {
        let id = kit::find(self.view.tree(), role, label).expect("control is drawn");
        let event = view::act_event(&mut self.view, id, Act::Press).expect("control acts");
        self.event(event);
    }

    fn follow(&mut self, label: &[u8]) {
        self.press(Role::Link, label);
        let Some(Request::Show { address, replace: false }) = self.out.pop() else { panic!("link moves") };
        self.event(Event::Went { place: Place::Address(address) });
    }
}

fn address(filter: WaitsFilter) -> Address {
    Address::Waits { project: PROJECT, filter }
}

fn open(filter: WaitsFilter, words: Words, value: &w::WaitPage) -> Page {
    let mut page = Page::start(address(filter), words);
    page.stream();
    page.answer(value);
    page
}

fn read(subject: &w::Owned) -> Option<w::Read> {
    (subject.owner == w::Owner::Jig)
        .then(|| w::Read::decode(&w::CEILINGS, &mut Reader::new(&subject.record)).expect("read"))
}

fn wait_reads(sent: &[WireRequest]) -> Vec<w::ReadWaits> {
    let mut reads = Vec::new();
    for request in sent {
        if let WireRequest::Exchange { message, .. } = request {
            match message {
                Message::Batch(body) => {
                    let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(body)).expect("batch");
                    for member in &batch.members {
                        if let w::Member::Read(subject) = member
                            && let Some(w::Read::Waits(waits)) = read(subject)
                        {
                            reads.push(waits);
                        }
                    }
                }
                Message::Read(body) => {
                    let subject = w::Owned::decode(&w::CEILINGS, &mut Reader::new(body)).expect("owned read");
                    if let Some(w::Read::Waits(waits)) = read(&subject) {
                        reads.push(waits);
                    }
                }
                _ => {}
            }
        }
    }
    reads
}

fn cursor() -> w::Cursor {
    w::Cursor { value: Box::new(CURSOR) }
}

fn first_read() -> w::ReadWaits {
    w::ReadWaits { project: PROJECT, kind: None, party: None, due: None, after: None }
}

fn waits(rows: &[w::WaitRow], more: bool) -> w::WaitPage {
    w::WaitPage { now: NOW.as_nanos(), rows: List::from_boxed(rows.into()), after: more.then(cursor) }
}

fn chip(task: u64, title: &[u8]) -> w::Chip {
    let w::Fact::Chip(mut row) = records::chip_root_fact() else { panic!("chip fact") };
    row.value.task = task;
    row.value.title = records::wire_text(title);
    let mut details = records::wire_owner_chip_details_row();
    details.value.project = PROJECT;
    let phase = records::owner_phase_records().into_iter().next().expect("waiting phase");
    records::chip_from_rows(&[
        w::Fact::Chip(row),
        w::Fact::ChipPhase(w::ChipPhaseRow { parent: Some(0), value: w::ChipPhaseOwnerFields { phase, details: 2 } }),
        w::Fact::ChipDetails(Box::new(details)),
    ])
}

fn blocker() -> w::WaitRow {
    w::WaitRow {
        chip: chip(BLOCKER, b"Blocker"),
        wait: w::Wait {
            task: BLOCKER,
            kind: w::WaitKind::Party,
            object: w::WaitObject::Entry(w::WaitEntry {
                kind: w::WaitedFor::Question,
                number: QUESTION,
                at: w::EscalationHolder::Person(w::EscalationHolderPerson { value: fixture::PARTY }),
            }),
            since: NOW.as_nanos().checked_sub(HOUR.checked_mul(3).expect("three hours")).expect("earlier"),
            bound: w::WaitBound::At(w::WaitAtBound {
                at: NOW.as_nanos().checked_sub(HOUR).expect("earlier"),
                then: w::AtBound::Overdue,
            }),
            hears: Some(w::EscalationHolder::Task(w::EscalationHolderTask { value: HEARER })),
        },
    }
}

fn signing() -> w::WaitRow {
    w::WaitRow {
        chip: chip(SIGNING, b""),
        wait: w::Wait {
            task: SIGNING,
            kind: w::WaitKind::SignOff,
            object: w::WaitObject::Procedure(w::WaitProcedure { step: 7, things: List::with_capacity(0) }),
            since: NOW.as_nanos(),
            bound: w::WaitBound::At(w::WaitAtBound {
                at: NOW.as_nanos().checked_add(HOUR).expect("later"),
                then: w::AtBound::Overdue,
            }),
            hears: None,
        },
    }
}

fn rows(tree: &skein_ui::Tree) -> Vec<Box<[u8]>> {
    tree.nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| n.styles.as_slice().contains(&look::WAIT_ROW))
        .map(|(at, _)| kit::node_text(tree, at))
        .collect()
}

#[test]
fn cold_waits_open_is_whole_within_works_budget() {
    let mut page = Page::start(address(WaitsFilter::ALL), kit::browser_words());
    let stream = page.stream();
    let batch = page.answer(&waits(&[blocker()], false));
    let cost = Budget { waves: 2, requests: page_requests(&stream).checked_add(page_requests(&batch)).expect("cost") };
    let [WireRequest::OpenStream { .. }] = stream.as_slice() else { panic!("first wave opens the stream") };
    let [WireRequest::Exchange { message: Message::Batch(_), .. }] = batch.as_slice() else {
        panic!("second wave is the page and frame batch")
    };
    assert_eq!(cost, BUDGET);
    assert_eq!(wait_reads(&batch), [first_read()]);
    assert!(page.out.is_empty(), "no follow-up exchange");
    let tree = page.view.tree();
    assert!(kit::find(tree, HEADING, b"One wait").is_some());
    assert!(kit::has_text_run(tree, "Shop · Work"));
    assert!(kit::has_text_run(tree, "overdue first, then by bound"));
    assert_eq!(rows(tree), [Box::from(FIRST_LINE.as_bytes())]);
}

#[test]
fn wait_rows_link_tasks_and_keep_the_reads_order() {
    let mut page = open(WaitsFilter::ALL, kit::browser_words(), &waits(&[blocker(), signing()], false));
    let tree = page.view.tree();
    let drawn = rows(tree);
    assert_eq!(drawn.len(), 2, "two waits in the answer");
    assert_eq!(drawn[0].as_ref(), FIRST_LINE.as_bytes());
    assert!(drawn[1].starts_with(b"T61"));
    assert!(kit::find(tree, HEADING, b"Two waits").is_some());
    let mark = kit::find(tree, Role::Text, b"Overdue").expect("overdue mark");
    assert_eq!(tree.find(mark).expect("mark").styles.as_slice(), &[look::WAIT_OVERDUE]);
    for (label, path) in [(b"Blocker".as_slice(), b"/tasks/91".as_slice()), (b"T61", b"/tasks/61")] {
        let id = kit::find(page.view.tree(), Role::Link, label).expect("task's title is a link");
        assert_eq!(view::href(&mut page.view, id).as_deref(), Some(path));
    }
}

#[test]
fn empty_waits_say_whether_a_filter_was_applied() {
    for (filter, empty) in [
        (WaitsFilter::ALL, "Nothing waits on Shop."),
        (WaitsFilter { mine: true, ..WaitsFilter::ALL }, "No wait matches."),
    ] {
        let page = open(filter, kit::browser_words(), &waits(&[], false));
        let tree = page.view.tree();
        assert!(kit::find(tree, HEADING, b"Nothing waits.").is_some());
        assert!(kit::has_text_run(tree, empty));
    }
}

#[test]
fn more_waits_uses_the_cursor_and_appends_in_order() {
    let mut page = open(WaitsFilter::ALL, kit::browser_words(), &waits(&[blocker()], true));
    assert!(kit::find(page.view.tree(), HEADING, b"1+ waits").is_some());
    page.press(Role::Button, b"More waits");
    let sent = page.answer(&waits(&[signing()], false));
    assert_eq!(wait_reads(&sent), [w::ReadWaits { after: Some(cursor()), ..first_read() }]);
    let tree = page.view.tree();
    let drawn = rows(tree);
    assert_eq!(drawn.len(), 2, "both pages' rows");
    assert!(drawn[0].starts_with(b"Blocker"));
    assert!(drawn[1].starts_with(b"T61"));
    assert!(kit::find(tree, Role::Button, b"More waits").is_none());
}

#[test]
fn refresh_reads_from_the_start_and_replaces_the_rows() {
    let mut page = open(WaitsFilter::ALL, kit::browser_words(), &waits(&[blocker()], true));
    page.press(Role::Button, b"Refresh");
    let sent = page.answer(&waits(&[signing()], false));
    assert_eq!(wait_reads(&sent), [first_read()]);
    let drawn = rows(page.view.tree());
    assert_eq!(drawn.len(), 1, "refresh replaces the fetched rows");
    assert!(drawn[0].starts_with(b"T61"));
}

#[test]
fn waits_filter_links_send_the_address_filters() {
    let mut page = open(WaitsFilter::ALL, kit::browser_words(), &waits(&[], false));
    let sign_off = w::ReadWaits { kind: Some(w::WaitKind::SignOff), ..first_read() };
    for (label, expected, response) in [
        (b"Mine".as_slice(), w::ReadWaits { party: Some(fixture::PARTY), ..first_read() }, waits(&[], false)),
        (b"Due within an hour", w::ReadWaits { due: Some(HOUR), ..first_read() }, waits(&[signing()], false)),
        (b"Sign-off", sign_off, waits(&[], false)),
        (b"All", first_read(), waits(&[], false)),
    ] {
        page.follow(label);
        let sent = page.answer(&response);
        assert_eq!(wait_reads(&sent), [expected]);
        let tree = page.view.tree();
        let chip = kit::find(tree, Role::Link, label).expect("the answer's filter chip");
        assert!(tree.find(chip).expect("filter link").states.as_slice().contains(&State::Current));
    }
}

#[test]
fn work_links_to_waits_even_before_its_rows_answer() {
    let mut page = Page::start(Address::Work { project: PROJECT, filter: WorkFilter::All }, kit::browser_words());
    page.stream();
    let id = kit::find(page.view.tree(), Role::Link, b"Waits").expect("Waits chip does not depend on reads");
    assert_eq!(view::href(&mut page.view, id).as_deref(), Some(b"/projects/1/waits".as_slice()));
    let sent = page.answer(&waits(&[], false));
    assert!(wait_reads(&sent).is_empty(), "Work's own batch reads no waits");
}

#[test]
fn terminal_prints_the_heading_filters_and_one_line_per_wait() {
    let page = open(WaitsFilter::ALL, kit::words(), &waits(&[blocker(), signing()], false));
    let mut print = skein_ui_text::Print::new(&page.view.limits().tree).expect("print limits");
    skein_ui_text::render(page.view.tree(), b"temper", None, &mut print);
    let text = print.text.as_slice();
    contains(text, "== Two waits == \n");
    for label in ["] All", "] Mine", "] Due within an hour"] {
        contains(text, label);
    }
    let printed = String::from_utf8_lossy(text);
    assert!(printed.lines().any(|line| line == FIRST_LINE));
    assert!(printed.lines().any(|line| line.trim_start().starts_with("T61 · ") && line.ends_with(" · Sign-off")));
}

#[test]
fn an_unanswered_waits_read_leaves_the_heading_held_and_offers_refresh() {
    for (why, alert) in [
        (w::ReadNone::Busy, "The engine is busy. Refresh to try again."),
        (w::ReadNone::Refused, "The waits read was refused."),
    ] {
        let mut page = open(WaitsFilter::ALL, kit::browser_words(), &waits(&[blocker()], false));
        page.press(Role::Button, b"Refresh");
        let sent = take(&mut page.out);
        let [WireRequest::Exchange { exchange, message: Message::Read(_), .. }] = sent.as_slice() else {
            panic!("Refresh sends a read: {sent:?}")
        };
        let answer = w::ReadAnswer::None(w::ReadAnswerNone { value: why });
        page.event(Event::Client {
            event: jig_client::Event::Reply {
                exchange: *exchange,
                message: answer.encode(&w::CEILINGS).expect("read refusal fits"),
            },
        });
        let tree = page.view.tree();
        assert!(tree.nodes().iter().any(|node| node.role == HEADING && node.name.is_none()));
        assert!(kit::find(tree, Role::Alert, alert.as_bytes()).is_some());
        assert!(kit::find(tree, Role::Button, b"Refresh").is_some());
    }
}
