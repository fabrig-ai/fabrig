//! Rows beyond the card window, including standing-role person questions.
use super::{c, d, f, has, look, main, styled};
use d::{
    Address, Event,
    pages::{self, needs},
};
use jig_client::{Down, Readable, Readout, Up, contract::Read};
use skein_lib::{Env, List, Queue, Time, Token, Wall};
use skein_ui::{Act, Role};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_fixtures::client_documents as documents;

const TOTAL: u64 = 29;
const STANDING: u64 = 55;
const GROUP: &str = "Start a goal for each?\n\nNone blocks a landing.\nGroup: bugs · in temper\nFindings: 1/2 1/8 1/10\nChoice: One goal for all\nChoice: A goal for each (recommended)";

fn limits() -> d::Limits {
    let mut limits = f::needs::limits();
    limits.client.domain.reads = 16;
    limits.client.domain.rows = 8;
    limits
}
fn event(root: &mut d::Domain, event: Event, out: &mut Queue<d::Request>) {
    d::step(root, &Env { now: Time::ZERO, wall: f::NOW, limits: limits() }, event, out);
}
fn object(number: u64) -> c::ObjectKey {
    if number <= 14 {
        c::ObjectKey::Question { task: number.checked_add(100).unwrap(), message: number }
    } else {
        c::ObjectKey::PersonTask(number.checked_add(100).unwrap())
    }
}
fn title(number: u64) -> Box<[u8]> {
    skein_lib::bytes::concat(&[b"Need ", skein_lib::Decimal::of(number).as_bytes()])
}
fn entries(start: u64, end: u64) -> List<c::InboxEntry> {
    let mut rows = List::uncapped();
    for number in start..end {
        let mut row = documents::entry(object(number), f::PROJECT, number, Wall::from_nanos(number), None);
        let mut chip = f::task::document().chip;
        chip.task = number.checked_add(100).unwrap();
        chip.title = f::task::document_words(&title(number));
        row.task = if number == 29 {
            c::TaskLink::Stored { number: chip.task, status: None }
        } else if number == 28 {
            c::TaskLink::Retained { chip, at: None }
        } else {
            c::TaskLink::Live(chip)
        };
        rows.push(row).unwrap();
    }
    rows
}
fn question(task: u64, message: u64) -> c::ReadValue {
    let mut chip = f::task::document().chip;
    chip.task = task;
    chip.title = f::task::document_words(&title(message));
    c::ReadValue::Question(Box::new(c::Question {
        task: chip,
        from: None,
        message,
        words: f::needs::words(if message == 1 {
            "One medium finding?\nFindings: 1/10\nChoice: Yes"
        } else {
            "Choose?\nChoice: Yes\nChoice: No"
        }),
        since: Wall::from_nanos(message),
        acts: f::needs::offers(&[c::ActKind::Answer]),
        ending: None,
    }))
}
fn person(task: u64) -> c::ReadValue {
    let mut person = f::needs::person();
    person.task.task = task;
    person.task.title = f::task::document_words(&title(task.checked_sub(100).unwrap()));
    person.requester = c::Requester::Task(STANDING);
    person.spec.specification.words = f::needs::words(GROUP);
    person.contract = c::TaskContractDocument::Report { words: 4096 };
    person.since = Some(Wall::from_nanos(task.checked_sub(100).unwrap()));
    person.acts = f::needs::offers(&[c::ActKind::Answer]);
    c::ReadValue::PersonTask(Box::new(person))
}
fn answer(root: &mut d::Domain, read: Token, value: c::ReadValue, out: &mut Queue<d::Request>) {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) },
            },
        },
        out,
    );
}
#[expect(
    clippy::wildcard_enum_match_arm,
    clippy::too_many_lines,
    reason = "the fixture answers its independent needs reads and findings and decisions watches together"
)]
fn settle(root: &mut d::Domain, out: &mut Queue<d::Request>) -> u32 {
    let mut pages = 0_u32;
    while let Some(request) = out.pop() {
        match request {
            d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
            } => {
                let value = match what {
                    Read::Inbox { group: c::InboxReadGroup::Needs, after, .. } => {
                        let start = match after {
                            Some(cursor) => u64::from(*cursor.value.bytes().first().unwrap()),
                            None => 1,
                        };
                        let end = start.checked_add(8).unwrap().min(TOTAL + 1);
                        pages = pages.checked_add(1).unwrap();
                        c::ReadValue::Inbox(Box::new(c::InboxPage {
                            rows: entries(start, end),
                            before: None,
                            after: if end <= TOTAL {
                                Some(c::Cursor::new(Box::new([u8::try_from(end).unwrap()]), 1).unwrap())
                            } else {
                                None
                            },
                            cut: false,
                            selection: None,
                        }))
                    }
                    Read::Inbox { group: c::InboxReadGroup::Updates, .. } => {
                        c::ReadValue::Inbox(Box::new(c::InboxPage {
                            rows: List::uncapped(),
                            before: None,
                            after: None,
                            cut: false,
                            selection: None,
                        }))
                    }
                    Read::Question { task, message } => question(task, message),
                    Read::PersonTask { task } => person(task),
                    Read::Task { task } => {
                        let mut document = f::task::document();
                        document.chip.task = task;
                        document.chip.title =
                            f::needs::words(if task == STANDING { "Standing triager" } else { "Worker" });
                        document.lineage.clear();
                        document.lineage_chips.clear();
                        document.goal = None;
                        c::ReadValue::Task(Box::new(document))
                    }
                    _ => continue,
                };
                answer(root, read, value, out);
            }
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } },
                    },
            } => {
                let Some(watching) = wire::decode_watching(&record.bytes, &w::CEILINGS) else { continue };
                let bytes = match &watching.watch {
                    w::Watch::Decided(subject) => wire::decisions(&w::Decisions {
                        project: subject.project,
                        rows: List::with_capacity(0),
                        asked: List::with_capacity(0),
                        cut: false,
                    }),
                    w::Watch::Findings(subject) => wire::findings(&w::Findings {
                        project: subject.project,
                        rows: List::from_boxed(if subject.project == f::PROJECT && !subject.mine {
                            f::groups::bugs()
                        } else {
                            Box::default()
                        }),
                        areas: List::with_capacity(0),
                        triager: None,
                        holder: w::Holder::Party(w::HolderParty { number: 1 }),
                        cut: false,
                    }),
                    _ => continue,
                };
                event(
                    root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes,
                                },
                            },
                        },
                    },
                    out,
                );
            }
            _ => {}
        }
    }
    pages
}
fn paged() -> d::Domain {
    let (mut root, _, _, _, _, _, _, standing, _) = f::root_at_with_classes(
        Address::Settings { project: f::PROJECT },
        &limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        Some(d::forge::ways::Ways::STOCK),
        true,
        true,
        None,
    );
    let mut out = Queue::uncapped();
    event(
        &mut root,
        Event::Went { place: d::Place::Address(Address::Inbox { filter: d::address::Filter::ALL }) },
        &mut out,
    );
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched {
                    watch: standing.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        entries(1, 9),
                        u32::try_from(TOTAL).unwrap(),
                        c::Count { value: 0, limited: false },
                        0,
                        true,
                    )))),
                },
            },
        },
        &mut out,
    );
    assert_eq!(settle(&mut root, &mut out), 4);
    let inbox = root.pages().inbox().unwrap();
    assert_eq!(inbox.cards.len(), 8);
    assert_eq!(inbox.rows.len(), 29);
    root
}
fn view(root: &d::Domain, browser: bool) -> crate::View {
    let words = if browser { f::browser_words() } else { f::words() };
    let domain = limits();
    let envelope = crate::Limits::of(&domain, &words).unwrap();
    let mut view = crate::View::new(&envelope, &domain, words, f::FORGE);
    crate::build(&mut view, root, f::NOW, &mut Queue::with_capacity(envelope.tree.patches));
    view
}
fn rows(view: &crate::View) -> Vec<&skein_ui::Node> {
    main(view).iter().filter(|n| n.styles.as_slice().contains(&look::NEED_ROW)).collect()
}

#[test]
fn every_paged_need_has_a_row_and_the_terminal_keeps_its_task_link() {
    let root = paged();
    let view = view(&root, true);
    assert!(has(&view, "Needs you · 29".as_bytes()));
    assert_eq!(rows(&view).len(), 28);
    let all = root.pages().needs().all(root.pages().inbox().unwrap(), root.forge().approvals());
    for (index, need) in all.iter().enumerate() {
        let number = if index == 28 { 1 } else { u64::try_from(index).unwrap() + 2 };
        let expected = object(number);
        let actual = match need {
            pages::findings::Need::Existing(need) => need.object(),
            pages::findings::Need::Row(row) => row.card_key(),
            pages::findings::Need::Finding(_) => panic!("no person's findings"),
        };
        assert_eq!(actual, expected);
    }
    assert_eq!(all.len(), 29);
    let closed = rows(&view);
    for (index, row) in closed.iter().enumerate().skip(6).take(21) {
        let text = f::nodes_text(super::super::inbox::subtree(
            view.tree(),
            view.tree().nodes().iter().position(|n| n.id == row.id).unwrap(),
        ));
        let number = u64::try_from(index).unwrap() + 3;
        let title = title(number);
        assert!(
            skein_lib::bytes::find(&text, if number == 29 { b"T129".as_slice() } else { title.as_ref() }).is_some()
        );
        assert!(skein_lib::bytes::find(&text, b"Choose?").is_none(), "no SHORT without a card");
        assert!(skein_lib::bytes::find(&text, b"Shop").is_some());
        assert!(skein_lib::bytes::find(&text, b"under 1 min").is_some());
        assert!(
            skein_lib::bytes::find(
                &text,
                if number > 14 { b"a person task".as_slice() } else { b"a question".as_slice() }
            )
            .is_some()
        );
    }
    let mut terminal = self::view(&root, false);
    let tree = terminal.tree();
    let list =
        tree.nodes().iter().position(|n| n.role == Role::List && n.name.as_deref() == Some(b"Needs you")).unwrap();
    let items = super::super::inbox::subtree(tree, list);
    assert_eq!(items.iter().filter(|n| n.role == Role::Item).count(), 29);
    let item =
        items.iter().find(|n| n.role == Role::Item && n.name.as_deref() == Some(b"Person task from T120")).unwrap();
    let text =
        f::nodes_text(super::super::inbox::subtree(tree, tree.nodes().iter().position(|n| n.id == item.id).unwrap()));
    assert!(skein_lib::bytes::find(&text, "Shop › T120 Need 20".as_bytes()).is_some());
    assert!(skein_lib::bytes::find(&text, b"Person task").is_some());
    assert!(skein_lib::bytes::find(&text, b"under 1 min").is_some());
    let open = f::find(tree, Role::Link, "Open T120 ↗".as_bytes()).unwrap();
    assert_eq!(
        crate::act(&mut terminal, open, Act::Press),
        Some(d::Action::Pages(pages::Act::Go { address: Address::Task { task: 120 } }))
    );
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the test extracts only outgoing card reads")]
fn opening_the_twentieth_row_reads_once_then_draws_the_standing_tasks_group() {
    let mut root = paged();
    let mut view = view(&root, true);
    let id = rows(&view)[18].id;
    let action = crate::act(&mut view, id, Act::Press).unwrap();
    assert_eq!(action, d::Action::Pages(pages::Act::Needs(needs::Act::Open { object: object(21) })));
    let mut out = Queue::uncapped();
    event(&mut root, Event::Act { action }, &mut out);
    let reads: Vec<_> = out
        .iter()
        .filter_map(|request| match request {
            d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
            } => Some((*read, what.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].1, Read::PersonTask { task: 121 });
    let pending = self::view(&root, true);
    assert_eq!(rows(&pending).len(), 29, "all stay closed until the read answers");
    assert_eq!(styled(&pending, look::NEEDS_BAR_AT).name.as_deref(), Some(b"20 of 29".as_slice()));
    settle(&mut root, &mut out);
    let view = self::view(&root, true);
    assert_eq!(rows(&view).len(), 28);
    assert!(has(&view, b"Standing triager"));
    assert!(has(&view, b"3 bugs"));
    assert!(has(&view, b"in temper"));
    assert!(f::find(view.tree(), Role::Button, b"A goal for each").is_some());
    assert_eq!(styled(&view, look::NEEDS_BAR_AT).name.as_deref(), Some(b"20 of 29".as_slice()));
    let inbox = root.pages().inbox().unwrap();
    assert_eq!(inbox.cards.len(), 8);
    assert!(!inbox.cards.iter().any(|card| card.object == object(8)));
    assert_eq!(root.pages().needs().all(inbox, root.forge().approvals()).len(), 29);
    let pages::home::Needs::Count { count, .. } = root.pages().needs().counted(inbox, root.forge().approvals()) else {
        panic!("known count")
    };
    assert_eq!(count, 29);
    let pages::home::Needs::Count { count, .. } = pages::all_home::facts(&root, f::NOW).needs else {
        panic!("Home's count is known")
    };
    assert_eq!(count, 29);
    let group = styled(&view, look::FINDING_ROWS);
    let index = view.tree().nodes().iter().position(|n| n.id == group.id).unwrap();
    assert_eq!(super::super::inbox::subtree(view.tree(), index).iter().filter(|n| n.role == Role::Item).count(), 3);
}
