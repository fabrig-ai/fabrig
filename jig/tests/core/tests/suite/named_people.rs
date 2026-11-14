//! Outside person names must link at the next start (jig/docs/design/domain/engine/README.md, 5.1 and jig/docs/design/domain/engine/restart.md).
use jig_core_world::fixtures::{ask, live_row};
use jig_core_world::world::{Setup, World};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{List, ReplyTo, Token};

const SEED: u64 = 29_501;
const PROJECT: u32 = 1;
const PERSON: u64 = 1;
const PERSON_TASK: u64 = 2;
const DELEGATE: u64 = 3;
const PROCEDURE_DELEGATE: u64 = 4;
// A root, the person task, the procedure, and its child.
const TASK_SLOTS: u32 = 4;
// Charter, procedure, and the owner's person role.
const EXECUTOR_KINDS: u32 = 3;
const ROLE: u32 = 0;

fn configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_test_profile::effect_fixture(SEED, false);
    // The fixture offers a charter, a procedure and the owner's person role.
    // A root, the live person task used as an absent person, a procedure and its delegate.
    limits.core.tasks.tasks = TASK_SLOTS;
    limits.core.tasks.project_tasks = TASK_SLOTS;
    limits.core.tasks.tree_tasks = 3;
    limits.core.tasks.depth = 2;
    limits.core.tasks.delegates = 2;
    limits.core.fleet.slots = 2;
    limits.core.tasks.executor_kinds = EXECUTOR_KINDS;
    limits.core.authority.executors = EXECUTOR_KINDS;
    limits.core.documents.executors = EXECUTOR_KINDS;
    let kinds: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(1), authority::Executor::Role(ROLE)]);
    config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    config.core.defaults.rules.ceiling.delegation.depth = 3;
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.ceiling.delegation.depth = 3;
    policy.roles[0].authority.delegation.depth = 3;
    policy.roles[0].authority.delegation.tasks = 3;
    policy.roles[0].authority.delegation.kinds = kinds;
    (config, limits)
}

fn world() -> World {
    let (config, limits) = configuration();
    let creating = jig_test_profile::fixtures::Creating::new(&config.core, limits.core.tasks.tree_tasks);
    let mut initial = jig_test_profile::fixtures::create_chat(&creating, PROJECT, 0, b"caller".as_slice().into());
    let people::Ask::Create { batch, .. } = &mut initial else { panic!("initial caller") };
    let caller = batch.first_mut().expect("one caller");
    caller.authority.delegation = people::Delegation {
        kinds: Box::new([people::Executor::Charter(1), people::Executor::Procedure(1), people::Executor::Role(ROLE)]),
        tasks: 2,
        depth: 2,
    };
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        limits,
        initial: Some(initial),
        ..Setup::default()
    });
    let mut member = jig_test_profile::fixtures::people_member(b"check the answer", 0);
    member.executor = people::TaskExecutor::Person(people::PersonAddress::Role(ROLE));
    ask(&mut world, 51, people::Ask::Create { project: PROJECT, batch: Box::new([member]) });
    world
}

fn amendment(person: u64) -> people::Amendment {
    people::Amendment {
        addressed: Some(people::AddressChange { revision: 0, address: people::PersonAddress::Person(person) }),
        tracked: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: b"name the person".as_slice().into(),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    Person(people::Refusal),
    Task(tasks::Problem),
    Step(core::StepRefusal),
    Compound { at: u32, why: people::Refusal },
    Unexpected(String),
}

fn amend(world: &mut World, person: u64) -> Option<Refusal> {
    let revision = live_row(world, PERSON_TASK).revisions.spec;
    ask(
        world,
        52,
        people::Ask::Amend {
            project: PROJECT,
            task: PERSON_TASK,
            revision,
            amendment: amendment(person),
            release: None,
        },
    );
    match world.people_answers.last() {
        Some(people::Reply::Outcome(people::Outcome::Amended { .. })) => None,
        Some(people::Reply::Outcome(people::Outcome::Refused(why))) => Some(Refusal::Person(why.clone())),
        other => Some(Refusal::Unexpected(format!("{other:?}"))),
    }
}

fn agent(world: &mut World, person: u64) -> Option<Refusal> {
    let member = person_member(world, person);
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9052)),
        key: world.call_key(1),
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    match world.answers.last().map(|answer| &answer.2) {
        Some(core::CallPart::Delegated(_)) => None,
        Some(core::CallPart::DelegationRefused(problem)) => Some(Refusal::Task(problem.clone())),
        other => Some(Refusal::Unexpected(format!("{other:?}"))),
    }
}

fn person_reply(world: &World) -> Option<Refusal> {
    match world.people_answers.last() {
        Some(people::Reply::Outcome(people::Outcome::Refused(why))) => Some(Refusal::Person(why.clone())),
        Some(people::Reply::Outcome(people::Outcome::CompoundRefused { at, why })) => {
            Some(Refusal::Compound { at: *at, why: why.clone() })
        }
        Some(people::Reply::Outcome(
            people::Outcome::Created { .. }
            | people::Outcome::Proposed { .. }
            | people::Outcome::RolesSet { .. }
            | people::Outcome::Compound { .. }
            | people::Outcome::Amended { .. },
        )) => None,
        other => Some(Refusal::Unexpected(format!("{other:?}"))),
    }
}

fn party_create(world: &mut World, person: u64) -> Option<Refusal> {
    let mut member = jig_test_profile::fixtures::people_member(b"a named executor", 0);
    member.executor = people::TaskExecutor::Person(people::PersonAddress::Person(person));
    ask(world, 52, people::Ask::Create { project: PROJECT, batch: Box::new([member]) });
    person_reply(world)
}

fn roles(world: &mut World, person: u64) -> Option<Refusal> {
    ask(
        world,
        52,
        people::Ask::SetRoles {
            project: PROJECT,
            holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
        },
    );
    person_reply(world)
}

fn compound_amend(world: &mut World, person: u64) -> Option<Refusal> {
    // A staged first operation must not escape a later unknown-person refusal.
    let mut operations = List::with_capacity(2);
    operations.push(people::Operation::Stop { task: 1 }).expect("first operation");
    operations
        .push(people::Operation::Amend {
            task: PERSON_TASK,
            revision: live_row(world, PERSON_TASK).revisions.spec,
            amendment: Box::new(amendment(person)),
            release: None,
        })
        .expect("second operation");
    ask(world, 52, people::Ask::Compound { project: PROJECT, operations });
    person_reply(world)
}

fn procedure_world() -> World {
    let mut world = world();
    world.hold_procedures = true;
    let mut member = jig_core_world::fixtures::member(
        &world,
        tasks::Executor::Procedure { connector: 1, code: 1 },
        b"ask from a procedure",
    );
    member.authority.budget.spend = 0;
    member.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Role(ROLE)]), tasks: 1, depth: 1 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9054)),
        key: world.call_key(1),
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    world
}

fn person_member(world: &World, person: u64) -> core::Delegate {
    let mut member = jig_core_world::fixtures::member(
        world,
        tasks::Executor::Person(tasks::PersonAddress::Person(person)),
        b"ask the person",
    );
    member.authority.tools = tasks::Tools(0);
    member.authority.models = tasks::Models::NONE;
    member.authority.budget.spend = 0;
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    member
}

fn procedure(world: &mut World, person: u64) -> Option<Refusal> {
    let member = person_member(world, person);
    let env = world.env();
    let answer = world.domain.core_mut().procedure_batch_admit(
        &env.limits.core.tasks,
        DELEGATE,
        std::slice::from_ref(&member),
        env.wall,
    );
    let (step, connector, code, _) = world.procedure_inputs.last().expect("procedure received its context");
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: DELEGATE,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        decision: core::ProcedureStep { batch: Box::new([member]), ..core::ProcedureStep::new(core::StepEnd::Wait) },
    }));
    answer.err().map(Refusal::Step)
}

fn person_delegate_world() -> World {
    let mut world = world();
    let mut member = person_member(&world, PERSON);
    member.executor = tasks::Executor::Person(tasks::PersonAddress::Role(ROLE));
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9054)),
        key: world.call_key(1),
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    world
}

fn task_amendment(world: &World, person: u64, task: u64) -> tasks::Amendment {
    tasks::Amendment {
        addressed: Some(tasks::AddressChange {
            revision: live_row(world, task).revisions.person,
            address: tasks::PersonAddress::Person(person),
        }),
        tracked: None,
        instructions: None,
        release: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: b"name the person".as_slice().into(),
    }
}

fn named_action(world: &mut World, task: u64, action: core::NamedAction) -> Option<Refusal> {
    let attempt = live_row(world, task).attempt;
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9055)),
        key: core::CallKey { task, attempt, completion: 2, position: 0 },
        action,
    }));
    match world.answers.last().map(|answer| &answer.2) {
        Some(core::CallPart::Controlled | core::CallPart::Revised { .. } | core::CallPart::Proposed { .. }) => None,
        Some(core::CallPart::ControlRefused(problem) | core::CallPart::ProposalRefused(problem)) => {
            Some(Refusal::Task(problem.clone()))
        }
        other => Some(Refusal::Unexpected(format!("{other:?}"))),
    }
}

fn agent_amend(world: &mut World, person: u64) -> Option<Refusal> {
    let amendment = task_amendment(world, person, DELEGATE);
    named_action(
        world,
        1,
        core::NamedAction::Amend {
            target: DELEGATE,
            revision: live_row(world, DELEGATE).revisions.spec,
            amendment,
            keep_parameters: false,
        },
    )
}

fn proposed_amend(world: &mut World, person: u64) -> Option<Refusal> {
    let amendment = task_amendment(world, person, DELEGATE);
    named_action(
        world,
        1,
        core::NamedAction::Propose {
            action: tasks::ProposalAction::Amend {
                task: DELEGATE,
                revision: live_row(world, DELEGATE).revisions.spec,
                amendment,
            },
            reason: b"review the address".as_slice().into(),
            as_holder: false,
        },
    )
}

// The application has no caller for raw task batches. These rows drive the
// core event through its root; admission's output is checked in the core's
// named-people unit table because the root has no raw-batch reply fixture.
fn party_proposed_amend(world: &mut World, person: u64) -> Option<Refusal> {
    ask(
        world,
        52,
        people::Ask::Propose {
            project: PROJECT,
            action: people::Action::Amend {
                task: PERSON_TASK,
                revision: live_row(world, PERSON_TASK).revisions.spec,
                amendment: Box::new(amendment(person)),
            },
            reason: b"review the address".as_slice().into(),
        },
    );
    person_reply(world)
}

fn procedure_person_world() -> World {
    let mut world = procedure_world();
    let _answer = procedure(&mut world, PERSON);
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::WakeProcedure { task: DELEGATE })));
    world
}

fn procedure_amend(world: &mut World, person: u64) -> Option<Refusal> {
    let amendment = task_amendment(world, person, PROCEDURE_DELEGATE);
    let limits = world.env().limits.core;
    let answer = world.domain.core_mut().task_amend_admit(
        DELEGATE,
        PROCEDURE_DELEGATE,
        &amendment,
        limits.tasks.depth,
        &limits.tasks,
    );
    let (step, connector, code, _) = world.procedure_inputs.last().expect("next procedure context");
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: DELEGATE,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        decision: core::ProcedureStep {
            amendments: Box::new([core::StepAmendment {
                task: PROCEDURE_DELEGATE,
                revision: live_row(world, PROCEDURE_DELEGATE).revisions.spec,
                amendment,
            }]),
            ..core::ProcedureStep::new(core::StepEnd::Wait)
        },
    }));
    match answer {
        Ok(_) => None,
        Err(core::TaskAmendDenied::Refused { task, why }) => {
            Some(Refusal::Step(core::StepRefusal::Refused(tasks::Problem::new(Some(task), why))))
        }
        Err(core::TaskAmendDenied::Denied(answer)) => Some(Refusal::Unexpected(format!("{answer:?}"))),
    }
}

fn raw_amend(world: &mut World, person: u64) -> Option<Refusal> {
    let message = core::fresh(&mut world.domain.core_mut().counters, core::Family::Message).expect("message number");
    let amendment = task_amendment(world, person, PERSON_TASK);
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Amend {
        reply_to: ReplyTo::new(Token::new(9053)),
        by: tasks::Party::Person(PERSON),
        funder: None,
        places: Box::new([]),
        operation: tasks::Amend { task: PERSON_TASK, message, stop_run: false, amendment: Box::new(amendment) },
    })));
    None
}

fn raw_new(world: &mut World) -> tasks::New {
    let number = core::fresh(&mut world.domain.core_mut().counters, core::Family::Task).expect("task number");
    let source = live_row(world, PERSON_TASK);
    tasks::New {
        number,
        project: PROJECT,
        label: 0,
        flags: tasks::Flags::PLAIN,
        executor: tasks::Executor::Person(tasks::PersonAddress::Role(ROLE)),
        llm: None,
        spec: source.spec.clone(),
        contract: source.contract.clone(),
        authority: source.authority.clone(),
        deadline: tasks::Deadline::Named,
        numbers: tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: PROJECT, period: 1 },
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: None,
    }
}

fn raw_batch(world: &mut World, person: u64, field: BatchName) -> Option<Refusal> {
    let mut row = raw_new(world);
    let mut creator = tasks::Party::Deployment { project: PROJECT };
    match field {
        BatchName::Executor => row.executor = tasks::Executor::Person(tasks::PersonAddress::Person(person)),
        BatchName::Funder => row.funder = tasks::Funder::Pool { project: PROJECT, person, period: 1 },
        BatchName::Requester => creator = tasks::Party::Person(person),
    }
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Make {
        reply_to: ReplyTo::new(Token::new(9053)),
        creator,
        batch: Box::new([row]),
    })));
    None
}

#[derive(Clone, Copy)]
enum BatchName {
    Executor,
    Funder,
    Requester,
}

#[derive(Clone, Copy)]
enum NamedLink {
    Executor,
    Funder,
    Requester,
    Proposal,
    Escalation,
    Holding,
    ProposedAddress,
    PartyProposedAddress,
    ProposedFunder,
}

fn executor_batch(world: &mut World, person: u64) -> Option<Refusal> {
    raw_batch(world, person, BatchName::Executor)
}
fn funder_batch(world: &mut World, person: u64) -> Option<Refusal> {
    raw_batch(world, person, BatchName::Funder)
}
fn requester_batch(world: &mut World, person: u64) -> Option<Refusal> {
    raw_batch(world, person, BatchName::Requester)
}

fn agent_new(world: &mut World) -> tasks::New {
    let mut row = raw_new(world);
    row.executor = tasks::Executor::Agent { charter: 1 };
    row.authority = live_row(world, 1).authority.clone();
    row.authority.budget.spend = 10;
    row.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    row.numbers.budget = 10;
    row
}

fn requester_agent(world: &mut World, person: u64) -> bool {
    let row = agent_new(world);
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Make {
        reply_to: ReplyTo::new(Token::new(9053)),
        creator: tasks::Party::Person(person),
        batch: Box::new([row]),
    })));
    world.domain.core_mut().tasks.task(DELEGATE).is_some()
}

fn proposed_funder(world: &mut World, person: u64) -> Option<Refusal> {
    let mut row = agent_new(world);
    row.funder = tasks::Funder::Pool { project: PROJECT, person, period: 1 };
    named_action(
        world,
        1,
        core::NamedAction::Propose {
            action: tasks::ProposalAction::Batch(Box::new([row])),
            reason: b"review the batch".as_slice().into(),
            as_holder: false,
        },
    )
}

fn proposal(world: &mut World, person: u64) -> Option<Refusal> {
    if !requester_agent(world, person) {
        return None;
    }
    let authority = live_row(world, DELEGATE).authority.clone();
    named_action(
        world,
        DELEGATE,
        core::NamedAction::Propose {
            action: tasks::ProposalAction::Widen { task: DELEGATE, authority },
            reason: b"review the work".as_slice().into(),
            as_holder: false,
        },
    )
}

fn escalation(world: &mut World, person: u64) -> Option<Refusal> {
    if !requester_agent(world, person) {
        return None;
    }
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task: DELEGATE, why: tasks::Hold::Stopped })));
    None
}

fn linked(world: &World, task: u64, named: u64, field: NamedLink) -> bool {
    if let NamedLink::PartyProposedAddress = field {
        return world.store.rows.values().any(|record| matches!(record,
            root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row)))
                if matches!(&row.action, tasks::PersonAction::Amend { amendment, .. }
                    if amendment.addressed.is_some_and(|address| address.address == tasks::PersonAddress::Person(named)))));
    }
    if let NamedLink::Holding = field {
        return world.store.rows.values().any(|record| {
            matches!(record,
            root::Record::Core(core::Record::People(people::Stored::Roles { project: PROJECT, holdings }))
                if holdings.iter().any(|holding| holding.person == named))
        });
    }
    let row = live_row(world, task);
    match field {
        NamedLink::Executor => row.executor == tasks::Executor::Person(tasks::PersonAddress::Person(named)),
        NamedLink::Funder => row.funder == tasks::Funder::Pool { project: PROJECT, person: named, period: 1 },
        NamedLink::Requester => row.requester == tasks::Party::Person(named),
        NamedLink::ProposedFunder => row.proposal.as_ref().is_some_and(|proposal| {
            matches!(&proposal.action,
            tasks::ProposalAction::Batch(batch) if batch.iter().any(|member| matches!(member.funder,
                tasks::Funder::Pool { person, .. } if person == named)))
        }),
        NamedLink::Proposal => row.proposal.as_ref().is_some_and(|proposal| {
            matches!(proposal.state,
            tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(person), .. } if person == named)
        }),
        NamedLink::Escalation => matches!(row.escalation,
            tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(person), .. } if person == named),
        NamedLink::ProposedAddress => row.proposal.as_ref().is_some_and(|proposal| {
            matches!(&proposal.action,
            tasks::ProposalAction::Amend { amendment, .. } if amendment.addressed.is_some_and(|address|
                address.address == tasks::PersonAddress::Person(named)))
        }),
        NamedLink::Holding | NamedLink::PartyProposedAddress => unreachable!("checked in separate rows"),
    }
}

fn names_absent(record: &root::Record) -> bool {
    match record {
        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) => {
            row.executor == tasks::Executor::Person(tasks::PersonAddress::Person(PERSON_TASK))
                || row.requester == tasks::Party::Person(PERSON_TASK)
                || matches!(row.funder, tasks::Funder::Pool { person: PERSON_TASK, .. })
                || row.taken_by == Some(PERSON_TASK)
                || row.person_answer.is_some_and(|answer| answer.by == PERSON_TASK)
                || matches!(row.escalation, tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(PERSON_TASK), .. })
                || row.proposal.as_ref().is_some_and(|proposal| {
                    matches!(proposal.state, tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(PERSON_TASK), .. })
                    || matches!(&proposal.action, tasks::ProposalAction::Batch(batch)
                        if batch.iter().any(|member| matches!(member.funder, tasks::Funder::Pool { person: PERSON_TASK, .. })))
                    || matches!(&proposal.action, tasks::ProposalAction::Amend { amendment, .. }
                        if amendment.addressed.is_some_and(|address| address.address == tasks::PersonAddress::Person(PERSON_TASK)))
                })
        }
        root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row))) => {
            matches!(&row.action, tasks::PersonAction::Amend { amendment, .. }
                if amendment.addressed.is_some_and(|address| address.address == tasks::PersonAddress::Person(PERSON_TASK)))
        }
        root::Record::Core(core::Record::People(people::Stored::Roles { holdings, .. })) => {
            holdings.iter().any(|holding| holding.person == PERSON_TASK)
        }
        _ => false,
    }
}

fn task_rows(world: &World) -> Vec<root::Record> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            record
            @ root::Record::Core(core::Record::Tasks(_) | core::Record::People(people::Stored::Roles { .. })) => {
                Some(record.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table keeps every route's held and unknown person rows together")]
fn a_person_named_from_outside_is_held_or_refused_and_the_start_links() {
    struct Route {
        name: &'static str,
        setup: fn() -> World,
        input: fn(&mut World, u64) -> Option<Refusal>,
        task: u64,
        refusal: Option<Refusal>,
        field: NamedLink,
    }
    let routes = [
        Route {
            name: "party amendment",
            setup: world,
            input: amend,
            task: PERSON_TASK,
            refusal: Some(Refusal::Person(people::Refusal::Unknown)),
            field: NamedLink::Executor,
        },
        Route {
            name: "agent person task",
            setup: world,
            input: agent,
            task: DELEGATE,
            refusal: Some(Refusal::Task(tasks::Problem::new(None, tasks::Refusal::Unknown))),
            field: NamedLink::Executor,
        },
        Route {
            name: "connector batch executor",
            setup: world,
            input: executor_batch,
            task: DELEGATE,
            refusal: None,
            field: NamedLink::Executor,
        },
        Route {
            name: "connector batch funder",
            setup: world,
            input: funder_batch,
            task: DELEGATE,
            refusal: None,
            field: NamedLink::Funder,
        },
        Route {
            name: "connector batch requester",
            setup: world,
            input: requester_batch,
            task: DELEGATE,
            refusal: None,
            field: NamedLink::Requester,
        },
        Route {
            name: "procedure batch executor",
            setup: procedure_world,
            input: procedure,
            task: PROCEDURE_DELEGATE,
            refusal: Some(Refusal::Step(core::StepRefusal::Refused(tasks::Problem::new(
                Some(DELEGATE),
                tasks::Refusal::Unknown,
            )))),
            field: NamedLink::Executor,
        },
        Route {
            name: "party batch executor",
            setup: world,
            input: party_create,
            task: DELEGATE,
            refusal: Some(Refusal::Person(people::Refusal::Unknown)),
            field: NamedLink::Executor,
        },
        Route {
            name: "role holding",
            setup: world,
            input: roles,
            task: PERSON_TASK,
            refusal: Some(Refusal::Person(people::Refusal::Unknown)),
            field: NamedLink::Holding,
        },
        Route {
            name: "proposal holder",
            setup: world,
            input: proposal,
            task: DELEGATE,
            refusal: None,
            field: NamedLink::Proposal,
        },
        Route {
            name: "escalation holder",
            setup: world,
            input: escalation,
            task: DELEGATE,
            refusal: None,
            field: NamedLink::Escalation,
        },
        Route {
            name: "compound amendment",
            setup: world,
            input: compound_amend,
            task: PERSON_TASK,
            refusal: Some(Refusal::Compound { at: 1, why: people::Refusal::Unknown }),
            field: NamedLink::Executor,
        },
        Route {
            name: "agent amendment",
            setup: person_delegate_world,
            input: agent_amend,
            task: DELEGATE,
            refusal: Some(Refusal::Task(tasks::Problem::new(Some(DELEGATE), tasks::Refusal::Unknown))),
            field: NamedLink::Executor,
        },
        Route {
            name: "proposed amendment",
            setup: person_delegate_world,
            input: proposed_amend,
            task: 1,
            refusal: Some(Refusal::Task(tasks::Problem::new(Some(1), tasks::Refusal::Unknown))),
            field: NamedLink::ProposedAddress,
        },
        Route {
            name: "party proposed amendment",
            setup: world,
            input: party_proposed_amend,
            task: PERSON_TASK,
            refusal: Some(Refusal::Person(people::Refusal::Unknown)),
            field: NamedLink::PartyProposedAddress,
        },
        Route {
            name: "procedure amendment",
            setup: procedure_person_world,
            input: procedure_amend,
            task: PROCEDURE_DELEGATE,
            refusal: Some(Refusal::Step(core::StepRefusal::Refused(tasks::Problem::new(
                Some(PROCEDURE_DELEGATE),
                tasks::Refusal::Unknown,
            )))),
            field: NamedLink::Executor,
        },
        Route {
            name: "application amendment",
            setup: world,
            input: raw_amend,
            task: PERSON_TASK,
            refusal: None,
            field: NamedLink::Executor,
        },
        Route {
            name: "proposed batch funder",
            setup: world,
            input: proposed_funder,
            task: 1,
            refusal: Some(Refusal::Task(tasks::Problem::new(Some(1), tasks::Refusal::Unknown))),
            field: NamedLink::ProposedFunder,
        },
    ];
    for Route { name: route, setup, input, task, refusal, field } in routes {
        for (named, held) in [(PERSON, true), (PERSON_TASK, false)] {
            let case = if held { "held" } else { "absent" };
            eprintln!("route {route}, named person {named}, case {case}");
            let mut world = setup();
            let before = task_rows(&world);
            let answer = input(&mut world, named);
            let after = task_rows(&world);
            world.restart_with(configuration().0);
            assert_eq!(world.domain.start_failure(), None, "{route}, named {named}");
            assert!(world.domain.ready(), "{route}, named {named}: {:?}", world.domain.start_phase());
            assert_eq!(world.domain.start_phase(), None, "{route} completed Open, named {named}");
            if held {
                assert_eq!(answer, None, "{route}");
                assert!(linked(&world, task, named, field), "{route}: named {named}");
            } else {
                assert_eq!(answer, refusal, "{route}");
                assert_eq!(after, before, "{route} changes nothing");
            }
            assert!(!world.store.rows.values().any(names_absent), "{route} saves no absent person");
        }
    }
}
