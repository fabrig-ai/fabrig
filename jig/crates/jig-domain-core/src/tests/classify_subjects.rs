//! Subject partitions preserve watch routing and gathered marks.
#![expect(clippy::disallowed_methods, reason = "table tests use ordinary Rust iterators (programming-model.md, 10.2)")]

use super::*;
use crate::documents::{Proposer, QueryOrder, SubjectShape as Shape, TaskQuery, Touch};
use jig_domain_people::ApplicationScope;

const PARTY: u64 = 1;
const OTHER_PARTY: u64 = 2;
const TASK: u64 = 7;
const CONNECTOR: u16 = 1;
const SUBJECT: Token = Token::new(50);

struct Row {
    subject: Subject,
    shape: Shape,
    // Party, other party, frames, settings, deployment, all accounts, own accounts, other accounts.
    marks: [bool; 8],
}

fn rows() -> Box<[Row]> {
    let no = [false; 8];
    let own = [true, false, true, false, false, true, true, false];
    let mut chips = List::with_capacity(2);
    chips.push(TASK).unwrap();
    chips.push(TASK + 1).unwrap();
    Box::new([
        Row {
            subject: Subject::Application {
                scope: ApplicationScope::Project(1),
                connector: CONNECTOR,
                subject: SUBJECT,
            },
            shape: Shape::Application { scope: ApplicationScope::Project(1), connector: CONNECTOR, subject: SUBJECT },
            marks: no,
        },
        Row { subject: Subject::Frame { party: PARTY }, shape: Shape::Other, marks: own },
        Row { subject: Subject::Task { task: TASK }, shape: Shape::Retained(TASK), marks: no },
        Row { subject: Subject::Tree { task: TASK }, shape: Shape::Retained(TASK), marks: no },
        Row { subject: Subject::Chips { tasks: chips }, shape: Shape::Chips, marks: no },
        Row {
            subject: Subject::Inbox { party: PARTY },
            shape: Shape::Other,
            marks: [true, false, false, false, false, false, false, false],
        },
        Row {
            subject: Subject::Accounts { party: PARTY },
            shape: Shape::Other,
            marks: [false, false, false, false, false, true, true, false],
        },
        Row {
            subject: Subject::Query {
                query: TaskQuery {
                    project: None,
                    requester: None,
                    root: None,
                    parent: None,
                    label: None,
                    phase: None,
                    executor: None,
                    tracked: None,
                    order: QueryOrder::Number,
                },
            },
            shape: Shape::Other,
            marks: no,
        },
        Row { subject: Subject::Run { task: TASK }, shape: Shape::Run(TASK), marks: no },
        Row {
            subject: Subject::Deployment,
            shape: Shape::Other,
            marks: [false, false, false, false, true, false, false, false],
        },
        Row {
            subject: Subject::Settings,
            shape: Shape::Other,
            marks: [false, false, false, true, false, false, false, false],
        },
        Row {
            subject: Subject::Proposal { proposer: Proposer::Task(TASK), number: 3 },
            shape: Shape::Other,
            marks: no,
        },
        Row {
            subject: Subject::Proposal { proposer: Proposer::Party(PARTY), number: 3 },
            shape: Shape::Other,
            marks: no,
        },
        Row { subject: Subject::Escalation { task: TASK }, shape: Shape::Other, marks: no },
        Row { subject: Subject::Question { task: TASK, message: 3 }, shape: Shape::Other, marks: no },
        Row { subject: Subject::PersonTask { task: TASK }, shape: Shape::Other, marks: no },
        Row { subject: Subject::Result { task: TASK }, shape: Shape::Other, marks: no },
        Row { subject: Subject::Project { project: 1 }, shape: Shape::Other, marks: no },
        Row { subject: Subject::Note { note: 3 }, shape: Shape::Other, marks: no },
    ])
}

fn window() -> Window {
    Window { rows: 1, after: None }
}

fn watched(subject: Subject) -> (Core, Token) {
    let (mut core, _) = crate::tests::core();
    let name = core.view_names.insert(WatchedSubject { subject, watchers: 1 }).unwrap();
    let watcher = core.fly(crate::routing::flight::Flight::Watch {
        watcher: Token::new(90),
        binding: WatchBinding { party: PARTY, sign_in: 10, subject: name, window: window() },
    });
    (core, watcher)
}

#[test]
fn subject_partitions_preserve_watch_routing() {
    for row in rows() {
        let (retained, application, chips) = match row.shape {
            Shape::Retained(task) => (Some(task), None, None),
            Shape::Application { scope, connector, subject } => {
                (None, Some((connector, subject, scope, PARTY, window())), None)
            }
            Shape::Chips => (None, None, Some([TASK, TASK + 1])),
            Shape::Run(_) | Shape::Other => (None, None, None),
        };
        assert_eq!(
            row.subject.chips().map(List::as_slice),
            chips.as_ref().map(<[u64; 2]>::as_slice),
            "{:?}",
            row.subject
        );
        assert_eq!(row.subject.shape(), row.shape, "{:?}", row.subject);
        let (mut core, watcher) = watched(row.subject.clone());
        assert_eq!(retained_subject(&row.subject), retained, "{:?}", row.subject);
        assert_eq!(application_subject(&core, watcher), application, "{:?}", row.subject);
        let mut refused = refused_subject(watcher, &row.subject, WatchRefusal::Busy);
        if let Some((connector, subject, ..)) = application {
            let Some(Request::Ask {
                connector: found_connector,
                ask: crate::Ask::DropDocument { subject: found_subject },
            }) = refused.pop()
            else {
                panic!("application document dropped before refusal: {:?}", row.subject);
            };
            assert_eq!((found_connector, found_subject), (connector, subject), "{:?}", row.subject);
        }
        let Some(Request::Now(now)) = refused.pop() else { panic!("watch refused: {:?}", row.subject) };
        let Now::WatchRefused { watcher: found, why } = *now else { panic!("watch refusal: {now:?}") };
        assert_eq!(found, watcher, "{:?}", row.subject);
        assert_eq!(why, WatchRefusal::Busy, "{:?}", row.subject);
        assert!(refused.is_empty(), "{:?}", row.subject);
        let mut names = List::with_capacity(1);
        names.push(SUBJECT).unwrap();
        let _out = marked(&mut core, CONNECTOR, names);
        assert_eq!(core.changed_views.len(), u32::from(application.is_some()), "{:?}", row.subject);
    }
}

#[test]
fn subject_partitions_preserve_gathered_marks() {
    for row in rows() {
        let (mut core, watcher) = watched(row.subject.clone());
        let name = core.watch_binding(watcher).unwrap().subject;
        let touches: [fn(&mut Core); 8] = [
            |core| core.touch_party(PARTY),
            |core| core.touch_party(OTHER_PARTY),
            Core::touch_frames,
            Core::touch_settings,
            Core::touch_deployment,
            |core| core.touch_accounts(None),
            |core| core.touch_accounts(Some(PARTY)),
            |core| core.touch_accounts(Some(OTHER_PARTY)),
        ];
        let changes = [
            Touch::Party(PARTY),
            Touch::Party(OTHER_PARTY),
            Touch::Frames,
            Touch::Settings,
            Touch::Deployment,
            Touch::Accounts(None),
            Touch::Accounts(Some(PARTY)),
            Touch::Accounts(Some(OTHER_PARTY)),
        ];
        for ((touch, change), expected) in touches.into_iter().zip(changes).zip(row.marks) {
            assert_eq!(row.subject.touched_by(change), expected, "{:?} {change:?}", row.subject);
            let _previous = core.changed_views.remove(&name);
            touch(&mut core);
            assert_eq!(core.changed_views.contains(&name), expected, "{:?}", row.subject);
        }
    }
}
