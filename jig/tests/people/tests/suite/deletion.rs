//! Deletion progress and retained project keys (jig/docs/design/domain/people/acts.md, 2).
use jig_domain_people::{
    self as p, Act, ActGrant, ActKey, ActKind, Ask, DeletionStep, Domain, Event, Holding, KeptDeletion, Key, Outcome,
    Refusal, Reply, Request, RequestKey, Role, Stored,
};
use jig_people_world::{
    LIMITS,
    fixtures::{CONNECTOR, PERSON, PROJECT, SIGN_IN, people_member},
    identity,
};
use skein_lib::{Env, List, Queue, ReplyTo, Time, Token, Wall};
use std::collections::BTreeMap;

const OTHER: u32 = 2;

struct People {
    domain: Domain,
    env: Env<p::Limits>,
    rows: BTreeMap<Key, Stored>,
}
impl People {
    fn new() -> Self {
        let env = Env { limits: LIMITS, now: Time::ZERO, wall: Wall::EPOCH };
        let domain = Domain::new(
            &env.limits,
            Box::new([p::InitialOwner { project: PROJECT, identity: identity(PERSON).key }]),
            1,
            Box::new([ActGrant { connector: CONNECTOR, may: ActKind::ALL.into() }]),
        );
        let mut people = Self { domain, env, rows: BTreeMap::new() };
        people.domain.start_restore(people.env.wall);
        for row in [
            Stored::Person { number: PERSON, identity: identity(PERSON) },
            Stored::SignIn { number: SIGN_IN, person: PERSON, expires: Wall::from_nanos(1_000_000_000) },
            Stored::Roles { project: PROJECT, holdings: Box::new([Holding { person: PERSON, role: Role::Owner }]) },
            Stored::Roles { project: OTHER, holdings: Box::new([Holding { person: PERSON, role: Role::Owner }]) },
        ] {
            assert!(people.send(Event::Restore { record: row }).is_empty());
        }
        assert!(people.domain.start_link());
        people.domain.start_sweep();
        assert!(people.domain.start_complete(&people.env));
        people
    }
    fn send(&mut self, event: Event) -> Vec<Request> {
        let mut out = Queue::uncapped();
        p::step(&mut self.domain, &self.env, event, &mut out);
        let mut requests = Vec::new();
        while let Some(request) = out.pop() {
            if let Request::Save { record } = &request {
                self.rows.insert(record.key(), record.clone());
            }
            if let Request::Erase { key } = &request {
                self.rows.remove(key);
            }
            requests.push(request);
        }
        self.domain.reclaim();
        requests
    }
    fn ask(&mut self, key: u8, ask: Ask) -> Vec<Request> {
        self.send(Event::Ask { reply_to: to(), sign_in: SIGN_IN, key: [key; 16], ask, models: Box::new([]) })
    }
    fn act(&mut self, key: u8, act: Act) -> Vec<Request> {
        self.send(Event::Act { reply_to: to(), connector: CONNECTOR, key: [key; 16], act: Box::new(act) })
    }
}
fn to() -> ReplyTo {
    ReplyTo::new(Token::new(1))
}
fn route(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|row| match row {
            Request::Route { request, .. } | Request::Act { request, .. } => Some(*request),
            Request::Acted { .. }
            | Request::ServiceMade { .. }
            | Request::RolesApplied { .. }
            | Request::Reply { .. }
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::RolesRefused { .. }
            | Request::RestoreRefused { .. } => None,
        })
        .expect("one admitted flight")
}
fn rename(project: u32) -> Ask {
    Ask::Rename { project, name: "Renamed".into() }
}
fn adoption() -> p::Adoption {
    p::Adoption {
        resource: p::ResourceName { connector: CONNECTOR, path: Box::new([Box::from(&b"home"[..])]) },
        role: p::ResourceRole::Owned,
        options: Box::new([]),
    }
}

#[test]
fn a_deleted_mark_is_kept_at_admission_after_completion_and_restart() {
    let mut people = People::new();
    let admitted = people.send(Event::Deleting { project: OTHER, step: Some(DeletionStep::Tasks), grace: None });
    let mark = Stored::Deleted { project: OTHER };
    assert!(admitted.contains(&Request::Save { record: mark.clone() }));
    assert!(people.domain.deleted(OTHER));
    for step in [DeletionStep::Rows, DeletionStep::Release] {
        people.send(Event::Deleting { project: OTHER, step: Some(step), grace: None });
        people.send(Event::ForgetProject { project: OTHER });
    }
    people.send(Event::Deleting { project: OTHER, step: None, grace: None });
    assert_eq!(people.rows.get(&Key::Deleted(OTHER)), Some(&mark));
    assert!(!people.rows.contains_key(&Key::Deletion(OTHER)));
    people.domain = Domain::new(&LIMITS, Box::new([]), 1, Box::new([]));
    people.domain.start_restore(people.env.wall);
    for row in people.rows.values().cloned().collect::<Vec<_>>() {
        assert!(people.send(Event::Restore { record: row }).is_empty());
    }
    assert!(people.domain.start_link());
    people.domain.start_sweep();
    assert!(people.domain.start_complete(&people.env));
    assert!(people.domain.deleted(OTHER));
    assert!(people.domain.deletions().is_empty());
}

#[test]
fn a_full_marks_set_erases_its_lowest_number_in_the_admitting_decision() {
    let mut people = People::new();
    for project in (1..=LIMITS.projects).rev() {
        people.send(Event::Deleting { project, step: Some(DeletionStep::Tasks), grace: None });
        people.send(Event::Deleting { project, step: None, grace: None });
    }
    assert!(people.domain.deleted(1));
    assert_eq!(people.rows.len(), usize::try_from(LIMITS.projects).expect("small project bound"));
    let project = LIMITS.projects + 1;
    let admitted = people.send(Event::Deleting { project, step: Some(DeletionStep::Tasks), grace: None });
    assert!(admitted.contains(&Request::Erase { key: Key::Deleted(1) }));
    assert!(admitted.contains(&Request::Save { record: Stored::Deleted { project } }));
    assert!(!people.domain.deleted(1));
    assert!(people.domain.deleted(project));
    assert!(!people.rows.contains_key(&Key::Deleted(1)));
    assert_eq!(people.rows.len(), usize::try_from(LIMITS.projects + 1).expect("marks and deletion"));
}

#[test]
fn a_deletion_row_is_kept_advanced_and_erased() {
    let grace = Some(skein_lib::Duration::from_millis(5000));
    let mut people = People::new();
    for step in [DeletionStep::Tasks, DeletionStep::Rows, DeletionStep::Release] {
        let row = Stored::Deletion { project: PROJECT, step, grace };
        let requests = people.send(Event::Deleting { project: PROJECT, step: Some(step), grace });
        assert!(requests.contains(&Request::Save { record: row.clone() }));
        assert_eq!(people.rows.get(&Key::Deletion(PROJECT)), Some(&row));
        assert_eq!(people.domain.deletions().len(), 1);
        assert_eq!(people.domain.deletions().get(&PROJECT), Some(&KeptDeletion { step, grace }));
    }
    people.send(Event::Deleting { project: OTHER, step: Some(DeletionStep::Tasks), grace });
    assert_eq!(
        people.send(Event::Deleting { project: PROJECT, step: None, grace }),
        [Request::Erase { key: Key::Deletion(PROJECT) }]
    );
    assert!(!people.rows.contains_key(&Key::Deletion(PROJECT)));
    assert_eq!(people.domain.deletions().len(), 1);
    assert!(people.domain.deletions().contains_key(&OTHER));
}

#[test]
fn a_kept_deletion_restores_at_start() {
    for selected in [DeletionStep::Tasks, DeletionStep::Rows, DeletionStep::Release] {
        let grace = Some(skein_lib::Duration::from_millis(5000));
        let mut people = People::new();
        people.domain = Domain::new(&LIMITS, Box::new([]), 1, Box::new([]));
        people.domain.start_restore(people.env.wall);
        for project in 1..=LIMITS.projects {
            assert!(
                people.send(Event::Restore { record: Stored::Deletion { project, step: selected, grace } }).is_empty()
            );
        }
        assert!(people.domain.start_link());
        people.domain.start_sweep();
        assert!(people.domain.start_complete(&people.env));
        assert!(people.domain.ready());
        assert_eq!(people.domain.deletions().len(), LIMITS.projects);
        assert_eq!(people.domain.deletions().get(&PROJECT), Some(&KeptDeletion { step: selected, grace }));
    }
}

#[test]
fn too_many_deletion_rows_refuse_the_start() {
    let mut people = People::new();
    people.env.limits.projects = 1;
    people.domain = Domain::new(&people.env.limits, Box::new([]), 1, Box::new([]));
    people.domain.start_restore(people.env.wall);
    people
        .send(Event::Restore { record: Stored::Deletion { project: PROJECT, step: DeletionStep::Tasks, grace: None } });
    assert_eq!(
        people.send(Event::Restore {
            record: Stored::Deletion { project: OTHER, step: DeletionStep::Tasks, grace: None }
        }),
        [Request::RestoreRefused {
            key: Key::Deletion(OTHER),
            refusal: Refusal::Limit { which: p::LimitKind::Restore }
        }]
    );
    assert!(!people.domain.ready());
    assert_eq!(people.domain.deletions().len(), 1);
    assert!(!people.domain.start_link());
    assert!(people.domain.restore_failure().is_some());
}

#[test]
fn a_closed_project_refuses_party_asks_as_one_never_made() {
    let mut people = People::new();
    let pending = route(&people.ask(99, rename(PROJECT)));
    assert_eq!(people.send(Event::CloseProject { project: PROJECT }), [Request::Erase { key: Key::Roles(PROJECT) }]);
    assert!(people.ask(99, rename(PROJECT)).is_empty(), "the routed flight still joins");
    assert!(!people.domain.has_project(PROJECT));
    assert_eq!(people.domain.role(PERSON, PROJECT), None);
    assert_eq!(
        &*people.domain.project_roles(PERSON).expect("known party").into_boxed(),
        &[p::ProjectRole { project: OTHER, role: Role::Owner }]
    );
    for project in [PROJECT, 99] {
        let mut operations = List::with_capacity(1);
        operations.push(p::Operation::Stop { task: 1 }).expect("one operation");
        for ask in [
            Ask::Compound { project, operations },
            Ask::EditNote {
                project,
                name: 1,
                scope: Box::new(p::NoteScope::Project),
                change: Box::new(p::NoteChange::Delete { recalled: 1 }),
            },
            Ask::MakeService { project, name: b"service".as_slice().into(), role: Role::Member },
            Ask::ChangePolicy { project, change: p::PolicyChange::ProjectSpend { period_spend: 1 } },
            Ask::SetPool { project, person: PERSON, budget: 1 },
            Ask::DecideEscalation { project, task: 1, revision: 1, decision: p::EscalationDecision::Pass },
            Ask::DecideProposal {
                project,
                proposer: p::Proposer::Party(PERSON),
                proposal: 1,
                decision: p::ProposalDecision::Pass,
                running: None,
            },
            Ask::AnswerQuestion { project, task: 1, question: 1, words: b"answer".as_slice().into() },
            Ask::Amend {
                project,
                task: 1,
                revision: 1,
                amendment: jig_people_world::fixtures::person_unchanged(b""),
                release: None,
            },
            Ask::Move { project, task: 1, reason: Box::new([]) },
            Ask::TakePerson { project, task: 1 },
            Ask::HandBackPerson { project, task: 1 },
            Ask::AnswerPerson {
                project,
                task: 1,
                revision: 1,
                result: p::PersonResult::Report { words: Box::new([]) },
            },
            Ask::Release { project, task: 1, revision: 1, again: None },
            Ask::Propose {
                project,
                action: p::Action::Batch(Box::new([people_member(b"work", 1)])),
                reason: Box::new([]),
            },
            Ask::Withdraw { project, proposal: 1 },
            Ask::SetRoles { project, holdings: Box::new([]) },
            rename(project),
            Ask::RemoveProject { project, grace: None },
            Ask::Adopt { project, adoption: adoption() },
            Ask::Create { project, batch: Box::new([people_member(b"work", 1)]) },
            Ask::Cancel { project, task: 1, grace: None, reason: Box::new([]) },
            Ask::Stop { project, task: 1 },
            Ask::Say { project, task: 1, words: b"hello".as_slice().into() },
            Ask::Application { scope: p::ApplicationScope::Project(project), connector: CONNECTOR, digest: [0; 32] },
        ] {
            assert_eq!(people.ask(1, ask), [Request::Reply { to: to(), reply: Reply::Refused(Refusal::Unknown) }]);
        }
    }
    let done = people.send(Event::Decided { request: pending, outcome: Outcome::Refused(Refusal::Unknown) });
    assert_eq!(
        done.iter().filter(|row| matches!(row, Request::Reply { .. })).count(),
        2,
        "routed flights still complete"
    );
}

#[test]
fn forgetting_a_project_erases_its_retained_answers_and_acts() {
    let mut people = People::new();
    for (key, ask, outcome) in [
        (1, rename(PROJECT), Outcome::Renamed { project: PROJECT }),
        (2, Ask::MakeProject { name: "Made".into(), policy: 1 }, Outcome::ProjectMade { project: PROJECT }),
        (3, rename(OTHER), Outcome::Renamed { project: OTHER }),
    ] {
        let request = route(&people.ask(key, ask));
        people.send(Event::Decided { request, outcome });
    }
    for (key, act, outcome) in [
        (1, Act::RemoveProject { project: PROJECT, grace: None }, Outcome::ProjectRemoved { project: PROJECT }),
        (
            2,
            Act::MakeProject { maker: PERSON, name: "Made".into(), policy: 1 },
            Outcome::ProjectMade { project: PROJECT },
        ),
        (3, Act::Adopt { project: OTHER, adoption: adoption() }, Outcome::Adopted { project: OTHER }),
        (
            4,
            Act::Close { project: PROJECT, task: 1, grace: None, reason: Box::new([]) },
            Outcome::Refused(Refusal::Unknown),
        ),
        (5, Act::Adopt { project: PROJECT, adoption: adoption() }, Outcome::Adopted { project: PROJECT }),
    ] {
        let request = route(&people.act(key, act));
        people.send(Event::Decided { request, outcome });
    }
    let erased = people.send(Event::ForgetProject { project: PROJECT });
    let expected: Vec<Key> = [1, 2]
        .into_iter()
        .map(|key| Key::Answer(RequestKey { person: PERSON, key: [key; 16] }))
        .chain([1, 2, 4, 5].into_iter().map(|key| Key::Acted(ActKey { connector: CONNECTOR, key: [key; 16] })))
        .collect();
    assert_eq!(erased, expected.iter().map(|key| Request::Erase { key: *key }).collect::<Vec<_>>());
    for key in expected {
        assert!(!people.rows.contains_key(&key));
    }
    assert_eq!(people.rows.len(), 2, "unrelated answer and act remain");
    assert!(matches!(
        people.ask(3, rename(OTHER)).as_slice(),
        [Request::Reply { reply: Reply::Outcome(Outcome::Renamed { project: OTHER }), .. }]
    ));
    route(&people.ask(1, rename(PROJECT)));
    route(&people.act(1, Act::RemoveProject { project: PROJECT, grace: None }));
    assert!(people.send(Event::ForgetProject { project: PROJECT }).is_empty());
}

#[test]
fn a_removal_answer_outlives_its_deletion_only_for_the_owner() {
    for connector in [false, true] {
        let mut people = People::new();
        let act = Act::RemoveProject { project: PROJECT, grace: None };
        let ask = Ask::RemoveProject { project: PROJECT, grace: None };
        let admitted = if connector { people.act(1, act.clone()) } else { people.ask(1, ask.clone()) };
        let request = route(&admitted);
        let outcome = Outcome::ProjectRemoved { project: PROJECT };
        people.send(Event::Deleting { project: PROJECT, step: Some(DeletionStep::Tasks), grace: None });
        people.send(Event::Decided { request, outcome: outcome.clone() });
        people.send(Event::CloseProject { project: PROJECT });
        for step in [DeletionStep::Rows, DeletionStep::Release] {
            people.send(Event::Deleting { project: PROJECT, step: Some(step), grace: None });
            people.send(Event::ForgetProject { project: PROJECT });
            let replay = if connector { people.act(1, act.clone()) } else { people.ask(1, ask.clone()) };
            let expected = if connector {
                Request::Acted { to: to(), outcome: outcome.clone() }
            } else {
                Request::Reply { to: to(), reply: Reply::Outcome(outcome.clone()) }
            };
            assert_eq!(replay, [expected], "removal answer during {step:?}");
        }
        let answer = if connector {
            Key::Acted(ActKey { connector: CONNECTOR, key: [1; 16] })
        } else {
            Key::Answer(RequestKey { person: PERSON, key: [1; 16] })
        };
        let erased = people.send(Event::Deleting { project: PROJECT, step: None, grace: None });
        assert_eq!(erased.contains(&Request::Erase { key: answer }), connector);
        assert_eq!(people.rows.contains_key(&answer), !connector);
        let replay = if connector { people.act(1, act) } else { people.ask(1, ask) };
        if connector {
            route(&replay); // The core judges the act afresh, as unknown.
        } else {
            assert_eq!(replay, [Request::Reply { to: to(), reply: Reply::Outcome(outcome) }]);
        }
        people.env.now = people.env.now.saturating_add(LIMITS.request_retention);
        let mut expired = Queue::uncapped();
        p::fire(&mut people.domain, &people.env, &mut expired);
        while let Some(request) = expired.pop() {
            if let Request::Erase { key } = request {
                people.rows.remove(&key);
            }
        }
        assert!(!people.rows.contains_key(&answer), "ordinary retention still expires the answer");
        assert!(!people.rows.contains_key(&Key::Deletion(PROJECT)));
    }
}

#[test]
fn an_owner_may_ask_to_remove_a_project_and_a_member_may_not() {
    let mut people = People::new();
    let ask = Ask::RemoveProject { project: PROJECT, grace: None };
    let request = route(&people.ask(1, ask.clone()));
    let outcome = Outcome::ProjectRemoved { project: PROJECT };
    people.send(Event::Decided { request, outcome: outcome.clone() });
    assert_eq!(people.ask(1, ask.clone()), [Request::Reply { to: to(), reply: Reply::Outcome(outcome) }]);
    for (key, role) in [(10, Role::Member), (11, Role::Maintainer), (12, Role::Observer)] {
        people.send(Event::Roles { project: PROJECT, holdings: Box::new([Holding { person: PERSON, role }]) });
        let rows = people.ask(key, ask.clone());
        assert!(rows.iter().any(|row| matches!(row, Request::Reply { reply: Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(held) })), .. } if *held == role)));
        assert!(!rows.iter().any(|row| matches!(row, Request::Route { .. })));
    }
    people.send(Event::Roles { project: PROJECT, holdings: Box::new([]) });
    let rows = people.ask(13, ask);
    assert!(rows.iter().any(|row| matches!(
        row,
        Request::Reply { reply: Reply::Outcome(Outcome::Refused(Refusal::Role { held: None })), .. }
    )));
    assert!(!rows.iter().any(|row| matches!(row, Request::Route { .. })));
}

#[test]
fn remove_project_is_valid_for_a_nonzero_project() {
    let mut people = People::new();
    assert_eq!(
        people.act(1, Act::RemoveProject { project: 0, grace: None }),
        [Request::Acted { to: to(), outcome: Outcome::Refused(Refusal::Unknown) }]
    );
    for project in [1, u32::MAX] {
        let request = route(&people.act(1, Act::RemoveProject { project, grace: None }));
        people.send(Event::Decided { request, outcome: Outcome::ProjectRemoved { project } });
        people.send(Event::ForgetProject { project });
    }
}
