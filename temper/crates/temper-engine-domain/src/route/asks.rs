//! The core's connector asks, translated for temper's forge and brief
//! (jig's `domain/root.md`, 5; `examples/ops/domain/src/route.rs`).
use super::{
    Decision, Delivery, Domain, Env, Limits, List, Queue, ReplyTo, Work, brief_outputs, decide_call, emit, forge,
    forge_release_ending, forge_route,
};
use crate::CallAnswer;
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::Token;

#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher translates the core's ask vocabulary")]
pub(crate) fn ask(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, connector: u16, ask: core::Ask) {
    let request = match ask {
        ask @ core::Ask::Public { .. } => {
            super::public::ask(domain, env, connector, ask);
            None
        }
        core::Ask::ReleaseProject { project } => {
            if connector == domain.config.forge_connector {
                domain.work.push(Work::Forge(forge::Event::ReleaseProject {
                    project,
                    request: Token::new(u64::from(project)),
                }));
            } else {
                domain.work.push(Work::Core(core::Event::ProjectReleased { connector, project }));
            }
            None
        }
        core::Ask::StepMade { task, step, first, .. } => {
            step_made(domain, env, decision, task, step, first);
            None
        }
        core::Ask::StepRefused { task, step, why } => {
            if connector == domain.config.forge_connector && forge_route::closer_refused(domain, task, &why) {
                return;
            }
            if connector == domain.config.forge_connector && forge_route::status_refused(domain, task, &why) {
                return;
            }
            let pending = domain.forge_delegating.remove(&(task, step));
            let delegation = match &pending {
                Some(pending) => pending.delegate,
                None => None,
            };
            if let Some(pending) = pending
                && forge_route::relay_refused(domain, env, task, step, pending, &why)
            {
                return;
            }
            let held = connector == domain.config.forge_connector
                && hold_changed_charter(domain, task, step, delegation, &why);
            if let core::StepRefusal::Denied { findings, .. } = &why {
                if connector == domain.config.forge_connector
                    && let Some(kind) = delegation
                {
                    domain.work.push(Work::Forge(forge::Event::DelegateAdmissionRefused { task, kind }));
                }
                if !held {
                    forge_route::refused_step(domain, env, task, step, findings);
                }
            }
            if let core::StepRefusal::Refused(problem) = why {
                if connector == domain.config.forge_connector
                    && problem.freeing_count().is_some()
                    && let Some(kind) = delegation
                {
                    domain.work.push(Work::Forge(forge::Event::DelegateAdmissionRefused { task, kind }));
                }
                // Jig's domain/connectors.md, 6.2: step again on refusal. A kept change waits
                // without a deadline; its dependency's Met wakes it (jig's domain/tasks/lifecycle.md, 8).
                if !held
                    && connector == domain.config.forge_connector
                    && domain.core.tasks.waiting_effect_problem(task).as_ref() == Some(&problem)
                    && let Some((owner, code @ 2, current)) = domain.core.tasks.procedure_due(task)
                    && owner == connector
                    && current == step
                {
                    let decision = forge_route::append_change_replies(
                        domain,
                        env,
                        task,
                        core::ProcedureStep::new(core::StepEnd::Wait),
                    );
                    domain.work.push(Work::Core(core::Event::ProcedureStep { task, step, connector, code, decision }));
                }
                if domain.facts.try_push(crate::Fact::StepRefused { task, step, problem }).is_err() {
                    domain.facts_lost = domain.facts_lost.saturating_add(1);
                }
            }
            None
        }
        core::Ask::Document { watcher, scope, subject, party, .. } => {
            super::panels::document(domain, env, decision, connector, watcher, scope, subject, party);
            None
        }
        core::Ask::Read { reply, scope, read, party, .. } => {
            let right = reply.into_token();
            let reply = ReplyTo::new(right);
            if super::see::person::is_reading(domain, right) {
                let document = if connector == domain.config.forge_connector {
                    domain.forge.document(read.raw(), scope, party, env.wall, &env.limits.forge)
                } else {
                    None
                };
                super::see::person::document(domain, reply, document);
                return;
            }
            emit(
                decision,
                &env.limits,
                Delivery::Read { reply, answer: core::reads::Answer::None(core::reads::Why::Unknown) },
            );
            None
        }
        core::Ask::Catalogue { models } => {
            domain.catalogue_pending = true;
            let mut names = List::with_capacity(u32::try_from(models.len()).expect("bounded catalogue"));
            for row in &models {
                names
                    .push(core::settings::ModelName {
                        name: row.model.name.clone(),
                        efforts: row.model.efforts.clone(),
                    })
                    .expect("one name per position");
            }
            assert!(
                super::now(domain, crate::Request::Store(crate::StoreRequest::Catalogue { names: names.into_boxed() })),
                "catalogue store door reserved"
            );
            assert!(
                super::now(domain, crate::Request::Party(crate::PartyRequest::Catalogue { models })),
                "catalogue clients door reserved"
            );
            None
        }
        core::Ask::DropDocument { .. } => None,
        core::Ask::DropApplication { owner } => {
            super::applications::drop(domain, owner);
            None
        }
        core::Ask::Application { request, token, owner, retention, role, person, .. } => {
            super::applications::admitted(domain, request, token, owner, retention, role, person, env, decision);
            None
        }
        core::Ask::ApplicationPartAnswered { request, outcome } => {
            domain.work.push(Work::Forge(forge::Event::ApplicationAnswered { request, outcome }));
            None
        }
        core::Ask::Effect(ask) => {
            forge_route::effect_ask(domain, env, connector, ask);
            None
        }
        core::Ask::Gather { section, budget } => Some(brief::GatherRequest::Gather { connector, section, budget }),
        core::Ask::CutTo { section, size } => Some(brief::GatherRequest::CutTo { connector, section, size }),
        core::Ask::Drop { section } => Some(brief::GatherRequest::Drop { connector, section }),
        core::Ask::Hold { entry, task, resource, .. } => {
            if connector == domain.config.forge_connector
                && let Some(name) =
                    forge_route::forge_name(domain.config.forge_connector, &resource, env.limits.forge.name_bytes)
            {
                let from = match domain.forge.hold(&name) {
                    Some(row) if row.task != task => Some(row.task),
                    Some(_) | None => None,
                };
                domain.work.push(Work::Forge(forge::Event::Hold { entry, task, resource: name, from }));
            }
            None
        }
        core::Ask::EndTopic { task, subscription } => {
            if connector == domain.config.forge_connector
                && let Some(topic) = domain.forge.subscription(task, subscription)
            {
                domain.work.push(Work::Forge(forge::Event::Unsubscribe { task, topic }));
            }
            None
        }
        core::Ask::Adopt { request, project, adoption } => {
            if connector == domain.config.forge_connector
                && domain.core.application_part_request(request).is_none()
                && let Some((adoption, repository)) = super::making::asked(domain, project, &adoption)
            {
                if domain.adoption_restore.len() < env.limits.forge.adoptions {
                    let previous = domain.forge.repository(repository).cloned();
                    assert!(
                        domain.adoption_restore.insert(request, (request, previous)) == Ok(None),
                        "one act adoption flight"
                    );
                    domain.work.push(Work::Forge(forge::Event::MakingAsked { request, adoption, repository }));
                } else {
                    domain.work.push(Work::People(people::Event::Decided {
                        request,
                        outcome: people::Outcome::Again(people::Again::Busy),
                    }));
                }
                return;
            }
            let located = if connector == domain.config.forge_connector
                && let Some(application) = domain.core.application_part_request(request)
                && let Some(repository) = domain.forge.applying(application)
            {
                Some((application, repository))
            } else {
                None
            };
            match located {
                Some((application, repository)) if domain.adoption_restore.len() < env.limits.forge.adoptions => {
                    let previous = domain.forge.repository(repository).cloned();
                    assert!(
                        domain.adoption_restore.insert(application, (request, previous)) == Ok(None),
                        "one keyed adoption flight"
                    );
                    domain.work.push(Work::Forge(forge::Event::Adopt { request: application }));
                }
                Some(_) => domain.work.push(Work::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::Again(people::Again::Busy),
                })),
                None => domain.work.push(Work::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::Refused(people::Refusal::NotOffered),
                })),
            }
            None
        }
        core::Ask::Close { task, root, ending } => {
            if connector == domain.config.forge_connector {
                domain.work.push(Work::Forge(forge::Event::SettleEffects {
                    task,
                    root,
                    ending: forge_release_ending(ending),
                }));
            }
            None
        }
        core::Ask::Release { task, root, ending, entry } => {
            if connector == domain.config.forge_connector {
                let branches = forge_route::saved_branches(domain, env, task, root);
                if !branches.is_empty() {
                    domain.work.push(Work::Forge(forge::Event::SavedWork { task, branches }));
                }
                let account = match &ending {
                    tasks::Ending::Done(tasks::TaskResult::NoChange { .. }) => forge::Ending::NoChange,
                    tasks::Ending::Done(_) => forge::Ending::Landed,
                    tasks::Ending::Failed { reason } => {
                        forge::Ending::Failed { line: temper_engine_domain_forge_closer::first_line(reason).into() }
                    }
                    tasks::Ending::Cancelled { .. } => forge::Ending::Cancelled,
                };
                domain.work.push(Work::Forge(forge::Event::ChangeEnding { task, ending: account }));
                if let Some(mut row) = domain.forge.closer(task).cloned() {
                    row.done = true;
                    domain.work.push(Work::Forge(forge::Event::Closer { row }));
                }
                let reason = release_reason(domain, task, &ending);
                domain.work.push(Work::Forge(forge::Event::Release {
                    task,
                    root,
                    ending: forge_release_ending(ending),
                    reason,
                    entry,
                }));
            }
            None
        }
        core::Ask::Lost { task, attempt } => {
            if connector == domain.config.forge_connector {
                match domain.forge_left.remove(&(task, attempt)) {
                    Some(pushed) if !pushed.is_empty() => {
                        forge_route::left(domain, env, task, attempt, pushed);
                    }
                    Some(_) | None => {
                        domain.work.push(Work::Forge(forge::Event::Lost { task, attempt }));
                    }
                }
            }
            None
        }
        core::Ask::TaskHoldings { request, project, root, number, executor, spec, tracked, symbols } => {
            let (holdings, refused) = if connector == domain.config.forge_connector {
                match forge_route::task_holdings(
                    domain, env, project, root, number, executor, &spec, None, tracked, &symbols,
                ) {
                    Ok(holdings) => (Some(holdings), None),
                    Err(refused) => (None, refused),
                }
            } else {
                (None, None)
            };
            domain.work.push(Work::Core(core::Event::Holdings { refused, request, connector, holdings }));
            None
        }
        core::Ask::DelegateHoldings { request, from, members } => {
            let mut refused = None;
            let holdings = if connector == domain.config.forge_connector {
                let mut collected = List::with_capacity(u32::try_from(members.len()).expect("bounded delegate batch"));
                let mut failed = false;
                for member in members {
                    match forge_route::task_holdings(
                        domain,
                        env,
                        member.project,
                        member.root,
                        member.number,
                        member.executor,
                        &member.spec,
                        Some(from),
                        member.tracked,
                        &member.symbols,
                    ) {
                        Ok(holdings) => collected.push(holdings).expect("one per member"),
                        Err(reason) => {
                            refused = reason;
                            failed = true;
                            break;
                        }
                    }
                }
                if failed { None } else { Some(collected.into_boxed()) }
            } else {
                None
            };
            domain.work.push(Work::Core(core::Event::DelegateHoldings { refused, request, connector, holdings }));
            None
        }
        core::Ask::ProcedureHoldings { task, step, members } => {
            let mut refused = None;
            let holdings = if connector == domain.config.forge_connector {
                let mut collected = List::with_capacity(u32::try_from(members.len()).expect("bounded procedure batch"));
                let mut failed = false;
                for member in members {
                    match forge_route::task_holdings(
                        domain,
                        env,
                        member.project,
                        member.root,
                        member.number,
                        member.executor,
                        &member.spec,
                        Some(task),
                        member.tracked,
                        &member.symbols,
                    ) {
                        Ok(holdings) => collected.push(holdings).expect("one per member"),
                        Err(reason) => {
                            refused = reason;
                            failed = true;
                            break;
                        }
                    }
                }
                if failed { None } else { Some(collected.into_boxed()) }
            } else {
                None
            };
            domain.work.push(Work::Core(core::Event::ProcedureHoldings { refused, task, step, connector, holdings }));
            None
        }
        core::Ask::ProjectGoal { feed } => {
            if connector == domain.config.forge_connector {
                domain.work.push(Work::ProjectGoal(feed));
            } else if feed.closing {
                domain.work.push(Work::Core(core::Event::ProjectionSettled { goal: feed.goal.number, connector }));
            }
            None
        }
        core::Ask::ForgetProjection { goal } => {
            if connector == domain.config.forge_connector {
                domain.work.push(Work::Forge(forge::Event::ForgetProjection { goal }));
            }
            None
        }
        core::Ask::SubscriptionDone { request, key, subscription } => {
            if connector == domain.config.forge_connector {
                if let Some(pending) = domain.forge_subscribing.remove(&request) {
                    let names = forge_route::watch_names(domain, &env.limits, &pending)
                        .expect("connector names preflighted at subscription");
                    let owner = pending.task;
                    domain.work.push(Work::Forge(forge::Event::Subscribe { subscription: pending }));
                    domain.work.push(Work::Forge(forge::Event::Names { task: owner, resources: names }));
                }
                decide_call(
                    domain,
                    &env.limits,
                    decision,
                    ReplyTo::new(request),
                    key,
                    CallAnswer::Subscribed { subscription },
                );
            }
            None
        }
        core::Ask::UnsubscriptionDone { request, key } => {
            if connector == domain.config.forge_connector {
                if let Some((owner, topic)) = domain.forge_unsubscribing.remove(&request) {
                    domain.work.push(Work::Forge(forge::Event::Unsubscribe { task: owner, topic }));
                }
                decide_call(domain, &env.limits, decision, ReplyTo::new(request), key, CallAnswer::Unsubscribed);
            }
            None
        }
        core::Ask::ClaimDone { task, attempt, kinds, version } => {
            if connector == domain.config.forge_connector {
                let workspace = &domain.assignment_parts.get(&task).expect("admitted assignment").workspace;
                let (writes, designs, holders) = forge_route::claimed_writes(domain, env, task, workspace)
                    .expect("the admitted assignment retains its bounded held forge writes");
                if !writes.is_empty() || !designs.is_empty() {
                    domain.work.push(Work::Forge(forge::Event::Claim { task, attempt, writes, designs, holders }));
                }
                let mut out = Queue::uncapped();
                jig_domain_hub_link::step(
                    &mut domain.hub_link,
                    env.now,
                    jig_domain_hub_link::Event::Start { task, attempt, kinds, version },
                    &mut out,
                );
                for _ in 0..out.len() {
                    let jig_domain_hub_link::Request::Core(core::Event::Fleet(mut event)) =
                        out.pop().expect("link start")
                    else {
                        unreachable!("a start routes to the fleet")
                    };
                    if let jig_domain_fleet::Event::Start { workstream, .. } = &mut event {
                        *workstream = domain.assignment_parts.get(&task).expect("temper workspace").workstream;
                    }
                    emit(decision, &env.limits, Delivery::Fleet(event));
                }
            }
            None
        }
        core::Ask::JobClaimed { task, attempt, kind, spec, deadline, result, grants, workspace } => {
            if connector == domain.config.forge_connector {
                let crate::boundary::PreparedWorkspace { workspace: prepared, claim_names } =
                    domain.run_workspaces.remove(&task).expect("prepared job workspace").into_prepared();
                drop(super::now::claim_workspace(domain, env, task, &prepared, claim_names));
                let (writes, designs, holders) = forge_route::claimed_writes(domain, env, task, &prepared.workspace)
                    .expect("the admitted job retains its bounded held forge writes");
                if !writes.is_empty() || !designs.is_empty() {
                    domain.work.push(Work::Forge(forge::Event::Claim { task, attempt, writes, designs, holders }));
                }
                if workspace.is_some() {
                    assert!(
                        domain.job_workspaces.insert(task, prepared.workspace).is_ok(),
                        "one job assignment per task"
                    );
                }
                let mut out = Queue::uncapped();
                jig_domain_hub_link::step(
                    &mut domain.hub_link,
                    env.now,
                    jig_domain_hub_link::Event::StartJob {
                        job: jig_domain_hub_link::JobStart {
                            task,
                            attempt,
                            workstream: prepared.workstream,
                            kind,
                            spec,
                            deadline,
                            result,
                            grants,
                            workspace,
                        },
                    },
                    &mut out,
                );
                for _ in 0..out.len() {
                    let jig_domain_hub_link::Request::Core(core::Event::Fleet(event)) =
                        out.pop().expect("link job start")
                    else {
                        unreachable!("a job start routes to the fleet")
                    };
                    emit(decision, &env.limits, Delivery::Fleet(event));
                }
            }
            None
        }
        core::Ask::DropSubscription { request } => {
            if connector == domain.config.forge_connector {
                drop(domain.forge_subscribing.remove(&request));
                drop(domain.forge_unsubscribing.remove(&request));
            }
            None
        }
        core::Ask::RepairRefused { repair } => {
            if connector == domain.config.forge_connector
                && let Some(task) = repair
                && let Some(project) = domain.forge.status_project(task)
            {
                domain.work.push(Work::Forge(forge::Event::StatusRefused { project }));
            }
            if connector == domain.config.forge_connector
                && let Some(repair) = repair
                && let Some(owner) = domain.forge.queue_repair_owner(repair)
            {
                forge_route::hold_reason(domain, owner, b"landing branch repair admission was refused");
            }
            None
        }
        core::Ask::DelegateRefused { task } => {
            if connector == domain.config.forge_connector
                && let Some(task) = task
                && let Some(row) = domain.forge.change(task)
                && let Some((child, _)) = row.delegate
            {
                domain.work.push(Work::Forge(forge::Event::DelegateRefused { task, child }));
                domain.work.push(Work::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Procedure }));
            }
            None
        }
        core::Ask::StartProcedure { context, step } => {
            let task = context.task;
            let code = match context.executor {
                tasks::Executor::Job { .. } => return,
                tasks::Executor::Procedure { code, .. } => code,
                tasks::Executor::Agent { .. } | tasks::Executor::Person(_) => {
                    unreachable!("procedure route owns a procedure context")
                }
            };
            if connector == domain.config.forge_connector && code == forge_route::CHANGE_CODE {
                if !forge_route::start_change(domain, env, &context) {
                    forge_route::start_refused(domain, task);
                }
            } else if connector == domain.config.forge_connector && code == forge_route::LANDING_CODE {
                forge_route::start_landing_step(domain, env, &context);
            } else if connector == domain.config.forge_connector && code == forge_route::GOAL_CLOSER_CODE {
                forge_route::start_closer_step(domain, env, &context);
            } else if connector == domain.config.forge_connector && code == forge_route::STATUS_CODE {
                forge_route::start_status_step(domain, env, &context);
            } else if connector == domain.config.forge_connector && code == forge_route::AFTER_LANDING_CODE {
                forge_route::start_tier_step(domain, env, &context);
            } else {
                emit(decision, &env.limits, Delivery::Procedure { task, step, connector, code });
            }
            None
        }
    };
    if let Some(request) = request {
        let mut child = Queue::with_capacity(1);
        child.push(request);
        brief_outputs(domain, env, decision, &mut child);
    }
}

/// A change keeps its roles' charters and choices (temper/docs/design/domain/forge/change/gates.md, 3).
fn hold_changed_charter(
    domain: &mut Domain,
    task: u64,
    step: u64,
    delegation: Option<temper_engine_domain_forge_change::Delegate>,
    refusal: &core::StepRefusal,
) -> bool {
    use temper_engine_domain_forge_change::{Delegate, GateKind};
    let Some((owner, 2, current)) = domain.core.tasks.procedure_due(task) else { return false };
    if owner != domain.config.forge_connector || current != step {
        return false;
    }
    let Some(row) = domain.forge.change(task) else { return false };
    let (role, charter): (&[u8], u32) = match delegation {
        Some(Delegate::Gate { number, .. }) => {
            let mut kind = None;
            for gate in &row.change.gates {
                if gate.number == number {
                    kind = Some(gate.kind);
                    break;
                }
            }
            match kind {
                Some(GateKind::Agent) => (b"review", row.review_charter),
                Some(GateKind::Check) => (b"producer", row.producing_charter),
                Some(GateKind::Person) | None => return false,
            }
        }
        Some(Delegate::Produce | Delegate::Repair(_) | Delegate::Resolve { .. }) => {
            (b"producer", row.producing_charter)
        }
        Some(Delegate::Merge { .. } | Delegate::Close) | None => return false,
    };
    let model = match refusal {
        core::StepRefusal::Denied { findings, .. }
            if executor_refused(findings)
                && (!domain
                    .core
                    .authority
                    .rules()
                    .ceiling
                    .delegation
                    .kinds
                    .contains(&jig_domain_authority::Executor::Charter(charter))
                    || domain.core.charter(charter).is_none()) =>
        {
            None
        }
        core::StepRefusal::Refused(tasks::Problem {
            why: tasks::Refusal::Llm { model, why: tasks::LlmFault::Charter },
            ..
        }) => Some(*model),
        core::StepRefusal::Moved { .. } | core::StepRefusal::Refused(_) | core::StepRefusal::Denied { .. } => {
            return false;
        }
    };
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append_hold(&mut words, b"the change's ");
    append_hold(&mut words, role);
    match model {
        Some(model) => {
            append_hold(&mut words, b" choice names model ");
            let Some(model) = domain.core.settings.models.get(usize::from(model)) else { return false };
            append_hold(&mut words, &model.name);
            append_hold(&mut words, b", which charter ");
            append_hold(&mut words, &forge_route::decimal(u64::from(charter)));
            append_hold(&mut words, b" no longer allows");
        }
        None => {
            append_hold(&mut words, b" charter ");
            append_hold(&mut words, &forge_route::decimal(u64::from(charter)));
            append_hold(&mut words, b" is no longer offered");
        }
    }
    append_hold(&mut words, b"; cancel the change or make it again");
    domain.work.push(Work::Tasks(tasks::Event::HoldFrom {
        task,
        why: tasks::Hold::Procedure,
        source: Box::new(tasks::HoldSource::Account {
            connector: domain.config.forge_connector,
            entry: None,
            words: words.into_boxed(),
        }),
    }));
    true
}

fn executor_refused(findings: &[jig_domain_authority::Finding]) -> bool {
    for finding in findings {
        if let jig_domain_authority::Finding::Executor { .. } = finding {
            return true;
        }
    }
    false
}

fn append_hold(words: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        if words.room() > 0 {
            words.push(*byte).expect("checked hold account room");
        }
    }
}

fn step_made(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    task: u64,
    step: u64,
    child: Option<u64>,
) {
    forge_route::closer_made(domain, task, child);
    if domain.forge.tier_task(task).is_some()
        && let Some(row) = domain.core.tasks.task(task)
    {
        domain.work.push(Work::Forge(forge::Event::TierWords { task, words: row.spec.words.clone() }));
    }
    if let Some(pending) = domain.forge_delegating.remove(&(task, step)) {
        if let Some(kind) = pending.delegate
            && let Some(child) = child
        {
            // Jobs prepare at activation, after their parent row names them.
            super::forge_event(domain, env, decision, forge::Event::Delegated { task, child, kind });
        }
        if let Some(mut relayed) = pending.relayed {
            if pending.foreman {
                let question = relayed.questions.open.first_mut().expect("the made foreman has a head question");
                question.stands = super::forge_change::Stands::Foreman { task: child };
            }
            let event = match pending.refusal {
                Some(words) => forge::Event::RelayRefused { task, questions: relayed.questions, words },
                None => forge::Event::Relayed { task, questions: relayed.questions },
            };
            domain.work.push(Work::Forge(event));
            if relayed.more {
                domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task }));
            }
        }
    }
}

/// The task owns its bounded ending until connector release settles.
fn release_reason(domain: &Domain, task: u64, ending: &tasks::Ending) -> Option<Box<[u8]>> {
    match ending {
        tasks::Ending::Cancelled { .. } => {}
        tasks::Ending::Done(_) | tasks::Ending::Failed { .. } => return None,
    }
    let task = domain.core.tasks.task(task)?;
    let closing = match &task.phase {
        tasks::Phase::Closing(closing) | tasks::Phase::Held { was: tasks::Was::Closing(closing), .. } => closing,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } | tasks::Phase::Ended(_) => {
            return None;
        }
    };
    match &closing.ending {
        tasks::Ending::Cancelled { reason, .. } => Some(reason.clone()),
        tasks::Ending::Done(_) | tasks::Ending::Failed { .. } => None,
    }
}

#[cfg(test)]
// The private routing seam is tested beside the root's world.
#[path = "../../../../tests/engine/domain/src/catalogue.rs"]
mod catalogue_tests;
