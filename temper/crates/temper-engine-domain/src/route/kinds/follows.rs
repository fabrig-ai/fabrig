//! Cross-domain checks of kind 12 (temper/docs/design/domain/forge/follows.md, 2).
use crate::Domain;
use alloc::boxed::Box;
use skein_lib::{Decimal, List};
use temper_engine_domain_forge::{
    Role,
    settings::{self, FollowOwner},
};

pub(super) fn defaults(domain: &Domain, edit: &mut settings::Edit) {
    let settings::Edit::Set(value) = edit else { return };
    let settings::Value::Follow(follow) = &mut value.value else { return };
    if follow.source.branch.is_empty()
        && let Some(source) = domain.forge.repository(follow.source.repository)
    {
        follow.source.branch.clone_from(&source.settings.default_branch);
    }
    if follow.target.branch.is_empty()
        && let Some(target) = domain.forge.repository(follow.target.repository)
    {
        follow.target.branch.clone_from(&target.settings.default_branch);
    }
    if follow.producer.as_deref() == Some(b"") {
        follow.producer = None;
    }
    if let Some(record) = settings::follow_record(follow) {
        value.size = record.measure();
    }
}

pub(super) fn refusal(domain: &Domain, asked: &settings::Asked) -> Option<Box<[u8]>> {
    let settings::Edit::Set(value) = &asked.edit else { return None };
    let settings::Value::Follow(follow) = &value.value else { return None };
    let source = follow.source.repository;
    let target = follow.target.repository;
    let project = Decimal::of(u64::from(asked.scope));
    if domain.forge.repository_role(asked.scope, source) != Some(Role::Context) {
        let forge = Decimal::of(u64::from(source.forge));
        let repository = Decimal::of(u64::from(source.repository));
        return Some(words(
            asked.number,
            &[
                b"its source ",
                forge.as_bytes(),
                b"/",
                repository.as_bytes(),
                b" is not adopted by project ",
                project.as_bytes(),
                b" as context",
            ],
        ));
    }
    let mut writable = false;
    for repository in domain.forge.writable_repositories(asked.scope) {
        writable |= repository.provider == target;
    }
    if domain.forge.repository_role(asked.scope, target) != Some(Role::Owned) || !writable {
        let forge = Decimal::of(u64::from(target.forge));
        let repository = Decimal::of(u64::from(target.repository));
        return Some(words(
            asked.number,
            &[
                b"its target ",
                forge.as_bytes(),
                b"/",
                repository.as_bytes(),
                b" is not a repository project ",
                project.as_bytes(),
                b" owns and temper writes",
            ],
        ));
    }
    match follow.owner {
        FollowOwner::Person(owner) => {
            if domain.core.people.role(owner, asked.scope).is_none()
                || domain.core.people.kind(owner) != Some(jig_domain_people::Kind::Person)
            {
                let owner = Decimal::of(owner);
                return Some(words(
                    asked.number,
                    &[b"its owner ", owner.as_bytes(), b" is not a person of project ", project.as_bytes()],
                ));
            }
        }
        FollowOwner::Task(owner) => {
            let standing = match domain.core.tasks.task(owner) {
                Some(row) => {
                    row.label == 3
                        && super::super::findings::stands(domain, asked.scope, settings::AreaOwner::Task(owner))
                            .is_some()
                }
                None => false,
            };
            if !standing {
                let owner = Decimal::of(owner);
                return Some(words(
                    asked.number,
                    &[
                        b"its owner task ",
                        owner.as_bytes(),
                        b" is not a live standing role of project ",
                        project.as_bytes(),
                    ],
                ));
            }
        }
    }
    let charter = match &follow.producer {
        Some(producer) => settings::producer_charter(producer).flatten(),
        None => None,
    }
    .unwrap_or(domain.forge.charters().change);
    if domain.core.charter(charter).is_none() {
        let charter = Decimal::of(u64::from(charter));
        return Some(words(
            asked.number,
            &[b"its producer's charter ", charter.as_bytes(), b" is not offered by the settings"],
        ));
    }
    if let Some(current) = domain.forge.follow(asked.scope, asked.number)
        && (current.source.repository != source
            || current.source.branch != follow.source.branch
            || current.target != follow.target)
    {
        return Some(words(asked.number, &[&settings::Refusal::FollowFixed.words()]));
    }
    None
}

pub(super) fn words(number: u32, parts: &[&[u8]]) -> Box<[u8]> {
    let number = Decimal::of(u64::from(number));
    let mut words = List::uncapped();
    for part in [b"follow ".as_slice(), number.as_bytes(), b": "] {
        for &byte in part {
            words.push(byte).expect("follow refusal prefix");
        }
    }
    for part in parts {
        for &byte in *part {
            words.push(byte).expect("follow refusal");
        }
    }
    words.into_boxed()
}
