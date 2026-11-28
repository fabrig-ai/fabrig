//! Creation checks and preparation use the held project charter (jig/docs/design/domain/engine/project-charters.md, 4–6).
use super::*;
use crate::routing::run::parts::{offered_kinds, project_words, run_instructions};
use crate::settings::{self as s, Overlay};
use jig_domain_contract as contract;
use tasks::{CharterScope, Executor, Reasons};

const PROJECT: u32 = 3;
const PARENT: u64 = 1;
const CHILD: u64 = 2;
const REQUESTED: Executor = Executor::Agent { charter: 2 };
const CAREFUL: Executor = Executor::Agent { charter: 33 };
const WHOLE_WORDS: &[u8] = b"Deployment instructions.";
const PROJECT_WORDS: &[u8] = b"Be careful.";
const FIRST_PART: &[u8] = b"Read the spec.";
const REVISION: u64 = 8;
const NEXT_SET: u64 = 9;
const KEY: CallKey = CallKey { task: PARENT, attempt: 1, completion: 1, position: 0 };

fn fixture(resources: &[u16]) -> (Core, Env<Limits>) {
    let mut cfg = config(17);
    let mut careful_model = scripted_model();
    careful_model.name = b"careful-model".as_slice().into();
    cfg.settings.models = Box::new([scripted_model(), careful_model.clone()]);
    let mut implementer = cfg.defaults.charters[0].clone();
    implementer.number = 2;
    implementer.name = Some(NAME.into());
    implementer.policy.resume = false;
    implementer.policy.instructions = WHOLE_WORDS.into();
    let mut careful = implementer.clone();
    careful.number = 33;
    careful.name = Some(b"careful".as_slice().into());
    careful.policy.model = careful_model;
    careful.hosts = fleet::Kinds::Engine;
    cfg.defaults.charters = Box::new([cfg.defaults.charters[0].clone(), implementer, careful]);
    let kinds: Box<[authority::Executor]> = Box::new([
        authority::Executor::Charter(1),
        authority::Executor::Charter(2),
        authority::Executor::Charter(33),
        authority::Executor::Procedure(1),
    ]);
    cfg.defaults.rules.ceiling.delegation.kinds = kinds.clone();
    cfg.seed_projects[0].policy.policy.ceiling.delegation.kinds = kinds.clone();
    cfg.seed_projects[0].policy.policy.roles[0].authority.delegation.kinds = kinds;
    let mut project = cfg.seed_projects[0].clone();
    project.number = PROJECT;
    project.name = Box::from("Project 3");
    cfg.seed_projects = Box::new([cfg.seed_projects[0].clone(), project]);
    let mut limits = limits();
    limits.tasks.charters = 3;
    limits.run_bytes = 4096;
    limits.authority.projects = 2;
    limits.authority.executors = 4;
    limits.people.projects = 3;
    limits.tasks.executor_kinds = 4;
    let declarations = fixture_declarations(&cfg, resources, &[], &[]);
    let mut core = core_with_roles(cfg, &limits, &declarations, &[0, 1, PROJECT]);
    let env = Env { limits, now: Time::ZERO, wall: Wall::EPOCH };
    fill::signed_in(&mut core, &env);
    drop(step(&mut core, &env, Event::Period { project: PROJECT, period: 1, budget: 1000, next: None }));
    core.counters.deployment.tasks = PARENT;
    (core, env)
}

fn take(core: &mut Core, env: &Env<Limits>, revision: u64, settings: &[u8], instructions: &[u8]) {
    let mut input = set(revision);
    input.project = PROJECT;
    input.charters[0].settings = settings.into();
    input.charters[0].instructions = instructions.into();
    input.charters[0].parts[0].instructions = FIRST_PART.into();
    assert!(saved(&step(core, env, Event::ProjectCharters(input.clone())), &input));
}

fn make(core: &mut Core, env: &Env<Limits>, new: tasks::New) -> Queue<Request> {
    step(
        core,
        env,
        Event::Tasks(tasks::Event::Make {
            reads_project: false,
            maker: None,
            places: Box::new([]),
            reply_to: ReplyTo::new(Token::new(520)),
            creator: tasks::Party::Deployment { project: PROJECT },
            batch: Box::new([new]),
        }),
    )
}

fn member(core: &Core, number: u64, funder: tasks::Funder) -> tasks::New {
    let mut member = new_row(core, number, funder, None);
    member.project = PROJECT;
    member.executor = REQUESTED;
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    member
}

const PERIOD: tasks::Funder = tasks::Funder::Period { project: PROJECT, period: 1 };

fn maker(core: &Core, executor: Executor) -> tasks::New {
    let mut parent = new_row(core, PARENT, PERIOD, None);
    parent.project = PROJECT;
    parent.executor = executor;
    parent
}

fn delegate(core: &Core) -> Delegate {
    let row = member(core, CHILD, tasks::Funder::Task(PARENT));
    Delegate {
        charter_scope: row.charter_scope,
        executor: row.executor,
        spec: row.spec,
        contract: row.contract,
        authority: row.authority,
        deadline: row.deadline,
        llm: row.llm,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: None,
        tracked: None,
        label: row.label,
        flags: row.flags,
        budget: people::Budget::Named,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Path {
    Party,
    Run,
    Procedure,
    Recurring,
    Proposal,
}

const PATHS: [Path; 5] = [Path::Party, Path::Run, Path::Procedure, Path::Recurring, Path::Proposal];

fn party(core: &mut Core, env: &Env<Limits>, key: u8, ask: people::Ask) -> Queue<Request> {
    step(
        core,
        env,
        Event::People(people::Event::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: 10,
            key: [key; 16],
            ask,
        }),
    )
}

fn parent(core: &mut Core, env: &Env<Limits>, row: tasks::New) {
    let out = make(core, env, row);
    assert!(core.tasks.task(PARENT).is_some(), "parent made: {out:?}");
}

fn create(core: &mut Core, env: &Env<Limits>, path: Path) -> Queue<Request> {
    match path {
        Path::Party => {
            let mut member = fixtures::member(b"child", 10);
            member.executor = people::TaskExecutor::Agent { charter: 2 };
            party(core, env, 72, people::Ask::Create { project: PROJECT, batch: Box::new([member]) })
        }
        Path::Run => {
            parent(core, env, maker(core, Executor::Agent { charter: 1 }));
            core.proofs.insert(PARENT, RunProof::claimed(PARENT, 1, 1, None, fleet::HostKind::Worker)).unwrap();
            let batch = Box::new([delegate(core)]);
            step(core, env, Event::Delegate { to: ReplyTo::new(Token::new(73)), key: KEY, batch })
        }
        Path::Procedure => {
            parent(core, env, maker(core, Executor::Procedure { connector: 1, code: 1 }));
            let (_, _, step_number) = core.tasks.procedure_due(PARENT).expect("procedure due");
            let batch = Box::new([delegate(core)]);
            connector_step(
                core,
                env,
                1,
                contract::Up::Procedures(contract::procedures::Up::Step {
                    task: PARENT,
                    step: step_number,
                    code: 1,
                    decision: ProcedureStep { batch, ..ProcedureStep::new(StepEnd::Wait) },
                }),
            )
        }
        Path::Recurring => {
            let mut row = maker(core, Executor::Procedure { connector: RECURRING, code: 1 });
            row.numbers.budget = 0;
            row.recurring = Some(Box::new(tasks::RecurringTemplate {
                key: 1,
                batch: Box::new([member(core, 1, PERIOD)]),
                overlap: tasks::RecurringOverlap::Skip,
            }));
            // The set is judged at each occurrence, not baked into the template.
            send_task(
                core,
                &env.limits,
                tasks::Event::Make {
                    reads_project: false,
                    maker: None,
                    places: Box::new([]),
                    reply_to: ReplyTo::new(Token::new(74)),
                    creator: tasks::Party::Deployment { project: PROJECT },
                    batch: Box::new([row]),
                },
            );
            assert!(core.tasks.task(PARENT).is_some(), "recurring root made");
            step(core, env, Event::Tasks(tasks::Event::TickRecurring { task: PARENT, period: 1 }))
        }
        Path::Proposal => {
            parent(core, env, maker(core, Executor::Agent { charter: 1 }));
            let proposal = 9;
            send_task(
                core,
                &env.limits,
                tasks::Event::Propose {
                    reply_to: ReplyTo::new(Token::new(75)),
                    proposal: tasks::Proposal {
                        number: proposal,
                        proposer: PARENT,
                        project: PROJECT,
                        action: tasks::ProposalAction::Batch(Box::new([member(core, CHILD, PERIOD)])),
                        reason: b"room".as_slice().into(),
                        as_holder: true,
                        state: tasks::ProposalState::Pending {
                            holder: tasks::ProposalHolder::Person(1),
                            since: env.wall,
                        },
                    },
                },
            );
            assert!(core.tasks.proposal(PARENT, proposal).is_some(), "proposal offered");
            party(
                core,
                env,
                76,
                people::Ask::DecideProposal {
                    project: PROJECT,
                    proposer: Proposer::Task(PARENT),
                    proposal,
                    decision: people::ProposalDecision::Accept,
                    running: None,
                },
            )
        }
    }
}

fn expect_bound(core: &Core, executor: Executor, revision: u64) -> &tasks::TaskRecord {
    let row = core.tasks.task(CHILD).expect("child made");
    assert_eq!(row.executor, executor);
    assert_eq!(row.charters, revision);
    assert_eq!(row.scope.as_deref(), Some(NAME));
    row
}

#[test]
fn every_creation_path_binds_the_task_to_the_newest_project_set() {
    for path in PATHS {
        let (mut core, env) = fixture(&[]);
        take(&mut core, &env, REVISION, NAME, PROJECT_WORDS);
        let made = create(&mut core, &env, path);
        assert!(core.tasks.task(CHILD).is_some(), "{path:?}: {made:?}");
        expect_bound(&core, REQUESTED, REVISION);
    }
}

#[test]
fn unbound_creations_keep_the_requested_charter() {
    for (installed, executor, scope) in [
        (false, REQUESTED, CharterScope::Project),
        (true, Executor::Agent { charter: 1 }, CharterScope::Project),
        (true, CAREFUL, CharterScope::Project),
        (true, REQUESTED, CharterScope::Deployment),
    ] {
        let (mut core, env) = fixture(&[]);
        if installed {
            take(&mut core, &env, REVISION, b"careful", PROJECT_WORDS);
        }
        let mut new = member(&core, CHILD, PERIOD);
        new.executor = executor;
        new.charter_scope = scope;
        let out = make(&mut core, &env, new);
        let Some(row) = core.tasks.task(CHILD) else { panic!("{out:?}") };
        assert_eq!(row.executor, executor);
        assert_eq!(row.scope, None);
        assert_eq!(row.charters, if installed { REVISION } else { 0 });
    }
}

fn choice(model: u8) -> tasks::LlmChoice {
    tasks::LlmChoice {
        later: None,
        account: None,
        main: tasks::ModelEffort { model, effort: Some(0), speed: None },
        subagents: Box::new([]),
    }
}

#[test]
fn settings_remap_the_executor_models_and_hosts_but_not_the_scope() {
    let (mut core, env) = fixture(&[]);
    take(&mut core, &env, REVISION, b"careful", PROJECT_WORDS);
    let mut new = member(&core, CHILD, PERIOD);
    let implementer = core.tasks.charter_models(2, core.tasks.newest()).unwrap()[0];
    let careful = core.tasks.charter_models(33, core.tasks.newest()).unwrap()[0];
    assert_ne!(implementer, careful);
    new.llm = Some(choice(implementer));
    let refused = make(&mut core, &env, new.clone());
    assert!(core.tasks.task(CHILD).is_none());
    let mut why = None;
    for request in &refused {
        if let Request::Now(now) = request
            && let Now::Link(LinkNow::RefusedPayload { problem, .. }) = now.as_ref()
        {
            why = Some(problem.why);
        }
    }
    assert_eq!(why, Some(tasks::Refusal::Llm { model: implementer, why: tasks::LlmFault::Charter }), "{refused:?}");
    new.llm = Some(choice(careful));
    let made = make(&mut core, &env, new);
    assert!(core.tasks.task(CHILD).is_some(), "{made:?}");
    expect_bound(&core, CAREFUL, REVISION);
    assert_eq!(core.charter_of(CHILD).hosts, fleet::Kinds::Engine);
    assert_eq!(core.charter_version(CHILD).unwrap().charter, 33);
}

fn setting(core: &mut Core, env: &Env<Limits>, value: s::Value, running: s::Running) -> u32 {
    core.reclaim();
    let against = core.current.newest();
    let next = against.checked_add(1).unwrap();
    let key = u8::try_from(next).unwrap();
    let out = step(
        core,
        env,
        Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(500_u64.checked_add(u64::from(key)).unwrap())),
            sign_in: 10,
            key: [key; 16],
            digest: [key; 32],
            change: Box::new(s::Change {
                against,
                items: Box::new([s::Item { setting: s::Setting::Charter(2), edit: s::Edit::Set(value) }]),
                running,
            }),
        },
    );
    assert_eq!(core.current.newest(), next, "{out:?}");
    next
}

fn overlay(words: &[u8], parts: Box<[CharterPart]>) -> Overlay {
    Overlay { instructions: words.into(), parts, ..Overlay::empty() }
}

fn words(core: &Core, reasons: Reasons) -> Box<[u8]> {
    let row = core.tasks.task(CHILD).expect("live task");
    run_instructions(core.charter_of(CHILD), project_words(core, CHILD), row.instructions.as_deref(), reasons, false)
}

#[test]
fn a_bound_run_takes_project_words_then_the_overlay_and_selected_parts() {
    let (mut core, env) = fixture(&[]);
    setting(&mut core, &env, s::Value::Overlay(Box::new(overlay(b"Use the box.", Box::new([])))), s::Running::Stays);
    take(&mut core, &env, REVISION, NAME, PROJECT_WORDS);
    let made = create(&mut core, &env, Path::Party);
    assert!(core.tasks.task(CHILD).is_some(), "{made:?}");
    assert_eq!(words(&core, Reasons::FIRST).as_ref(), b"Be careful.\n\nUse the box.\n\nRead the spec.");
    assert_eq!(words(&core, Reasons::QUESTION).as_ref(), b"Be careful.\n\nUse the box.");
    assert_eq!(
        offered_kinds(core.charter_of(CHILD), project_words(&core, CHILD), false, Reasons::FIRST, false),
        Reasons::EMPTY
    );
    assert_eq!(core.charter_of(CHILD).policy.instructions.as_ref(), b"Deployment instructions.\n\nUse the box.");
    let amended = step(
        &mut core,
        &env,
        Event::Tasks(tasks::Event::Amend {
            reads_project: false,
            operation: tasks::Amend {
                funder: None,
                task: CHILD,
                message: 1,
                stop_run: false,
                amendment: Box::new(tasks::Amendment {
                    tracked: None,
                    addressed: None,
                    release: None,
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: None,
                    llm: None,
                    reason: b"Task words".as_slice().into(),
                    instructions: Some(b"Amended instructions.".as_slice().into()),
                }),
            },
            places: Box::new([]),
            reply_to: ReplyTo::new(Token::new(521)),
            by: tasks::Party::Person(1),
        }),
    );
    assert_eq!(words(&core, Reasons::FIRST).as_ref(), b"Amended instructions.", "{amended:?}");
}

#[test]
fn whole_empty_and_resuming_charters_take_only_their_applicable_words() {
    for (overlay_words, resume, project, expected) in [
        (None, false, PROJECT_WORDS, b"Be careful.\n\nRead the spec.".as_slice()),
        (Some(b"Use the box.".as_slice()), false, b"".as_slice(), b"Use the box.\n\nRead the spec.".as_slice()),
        (Some(b"Use the box.".as_slice()), true, PROJECT_WORDS, b"Be careful.\n\nUse the box.".as_slice()),
    ] {
        let (mut core, env) = fixture(&[]);
        if let Some(added) = overlay_words {
            let mut value = overlay(added, Box::new([]));
            value.resume = Some(resume);
            setting(&mut core, &env, s::Value::Overlay(Box::new(value)), s::Running::Stays);
        }
        take(&mut core, &env, REVISION, NAME, project);
        let made = create(&mut core, &env, Path::Party);
        assert!(core.tasks.task(CHILD).is_some(), "{made:?}");
        assert_eq!(words(&core, Reasons::FIRST).as_ref(), expected);
    }
}

#[test]
fn moving_settings_keeps_the_set_and_scope_but_takes_the_new_overlay() {
    let (mut core, env) = fixture(&[]);
    let first = overlay(b"Old overlay.", Box::new([]));
    let revision = setting(&mut core, &env, s::Value::Overlay(Box::new(first)), s::Running::Stays);
    take(&mut core, &env, REVISION, NAME, PROJECT_WORDS);
    let made = create(&mut core, &env, Path::Party);
    assert!(core.tasks.task(CHILD).is_some(), "{made:?}");
    take(&mut core, &env, NEXT_SET, b"careful", b"New project words.");
    assert_eq!(words(&core, Reasons::FIRST).as_ref(), b"Be careful.\n\nOld overlay.\n\nRead the spec.");
    let mut value = overlay(b"New overlay.", Box::new([part(Reasons::QUESTION, b"Answer the question.")]));
    value.hosts = Some(fleet::Kinds::Engine);
    let newest = setting(&mut core, &env, s::Value::Overlay(Box::new(value)), s::Running::Moves);
    let row = expect_bound(&core, REQUESTED, REVISION);
    assert_ne!(revision, newest);
    assert_eq!(row.settings.task, newest);
    assert_eq!(core.charter_of(CHILD).hosts, fleet::Kinds::Engine);
    let reasons = Reasons::FIRST.union(Reasons::QUESTION);
    assert_eq!(
        words(&core, reasons).as_ref(),
        b"Be careful.\n\nNew overlay.\n\nRead the spec.\n\nAnswer the question."
    );
}

fn expect_refused(core: &Core, requests: &Queue<Request>, path: Path) {
    use contract::procedures::Ask::StepRefused as ProcedureRefused;
    assert!(core.tasks.task(CHILD).is_none(), "{path:?} must not make a child");
    if path == Path::Recurring {
        assert_eq!(core.tasks.task(PARENT).unwrap().recurring.as_ref().unwrap().last_period, 0);
        return;
    }
    for request in requests {
        if let Request::Held(held) = request {
            if let Held::PeopleReply {
                reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotOffered)),
                ..
            } = held.as_ref()
                && (path == Path::Party || path == Path::Proposal)
            {
                return;
            }
            if let Held::CallAnswer { part: CallPart::DelegationRefused(problem), .. } = held.as_ref()
                && path == Path::Run
                && problem.why == tasks::Refusal::Executor
            {
                return;
            }
        }
        if let Request::Ask {
            ask: Ask::Connector(contract::Ask::Procedures(ProcedureRefused { why: StepRefusal::Refused(problem), .. })),
            ..
        } = request
            && path == Path::Procedure
            && problem.why == tasks::Refusal::Executor
        {
            return;
        }
    }
    panic!("{path:?}: {requests:?}");
}

#[test]
fn an_unoffered_project_binding_refuses_on_every_creation_path() {
    for path in PATHS {
        let (mut core, env) = fixture(&[]);
        take(&mut core, &env, REVISION, b"missing", PROJECT_WORDS);
        let refused = create(&mut core, &env, path);
        expect_refused(&core, &refused, path);
    }
}

#[test]
fn binding_faults_name_the_project_charter_settings_revision_and_bound() {
    use s::binding::{BindingFault, Bound};
    let (mut core, mut env) = fixture(&[]);
    let value = overlay(b"Use the box.", Box::new([part(Reasons::QUESTION, b"Answer the question.")]));
    setting(&mut core, &env, s::Value::Overlay(Box::new(value)), s::Running::Stays);
    take(&mut core, &env, REVISION, b"missing", PROJECT_WORDS);
    assert_eq!(
        core.bind_charter(&env.limits, PROJECT, REQUESTED, CharterScope::Project),
        Err(BindingFault::Project {
            name: NAME.into(),
            settings: b"missing".as_slice().into(),
            revision: REVISION,
            bound: None
        }),
    );
    take(&mut core, &env, NEXT_SET, NAME, PROJECT_WORDS);
    let charter = core.charter(2).unwrap();
    let words = u64::try_from(PROJECT_WORDS.len()).unwrap();
    let deployment = u64::try_from(WHOLE_WORDS.len()).unwrap();
    let policy =
        policy_owned_bytes(&charter.policy).unwrap().checked_sub(deployment).unwrap().checked_add(words).unwrap();
    let parts = parts_owned_bytes(&charter.parts)
        .unwrap()
        .checked_add(parts_owned_bytes(&[part(Reasons::FIRST, FIRST_PART)]).unwrap())
        .unwrap();
    env.limits.run_bytes = u32::try_from(policy.checked_add(parts).unwrap()).unwrap();
    env.limits.tasks.charter_parts = 2;
    core.bind_charter(&env.limits, PROJECT, REQUESTED, CharterScope::Project).unwrap();
    env.limits.run_bytes = env.limits.run_bytes.checked_sub(1).unwrap();
    for bound in [Bound::RunBytes(env.limits.run_bytes), Bound::Parts(1)] {
        assert_eq!(
            core.bind_charter(&env.limits, PROJECT, REQUESTED, CharterScope::Project),
            Err(BindingFault::Project {
                name: NAME.into(),
                settings: NAME.into(),
                revision: NEXT_SET,
                bound: Some(bound)
            }),
        );
        env.limits.tasks.charter_parts = 1;
    }
}

#[test]
fn a_creation_waiting_for_holdings_uses_the_newest_set_only_for_the_same_executor() {
    use contract::resources::{Ask as ResourceAsk, Up as ResourceUp};
    for path in [Path::Party, Path::Run, Path::Procedure] {
        for (settings, allowed) in [(NAME, true), (b"careful".as_slice(), false)] {
            let (mut core, env) = fixture(&[1]);
            take(&mut core, &env, REVISION, NAME, PROJECT_WORDS);
            let asked = create(&mut core, &env, path);
            let mut answer = None;
            for request in &asked {
                let Request::Ask { connector: 1, ask: Ask::Connector(contract::Ask::Resources(ask)) } = request else {
                    continue;
                };
                if let ResourceAsk::Holdings { request, executor, .. } = ask {
                    assert_eq!(*executor, REQUESTED);
                    answer =
                        Some(ResourceUp::Holdings { request: *request, holdings: Some(Box::new([])), refused: None });
                } else if let ResourceAsk::DelegateHoldings { request, members, .. } = ask {
                    assert_eq!(members[0].executor, REQUESTED);
                    answer = Some(ResourceUp::DelegateHoldings {
                        request: *request,
                        holdings: Some(Box::new([Box::new([])])),
                        refused: None,
                    });
                } else if let ResourceAsk::ProcedureHoldings { task, step, members } = ask {
                    assert_eq!(members[0].executor, REQUESTED);
                    answer = Some(ResourceUp::ProcedureHoldings {
                        task: *task,
                        step: *step,
                        holdings: Some(Box::new([Box::new([])])),
                        refused: None,
                    });
                }
            }
            let Some(answer) = answer else { panic!("{path:?}: {asked:?}") };
            assert!(core.tasks.task(CHILD).is_none());
            take(&mut core, &env, NEXT_SET, settings, b"New project words.");
            let answered = connector_step(&mut core, &env, 1, contract::Up::Resources(answer));
            if allowed {
                assert!(core.tasks.task(CHILD).is_some(), "{path:?}: {answered:?}");
                expect_bound(&core, REQUESTED, NEXT_SET);
            } else {
                expect_refused(&core, &answered, path);
            }
        }
    }
}
