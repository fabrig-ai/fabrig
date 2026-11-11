//! Deployment-funded landing tasks and their authorized steps (domain/landing.md, 4.2, 11).
use super::{
    Box, Domain, EffectDescription, EffectForm, Env, Family, LANDING_CODE, Limits, List, ProcedureStep, Recovery,
    StepEnd, Work, append, authority, effect_access, forge, forge_change, landing, resource_name, tasks,
};

fn changes(domain: &Domain, env: &Env<Limits>, task: u64) -> Box<[u64]> {
    let Some(change) = domain.forge.change(task) else { return Box::new([task]) };
    let Some(repository) = domain.forge.repository(change.repository) else { return Box::new([task]) };
    let mut graph = List::with_capacity(env.limits.forge.changes);
    for repository in domain.forge.writable_repositories(repository.project) {
        for change in domain.forge.changes_for(repository.provider, env.limits.forge.changes) {
            if let Some(row) = domain.core.tasks.task(change) {
                graph
                    .push(forge::ChangeDependencies { change, pending: row.waiting_on.clone() })
                    .expect("bounded changes");
            }
        }
    }
    domain.forge.joined_changes(repository.project, task, graph.as_slice())
}

/// Joined dependencies permit the forming decision to refuse the whole unit.
/// Jig still orders successful task results (domain/landing.md, 4.1).
pub(super) fn kept(domain: &Domain, env: &Env<Limits>, task: u64) -> bool {
    let Some(row) = domain.core.tasks.task(task) else { return false };
    if row.waiting_on.is_empty() {
        return false;
    }
    let joined = changes(domain, env, task);
    let formed = match domain.forge.change(task) {
        Some(change) => change.change.landing.is_some(),
        None => false,
    };
    for dependency in &row.waiting_on {
        if joined.contains(dependency) || formed {
            continue;
        }
        return true;
    }
    false
}

/// Pending dependencies in the task's dependency order (domain/forge/change/questions.md, 1).
pub(super) fn pending(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    repository: skein_forge::api::Repository,
    base: &[u8],
) -> Box<[forge_change::Pending]> {
    let Some(row) = domain.core.tasks.task(task) else { return Box::new([]) };
    let mut pending = List::with_capacity(env.limits.tasks.dependencies);
    for dependency in &row.dependencies {
        if !row.waiting_on.contains(dependency) {
            continue;
        }
        let Some(upstream_task) = domain.core.tasks.task(*dependency) else { continue };
        let kind = match domain.forge.change(*dependency) {
            Some(upstream) if upstream.repository != repository => forge_change::PendingKind::Elsewhere,
            Some(upstream) if upstream.base.as_ref() == base => {
                let head = match super::branch_parts(&upstream.branch, env.limits.forge.authority_segments) {
                    Some(parts) => domain.forge.branch_head(&forge::Name {
                        forge: upstream.repository.forge,
                        repository: upstream.repository.repository,
                        what: forge::What::Branch(parts),
                    }),
                    None => None,
                };
                forge_change::PendingKind::Change { head, kept: !upstream_task.waiting_on.is_empty() }
            }
            Some(_) => forge_change::PendingKind::Other,
            None => match upstream_task.executor {
                tasks::Executor::Procedure { connector, code: super::CHANGE_CODE }
                    if connector == domain.config.forge_connector =>
                {
                    // A sibling made in the same batch may not have registered its forge row yet.
                    forge_change::PendingKind::Change { head: None, kept: !upstream_task.waiting_on.is_empty() }
                }
                tasks::Executor::Agent { .. }
                | tasks::Executor::Procedure { .. }
                | tasks::Executor::Person(_)
                | tasks::Executor::Job { .. } => forge_change::PendingKind::Other,
            },
        };
        pending.push(forge_change::Pending { task: *dependency, kind }).expect("bounded task dependencies");
    }
    pending.into_boxed()
}

pub(super) fn landed_step_due(domain: &Domain, task: u64) -> bool {
    let Some(change) = domain.forge.change(task) else { return false };
    if let forge_change::State::Ready { .. } = change.change.state
        && let Some(task) = change.change.landing
        && let Some(row) = domain.forge.landing(task)
    {
        for member in &row.landing.members {
            if member.change == change.task {
                return member.landed.is_some();
            }
        }
    }
    false
}

fn member(domain: &Domain, env: &Env<Limits>, task: u64) -> Option<landing::Member> {
    let row = domain.forge.change(task)?;
    let project = domain.forge.repository(row.repository)?.project;
    let mut gates = List::with_capacity(env.limits.forge.gates);
    if let Some(config) = domain.config.landings.get(&project) {
        for repository in &config.repositories {
            if repository.repository == row.repository {
                for _ in &repository.gates {
                    gates
                        .push(landing::GateProof {
                            proof: landing::Proof::Unkeyed,
                            posted: landing::Posted::Nothing,
                            faults: 0,
                        })
                        .expect("configured gates");
                }
            }
        }
    }
    Some(landing::Member {
        change: task,
        repository: landing::Repository { forge: row.repository.forge, repository: row.repository.repository },
        // A producing member has no pull yet; the top fills it before reading.
        pull: row.pull.unwrap_or(0),
        head: None,
        base: None,
        candidate: None,
        gates: gates.into_boxed(),
        asked: None,
        landed: None,
    })
}

/// Judge every derived train at quiescence, after readiness, ending, priority and
/// restart work has settled. One queued decision is committed before judging again
/// so neither a train nor the live-row bound can be reserved twice (landing.md, 9).
pub(crate) fn form_landings(domain: &mut Domain, env: &Env<Limits>) -> bool {
    for (repository, branch) in domain.forge.waiting_trains() {
        if form_train(domain, env, repository, &branch) {
            return true;
        }
    }
    false
}

#[cfg(test)]
pub(super) fn start_landing(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    let Some(change) = domain.forge.change(task) else { return };
    let repository = change.repository;
    let branch = change.base.clone();
    form_train(domain, env, repository, &branch);
}

fn form_train(domain: &mut Domain, env: &Env<Limits>, repository: skein_forge::api::Repository, branch: &[u8]) -> bool {
    let mut excluded = List::with_capacity(env.limits.forge.changes);
    for candidate in domain.forge.waiting_changes(repository, branch) {
        let Some(task) = domain.core.tasks.task(candidate.task) else { continue };
        let held = match task.phase {
            tasks::Phase::Active(_) | tasks::Phase::Waiting => false,
            tasks::Phase::Held { .. } | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => true,
        };
        if held || kept(domain, env, candidate.task) {
            excluded.push(candidate.task).expect("bounded changes");
        }
    }
    let Some(task) = domain.forge.first_waiting(repository, branch, excluded.as_slice(), env.wall) else {
        return false;
    };
    let change = domain.forge.change(task).expect("waiting change");
    let project = domain.forge.repository(change.repository).expect("train repository").project;
    let joined = changes(domain, env, task);
    if joined.len() > 1 {
        let words = domain.forge.unit_words(&joined);
        super::holds::hold(domain, task, &words);
        return domain.core.tasks.task(task).is_some();
    }
    create_landing(domain, env, task, project, branch)
}

fn create_landing(domain: &mut Domain, env: &Env<Limits>, task: u64, project: u32, branch: &[u8]) -> bool {
    let change = domain.forge.change(task).expect("waiting change");
    let mut words = List::with_capacity(env.limits.tasks.spec_bytes.saturating_sub(32));
    append(&mut words, b"Land ");
    let room = skein_lib::usize_of(words.capacity().saturating_sub(words.len()));
    let title = temper_engine_domain_forge_issues::text::fit(&change.title, room, "…".as_bytes());
    append(&mut words, &title);
    let mut members = List::with_capacity(env.limits.forge.landing_members);
    let mut asks = List::with_capacity(env.limits.forge.landing_members);
    let Some(member) = member(domain, env, task) else { return false };
    members.push(member).expect("one admitted member");
    asks.push(None).expect("one admitted member");
    let Some(seed) = domain.core.procedure_seed(project) else { return false };
    let number = seed.number();
    domain.work.push(Work::Forge(forge::Event::Landing {
        row: forge::LandingRow {
            landing: landing::Landing {
                task: number,
                project,
                phase: landing::Phase::Waiting,
                preparation: 0,
                stale_proofs: 0,
                branch: Box::from(branch),
                members: members.into_boxed(),
            },
            asks: asks.into_boxed(),
            entries: Box::new([]),
        },
    }));
    domain.work.push(Work::Core(jig_domain_core::Event::ConnectorRepair {
        seed: Box::new(seed),
        executor: tasks::Executor::Procedure { connector: domain.config.forge_connector, code: LANDING_CODE },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: words.into_boxed(),
            parameters: Box::new([
                tasks::Parameter::Number { name: 1, value: u64::from(project) },
                tasks::Parameter::Number { name: 2, value: task },
            ]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        holdings: Box::new([]),
    }));
    true
}

#[expect(clippy::match_like_matches_macro, reason = "the foundation requires explicit matching")]
pub(crate) fn start_landing_step(domain: &mut Domain, _env: &Env<Limits>, context: &tasks::RunContext) {
    let task = context.task;
    let Some(row) = domain.forge.landing(task) else { return };
    let released = if let landing::Phase::Held { .. } = row.landing.phase { true } else { false };
    let Some(entry) = crate::fresh(&mut domain.core.counters, Family::ConnectorRow) else { return };
    domain.forge.offered_landing(task);
    domain.work.push(Work::Forge(forge::Event::StepLanding {
        task,
        entry,
        heard: forge::LandingHeard { released, cancelled: false, kept: Box::new([]) },
    }));
}

pub(super) fn describe_landing_effect(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
) -> Option<EffectDescription> {
    let (provider, what, described) = domain.forge.describe_landing_effect(task, entry)?;
    let repository = domain.forge.repository(provider)?;
    let context = domain.core.tasks.delegation(task)?;
    if repository.project != context.project {
        return None;
    }
    Some(EffectDescription {
        waits_for_dependencies: false,
        connector: domain.config.forge_connector,
        purpose: entry,
        effect: authority::Effect {
            connector: domain.config.forge_connector,
            kind: described.kind,
            name: resource_name(repository, &what, env.limits.authority.segments)?,
            state: described.state,
            price: None,
            access: effect_access(repository, &what, described.kind),
            additional: Box::new([]),
            guards: Box::new([]),
        },
        form: match described.form {
            forge::EffectForm::Creation => EffectForm::Creation,
            forge::EffectForm::Transition => EffectForm::Transition,
            forge::EffectForm::Set => EffectForm::Set,
        },
        recovery: match described.recovery {
            forge::Recovery::Keyed => Recovery::Keyed,
            forge::Recovery::Conditional => Recovery::Conditional,
            forge::Recovery::Idempotent => Recovery::Idempotent,
            forge::Recovery::Unrecoverable => Recovery::Unrecoverable,
        },
    })
}

pub(super) fn landing_decision(domain: &mut Domain, env: &Env<Limits>, task: u64, choice: landing::Decision) {
    let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return };
    if connector != domain.config.forge_connector || code != LANDING_CODE {
        return;
    }
    domain.forge.offered_landing(task);
    let end = match choice {
        landing::Decision::Wait { until } => {
            domain.forge.wait_landing(&super::super::environment_forge(env), task, until);
            StepEnd::Wait
        }
        landing::Decision::Finish => {
            StepEnd::Result(tasks::TaskResult::Report { words: landing_result_words(domain, env, task) })
        }
        landing::Decision::Cancel => {
            domain.work.push(Work::Tasks(tasks::Event::Cancel {
                reply_to: super::super::internal(u64::MAX),
                by: tasks::Party::Deployment { project: domain.forge.landing(task).expect("landing").landing.project },
                operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
            }));
            StepEnd::Wait
        }
        landing::Decision::Hold(landing::Hold::Faults { member, gate }) => {
            let row = domain.forge.landing(task).expect("landing");
            let member = row.landing.members.get(usize::from(member)).expect("member");
            let provider = skein_forge::api::Repository {
                forge: member.repository.forge,
                repository: member.repository.repository,
            };
            let mut words = List::with_capacity(env.limits.tasks.message_bytes);
            append(&mut words, b"gate ");
            if let Some(config) = domain.config.landings.get(&row.landing.project) {
                for repository in &config.repositories {
                    if repository.repository == provider
                        && let Some(check) = repository.gates.get(usize::from(gate))
                    {
                        append(&mut words, &check.name);
                    }
                }
            }
            append(&mut words, b" of ");
            if let Some(repository) = domain.forge.repository(provider) {
                append(&mut words, &repository.owner);
                append(&mut words, b"/");
                append(&mut words, &repository.name);
            }
            append(&mut words, b" failed on its runner past its retries");
            let decision = ProcedureStep {
                messages: Box::new([jig_domain_core::StepMessage {
                    target: task,
                    kind: tasks::MessageKind::Words,
                    words: words.into_boxed(),
                }]),
                ..ProcedureStep::new(StepEnd::Hold(tasks::Hold::Procedure))
            };
            domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep {
                task,
                step,
                connector,
                code,
                decision,
            }));
            return;
        }
        landing::Decision::None
        | landing::Decision::Prepare { .. }
        | landing::Decision::Key { .. }
        | landing::Decision::Dispatch { .. }
        | landing::Decision::Retry { .. }
        | landing::Decision::Post { .. }
        | landing::Decision::Repair { .. }
        | landing::Decision::Proved
        | landing::Decision::Merge { .. }
        | landing::Decision::Stale => StepEnd::Wait,
    };
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep {
        task,
        step,
        connector,
        code,
        decision: ProcedureStep::new(end),
    }));
    if choice == landing::Decision::Finish || choice == landing::Decision::Cancel {
        wake_ready(domain, env, task);
    } else if let landing::Decision::Wait { .. } = choice {
    } else {
        domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task }));
    }
}

fn wake_ready(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    let Some(row) = domain.forge.landing(task) else { return };
    let project = row.landing.project;
    for repository in domain.forge.writable_repositories(project) {
        for change in domain.forge.changes_for(repository.provider, env.limits.forge.changes) {
            if let Some(row) = domain.forge.change(change)
                && row.change.landing.is_none()
                && let forge_change::State::Ready { .. } = row.change.state
            {
                domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task: change }));
            }
        }
    }
}
/// Aggregate report contract: owner/name, a space, then the landed commit in hex, one line each.
fn landing_result_words(domain: &Domain, env: &Env<Limits>, task: u64) -> Box<[u8]> {
    let mut words = List::with_capacity(env.limits.tasks.result_bytes);
    for member in &domain.forge.landing(task).expect("finished landing").landing.members {
        let provider =
            skein_forge::api::Repository { forge: member.repository.forge, repository: member.repository.repository };
        if let Some(repository) = domain.forge.repository(provider) {
            append(&mut words, &repository.owner);
            append(&mut words, b"/");
            append(&mut words, &repository.name);
        }
        append(&mut words, b" ");
        let commit = member.landed.expect("finished member landed");
        let width = if commit[20..] == [0; 12] { 20 } else { 32 };
        for byte in commit.get(..width).expect("commit width") {
            append(&mut words, &forge::hex(*byte));
        }
        append(&mut words, b"\n");
    }
    words.into_boxed()
}
