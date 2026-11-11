//! Checkout items (jig's domain/connectors.md, section 10). The connector
//! selects starts and write destinations from its adopted and procedure facts.
//! Items are returned whole with their byte size; no copy is retained here.
use crate::{Domain, Role};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::List;
use temper_engine_domain_forge_change as change;

/// An immutable checkout start, or a branch the worker resolves.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Start {
    /// The project's landing branch.
    Base(Box<[u8]>),
    /// A change's current branch.
    Branch(Box<[u8]>),
    /// The task's saved work takes precedence over its procedure's start.
    Saved(Box<[u8]>),
    /// Resolve this branch against the named base commit.
    Merge { branch: Box<[u8]>, base: api::Commit },
    /// The landing branch with each head merged into it in order
    /// (temper/docs/design/domain/worker/README.md, 4.1).
    Merged { branch: Box<[u8]>, heads: Box<[api::Commit]> },
}
/// Root-translated identities and branch names for one repository.
#[derive(Clone, Debug)]
pub struct Seed {
    /// The run may write its goal's design (temper/docs/design/domain/forge/branches.md, 3).
    pub modify: bool,
    pub task: u64,
    pub parent: Option<u64>,
    pub repository: api::Repository,
    pub run_branch: Box<[u8]>,
    pub saved_branch: Option<Box<[u8]>>,
}
/// A sized workspace item ready to hand to the worker.
#[derive(Debug)]
pub struct Item {
    /// This item delivers a goal's design straight to its goal branch.
    pub delivers: bool,
    pub provider: api::Repository,
    pub name: Box<[u8]>,
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub start: Start,
    pub push: Option<Box<[u8]>>,
    pub landing: Option<Box<[u8]>>,
    pub holder: Option<u64>,
    /// The repository's git identity, which the run's grants carry.
    pub credential: Option<u32>,
    pub bytes: u64,
}
/// A checkout's start, push branch, holder, landing comparison and delivery fact.
type CheckoutSelection = (Start, Option<Box<[u8]>>, Option<u64>, Option<Box<[u8]>>, bool);

impl Domain {
    /// The project's context repositories are added to every checkout read-only.
    #[must_use]
    pub fn context_repositories(&self, project: u32, limit: u32) -> Option<Box<[api::Repository]>> {
        let mut selected = List::with_capacity(limit);
        for (_, repository) in &self.repositories {
            if repository.project == project
                && repository.role == Role::Context
                && !self.forge_ended(repository.provider.forge)
            {
                selected.push(repository.provider).ok()?;
            }
        }
        Some(selected.into_boxed())
    }
    /// A modifying goal waits for its held branch's taking and owned head
    /// (temper/docs/design/domain/forge/branches.md, 3).
    #[must_use]
    pub fn design_workspace_waiting(&self, task: u64) -> bool {
        for (name, hold) in &self.holds {
            if hold.task != task {
                continue;
            }
            let provider = api::Repository { forge: name.forge, repository: name.repository };
            let Some(repository) = self.repository(provider) else { continue };
            if repository.role == Role::Context {
                continue;
            }
            if let Some((_, goal)) = goal_branch(name, &repository.prefix)
                && goal == task
                && self.design_branch_waiting(name)
            {
                return true;
            }
        }
        false
    }

    fn design_branch_waiting(&self, name: &crate::Name) -> bool {
        let undrifted = match self.holds.get(name) {
            Some(hold) => hold.drift.is_none(),
            None => false,
        };
        let owned = match self.heads.get(name) {
            Some(head) => head.owned,
            None => false,
        };
        !undrifted || !owned || self.goal_branch_waiting(name)
    }

    /// Select one checkout item from the task's saved and procedure state.
    #[must_use]
    #[expect(
        clippy::too_many_lines,
        reason = "one workspace item selects its start, destination and byte size together"
    )]
    pub fn workspace_item(&self, seed: Seed) -> Option<Item> {
        let repository = self.repository(seed.repository)?;
        let inherited = match seed.parent {
            Some(parent) => match self.change(parent) {
                Some(row) if row.repository == seed.repository => match row.delegate {
                    Some((child, kind)) if child == seed.task => Some((row, kind)),
                    Some(_) | None => None,
                },
                Some(_) | None => None,
            },
            None => None,
        };
        let (mut start, mut push, holder, selected_landing, delivers) = match inherited {
            Some((row, kind)) => match kind {
                change::Delegate::Produce => {
                    let existing = branch_head(self, seed.repository, &row.branch).is_some();
                    let start = if existing {
                        Start::Branch(row.branch.clone())
                    } else if let change::Early::Started(heads) = &row.change.early {
                        if heads.is_empty() {
                            Start::Base(row.base.clone())
                        } else {
                            let mut commits = List::with_capacity(u32::try_from(heads.len()).ok()?);
                            for head in heads {
                                commits.push(head.head).ok()?;
                            }
                            Start::Merged { branch: row.base.clone(), heads: commits.into_boxed() }
                        }
                    } else {
                        Start::Base(row.base.clone())
                    };
                    (start, Some(row.branch.clone()), Some(row.task), None, false)
                }

                change::Delegate::Close => {
                    (Start::Branch(row.base.clone()), Some(row.branch.clone()), Some(row.task), None, false)
                }
                change::Delegate::Repair(_) => (
                    Start::Branch(row.branch.clone()),
                    Some(row.branch.clone()),
                    Some(self.change_branch_holder(row)),
                    None,
                    false,
                ),
                change::Delegate::Resolve { base } => (
                    Start::Merge {
                        branch: if row.change.last_head.is_some() { row.branch.clone() } else { row.base.clone() },
                        base,
                    },
                    Some(row.branch.clone()),
                    Some(self.change_branch_holder(row)),
                    None,
                    false,
                ),
                change::Delegate::Merge { base, from } => (
                    Start::Merge { branch: if from.is_some() { row.branch.clone() } else { row.base.clone() }, base },
                    Some(row.branch.clone()),
                    Some(self.change_branch_holder(row)),
                    None,
                    false,
                ),
                change::Delegate::Gate { .. } => (Start::Branch(row.branch.clone()), None, None, None, false),
            },
            None => self.task_workspace_item(&seed, repository)?,
        };
        if let Some(saved) = seed.saved_branch {
            start = Start::Saved(saved);
        }
        if let Some((row, change::Delegate::Produce)) = inherited {
            let merge_landing = match &row.change.early {
                change::Early::Started(_) | change::Early::Spent(_) => true,
                change::Early::None => !self.requires(repository, &row.base).up_to_date,
            };
            if merge_landing {
                match &start {
                    Start::Branch(branch) | Start::Saved(branch) => {
                        start = Start::Merge {
                            branch: branch.clone(),
                            base: branch_head(self, seed.repository, &row.base)?,
                        };
                    }
                    Start::Base(_) | Start::Merge { .. } | Start::Merged { .. } => {}
                }
            }
        }
        if repository.role == Role::Context || !repository.kinds.push {
            push = None;
        }
        let landing = match inherited {
            Some((row, change::Delegate::Produce)) => match &push {
                Some(_) => Some(row.base.clone()),
                None => None,
            },
            Some((
                _,
                change::Delegate::Close
                | change::Delegate::Repair(_)
                | change::Delegate::Resolve { .. }
                | change::Delegate::Merge { .. }
                | change::Delegate::Gate { .. },
            )) => None,
            None if delivers && push.is_some() => selected_landing,
            None => match &start {
                Start::Merge { .. } if push.is_some() => selected_landing,
                Start::Base(_) | Start::Branch(_) | Start::Saved(_) | Start::Merge { .. } | Start::Merged { .. } => {
                    None
                }
            },
        };
        let name = repository.name.clone();
        let mut bytes = u64::try_from(size_of::<Item>())
            .ok()?
            .checked_add(skein_lib::len_u64(name.len()))?
            .checked_add(skein_lib::len_u64(repository.host.len()))?
            .checked_add(skein_lib::len_u64(repository.owner.len()))?;
        let branch = match &start {
            Start::Base(branch)
            | Start::Branch(branch)
            | Start::Saved(branch)
            | Start::Merge { branch, .. }
            | Start::Merged { branch, .. } => branch,
        };
        bytes = bytes.checked_add(skein_lib::len_u64(branch.len()))?;
        if let Start::Merged { heads, .. } = &start {
            bytes = bytes.checked_add(
                skein_lib::len_u64(heads.len()).checked_mul(u64::try_from(size_of::<api::Commit>()).ok()?)?,
            )?;
        }
        if let Some(push) = &push {
            bytes = bytes.checked_add(skein_lib::len_u64(push.len()))?;
        }
        if let Some(landing) = &landing {
            bytes = bytes.checked_add(skein_lib::len_u64(landing.len()))?;
        }
        Some(Item {
            delivers: delivers && push.is_some(),
            provider: repository.provider,
            name,
            host: repository.host.clone(),
            owner: repository.owner.clone(),
            start,
            push,
            landing,
            holder,
            credential: match self.records.get(&repository.provider.forge) {
                Some(record) => Some(u32::from(record.number)),
                None => self.identities.get(&repository.provider.forge).copied(),
            },
            bytes,
        })
    }

    /// A modifying goal delivers design straight to its confirmed goal branch;
    /// other tasks start at the repository's base
    /// (temper/docs/design/domain/forge/branches.md, 1).
    fn task_workspace_item(&self, seed: &Seed, repository: &crate::Repository) -> Option<CheckoutSelection> {
        if repository.role != Role::Context {
            for (name, hold) in &self.holds {
                if name.forge != seed.repository.forge
                    || name.repository != seed.repository.repository
                    || hold.task != seed.task
                {
                    continue;
                }
                if let Some((_, task)) = goal_branch(name, &repository.prefix)
                    && task == seed.task
                    && seed.modify
                    && self.design_branch_waiting(name)
                {
                    return None;
                }
                if hold.drift.is_some() {
                    continue;
                }
                if let Some((_, task)) = goal_branch(name, &repository.prefix)
                    && task == seed.task
                    && let Some(head) = self.heads.get(name)
                    && head.owned
                {
                    let branch = branch_bytes(name)?;
                    if seed.modify {
                        let start = match branch_head(self, seed.repository, &seed.run_branch) {
                            Some(base) => Start::Merge { branch: branch.clone(), base },
                            None => Start::Base(branch.clone()),
                        };
                        return Some((start, Some(branch.clone()), None, Some(branch), true));
                    }
                    return Some((Start::Branch(branch), None, None, None, false));
                }
            }
        }
        Some((
            Start::Base(repository.settings.default_branch.clone()),
            Some(seed.run_branch.clone()),
            None,
            None,
            false,
        ))
    }

    /// Repairs and resolutions write under the branch's hold (temper/docs/design/domain/forge/change/README.md, 1).
    fn change_branch_holder(&self, row: &crate::ChangeRow) -> u64 {
        match crate::domain::change_branch_hold(self, row) {
            Some((holder, _)) => holder,
            None => row.task,
        }
    }
}

fn branch_head(domain: &Domain, repository: api::Repository, branch: &[u8]) -> Option<api::Commit> {
    for (name, head) in &domain.heads {
        if name.forge == repository.forge && name.repository == repository.repository && branch_matches(name, branch) {
            return Some(head.commit);
        }
    }
    None
}

pub(crate) fn branch_matches(name: &crate::Name, branch: &[u8]) -> bool {
    let parts = match &name.what {
        crate::What::Branch(parts) => parts,
        crate::What::Repository | crate::What::Pull(_) | crate::What::Issue(_) => return false,
    };
    let mut at = 0_usize;
    for (index, part) in parts.iter().enumerate() {
        if index != 0 {
            if branch.get(at) != Some(&b'/') {
                return false;
            }
            let Some(next) = at.checked_add(1) else { return false };
            at = next;
        }
        let Some(end) = at.checked_add(part.len()) else { return false };
        if branch.get(at..end) != Some(part.as_ref()) {
            return false;
        }
        at = end;
    }
    at == branch.len()
}

/// Compare the prefix without allocating a joined branch name.
pub(crate) fn branch_starts_with(name: &crate::Name, prefix: &[u8]) -> bool {
    let parts = match &name.what {
        crate::What::Branch(parts) => parts,
        crate::What::Repository | crate::What::Pull(_) | crate::What::Issue(_) => return false,
    };
    let mut at = 0_usize;
    for (index, part) in parts.iter().enumerate() {
        if index != 0 {
            if at == prefix.len() {
                return true;
            }
            if prefix.get(at) != Some(&b'/') {
                return false;
            }
            at = at.checked_add(1).expect("prefix offset bounded by prefix length");
        }
        for byte in part {
            if at == prefix.len() {
                return true;
            }
            if prefix.get(at) != Some(byte) {
                return false;
            }
            at = at.checked_add(1).expect("prefix offset bounded by prefix length");
        }
    }
    at == prefix.len()
}

/// The goal's own branch (temper/docs/design/domain/forge/branches.md, 1).
pub(crate) fn goal_branch(name: &crate::Name, prefix: &[u8]) -> Option<(u64, u64)> {
    task_branch(name, prefix, b"g")
}

pub(crate) fn change_branch(name: &crate::Name, prefix: &[u8]) -> Option<(u64, u64)> {
    task_branch(name, prefix, b"c")
}

pub(crate) fn run_branch(name: &crate::Name, prefix: &[u8]) -> Option<(u64, u64)> {
    task_branch(name, prefix, b"r")
}

fn task_branch(name: &crate::Name, prefix: &[u8], letter: &[u8]) -> Option<(u64, u64)> {
    let branch = branch_bytes(name)?;
    let own = branch.strip_prefix(prefix)?;
    let slash = skein_lib::bytes::find(own, b"/")?;
    let root = decimal(own.get(..slash)?)?;
    let task = own.get(slash.checked_add(1)?..)?.strip_prefix(letter)?;
    Some((root, decimal(task)?))
}

fn branch_bytes(name: &crate::Name) -> Option<Box<[u8]>> {
    let parts = match &name.what {
        crate::What::Branch(parts) => parts,
        crate::What::Repository | crate::What::Pull(_) | crate::What::Issue(_) => return None,
    };
    let mut length = 0_u32;
    for (index, part) in parts.iter().enumerate() {
        if index != 0 {
            length = length.checked_add(1)?;
        }
        length = length.checked_add(u32::try_from(part.len()).ok()?)?;
    }
    let mut branch = List::with_capacity(length);
    for (index, part) in parts.iter().enumerate() {
        if index != 0 {
            branch.push(b'/').ok()?;
        }
        for byte in part {
            branch.push(*byte).ok()?;
        }
    }
    Some(branch.into_boxed())
}

fn decimal(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() || (bytes.len() != 1 && bytes.first() == Some(&b'0')) {
        return None;
    }
    let mut number = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
