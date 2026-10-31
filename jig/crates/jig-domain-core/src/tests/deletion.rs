//! Core deletion decisions and kept-step recovery (jig/docs/design/domain/people.md, 5.2).
use super::*;

fn core() -> (Core, Limits) {
    let (mut core, limits) = super::core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks };
    let mut out = Queue::uncapped();
    tasks::step(&mut core.tasks, &env, tasks::Event::Check, &mut out);
    tasks::step(&mut core.tasks, &env, tasks::Event::Restored, &mut out);
    (core, limits)
}

fn environment(limits: &Limits) -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }
}

fn begin(core: &mut Core, limits: &Limits, project: u32) -> (Queue<Work>, Queue<Request>) {
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    crate::deletion::begin(core, &environment(limits), &mut work, &mut out, Token::new(90), project).unwrap();
    (work, out)
}

fn outcome(mut work: Queue<Work>) -> people::Outcome {
    let Some(Work::Event(Event::People(people::Event::Decided { request, outcome }))) = work.pop() else {
        panic!("one deletion answer")
    };
    assert_eq!(request, Token::new(90));
    assert!(work.is_empty());
    outcome
}

fn collect(mut out: Queue<Request>) -> Box<[Request]> {
    let mut requests = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        requests.push(request).unwrap();
    }
    requests.into_boxed()
}

fn removed_policy(request: &Request) -> bool {
    match request {
        Request::Write(Write::Erase(Key::People(people::Key::Policy(1)))) => true,
        Request::Write(_) | Request::Ask { .. } | Request::Held(_) | Request::Now(_) | Request::ProjectMade { .. } => {
            false
        }
    }
}

fn contains_policy(requests: &[Request]) -> bool {
    for request in requests {
        if removed_policy(request) {
            return true;
        }
    }
    false
}

fn closing(core: &Core, task: u64) -> bool {
    match core.tasks.task(task).unwrap().phase {
        tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => true,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => false,
    }
}

fn only_decided(out: Queue<Request>) {
    assert!(out.is_empty());
}

fn released_connectors(requests: &[Request]) -> Box<[u16]> {
    let mut connectors = List::uncapped();
    for request in requests {
        if let Request::Ask { connector, ask: Ask::ReleaseProject { project } } = request {
            assert_eq!(*project, 1);
            connectors.push(*connector).unwrap();
        }
    }
    connectors.into_boxed()
}

#[test]
fn removing_a_project_closes_it_and_answers_in_one_decision() {
    let (mut core, limits) = make_tree();
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    routing::deployment_acts::route(
        &mut core,
        &environment(&limits),
        &mut work,
        &mut out,
        Token::new(90),
        people::Act::RemoveProject { project: 1 },
    );
    assert_eq!(outcome(work), people::Outcome::ProjectRemoved { project: 1 });
    assert!(!core.open_project(1));
    assert!(!core.projects.as_slice().contains(&1));
    assert!(!core.names.contains_key(&1));
    assert!(core.authority.policy(1).is_some());
    for number in 1..=4 {
        assert!(closing(&core, number));
    }
}

#[test]
fn an_owners_ask_removes_it_and_a_members_is_refused() {
    for role in [people::Role::Owner, people::Role::Member] {
        let (mut core, limits) = core();
        let mut child = Queue::uncapped();
        child.push(people::Request::Route {
            request: Token::new(90),
            person: 1,
            project: 1,
            role: Some(role),
            ask: Box::new(people::Ask::RemoveProject { project: 1 }),
            models: Box::new([]),
        });
        let mut work = Queue::uncapped();
        let _requests = routing::people_routes::tag_people(&mut core, &environment(&limits), child, &mut work);
        if role == people::Role::Owner {
            assert_eq!(outcome(work), people::Outcome::ProjectRemoved { project: 1 });
            assert!(!core.open_project(1));
        } else {
            assert_eq!(outcome(work), people::Outcome::Refused(people::Refusal::Role { held: Some(role) }));
            assert!(core.open_project(1));
        }
    }
}

#[test]
fn removing_an_unknown_or_deleting_project_is_refused_unknown() {
    let (mut core, limits) = core();
    let _started = begin(&mut core, &limits, 1);
    for project in [1, 99] {
        let mut work = Queue::uncapped();
        let mut out = Queue::uncapped();
        routing::deployment_acts::route(
            &mut core,
            &environment(&limits),
            &mut work,
            &mut out,
            Token::new(90),
            people::Act::RemoveProject { project },
        );
        assert_eq!(outcome(work), people::Outcome::Refused(people::Refusal::Unknown));
        assert!(out.is_empty());
    }
}

#[test]
fn live_roots_past_the_operation_bound_are_cancelled_next() {
    let (mut core, mut limits) = tree_core();
    limits.people.operations = 1;
    let first = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    let second = new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([first, second]));
    let _started = begin(&mut core, &limits, 1);
    assert!(closing(&core, 1));
    assert!(!closing(&core, 2));
    let _resumed = resume_fleet(&mut core, &environment(&limits));
    assert!(closing(&core, 2));
}

#[test]
fn the_steps_run_in_order_and_the_row_goes_last() {
    let (mut core, limits) = core();
    let _started = begin(&mut core, &limits, 1);
    assert!(core.authority.policy(1).is_some());
    let mut out = Queue::uncapped();
    crate::deletion::tasks_erased(&mut core, &environment(&limits), &mut out, 1);
    let requests = settle_notes(&mut core, &limits, out);
    assert!(contains_policy(&requests));
    assert!(core.authority.policy(1).is_none());
    assert!(!core.permission_roles.contains_key(&1));
    assert!(!core.template_overrides.contains_key(&1));
    assert_eq!(released_connectors(&requests).as_ref(), &[1, 2]);
    // Completion is idempotent even when a connector repeats its acknowledgement.
    let _first = crate::deletion::released(&mut core, &environment(&limits), 1, 1);
    assert!(core.deletions.contains(1));
    let _duplicate = crate::deletion::released(&mut core, &environment(&limits), 1, 1);
    let _last = crate::deletion::released(&mut core, &environment(&limits), 2, 1);
    assert!(!core.deletions.contains(1));
    assert!(!core.open_project(1));
}

#[test]
fn every_connector_is_asked_and_the_row_waits_for_all() {
    let (mut core, limits) = core();
    let _started = begin(&mut core, &limits, 1);
    let mut out = Queue::uncapped();
    crate::deletion::tasks_erased(&mut core, &environment(&limits), &mut out, 1);
    assert_eq!(released_connectors(&settle_notes(&mut core, &limits, out)).as_ref(), &[1, 2]);
    let first = crate::deletion::released(&mut core, &environment(&limits), 2, 1);
    assert!(first.is_empty());
    let unknown = crate::deletion::released(&mut core, &environment(&limits), 99, 1);
    assert!(unknown.is_empty());
    let _last = crate::deletion::released(&mut core, &environment(&limits), 1, 1);
    let repeated = crate::deletion::released(&mut core, &environment(&limits), 1, 1);
    assert!(repeated.is_empty());
}

#[test]
fn close_and_adopt_on_a_closed_project_are_refused_unknown() {
    let (mut core, limits) = make_tree();
    let _started = begin(&mut core, &limits, 1);
    for act in [
        people::Act::Close { project: 1, task: 1, grace: None, reason: Box::new([]) },
        people::Act::Adopt {
            project: 1,
            adoption: people::Adoption {
                resource: people::ResourceName { connector: 1, path: Box::new([]) },
                role: people::ResourceRole::Context,
                options: Box::new([]),
            },
        },
    ] {
        let mut work = Queue::uncapped();
        let mut out = Queue::uncapped();
        routing::deployment_acts::route(&mut core, &environment(&limits), &mut work, &mut out, Token::new(90), act);
        assert_eq!(outcome(work), people::Outcome::Refused(people::Refusal::Unknown));
        assert!(out.is_empty());
    }
}

#[test]
fn a_period_or_recurring_start_for_a_closed_project_decides_nothing() {
    let (mut core, limits) = core();
    let authority = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None).authority;
    let _started = begin(&mut core, &limits, 1);
    let out = step(&mut core, &environment(&limits), Event::Period { project: 1, period: 2, budget: 1, next: None });
    only_decided(out);
    let out = step(
        &mut core,
        &environment(&limits),
        Event::StartRecurring {
            project: 1,
            authority,
            template: tasks::RecurringTemplate { key: 1, batch: Box::new([]), overlap: tasks::RecurringOverlap::Skip },
        },
    );
    only_decided(out);
    assert!(core.tasks.recurring_tasks(1).is_empty());
}

fn restored_projects() -> (Core, Limits) {
    let mut limits = limits();
    limits.authority.projects = 2;
    let mut config = config(41);
    let value = policy::snapshot(&config.seed_projects[0].policy.policy, &[], &[]).unwrap();
    config.seed_projects = Box::new([]);
    let mut core = Core::new(config, &limits).unwrap();
    core.counters.deployment.projects = 2;
    let _start = core.start_begin(false, Wall::EPOCH);
    core.restore_core(CoreRecord::Deployment(core.counters.deployment()), &limits);
    assert!(core.restore_policy(&limits, 1, Box::from("Tally"), value.clone()));
    assert!(core.restore_policy(&limits, 2, Box::from("tally"), value));
    (core, limits)
}

#[test]
fn a_freed_name_restores_beside_its_new_holder() {
    let (mut core, limits) = restored_projects();
    // People projects the committed deletion row after the policy rows load.
    crate::deletion::restore_project(&mut core, 1, people::DeletionStep::Tasks);
    let mut out = Queue::uncapped();
    let env = environment(&limits);
    let crate::start::Action::Step(crate::start::Phase::LoadInbox) =
        core.start_done(&env, crate::start::Phase::LoadCore, &mut out)
    else {
        panic!("loaded policies and deletions precede inbox")
    };
    assert_eq!(core.projects.as_slice(), &[2]);
    assert!(!core.names.contains_key(&1));
    assert_eq!(core.names.get(&2).unwrap().as_ref(), "tally");
    assert!(!core.open_project(1));
    assert!(core.open_project(2));
}

#[test]
fn a_kept_deletion_resumes_at_its_tasks_step_after_open() {
    let (mut core, limits) = make_tree();
    crate::deletion::restore_project(&mut core, 1, people::DeletionStep::Tasks);
    core.start.phase = Some(crate::start::Phase::Open);
    let _closed = resume_fleet(&mut core, &environment(&limits));
    assert!(!closing(&core, 1));
    let mut out = Queue::uncapped();
    let crate::start::Action::Idle = core.start_done(&environment(&limits), crate::start::Phase::Open, &mut out) else {
        panic!("open completes the start")
    };
    let _opened = resume_fleet(&mut core, &environment(&limits));
    assert!(closing(&core, 1));
    assert!(core.authority.policy(1).is_some());
}

#[test]
fn a_kept_deletion_resumes_at_its_rows_step_after_open() {
    let (mut core, limits) = core();
    crate::deletion::restore_project(&mut core, 1, people::DeletionStep::Rows);
    core.start.phase = Some(crate::start::Phase::Open);
    let closed = resume_fleet(&mut core, &environment(&limits));
    let closed = collect(closed);
    assert!(!contains_policy(&closed));
    assert!(released_connectors(&closed).is_empty());
    assert!(core.authority.policy(1).is_some());
    let mut out = Queue::uncapped();
    let crate::start::Action::Idle = core.start_done(&environment(&limits), crate::start::Phase::Open, &mut out) else {
        panic!("open completes the start")
    };
    let opened = resume_fleet(&mut core, &environment(&limits));
    assert!(contains_policy(&collect(opened)));
    assert!(core.authority.policy(1).is_none());
}

#[test]
fn a_kept_deletion_resumes_at_its_release_step_after_open() {
    let (mut core, limits) = core();
    crate::deletion::restore_project(&mut core, 1, people::DeletionStep::Release);
    core.start.phase = Some(crate::start::Phase::Open);
    let closed = resume_fleet(&mut core, &environment(&limits));
    let closed = collect(closed);
    assert!(!contains_policy(&closed));
    assert!(released_connectors(&closed).is_empty());
    let mut out = Queue::uncapped();
    let crate::start::Action::Idle = core.start_done(&environment(&limits), crate::start::Phase::Open, &mut out) else {
        panic!("open completes the start")
    };
    let opened = resume_fleet(&mut core, &environment(&limits));
    assert_eq!(released_connectors(&collect(opened)).as_ref(), &[1, 2]);
    let again = resume_fleet(&mut core, &environment(&limits));
    assert!(released_connectors(&collect(again)).is_empty());
}

#[test]
fn the_load_and_link_start_checks_names_after_hiding_deletions() {
    for deleting in [false, true] {
        let (mut core, limits) = restored_projects();
        core.start.phase = Some(crate::start::Phase::LoadCore);
        if deleting {
            crate::deletion::restore_project(&mut core, 1, people::DeletionStep::Tasks);
        }
        let mut out = Queue::uncapped();
        let action = core.start_done(&environment(&limits), crate::start::Phase::LoadCore, &mut out);
        if deleting {
            assert_eq!(core.start_phase(), Some(crate::start::Phase::LoadInbox));
            assert!(core.start_failure().is_none());
        } else {
            let crate::start::Action::Refused(crate::start::Failure::ProjectPolicy { project: 1 }) = action else {
                panic!("two open projects cannot share a name")
            };
        }
    }
}

#[test]
fn a_load_and_link_deletion_waits_until_open_completes() {
    let (mut core, limits) = core();
    crate::deletion::restore_project(&mut core, 1, people::DeletionStep::Release);
    core.start.phase = Some(crate::start::Phase::Open);
    let closed = resume_fleet(&mut core, &environment(&limits));
    assert!(released_connectors(&collect(closed)).is_empty());
    let mut out = Queue::uncapped();
    let _opened = core.start_done(&environment(&limits), crate::start::Phase::Open, &mut out);
    let opened = resume_fleet(&mut core, &environment(&limits));
    assert_eq!(released_connectors(&collect(opened)).as_ref(), &[1, 2]);
}

fn settle_notes(core: &mut Core, limits: &Limits, mut pending: Queue<Request>) -> Box<[Request]> {
    let mut requests = List::uncapped();
    while let Some(request) = pending.pop() {
        if let Request::Held(held) = &request
            && let Held::NotesLoad { owner, .. } = &**held
        {
            let mut follow = step(
                core,
                &environment(limits),
                Event::Notes(notes::Event::Loaded {
                    owner: *owner,
                    rows: notes::Rows { records: List::with_capacity(limits.notes.load_rows) },
                    more: false,
                }),
            );
            while let Some(request) = follow.pop() {
                pending.push(request);
            }
        }
        requests.push(request).unwrap();
    }
    requests.into_boxed()
}

#[test]
fn a_deletion_without_connectors_finishes_after_its_notes() {
    let (mut core, limits) = core();
    core.connectors = Box::new([]);
    let _started = begin(&mut core, &limits, 1);
    let mut out = Queue::uncapped();
    crate::deletion::tasks_erased(&mut core, &environment(&limits), &mut out, 1);
    let requests = settle_notes(&mut core, &limits, out);
    assert!(released_connectors(&requests).is_empty());
    assert!(!core.deletions.contains(1));
}

#[test]
fn a_kept_deletion_advances_after_fleet_inputs_and_timers() {
    for timer in [false, true] {
        let (mut core, limits) = core();
        let _started = begin(&mut core, &limits, 1);
        let metered = step(&mut core, &environment(&limits), Event::Fill { bytes: 1 });
        only_decided(metered);
        let requests = if timer {
            fire(&mut core, &environment(&limits), Timer::Tasks)
        } else {
            step(
                &mut core,
                &environment(&limits),
                Event::Fleet(fleet::Event::Acknowledge { run: Token::new(99), attempt: Token::new(1) }),
            )
        };
        let mut loads = 0_u32;
        for request in collect(requests) {
            if let Request::Held(held) = request
                && let Held::ReadLoad { range, .. } = *held
                && range.scope == reads::Scope::EndedTasks
            {
                loads += 1;
            }
        }
        assert_eq!(loads, 1, "a kept deletion resumes its ended-task walk, timer={timer}");
    }
}
