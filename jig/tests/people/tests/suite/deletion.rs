//! Deletion progress and retained project keys (jig/docs/design/domain/people.md, 5.2).
use jig_domain_people::{
    self as p, Act, ActGrant, ActKey, ActKind, Ask, DeletionStep, Domain, Event, Holding, Key, Outcome, Refusal, Reply,
    Request, RequestKey, Role, Stored,
};
use jig_people_world::{LIMITS, fixtures::people_member, identity};
use skein_lib::{Env, List, Queue, ReplyTo, Time, Token, Wall};
use std::collections::BTreeMap;

const PROJECT: u32 = 1;
const OTHER: u32 = 2;
const PERSON: u64 = 1;
const SIGN_IN: u64 = 10;
const CONNECTOR: u16 = 1;

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
        for row in [
            Stored::Person { number: PERSON, identity: identity(PERSON) },
            Stored::SignIn { number: SIGN_IN, person: PERSON, expires: Wall::from_nanos(1_000_000_000) },
            Stored::Roles { project: PROJECT, holdings: Box::new([Holding { person: PERSON, role: Role::Owner }]) },
            Stored::Roles { project: OTHER, holdings: Box::new([Holding { person: PERSON, role: Role::Owner }]) },
        ] {
            assert!(people.send(Event::Restore { record: row }).is_empty());
        }
        assert!(people.send(Event::Restored).is_empty());
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
fn a_deletion_row_is_kept_advanced_and_erased() {
    let mut people = People::new();
    for step in [DeletionStep::Tasks, DeletionStep::Rows, DeletionStep::Release] {
        let row = Stored::Deletion { project: PROJECT, step };
        assert_eq!(
            people.send(Event::Deleting { project: PROJECT, step: Some(step) }),
            [Request::Save { record: row.clone() }]
        );
        assert_eq!(people.rows.get(&Key::Deletion(PROJECT)), Some(&row));
        assert_eq!(people.domain.deletions().len(), 1);
        assert_eq!(people.domain.deletions().get(&PROJECT), Some(&step));
    }
    people.send(Event::Deleting { project: OTHER, step: Some(DeletionStep::Tasks) });
    assert_eq!(
        people.send(Event::Deleting { project: PROJECT, step: None }),
        [Request::Erase { key: Key::Deletion(PROJECT) }]
    );
    assert!(!people.rows.contains_key(&Key::Deletion(PROJECT)));
    assert_eq!(people.domain.deletions().len(), 1);
    assert!(people.domain.deletions().contains_key(&OTHER));
}

#[test]
fn a_kept_deletion_restores_at_start() {
    for selected in [false, true] {
        let mut people = People::new();
        people.domain = Domain::new(&LIMITS, Box::new([]), 1, Box::new([]));
        if selected {
            people.domain.start_restore(people.env.wall);
        }
        for project in 1..=LIMITS.projects {
            assert!(
                people
                    .send(Event::Restore { record: Stored::Deletion { project, step: DeletionStep::Rows } })
                    .is_empty()
            );
        }
        if selected {
            assert!(people.domain.start_link(&people.env));
            people.domain.start_sweep();
            assert!(people.domain.start_complete(&people.env));
        } else {
            assert!(people.send(Event::Restored).is_empty());
        }
        assert!(people.domain.ready());
        assert_eq!(people.domain.deletions().len(), LIMITS.projects);
        assert_eq!(people.domain.deletions().get(&PROJECT), Some(&DeletionStep::Rows));
    }
}

#[test]
fn too_many_deletion_rows_refuse_the_start() {
    for selected in [false, true] {
        let mut people = People::new();
        people.env.limits.projects = 1;
        people.domain = Domain::new(&people.env.limits, Box::new([]), 1, Box::new([]));
        if selected {
            people.domain.start_restore(people.env.wall);
        }
        people.send(Event::Restore { record: Stored::Deletion { project: PROJECT, step: DeletionStep::Tasks } });
        assert_eq!(
            people.send(Event::Restore { record: Stored::Deletion { project: OTHER, step: DeletionStep::Tasks } }),
            [Request::RestoreRefused {
                key: Key::Deletion(OTHER),
                refusal: Refusal::Limit { which: p::LimitKind::Restore }
            }]
        );
        assert!(!people.domain.ready());
        assert_eq!(people.domain.deletions().len(), 1);
        if selected {
            assert!(!people.domain.start_link(&people.env));
            assert!(people.domain.restore_failure().is_some());
        }
    }
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
            Ask::RemoveProject { project },
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
        (1, Act::RemoveProject { project: PROJECT }, Outcome::ProjectRemoved { project: PROJECT }),
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
    route(&people.act(1, Act::RemoveProject { project: PROJECT }));
    assert!(people.send(Event::ForgetProject { project: PROJECT }).is_empty());
}

#[test]
fn an_owner_may_ask_to_remove_a_project_and_a_member_may_not() {
    let mut people = People::new();
    let ask = Ask::RemoveProject { project: PROJECT };
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
        people.act(1, Act::RemoveProject { project: 0 }),
        [Request::Acted { to: to(), outcome: Outcome::Refused(Refusal::Unknown) }]
    );
    for project in [1, u32::MAX] {
        let request = route(&people.act(1, Act::RemoveProject { project }));
        people.send(Event::Decided { request, outcome: Outcome::ProjectRemoved { project } });
        people.send(Event::ForgetProject { project });
    }
}
