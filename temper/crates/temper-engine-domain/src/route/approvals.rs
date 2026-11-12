//! Each watcher's changes waiting on a person's approval (domain/forge/change/questions.md, 2).
use crate::{Domain, Limits};
use alloc::boxed::Box;
use jig_domain_people::Role;
use skein_lib::{Env, List, Wall};
use temper_engine_domain_forge::{self as forge, settings};

/// Whether the watcher's review would count for the approval a row waits on (domain/forge/change/questions.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum YourReview {
    Counts,
    /// The watcher doesn't count as a person on the change's forge.
    NoAccount,
    /// Fewer members count as people there with the role than the approval asks.
    TooFew,
    /// The approval's role isn't the watcher's: an owner seeing the wait.
    OtherRole,
}

/// A change waiting on a person's approval, as one watcher sees it (domain/forge/change/questions.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ApprovalRow {
    pub task: u64,
    pub project: u32,
    /// The outermost goal above the change's task, if any.
    pub goal: Option<u64>,
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub pull: u64,
    pub title: Box<[u8]>,
    pub since: Wall,
    /// The approval it waits on: 0 for the deployment's rules, else the project.
    pub scope: u32,
    pub rule: u32,
    pub role: u32,
    pub required: u32,
    pub counted: u32,
    pub sets: Box<[ApprovalSet]>,
    pub whole: bool,
    pub yours: YourReview,
    pub signed: Box<[ApprovalPerson]>,
    pub can_sign: Box<[ApprovalPerson]>,
    pub can_sign_more: u32,
    pub size: Option<ApprovalSize>,
}

/// The forge's counts narrowed for an approval row (client/forge/records.md, `approvals`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ApprovalSize {
    pub files: u32,
    pub additions: u32,
    pub deletions: u32,
    pub cut: bool,
}

/// A touched path set of this row's approval (domain/forge/change/questions.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ApprovalSet {
    pub number: u8,
    pub name: Box<[u8]>,
}

/// A signer at the head or a person who can still sign (domain/forge/change/questions.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ApprovalPerson {
    pub person: u64,
    pub name: Box<[u8]>,
}

/// One watcher's approvals document (client/forge/records.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Approvals {
    pub rows: Box<[ApprovalRow]>,
    /// The rows past the bound.
    pub more: u32,
}

impl Approvals {
    /// Deeply owned row bytes in one whole delivery (domain/forge/change/questions.md, 2).
    pub(crate) fn owned_bytes(&self) -> usize {
        let mut bytes = 0_usize;
        for row in &self.rows {
            bytes = bytes
                .saturating_add(size_of::<ApprovalRow>())
                .saturating_add(row.owner.len())
                .saturating_add(row.name.len())
                .saturating_add(row.title.len())
                .saturating_add(size_of_val(row.sets.as_ref()));
            for people in [&row.signed, &row.can_sign] {
                bytes = bytes.saturating_add(size_of_val(people.as_ref()));
                for person in people {
                    bytes = bytes.saturating_add(person.name.len());
                }
            }
            for set in &row.sets {
                bytes = bytes.saturating_add(set.name.len());
            }
        }
        bytes
    }
}

/// The watcher's approvals (domain/forge/change/questions.md, 2).
pub(super) fn document(domain: &Domain, env: &Env<Limits>, party: u64) -> Approvals {
    let mut rows = List::uncapped();
    for wait in domain.forge.approval_waits(env.wall, &env.limits.forge) {
        if let Some(row) = row(domain, &wait, party) {
            rows.push(row).expect("approval waits are bounded by live changes");
        }
    }
    cut(rows.into_boxed(), crate::room::approval_rows(&env.limits))
}

fn cut(rows: Box<[ApprovalRow]>, bound: u32) -> Approvals {
    let mut kept = List::with_capacity(bound);
    let mut more = 0_u32;
    for row in rows {
        if kept.len() < bound {
            kept.push(row).expect("bounded approvals document");
        } else {
            more = more.saturating_add(1);
        }
    }
    Approvals { rows: kept.into_boxed(), more }
}

/// One wait as `party` sees it, or none (domain/forge/change/questions.md, 2).
#[expect(
    clippy::too_many_lines,
    reason = "the watcher-specific approval carries both signer eligibility and forge size"
)]
pub(super) fn row(domain: &Domain, wait: &forge::ApprovalWait, party: u64) -> Option<ApprovalRow> {
    let role = domain.core.people.role(party, wait.project)?;
    let mut chosen = None;
    for approval in &wait.short {
        if approval.role == role.number() {
            chosen = Some(approval);
            break;
        }
    }
    let approval = match chosen {
        Some(approval) => approval,
        None => match role {
            Role::Owner => wait.short.first()?,
            Role::Maintainer | Role::Member | Role::Observer | Role::Policy { .. } => return None,
        },
    };
    for review in &approval.counted {
        if review.person == party {
            return None;
        }
    }
    let eligible = super::kinds::eligible_members(domain, wait.project, wait.forge, approval.role);
    let mut signed = List::with_capacity(8);
    for review in approval.counted.iter().take(8) {
        signed.push(person(domain, review.person)).expect("eight signed people");
    }
    let mut can_sign = List::with_capacity(8);
    let mut can_sign_more = 0_u32;
    if eligible.as_slice().contains(&party) {
        can_sign.push(person(domain, party)).expect("watcher first");
    }
    for &member in &eligible {
        if member == party {
            continue;
        }
        let mut counted = false;
        for review in &approval.counted {
            if review.person == member {
                counted = true;
                break;
            }
        }
        if counted {
            continue;
        }
        if can_sign.len() < 8 {
            can_sign.push(person(domain, member)).expect("eight eligible people");
        } else {
            can_sign_more = can_sign_more.saturating_add(1);
        }
    }
    let yours = if eligible.len() < approval.required {
        YourReview::TooFew
    } else if !super::forge_route::forge_human(domain, wait.project, wait.forge, party) {
        YourReview::NoAccount
    } else if approval.role != role.number() {
        YourReview::OtherRole
    } else {
        YourReview::Counts
    };
    let mut sets = List::with_capacity(6);
    let classes = domain.forge.project_classes(wait.project);
    for number in 1_u8..=6 {
        let bit = 1_u8.checked_shl(u32::from(number.saturating_sub(1))).expect("six path sets");
        if approval.sets & bit == 0 {
            continue;
        }
        let mut name: Box<[u8]> = Box::default();
        for set in &classes.sets {
            if set.number == number {
                name.clone_from(&set.name);
                break;
            }
        }
        sets.push(ApprovalSet { number, name }).expect("six path sets");
    }
    Some(ApprovalRow {
        task: wait.task,
        project: wait.project,
        goal: super::public::public_goal(domain, wait.task),
        forge: wait.forge,
        owner: wait.owner.clone(),
        name: wait.name.clone(),
        pull: wait.pull,
        title: wait.title.clone(),
        since: wait.since,
        scope: approval.scope,
        rule: approval.rule,
        role: approval.role,
        required: approval.required,
        counted: u32::try_from(approval.counted.len()).expect("bounded counted people"),
        sets: sets.into_boxed(),
        whole: wait.whole,
        yours,
        signed: signed.into_boxed(),
        can_sign: can_sign.into_boxed(),
        can_sign_more,
        size: match wait.size {
            Some(size) => Some(narrow(size)),
            None => None,
        },
    })
}

fn person(domain: &Domain, person: u64) -> ApprovalPerson {
    let name = match domain.core.people.identity(person) {
        Some(identity) => {
            let name = if identity.name.is_empty() { &identity.login } else { &identity.name };
            Box::from(skein_lib::bytes::utf8_prefix(name, 64))
        }
        None => Box::default(),
    };
    ApprovalPerson { person, name }
}

fn narrow(size: skein_forge::api::Size) -> ApprovalSize {
    ApprovalSize {
        files: u32::try_from(size.files).unwrap_or(u32::MAX),
        additions: u32::try_from(size.additions).unwrap_or(u32::MAX),
        deletions: u32::try_from(size.deletions).unwrap_or(u32::MAX),
        cut: size.files > u64::from(u32::MAX)
            || size.additions > u64::from(u32::MAX)
            || size.deletions > u64::from(u32::MAX),
    }
}

/// Whether a proposal sets an enforcing approval covering an owned forge's writable repository
/// (domain/forge/change/questions.md, 2), judged as approval eligibility is (8.9).
pub(super) fn owned_approval(domain: &Domain, items: &[(settings::SettingKey, settings::Edit)]) -> bool {
    for (key, edit) in items {
        if key.kind != 1 && key.kind != 2 {
            continue;
        }
        let settings::Edit::Set(value) = edit else { continue };
        let settings::Value::Rule(rule) = &value.value else { continue };
        if !rule.enforce || rule.approvals.is_empty() {
            continue;
        }
        for &project in &domain.core.projects {
            if key.scope != 0 && key.scope != project {
                continue;
            }
            for repository in domain.forge.writable_repositories(project) {
                let Some(forge) = domain.forge.configured(repository.provider.forge) else { continue };
                if forge.mode == forge::Mode::Owned && super::kinds::covers_repository(rule, repository) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
pub(crate) mod tests;
