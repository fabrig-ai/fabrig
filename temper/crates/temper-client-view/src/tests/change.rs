use super::fixtures::{self as f, panel as p, task as t};

use crate::look;

use jig_client::{Down, Up, Watchable, contract as c};

use skein_lib::{List, Queue, Token};

use skein_ui::{Role, Tree};

use temper_client_domain::{Domain, Event, Request, forge};

use temper_client_protocol_wire::{self as wire, v1 as w};
use crate::work_tests::change::{TASK, contains, deliver, event, joined, opened, panel, parts, position, queued, styled, task_name};

fn page_view(root: &Domain) -> crate::View {
    let words = f::words();
    let domain = f::limits();
    let limits = crate::Limits::of(&domain, &words).unwrap();
    let mut view = crate::View::new(&limits, &domain, words, f::FORGE);
    f::build(&mut view, root);
    view
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
