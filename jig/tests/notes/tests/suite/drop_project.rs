//! Project deletion pages the project's index lines and erases their entries
//! (jig/docs/design/domain/engine.md, 10).

use jig_domain_notes::{Event, Key, Last, Pattern, Record, Scope};
use jig_notes_world::{Answer, PROJECT, World, one_scope, saved_entry, task_entry};
use skein_lib::Token;

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
        (Scope::Party { party: u64::from(PROJECT) }, vec![34]),
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

fn in_project(key: &Key, record: &Record) -> bool {
    let scope = match key {
        Key::Entry { .. } => match record {
            Record::Entry(entry) => &entry.scope,
            Record::Line(_) => unreachable!("an entry key holds an entry"),
        },
        Key::Line { scope, .. } => scope,
    };
    match scope {
        Scope::Deployment | Scope::Party { .. } => false,
        Scope::Project { project } | Scope::Goal { project, .. } | Scope::Resources { project, .. } => {
            *project == PROJECT
        }
    }
}

#[test]
fn dropping_a_project_erases_its_scopes_lines_and_entries() {
    let mut world = fixture();
    let kept: Vec<_> = world
        .records()
        .iter()
        .filter(|(key, record)| !in_project(key, record))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for scope in [Scope::Project { project: PROJECT }, Scope::Goal { project: PROJECT, goal: 11 }] {
        let _answer = world.call(Event::Index { owner: OWNER, scopes: one_scope(scope), most: 3 });
    }
    let commits = world.commits();
    assert_eq!(world.call(Event::DropProject { owner: OWNER, project: PROJECT }), Answer::ProjectDropped);
    assert!(world.records().iter().all(|(key, record)| !in_project(key, record)));
    assert_eq!(world.records(), &kept.into_iter().collect());
    assert_eq!(world.commits() - commits, 4, "eight lines and their entries fit four two-row pages");
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
fn an_entry_without_an_index_line_is_not_scanned() {
    let mut world = fixture();
    let entry = saved_entry(40, Scope::Project { project: PROJECT }, 1);
    world.seed(Record::Entry(entry.clone()));
    assert_eq!(world.call(Event::DropProject { owner: OWNER, project: PROJECT }), Answer::ProjectDropped);
    assert_eq!(world.stored_entry(entry.name), Some(&entry));
    world.restart();
    assert_eq!(world.stored_entry(entry.name), Some(&entry), "an unindexed entry survives restart");
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
        let kept = world
            .records()
            .iter()
            .filter(|(key, record)| !in_project(key, record))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        world.cut_after(Event::DropProject { owner: OWNER, project: PROJECT }, cut);
        assert_eq!(
            world.call(Event::DropProject { owner: OWNER, project: PROJECT }),
            Answer::ProjectDropped,
            "cut {cut}"
        );
        assert_eq!(world.records(), &kept, "cut {cut} preserves all unrelated lines and entries");
    }
}
