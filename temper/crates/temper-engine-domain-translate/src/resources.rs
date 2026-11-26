//! Literal forge and task resource identities (temper/docs/design/domain/forge/branches.md, 1).
use crate::append;
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::List;
use temper_engine_domain_forge as forge;

/// Translate the worker's connector-specific saved tags at the root boundary.
#[must_use]
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
#[must_use]
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

/// A closing task's saved-work branches, from its record's saved set.
#[must_use]
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

#[must_use]
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

#[must_use]
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

#[must_use]
pub fn effect_access(repository: &forge::Repository, what: &forge::What, kind: u16) -> authority::EffectAccess {
    match forge::effect_access(repository, what, kind) {
        forge::Access::Owned => authority::EffectAccess::Owned,
        forge::Access::Participant => authority::EffectAccess::Participant,
        forge::Access::Context => authority::EffectAccess::Context,
        forge::Access::Unavailable => authority::EffectAccess::Unavailable,
    }
}

#[must_use]
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

#[must_use]
pub fn branch_what(branch: &[u8], limit: u32) -> Option<forge::What> {
    Some(forge::What::Branch(branch_parts(branch, limit)?))
}

#[must_use]
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
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

/// The hub sees an opaque, bounded name for a forge branch. Forge path syntax
/// stays here; the task child only compares these literal bytes.
#[must_use]
pub fn hub_name(connector: u16, name: &forge::Name, limits: &tasks::Limits) -> Option<tasks::Name> {
    let forge::What::Branch(parts) = &name.what else { return None };
    if limits.hold_segments < 3 {
        return None;
    }
    let mut branch = List::with_capacity(limits.hold_bytes);
    for (at, part) in parts.iter().enumerate() {
        if at != 0 {
            branch.push(b'/').ok()?;
        }
        for byte in part.iter().copied() {
            branch.push(byte).ok()?;
        }
    }
    if branch.len().checked_add(6)? > limits.hold_bytes {
        return None;
    }
    Some(tasks::Name {
        connector,
        path: Box::new([
            Box::from(name.forge.to_be_bytes()),
            Box::from(name.repository.to_be_bytes()),
            branch.into_boxed(),
        ]),
    })
}

/// Decode only this connector's branch names when projecting a hub hold.
#[must_use]
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

#[must_use]
pub fn write_holding(connector: u16, name: &forge::Name, limits: &tasks::Limits) -> Option<tasks::Holding> {
    Some(tasks::Holding::Write { resource: hub_name(connector, name, limits)?, kind: 1 })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn adopted(forge: u16) -> forge::Repository {
        forge::Repository {
            contexts: Box::new([]),
            landing: b"main".as_slice().into(),
            refusals: Box::default(),
            workflows: Box::default(),
            project: 5,
            home: forge == 1,
            provider: skein_forge::api::Repository { forge, repository: 1 },
            host: Box::from(&b"forge.example"[..]),
            owner: Box::from(&b"org"[..]),
            name: Box::from(&b"repo"[..]),
            prefix: Box::from(&b"temper/"[..]),
            role: forge::Role::Owned,
            ci: true,
            checks: Box::new([]),
            kinds: forge::Kinds {
                read: true,
                push: true,
                open: true,
                land: true,
                review: true,
                status: true,
                comment: true,
                issue: true,
                branch: true,
            },
            protection: forge::Protection::Absent,
            settings: skein_forge::api::Settings {
                default_branch: Box::from(&b"main"[..]),
                merge: true,
                squash: true,
                rebase: false,
            },
        }
    }

    #[test]
    fn tree_branches_count_prefix_slashes_against_authority_segments() {
        // forge, host, owner, repository, branch, temper, root, marker/task.
        const AUTHORITY_SEGMENTS: u32 = 8;
        // The old live prefix puts live-x in a branch segment of its own.
        const DEEP_AUTHORITY_SEGMENTS: u32 = 9;
        const BRANCH_BYTES: u32 = 128;
        for (prefix, segments, fits) in [
            (b"temper/live-x/".as_slice(), AUTHORITY_SEGMENTS, false),
            (b"temper/live-x/", DEEP_AUTHORITY_SEGMENTS, true),
            (b"temper/", AUTHORITY_SEGMENTS, true),
            (b"temper/live-x-", AUTHORITY_SEGMENTS, true),
        ] {
            let mut repository = adopted(1);
            repository.prefix = prefix.into();
            for marker in *b"crs" {
                for attempt in [None, Some(3)] {
                    let branch = tree_branch(&repository, 1, marker, 2, attempt, BRANCH_BYTES).expect("tree branch");
                    let what = branch_what(&branch, BRANCH_BYTES).expect("branch parts");
                    assert_eq!(resource_name(&repository, &what, segments).is_some(), fits, "branch {branch:?}");
                }
            }
        }
    }
}
