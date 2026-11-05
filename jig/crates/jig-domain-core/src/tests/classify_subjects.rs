//! Subject partitions preserve watch routing and gathered marks.
use super::*;
use crate::documents::{Proposer, QueryOrder, TaskQuery};
use jig_domain_people::ApplicationScope;

const PARTY: u64 = 1;
const OTHER_PARTY: u64 = 2;
const TASK: u64 = 7;
const CONNECTOR: u16 = 1;
const SUBJECT: Token = Token::new(50);

#[expect(dead_code, reason = "expected shape records document the watch partition")]
#[derive(Debug)]
enum Shape {
    Application { scope: ApplicationScope, connector: u16, subject: Token },
    Run(u64),
    Retained(u64),
    Chips,
    Other,
}

struct Row {
    subject: Subject,
    shape: Shape,
    retained: Option<u64>,
    application: Option<(u16, Token, ApplicationScope, u64, Window)>,
    // Party, other party, frames, settings, deployment, all accounts, own accounts, other accounts.
    marks: [bool; 8],
}

fn rows() -> Vec<Row> {
    let no = [false; 8];
    let own = [true, false, true, false, false, false, false, false];
    let mut chips = List::with_capacity(2);
    chips.push(TASK).unwrap();
    chips.push(TASK + 1).unwrap();
    vec![
        Row {
            subject: Subject::Application { scope: ApplicationScope::Project(1), connector: CONNECTOR, subject: SUBJECT },
            shape: Shape::Application { scope: ApplicationScope::Project(1), connector: CONNECTOR, subject: SUBJECT },
            retained: None,
            application: Some((CONNECTOR, SUBJECT, ApplicationScope::Project(1), PARTY, window())),
            marks: no,
        },
        Row { subject: Subject::Frame { party: PARTY }, shape: Shape::Other, retained: None, application: None, marks: own },
        Row { subject: Subject::Task { task: TASK }, shape: Shape::Retained(TASK), retained: Some(TASK), application: None, marks: no },
        Row { subject: Subject::Tree { task: TASK }, shape: Shape::Retained(TASK), retained: Some(TASK), application: None, marks: no },
        Row { subject: Subject::Chips { tasks: chips }, shape: Shape::Chips, retained: None, application: None, marks: no },
        Row { subject: Subject::Inbox { party: PARTY }, shape: Shape::Other, retained: None, application: None, marks: [true, false, false, false, false, false, false, false] },
        Row { subject: Subject::Accounts { party: PARTY }, shape: Shape::Other, retained: None, application: None, marks: [false, false, false, false, false, true, true, false] },
        Row { subject: Subject::Query { query: TaskQuery { project: None, requester: None, root: None, parent: None, label: None, phase: None, executor: None, tracked: None, order: QueryOrder::Number } }, shape: Shape::Other, retained: None, application: None, marks: no },
        Row { subject: Subject::Run { task: TASK }, shape: Shape::Run(TASK), retained: None, application: None, marks: no },
        Row { subject: Subject::Deployment, shape: Shape::Other, retained: None, application: None, marks: [false, false, false, false, true, false, false, false] },
        Row { subject: Subject::Settings, shape: Shape::Other, retained: None, application: None, marks: [false, false, false, true, false, false, false, false] },
        Row { subject: Subject::Proposal { proposer: Proposer::Task(TASK), number: 3 }, shape: Shape::Other, retained: None, application: None, marks: no },
        Row { subject: Subject::Proposal { proposer: Proposer::Party(PARTY), number: 3 }, shape: Shape::Other, retained: None, application: None, marks: no },
        Row { subject: Subject::Escalation { task: TASK }, shape: Shape::Other, retained: None, application: None, marks: no },
        Row { subject: Subject::Question { task: TASK, message: 3 }, shape: Shape::Other, retained: None, application: None, marks: no },
        Row { subject: Subject::PersonTask { task: TASK }, shape: Shape::Other, retained: None, application: None, marks: no },
        Row { subject: Subject::Result { task: TASK }, shape: Shape::Other, retained: None, application: None, marks: no },
        Row { subject: Subject::Project { project: 1 }, shape: Shape::Other, retained: None, application: None, marks: no },
        Row { subject: Subject::Note { note: 3 }, shape: Shape::Other, retained: None, application: None, marks: no },
    ]
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
        let (mut core, watcher) = watched(row.subject.clone());
        assert_eq!(retained_subject(&row.subject), row.retained, "{:?}", row.subject);
        assert_eq!(application_subject(&core, watcher), row.application, "{:?}", row.subject);
        let mut refused = refused_subject(watcher, &row.subject, WatchRefusal::Busy);
        let drops: Vec<_> = (&refused).into_iter().filter_map(|request| match request {
            Request::Ask { connector, ask: crate::Ask::DropDocument { subject } } => Some((*connector, *subject)),
            _ => None,
        }).collect();
        assert_eq!(drops, row.application.map(|(connector, subject, ..)| (connector, subject)).into_iter().collect::<Vec<_>>(), "{:?}", row.subject);
        assert!(refused.pop().is_some(), "{:?}", row.shape);
        let mut names = List::with_capacity(1);
        names.push(SUBJECT).unwrap();
        let _out = marked(&mut core, CONNECTOR, names);
        assert_eq!(core.changed_views.len(), u32::from(row.application.is_some()), "{:?}", row.subject);
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
        for (touch, expected) in touches.into_iter().zip(row.marks) {
            let _previous = core.changed_views.remove(&name);
            touch(&mut core);
            assert_eq!(core.changed_views.contains(&name), expected, "{:?}", row.subject);
        }
    }
}
