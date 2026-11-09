//! Run writes and deletions of party-scoped preferences (`jig/docs/design/domain/engine.md`, 10.1).
use crate::settings_support::{self as support, ask, run_configuration, settings_run_with_limits};
use jig_core_world::{fixtures, world::World};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{List, ReplyTo, Token};
use std::collections::BTreeMap;

// Preference bounds in domain/engine.md, 10.1, supplied by the settings fixture.
const PARTY_BYTES: usize = 120;
const PARTY_ENTRIES: u32 = 12;
const OTHER_PERSON: u64 = support::OWNER + 1;

#[derive(Clone, Copy, Debug)]
enum Caller {
    Person,
    WithoutAsking,
    Delegate,
}

fn caller(mode: Caller) -> (World, notes::Scope) {
    let (mut world, signed, task) = settings_run_with_limits(true, None, run_configuration().1);
    let asking = core::ToolFamilies::JIG.ask.0;
    if !matches!(mode, Caller::WithoutAsking) {
        let people::Ask::Create { batch, .. } = fixtures::chat(&world, 1, b"Keep preferences".as_slice().into()) else {
            panic!("creation fixture")
        };
        let mut granted = batch[0].authority.clone();
        granted.spend = 40;
        granted.notes = 0;
        granted.note_resources = Box::new([]);
        granted.tools = asking | core::ToolFamilies::JIG.delegate.0;
        let amendment = people::Amendment {
            authority: Some(granted),
            ..jig_test_profile::fixtures::person_unchanged(b"Keep preferences")
        };
        let revision = fixtures::live_row(&world, task).revisions.spec;
        let reply =
            ask(&mut world, signed, 102, people::Ask::Amend { project: 1, task, revision, amendment, release: None });
        assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Amended { .. })), "{reply:?}");
    }
    if matches!(mode, Caller::Delegate) {
        let run = fixtures::run(&world);
        let mut member = fixtures::member(&world, tasks::Executor::Agent { charter: 1 }, b"Explain delegated work");
        member.authority.tools = tasks::Tools(asking);
        member.authority.notes = tasks::Scopes(0);
        member.authority.note_resources = Box::new([]);
        member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
        let child = fixtures::delegate(&mut world, run, 80, member);
        fixtures::finish_chat(&mut world, 1, b"The delegate continues");
        assert_eq!(world.assigned.as_ref().expect("delegate assigned").task, child);
        assert_eq!(fixtures::live_row(&world, child).requester, tasks::Party::Task(task));
    } else {
        assert_eq!(fixtures::live_row(&world, task).requester, tasks::Party::Person(support::OWNER));
    }
    let row = fixtures::live_row(&world, world.call_key(1).task);
    assert_eq!(row.authority.tools.0 & asking != 0, !matches!(mode, Caller::WithoutAsking));
    (world, notes::Scope::Party { party: support::OWNER })
}

fn note(scope: &notes::Scope, name: u64, description: &[u8], body: &[u8], recalled: Option<u32>) -> core::NamedAction {
    core::NamedAction::Note {
        entry: notes::New {
            name,
            scope: scope.clone(),
            description: description.into(),
            body: body.into(),
            references: List::with_capacity(0),
            // The route must replace a caller-supplied author with its run's task and attempt.
            author: notes::Author::Party { party: support::OWNER },
        },
        recalled,
    }
}

fn call(world: &mut World, completion: u32, action: core::NamedAction) -> core::CallPart {
    let key = world.call_key(completion);
    let token = Token::new(u64::from(completion));
    world.send(root::Event::Core(core::Event::NamedAction { to: ReplyTo::new(token), key, action }));
    let (to, found, answer) = world.answers.last().expect("named note answer");
    assert_eq!(*to, token);
    assert_eq!(*found, key);
    let stored = &world.store.rows[&root::Key::Core(core::Key::Core(core::CoreKey::Call(key)))];
    let root::Record::Core(core::Record::Core(core::CoreRecord::Call(record))) = stored else {
        panic!("call recorded")
    };
    assert_eq!(&record.part, answer);
    answer.clone()
}

fn stored_notes(world: &World) -> BTreeMap<root::Key, root::Record> {
    world
        .store
        .rows
        .iter()
        .filter(|(key, _)| matches!(key, root::Key::Core(core::Key::Notes(_))))
        .map(|(key, row)| (key.clone(), row.clone()))
        .collect()
}

fn written(
    world: &mut World,
    completion: u32,
    action: core::NamedAction,
    scope: &notes::Scope,
    description: &[u8],
    revision: u32,
) -> u64 {
    let before = world.env().wall;
    let answer = call(world, completion, action);
    let core::CallPart::NoteWritten { name, revision: found, lines: 0, added: 0, removed: 0, first: 1 } = answer else {
        panic!("preference written: {answer:?}")
    };
    assert_eq!(found, revision);
    let key = world.call_key(completion);
    let stored = &world.store.rows[&root::Key::Core(core::Key::Notes(notes::Key::Entry { name }))];
    let root::Record::Core(core::Record::Notes(notes::Record::Entry(entry))) = stored else { panic!("stored entry") };
    assert!(entry.changed >= before && entry.changed <= world.env().wall, "time belongs to this decision");
    let expected = notes::Entry {
        name,
        scope: scope.clone(),
        description: description.into(),
        body: Box::new([]),
        references: List::with_capacity(0),
        author: notes::Author::Task { task: key.task, attempt: key.attempt },
        revision,
        changed: entry.changed,
    };
    assert_eq!(
        world.store.rows[&root::Key::Core(core::Key::Notes(notes::Key::Entry { name }))],
        root::Record::Core(core::Record::Notes(notes::Record::Entry(expected)))
    );
    let line = notes::Line { name, scope: scope.clone(), description: description.into(), revision };
    assert_eq!(
        world.store.rows[&root::Key::Core(core::Key::Notes(notes::Key::Line { scope: scope.clone(), name }))],
        root::Record::Core(core::Record::Notes(notes::Record::Line(line)))
    );
    name
}

fn replay(world: &mut World, completion: u32, action: core::NamedAction, expected: &core::CallPart) {
    let rows = world.store.rows.clone();
    let applied = world.store.applied;
    assert_eq!(&call(world, completion, action), expected);
    assert_eq!(world.store.rows, rows, "replay changes no durable row");
    assert_eq!(world.store.applied, applied, "replay commits nothing");
}

fn preferences(world: &mut World, scope: &notes::Scope) {
    let line = [b'x'; PARTY_BYTES];
    let name = written(world, 1, note(scope, 0, &line, b"", None), scope, &line, 1);
    assert_eq!(stored_notes(world).len(), 2);
    let kept = world.answers.last().expect("kept").2.clone();
    replay(world, 1, note(scope, 0, b"different replay payload", b"", None), &kept);
    written(world, 2, note(scope, name, b"Use short answers", b"", Some(1)), scope, b"Use short answers", 2);
    assert_eq!(stored_notes(world).len(), 2);
    let changed = world.answers.last().expect("changed").2.clone();
    replay(world, 2, note(scope, name, b"stale replay payload", b"", Some(1)), &changed);
    let rows = stored_notes(world);
    for (completion, action) in [
        (3, note(scope, name, b"stale", b"", Some(1))),
        (4, core::NamedAction::NoteDelete { scope: scope.clone(), name, recalled: 1 }),
    ] {
        assert_eq!(call(world, completion, action), core::CallPart::NoteRefused(notes::Refusal::Moved { revision: 2 }));
        assert_eq!(stored_notes(world), rows);
    }
    let drop = core::NamedAction::NoteDelete { scope: scope.clone(), name, recalled: 2 };
    assert_eq!(
        call(world, 5, core::NamedAction::NoteDelete { scope: scope.clone(), name, recalled: 2 }),
        core::CallPart::NoteDeleted { name }
    );
    assert!(stored_notes(world).is_empty(), "entry and index line erased together");
    replay(world, 5, drop, &core::CallPart::NoteDeleted { name });
    replay(world, 1, note(scope, 0, &line, b"", None), &kept);
    for completion in 10..10 + PARTY_ENTRIES {
        written(world, completion, note(scope, 0, b"A preference", b"", None), scope, b"A preference", 1);
    }
    assert_eq!(
        stored_notes(world).len(),
        usize::try_from(PARTY_ENTRIES).expect("fixture count") * 2,
        "twelve entries and their index lines"
    );
    for (completion, action, refusal) in [
        (22, note(scope, 0, b"Thirteenth", b"", None), notes::Refusal::Full),
        (23, note(scope, 0, &[b'x'; PARTY_BYTES + 1], b"", None), notes::Refusal::PastBound { bytes: 121, bound: 120 }),
        (24, note(scope, 0, b"Body", b"x", None), notes::Refusal::PastBound { bytes: 1, bound: 0 }),
        (
            25,
            core::NamedAction::NoteLines {
                entry: notes::New {
                    name: 1,
                    scope: scope.clone(),
                    description: b"Range".as_slice().into(),
                    body: Box::new([]),
                    references: List::with_capacity(0),
                    author: notes::Author::Party { party: support::OWNER },
                },
                recalled: 1,
                range: notes::LineRange { first: 1, remove: 0, join: false },
            },
            notes::Refusal::PastBound { bytes: 0, bound: 0 },
        ),
    ] {
        let before = stored_notes(world);
        assert_eq!(call(world, completion, action), core::CallPart::NoteRefused(refusal));
        assert_eq!(stored_notes(world), before, "bounds leave preferences unchanged");
    }
}

#[test]
fn a_run_keeps_its_persons_preferences_and_no_one_elses() {
    let (mut world, scope) = caller(Caller::Person);
    preferences(&mut world, &scope);
    for (completion, scope) in [
        (30, notes::Scope::Deployment),
        (31, notes::Scope::Project { project: 1 }),
        (32, notes::Scope::Goal { project: 1, goal: 1 }),
        (
            33,
            notes::Scope::Resources {
                project: 1,
                connector: 1,
                pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
            },
        ),
    ] {
        let rows = stored_notes(&world);
        assert_eq!(
            call(&mut world, completion, core::NamedAction::NoteDelete { scope, name: 1, recalled: 1 }),
            core::CallPart::ToolDenied {
                answer: authority::Answer::Refuse,
                findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }])
            }
        );
        assert_eq!(stored_notes(&world), rows);
    }
    refused(&mut world, &notes::Scope::Party { party: OTHER_PERSON });
    world.broken();
    drop(world);
    for mode in [Caller::WithoutAsking, Caller::Delegate] {
        let (mut world, scope) = caller(mode);
        refused(&mut world, &scope);
        world.broken();
    }
}

fn refused(world: &mut World, scope: &notes::Scope) {
    for (completion, action) in [
        (40, note(scope, 0, b"A preference", b"", None)),
        (41, note(scope, 1, b"Changed", b"", Some(1))),
        (42, core::NamedAction::NoteDelete { scope: scope.clone(), name: 1, recalled: 1 }),
    ] {
        let rows = stored_notes(world);
        let denied = core::CallPart::ToolDenied {
            answer: authority::Answer::Refuse,
            findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
        };
        assert_eq!(call(world, completion, action), denied);
        assert_eq!(stored_notes(world), rows);
        replay(world, completion, note(scope, 0, b"Replay", b"", None), &denied);
    }
}
