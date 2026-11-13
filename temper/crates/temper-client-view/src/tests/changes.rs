//! Every form of Changes' words and links (temper/docs/design/client/pages/README.md, temper/docs/design/client/view.md, 3, temper/docs/design/client/pages/task.md, 3).
use super::fixtures as f;

use crate::{
    look,
    pages::changes::{LandingFacts, landing_line, ordinal},
    words::Times,
};

use jig_client::{Down, Readable, Up, contract as c};

use skein_lib::{List, Queue, Token, Wall};

use skein_ui::Role;

use temper_client_domain::{Address, Domain, Event, Place, Request};

use temper_client_protocol_wire::{self as wire, v1 as w};

const NOW: Wall = Wall::from_nanos(1_792_656_000_000_000_000);

const SIX_MIN_AGO: u64 = 1_792_655_640_000_000_000;

const FIRST: u64 = 9;


fn queue(repository: &[u8], branch: &[u8], count: u32, red: bool) -> w::ChangesQueue {
    let mut changes = List::with_capacity(32);
    for index in 0..count {
        changes
            .push(
                w::ChangesQueued::new(
                    &w::CEILINGS,
                    w::ChangesQueuedParts {
                        task: FIRST + u64::from(index),
                        title: if index == 0 {
                            b"Store: policy record".as_slice().into()
                        } else {
                            b"Another change".as_slice().into()
                        },
                        ready_since: SIX_MIN_AGO,
                        checks_since: if index == 0 { Some(SIX_MIN_AGO) } else { None },
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::ChangesQueue::new(
        &w::CEILINGS,
        w::ChangesQueueParts {
            way: w::Way::Queue,
            in_flight: None,
            repository: repository.into(),
            branch: branch.into(),
            red,
            changes,
        },
    )
    .unwrap()
}

fn panel(queues: &[w::ChangesQueue], more: u32, landed: bool, landing: &[u8]) -> w::ChangesPanel {
    let mut rows = List::with_capacity(32);
    for row in queues {
        rows.push(row.clone()).unwrap();
    }
    let mut recent = List::with_capacity(8);
    if landed {
        recent
            .push(
                w::ChangesLanded::new(
                    &w::CEILINGS,
                    w::ChangesLandedParts {
                        signed: List::with_capacity(8),
                        commit: [0xab; 32],
                        default_branch: true,
                        task: 80,
                        title: b"A landed change".as_slice().into(),
                        at: SIX_MIN_AGO,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::ChangesPanel::new(
        &w::CEILINGS,
        w::ChangesPanelParts {
            landed_default: List::with_capacity(0),
            landed_default_more: 0,
            project: f::PROJECT,
            landing: landing.into(),
            queues: rows,
            more,
            landed: recent,
            landed_more: 14,
        },
    )
    .unwrap()
}

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: NOW, limits: f::domain_limits() },
        event,
        &mut out,
    );
    out
}

fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}

fn opened() -> (Domain, Token) {
    let (mut root, _, _) = f::root_at(Address::Settings { project: f::PROJECT }, &f::domain_limits());
    let out = event(&mut root, Event::Went { place: Place::Address(Address::Changes { project: f::PROJECT }) });
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && Some(subject.clone())
                == temper_client_domain::forge::changes::watch(f::PROJECT, &f::domain_limits().forge)
        {
            up(&mut root, Up::Watched { watch: *watch, event: c::WatchEvent::Opened });
            return (root, *watch);
        }
    }
    panic!("changes watch");
}

fn show(root: &mut Domain, watch: Token, panel: &w::ChangesPanel) -> Queue<Request> {
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::changes_panel(panel),
            },
        },
    )
}

fn view(root: &Domain, browser: bool) -> crate::View {
    let words = if browser { f::browser_words() } else { f::words() };
    let limits = crate::Limits::of(&f::domain_limits(), &words).unwrap();
    let mut view = crate::View::new(&limits, &f::domain_limits(), words, f::FORGE);
    crate::build(&mut view, root, NOW, &mut Queue::uncapped());
    view
}

fn main_words(root: &Domain) -> Box<[u8]> {
    f::main_text(view(root, true).tree())
}

fn contains(text: &[u8], words: &str) -> bool {
    skein_lib::bytes::find(text, words.as_bytes()).is_some()
}

fn assert_line(panel: &w::ChangesPanel, expected: &[(&str, bool)]) {
    let mut red = List::uncapped();
    for queue in panel.queues() {
        if queue.red() {
            red.push(queue.branch()).unwrap();
        }
    }
    let parts = landing_line(&LandingFacts::panel(panel, red.as_slice()), &f::words(), &mut Times::new(NOW));
    assert_eq!(parts.len(), expected.len());
    for (part, (words, red)) in parts.iter().zip(expected) {
        assert_eq!(&*part.words, words.as_bytes());
        assert_eq!(part.red, *red);
    }
}

fn has_style(tree: &skein_ui::Tree, style: skein_ui::Style) -> bool {
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&style) {
            return true;
        }
    }
    false
}

fn named<'a>(tree: &'a skein_ui::Tree, role: Role, name: &[u8]) -> &'a skein_ui::Node {
    for node in tree.nodes() {
        if node.role == role && node.name.as_deref() == Some(name) {
            return node;
        }
    }
    panic!("named node");
}

#[test]
fn headings_and_ledes_name_counts_and_the_one_branch_name_across_repositories() {
    for (count, heading) in [
        (0, "Nothing is on the way to main."),
        (1, "One change is on the way to main."),
        (3, "Three changes are on the way to main."),
        (9, "Nine changes are on the way to main."),
        (10, "10 changes are on the way to main."),
        (12, "12 changes are on the way to main."),
    ] {
        let (mut root, watch) = opened();
        show(&mut root, watch, &panel(&[queue(b"owner/repo", b"main", count, false)], 0, false, b"main"));
        let view = view(&root, true);
        assert!(f::find(view.tree(), Role::Heading { level: 2 }, heading.as_bytes()).is_some(), "{heading}");
        assert!(contains(&main_words(&root), "Each change is checked on top of main before it lands."));
    }
    for (branches, heading, lede) in [
        ([b"main".as_slice(), b"main".as_slice()], "Two changes are on the way to main.", "top of main"),
        ([b"main".as_slice(), b"release".as_slice()], "Two changes are on the way to landing.", "top of its branch"),
    ] {
        let (mut root, watch) = opened();
        show(
            &mut root,
            watch,
            &panel(
                &[
                    queue(b"one/repo", branches.first().unwrap(), 1, false),
                    queue(b"two/repo", branches.get(1).unwrap(), 1, false),
                ],
                0,
                false,
                b"default",
            ),
        );
        let text = main_words(&root);
        assert!(contains(&text, heading));
        assert!(contains(&text, lede));
        assert!(contains(&text, "Queue · one/repo"));
        assert!(contains(&text, "Queue · two/repo"));
    }
    let (mut root, watch) = opened();
    show(&mut root, watch, &panel(&[], 0, false, b""));
    let text = main_words(&root);
    assert!(contains(&text, "Nothing is on the way to landing."));
    assert!(contains(&text, "top of its branch"));
    let branch = Box::new([b'b'; 1024]);
    let repository = Box::new([b'r'; 2048]);
    let (mut root, watch) = opened();
    show(
        &mut root,
        watch,
        &panel(
            &[
                queue(repository.as_slice(), branch.as_slice(), 1, false),
                queue(b"other/repo", branch.as_slice(), 1, false),
            ],
            0,
            false,
            b"main",
        ),
    );
    assert!(skein_lib::bytes::find(&main_words(&root), branch.as_slice()).is_some());
}

#[test]
fn landing_lines_cover_queued_landed_empty_and_red_with_no_queue() {
    for (count, landed, expected) in [
        (3, true, [("3 queued", false), ("last landed 6 min ago", false)].as_slice()),
        (3, false, [("3 queued", false)].as_slice()),
        (0, true, [("main last moved 6 min ago", false), ("nothing queued", false)].as_slice()),
        (0, false, [("nothing has landed yet", false)].as_slice()),
    ] {
        assert_line(&panel(&[queue(b"owner/repo", b"main", count, false)], 0, landed, b"main"), expected);
    }
    let red = panel(&[queue(b"owner/repo", b"main", 0, true)], 0, false, b"main");
    assert_line(&red, &[("nothing has landed yet", false), ("main red", true)]);
    let (mut root, watch) = opened();
    show(&mut root, watch, &red);
    assert!(has_style(view(&root, true).tree(), look::CHANGES_RED));
    let more = panel(&[queue(b"owner/repo", b"main", 1, false)], 11, false, b"main");
    assert_line(&more, &[("12 queued", false)]);
}

#[test]
fn rows_link_to_tasks_show_arriving_origins_and_put_more_last_without_a_link() {
    let (mut root, watch) = opened();
    let out = show(&mut root, watch, &panel(&[queue(b"owner/repo", b"main", 3, false)], 5, true, b"main"));
    let first = view(&root, true);
    assert!(!has_style(first.tree(), look::CHANGES_MONO));
    let mut read = None;
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Read { read: token, what: Readable::Jig(c::Read::Task { task: FIRST }) },
                },
        } = request
        {
            read = Some(*token);
        }
    }
    let mut task = f::task::document();
    task.chip.task = FIRST;
    task.goal = Some(7);
    let mut chip = task.chip.clone();
    chip.task = 7;
    chip.title = f::task::document_words(b"first-run-projects");
    task.lineage_chips = List::with_capacity(1);
    task.lineage_chips.push(chip).unwrap();
    up(
        &mut root,
        Up::ReadDone {
            read: read.unwrap(),
            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(task)))),
        },
    );
    let mut view = view(&root, true);
    assert_eq!(skein_ui::outline::check(view.tree()), None);
    let text = f::main_text(view.tree());
    for words in [
        "Store: policy record",
        "first-run-projects",
        "checking · checks started 6 min ago",
        "queued 2nd",
        "queued 3rd",
        "and 5 more",
        "07:54",
    ] {
        assert!(contains(&text, words), "{words}");
    }
    let mut first = None;
    for node in view.tree().nodes() {
        if node.role == Role::Link
            && let Some(name) = &node.name
            && name.starts_with(b"Store: policy record")
        {
            first = Some(node.id);
        }
    }
    assert_eq!(crate::href(&mut view, first.unwrap()).as_deref(), Some(b"/tasks/9".as_slice()));
    let more = named(view.tree(), Role::Item, b"and 5 more");
    assert!(more.link.is_none());
    assert_eq!(more.size, 2, "the item contains its text leaf alone");
    assert!(has_style(view.tree(), look::CHANGES_RUNNING));
    show(
        &mut root,
        watch,
        &panel(&[queue(b"one/repo", b"main", 1, false), queue(b"two/repo", b"release", 1, false)], 5, false, b"main"),
    );
    let bytes = main_words(&root);
    assert!(contains(&bytes, "Seven changes are on the way to landing."));
    assert_eq!(skein_lib::bytes::count(&bytes, b"and 5 more", u32::MAX), 1);
    assert!(
        skein_lib::bytes::find(&bytes, "Queue · two/repo".as_bytes()).unwrap()
            < skein_lib::bytes::find(&bytes, b"and 5 more").unwrap()
    );
}

#[test]
fn loading_ended_and_empty_keep_their_own_words_and_the_landed_rule() {
    let (mut root, watch) = opened();
    assert!(contains(&main_words(&root), "Shop · Work"));
    assert!(!contains(&main_words(&root), "Nothing in flight yet."));
    up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::NotServed });
    assert!(contains(&main_words(&root), "Shop · Work"));
    assert!(contains(&main_words(&root), "Shown on the forge."));
    let (mut root, watch) = opened();
    show(&mut root, watch, &panel(&[], 0, false, b"main"));
    for browser in [false, true] {
        let view = view(&root, browser);
        let text = f::main_text(view.tree());
        assert!(contains(&text, "Empty."));
        assert!(f::find(view.tree(), Role::Heading { level: 2 }, b"Landed recently").is_some());
        let mut empty = false;
        for node in view.tree().nodes() {
            if node.role == Role::List && node.size == 1 && node.styles.as_slice().contains(&look::HOME_ROWS) {
                empty = true;
            }
        }
        assert!(empty, "empty landed list retains its rule");
    }
}

#[test]
fn work_is_current_for_changes_in_the_rail_and_tabbar() {
    let (mut root, watch) = opened();
    show(&mut root, watch, &panel(&[], 0, false, b"main"));
    let mut view = view(&root, true);
    let mut changes = None;
    for (index, node) in view.tree().nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::RAIL_NAV) || node.styles.as_slice().contains(&look::TABBAR) {
            let end = index.checked_add(usize::try_from(node.size).unwrap()).unwrap();
            let mut rows = List::with_capacity(4);
            for child in view.tree().nodes().get(index..end).unwrap() {
                if child.role == Role::Link {
                    rows.push(child).unwrap();
                }
            }
            if node.styles.as_slice().contains(&look::RAIL_NAV) {
                assert_eq!(rows.len(), 4);
                for (row, name) in rows.iter().zip([b"Home".as_slice(), b"Needs you", b"Work", b"How work runs"]) {
                    assert_eq!(row.name.as_deref(), Some(name));
                }
                let row = rows.get(2).unwrap();
                assert!(row.states.as_slice().contains(&skein_ui::State::Current));
                changes = Some(row.id);
            } else {
                for row in &rows {
                    assert_ne!(row.name.as_deref(), Some(b"Changes".as_slice()));
                }
            }
        }
    }
    assert_eq!(crate::href(&mut view, changes.unwrap()).as_deref(), Some(b"/projects/1/work".as_slice()));
}

#[test]
fn a_changes_task_bar_opens_with_its_projects_changes_breadcrumb() {
    let mut task = f::task::document();
    task.chip.task = FIRST;
    task.chip.executor = c::ChipExecutor::Procedure { connector: f::FORGE + 1, kind: 1 };
    task.chip.details.details.procedure =
        Some(c::Part { connector: f::FORGE, kind: c::PartKind::Procedure, key: c::PartKey::Number(FIRST) });
    let (mut root, token) = f::task::supplied_root_watching(task.clone(), false);
    task.chip.executor = c::ChipExecutor::Procedure { connector: f::FORGE, kind: 1 };
    up(&mut root, Up::Watched { watch: token, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))) });
    let mut view = view(&root, true);
    let mut bar = None;
    for (index, node) in view.tree().nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::PAGEBAR) {
            bar = Some(index);
        }
    }
    let first = view.tree().nodes().get(bar.unwrap().checked_add(1).unwrap()).unwrap();
    assert_eq!(first.role, Role::Link);
    assert_eq!(first.name.as_deref(), Some(b"Changes".as_slice()));
    let id = first.id;
    assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/projects/7/changes".as_slice()));
}


#[test]
fn a_searching_train_shows_probing_at_its_prefix() {
    let mut busy = queue(b"work/hello", b"main", 1, false).into_parts();
    busy.way = w::Way::Turn;
    busy.in_flight = Some(
        w::TrainLanding::new(
            &w::CEILINGS,
            w::TrainLandingParts {
                task: 61,
                member: 51,
                round: 2,
                step: w::LandingStep::Probing,
                probe: Some([0x33; 32]),
                title: b"In flight".as_slice().into(),
                ready_since: SIX_MIN_AGO,
            },
        )
        .unwrap(),
    );
    let value = panel(&[w::ChangesQueue::new(&w::CEILINGS, busy).unwrap()], 0, false, b"main");
    let (mut root, watch) = opened();
    show(&mut root, watch, &value);
    let text = main_words(&root);
    assert!(contains(&text, "Landing 61 · member 51 · round 2"));
    assert!(contains(&text, "probing at 3333333"));
}


#[test]
fn turn_trains_show_the_landing_then_waiters_and_a_free_train() {
    let mut busy = queue(b"work/hello", b"main", 1, false).into_parts();
    busy.way = w::Way::Turn;
    busy.in_flight = Some(
        w::TrainLanding::new(
            &w::CEILINGS,
            w::TrainLandingParts {
                probe: None,
                task: 61,
                member: 51,
                round: 2,
                step: w::LandingStep::Held,
                title: b"In flight".as_slice().into(),
                ready_since: SIX_MIN_AGO,
            },
        )
        .unwrap(),
    );
    let mut waiting = busy.changes.get(0).unwrap().clone().into_parts();
    waiting.title = b"Waiting behind".as_slice().into();
    waiting.checks_since = None;
    busy.changes = List::from_boxed(Box::new([w::ChangesQueued::new(&w::CEILINGS, waiting).unwrap()]));
    let mut free = queue(b"work/hello", b"release", 2, false).into_parts();
    free.way = w::Way::Turn;
    for (index, title) in [(0, b"First waiting".as_slice()), (1, b"Second waiting".as_slice())] {
        let mut row = free.changes.get(index).unwrap().clone().into_parts();
        row.title = title.into();
        row.task = 71 + u64::from(index);
        row.checks_since = None;
        *free.changes.get_mut(index).unwrap() = w::ChangesQueued::new(&w::CEILINGS, row).unwrap();
    }
    let value = panel(
        &[w::ChangesQueue::new(&w::CEILINGS, busy).unwrap(), w::ChangesQueue::new(&w::CEILINGS, free).unwrap()],
        0,
        false,
        b"main",
    );
    let (mut root, watch) = opened();
    show(&mut root, watch, &value);
    let text = main_words(&root);
    for expected in [
        "Train · work/hello main",
        "Landing 61 · member 51 · round 2",
        "held",
        "Train · work/hello release",
        "2nd",
        "1st",
    ] {
        assert!(contains(&text, expected), "{expected}");
    }
    let mut previous = 0;
    for expected in ["In flight", "Waiting behind", "First waiting", "Second waiting"] {
        let position = skein_lib::bytes::find(&text, expected.as_bytes()).unwrap();
        assert!(position > previous, "{expected}");
        previous = position;
    }
    assert!(!contains(&text, "Checking now"));
}
