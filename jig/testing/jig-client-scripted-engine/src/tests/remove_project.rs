use super::*;

const OWNER: u64 = 1;
const MEMBER: u64 = 2;
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const TASK: u64 = 8;
const OTHER_TASK: u64 = 9;
const ME: Token = Token::new(1);
const TASK_WATCH: Token = Token::new(2);
// Deletion leaves the global inbox source and makes one keyed decision.
const REMAINING_SOURCES: usize = 1;
const DELETION_DECISIONS: usize = 1;

fn project_document() -> Project {
    Project {
        number: PROJECT,
        name: text(b"project"),
        period: 0,
        policy: Policy {
            ceiling: authority(),
            period: 0,
            spend: 0,
            roles: list(vec![]),
            requirements: list(vec![]),
            deciders: list(vec![]),
        },
        roles: list(vec![
            Holding { party: OWNER, role: HeldRole::Owner },
            Holding { party: MEMBER, role: HeldRole::Member },
        ]),
        spend: budget(),
        pools: list(vec![]),
        resources: list(vec![]),
        services: list(vec![]),
        recurring: list(vec![]),
        parts: list(vec![]),
        spend_lines: list(vec![]),
        acts: list(vec![]),
    }
}

fn owned() -> Engine {
    let mut setup = script();
    setup.truth.frames.get_mut(&OWNER).unwrap().projects.get_mut(0).unwrap().role = HeldRole::Owner;
    setup.truth.frames.get_mut(&MEMBER).unwrap().projects.get_mut(0).unwrap().role = HeldRole::Member;
    let mut other = setup.truth.frames[&OWNER].projects.get(0).unwrap().clone();
    other.project = OTHER_PROJECT;
    other.name = DocumentText::complete(text(b"Other"));
    let original = setup.truth.frames[&OWNER].projects.get(0).unwrap().clone();
    setup.truth.frames.get_mut(&OWNER).unwrap().projects = list(vec![original, other]);
    let mut document = task();
    document.project = PROJECT;
    setup.truth.objects.insert(ObjectKey::Task(TASK), ReadValue::Task(Box::new(document)));
    setup.truth.object_projects.insert(ObjectKey::Task(TASK), PROJECT);
    setup.truth.projects.insert(PROJECT, project_document());
    setup.truth.replies_seen.insert(OWNER, vec![ReadPosition { task: TASK, through: 1 }]);
    let mut kept = cursor_inbox_entry(OTHER_TASK, InboxGroup::Results, 0, None);
    kept.project = OTHER_PROJECT;
    let removed = cursor_inbox_entry(TASK, InboxGroup::Results, 0, None);
    setup.truth.sources.push(Source {
        party: OWNER,
        read: Read::Inbox { group: InboxReadGroup::Updates, project: None, kind: None, goal: None, after: None },
        rows: Rows::Inbox(vec![removed, kept]),
    });
    setup.truth.sources.push(Source {
        party: OWNER,
        read: Read::Activity { task: TASK, before: None },
        rows: Rows::Activity(vec![]),
    });
    Engine::new(setup, 1).unwrap()
}

#[test]
fn an_owner_removes_a_project_and_it_leaves_me() {
    let mut engine = owned();
    engine.open(Time::ZERO, 11, ME, Subject::Me).unwrap();
    engine.open(Time::ZERO, 11, TASK_WATCH, Subject::Task { task: TASK }).unwrap();
    drop(engine.poll(Time::ZERO));
    engine.taken(ME);
    engine.taken(TASK_WATCH);
    let ask = Ask::RemoveProject { project: PROJECT };
    send(&mut engine, 0, 11, 1, 3, ask.clone());
    let up = engine.poll(at(10_000_000));
    assert!(up.contains(&Up::Answer {
        attempt: Token::new(3),
        envelope: Envelope::Final,
        answer: Some(Answer::Done(Outcome::Owner(OwnerOutcome::ProjectRemoved { project: PROJECT }))),
    }));
    let remaining = &engine.truth().frames[&OWNER];
    assert_eq!(remaining.projects.iter().map(|p| p.project).collect::<Vec<_>>(), vec![OTHER_PROJECT]);
    assert!(engine.truth().frames[&MEMBER].projects.is_empty());
    assert!(engine.truth().objects.is_empty());
    assert!(engine.truth().object_projects.is_empty());
    assert!(engine.truth().projects.is_empty());
    assert!(engine.truth().replies_seen[&OWNER].is_empty());
    assert_eq!(engine.truth().sources.len(), REMAINING_SOURCES, "only the global source remains");
    let Rows::Inbox(rows) = &engine.truth().sources[0].rows else {
        panic!("remaining global inbox source");
    };
    assert_eq!(rows.iter().map(|row| row.project).collect::<Vec<_>>(), vec![OTHER_PROJECT]);
    assert!(
        up.contains(&Up::Watch { token: ME, event: WatchEvent::Delivered(Delivery::Me(Box::new(remaining.clone()))) })
    );
    assert!(up.contains(&Up::Ended { token: TASK_WATCH, why: WatchEnd::SubjectEnded }));
    send(&mut engine, 10_000_000, 11, 1, 4, ask);
    let replay = engine.poll(at(20_000_000));
    assert_eq!(
        final_answer(&replay[0]),
        &Answer::Done(Outcome::Owner(OwnerOutcome::ProjectRemoved { project: PROJECT }))
    );
    assert_eq!(engine.commits().len(), DELETION_DECISIONS, "replay makes no second deletion decision");
}

#[test]
fn a_member_who_is_no_owner_is_refused() {
    refused_as(MEMBER, PROJECT, Refusal::Role { held: Some(HeldRole::Member) });
}

#[test]
fn an_unknown_project_is_refused_as_unknown() {
    refused_as(OWNER, u32::MAX, Refusal::Unknown);
}

fn refused_as(party: u64, project: u32, refusal: Refusal) {
    let mut engine = owned();
    let before = engine.truth().clone();
    let (answer, changes) = engine.decide(party, &Ask::RemoveProject { project }, Wall::EPOCH).unwrap();
    assert_eq!(answer, Answer::Refused { refusal, at: None });
    assert!(changes.is_empty());
    assert_eq!(engine.truth(), &before);
}
