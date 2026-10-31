//! Link obligations and stored diagnostics (jig/docs/design/domain/engine.md, 6).
#![expect(clippy::disallowed_macros, reason = "stored fixtures exercise byte limits")]
use crate::{
    CoreRecord, Event, Restored,
    tests::start::{
        Action, Core, Env, Failure, Key, Phase, Queue, Request, Time, Wall, config, people, step_phase, tasks,
    },
};
use core::panic::AssertUnwindSafe;
use skein_lib::Token;
use std::panic::catch_unwind;

fn loaded() -> (Core, Env<crate::Limits>) {
    let (_, limits) = crate::tests::tree_core();
    let configured = config(41);
    let policy =
        crate::admission::policy::snapshot(&configured.seed_projects[0].policy.policy, &[], &[]).expect("policy");
    let mut core = Core::new(configured, &limits).expect("configured core");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    let mut header = core.counters.deployment();
    header.commits = 1;
    header.tasks = 100;
    header.people = 1;
    header.projects = 1;
    assert_eq!(core.restore_core(CoreRecord::Deployment(header), &limits), Restored::Deployment { commits: 1 });
    assert!(core.restore_policy(&limits, 1, Box::from("Project 1"), policy));
    restore_person(
        &mut core,
        &env,
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
    );
    (core, env)
}

fn restore_person(core: &mut Core, env: &Env<crate::Limits>, record: people::Stored) {
    let mut out = crate::step(core, env, Event::People(people::Event::Restore { record }));
    while let Some(request) = out.pop() {
        if let Request::Now(now) = request
            && let crate::Now::RestoreRefused = *now
        {
            panic!("the start emits no RestoreRefused");
        }
    }
}

fn restore_task(core: &mut Core, env: &Env<crate::Limits>, record: tasks::Stored) {
    assert!(core.restore_task_row(&record));
    let mut out = crate::step(core, env, Event::Tasks(tasks::Event::Restore { record }));
    while let Some(request) = out.pop() {
        if let Request::Now(now) = request
            && let crate::Now::RestoreRefused = *now
        {
            panic!("the start emits no RestoreRefused");
        }
    }
}

fn link(core: &mut Core, env: &Env<crate::Limits>) -> Action {
    let mut out = Queue::uncapped();
    step_phase(core.start_done(env, Phase::LoadCore, &mut out), Phase::LoadInbox);
    let mut previous = Phase::LoadInbox;
    for connector in core.connectors.clone() {
        let next = Phase::RestoreConnector { connector };
        step_phase(core.start_done(env, previous, &mut out), next);
        previous = next;
    }
    step_phase(core.start_done(env, previous, &mut out), Phase::Link);
    core.start_done(env, Phase::Link, &mut out)
}

fn task() -> tasks::TaskRecord {
    let (mut source, limits) = crate::tests::tree_core();
    crate::tests::make_task(&mut source, &limits, 1);
    source.tasks.task(1).expect("source task").clone()
}

#[test]
fn link_preserves_the_committed_waiting_holder_and_its_projection() {
    let (mut core, env) = loaded();
    restore_person(
        &mut core,
        &env,
        people::Stored::Person {
            number: 2,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([2]) },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
    );
    restore_person(
        &mut core,
        &env,
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 2, role: people::Role::Owner }]),
        },
    );
    restore_task(&mut core, &env, period());
    let mut row = task();
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Person(1));
    row.phase =
        tasks::Phase::Held { was: tasks::Was::Active(tasks::Active::Due), why: tasks::Hold::Budget, source: None };
    row.escalation = tasks::Escalation::Waiting {
        revision: 1,
        holder: tasks::EscalationHolder::Person(1),
        entry: 1,
        since: Wall::EPOCH,
    };
    restore_task(&mut core, &env, tasks::Stored::Live(Box::new(row.clone())));
    let mut out = Queue::uncapped();
    step_phase(core.start_done(&env, Phase::LoadCore, &mut out), Phase::LoadInbox);
    let _census = core.census(&env.limits);
    let mut previous = Phase::LoadInbox;
    for connector in core.connectors.clone() {
        let next = Phase::RestoreConnector { connector };
        step_phase(core.start_done(&env, previous, &mut out), next);
        previous = next;
    }
    step_phase(core.start_done(&env, previous, &mut out), Phase::Link);
    linked(core.start_done(&env, Phase::Link, &mut out));
    assert_eq!(core.people.needs(1), Some(0));
    assert_eq!(core.people.needs(2), Some(0));
    assert_eq!(core.tasks.task(row.number), Some(&row));
    assert_eq!(core.start_failure(), None);
}

fn answer(outcome: people::Outcome) -> people::Stored {
    people::Stored::Answer {
        key: people::RequestKey { person: 1, key: [1; 16] },
        ask: Box::new(people::Ask::Application {
            scope: people::ApplicationScope::Deployment,
            connector: 1,
            digest: [0; 32],
        }),
        models: Box::new([]),
        outcome,
        at: Wall::EPOCH,
    }
}

#[test]
fn applied_tokens_are_not_judged_and_missing_connectors_are_links() {
    for (connector, token) in [(1, 0), (1, u64::MAX), (99, 0), (99, u64::MAX)] {
        let (mut core, env) = loaded();
        let row = answer(people::Outcome::Applied { connector, token: Token::new(token) });

        restore_person(&mut core, &env, row);
        let result = catch_unwind(AssertUnwindSafe(|| link(&mut core, &env)));
        if connector == 1 {
            linked(result.expect("configured connector links"));
        } else {
            let _panic = result.expect_err("missing link asserts");
            assert_eq!(core.start_phase(), Some(Phase::Link));
        }
        assert_eq!(core.start_failure(), None);
    }
}

#[test]
fn tasks_fit_refusal_keeps_the_key_without_failing_the_other_script() {
    for accounting in [false, true] {
        let (mut core, env) = loaded();
        let mut row = task();
        if accounting {
            row.numbers.spent = u64::MAX;
            row.numbers.spent_below = 1;
        } else {
            row.spec.words =
                vec![b'x'; usize::try_from(env.limits.tasks.spec_bytes + 1).expect("small spec")].into_boxed_slice();
        }
        let key = Key::Tasks(tasks::Key::Live(row.number));
        restore_task(&mut core, &env, tasks::Stored::Live(Box::new(row)));
        let expected = if accounting {
            Failure::Accounting { key }
        } else {
            Failure::LiveSet {
                set: crate::start::LiveSet::TasksBound(tasks::RestoreBound::SpecBytes),
                key,
                held: u64::from(env.limits.tasks.spec_bytes) + 1,
                limit: u64::from(env.limits.tasks.spec_bytes),
            }
        };
        assert_eq!(core.start_failure(), Some(expected));
    }
}

#[test]
fn people_fit_refusal_keeps_the_key_without_failing_the_other_script() {
    let (mut core, env) = loaded();
    let held = env.limits.people.identity_bytes + 1;
    restore_person(
        &mut core,
        &env,
        people::Stored::Person {
            number: 2,
            identity: people::Identity {
                key: people::IdentityKey {
                    provider: 0,
                    subject: vec![b'x'; usize::try_from(held).expect("small identity")].into_boxed_slice(),
                },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
    );
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet {
            set: crate::start::LiveSet::PeopleBound(people::RestoreBound::IdentityBytes),
            key: Key::People(people::Key::Person(2)),
            held: u64::from(held),
            limit: u64::from(env.limits.people.identity_bytes),
        })
    );
}

#[test]
fn stored_llm_choice_names_the_task_that_configuration_cannot_hold() {
    let (mut core, env) = loaded();
    let mut row = task();
    row.llm = Some(tasks::LlmChoice {
        account: None,
        main: tasks::ModelEffort { model: u8::MAX, effort: None },
        subagents: Box::new([]),
    });
    restore_task(&mut core, &env, period());
    restore_task(&mut core, &env, tasks::Stored::Live(Box::new(row)));
    assert_eq!(core.start_failure(), None);
    let Action::Refused(failure) = link(&mut core, &env) else { panic!("configured choice refuses") };
    assert_eq!(failure, Failure::LlmChoice { key: Key::Tasks(tasks::Key::Live(1)) });
}

#[test]
fn unknown_introduction_diagnostic_is_retained_and_replayed() {
    let (mut core, env) = loaded();
    let key = super::call_key();
    let part = crate::CallPart::MessageRefused(tasks::Problem::new(Some(u64::MAX), tasks::Refusal::Unknown));
    let record = crate::CallRecord {
        key,
        part: part.clone(),
        settled: Some(crate::SettledCall {
            serial: 1,
            tool: Box::from(b"introduce".as_slice()),
            answer: crate::SettledAnswer::Host { error: true, body: Box::from(b"unknown target".as_slice()) },
        }),
    };
    assert_eq!(core.restore_core(CoreRecord::Call(record.clone()), &env.limits), Restored::Live);
    let Some(crate::CallReplay::Core(found)) = core.replay_call(key) else { panic!("retained diagnostic replays") };
    assert_eq!(found, part);
    assert_eq!(core.call_record(key), Some(record));
    assert_eq!(core.start_failure(), None);
}

#[test]
fn missing_ended_read_owner_is_a_link_before_sweep() {
    let (mut core, env) = loaded();
    let mut out = Queue::uncapped();
    step_phase(core.start_done(&env, Phase::LoadCore, &mut out), Phase::LoadInbox);
    let _census = core.census(&env.limits);
    let row = crate::Record::People(people::Stored::EndedRead { task: 7, party: 1, through: 0 });
    assert!(core.restore_inbox_row(&env.limits, &row));
    let mut previous = Phase::LoadInbox;
    for connector in core.connectors.clone() {
        let next = Phase::RestoreConnector { connector };
        step_phase(core.start_done(&env, previous, &mut out), next);
        previous = next;
    }
    step_phase(core.start_done(&env, previous, &mut out), Phase::Link);
    let _panic = catch_unwind(AssertUnwindSafe(|| core.start_done(&env, Phase::Link, &mut out)))
        .expect_err("missing ended owner asserts");
    assert_eq!(core.start_phase(), Some(Phase::Link));
    assert!(out.is_empty());
}

fn period() -> tasks::Stored {
    tasks::Stored::Ledger(tasks::FundingRecord {
        period: None,
        funder: tasks::Funder::Period { project: 1, period: 1 },
        parent: None,
        numbers: tasks::Numbers { budget: 1000, spent: 0, spent_below: 0, reserved: 100 },
        made: 0,
        closed: false,
    })
}

#[test]
fn recurring_lifetime_allotments_are_not_simultaneous_live_capacity() {
    let (mut core, env) = loaded();
    restore_task(&mut core, &env, period());
    let mut row = task();
    row.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    row.recurring = Some(Box::new(tasks::RecurringState {
        template: tasks::RecurringTemplate { key: 1, batch: Box::new([]), overlap: tasks::RecurringOverlap::Skip },
        last_batch: Box::new([]),
        last_period: 1,
        pending_period: None,
    }));
    restore_task(&mut core, &env, tasks::Stored::Live(Box::new(row)));
    let funder = tasks::Funder::Recurring { project: 1, task: 1, period: 1 };
    let made = env.limits.tasks.tasks + 1;
    restore_task(
        &mut core,
        &env,
        tasks::Stored::Ledger(tasks::FundingRecord {
            period: None,
            funder,
            parent: Some(tasks::Funder::Period { project: 1, period: 1 }),
            numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            made,
            closed: false,
        }),
    );
    linked(link(&mut core, &env));
    assert_eq!(core.tasks.funding(funder).expect("retained recurring ledger").made, made);
    assert_eq!(core.start_failure(), None);
}

fn member() -> tasks::New {
    let row = task();
    tasks::New {
        deadline: tasks::Deadline::Named,
        label: row.label,
        flags: row.flags,
        number: 2,
        project: row.project,
        executor: row.executor,
        llm: row.llm,
        spec: row.spec,
        contract: row.contract,
        authority: row.authority,
        numbers: row.numbers,
        funder: row.funder,
        dependencies: row.dependencies,
        holdings: row.holdings,
        wake: row.wake,
        recurring: None,
        tracked: None,
    }
}

fn amendment() -> Box<tasks::Amendment> {
    Box::new(tasks::Amendment {
        tracked: None,
        addressed: Some(tasks::AddressChange { revision: 0, address: tasks::PersonAddress::Person(99) }),
        instructions: None,
        release: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: Box::new([]),
    })
}

fn unmade(core: &mut Core, env: &Env<crate::Limits>) {
    assert_eq!(core.start_failure(), None);
    let _panic = catch_unwind(AssertUnwindSafe(|| link(core, env))).expect_err("missing link asserts");
    assert_eq!(core.start_phase(), Some(Phase::Link));
    assert_eq!(core.start_failure(), None);
}

#[test]
fn every_placed_hold_links_its_project_and_person_owner() {
    for (project, person, present) in [(1, 1, true), (99, 1, false), (1, 99, false)] {
        let (mut core, env) = loaded();
        let hold = tasks::PlacedHold {
            number: 7,
            project,
            owner: tasks::Owner::Party { person },
            mode: tasks::Mode::Drain,
            scope: tasks::Scope::Project,
            reason: Box::new([]),
            placed_at: 0,
            revision: 0,
            exempt: Box::new([]),
            lifted: false,
        };
        restore_task(&mut core, &env, tasks::Stored::PlacedHold(Box::new(hold)));
        if present {
            linked(link(&mut core, &env));
        } else {
            unmade(&mut core, &env);
        }
    }
}

#[test]
fn a_tasks_placed_number_links_the_retained_hold() {
    let (mut core, env) = loaded();
    restore_task(&mut core, &env, period());
    let mut row = task();
    row.placed = Some(7);
    restore_task(&mut core, &env, tasks::Stored::Live(Box::new(row)));
    unmade(&mut core, &env);
}

#[test]
fn task_proposal_actions_and_recurring_templates_link_their_people() {
    let mut executor = member();
    executor.executor = tasks::Executor::Person(tasks::PersonAddress::Person(99));
    let mut funder = member();
    funder.funder = tasks::Funder::Pool { project: 1, person: 99, period: 1 };
    for action in [
        tasks::ProposalAction::Batch(Box::new([executor.clone()])),
        tasks::ProposalAction::Batch(Box::new([funder.clone()])),
        tasks::ProposalAction::Amend { task: 1, revision: 0, amendment: *amendment() },
    ] {
        let (mut core, env) = loaded();
        restore_task(&mut core, &env, period());
        let mut row = task();
        row.proposal = Some(Box::new(tasks::Proposal {
            number: 1,
            proposer: 1,
            project: 1,
            action,
            reason: Box::new([]),
            as_holder: false,
            state: tasks::ProposalState::Withdrawn,
        }));
        restore_task(&mut core, &env, tasks::Stored::Live(Box::new(row)));
        unmade(&mut core, &env);
    }
    for member in [executor, funder] {
        let (mut core, env) = loaded();
        restore_task(&mut core, &env, period());
        let mut row = task();
        row.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
        row.recurring = Some(Box::new(tasks::RecurringState {
            template: tasks::RecurringTemplate {
                key: 1,
                batch: Box::new([member]),
                overlap: tasks::RecurringOverlap::Skip,
            },
            last_batch: Box::new([]),
            last_period: 1,
            pending_period: None,
        }));
        restore_task(&mut core, &env, tasks::Stored::Live(Box::new(row)));
        unmade(&mut core, &env);
    }
}

#[test]
fn person_proposals_link_their_parties_and_actions() {
    for (proposer, project) in [(99, 1), (1, 99)] {
        let (mut core, env) = loaded();
        let row = tasks::PersonProposal {
            number: 7,
            proposer,
            project,
            action: tasks::PersonAction::Batch(Box::new([])),
            reason: Box::new([]),
            state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
        };
        restore_task(&mut core, &env, tasks::Stored::PersonProposal(Box::new(row)));
        unmade(&mut core, &env);
    }
    let mut missing_executor = member();
    missing_executor.executor = tasks::Executor::Person(tasks::PersonAddress::Person(99));
    let mut missing_funder = member();
    missing_funder.funder = tasks::Funder::Pool { project: 1, person: 99, period: 1 };
    for action in [
        tasks::PersonAction::Batch(Box::new([missing_executor])),
        tasks::PersonAction::Batch(Box::new([missing_funder])),
    ] {
        let (mut core, env) = loaded();
        restore_task(
            &mut core,
            &env,
            tasks::Stored::PersonProposal(Box::new(tasks::PersonProposal {
                number: 7,
                proposer: 1,
                project: 1,
                action,
                reason: Box::new([]),
                state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
            })),
        );
        unmade(&mut core, &env);
    }
}

#[test]
fn settings_proposals_link_both_records_and_the_retained_revision() {
    for (settings, person, against, present) in
        [(false, true, 1, false), (true, false, 1, false), (true, true, 99, false), (true, true, 1, true)]
    {
        let (mut core, env) = loaded();
        if settings {
            assert_eq!(
                core.restore_core(
                    CoreRecord::Settings(crate::settings::Record::Proposal {
                        proposer: people::Proposer::Party(1),
                        proposal: 7,
                        value: crate::settings::Proposal { against, items: Box::new([]) },
                    }),
                    &env.limits
                ),
                Restored::Live
            );
        }
        if person {
            restore_task(
                &mut core,
                &env,
                tasks::Stored::PersonProposal(Box::new(tasks::PersonProposal {
                    number: 7,
                    proposer: 1,
                    project: people::DEPLOYMENT,
                    action: tasks::PersonAction::Settings { against, items: Box::new([]) },
                    reason: Box::new([]),
                    state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
                })),
            );
        }
        if present {
            linked(link(&mut core, &env));
        } else {
            unmade(&mut core, &env);
        }
    }
}

#[test]
fn projections_link_live_plan_tasks_and_keep_ended_tasks_as_history() {
    for live in [false, true] {
        let (mut core, env) = loaded();
        let ended = tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) });
        for row in [
            crate::ProjectionRecord::Goal {
                goal: crate::ProjectionGoal {
                    number: 1,
                    project: 1,
                    priority: 1,
                    words: Box::new([]),
                    phase: ended.clone(),
                },
                pending: Box::new([]),
                changed: false,
                fed_at: None,
            },
            crate::ProjectionRecord::Task {
                goal: 1,
                task: crate::ProjectionTask {
                    number: 2,
                    requester: tasks::Party::Task(1),
                    words: Box::new([]),
                    phase: if live { tasks::Phase::Waiting } else { ended },
                },
            },
        ] {
            assert_eq!(core.restore_core(CoreRecord::Projection(row), &env.limits), Restored::Live);
        }
        if live {
            unmade(&mut core, &env);
        } else {
            linked(link(&mut core, &env));
        }
    }
}

#[test]
fn connector_resource_roles_link_their_projects() {
    for (project, deletion, present) in [
        (1, None, true),
        (99, None, false),
        (99, Some(people::DeletionStep::Rows), true),
        (99, Some(people::DeletionStep::Release), true),
    ] {
        let (mut core, env) = loaded();
        if let Some(step) = deletion {
            restore_person(&mut core, &env, people::Stored::Deletion { project, step });
            assert!(core.authority.policy(project).is_none());
        }
        let mut out = Queue::uncapped();
        step_phase(core.start_done(&env, Phase::LoadCore, &mut out), Phase::LoadInbox);
        let connector = core.connectors[0];
        step_phase(core.start_done(&env, Phase::LoadInbox, &mut out), Phase::RestoreConnector { connector });
        let name = tasks::Name { connector, path: Box::new([Box::from(b"resource".as_slice())]) };
        assert!(core.restore_resource_role(&env, project, name, crate::effects::connector::ResourceRole::Context));
        let mut previous = Phase::RestoreConnector { connector };
        for connector in core.connectors.clone().into_iter().skip(1) {
            let next = Phase::RestoreConnector { connector };
            step_phase(core.start_done(&env, previous, &mut out), next);
            previous = next;
        }
        step_phase(core.start_done(&env, previous, &mut out), Phase::Link);
        let result = catch_unwind(AssertUnwindSafe(|| core.start_done(&env, Phase::Link, &mut out)));
        if present {
            linked(result.expect("project links"));
            if deletion.is_some() {
                assert!(!core.open_project(project), "kept deletion remains closed");
            }
        } else {
            let _panic = result.expect_err("missing link asserts");
            assert_eq!(core.start_phase(), Some(Phase::Link));
        }
    }
}

#[test]
fn answer_tasks_link_allocated_identities_after_their_live_rows_end() {
    for (task, present) in [(99, true), (101, false)] {
        let (mut core, env) = loaded();
        restore_person(&mut core, &env, answer(people::Outcome::Stopped { task }));
        assert!(core.tasks.task(task).is_none());
        if present {
            linked(link(&mut core, &env));
        } else {
            unmade(&mut core, &env);
        }
    }
    let (mut core, env) = loaded();
    restore_person(&mut core, &env, answer(people::Outcome::Created { first: 99, last: 101, given: None }));
    unmade(&mut core, &env);
}

fn linked(action: Action) {
    match action {
        Action::Step(phase) => assert_eq!(phase, Phase::Sweep),
        Action::Commit(_) => {}
        Action::Idle | Action::Refused(_) => panic!("linked start reaches Sweep"),
    }
}
