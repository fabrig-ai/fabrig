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
            step_made(domain, task, step, first);
            None
        }
        core::Ask::StepRefused { task, step, why } => {
            let delegation = domain.forge_delegating.remove(&(task, step));
            if let core::StepRefusal::Denied { findings, .. } = &why {
                if connector == domain.config.forge_connector
                    && let Some(kind) = delegation
                {
                    domain.work.push(Work::Forge(forge::Event::DelegateAdmissionRefused { task, kind }));
                }
                forge_route::refused_step(domain, env, task, step, findings);
            }
            if let core::StepRefusal::Refused(problem) = why {
                // Jig's domain/connectors.md, 6.2: step again on refusal. A kept change waits
                // without a deadline; its dependency's Met wakes it (jig's domain/tasks.md, 5.7).
                if connector == domain.config.forge_connector
                    && domain.core.tasks.waiting_effect_problem(task).as_ref() == Some(&problem)
                    && let Some((owner, code @ 2, current)) = domain.core.tasks.procedure_due(task)
                    && owner == connector
                    && current == step
                {
                    domain.work.push(Work::Core(core::Event::ProcedureStep {
                        task,
                        step,
                        connector,
                        code,
                        decision: forge_route::append_change_replies(
                            domain,
                            env,
                            task,
                            core::ProcedureStep::new(core::StepEnd::Wait),
                        ),
                    }));
                }
                if domain.facts.try_push(crate::Fact::StepRefused { task, step, problem }).is_err() {
                    domain.facts_lost = domain.facts_lost.saturating_add(1);
                }
            }
            None
        }
        core::Ask::Document { watcher, scope, subject, .. } => {
            super::panels::document(domain, env, decision, connector, watcher, scope, subject);
            None
        }
        core::Ask::Read { reply, .. } => {
            emit(
                decision,
                &env.limits,
                Delivery::Read { reply, answer: core::reads::Answer::None(core::reads::Why::Unknown) },
            );
            None
        }
        core::Ask::DropDocument { .. } => None,
        core::Ask::DropApplication { owner } => {
            super::applications::drop(domain, owner);
            None
        }
        core::Ask::Application { request, token, owner, retention, role, .. } => {
            super::applications::admitted(domain, request, token, owner, retention, role, env);
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
        core::Ask::Adopt { request, .. } => {
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
                let (writes, holders) = forge_route::claimed_writes(domain, env, task)
                    .expect("the admitted assignment retains its bounded held forge writes");
                if !writes.is_empty() {
                    domain.work.push(Work::Forge(forge::Event::Claim { task, attempt, writes, holders }));
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
                let mut out = Queue::uncapped();
                jig_domain_hub_link::step(
                    &mut domain.hub_link,
                    env.now,
                    jig_domain_hub_link::Event::StartJob {
                        job: jig_domain_hub_link::JobStart {
                            task,
                            attempt,
                            workstream: task,
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

fn step_made(domain: &mut Domain, task: u64, step: u64, child: Option<u64>) {
    if let Some(kind) = domain.forge_delegating.remove(&(task, step))
        && let Some(child) = child
    {
        domain.work.push(Work::Forge(forge::Event::Delegated { task, child, kind }));
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
