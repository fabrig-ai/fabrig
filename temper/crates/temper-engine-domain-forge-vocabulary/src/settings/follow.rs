//! Structure and codec of kind 12 (temper/docs/design/domain/forge/follows.md, 2).
use super::{Bounds, Follow, FollowCheck, FollowOwner, FollowSource, FollowStart, FollowTarget, Refusal, within};
use skein_lib::{
    List,
    bytes::{decimal, find},
};
use temper_client_protocol_wire::v1 as w;
use temper_engine_domain_forge_tiers::Format;

pub(super) fn judge(value: &Follow, bounds: &Bounds) -> Result<(), Refusal> {
    if !within(value.source.paths.len(), bounds.follow_paths) {
        return Err(Refusal::FollowPaths);
    }
    for path in &value.source.paths {
        if path.is_empty() || !within(path.len(), 256) || path.first() == Some(&b'/') || path.last() == Some(&b'/') {
            return Err(Refusal::FollowPath);
        }
        for pair in path.windows(2) {
            if pair == b"//" {
                return Err(Refusal::FollowPath);
            }
        }
    }
    if !within(value.source.branch.len(), bounds.name_bytes) || !within(value.target.branch.len(), bounds.name_bytes) {
        return Err(Refusal::FollowBranch);
    }
    if let Some(producer) = &value.producer
        && (!within(producer.len(), 8192) || producer_charter(producer).is_none())
    {
        return Err(Refusal::FollowProducer);
    }
    for member in [&value.check.workflow, &value.check.artifact] {
        if member.is_empty() || !within(member.len(), 256) {
            return Err(Refusal::FollowCheck);
        }
    }
    Ok(())
}

/// The charter in change parameter 11; `None` outside its grammar. An empty
/// value uses the defaults (temper/docs/design/domain/forge/follows.md, 2).
#[must_use]
pub fn producer_charter(text: &[u8]) -> Option<Option<u32>> {
    if text.is_empty() {
        return Some(None);
    }
    let (charter, choices) = split(text, b';')?;
    let (producing, review) = split(choices, b';')?;
    if find(review, b";").is_some() || !choice(producing) || !choice(review) {
        return None;
    }
    if charter.is_empty() {
        return Some(None);
    }
    let charter = u32::try_from(decimal(charter)?).ok()?;
    if charter == 0 { None } else { Some(Some(charter)) }
}

fn split(text: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let at = find(text, &[separator])?;
    Some((text.get(..at)?, text.get(at.checked_add(1)?..)?))
}

fn choice(text: &[u8]) -> bool {
    if text.is_empty() {
        return true;
    }
    let mut remaining = text;
    for _ in 0..=jig_domain_tasks::MODELS {
        let (entry, rest) = match split(remaining, b' ') {
            Some((entry, rest)) => (entry, Some(rest)),
            None => (remaining, None),
        };
        if entry_number(entry).is_none() {
            return false;
        }
        match rest {
            Some(rest) => remaining = rest,
            None => return true,
        }
    }
    false
}

fn entry_number(entry: &[u8]) -> Option<u8> {
    let (model, _effort) = match split(entry, b'.') {
        Some((model, effort)) => (model, Some(u8::try_from(decimal(effort)?).ok()?)),
        None => (entry, None),
    };
    u8::try_from(decimal(model)?).ok()
}

/// The kind's wire record, shared by its entrances and store's matching record.
#[must_use]
pub fn follow_record(value: &Follow) -> Option<w::Follow> {
    let limits = &w::CEILINGS;
    let mut paths = List::with_capacity(u32::try_from(value.source.paths.len()).ok()?);
    for path in &value.source.paths {
        paths.push(path.clone()).ok()?;
    }
    let source = w::FollowSource::new(
        limits,
        w::FollowSourceParts {
            repository: repository_record(value.source.repository)?,
            branch: value.source.branch.clone(),
            paths,
        },
    )
    .ok()?;
    let target = w::FollowTarget::new(
        limits,
        w::FollowTargetParts {
            repository: repository_record(value.target.repository)?,
            branch: value.target.branch.clone(),
        },
    )
    .ok()?;
    let owner = match value.owner {
        FollowOwner::Person(person) => {
            w::FollowOwner::Person(w::FollowPerson::new(limits, w::FollowPersonParts { person }).ok()?)
        }
        FollowOwner::Task(task) => {
            w::FollowOwner::Task(w::FollowOwnerTask::new(limits, w::FollowOwnerTaskParts { task }).ok()?)
        }
    };
    let start = match value.start {
        FollowStart::Owner => w::FollowStart::Owner,
        FollowStart::Code => w::FollowStart::Code,
    };
    let check = w::FollowCheck::new(
        limits,
        w::FollowCheckParts {
            workflow: value.check.workflow.clone(),
            artifact: value.check.artifact.clone(),
            format: match value.check.format {
                Format::Junit => w::FollowFormat::Junit,
                Format::Lines => w::FollowFormat::Lines,
            },
        },
    )
    .ok()?;
    w::Follow::new(
        limits,
        w::FollowParts { source, target, owner, start, producer: value.producer.clone(), verification: check },
    )
    .ok()
}

fn repository_record(repository: skein_forge::api::Repository) -> Option<w::FollowRepository> {
    w::FollowRepository::new(
        &w::CEILINGS,
        w::FollowRepositoryParts { forge: repository.forge, repository: repository.repository },
    )
    .ok()
}

/// Move a decoded kind 12 into the domain, leaving judgement to its owner.
#[must_use]
pub fn follow_from(value: w::Follow) -> Follow {
    let w::FollowParts { source, target, owner, start, producer, verification: check } = value.into_parts();
    let source = source.into_parts();
    let target = target.into_parts();
    let check = check.into_parts();
    Follow {
        source: FollowSource {
            repository: repository_from(source.repository),
            branch: source.branch,
            paths: source.paths.into_boxed(),
        },
        target: FollowTarget { repository: repository_from(target.repository), branch: target.branch },
        owner: match owner {
            w::FollowOwner::Person(person) => FollowOwner::Person(person.person()),
            w::FollowOwner::Task(task) => FollowOwner::Task(task.task()),
        },
        start: match start {
            w::FollowStart::Owner => FollowStart::Owner,
            w::FollowStart::Code => FollowStart::Code,
        },
        producer,
        check: FollowCheck {
            workflow: check.workflow,
            artifact: check.artifact,
            format: match check.format {
                w::FollowFormat::Junit => Format::Junit,
                w::FollowFormat::Lines => Format::Lines,
            },
        },
    }
}

fn repository_from(value: w::FollowRepository) -> skein_forge::api::Repository {
    skein_forge::api::Repository { forge: value.forge(), repository: value.repository() }
}
