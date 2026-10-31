//! Authority translations preserve the child values (jig/docs/design/domain/authority.md, 2–4).
use super::*;
use crate::translate as values;

fn people_pattern(open: bool) -> people::Pattern {
    let bytes = Box::from(b"last\0".as_slice());
    people::Pattern {
        segments: Box::new([Box::from(b"first".as_slice()), Box::from(b"second".as_slice())]),
        last: if open { people::Last::Open(bytes) } else { people::Last::Exact(bytes) },
    }
}

fn tasks_pattern(open: bool) -> tasks::Pattern {
    let bytes = Box::from(b"last\0".as_slice());
    tasks::Pattern {
        segments: Box::new([Box::from(b"first".as_slice()), Box::from(b"second".as_slice())]),
        last: if open { tasks::Last::Open(bytes) } else { tasks::Last::Exact(bytes) },
    }
}

fn people_value() -> people::Authority {
    people::Authority {
        tools: 0b101,
        models: Some(people::Models::Named(0b101)),
        grants: Box::new([
            people::Grant { connector: 1, kind: 2, pattern: people_pattern(false) },
            people::Grant { connector: 3, kind: 4, pattern: people_pattern(true) },
        ]),
        delegation: people::Delegation {
            kinds: Box::new([people::Executor::Charter(1), people::Executor::Procedure(2), people::Executor::Role(3)]),
            tasks: 17,
            depth: 5,
        },
        spend: 123,
        deadline: Some(Wall::from_nanos(456)),
        notes: 1,
        note_resources: Box::new([
            people::ResourceScope { connector: 5, pattern: people_pattern(true) },
            people::ResourceScope { connector: 6, pattern: people_pattern(false) },
        ]),
    }
}

fn tasks_value() -> tasks::Authority {
    tasks::Authority {
        tools: tasks::Tools(0b101),
        models: tasks::Models::Named(0b101),
        grants: Box::new([
            tasks::Grant { connector: 1, kind: 2, pattern: tasks_pattern(false) },
            tasks::Grant { connector: 3, kind: 4, pattern: tasks_pattern(true) },
        ]),
        delegation: tasks::Delegation {
            kinds: Box::new([
                tasks::AuthorityExecutor::Charter(1),
                tasks::AuthorityExecutor::Procedure(2),
                tasks::AuthorityExecutor::Role(3),
            ]),
            tasks: 17,
            depth: 5,
        },
        budget: tasks::Budget { spend: 123, deadline: Some(Wall::from_nanos(456)) },
        notes: tasks::Scopes(1),
        note_resources: Box::new([
            tasks::ResourceScope { connector: 5, pattern: tasks_pattern(true) },
            tasks::ResourceScope { connector: 6, pattern: tasks_pattern(false) },
        ]),
    }
}

#[test]
fn patterns_round_trip_literal_and_open_bytes() {
    for open in [false, true] {
        for empty in [false, true] {
            let mut people = people_pattern(open);
            let mut tasks = tasks_pattern(open);
            if empty {
                people.segments = Box::new([]);
                tasks.segments = Box::new([]);
                people.last = if open { people::Last::Open(Box::new([])) } else { people::Last::Exact(Box::new([])) };
                tasks.last = if open { tasks::Last::Open(Box::new([])) } else { tasks::Last::Exact(Box::new([])) };
            }
            let authority = values::pattern_to_authority(people.clone());
            assert_eq!(pattern_to_people(&authority), people);
            assert_eq!(values::task_pattern_to_authority(&tasks), authority);
            assert_eq!(values::authority_pattern_to_tasks(&authority), tasks);
        }
    }
}

#[test]
fn grants_and_requirements_round_trip_their_patterns_and_guards() {
    for open in [false, true] {
        let grant = people::Grant { connector: u16::MAX, kind: 9, pattern: people_pattern(open) };
        let authority = values::grant_to_authority(grant.clone());
        assert_eq!(values::grant_to_people(&authority), grant);
        assert_eq!(grants_to_authority(Box::new([grant])).unwrap().as_ref(), &[authority]);
        for guard in [people::Guard::Guarded, people::Guard::Observed { freshness: Duration::from_secs(19) }] {
            for must_be_guarded in [false, true] {
                let requirement = people::Requirement {
                    connector: 7,
                    kind: 8,
                    pattern: people_pattern(open),
                    judge: people::Judge { connector: 9, requirement: 10, parameters: u32::MAX },
                    guard,
                    must_be_guarded,
                };
                assert_eq!(values::requirement_to_people(&requirement_to_authority(requirement.clone())), requirement);
            }
        }
    }
}

#[test]
fn people_authority_round_trip_preserves_every_part_and_explicit_model_selector() {
    for models in
        [people::Models::NONE, people::Models::Named(0b101), people::Models::Named(u64::MAX), people::Models::ALL]
    {
        for deadline in [None, Some(Wall::from_nanos(456))] {
            let mut people = people_value();
            people.models = Some(models);
            people.deadline = deadline;
            let translated = people_authority(people.clone()).unwrap();
            assert_eq!(authority_to_people(&translated).unwrap(), people);
            let mut tasks = tasks_value();
            tasks.models = tasks::Models::from_parts(models.bits(), models.is_all());
            tasks.budget.deadline = deadline;
            assert_eq!(task_authority(&translated), tasks);
        }
    }
    let mut omitted = people_value();
    omitted.models = None;
    let translated = people_authority(omitted.clone()).unwrap();
    assert_eq!(translated.models, authority::Models::NONE);
    omitted.models = Some(people::Models::NONE);
    assert_eq!(authority_to_people(&translated).unwrap(), omitted);
}

#[test]
fn task_authority_round_trip_preserves_every_part_and_model_selector() {
    for models in [tasks::Models::NONE, tasks::Models::Named(0b101), tasks::Models::Named(u64::MAX), tasks::Models::ALL]
    {
        for deadline in [None, Some(Wall::from_nanos(456))] {
            let mut tasks = tasks_value();
            tasks.models = models;
            tasks.budget.deadline = deadline;
            let translated = values::tasks_authority(&tasks);
            assert_eq!(task_authority(&translated), tasks);
            let mut people = people_value();
            people.models = Some(people::Models::from_parts(models.bits(), models.is_all()));
            people.deadline = deadline;
            assert_eq!(authority_to_people(&translated).unwrap(), people);
        }
    }
}

#[test]
fn policy_projection_snapshot_uses_the_same_grant_translation() {
    let people = people_value();
    let authority = people_authority(people.clone()).unwrap();
    let policy = authority::Policy {
        ceiling: authority.clone(),
        period_spend: 0,
        escalation_role: None,
        projections: authority.grants,
        roles: Box::new([]),
        requirements: Box::new([]),
    };
    assert_eq!(policy_snapshot(&policy, &[], &[]).unwrap().projections, people.grants);
}

#[test]
fn amendment_authority_preserves_model_inheritance_and_admission_bounds() {
    let mut limits = limits().tasks;
    limits.authority_grants = 2;
    limits.authority_segments = 2;
    limits.executor_kinds = 3;
    let models = tasks::Models::Named(0b10);
    assert_eq!(amendments::granted(people_value(), models, &limits), Some(tasks_value()));
    let mut omitted = people_value();
    omitted.models = None;
    let mut expected = tasks_value();
    expected.models = models;
    assert_eq!(amendments::granted(omitted, models, &limits), Some(expected));
    let mut too_few_grants = limits;
    too_few_grants.authority_grants = 1;
    let mut too_few_segments = limits;
    too_few_segments.authority_segments = 1;
    let mut too_few_executors = limits;
    too_few_executors.executor_kinds = 2;
    for bounds in [too_few_grants, too_few_segments, too_few_executors] {
        assert_eq!(amendments::granted(people_value(), models, &bounds), None);
    }
    let mut scopes_only = people_value();
    scopes_only.grants = Box::new([]);
    assert_eq!(amendments::granted(scopes_only.clone(), models, &too_few_grants), None);
    assert_eq!(amendments::granted(scopes_only, models, &too_few_segments), None);
}

#[test]
fn task_and_project_documents_show_the_same_authority_line() {
    let (mut core, mut limits) = tree_core();
    let tasks = tasks_value();
    let authority = values::tasks_authority(&tasks);
    limits.documents.grants = 2;
    limits.documents.executors = 3;
    limits.authority.grants = 2;
    limits.authority.executors = 3;
    limits.authority.segments = 2;
    let mut rules = core.authority.rules().clone();
    rules.ceiling = authority.clone();
    core.authority = authority::Domain::new(rules, limits.authority).unwrap();
    let mut facts = Queue::uncapped();
    authority::step(
        &mut core.authority,
        authority::Event::Policy {
            project: 1,
            policy: authority::Policy {
                ceiling: authority,
                period_spend: 0,
                escalation_role: None,
                projections: Box::new([]),
                roles: Box::new([]),
                requirements: Box::new([]),
            },
        },
        &mut facts,
    );
    assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    let document = make(&core, &limits, 1, &Subject::Project { project: 1 }, &window(), Wall::EPOCH).unwrap();
    let Document::Project(project) = document else { panic!("project document") };
    let line = act_authority(&limits, &tasks).unwrap();
    assert_eq!(line, project.policy.ceiling);
    assert_eq!(line.grants.len(), 2);
    assert_eq!(line.executors.len(), 3);
    assert_eq!(line.note_resources.len(), 2);
    limits.documents.text_bytes = 4;
    assert_eq!(act_authority(&limits, &tasks), Err(Missing::TooLarge));
    assert_eq!(
        make(&core, &limits, 1, &Subject::Project { project: 1 }, &window(), Wall::EPOCH),
        Err(Missing::TooLarge)
    );
}
