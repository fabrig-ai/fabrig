//! All projects' words, parts and actions (temper/docs/design/client/pages/home.md, 1, Home on All projects).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "view tables inspect semantic trees with ordinary Rust"
)]
use super::fixtures as f;

use crate::{
    frame::Bindings,
    look, pages,
    words::{Shell, Times, Words},
};

use skein_lib::Wall;

use skein_ui::{Builder, Role, Tree};

use temper_client_domain::{
    self as d,
    pages::{
        all_home::{self as h, Facts, Project},
        home::{Needs, Status},
    },
};

use temper_client_protocol_wire::v1 as w;


// Sunday 08:04, with brief and decision clocks on the same day.
const MINUTE: u64 = 60_000_000_000;

const DAY: u64 = 24 * 60 * MINUTE;

const NOW: u64 = 3 * DAY + (8 * 60 + 4) * MINUTE;

fn status(
    project: u32,
    landed: u64,
    seen: u64,
    previous: Option<u64>,
    last: Option<u64>,
    red: bool,
    brief: Option<w::StatusBrief>,
) -> Status {
    Status::Shown(Box::new(
        w::ProjectStatus::new(
            &w::CEILINGS,
            w::ProjectStatusParts {
                project,
                landed,
                seen,
                previous,
                last_landed: last,
                queued: 0,
                red,
                branch: b"main".as_slice().into(),
                brief,
            },
        )
        .unwrap(),
    ))
}

fn project(number: u32, name: &str, landed: u64, seen: u64) -> Project {
    Project {
        project: number,
        name: name.as_bytes().into(),
        status: status(number, landed, seen, Some(NOW - 9 * 60 * MINUTE), Some(NOW - 2 * 60 * MINUTE), false, None),
    }
}

fn facts(projects: Vec<Project>) -> Facts {
    Facts {
        projects: projects.into_boxed_slice(),
        needs: Needs::Count { count: 0, cut: false },
        decided: Box::default(),
        showing: 0,
        away: None,
        follows: Box::default(),
    }
}

fn drawn(facts: &Facts, words: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(128);
    pages::all_home::draw(&mut b, facts, words, &mut bindings, &mut Times::new(Wall::from_nanos(NOW)));
    (b.finish(), bindings)
}

fn has(tree: &Tree, text: &str) -> bool {
    skein_lib::bytes::find(&f::text(tree), text.as_bytes()).is_some()
}

fn named(tree: &Tree, role: Role, text: &str) -> bool {
    f::find(tree, role, text.as_bytes()).is_some()
}

fn action(tree: &Tree, bindings: &Bindings, name: &str) -> d::Action {
    let node = tree
        .nodes()
        .iter()
        .find(|n| n.name.as_deref() == Some(name.as_bytes()) && n.binding.is_some())
        .expect("bound control");
    bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press).unwrap()
}

#[test]
fn done_home_and_the_rail_use_all_projects() {
    for (count, expected) in [(1_u32, d::Address::Home { project: f::PROJECT }), (2, d::Address::AllHome)] {
        let mut page = f::ReadPage::new(d::Address::Inbox { filter: d::address::Filter::ALL });
        page.refresh_inbox();
        let reads: Vec<_> = page.reads.iter().cloned().collect();
        for (read, what) in reads {
            if let jig_client::contract::Read::Inbox { .. } = what {
                page.answer(
                    read,
                    jig_client::contract::ReadResult::Value(jig_client::contract::ReadValue::Inbox(Box::new(
                        jig_client::contract::InboxPage {
                            selection: None,
                            before: None,
                            rows: skein_lib::List::uncapped(),
                            after: None,
                            cut: false,
                        },
                    ))),
                );
            }
        }
        let watch = page.me.unwrap();
        let mut root = page.root;
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.projects = skein_lib::List::from_boxed(
            me.projects.iter().take(usize::try_from(count).unwrap()).cloned().collect::<Vec<_>>().into_boxed_slice(),
        );
        f::deliver_me(&mut root, watch, me);
        for words in [f::words(), f::browser_words()] {
            let mut builder = Builder::new(&f::tree_limits());
            let mut bindings = Bindings::new(128);
            pages::needs::main(
                &mut builder,
                &root,
                &words,
                &mut bindings,
                40,
                &mut Times::new(f::NOW),
                &crate::forge_sign_in::Forges::new(),
            );
            let tree = builder.finish();
            assert_eq!(
                action(&tree, &bindings, "Home"),
                d::Action::Pages(d::pages::Act::Go { address: expected.clone() })
            );
        }
    }
    let (root, _, _) = f::root_at(d::Address::AllHome, &f::domain_limits());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(view.tree().nodes().iter().filter(|n| n.role == Role::Main).count(), 1);
        assert!(
            view.tree()
                .nodes()
                .iter()
                .any(|n| n.name.as_deref() == Some(b"Home") && n.states.as_slice().contains(&skein_ui::State::Current))
        );
    }
}


#[test]
fn follows_boxes_keep_their_names_words_marks_and_wide_only_region() {
    let mut facts = facts(vec![project(1, "vsr", 0, 0), project(2, "vsr-rs", 0, 0), project(4, "fabrig", 0, 0)]);
    facts.follows = Box::new([
        h::Relation::Follow {
            project: 2,
            name: b"vsr-rs".as_slice().into(),
            source: b"acme/vsr".as_slice().into(),
            line: "2 behind · check passes at 9c04d7e1a2b3".as_bytes().into(),
            stopped: Some(b"Check is missing".as_slice().into()),
        },
        h::Relation::Dependency {
            project: 4,
            name: b"fabrig".as_slice().into(),
            dependency: b"vsr-rs".as_slice().into(),
        },
    ]);
    for words in [f::words(), f::browser_words()] {
        let (tree, _) = drawn(&facts, &words);
        let region = tree
            .nodes()
            .iter()
            .position(|n| n.role == Role::Region && n.name.as_deref() == Some(b"What follows what"))
            .unwrap();
        let projects =
            tree.nodes().iter().position(|n| n.role == Role::Region && n.name.as_deref() == Some(b"Projects")).unwrap();
        assert!(region > projects);
        assert_eq!(tree.nodes()[region].styles.as_slice().contains(&look::HOME_WIDE), words.shell == Shell::Browser);
        assert!(named(&tree, Role::Heading { level: 2 }, "What follows what"));
        assert!(has(&tree, "across projects"));
        for (name, chip, title, line, mark) in [
            (
                "vsr-rs follows acme/vsr",
                "follows acme/vsr",
                "vsr-rs",
                "2 behind · check passes at 9c04d7e1a2b3",
                look::HOW_WHO_CODE,
            ),
            (
                "fabrig depends on vsr-rs",
                "depends on vsr-rs",
                "fabrig",
                "Nothing follows its landings.",
                look::HOW_WHO_NONE,
            ),
        ] {
            let at = tree
                .nodes()
                .iter()
                .position(|n| n.role == Role::Group && n.name.as_deref() == Some(name.as_bytes()))
                .unwrap();
            let end = at + usize::try_from(tree.nodes()[at].size).unwrap();
            let nodes = &tree.nodes()[at..end];
            assert!(nodes.iter().any(|n| n.role == Role::Text
                && n.name.as_deref() == Some(chip.as_bytes())
                && n.styles.as_slice().contains(&mark)));
            assert!(f::node_text(&tree, at).windows(title.len()).any(|bytes| bytes == title.as_bytes()));
            assert!(f::node_text(&tree, at).windows(line.len()).any(|bytes| bytes == line.as_bytes()));
            assert!(nodes.iter().all(|n| n.binding.is_none()), "no box is a control");
            if words.shell == Shell::Terminal {
                let run = format!("{chip} {title} {line}");
                assert!(
                    f::node_text(&tree, at).windows(run.len()).any(|bytes| bytes == run.as_bytes()),
                    "terminal box is one readable run"
                );
            } else {
                assert!(
                    nodes
                        .iter()
                        .any(|n| n.role == Role::Heading { level: 3 } && n.name.as_deref() == Some(title.as_bytes()))
                );
            }
        }
        assert!(has(&tree, "Stopped: Check is missing."));
    }
    facts.follows = Box::default();
    for words in [f::words(), f::browser_words()] {
        assert!(!named(&drawn(&facts, &words).0, Role::Region, "What follows what"));
    }
    // A project's Home never borrows the cross-project region.
    let page = f::ReadPage::new(d::Address::Home { project: f::PROJECT });
    let mut view = f::view_of(f::browser_words());
    assert!(!named(f::build(&mut view, &page.root), Role::Region, "What follows what"));
}
