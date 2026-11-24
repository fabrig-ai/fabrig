//! Forge names and pure judgements at the root boundary
//! (temper/docs/design/domain/forge/README.md, 2–3;
//! temper/docs/design/domain/forge/branches.md, 1;
//! temper/docs/design/domain/forge/change/README.md, 1;
//! temper/docs/design/protocol/channel.md, 4).
use crate as forge;
use crate::workspace::{ForgeRepository, ForgeStart};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core::StepEnd;
use jig_domain_core_vocabulary as jig_domain_core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;
use temper_engine_domain_forge_change as forge_change;
use temper_engine_domain_forge_issues as forge_issues;

/// Translate the worker's connector-specific saved tags at the root boundary.
pub fn saved_resources(connector: u16, tags: &[u64]) -> Option<Box<[tasks::SavedResource]>> {
    let mut resources = List::with_capacity(u32::try_from(tags.len()).ok()?);
    for tag in tags {
        let (forge, repository) = forge::untag(*tag)?;
        resources
            .push(tasks::SavedResource {
                connector,
                path: Box::new([Box::from(forge.to_be_bytes()), Box::from(repository.to_be_bytes())]),
            })
            .ok()?;
    }
    Some(resources.into_boxed())
}

/// Return the forge's resource tags from task-owned generic saved names.
pub fn saved_tags(resources: &[tasks::SavedResource], connector: u16) -> Option<Box<[u64]>> {
    let mut tags = List::with_capacity(u32::try_from(resources.len()).ok()?);
    for resource in resources {
        if resource.connector != connector {
            continue;
        }
        let [forge, repository] = resource.path.as_ref() else { return None };
        let forge = u16::from_be_bytes(forge.as_ref().try_into().ok()?);
        let repository = u32::from_be_bytes(repository.as_ref().try_into().ok()?);
        tags.push(forge::tag(forge, repository)).ok()?;
    }
    Some(tags.into_boxed())
}

/// Build a generic people request from temper's forge repository options.
/// The people child retains only the resource name, role and opaque options.
#[must_use]
pub fn adopt_repository_ask(
    adoption: forge::Adoption,
    repository: skein_forge::api::Repository,
    connector: u16,
) -> Option<people::Ask> {
    let (project, adoption) = adoption_parts(adoption, repository, connector)?;
    Some(people::Ask::Adopt { project, adoption })
}

pub fn adoption_parts(
    adoption: forge::Adoption,
    repository: skein_forge::api::Repository,
    connector: u16,
) -> Option<(u32, people::Adoption)> {
    let role = match adoption.role {
        forge::Role::Owned => people::ResourceRole::Owned,
        forge::Role::Fork => people::ResourceRole::Fork,
        forge::Role::Context => people::ResourceRole::Context,
        forge::Role::Adopted => return None,
    };
    let count = u32::try_from(adoption.checks.len()).ok()?.checked_add(7)?;
    let mut options = List::with_capacity(count);
    options.push(Box::from([u8::from(adoption.home)])).ok()?;
    options.push(adoption.host).ok()?;
    options.push(adoption.owner).ok()?;
    options.push(adoption.name).ok()?;
    options.push(adoption.prefix).ok()?;
    options.push(adoption.landing).ok()?;
    options.push(Box::from([u8::from(adoption.ci)])).ok()?;
    for check in adoption.checks {
        options.push(Box::from(check.to_be_bytes())).ok()?;
    }
    Some((
        adoption.project,
        people::Adoption {
            resource: people::ResourceName {
                connector,
                path: Box::new([
                    Box::from(repository.forge.to_be_bytes()),
                    Box::from(repository.repository.to_be_bytes()),
                ]),
            },
            role,
            options: options.into_boxed(),
        },
    ))
}

pub fn permission_number(permission: skein_forge::api::Permission) -> u16 {
    match permission {
        skein_forge::api::Permission::None => 0,
        skein_forge::api::Permission::Read => 1,
        skein_forge::api::Permission::Write => 2,
        skein_forge::api::Permission::Admin => 3,
    }
}

pub fn append(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        if out.room() > 0 {
            out.push(*byte).expect("checked message room");
        }
    }
}

pub fn topic_name(topic: &forge::Topic, limits: &forge::Limits) -> Option<forge::Name> {
    let (repository, what) = match topic {
        forge::Topic::Findings { .. } => return None,
        forge::Topic::Landings { repository, branch } => {
            let mut parts = List::with_capacity(limits.name_bytes);
            let mut start = 0;
            for at in 0..=branch.len() {
                if at == branch.len() || branch.get(at) == Some(&b'/') {
                    let part = branch.get(start..at)?;
                    if part.is_empty() || parts.push(Box::from(part)).is_err() {
                        return None;
                    }
                    start = at.checked_add(1)?;
                }
            }
            (*repository, forge::What::Branch(parts.into_boxed()))
        }
        forge::Topic::Pull { repository, number } => (*repository, forge::What::Pull(*number)),
        forge::Topic::Participation { repository, number } => (*repository, forge::What::Issue(*number)),
        forge::Topic::Ci { repository, .. } => (*repository, forge::What::Repository),
    };
    Some(forge::Name { forge: repository.forge, repository: repository.repository, what })
}

pub fn decimal(number: u64) -> Box<[u8]> {
    let mut digits = List::with_capacity(20);
    let mut value = number;
    for _ in 0_u32..20_u32 {
        digits
            .push(b'0'.checked_add(u8::try_from(value % 10).expect("decimal digit")).expect("decimal ascii digit"))
            .expect("u64 decimal width");
        value /= 10;
        if value == 0 {
            break;
        }
    }
    let mut forward = List::with_capacity(20);
    for at in (0..digits.len()).rev() {
        forward.push(*digits.get(at).expect("measured decimal digit")).expect("same width");
    }
    forward.into_boxed()
}

pub fn read_name(repository: &forge::Repository, read: &skein_forge::api::Read, limit: u32) -> Option<authority::Name> {
    let mut segments = List::with_capacity(limit);
    for segment in [b"forge".as_slice(), &repository.host, &repository.owner, &repository.name] {
        segments.push(Box::from(segment)).ok()?;
    }
    let what = match read {
        skein_forge::api::Read::Pull { number, .. }
        | skein_forge::api::Read::Reviews { number, .. }
        | skein_forge::api::Read::Remarks { number, .. }
        | skein_forge::api::Read::PullFiles { number, .. } => Some((b"pull".as_slice(), Some(decimal(*number)))),
        skein_forge::api::Read::Item { number, .. } => Some((b"issue".as_slice(), Some(decimal(*number)))),
        skein_forge::api::Read::Branch { branch } | skein_forge::api::Read::Protection { branch } => {
            Some((b"branch".as_slice(), Some(branch.clone())))
        }
        skein_forge::api::Read::Locate { .. }
        | skein_forge::api::Read::File { .. }
        | skein_forge::api::Read::Items { .. }
        | skein_forge::api::Read::PullFor { .. }
        | skein_forge::api::Read::Statuses { .. }
        | skein_forge::api::Read::Compare { .. }
        | skein_forge::api::Read::Checks { .. }
        | skein_forge::api::Read::JobState { .. }
        | skein_forge::api::Read::Job { .. }
        | skein_forge::api::Read::Branches { .. }
        | skein_forge::api::Read::Settings
        | skein_forge::api::Read::Collaborators { .. }
        | skein_forge::api::Read::Runs { .. }
        | skein_forge::api::Read::Run { .. }
        | skein_forge::api::Read::Artifact { .. }
        | skein_forge::api::Read::Commits { .. }
        | skein_forge::api::Read::Tree { .. }
        | skein_forge::api::Read::Permission { .. } => None,
    };
    if let Some((kind, part)) = what {
        segments.push(Box::from(kind)).ok()?;
        if let Some(part) = part {
            let mut start = 0;
            for at in 0..=part.len() {
                if at == part.len() || part.get(at) == Some(&b'/') {
                    let segment = part.get(start..at)?;
                    if segment.is_empty() {
                        return None;
                    }
                    segments.push(Box::from(segment)).ok()?;
                    start = at.checked_add(1)?;
                }
            }
        }
    }
    Some(authority::Name { segments: segments.into_boxed() })
}

pub fn resource_name(repository: &forge::Repository, what: &forge::What, limit: u32) -> Option<authority::Name> {
    let mut segments = List::with_capacity(limit);
    for segment in [b"forge".as_slice(), &repository.host, &repository.owner, &repository.name] {
        segments.push(Box::from(segment)).ok()?;
    }
    match what {
        forge::What::Repository => {}
        forge::What::Issue(number) => {
            segments.push(Box::from(&b"issue"[..])).ok()?;
            segments.push(decimal(*number)).ok()?;
        }
        forge::What::Pull(number) => {
            segments.push(Box::from(&b"pull"[..])).ok()?;
            segments.push(decimal(*number)).ok()?;
        }
        forge::What::Branch(parts) => {
            segments.push(Box::from(&b"branch"[..])).ok()?;
            for part in parts {
                if part.is_empty() || part.contains(&b'/') {
                    return None;
                }
                segments.push(part.clone()).ok()?;
            }
        }
    }
    Some(authority::Name { segments: segments.into_boxed() })
}

pub fn effect_access(repository: &forge::Repository, what: &forge::What, kind: u16) -> authority::EffectAccess {
    match forge::effect_access(repository, what, kind) {
        forge::Access::Owned => authority::EffectAccess::Owned,
        forge::Access::Participant => authority::EffectAccess::Participant,
        forge::Access::Context => authority::EffectAccess::Context,
        forge::Access::Unavailable => authority::EffectAccess::Unavailable,
    }
}

pub fn branch_parts(branch: &[u8], limit: u32) -> Option<Box<[Box<[u8]>]>> {
    let mut parts = List::with_capacity(limit);
    let mut start = 0;
    for at in 0..=branch.len() {
        if at == branch.len() || branch.get(at) == Some(&b'/') {
            let part = branch.get(start..at)?;
            if part.is_empty() {
                return None;
            }
            parts.push(Box::from(part)).ok()?;
            start = at.checked_add(1)?;
        }
    }
    Some(parts.into_boxed())
}

pub fn branch_what(branch: &[u8], limit: u32) -> Option<forge::What> {
    Some(forge::What::Branch(branch_parts(branch, limit)?))
}

#[expect(clippy::manual_map, reason = "the programming model keeps explicit branches in pure routing helpers")]
pub fn tree_branch(
    repository: &forge::Repository,
    root: u64,
    marker: u8,
    task: u64,
    attempt: Option<u64>,
    limit: u32,
) -> Option<Box<[u8]>> {
    let root_bytes = decimal(root);
    let task_bytes = decimal(task);
    let attempt_bytes = match attempt {
        Some(number) => Some(decimal(number)),
        None => None,
    };
    let mut needed =
        repository.prefix.len().checked_add(root_bytes.len())?.checked_add(2)?.checked_add(task_bytes.len())?;
    if let Some(number) = &attempt_bytes {
        needed = needed.checked_add(1)?.checked_add(number.len())?;
    }
    if needed > usize::try_from(limit).ok()? {
        return None;
    }
    let mut result = List::with_capacity(limit);
    append(&mut result, &repository.prefix);
    append(&mut result, &root_bytes);
    append(&mut result, b"/");
    append(&mut result, &[marker]);
    append(&mut result, &task_bytes);
    if let Some(attempt) = attempt_bytes {
        append(&mut result, b"-");
        append(&mut result, &attempt);
    }
    let branch = result.into_boxed();
    branch_what(&branch, limit)?;
    Some(branch)
}

pub use forge::resources::hub_name;

/// Decode only this connector's branch names when projecting a hub hold.
pub fn forge_name(connector: u16, resource: &tasks::Name, limit: u32) -> Option<forge::Name> {
    if resource.connector != connector {
        return None;
    }
    let [forge, repository, branch] = resource.path.as_ref() else { return None };
    let [one, two] = forge.as_ref() else { return None };
    let [three, four, five, six] = repository.as_ref() else { return None };
    Some(forge::Name {
        forge: u16::from_be_bytes([*one, *two]),
        repository: u32::from_be_bytes([*three, *four, *five, *six]),
        what: branch_what(branch, limit)?,
    })
}

pub fn write_holding(connector: u16, name: &forge::Name, limits: &tasks::Limits) -> Option<tasks::Holding> {
    Some(tasks::Holding::Write { resource: hub_name(connector, name, limits)?, kind: 1 })
}

pub fn outbox_outcome(
    entry: u64,
    outcome: skein_forge_client::Outcome,
) -> jig_domain_core::effects::connector::OutboxOutcome {
    match outcome {
        skein_forge_client::Outcome::Made { .. } => jig_domain_core::effects::connector::OutboxOutcome::Made,
        skein_forge_client::Outcome::Failed(_) | skein_forge_client::Outcome::Raced { .. } => {
            jig_domain_core::effects::connector::OutboxOutcome::Failed
        }
        skein_forge_client::Outcome::Withdrawn => jig_domain_core::effects::connector::OutboxOutcome::Withdrawn,
        skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => {
            jig_domain_core::effects::connector::OutboxOutcome::Uncertain
        }
        skein_forge_client::Outcome::Held => jig_domain_core::effects::connector::OutboxOutcome::Held { entry },
    }
}

pub fn projection_phase(phase: &tasks::Phase) -> forge_issues::Phase {
    match phase {
        tasks::Phase::Waiting => forge_issues::Phase::Waiting,
        tasks::Phase::Active(_) => forge_issues::Phase::Active,
        tasks::Phase::Held { .. } => forge_issues::Phase::Held,
        tasks::Phase::Closing(_) => forge_issues::Phase::Settling,
        tasks::Phase::Ended(ending) => ending_phase(ending),
    }
}

pub fn ending_phase(ending: &tasks::Ending) -> forge_issues::Phase {
    match ending {
        tasks::Ending::Done(_) => forge_issues::Phase::Done,
        tasks::Ending::Failed { .. } => forge_issues::Phase::Failed,
        tasks::Ending::Cancelled { .. } => forge_issues::Phase::Cancelled,
    }
}

pub fn goal_phase(phase: &tasks::Phase) -> forge_issues::Phase {
    match phase {
        tasks::Phase::Closing(closing) | tasks::Phase::Held { was: tasks::Was::Closing(closing), .. } => {
            ending_phase(&closing.ending)
        }
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } | tasks::Phase::Ended(_) => {
            projection_phase(phase)
        }
    }
}

pub fn include_repository(selected: &mut List<forge::Repository>, repository: &forge::Repository) -> Option<()> {
    for prior in selected.as_slice() {
        if prior.provider == repository.provider {
            return Some(());
        }
    }
    selected.push(repository.clone()).ok()
}

/// Translate a connector item after its write destination is checked
/// (temper/docs/design/protocol/channel.md, 4).
pub fn workspace_repository(
    item: forge::items::Item,
    push: Option<Box<[u8]>>,
    save: Box<[u8]>,
) -> Option<ForgeRepository> {
    let start = match item.start {
        forge::items::Start::Commit(commit) => ForgeStart::Commit(commit),
        forge::items::Start::Base(branch) => ForgeStart::Base(branch),
        forge::items::Start::Branch(branch) => ForgeStart::Branch(branch),
        forge::items::Start::Saved(branch) => ForgeStart::Saved(branch),
        forge::items::Start::Merge { branch, base } => ForgeStart::Merge { branch, base },
        forge::items::Start::Merged { branch, heads } => ForgeStart::Merged { branch, heads },
    };
    Some(ForgeRepository {
        prefixes: Box::new([]),
        delivers: item.delivers && push.is_some(),
        save: if push.is_some() { Some(save) } else { None },
        tag: forge::tag(item.provider.forge, item.provider.repository),
        provider: item.provider,
        name: item.name,
        host: item.host,
        owner: item.owner,
        start,
        landing: if push.is_some() { item.landing } else { None },
        delivered: item.delivered,
        pin: if push.is_some() { item.pin } else { None },
        push,
        // The repository's git identity, from configuration (D10).
        identity: item.credential?,
    })
}

#[expect(clippy::manual_map, reason = "the programming model keeps explicit branches in pure routing helpers")]
pub fn pull_what(number: Option<u64>) -> Option<forge::What> {
    match number {
        Some(number) => Some(forge::What::Pull(number)),
        None => None,
    }
}

pub fn initial_change_title(parameters: &[tasks::Parameter]) -> Box<[u8]> {
    let mut handed = false;
    let mut title = None;
    for parameter in parameters {
        match parameter {
            tasks::Parameter::Bytes { name: 3 | 19, .. } => handed = true,
            tasks::Parameter::Bytes { name: 10, value } => title = Some(value.as_ref()),
            tasks::Parameter::Bytes { .. } | tasks::Parameter::Number { .. } | tasks::Parameter::Resource { .. } => {}
        }
    }
    if !handed {
        return Box::new([]);
    }
    match title {
        Some(title) if !title.is_empty() => forge::text::fit(title, 256, "…".as_bytes()),
        Some(_) | None => Box::from(forge::DEFAULT_CHANGE_TITLE),
    }
}

/// Register the connector's procedure beside the task's first due step.
/// Parameter 1 names a project-unique repository tag, 2 the base,
/// 3 an optional already-pushed branch, 4 the pull body, 5 priority, and 10 the pull title.
pub fn change_delegate_status(
    kind: &tasks::MessageKind,
    delegate: forge_change::Delegate,
) -> Option<(forge_change::Status, bool, bool)> {
    match kind {
        tasks::MessageKind::Result(
            tasks::ResultKind::NoChange
            | tasks::ResultKind::Report
            | tasks::ResultKind::Change { .. }
            | tasks::ResultKind::Verdict { code: 1, .. }
            | tasks::ResultKind::Job,
        ) => Some((forge_change::Status::Passed, false, false)),
        tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: forge::REPAIR_NO_TEST, .. })
            if repair_verdict(delegate, forge::REPAIR_NO_TEST) =>
        {
            Some((forge_change::Status::Passed, false, false))
        }
        tasks::MessageKind::Result(tasks::ResultKind::Failed) => Some((forge_change::Status::Failed, true, false)),
        tasks::MessageKind::Result(tasks::ResultKind::Cancelled) => Some((forge_change::Status::Failed, false, true)),
        tasks::MessageKind::Result(tasks::ResultKind::Verdict { .. } | tasks::ResultKind::Approved { .. }) => {
            Some((forge_change::Status::Failed, false, false))
        }
        tasks::MessageKind::Escalation { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::SettingsDecision { .. }
        | tasks::MessageKind::ProposalDecision { .. }
        | tasks::MessageKind::Words
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Question
        | tasks::MessageKind::Answer { .. }
        | tasks::MessageKind::Met { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::QuestionAnswered { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::QuestionEnded { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::PersonStalled { .. }
        | tasks::MessageKind::EffectNotMade { .. }
        | tasks::MessageKind::News { .. } => None,
    }
}

// Every live pushing child counts, even after a gate takes the current delegate slot
// (temper/docs/design/domain/forge/change/repairs.md). Jig keeps these through their terminals.
pub fn repair_verdict(kind: forge_change::Delegate, code: u32) -> bool {
    match kind {
        forge_change::Delegate::Repair(
            forge_change::Repair::Landing { .. } | forge_change::Repair::LandingAt { .. },
        ) => code == forge::REPAIR_FLAKE || code == forge::REPAIR_NO_TEST,
        forge_change::Delegate::Repair(
            forge_change::Repair::Ci
            | forge_change::Repair::Semantic
            | forge_change::Repair::Gate(_)
            | forge_change::Repair::Review { .. },
        ) => code == forge::REPAIR_FLAKE,
        forge_change::Delegate::Produce
        | forge_change::Delegate::Close
        | forge_change::Delegate::Gate { .. }
        | forge_change::Delegate::Resolve { .. }
        | forge_change::Delegate::Merge { .. } => false,
    }
}

// Only a procedure's own hold can be released by the deployment (jig's domain/tasks/changes.md).
pub fn own_hold(task: &tasks::TaskRecord) -> bool {
    if task.placed.is_some() {
        return false;
    }
    match task.phase {
        tasks::Phase::Held { why, .. } => match why {
            tasks::Hold::Procedure | tasks::Hold::EffectFailed | tasks::Hold::Stalled | tasks::Hold::Budget => true,
            tasks::Hold::Exceeded(_)
            | tasks::Hold::Failures(_)
            | tasks::Hold::Dependency(_)
            | tasks::Hold::Stopped
            | tasks::Hold::Placed { .. }
            | tasks::Hold::Drift
            | tasks::Hold::Effects
            | tasks::Hold::Uncertain { .. }
            | tasks::Hold::Deadline
            | tasks::Hold::HoldsWaited
            | tasks::Hold::ResourceUnavailable
            | tasks::Hold::ResourceAccess
            | tasks::Hold::PoolLost { .. }
            | tasks::Hold::Circle => false,
        },
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
    }
}

/// The code-2 procedure's repository and landing parameters, shared by chat
/// changes and deployment queue repairs (domain/agent/tools-change.md, 1).
pub fn change_parameters(
    connector: u16,
    provider: skein_forge::api::Repository,
    base: Box<[u8]>,
) -> [tasks::Parameter; 2] {
    [
        tasks::Parameter::Resource { name: 1, connector, resource: forge::tag(provider.forge, provider.repository) },
        tasks::Parameter::Bytes { name: 2, value: base },
    ]
}

pub fn repository_name(repository: &forge::Repository) -> Box<[u8]> {
    let count = repository
        .owner
        .len()
        .checked_add(repository.name.len())
        .expect("bounded repository name")
        .checked_add(1)
        .expect("repository slash");
    let count = u32::try_from(count).expect("bounded repository name");
    let mut name = List::with_capacity(count);
    append(&mut name, &repository.owner);
    append(&mut name, b"/");
    append(&mut name, &repository.name);
    name.into_boxed()
}

pub fn child_authority(context: &tasks::DelegationContext) -> Option<tasks::Authority> {
    let mut authority = context.authority.clone();
    let available = context
        .numbers
        .budget
        .checked_sub(context.numbers.spent)?
        .checked_sub(context.numbers.spent_below)?
        .checked_sub(context.numbers.reserved)?;
    authority.budget.spend = authority.budget.spend.min(available);
    if authority.budget.spend == 0 {
        return None;
    }
    authority.delegation.kinds = Box::new([]);
    authority.delegation.tasks = 0;
    authority.delegation.depth = 0;
    Some(authority)
}

/// Feedback names the question route and answer form (domain/forge/change/README.md, 1, domain/forge/change/questions.md, 1).
pub fn takes_no_messages(requester: tasks::Party, route: forge_change::Route, limit: u32) -> Box<[u8]> {
    let mut words = List::with_capacity(limit);
    if route != forge_change::Route::None {
        append(&mut words, b"Relays questions. Use read_task on this change; message target=this change, form=answer, question=number. Words do not answer.");
        return words.into_boxed();
    }
    append(&mut words, b"A change takes no questions, answers or words; its requester is ");
    let (kind, number): (&[u8], u64) = match requester {
        tasks::Party::Task(number) => (b"task ", number),
        tasks::Party::Person(number) => (b"person ", number),
        tasks::Party::Deployment { project } => (b"deployment for project ", u64::from(project)),
    };
    append(&mut words, kind);
    append(&mut words, &decimal(number));
    append(&mut words, b".");
    words.into_boxed()
}

pub fn gate_executor(row: &forge::ChangeRow, kind: forge_change::GateKind) -> tasks::Executor {
    match kind {
        forge_change::GateKind::Agent => tasks::Executor::Agent { charter: row.review_charter },
        forge_change::GateKind::Check => tasks::Executor::Agent { charter: row.producing_charter },
        forge_change::GateKind::Follow => unreachable!("a dispatched check has no delegate"),
        forge_change::GateKind::Person => tasks::Executor::Person(tasks::PersonAddress::Role(1)),
    }
}

pub fn change_hold_event(task: u64, end: &StepEnd) -> tasks::Event {
    match end {
        StepEnd::Hold(why) => tasks::Event::Hold { task, why: *why },
        StepEnd::HoldFrom { why, source } => tasks::Event::HoldFrom { task, why: *why, source: source.clone() },
        StepEnd::Result(_) | StepEnd::Wait => unreachable!("a change hold decides a task hold"),
    }
}

/// Installation is missing access, not a transient busy adoption (domain/forge/adoption.md, 7).
pub fn adoption_failure(error: skein_forge::api::Error) -> people::Outcome {
    match error {
        skein_forge::api::Error::NotConnected
        | skein_forge::api::Error::Busy
        | skein_forge::api::Error::Unavailable
        | skein_forge::api::Error::Timeout
        | skein_forge::api::Error::RateLimited { .. } => people::Outcome::Again(people::Again::Busy),
        skein_forge::api::Error::TooLarge(_) | skein_forge::api::Error::Full => {
            people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Resource })
        }
        skein_forge::api::Error::NotInstalled
        | skein_forge::api::Error::Forbidden
        | skein_forge::api::Error::Protected
        | skein_forge::api::Error::Refused => people::Outcome::Refused(people::Refusal::Authority {
            lacking: people::Lack::ResourceAccess,
            proposable: None,
        }),
        skein_forge::api::Error::Missing => people::Outcome::Refused(people::Refusal::Authority {
            lacking: people::Lack::ResourceUnavailable,
            proposable: None,
        }),
        skein_forge::api::Error::MissingJob
        | skein_forge::api::Error::Empty
        | skein_forge::api::Error::Exists
        | skein_forge::api::Error::NothingToMerge
        | skein_forge::api::Error::Closed
        | skein_forge::api::Error::Stale
        | skein_forge::api::Error::BaseMoved { .. }
        | skein_forge::api::Error::Conflict
        | skein_forge::api::Error::InvalidAnswer => people::Outcome::Refused(people::Refusal::Unknown),
    }
}

/// Parameter 18's decimal follow and ordered landings (domain/forge/follows.md, 4).
pub fn change_follows(text: &[u8], limit: u32) -> Option<forge::ChangeFollows> {
    let colon = skein_lib::bytes::find(text, b":")?;
    let follow = u32::try_from(skein_lib::bytes::decimal(text.get(..colon)?)?).ok()?;
    let mut remaining = Some(text.get(colon.checked_add(1)?..)?);
    let mut commits = List::with_capacity(limit);
    for _ in 0..limit {
        let Some(text) = remaining else { break };
        let end = skein_lib::bytes::find(text, b",").unwrap_or(text.len());
        commits.push(catch_up_commit(text.get(..end)?)?).ok()?;
        remaining = if end == text.len() { None } else { Some(text.get(end.checked_add(1)?..)?) };
    }
    if remaining.is_some() {
        return None;
    }
    Some(forge::ChangeFollows { follow, commits: commits.into_boxed() })
}

/// The source parameter (temper/docs/design/domain/forge/change/opened.md, 3.1).
pub fn change_source(text: &[u8], limit: u32) -> Option<forge::ChangeSource> {
    let mut last_colon = None;
    for (index, byte) in text.iter().enumerate() {
        if *byte == b':' {
            last_colon = Some(index);
        }
    }
    let (branch, at) = match last_colon {
        Some(colon) => (text.get(..colon)?, Some(catch_up_commit(text.get(colon.checked_add(1)?..)?)?)),
        None => (text, None),
    };
    if branch.len() > skein_lib::usize_of(limit) || branch.contains(&b':') {
        return None;
    }
    branch_parts(branch, limit)?;
    Some(forge::ChangeSource { branch: Box::from(branch), at })
}

pub fn catch_up_commit(text: &[u8]) -> Option<[u8; 32]> {
    if text.len() != 40 && text.len() != 64 {
        return None;
    }
    let mut commit = [0_u8; 32];
    for (pair, cell) in text.as_chunks::<2>().0.iter().zip(&mut commit) {
        let mut byte = 0_u8;
        for digit in pair {
            let digit = match digit {
                b'0'..=b'9' => digit.saturating_sub(b'0'),
                b'a'..=b'f' => digit.saturating_sub(b'a').saturating_add(10),
                _ => return None,
            };
            byte = byte.saturating_mul(16).saturating_add(digit);
        }
        *cell = byte;
    }
    Some(commit)
}
