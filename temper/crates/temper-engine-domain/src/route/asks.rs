//! Route the engine's connector port and temper's application asks
//! (jig's `domain/root.md`, 5; `examples/ops/domain/src/route.rs`).
use super::{
    Decision, Delivery, Domain, Env, Limits, List, Queue, ReplyTo, Work, brief_outputs, emit, forge,
    forge_release_ending, forge_route,
};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_contract::{
    Ask, Connector, Released, Up, effects, lifecycle, parts, procedures, projections, requests, resources, settings,
    topics,
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

/// Contract cases that need root state stay here; the rest reach the forge unchanged.
#[expect(clippy::too_many_lines, reason = "root-owned cases of the connector port, grouped by capability")]
pub(super) fn connector_ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    Connector(connector): Connector,
    ask: Ask,
) {
    match ask {
        Ask::Parts(parts::Ask::Lines(ask)) => {
            super::parts::ask(domain, env, decision, connector, ask);
        }
        Ask::Parts(parts::Ask::Document { watcher, scope, subject, party, .. }) => {
            super::panels::document(domain, env, decision, connector, watcher, scope, subject, party);
        }
        Ask::Parts(parts::Ask::Read { reply, scope, read, party, .. }) => {
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
        }
        Ask::Procedures(procedures::Ask::StepMade { task, step, first, .. }) => {
            step_made(domain, env, decision, task, step, first);
        }
        Ask::Procedures(procedures::Ask::StepRefused { task, step, why }) => {
            if connector == domain.config.forge_connector && forge_route::closer_refused(domain, task, step, &why) {
                return;
            }
            if connector == domain.config.forge_connector && forge_route::status_refused(domain, task, &why) {
                return;
            }
            if connector == domain.config.forge_connector
                && forge_route::follow_route::refused(domain, task, step, &why)
            {
                if let core::StepRefusal::Refused(problem) = &why
                    && domain.facts.try_push(crate::Fact::StepRefused { task, step, problem: problem.clone() }).is_err()
                {
                    domain.facts_lost = domain.facts_lost.saturating_add(1);
                }
                return;
            }
            let pending = domain.forge_delegating.remove(&(task, step));
            let delegation = if let Some(pending) = &pending
                && let Some((kind, _)) = pending.delegate
            {
                Some(kind)
            } else {
                None
            };
            if let Some(pending) = pending
                && forge_route::relay_refused(domain, env, task, step, pending, &why)
            {
                return;
            }
            let held = connector == domain.config.forge_connector
                && forge_route::hold_changed_charter(domain, task, step, delegation, &why);
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
                    && tasks::freeing_count(&problem).is_some()
                    && let Some(kind) = delegation
                {
                    domain.work.push(Work::Forge(forge::Event::DelegateAdmissionRefused { task, kind }));
                }
                // Jig's domain/connectors-procedures.md, 1.2: step again on refusal. A kept change waits
                // without a deadline; its dependency's Met wakes it (jig's domain/tasks/lifecycle-endings.md, 1).
                if !held
                    && connector == domain.config.forge_connector
                    && domain.engine.core().tasks.waiting_effect_problem(task).as_ref() == Some(&problem)
                    && let Some((owner, code @ 2, current)) = domain.engine.core().tasks.procedure_due(task)
                    && owner == connector
                    && current == step
                {
                    let decision = forge_route::append_change_replies(
                        domain,
                        env,
                        task,
                        core::ProcedureStep::new(core::StepEnd::Wait),
                    );
                    domain.work.push(Work::Connector(
                        Connector(connector),
                        Up::Procedures(procedures::Up::Step { task, step, code, decision }),
                    ));
                }
                if domain.facts.try_push(crate::Fact::StepRefused { task, step, problem }).is_err() {
                    domain.facts_lost = domain.facts_lost.saturating_add(1);
                }
            }
        }
        Ask::Procedures(procedures::Ask::Start { context, step, .. }) => {
            if domain.engine.core().settling() {
                return;
            }
            let task = context.task;
            let code = match context.executor {
                tasks::Executor::Job { .. } => return,
                tasks::Executor::Procedure { code, .. } => code,
                tasks::Executor::Agent { .. } | tasks::Executor::Person(_) => {
                    unreachable!("procedure route owns a procedure context")
                }
            };
            if connector == domain.config.forge_connector && code == forge_route::CHANGE_CODE {
                if forge_route::delegate_record::start(domain, env, decision, &context) {
                    return;
                }
                if !forge_route::start_change(domain, env, &context) {
                    forge_route::start_refused(domain, task);
                }
            } else if connector == domain.config.forge_connector && code == forge_route::LANDING_CODE {
                forge_route::start_landing_step(domain, env, &context);
            } else if connector == domain.config.forge_connector && code == forge_route::GOAL_CLOSER_CODE {
                if let Some(row) = domain.engine.core().tasks.task(task)
                    && row.due_reason == tasks::Why::Released
                {
                    domain.forge.release_closer_wait(task, env.wall);
                }
                forge_route::start_closer_step(domain, env, decision, &context);
            } else if connector == domain.config.forge_connector && code == forge_route::STATUS_CODE {
                forge_route::start_status_step(domain, env, &context);
            } else if connector == domain.config.forge_connector && code == forge_route::FOLLOW_CODE {
                forge_route::follow_route::step(domain, env, &context);
            } else if connector == domain.config.forge_connector && code == forge_route::AFTER_LANDING_CODE {
                forge_route::start_tier_step(domain, env, &context);
            } else {
                emit(decision, &env.limits, Delivery::Procedure { task, step, connector, code });
            }
        }
        Ask::Lifecycle(lifecycle::Ask::Adopt { request, project, adoption }) => {
            if connector == domain.config.forge_connector
                && domain.engine.core().application_part_request(request).is_none()
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
                && let Some(application) = domain.engine.core().application_part_request(request)
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
        }
        Ask::Effects(effects::Ask::Close { task, root, ending }) => {
            if connector == domain.config.forge_connector {
                forge_route::goal_report::forget(domain, task);
                forward(domain, Ask::Effects(effects::Ask::Close { task, root, ending }));
            }
        }
        Ask::Effects(effects::Ask::Release { task, root, ending, entry }) => {
            if connector == domain.config.forge_connector {
                super::findings::goal_ended(domain, env, decision, task, forge_release_ending(ending.clone()));
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
        }
        ask @ Ask::Resources(resources::Ask::Lost { task, attempt, .. }) => {
            if connector == domain.config.forge_connector {
                forge_route::push_admission::forget(domain, task, attempt);
                let pushed = domain.forge_left.remove(&(task, attempt));
                domain.work.push(Work::Forge(forge::Event::Finished { task, attempt, lost: pushed.is_none() }));
                match pushed {
                    Some(pushed) if !pushed.is_empty() => {
                        forge_route::left(domain, env, task, attempt, pushed);
                    }
                    Some(_) | None => {
                        forward(domain, ask);
                    }
                }
            }
        }
        Ask::Resources(resources::Ask::Holdings {
            request,
            project,
            root,
            number,
            executor,
            spec,
            tracked,
            symbols,
        }) => {
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
            domain.work.push(Work::Connector(
                Connector(connector),
                Up::Resources(resources::Up::Holdings { refused, request, holdings }),
            ));
        }
        Ask::Resources(resources::Ask::DelegateHoldings { request, from, members }) => {
            let (holdings, refused) = batch_holdings(domain, env, connector, from, &members);
            domain.work.push(Work::Connector(
                Connector(connector),
                Up::Resources(resources::Up::DelegateHoldings { refused, request, holdings }),
            ));
        }
        Ask::Resources(resources::Ask::ProcedureHoldings { task, step, members }) => {
            let (holdings, refused) = batch_holdings(domain, env, connector, task, &members);
            domain.work.push(Work::Connector(
                Connector(connector),
                Up::Resources(resources::Up::ProcedureHoldings { refused, task, step, holdings }),
            ));
            if domain.forge.status_owner(task).is_some() {
                // The holdings answer drains the status delegate's make before its account is read.
                domain.work.push(Work::StatusAccount { procedure: task });
            }
        }
        Ask::Resources(resources::Ask::RepairRefused { repair, number, why }) => {
            if connector == domain.config.forge_connector
                && let Some((forming, reason)) = &mut domain.forge_forming
                && *forming == number
            {
                *reason = Some(why);
            }
            if connector == domain.config.forge_connector
                && let Some(task) = repair
                && let Some((party, project)) = domain.forge.status_owner(task)
            {
                domain.work.push(Work::Forge(forge::Event::StatusRefused { project, party }));
            }
            if connector == domain.config.forge_connector
                && let Some(repair) = repair
                && let Some(owner) = domain.forge.queue_repair_owner(repair)
            {
                forge_route::hold_reason(domain, owner, b"landing branch repair admission was refused");
            }
        }
        Ask::Resources(resources::Ask::DelegateRefused { task }) => {
            if connector == domain.config.forge_connector
                && let Some(task) = task
                && let Some(row) = domain.forge.change(task)
                && let Some((child, _)) = row.delegate
            {
                domain.work.push(Work::Forge(forge::Event::DelegateRefused { task, child }));
                forge_route::hold_reason(
                    domain,
                    task,
                    b"its delegate admission was refused before work could be accepted",
                );
            }
        }
        Ask::Projections(projections::Ask::Project { feed }) => {
            if connector == domain.config.forge_connector {
                domain.work.push(Work::ProjectGoal(feed));
            } else if feed.closing {
                domain.work.push(Work::Connector(
                    Connector(connector),
                    Up::Projections(projections::Up::Settled { goal: feed.goal.number }),
                ));
            }
        }
        Ask::Topics(topics::Ask::Subscribed { request, .. }) => {
            if connector == domain.config.forge_connector
                && let Some(pending) = domain.forge_subscribing.remove(&request)
            {
                let names = forge_route::watch_names(domain, &env.limits, &pending)
                    .expect("connector names preflighted at subscription");
                let owner = pending.task;
                domain.work.push(Work::Forge(forge::Event::Subscribe { subscription: pending }));
                domain.work.push(Work::Forge(forge::Event::Names { task: owner, resources: names }));
            }
        }
        Ask::Topics(topics::Ask::Unsubscribed { request, .. }) => {
            if connector == domain.config.forge_connector
                && let Some((owner, topic)) = domain.forge_unsubscribing.remove(&request)
            {
                domain.work.push(Work::Forge(forge::Event::Unsubscribe { task: owner, topic }));
            }
        }
        Ask::Topics(topics::Ask::Refused { request }) => {
            if connector == domain.config.forge_connector {
                drop(domain.forge_subscribing.remove(&request));
                drop(domain.forge_unsubscribing.remove(&request));
            }
        }
        Ask::Requests(requests::Ask::Drop { owner }) => {
            super::applications::drop(domain, owner);
        }
        Ask::Requests(requests::Ask::Decide { request, token, owner, retention, role, person, .. }) => {
            super::applications::admitted(domain, request, token, owner, retention, role, person, env, decision);
        }
        Ask::Lifecycle(lifecycle::Ask::ReleaseProject { project }) if connector != domain.config.forge_connector => {
            domain.work.push(Work::Connector(Connector(connector), Up::Lifecycle(lifecycle::Up::Released { project })));
        }
        Ask::Resources(resources::Ask::Unname { task }) if connector == domain.config.forge_connector => {
            forge_route::drop_erased(domain, env, decision, task);
            forward(domain, ask);
        }
        Ask::Effects(
            ask @ (effects::Ask::Describe { .. }
            | effects::Ask::DescribeProposal { .. }
            | effects::Ask::Keep { .. }
            | effects::Ask::KeepProposal { .. }
            | effects::Ask::Drop { .. }
            | effects::Ask::Judge { .. }),
        ) => {
            forge_route::effect_ask(domain, env, decision, connector, ask);
        }
        Ask::Parts(ask @ parts::Ask::Public { .. }) => super::public::ask(domain, env, connector, ask),
        Ask::Settings(settings::Ask::Judge { owner, revision, proposal, items }) => {
            if connector == domain.config.forge_connector {
                let mut local = true;
                for item in &items {
                    local &= item.edit == settings::Edit::Remove && item.kind != 12 && item.kind != 13;
                }
                let asked = settings::Ask::Judge { owner, revision, proposal, items };
                if local {
                    forward(domain, Ask::Settings(asked));
                } else {
                    super::kinds::ask(domain, env, decision, asked);
                }
            }
        }
        Ask::Settings(
            ask @ (settings::Ask::Keep { .. } | settings::Ask::KeepProposal { .. } | settings::Ask::Restore { .. }),
        ) => {
            if connector == domain.config.forge_connector {
                super::kinds::ask(domain, env, decision, ask);
            }
        }
        Ask::Lifecycle(lifecycle::Ask::Restore) => {
            assert_eq!(connector, domain.config.forge_connector, "the configured restore connector");
            crate::store_route::load_forge(domain, env, decision, None);
        }
        ask @ (Ask::Lifecycle(lifecycle::Ask::CatchUp | lifecycle::Ask::ReleaseProject { .. })
        | Ask::Resources(resources::Ask::Hold { .. } | resources::Ask::Unname { .. })
        | Ask::Effects(effects::Ask::Resolve { .. } | effects::Ask::DropProposal { .. })
        | Ask::Topics(topics::Ask::Ended { .. })
        | Ask::Parts(parts::Ask::DropDocument { .. })
        | Ask::Procedures(procedures::Ask::WaitsRefused { .. })
        | Ask::Projections(projections::Ask::Forget { .. })
        | Ask::Settings(
            settings::Ask::ForgetReplaced { .. }
            | settings::Ask::DropProposal { .. }
            | settings::Ask::SaveDefaults { .. },
        )
        | Ask::Requests(requests::Ask::PartAnswered { .. })) => {
            if connector == domain.config.forge_connector {
                forward(domain, ask);
            }
        }
    }
}

type BatchHoldings = Box<[Box<[resources::Holding]>]>;

fn batch_holdings(
    domain: &mut Domain,
    env: &Env<Limits>,
    connector: u16,
    parent: u64,
    members: &[resources::HoldingsNeed],
) -> (Option<BatchHoldings>, Option<resources::HoldingsRefusal>) {
    if connector != domain.config.forge_connector {
        return (None, None);
    }
    let mut collected = List::with_capacity(u32::try_from(members.len()).expect("bounded delegate batch"));
    for member in members {
        match forge_route::task_holdings(
            domain,
            env,
            member.project,
            member.root,
            member.number,
            member.executor,
            &member.spec,
            Some(parent),
            member.tracked,
            &member.symbols,
        ) {
            Ok(holdings) => collected.push(holdings).expect("one per member"),
            Err(refused) => return (None, refused),
        }
    }
    (Some(collected.into_boxed()), None)
}

pub(super) fn forward(domain: &mut Domain, ask: Ask) {
    domain.work.push(Work::Forge(forge::Event::Ask(ask)));
}

/// The engine marks a held output in the decision; the journal releases it after commit.
pub(super) fn connector_released(
    domain: &Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    Connector(connector): Connector,
    released: Released,
) {
    assert_eq!(connector, domain.config.forge_connector, "numbered forge release");
    match released {
        Released::Make { entry } => emit(decision, &env.limits, Delivery::ForgeCommitted { entry }),
        Released::Answered { .. } => unreachable!("this root sends its own acts with reply tokens"),
    }
}

/// Application routes that are not connector capabilities.
#[expect(clippy::too_many_lines, reason = "one dispatcher for the remaining application hand-offs")]
pub(super) fn application(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    connector: u16,
    ask: core::Ask,
) {
    let request = if let core::Ask::Workers(workers) = ask {
        assert!(
            super::now(domain, crate::Request::Worker(crate::WorkerRequest::Settings(workers))),
            "worker table door reserved"
        );
        None
    } else if let core::Ask::Catalogue { models } = ask {
        domain.catalogue_pending = true;
        let names = core::settings::model_names(&core::settings::catalogue::rows(&models));
        assert!(
            super::now(domain, crate::Request::Store(crate::StoreRequest::Catalogue { names })),
            "catalogue store door reserved"
        );
        assert!(
            super::now(domain, crate::Request::Party(crate::PartyRequest::Catalogue { models })),
            "catalogue clients door reserved"
        );
        None
    } else if let core::Ask::Providers(providers) = ask {
        emit(decision, &env.limits, Delivery::Providers(providers));
        None
    } else if let core::Ask::Gather { section, budget } = ask {
        Some(brief::GatherRequest::Gather { connector, section, budget })
    } else if let core::Ask::CutTo { section, size } = ask {
        Some(brief::GatherRequest::CutTo { connector, section, size })
    } else if let core::Ask::Drop { section } = ask {
        Some(brief::GatherRequest::Drop { connector, section })
    } else if let core::Ask::ClaimDone { task, attempt, kinds, version } = ask {
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
                let jig_domain_hub_link::Request::Core(core::Event::Fleet(mut event)) = out.pop().expect("link start")
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
    } else if let core::Ask::JobClaimed { task, attempt, kind, spec, deadline, result, grants, workspace } = ask {
        if connector == domain.config.forge_connector {
            forge_route::job_history::claimed(domain, task, attempt, kind.0);
            let crate::boundary::PreparedWorkspace { workspace: prepared, claim_names } =
                domain.run_workspaces.remove(&task).expect("prepared job workspace").into_prepared();
            drop(super::now::claim_workspace(domain, env, task, &prepared, claim_names));
            let (writes, designs, holders) = forge_route::claimed_writes(domain, env, task, &prepared.workspace)
                .expect("the admitted job retains its bounded held forge writes");
            if !writes.is_empty() || !designs.is_empty() {
                domain.work.push(Work::Forge(forge::Event::Claim { task, attempt, writes, designs, holders }));
            }
            if workspace.is_some() {
                let kept = crate::boundary::JobWorkspace { attempt, workspace: prepared.workspace, offered: false };
                assert!(domain.job_workspaces.insert(task, kept).is_ok(), "one job assignment per task");
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
                let jig_domain_hub_link::Request::Core(core::Event::Fleet(event)) = out.pop().expect("link job start")
                else {
                    unreachable!("a job start routes to the fleet")
                };
                emit(decision, &env.limits, Delivery::Fleet(event));
            }
        }
        None
    } else {
        unreachable!("the engine separates connector asks from application asks: {ask:?}")
    };
    if let Some(request) = request {
        let mut child = Queue::with_capacity(1);
        child.push(request);
        brief_outputs(domain, env, decision, &mut child);
    }
}

#[cfg(test)]
pub(crate) fn ask(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, connector: u16, ask: Ask) {
    let mut requests = Queue::with_capacity(1);
    requests.push(core::Request::Ask { connector, ask: core::Ask::Connector(ask) });
    super::route_core_requests(domain, env, decision, requests);
}

fn step_made(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    task: u64,
    step: u64,
    child: Option<u64>,
) {
    forge_route::follow_route::made(domain, task, step, child);
    forge_route::closer_made(domain, task, step, child);
    if domain.forge.tier_task(task).is_some()
        && let Some(row) = domain.engine.core().tasks.task(task)
    {
        domain.work.push(Work::Forge(forge::Event::TierWords { task, words: row.spec.words.clone() }));
    }
    if let Some(pending) = domain.forge_delegating.remove(&(task, step)) {
        if let Some((kind, close_at)) = pending.delegate
            && let Some(child) = child
        {
            // Jobs prepare at activation, after their parent row names them.
            super::forge_event(domain, env, decision, forge::Event::Delegated { task, child, kind, close_at });
        }
        if let Some(relayed) = pending.relayed {
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
    let task = domain.engine.core().tasks.task(task)?;
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
