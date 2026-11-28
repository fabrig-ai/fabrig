use super::{
    Ask, Connector, Declaration, Released, Up, effects, lifecycle, ops, parts, procedures, projections, requests,
    resources, settings, topics,
};
use skein_lib::{List, Token};

#[test]
fn empty_declaration_has_no_kinds_or_goal_projection() {
    const CONNECTOR: Connector = Connector(7);
    let declaration = Declaration::empty(CONNECTOR);
    assert_eq!(declaration.connector, CONNECTOR);
    assert!(declaration.resources.is_empty());
    assert!(declaration.effects.is_empty());
    assert!(!declaration.projects_goals);
}

#[test]
fn all_capabilities_can_be_used_from_the_contract_alone() {
    const TASK: u64 = 3;
    const GOAL: u64 = 2;
    const STEP: u64 = 1;
    const PERSON: u64 = 4;
    const PROJECT: u32 = 5;
    const SUBSCRIPTION: u64 = 6;
    const TOPIC: u64 = 7;
    const DOCUMENT_ROWS: u32 = 1; // One row in the requested window.
    const ENTRY: u64 = 7;
    const KEY: [u8; 16] = [1; 16];
    const OWNER: Token = Token::new(8);
    const REQUEST: Token = Token::new(9);
    const WATCHER: Token = Token::new(10);

    // Construction is the check: these values require no engine or child crate.
    let asks = [
        Ask::Lifecycle(lifecycle::Ask::CatchUp),
        Ask::Resources(resources::Ask::Unname { task: TASK }),
        Ask::Effects(effects::Ask::Describe { owner: OWNER }),
        Ask::Topics(topics::Ask::Refused { request: REQUEST }),
        Ask::Parts(parts::Ask::Document {
            watcher: WATCHER,
            scope: requests::ApplicationScope::Project(PROJECT),
            subject: OWNER,
            party: PERSON,
            window: parts::Window { rows: DOCUMENT_ROWS, after: None },
        }),
        Ask::Procedures(procedures::Ask::StepRefused {
            task: TASK,
            step: STEP,
            why: procedures::StepRefusal::Refused(ops::Problem {
                task: Some(TASK),
                why: ops::Refusal::NotStanding,
                blocked_by: None,
                capacity: None,
            }),
        }),
        Ask::Projections(projections::Ask::Forget { goal: GOAL }),
        Ask::Settings(settings::Ask::SaveDefaults { revision: None }),
        Ask::Requests(requests::Ask::Drop { owner: OWNER }),
    ];
    let up = [
        Up::Lifecycle(lifecycle::Up::Done { stage: lifecycle::RestartStage::Settled }),
        Up::Resources(resources::Up::Holdings { request: REQUEST, holdings: Some(Box::new([])), refused: None }),
        Up::Effects(effects::Up::Busy { owner: OWNER }),
        Up::Topics(topics::Up::News {
            task: TASK,
            subscription: SUBSCRIPTION,
            topic: TOPIC,
            class: topics::NewsClass::Wakes,
            words: Box::new([]),
        }),
        Up::Parts(parts::Up::Made { watcher: WATCHER, outcome: parts::Made::Ready }),
        Up::Procedures(procedures::Up::Waits { task: TASK, waits: List::with_capacity(0) }),
        Up::Projections(projections::Up::Settled { goal: GOAL }),
        Up::Settings(settings::Up::Judged { owner: OWNER, judged: Box::new([settings::Judged::Same]) }),
        Up::Requests(requests::Up::Decided { request: REQUEST, outcome: Err(ops::party::Again::Busy) }),
        Up::Act { key: KEY, act: Box::new(ops::Act::RemoveProject { project: PROJECT, grace: None }), carried: None },
    ];
    let released = [
        Released::Make { entry: ENTRY },
        Released::Answered { key: KEY, outcome: ops::Outcome::ProjectRemoved { project: PROJECT } },
    ];
    drop((asks, up, released));
}
