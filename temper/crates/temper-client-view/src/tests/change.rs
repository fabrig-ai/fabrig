use super::fixtures::{self as f, panel as p, task as t};

use crate::look;

use jig_client::{Down, Up, Watchable, contract as c};

use skein_lib::{List, Queue, Token};

use skein_ui::{Role, Tree};

use temper_client_domain::{Domain, Event, Request, forge};

use temper_client_protocol_wire::{self as wire, v1 as w};

const TASK: u64 = 9;

fn event(root: &mut Domain, up: Up) -> Queue<Request> {
    let limits = f::limits();
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
        Event::Client { event: jig_client::Event::Up { up } },
        &mut out,
    );
    out
}

fn opened() -> (Domain, Token) {
    let mut task = t::document();
    task.chip.task = TASK;
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.procedure =
        Some(c::Part { connector: f::FORGE, kind: c::PartKind::Procedure, key: c::PartKey::Number(TASK) });
    task.priority = Some(3);
    task.chip.executor = c::ChipExecutor::Procedure { connector: f::FORGE + 1, kind: 1 };
    let (mut root, watch) = t::supplied_root_watching(task.clone(), false);
    task.chip.executor = c::ChipExecutor::Procedure { connector: f::FORGE, kind: 1 };
    let mut out =
        event(&mut root, Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))) });
    let mut family = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject: Watchable::Family { .. } } },
        } = request
        {
            assert!(family.is_none());
            family = Some(watch);
            event(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
        }
    }
    (root, family.expect("family_open"))
}

fn deliver(root: &mut Domain, watch: Token, panel: &w::ChangePanel) {
    event(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::change_panel(panel),
            },
        },
    );
}

fn parts(step: w::Step) -> w::ChangePanelParts {
    let mut parts = p::panel(TASK, step).into_parts();
    parts.owner = Box::from(*b"acme");
    parts.name = Box::from(*b"shop");
    parts.branch = Box::from(*b"temper/9");
    parts.landing = Box::from(*b"main");
    parts.pull = Some(
        w::Pull::new(
            &w::CEILINGS,
            w::PullParts { number: 23, address: Box::from(*b"https://forge.example/acme/shop/pull/23") },
        )
        .unwrap(),
    );
    parts.hold = None;
    parts.runs = None;
    parts.gates = List::with_capacity(0);
    parts
}

fn panel(parts: w::ChangePanelParts) -> w::ChangePanel {
    w::ChangePanel::new(&w::CEILINGS, parts).unwrap()
}

fn page_view(root: &Domain) -> crate::View {
    let words = f::words();
    let domain = f::limits();
    let limits = crate::Limits::of(&domain, &words).unwrap();
    let mut view = crate::View::new(&limits, &domain, words, f::FORGE);
    f::build(&mut view, root);
    view
}

fn position(bytes: &[u8], needle: &[u8]) -> Option<usize> {
    for (index, span) in bytes.windows(needle.len()).enumerate() {
        if span == needle {
            return Some(index);
        }
    }
    None
}

fn contains(tree: &Tree, value: &str) -> bool {
    position(&f::text(tree), value.as_bytes()).is_some()
}

fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).unwrap();
    }
    let mut writer = skein_lib::Writer::new(length);
    for part in parts {
        writer.put(part).unwrap();
    }
    writer.finish()
}

fn task_name(task: u64) -> Box<[u8]> {
    joined(&[b"Task ", skein_lib::Decimal::of(task).as_bytes()])
}

fn styled(tree: &Tree, token: skein_ui::Style) -> Box<[usize]> {
    let mut found = List::with_capacity(u32::try_from(tree.nodes().len()).unwrap());
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&token) {
            found.push(index).unwrap();
        }
    }
    found.into_boxed()
}

fn queued(place: u32, names: &[u64]) -> w::Step {
    let mut ahead = List::with_capacity(u32::try_from(names.len()).unwrap());
    for n in names {
        ahead.push(*n).unwrap();
    }
    w::Step::Queued(w::Queued::new(&w::CEILINGS, w::QueuedParts { place, ahead }).unwrap())
}

#[test]
fn unshown_panels_say_where_they_are_and_other_subjects_draw_nothing() {
    for (end, word) in [
        (None, "Loading the change from the forge."),
        (Some(c::WatchEnd::Unknown), "The forge knows no change for this task."),
        (Some(c::WatchEnd::NotAllowed), "The forge cannot show this change."),
    ] {
        let (mut root, watch) = opened();
        if let Some(end) = end {
            event(&mut root, Up::WatchEnded { watch, end });
        }
        let view = page_view(&root);
        let tree = view.tree();
        assert!(contains(tree, word));
        assert!(styled(tree, look::CHANGE).is_empty());
        assert!(!contains(tree, "Change · Task"));
    }
    let (mut root, _) = opened();
    f::root_event(
        &mut root,
        Event::Forge(forge::Event::Panel { subject: Some(forge::panel::Subject { task: 77, project: 1 }) }),
    );
    let view = page_view(&root);
    assert!(!contains(view.tree(), "Loading the change"));
    assert!(contains(view.tree(), "Shown on the forge."));
    assert!(styled(view.tree(), look::CHANGE).is_empty());
    let root = t::root(c::ChipExecutor::Procedure { connector: f::FORGE + 1, kind: 1 }, None);
    assert!(styled(page_view(&root).tree(), look::CHANGE).is_empty());
}

#[test]
fn a_full_bound_panel_fits_the_production_tree_text_and_links() {
    let (mut root, watch) = opened();
    let panel = p::panel(TASK, p::queued());
    deliver(&mut root, watch, &panel);
    let view = page_view(&root);
    let tree = view.tree();
    assert_eq!(styled(tree, look::GATE).len(), 65);
    assert_eq!(styled(tree, look::STEP).len(), 6);
    let text = f::text(tree);
    assert!(position(&text, panel.branch()).is_some());
    for task in 1_u64..=64 {
        assert!(f::find(tree, Role::Link, &task_name(task)).is_some());
    }
    assert!(contains(tree, "Repairs 1 of 2 · resolutions 3 of 4 · updates 5 of 6"));
}


#[test]
fn the_queue_offers_each_task_ahead_as_a_terminal_command() {
    let (mut root, watch) = opened();
    deliver(&mut root, watch, &panel(parts(queued(4, &[7, 8]))));
    let mut view = page_view(&root);
    for task in [7, 8] {
        let label = task_name(task);
        let control = f::find(view.tree(), Role::Link, &label).unwrap();
        let (node, first) = f::terminal_command(
            view.tree(),
            &view.limits().tree,
            control,
            &label,
            b"4th in the queue for main, after ",
        );
        assert_eq!(
            crate::act(&mut view, node, first),
            Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                address: temper_client_domain::Address::Task { task },
            },))
        );
    }
}


#[test]
fn a_change_panel_names_its_way_and_retained_landing() {
    let (mut root, watch) = opened();
    let mut value = parts(w::Step::Landed(w::Landed::new(&w::CEILINGS, w::LandedParts { merge: [9; 32] }).unwrap()));
    value.way = w::Way::Turn;
    value.in_landing = Some(61);
    deliver(&mut root, watch, &w::ChangePanel::new(&w::CEILINGS, value).unwrap());
    let text = f::main_text(page_view(&root).tree());
    assert!(skein_lib::bytes::find(&text, b"Way: turn.").is_some());
    assert!(skein_lib::bytes::find(&text, b"Landing: task 61.").is_some());
}
