use super::*;
use skein_ui::{Builder, Limits, View};
use std::{fs, path::Path};
use temper_client_walk_lines::walk::{self, Roots};

#[test]
fn moved_journeys_read_and_join_their_scripts_without_a_world() {
    let client = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design/client");
    let roots = Roots { web: client.join("web/journeys"), tui: client.join("tui/journeys") };
    for root in [&roots.web, &roots.tui] {
        let mut directories = fs::read_dir(root)
            .unwrap_or_else(|why| panic!("{}: {why}", root.display()))
            .map(|entry| entry.expect("journey directory entry").path())
            .collect::<Vec<_>>();
        directories.sort();
        for directory in directories {
            if !directory.is_dir() {
                continue;
            }
            let name = directory.file_name().expect("journey name").to_str().expect("UTF-8 journey name");
            if !walk::moved(&directory).unwrap_or_else(|why| panic!("{name}: {why}")) {
                continue;
            }
            let chain = walk::read(&roots, name).unwrap_or_else(|why| panic!("{name}: {why:?}"));
            joined_scripts(&chain).unwrap_or_else(|why| panic!("{name}: {why}"));
        }
    }
}

const LIMITS: Limits = Limits { nodes: 8, depth: 2, text_per_node: 32, text_bytes: 128, patches: 48, ..Limits::EMPTY };
fn tree(revealed: bool) -> View {
    let mut view = View::new(&LIMITS);
    let mut b: Builder = view.builder();
    b.open(Role::Page);
    for role in [Role::TextArea, Role::TextField, Role::Check] {
        b.open(role);
        b.name(b"Control");
        b.close();
    }
    if revealed {
        b.open(Role::Button);
        b.name(b"Revealed");
        b.close();
    }
    b.close();
    view.finish(b, 1, &mut skein_lib::Queue::with_capacity(LIMITS.patches));
    view
}
fn lines(source: &[&str]) -> Vec<Numbered> {
    source
        .iter()
        .enumerate()
        .map(|(index, line)| Numbered {
            number: u32::try_from(index + 1).expect("line number"),
            line: temper_client_walk_lines::line::Line::parse(line.as_bytes()).expect("line"),
        })
        .collect()
}
#[test]
fn tui_links_use_the_marked_event_and_only_see_lines_before_the_first_key() {
    use crate::tui::{Connection, Held};
    use skein_lib::Time;

    let mut terminal = Terminal::new(0, Held::default(), facts::tui_seed(0), Time::ZERO, facts::WALL);
    terminal.start(&mut Connection::new()).expect("start host without a world");
    terminal.frame();
    assert_eq!(terminal.event_of(&Key::Enter), Some(Event::SignIn));
    for (following, want) in [
        (vec!["tui see \"Sign in\"", "web see \"welcome\"", "tui key enter", "tui mark"], Some(Key::Enter)),
        (vec!["tui key char \"x\"", "tui key enter", "tui mark"], None),
        (vec!["tui key control \"c\"", "tui mark"], None),
        (vec!["tui approve", "tui key enter", "tui mark"], None),
        (vec!["web press button \"Try the playground\"", "tui key enter", "tui mark"], None),
        (vec!["tui stop", "tui key enter", "tui mark"], None),
        (vec!["tui key enter"], None),
        (vec!["tui mark", "tui key enter", "tui mark"], None),
        (vec!["tui key enter", "tui approve", "tui mark"], Some(Key::Enter)),
    ] {
        assert_eq!(tui_link(&terminal, &lines(&following)), want, "{following:?}");
    }
}

#[test]
fn links_use_the_marked_tree_first_act_exact_role_name_order_and_lead() {
    let marked = tree(false);
    let typed = tree(true);
    let following = lines(&[
        "web input textbox \"Control\" \"typed\"",
        "web press button \"Revealed\"",
        "web toggle checkbox \"Control\"",
        "web mark",
    ]);
    assert!(
        link(marked.tree(), &following).0.is_empty(),
        "typing's revealed control is absent at the mark; no later act is substituted"
    );
    assert_eq!(link(typed.tree(), &following).0.len(), 1, "the same find sees it after typing");
    let (nodes, lead) = link(marked.tree(), &lines(&["web submit textbox \"Control\" 1", "web mark"]));
    assert_eq!(nodes.len(), 1);
    assert_eq!(marked.tree().find(nodes[0]).expect("found node").role, Role::TextField);
    assert_eq!(lead, Lead::Submit);
    let (nodes, lead) = link(marked.tree(), &lines(&["web toggle checkbox \"Control\"", "web mark"]));
    assert_eq!(nodes.len(), 1);
    assert_eq!(lead, Lead::Toggle);
    assert!(
        link(marked.tree(), &lines(&["web submit textbox \"control\"", "web mark"])).0.is_empty(),
        "whole case-sensitive names"
    );
    assert!(
        link(marked.tree(), &lines(&["web submit textbox \"Control\""])).0.is_empty(),
        "a final mark has no next page"
    );
}
