//! Project deletion retains globally named entries and pages the project's
//! index lines (jig/docs/design/domain/engine.md, 10).

use jig_domain_notes::{Event, Key, Last, Line, Pattern, Recall, Record, Scope};
use jig_notes_world::{Answer, World, one_scope, task_entry};
use skein_lib::Token;

const PROJECT: u32 = 7;
const OWNER: Token = Token::new(100);

fn fixture() -> World {
    let mut world = World::default();
    for (scope, names) in [
        (Scope::Project { project: PROJECT }, vec![20, 21, 22]),
        (Scope::Goal { project: PROJECT, goal: 11 }, vec![1, 2]),
        (Scope::Goal { project: PROJECT, goal: 12 }, vec![3]),
        (
            Scope::Resources {
                project: PROJECT,
                connector: 1,
                pattern: Pattern {
                    segments: Box::new([Box::from(b"area".as_slice())]),
                    last: Last::Open(Box::new([])),
                },
            },
            vec![4, 5],
        ),
        (Scope::Deployment, vec![30]),
        (Scope::Project { project: 8 }, vec![31]),
        (Scope::Goal { project: 8, goal: 11 }, vec![32]),
        (
            Scope::Resources {
                project: 8,
                connector: 1,
                pattern: Pattern { segments: Box::new([]), last: Last::Exact(Box::from(b"area".as_slice())) },
            },
            vec![33],
        ),
    ] {
        for name in names {
            assert_eq!(
                world.call(Event::Write {
                    owner: Token::new(name),
                    entry: task_entry(name, scope.clone(), b"note", b"body", 9),
                    recalled: None,
                }),
                Answer::Written { name, revision: 1 }
            );
        }
    }
    world
}

fn in_project(key: &Key) -> bool {
    match key {
        Key::Entry { .. } => false,
        Key::Line { scope, .. } => match scope {
            Scope::Deployment => false,
            Scope::Project { project } | Scope::Goal { project, .. } | Scope::Resources { project, .. } => {
                *project == PROJECT
            }
        },
    }
}

#[test]
fn dropping_a_project_erases_its_scopes_lines_and_retains_entries() {
    let mut world = fixture();
    let kept: Vec<_> =
        world.records().iter().filter(|(key, _)| !in_project(key)).map(|(k, v)| (k.clone(), v.clone())).collect();
    for scope in [Scope::Project { project: PROJECT }, Scope::Goal { project: PROJECT, goal: 11 }] {
        let _answer = world.call(Event::Index { owner: OWNER, scopes: one_scope(scope), most: 3 });
    }
    let commits = world.commits();
    assert_eq!(world.call(Event::DropProject { owner: OWNER, project: PROJECT }), Answer::ProjectDropped);
    assert!(world.records().keys().all(|key| !in_project(key)));
    assert_eq!(world.records(), &kept.into_iter().collect());
    assert_eq!(world.commits() - commits, 4, "eight erases fit four two-row decisions");
    for scope in [Scope::Project { project: PROJECT }, Scope::Goal { project: PROJECT, goal: 11 }] {
        let Answer::Indexed { lines, more } =
            world.call(Event::Index { owner: OWNER, scopes: one_scope(scope), most: 3 })
        else {
            panic!("deleted scope is indexed");
        };
        assert!(lines.is_empty(), "cached lines are invalidated");
        assert_eq!(more, 0);
    }
    assert_eq!(world.call(Event::DropProject { owner: OWNER, project: PROJECT }), Answer::ProjectDropped);
}

#[test]
fn an_entry_shared_with_another_scope_keeps_that_line() {
    let mut world = fixture();
    let entry = world.stored_entry(20).expect("project entry").clone();
    for scope in [Scope::Deployment, Scope::Project { project: 8 }] {
        world.seed(Record::Line(Line {
            scope,
            name: entry.name,
            description: entry.description.clone(),
            revision: entry.revision,
        }));
    }
    let kept =
        world.records().iter().filter(|(key, _)| !in_project(key)).map(|(k, v)| (k.clone(), v.clone())).collect();
    assert_eq!(world.call(Event::DropProject { owner: OWNER, project: PROJECT }), Answer::ProjectDropped);
    assert_eq!(world.records(), &kept);
    let Answer::Recalled { entries, more: false } =
        world.call(Event::Recall { owner: OWNER, by: Recall::Name { name: 20 }, page: 0 })
    else {
        panic!("shared entry is recalled");
    };
    assert_eq!(entries.get(0), Some(&entry));
    world.restart();
    assert_eq!(world.records(), &kept, "all remaining records survive restart");
}

#[test]
fn a_project_with_no_notes_answers_at_once() {
    let mut world = World::default();
    assert_eq!(world.call(Event::DropProject { owner: OWNER, project: PROJECT }), Answer::ProjectDropped);
    assert_eq!(world.commits(), 0);
    assert_eq!(world.trace().iter().filter(|line| line.contains("out Load")).count(), 1, "first empty page completes");
}

#[test]
fn a_drop_sent_again_after_a_cut_finishes() {
    for cut in 0..=5 {
        let mut world = fixture();
        let kept =
            world.records().iter().filter(|(key, _)| !in_project(key)).map(|(k, v)| (k.clone(), v.clone())).collect();
        world.cut_after(Event::DropProject { owner: OWNER, project: PROJECT }, cut);
        assert_eq!(
            world.call(Event::DropProject { owner: OWNER, project: PROJECT }),
            Answer::ProjectDropped,
            "cut {cut}"
        );
        assert_eq!(world.records(), &kept, "cut {cut} preserves all unrelated lines and every entry");
    }
}
