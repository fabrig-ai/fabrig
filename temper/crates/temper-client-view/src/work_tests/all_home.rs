//! All projects' words, parts and actions (temper/docs/design/client/pages/home.md, 1, Home on All projects).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "view tables inspect semantic trees with ordinary Rust"
)]
use super::fixtures as f;

use crate::{
    bindings::Bindings,
    look, pages,
    words::{Shell, Times, Words},
};

use skein_lib::Wall;

use skein_ui::{Builder, Role, Run, Tree};

use temper_client_domain::{
    self as d,
    pages::{
        all_home::{self as h, Facts, Project},
        decided::{DecisionLine, DecisionTask},
        home::{Needs, Status},
    },
};

use temper_client_protocol_wire::v1 as w;


// Sunday 08:04, with brief and decision clocks on the same day.
pub(crate) const MINUTE: u64 = 60_000_000_000;

pub(crate) const DAY: u64 = 24 * 60 * MINUTE;

pub(crate) const NOW: u64 = 3 * DAY + (8 * 60 + 4) * MINUTE;

const BRIEF_AT: u64 = 3 * DAY + (7 * 60 + 52) * MINUTE;

pub(crate) fn status(
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

pub(crate) fn project(number: u32, name: &str, landed: u64, seen: u64) -> Project {
    Project {
        project: number,
        name: name.as_bytes().into(),
        status: status(number, landed, seen, Some(NOW - 9 * 60 * MINUTE), Some(NOW - 2 * 60 * MINUTE), false, None),
    }
}

pub(crate) fn facts(projects: Vec<Project>) -> Facts {
    Facts {
        projects: projects.into_boxed_slice(),
        needs: Needs::Count { count: 0, cut: false },
        decided: Box::default(),
        showing: 0,
        away: None,
        follows: Box::default(),
    }
}

pub(crate) fn drawn(facts: &Facts, words: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(128);
    pages::all_home::draw(&mut b, facts, words, &mut bindings, &mut Times::new(Wall::from_nanos(NOW)));
    (b.finish(), bindings)
}

pub(crate) fn has(tree: &Tree, text: &str) -> bool {
    skein_lib::bytes::find(&f::text(tree), text.as_bytes()).is_some()
}

pub(crate) fn named(tree: &Tree, role: Role, text: &str) -> bool {
    f::find(tree, role, text.as_bytes()).is_some()
}

pub(crate) fn action(tree: &Tree, bindings: &Bindings, name: &str) -> d::Action {
    let node = tree
        .nodes()
        .iter()
        .find(|n| n.name.as_deref() == Some(name.as_bytes()) && n.binding.is_some())
        .expect("bound control");
    bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press).unwrap()
}

#[test]
fn landed_and_needs_sentences_compose_from_the_facts() {
    for (projects, needs, expected, phone) in [
        (
            vec![
                project(1, "vsr", 10, 0),
                project(2, "rust", 10, 0),
                project(3, "sql", 10, 0),
                project(4, "fabrig", 1, 0),
            ],
            Needs::Count { count: 2, cut: false },
            "31 changes landed across four projects. Two things need you.",
            "31 changes landed. Two things need you.",
        ),
        (
            vec![project(1, "vsr", 3, 0)],
            Needs::Count { count: 1, cut: false },
            "3 changes landed in vsr. One thing needs you.",
            "3 changes landed. One thing needs you.",
        ),
        (
            vec![project(1, "vsr", 1, 0)],
            Needs::Count { count: 0, cut: false },
            "One change landed in vsr. Nothing needs you.",
            "One change landed. Nothing needs you.",
        ),
        (
            vec![project(1, "vsr", 3, 3)],
            Needs::Count { count: 12, cut: true },
            "The last change landed 2 hours ago. 12+ things need you.",
            "The last change landed 2 hours ago. 12+ things need you.",
        ),
        (vec![], Needs::Count { count: 10, cut: false }, "10 things need you.", "10 things need you."),
        (vec![], Needs::Unknown, "", ""),
    ] {
        let mut facts = facts(projects);
        facts.needs = needs;
        for words in [f::words(), f::browser_words()] {
            let (tree, _) = drawn(&facts, &words);
            assert!(named(&tree, Role::Heading { level: 1 }, expected), "{expected}");
            let headings: Vec<_> = tree.nodes().iter().filter(|n| n.role == Role::Heading { level: 1 }).collect();
            assert_eq!(headings.len(), if words.shell == Shell::Browser { 2 } else { 1 });
            for (form, text) in [(look::HOME_WIDE, expected), (look::HOME_PHONE, phone)] {
                let heading = headings.iter().find(|n| n.styles.as_slice().contains(&form));
                assert_eq!(heading.is_some(), words.shell == Shell::Browser, "{form:?}");
                if let Some(heading) = heading {
                    assert_eq!(heading.name.as_deref(), Some(text.as_bytes()));
                    assert!(heading.styles.as_slice().contains(&look::HOME_HEAD));
                    let at = tree.nodes().iter().position(|n| core::ptr::eq(n, *heading)).unwrap();
                    assert_eq!(f::node_text(&tree, at).as_ref(), text.as_bytes());
                }
            }
            assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Main).count(), 1);
        }
    }
}

#[test]
fn the_kicker_uses_only_the_look_and_away_given() {
    for (previous, away, looked, spell) in [
        (Some(NOW - 9 * 60 * MINUTE), true, true, true),
        (None, false, false, false),
        (Some(NOW - MINUTE), false, false, false),
        (Some(NOW - 9 * 60 * MINUTE), false, true, false),
    ] {
        let mut facts = facts(vec![Project {
            project: 1,
            name: b"vsr".as_slice().into(),
            status: status(1, 0, 0, previous, None, false, None),
        }]);
        if away {
            facts.away = Some(h::Spell {
                since: Wall::from_nanos(2 * DAY + (23 * 60 + 10) * MINUTE),
                off: Wall::from_nanos(3 * DAY + 8 * 60 * MINUTE),
            });
        }
        for words in [f::words(), f::browser_words()] {
            let (tree, _) = drawn(&facts, &words);
            assert_eq!(has(&tree, "you last looked"), looked);
            if looked {
                assert!(has(&tree, "you last looked 9 hours ago"));
            }
            assert_eq!(has(&tree, " · away 23:10 to 08:00"), spell);
        }
    }
}

#[test]
fn briefs_join_four_sentences_with_strong_project_names() {
    for (count, expected, byline) in [
        (0, "", false),
        (1, "vsr: First sentence.", true),
        (5, "vsr: First sentence. rust: First sentence. sql: First sentence. fabrig: First sentence.", true),
    ] {
        let projects: Vec<_> = ["vsr", "rust", "sql", "fabrig", "extra"]
            .iter()
            .take(count)
            .enumerate()
            .map(|(i, name)| {
                let mut project = project(u32::try_from(i).unwrap(), name, 1, 0);
                let brief = w::StatusBrief::new(
                    &w::CEILINGS,
                    w::StatusBriefParts {
                        at: BRIEF_AT - u64::try_from(i).unwrap() * MINUTE,
                        landed: 1,
                        sentence: b"Never the headline.".as_slice().into(),
                        lede: b"First sentence. Second sentence!".as_slice().into(),
                    },
                )
                .unwrap();
                project.status =
                    status(project.project, 1, 0, Some(NOW - 9 * 60 * MINUTE), Some(NOW), false, Some(brief));
                project
            })
            .collect();
        for words in [f::words(), f::browser_words()] {
            let (tree, _) = drawn(&facts(projects_for_briefs(&projects)), &words);
            let lede = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::HOME_LEDE));
            assert_eq!(lede.is_some(), byline);
            if let Some(lede) = lede {
                let at = tree.nodes().iter().position(|n| core::ptr::eq(n, lede)).unwrap();
                assert_eq!(f::node_text(&tree, at).as_ref(), expected.as_bytes());
                assert_eq!(
                    tree.nodes()[at..at + usize::try_from(lede.size).unwrap()]
                        .iter()
                        .flat_map(|n| n.text.iter())
                        .filter(|run| run.run == Run::Strong)
                        .count(),
                    count.min(4)
                );
            }
            let expected_byline =
                "Each project's brief, its first line · the newest, vsr's, by the status agent at 07:52";
            assert_eq!(named(&tree, Role::Paragraph, expected_byline), byline);
            assert!(!has(&tree, "Written by the status agent at"));
            if byline {
                let node = tree.nodes().iter().find(|n| n.name.as_deref() == Some(expected_byline.as_bytes())).unwrap();
                assert!(node.styles.as_slice().contains(&look::HOME_BYLINE));
                assert_eq!(node.styles.as_slice().contains(&look::HOME_WIDE), words.shell == Shell::Browser);
                assert!(!node.styles.as_slice().contains(&look::HOME_PHONE));
                assert!(has(&tree, expected_byline));
            }
            assert_eq!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::HOME_BYLINE)), byline);
            assert_eq!(has(&tree, "The status agent, 07:52"), byline && words.shell == Shell::Browser);
            assert!(!has(&tree, "Never the headline."));
        }
    }
}

fn projects_for_briefs(projects: &[Project]) -> Vec<Project> {
    projects
        .iter()
        .rev()
        .map(|p| Project {
            project: p.project,
            name: p.name.clone(),
            status: match &p.status {
                Status::Unknown => Status::Unknown,
                Status::Shown(s) => Status::Shown(s.clone()),
            },
        })
        .collect()
}

#[test]
fn each_region_obeys_its_own_parts_rule() {
    for (projects, open, showing, expected) in
        [(0, 0, 0, [false, false, false]), (1, 1, 0, [true, false, false]), (2, 0, 1, [false, true, true])]
    {
        let mut facts = facts((0..projects).map(|i| project(i, "vsr", 0, 0)).collect());
        facts.needs = Needs::Count { count: open, cut: false };
        facts.showing = showing;
        for words in [f::words(), f::browser_words()] {
            let (tree, _) = drawn(&facts, &words);
            for (name, expected) in ["Needs you", "Decided for you", "Projects"].into_iter().zip(expected) {
                assert_eq!(named(&tree, Role::Region, name), expected, "{name}");
            }
        }
    }
}

fn decision(number: u32) -> h::Decided {
    h::Decided {
        project: 2,
        line: DecisionLine {
            project: 2,
            reach: None,
            number,
            task: u64::from(number),
            title: Some(b"Porter".as_slice().into()),
            at: Wall::from_nanos(3 * DAY + 65 * MINUTE),
            question: b"A compiler builtin?".as_slice().into(),
            took: b"Ported as checked_add.".as_slice().into(),
            why: b"Keeps its shape.".as_slice().into(),
            overturned: false,
            state: DecisionTask::Live,
        },
    }
}

#[test]
fn decisions_keep_the_project_words_bound_and_seven_rows() {
    for (rows, showing, expected) in [(4, 4, 4), (8, 9, 7)] {
        let mut facts = facts(vec![project(1, "vsr", 0, 0), project(2, "rust", 0, 0)]);
        facts.decided = (1..=rows).map(decision).collect::<Vec<_>>().into_boxed_slice();
        facts.showing = showing;
        for words in [f::words(), f::browser_words()] {
            let (tree, bindings) = drawn(&facts, &words);
            assert!(named(&tree, Role::Heading { level: 2 }, &format!("Decided for you · {showing}")));
            for word in [
                "rust · Porter · 01:05",
                "A compiler builtin?",
                "Ported as checked_add.",
                "Keeps its shape.",
                "What reaches you",
            ] {
                assert!(has(&tree, word), "{word}");
            }
            assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::DECISION)).count(), expected);
            assert_eq!(
                action(&tree, &bindings, "Overturn"),
                d::Action::Pages(d::pages::Act::Away(d::pages::away::Act::Overturn { project: 2, number: 1 }))
            );
        }
    }
}

#[test]
fn projects_link_home_and_only_known_health_has_chips() {
    for (status, chip, style) in [
        (Status::Unknown, "", None),
        (status(1, 1, 0, None, None, false, None), "main", Some(look::CHIP_GOOD)),
        (status(1, 1, 0, None, None, true, None), "main red", Some(look::CHIP_SIGNAL)),
    ] {
        let facts =
            facts(vec![Project { project: 1, name: b"vsr".as_slice().into(), status }, project(2, "rust", 0, 0)]);
        for words in [f::words(), f::browser_words()] {
            let (tree, bindings) = drawn(&facts, &words);
            assert_eq!(
                action(&tree, &bindings, "vsr"),
                d::Action::Pages(d::pages::Act::Go { address: d::Address::Home { project: 1 } })
            );
            let chips: Vec<_> = tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::CHIP)).collect();
            assert_eq!(chips.len(), if style.is_some() { 2 } else { 1 });
            if let Some(style) = style {
                assert!(chips.iter().any(|n| {
                    n.styles.as_slice().contains(&style)
                        && f::node_text(&tree, tree.nodes().iter().position(|node| core::ptr::eq(node, *n)).unwrap())
                            .as_ref()
                            == chip.as_bytes()
                }));
            }
        }
    }
}

#[test]
fn the_phone_has_its_short_words_and_decisions_link() {
    let mut facts = facts(vec![project(1, "vsr", 0, 0)]);
    facts.showing = 1;
    facts.decided = Box::new([decision(1)]);
    for (words, phone) in [(f::words(), false), (f::browser_words(), true)] {
        let (tree, bindings) = drawn(&facts, &words);
        assert_eq!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::HOME_PHONE)), phone);
        if phone {
            assert_eq!(look::NAMES.style(look::HOME_PHONE), Some(b"temper-home-phone".as_slice()));
            assert!(has(&tree, "last looked 9 h ago"));
            assert_eq!(
                action(&tree, &bindings, "Decided for you · 1 ›"),
                d::Action::Pages(d::pages::Act::Go { address: d::Address::Inbox { filter: d::address::Filter::ALL } })
            );
        }
    }
}


#[test]
fn main_fills_only_the_unfiltered_needs_and_its_confirmation() {
    for filter in [d::address::Filter::ALL, d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL }]
    {
        let root = super::needs::fixture(
            "Which way?\n\nChoice: This way (recommended)\nChoice: That way",
            false,
            1,
            false,
            filter,
        );
        for words in [f::words(), f::browser_words()] {
            let (tree, _) = main_tree(&root, &words);
            assert_eq!(
                tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::NEED_CARD)),
                filter == d::address::Filter::ALL
            );
        }
    }
    let proposal = super::needs::proposal(jig_client::contract::Proposer::Task(f::INBOX_TASK));
    let key = proposal.key;
    let mut root = super::needs::kinds(&[(
        jig_client::contract::ObjectKey::Proposal(key),
        jig_client::contract::Read::Proposal { proposal: key },
        jig_client::contract::ReadValue::Proposal(Box::new(proposal)),
    )]);
    for words in [f::words(), f::browser_words()] {
        let (tree, bindings) = main_tree(&root, &words);
        let accept = action(&tree, &bindings, "Accept");
        f::root_event(&mut root, d::Event::Act { action: accept });
        let (tree, _) = main_tree(&root, &words);
        assert!(named(&tree, Role::Dialog, "Accept this proposal?"));
    }
}

fn main_tree(root: &d::Domain, words: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(128);
    pages::all_home::main(&mut b, root, words, &mut bindings, &mut Times::new(f::NOW), 40);
    (b.finish(), bindings)
}


#[test]
fn the_shared_headline_and_joined_brief_are_the_pages_words() {
    let mut projects = vec![project(1, "Hello", 3, 0), project(2, "shop", 1, 0)];
    for project in &mut projects {
        let brief = w::StatusBrief::new(
            &w::CEILINGS,
            w::StatusBriefParts {
                at: BRIEF_AT - u64::from(project.project) * MINUTE,
                landed: 1,
                sentence: b"A change landed.".as_slice().into(),
                lede: b"Ready for review. More details.".as_slice().into(),
            },
        )
        .unwrap();
        project.status = status(project.project, 1, 0, Some(0), Some(NOW), false, Some(brief));
    }
    let all = facts(projects);
    let brief = h::brief(&all).unwrap();
    for words in [f::words(), f::browser_words()] {
        let (tree, _) = drawn(&all, &words);
        let (headline, tick) = crate::shared::all_home_headline(&words, &all, Wall::from_nanos(NOW));
        assert!(f::find(&tree, Role::Heading { level: 1 }, &headline).is_some());
        assert_eq!(tick, None);
        let plain = crate::shared::joined_brief(&words, &brief);
        assert_eq!(plain.as_ref(), b"Hello: Ready for review. shop: Ready for review.");
        let at = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::HOME_LEDE)).unwrap();
        assert_eq!(plain, f::node_text(&tree, at));
    }
    let facts = facts(vec![project(1, "Hello", 3, 3)]);
    let (_, tick) = crate::shared::all_home_headline(&f::words(), &facts, Wall::from_nanos(NOW));
    assert_eq!(tick, Some(Wall::from_nanos(NOW + MINUTE)));
}
