//! Deployment-funded landing tasks and their authorized steps (domain/landing.md, 4.2, 11).
//! Landing targets use the adoption (temper/docs/design/domain/forge/adoption.md, 12).
//! Task-first admission and refused trains follow domain/landing-forming.md, 2 and 3.
use super::{
    Box, Domain, EffectDescription, Env, Family, LANDING_CODE, Limits, List, ProcedureStep, Queue, StepEnd, Work,
    append, authority, effect_access, forge, forge_change, landing, resource_name, tasks,
};
use forge::LandingHeadTrailers as HeadTrailers;
use temper_engine_domain_translate::prepare;

fn changes(domain: &Domain, env: &Env<Limits>, task: u64) -> Box<[u64]> {
    let Some(change) = domain.forge.change(task) else { return Box::new([task]) };
    let Some(repository) = domain.forge.repository(change.repository) else { return Box::new([task]) };
    let graph = dependencies(domain, env, repository.project);
    domain.forge.joined_changes(repository.project, task, &graph)
}

fn dependencies(domain: &Domain, env: &Env<Limits>, project: u32) -> Box<[forge::ChangeDependencies]> {
    domain.forge.forming_dependencies(&domain.engine.core().tasks, project, env.limits.forge.changes)
}

/// Joined dependencies permit the forming decision to refuse the whole unit.
/// Jig still orders successful task results (domain/landing.md, 4.1).
pub(super) fn kept(domain: &Domain, env: &Env<Limits>, task: u64) -> bool {
    let Some(row) = domain.engine.core().tasks.task(task) else { return false };
    if row.waiting_on.is_empty() {
        return false;
    }
    let joined = changes(domain, env, task);
    let formed = domain.forge.landing_of(task).is_some();
    for dependency in &row.waiting_on {
        if joined.contains(dependency) || formed {
            continue;
        }
        return true;
    }
    false
}

/// A saved task lifecycle can end a candidate's wait or release its parent
/// by delivering the parent's final gate result.
pub(super) fn cohort_saved(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: &tasks::TaskRecord,
    out: &mut Queue<forge::Request>,
) {
    if let Some(change) = domain.forge.change(task.number)
        && change.settling
        && (!live(task) || kept(domain, env, task.number))
    {
        domain.forge.set_settling(task.number, false, out);
    }
    if let tasks::Phase::Ended(_) = task.phase
        && let tasks::Party::Task(owner) = task.requester
        && let Some(row) = domain.forge.change(owner)
        && let Some((child, forge_change::Delegate::Gate { number, head })) = row.delegate
        && child == task.number
    {
        release(domain, env, owner, Some((number, head)), out);
    }
}

/// The live phases that withdraw a landing's wake (domain/landing-liveness.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Unavailable {
    Held,
    Closing,
}

impl Unavailable {
    pub(crate) const fn of(phase: &tasks::Phase) -> Option<Self> {
        match phase {
            tasks::Phase::Held { .. } => Some(Self::Held),
            tasks::Phase::Closing(_) => Some(Self::Closing),
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Ended(_) => None,
        }
    }
}

pub(super) fn wake_member(domain: &mut Domain, change: u64) {
    if let Some(task) = domain.forge.landing_of(change) {
        domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task }));
    }
}

pub(super) fn lifecycle_saved(domain: &mut Domain, task: &tasks::TaskRecord) {
    let ended = match task.phase {
        tasks::Phase::Ended(_) => true,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } | tasks::Phase::Closing(_) => false,
    };
    let changed = match Unavailable::of(&task.phase) {
        Some(phase) => {
            let previous = domain.landing_unavailable.insert(task.number, phase).expect("one phase per live task");
            previous != Some(phase)
        }
        None => {
            domain.landing_unavailable.remove(&task.number);
            ended
        }
    };
    if !changed {
        return;
    }
    wake_member(domain, task.number);
    if task.executor == (tasks::Executor::Job { kind: forge::PREPARE_KIND })
        && let tasks::Party::Task(owner) = task.requester
        && domain.forge.landing(owner).is_some()
    {
        let affected = match prepare_child(domain, owner) {
            Some(child) => child.number == task.number,
            // Jig removes an ended delegate before routing the terminal save.
            None => ended,
        };
        if affected {
            domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task: owner }));
        }
    }
}

fn live(task: &tasks::TaskRecord) -> bool {
    match task.phase {
        tasks::Phase::Waiting | tasks::Phase::Active(_) => task.placed.is_none(),
        tasks::Phase::Held { .. } | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
    }
}

pub(super) fn release(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    gate: Option<(u64, skein_forge::api::Commit)>,
    out: &mut Queue<forge::Request>,
) {
    let Some(row) = domain.forge.change(task) else { return };
    let Some(owner) = domain.engine.core().tasks.task(task) else { return };
    if live(owner)
        && forge::gates_settled(&domain.forge, row, gate, env.limits.forge.change_policy.gates)
        && !kept(domain, env, task)
    {
        domain.forge.set_settling(task, true, out);
    }
}

/// Pending dependencies in the task's dependency order (domain/forge/change/questions.md, 1).
pub(super) fn pending(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    repository: skein_forge::api::Repository,
    base: &[u8],
) -> Box<[forge_change::Pending]> {
    let Some(row) = domain.engine.core().tasks.task(task) else { return Box::new([]) };
    let mut pending = List::with_capacity(env.limits.tasks.dependencies);
    for dependency in &row.dependencies {
        if !row.waiting_on.contains(dependency) {
            continue;
        }
        let Some(upstream_task) = domain.engine.core().tasks.task(*dependency) else { continue };
        let kind = match domain.forge.change(*dependency) {
            Some(upstream) if upstream.repository != repository => forge_change::PendingKind::Elsewhere,
            Some(upstream) if upstream.base.as_ref() == base => forge_change::PendingKind::Change {
                checked: upstream.change.last_checked,
                kept: !upstream_task.waiting_on.is_empty(),
            },
            Some(_) => forge_change::PendingKind::Other,
            None => match upstream_task.executor {
                tasks::Executor::Procedure { connector, code: super::CHANGE_CODE }
                    if connector == domain.config.forge_connector =>
                {
                    // A sibling made in the same batch may not have registered its forge row yet.
                    forge_change::PendingKind::Change { checked: None, kept: !upstream_task.waiting_on.is_empty() }
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

/// Judge every derived train at quiescence, after readiness, ending, priority and
/// restart work has settled. One queued decision is committed before judging again
/// so neither a train nor the live-row bound can be reserved twice (domain/landing-turn.md, 2).
pub(crate) fn form_landings(domain: &mut Domain, env: &Env<Limits>) -> bool {
    let trains = domain.forge.waiting_trains();
    if let Some((repository, branch)) = domain.forge.empty_refused_train(&trains) {
        domain.work.push(Work::Forge(forge::Event::TrainRefusal { repository, branch, cause: None }));
        return true;
    }
    if !domain.forge.landing_held() {
        for (repository, branch) in &trains {
            if domain.forge.repository_fresh(*repository) && form_train(domain, env, *repository, branch) {
                return true;
            }
        }
    }
    domain.forming_judged = Some(env.wall);
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
    if !domain.forge.settling_changes(repository, branch, 1).is_empty()
        || domain.forge.probing_train(repository, branch)
    {
        return false;
    }
    if let Some((refusal, _)) = domain.forge.forming_refusal(repository, branch) {
        match refusal.cause {
            // A saturated retry cannot establish that its queue window has passed.
            forge::FormingRefusal::Jig { retry, .. } if env.wall < retry || retry.as_nanos() == u64::MAX => {
                return false;
            }
            forge::FormingRefusal::Jig { .. } | forge::FormingRefusal::Landings => {}
        }
    }
    let mut excluded = List::with_capacity(env.limits.forge.changes);
    let Some(repo) = domain.forge.repository(repository) else { return false };
    let project = repo.project;
    for (provider, base) in domain.forge.waiting_trains() {
        for candidate in domain.forge.waiting_changes(provider, &base) {
            let Some(task) = domain.engine.core().tasks.task(candidate.task) else { continue };
            let held = match task.phase {
                tasks::Phase::Active(_) | tasks::Phase::Waiting => false,
                tasks::Phase::Held { .. } | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => true,
            };
            if (held || kept(domain, env, candidate.task) || super::change_delegate_live(domain, candidate.task))
                && !excluded.as_slice().contains(&candidate.task)
            {
                excluded.push(candidate.task).expect("bounded changes");
            }
        }
    }
    let graph = dependencies(domain, env, project);
    let units = domain.forge.waiting_units(repository, branch, excluded.as_slice(), &graph, env.wall);
    // The outer forming loop checked this train's repository. A dependency unit
    // can also judge other repositories, including a refusal to join them.
    for unit in &units {
        for task in &unit.changes {
            if let Some(row) = domain.forge.change(*task)
                && row.repository != repository
                && !domain.forge.repository_fresh(row.repository)
            {
                return false;
            }
        }
    }
    let selection = domain.forge.select_waiting(repository, branch, &units);
    // A hold is a committed decision too: judge the remaining train again afterwards.
    if let Some(index) = selection.hold.first() {
        let unit = units.get(skein_lib::usize_of(*index)).expect("selected unit");
        let words = domain.forge.unit_words(&unit.changes);
        let first = unit.changes.first().copied().unwrap_or(unit.task);
        super::holds::hold(domain, first, &words);
        return true;
    }
    let mut members = List::with_capacity(env.limits.forge.landing_members);
    let mut changes = List::with_capacity(env.limits.forge.landing_members);
    for (number, index) in selection.take.iter().enumerate() {
        let unit = units.get(skein_lib::usize_of(*index)).expect("selected unit");
        for task in &unit.changes {
            let Some(mut member) = domain.forge.forming_member(*task, env.limits.forge.gates) else { return false };
            member.unit = u16::try_from(number).expect("bounded units");
            members.push(member).expect("selected members");
            changes.push(*task).expect("selected members");
        }
    }
    match domain.forge.landing_admits(project, changes.as_slice()) {
        Ok(()) => create_landing(domain, env, members, project, branch),
        Err(forge::LandingRefusal::Landings) => refuse_table(domain, env, repository, branch, changes.into_boxed()),
        Err(
            forge::LandingRefusal::Way
            | forge::LandingRefusal::Members
            | forge::LandingRefusal::Repository
            | forge::LandingRefusal::Train,
        ) => false,
    }
}

fn create_landing(
    domain: &mut Domain,
    env: &Env<Limits>,
    mut members: List<landing::Member>,
    project: u32,
    branch: &[u8],
) -> bool {
    let Some(first) = members.as_slice().first() else { return false };
    let task = first.change;
    let change = domain.forge.change(task).expect("waiting change");
    let mut words = List::with_capacity(env.limits.tasks.spec_bytes.saturating_sub(32));
    append(&mut words, b"Land ");
    let room = skein_lib::usize_of(words.capacity().saturating_sub(words.len()));
    let title = temper_engine_domain_forge_issues::text::fit(&change.title, room, "…".as_bytes());
    append(&mut words, &title);
    let Some(seed) = domain.engine.core_mut().procedure_seed(project) else { return false };
    let number = seed.number();
    let provider = domain.forge.change(task).expect("member").repository;
    let Some(repository) = domain.forge.repository(provider) else { return false };
    let Some(own_branch) = super::tree_branch(repository, number, b'l', number, None, env.limits.forge.name_bytes)
    else {
        return false;
    };
    let mut holdings = List::with_capacity(env.limits.forge.landing_members);
    let mut names = List::with_capacity(env.limits.forge.landing_members);
    for (index, member) in members.iter_mut().enumerate() {
        let suffix =
            if index == 0 { None } else { Some(u64::try_from(index).expect("member index").saturating_add(1)) };
        let Some(branch) = super::tree_branch(repository, number, b'l', number, suffix, env.limits.forge.name_bytes)
        else {
            return false;
        };
        let Some(what) = super::branch_what(&branch, env.limits.forge.name_bytes) else { return false };
        member.branch = branch;
        let name = forge::Name { forge: provider.forge, repository: provider.repository, what };
        let Some(holding) = super::write_holding(domain.config.forge_connector, &name, &env.limits.tasks) else {
            return false;
        };
        holdings.push(holding).expect("one holding per member");
        names.push(name).expect("one branch per member");
    }
    for name in &names {
        super::report_resource(domain, env, project, name);
    }
    let row = forge::LandingRow {
        read_hold: None,
        gate_hold: None,
        last_event: None,
        turn_since: None,
        waiting: None,
        follows: None,
        flake_reruns: Box::new([]),
        room: None,
        job: None,
        landing: landing::Landing {
            last_left: None,
            searched: None,
            round: landing::Round::new(own_branch.clone()),

            task: number,
            project,
            phase: landing::Phase::Waiting,
            preparation: 0,
            stale_proofs: 0,
            last_prepare: None,
            branch: Box::from(branch),
            members: members.into_boxed(),
        },
        previous_asks: Box::new([]),
        entries: Box::new([]),
    };
    domain.forge_forming = Some((number, None));
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
        holdings: holdings.into_boxed(),
    }));
    domain.work.push(Work::LandingAdmission { row, names: names.into_boxed() });
    true
}

/// Admission precedes activation, so the first step finds its row (landing-forming.md, 2).
pub(crate) fn admission(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut super::Decision,
    row: forge::LandingRow,
    names: &[forge::Name],
) {
    let number = row.landing.task;
    let (forming, why) = domain.forge_forming.take().expect("one forming awaits admission");
    assert_eq!(forming, number, "admission answers its make");
    let first = row.landing.members.first().expect("a landing has members");
    let repository =
        skein_forge::api::Repository { forge: first.repository.forge, repository: first.repository.repository };
    if domain.engine.core().tasks.task(number).is_some() {
        super::super::forge_event(domain, env, decision, forge::Event::Landing { row });
        for name in names {
            super::super::forge_event(
                domain,
                env,
                decision,
                forge::Event::Hold { entry: 0, task: number, resource: name.clone(), from: None },
            );
        }
        return;
    }
    let words = refusal_words(why);
    let branch = row.landing.branch;
    let log = match domain.forge.forming_refusal(repository, &branch) {
        Some((previous, _)) => match &previous.cause {
            forge::FormingRefusal::Jig { why, .. } => why.as_ref() != words,
            forge::FormingRefusal::Landings => true,
        },
        None => true,
    };
    let retry =
        skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(env.limits.forge.queue_window.as_nanos()));
    let cause = forge::FormingRefusal::Jig { number, why: Box::from(words), retry };
    super::super::forge_event(
        domain,
        env,
        decision,
        forge::Event::TrainRefusal { repository, branch: branch.clone(), cause: Some(cause) },
    );
    if log {
        let mut changes = List::with_capacity(env.limits.forge.landing_members);
        for member in &row.landing.members {
            changes.push(member.change).expect("bounded landing members");
        }
        super::super::diagnostics::emit(
            domain,
            crate::Fact::LandingRefused {
                task: number,
                repository,
                branch,
                why: Box::from(words),
                changes: changes.into_boxed(),
            },
        );
    }
}

fn refuse_table(
    domain: &mut Domain,
    env: &Env<Limits>,
    repository: skein_forge::api::Repository,
    branch: &[u8],
    changes: Box<[u64]>,
) -> bool {
    if let Some((refusal, _)) = domain.forge.forming_refusal(repository, branch)
        && refusal.cause == forge::FormingRefusal::Landings
    {
        return false;
    }
    let holders = domain.forge.landing_room_holders();
    domain.work.push(Work::Forge(forge::Event::TrainRefusal {
        repository,
        branch: Box::from(branch),
        cause: Some(forge::FormingRefusal::Landings),
    }));
    super::super::diagnostics::emit(
        domain,
        crate::Fact::LandingTableRefused {
            repository,
            branch: Box::from(branch),
            limit: env.limits.forge.landings,
            holders,
            changes,
        },
    );
    true
}

/// Retry an idle train from its stored time, not from the current queue window.
pub(crate) fn refusal_deadline(domain: &Domain) -> Option<skein_lib::Wall> {
    if domain.forge.landing_held() {
        return None;
    }
    domain.forge.forming_retry_after(domain.forming_judged)
}

/// Bounded refusal names for the sans-I/O domain (temper/docs/design/domain/landing-forming.md, 3).
fn refusal_words(why: Option<tasks::Refusal>) -> &'static [u8] {
    match why {
        Some(tasks::Refusal::Concluded { .. }) => b"Concluded",
        Some(tasks::Refusal::NotStanding) => b"NotStanding",
        Some(tasks::Refusal::TalkOpen { .. }) => b"TalkOpen",
        Some(tasks::Refusal::NoTalk) => b"NoTalk",
        Some(tasks::Refusal::Placed { .. }) => b"Placed",
        Some(tasks::Refusal::HoldOwner { .. }) => b"HoldOwner",
        Some(tasks::Refusal::HoldRevision { .. }) => b"HoldRevision",
        Some(tasks::Refusal::Account { .. }) => b"Account",
        Some(tasks::Refusal::Grace) => b"Grace",
        Some(tasks::Refusal::Goals) => b"Goals",
        Some(tasks::Refusal::Unproposable) => b"Unproposable",
        Some(tasks::Refusal::PersonRevision { .. }) => b"PersonRevision",
        Some(tasks::Refusal::ResourceUnavailable) => b"ResourceUnavailable",
        Some(tasks::Refusal::Unreported) => b"Unreported",
        Some(tasks::Refusal::NotReady) => b"NotReady",
        Some(tasks::Refusal::Unknown) => b"Unknown",
        Some(tasks::Refusal::Busy) => b"Busy",
        Some(tasks::Refusal::Duplicate) => b"Duplicate",
        Some(tasks::Refusal::Empty) => b"Empty",
        Some(tasks::Refusal::Batch) => b"Batch",
        Some(tasks::Refusal::Live) => b"Live",
        Some(tasks::Refusal::Project) => b"Project",
        Some(tasks::Refusal::Tree) => b"Tree",
        Some(tasks::Refusal::Depth) => b"Depth",
        Some(tasks::Refusal::Delegates) => b"Delegates",
        Some(tasks::Refusal::Reference) => b"Reference",
        Some(tasks::Refusal::Subscription) => b"Subscription",
        Some(tasks::Refusal::Dependencies) => b"Dependencies",
        Some(tasks::Refusal::Circle { .. }) => b"Circle",
        Some(tasks::Refusal::DependencyEnded { .. }) => b"DependencyEnded",
        Some(tasks::Refusal::Kept) => b"Kept",
        Some(tasks::Refusal::KeptQuestion { .. }) => b"KeptQuestion",
        Some(tasks::Refusal::QuestionAnswered { .. }) => b"QuestionAnswered",
        Some(tasks::Refusal::PersonAnswers { .. }) => b"PersonAnswers",
        Some(tasks::Refusal::Executor) => b"Executor",
        Some(tasks::Refusal::Spec) => b"Spec",
        Some(tasks::Refusal::Contract) => b"Contract",
        Some(tasks::Refusal::AuthorityShape) => b"AuthorityShape",
        Some(tasks::Refusal::Inputs) => b"Inputs",
        Some(tasks::Refusal::State) => b"State",
        Some(tasks::Refusal::Attempt) => b"Attempt",
        Some(tasks::Refusal::LiveDelegates) => b"LiveDelegates",
        Some(tasks::Refusal::Restore) => b"Restore",
        Some(tasks::Refusal::Read) => b"Read",
        Some(tasks::Refusal::Turn) => b"Turn",
        Some(tasks::Refusal::Funding) => b"Funding",
        Some(tasks::Refusal::HoldKind) => b"HoldKind",
        Some(tasks::Refusal::HoldTaken) => b"HoldTaken",
        Some(tasks::Refusal::Holds) => b"Holds",
        Some(tasks::Refusal::Llm { .. }) => b"Llm",
        None => b"unknown",
    }
}

#[expect(clippy::match_like_matches_macro, reason = "the foundation requires explicit matching")]
pub(crate) fn start_landing_step(domain: &mut Domain, env: &Env<Limits>, context: &tasks::RunContext) {
    if domain.forge.landing_held() {
        if let Some((connector, code, step)) = domain.engine.core().tasks.procedure_due(context.task) {
            domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep {
                task: context.task,
                connector,
                code,
                step,
                decision: ProcedureStep::new(StepEnd::Wait),
            }));
        }
        return;
    }
    let task = context.task;
    let Some(row) = domain.forge.landing(task) else { return };
    let released = if let landing::Phase::Held { .. } = row.landing.phase { true } else { false };
    let mut held = List::with_capacity(env.limits.forge.landing_members);
    let mut ended = List::with_capacity(env.limits.forge.landing_members);
    for member in &row.landing.members {
        if member.left.is_some() || member.landed.is_some() {
            continue;
        }
        let phase = match domain.engine.core().tasks.task(member.change) {
            Some(task) => Some(&task.phase),
            None => None,
        };
        match phase {
            Some(tasks::Phase::Held { .. }) => held.push(member.change).expect("bounded members"),
            Some(tasks::Phase::Closing(_) | tasks::Phase::Ended(_)) | None => {
                ended.push(member.change).expect("bounded members");
            }
            Some(tasks::Phase::Waiting | tasks::Phase::Active(_)) => {}
        }
    }
    let graph = dependencies(domain, env, row.landing.project);
    let Some(entry) = crate::fresh(&mut domain.engine.core_mut().counters, Family::ConnectorRow) else { return };
    domain.forge.offered_landing(task);
    domain.work.push(Work::Forge(forge::Event::StepLanding {
        task,
        entry,
        heard: forge::LandingHeard {
            job: None,
            outcomes: prepare::outcomes(&context.inbox, env.limits.forge.landing_members),
            dependencies: graph,
            held: held.into_boxed(),
            ended: ended.into_boxed(),
            failure: prepare::failure(&context.inbox),
            preparation: Some(preparation(domain, context)),
            released,
            cancelled: false,
            kept: Box::new([]),
        },
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
    let context = domain.engine.core().tasks.delegation(task)?;
    if repository.project != context.project {
        return None;
    }
    Some(EffectDescription {
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
    })
}

pub(super) fn landing_decision(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    choice: landing::Decision,
    head_trailers: &[HeadTrailers],
) {
    let Some((connector, code, step)) = domain.engine.core().tasks.procedure_due(task) else {
        domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task }));
        return;
    };
    if connector != domain.config.forge_connector || code != LANDING_CODE {
        return;
    }
    domain.forge.offered_landing(task);
    let waiting = ProcedureStep::new(StepEnd::Wait);
    let (decision, wake) = match choice {
        landing::Decision::Wait { until, on: _ } => {
            domain.forge.wait_landing(&super::super::environment_forge(env), task, until);
            (waiting, false)
        }
        landing::Decision::Finish => {
            let words = domain.forge.landing_result_words(task, env.limits.tasks.result_bytes);
            (ProcedureStep::new(StepEnd::Result(tasks::TaskResult::Report { words })), false)
        }
        landing::Decision::Cancel | landing::Decision::End(_) => {
            let reason = if let landing::Decision::End(cause) = choice {
                domain.forge.landing_stuck_words(task, cause).expect("ending landing")
            } else {
                Box::new([])
            };
            domain.work.push(Work::Tasks(tasks::Event::Cancel {
                reply_to: super::super::internal(u64::MAX),
                by: tasks::Party::Deployment { project: domain.forge.landing(task).expect("landing").landing.project },
                operation: tasks::Cancel { task, reason, grace: None, message: None },
            }));
            (waiting, false)
        }
        landing::Decision::Prepare { .. } => {
            let decision = match prepare_delegate(domain, env, task, head_trailers) {
                Ok(delegate) => ProcedureStep { batch: Box::new([delegate]), ..waiting },
                Err(reason) => {
                    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
                    append(&mut words, b"prepare job: ");
                    append(&mut words, reason);
                    super::holds::recovery(domain, task, &mut words);
                    super::holds::decision(domain, env, task, words.into_boxed())
                }
            };
            (decision, false)
        }
        landing::Decision::Hold(_) => {
            let words = domain.forge.landing_hold_words(task).unwrap_or(Box::from(b"prepare job: failed".as_slice()));
            (super::holds::decision(domain, env, task, words), false)
        }
        landing::Decision::None
        | landing::Decision::Open
        | landing::Decision::Close
        | landing::Decision::Delete { .. }
        | landing::Decision::Record { .. }
        | landing::Decision::Landed { .. }
        | landing::Decision::Leave { .. }
        | landing::Decision::Judge { .. }
        | landing::Decision::Resolve { .. }
        | landing::Decision::Key { .. }
        | landing::Decision::Dispatch { .. }
        | landing::Decision::Retry { .. }
        | landing::Decision::Post { .. }
        | landing::Decision::Repair { .. }
        | landing::Decision::Proved
        | landing::Decision::Merge { .. }
        | landing::Decision::Stale => (waiting, true),
    };
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
    if let landing::Decision::Finish | landing::Decision::Cancel | landing::Decision::End(_) = choice {
        wake_train_ready(domain, env, task);
    } else if wake {
        domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task }));
    }
}

// A landing ends for its train alone (domain/landing.md, 4.2, and domain/landing-turn.md, 2).
fn wake_train_ready(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    for change in domain.forge.waiting_after_landing(task, env.limits.forge.changes) {
        domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task: change }));
    }
}

// Read the inbox, live children, and the round's kept answer, in that order;
// otherwise the prepare is absent (temper/docs/design/domain/landing-preparing.md, 2).
pub(super) fn preparation(domain: &Domain, context: &tasks::RunContext) -> landing::Preparation {
    let members = match domain.forge.landing(context.task) {
        Some(row) => u32::try_from(row.landing.members.len()).expect("bounded members"),
        None => 0,
    };
    for word in &context.inbox {
        if let tasks::Party::Task(child) = word.from {
            if super::close::failure(domain, child, &word.kind).is_some()
                || (word.kind == tasks::MessageKind::Result(tasks::ResultKind::Job)
                    && forge::preparation::outcomes(&word.words, members).is_none())
            {
                return landing::Preparation::Failed;
            }
            if word.kind == tasks::MessageKind::Result(tasks::ResultKind::Job)
                && match domain.forge.landing(context.task) {
                    Some(row) => row.landing.round.branch.is_empty(),
                    None => false,
                }
                && prepare::conflict_files(&word.words).is_some()
            {
                return landing::Preparation::Conflict;
            }
            if word.kind == tasks::MessageKind::Result(tasks::ResultKind::Job) {
                return landing::Preparation::Idle;
            }
        }
    }
    if let Some(child) = prepare_child(domain, context.task) {
        return match child.phase {
            tasks::Phase::Held { .. } => landing::Preparation::Failed,
            tasks::Phase::Active(_) | tasks::Phase::Waiting | tasks::Phase::Closing(_) => landing::Preparation::Running,
            tasks::Phase::Ended(_) => unreachable!("prepare child is live"),
        };
    }
    if kept_prepare(domain, context.task) { landing::Preparation::Idle } else { landing::Preparation::Absent }
}

fn prepare_child(domain: &Domain, landing: u64) -> Option<&tasks::TaskRecord> {
    let task = domain.engine.core().tasks.task(landing)?;
    for child in task.delegates.iter().rev() {
        if let Some(child) = domain.engine.core().tasks.task(*child)
            && child.executor == (tasks::Executor::Job { kind: forge::PREPARE_KIND })
        {
            match child.phase {
                tasks::Phase::Held { .. }
                | tasks::Phase::Active(_)
                | tasks::Phase::Waiting
                | tasks::Phase::Closing(_) => {
                    return Some(child);
                }
                tasks::Phase::Ended(_) => {}
            }
        }
    }
    None
}

// An ended child's record counts only with an answer the current round keeps for its suffix
// (temper/docs/design/domain/landing-preparing.md, 2).
fn kept_prepare(domain: &Domain, task: u64) -> bool {
    let Some(row) = domain.forge.landing(task) else { return false };
    let Some(job) = &row.job else { return false };
    if job.round != row.landing.preparation || job.step.job != forge::history::Job::Prepare {
        return false;
    }
    if let Some(child) = domain.engine.core().tasks.task(job.step.task) {
        match child.phase {
            tasks::Phase::Active(_) | tasks::Phase::Waiting | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                return false;
            }
            tasks::Phase::Ended(_) => {}
        }
    }
    match job.step.outcome {
        Some(
            forge::history::JobOutcome::Lost
            | forge::history::JobOutcome::Cancelled
            | forge::history::JobOutcome::Timeout
            | forge::history::JobOutcome::Failed { .. },
        ) => return false,
        Some(forge::history::JobOutcome::Done | forge::history::JobOutcome::Conflict) | None => {}
    }
    let mut admitted = job.members.iter();
    for (index, member) in row.landing.members.iter().enumerate() {
        if member.left.is_none()
            && member.landed.is_none()
            && member.candidate.is_none()
            && admitted.next().copied() != Some(u16::try_from(index).expect("bounded members"))
        {
            return false;
        }
    }
    if admitted.next().is_some() {
        return false;
    }
    let members = u32::try_from(row.landing.members.len()).expect("bounded members");
    let Some(outcomes) = forge::preparation::outcomes(&row.landing.round.failure, members) else { return false };
    for outcome in &outcomes {
        if job.members.contains(&outcome.member) {
            return true;
        }
    }
    false
}
fn prepare_delegate(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    head_trailers: &[HeadTrailers],
) -> Result<super::super::Delegate, &'static [u8]> {
    let bound = domain.config.prepare_kind.ok_or(b"prepare job kind 1 is not declared".as_slice())?;
    let request = prepare::request(&domain.forge, task, env.limits.forge.landing_members, head_trailers)
        .ok_or(b"landing round or repository cannot form its prepare request".as_slice())?;
    let context =
        domain.engine.core().tasks.delegation(task).ok_or(b"landing task has no delegation context".as_slice())?;
    let spec = prepare::spec(&request, bound.spec)
        .ok_or(b"prepare specification exceeds its declared spec bytes".as_slice())?;
    let mut authority = context.authority;
    authority.tools.0 = 0;
    authority.budget.spend = 0;
    authority.delegation.kinds = Box::new([]);
    authority.delegation.tasks = 0;
    authority.delegation.depth = 0;
    Ok(super::super::Delegate {
        executor: tasks::Executor::Job { kind: forge::PREPARE_KIND },
        spec: tasks::Spec {
            job: Some(Box::new(tasks::JobSpec { bytes: spec, deadline: bound.deadline })),
            asks: false,
            words: Box::new([]),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Job { bytes: bound.result },
        authority,
        deadline: tasks::Deadline::Named,
        tracked: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        budget: jig_domain_people::Budget::Named,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: Some(tasks::WakePolicy::DEFAULT),
        llm: None,
    })
}
pub(super) fn prepare_workspace(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    project: u32,
) -> Option<super::RunWorkspace> {
    let request = prepare::request(&domain.forge, task, env.limits.forge.landing_members, &[])?;
    let row = domain.forge.landing(task)?;
    let member = row.landing.members.first()?;
    let provider =
        skein_forge::api::Repository { forge: member.repository.forge, repository: member.repository.repository };
    domain.engine.core().tasks.root(task)?;
    let repository = domain.forge.repository(provider)?;
    if repository.name != request.repository {
        return None;
    }
    let mut branches = List::with_capacity(u32::try_from(request.members.len()).ok()?);
    for member in &request.members {
        branches.push(member.prefix.clone()).ok()?;
    }
    // Start from the request's immutable commit. Its members supply the writable prefixes.
    let item = domain.forge.prepare_item(provider, branches.as_slice().first()?.clone(), request.from, task)?;
    let mut checkout = super::workspace_repository(item, None, Box::new([]))?;
    checkout.prefixes = branches.into_boxed();
    checkout.save = None;
    let workspace = super::ForgeWorkspace {
        rerun: super::rerun_bound(domain, project),
        hook_timeout_max_seconds: super::hook_timeout_max_seconds(domain, project),
        repositories: Box::new([checkout]),
    };
    let mut writes = List::with_capacity(env.limits.forge.landing_members);
    let mut names = List::with_capacity(env.limits.forge.landing_members);
    for member in &request.members {
        let what = super::branch_what(&member.prefix, env.limits.forge.name_bytes)?;
        let name = forge::Name { forge: provider.forge, repository: provider.repository, what };
        writes
            .push(authority::Write {
                held: authority::Writer::Ancestor,
                effect: authority::Effect {
                    connector: domain.config.forge_connector,
                    kind: 2,
                    name: resource_name(repository, &name.what, env.limits.authority.segments)?,
                    state: [0; 32],
                    price: None,
                    access: effect_access(repository, &name.what, 2),
                    additional: Box::new([]),
                    guards: Box::new([]),
                },
            })
            .ok()?;
        names.push(name).ok()?;
    }
    Some(super::RunWorkspace {
        workspace,
        workstream: task,
        writes: writes.into_boxed(),
        names: names.into_boxed(),
        designs: Box::new([]),
        own_holds: Box::new([]),
    })
}
pub(super) fn tell(domain: &mut Domain, env: &Env<Limits>, task: u64, member: u16, step: forge::LandingNews) {
    let Some((change, words)) = domain.forge.landing_news_words(task, member, step) else { return };
    tell_change(domain, env, change, &words);
}

pub(super) fn tell_change(domain: &mut Domain, env: &Env<Limits>, change: u64, words: &[u8]) {
    let Some(row) = domain.forge.change(change) else { return };
    let Some(goal) = row.goal else { return };
    let words = forge::text::fit(words, skein_lib::usize_of(env.limits.tasks.message_bytes), b"...");
    if let Some(event) = domain.engine.core_mut().connector_news(goal, 0, tasks::NewsClass::Kept, words, env.wall) {
        domain.work.push(Work::Tasks(event));
    }
}
