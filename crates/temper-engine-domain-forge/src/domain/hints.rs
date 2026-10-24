//! Procedure hint dependencies, independent of the client's keep reads
//! (domain/forge.md, 5.2). Unknown facts cannot establish irrelevance.
use super::Domain;
use crate::{ChangeRow, Topic, What};
use alloc::boxed::Box;
use skein_forge::api::{Change, Commit, Hint};

pub(super) fn concerns(d: &Domain, row: &ChangeRow, hint: &Hint) -> bool {
    let own = match &hint.change {
        Change::Branch(branch) | Change::BranchMoved { branch, .. } => {
            branch.is_empty() || *branch == row.branch || *branch == row.base
        }
        Change::Commit(head) => commit(d, row, *head),
        Change::Item(number) => item(d, row, *number),
    };
    own || dependencies(d, row.task, hint)
}

fn item(d: &Domain, row: &ChangeRow, number: u64) -> bool {
    // A pull may open before its outcome reaches this owner. Reviews name
    // that same item, and cannot be excluded before the pull is known.
    if number == 0 || row.pull.is_none() || row.pull == Some(number) {
        return true;
    }
    if let Some(issue) = d.issues.get(&row.task)
        && issue.number == Some(number)
    {
        return true;
    }
    for (_, pending) in &d.steps {
        if pending.task == row.task
            && let Some(pull) = &pending.pull
            && pull.number == number
        {
            return true;
        }
    }
    false
}

fn commit(d: &Domain, row: &ChangeRow, head: Commit) -> bool {
    if head == [0; 32] || row.change.last_head.is_none() || row.change.last_head == Some(head) {
        return true;
    }
    let base = match d.settling.get(&row.task) {
        Some(settle) => {
            if settle.uses_base(head) {
                return true;
            }
            settle.read_base()
        }
        None => None,
    };
    if base.is_none() || base == Some(head) {
        return true;
    }
    // A fresh chain may already know a newer head, base or pull than its
    // saved row. A hint about those facts must still restart that chain.
    for (_, pending) in &d.steps {
        if pending.task != row.task {
            continue;
        }
        if pending.branch == Some(head) || pending.base_tip == Some(head) {
            return true;
        }
        if let Some(pull) = &pending.pull
            && (pull.commit == head || pull.base_commit == Some(head))
        {
            return true;
        }
    }
    for (name, pull) in &d.pulls {
        if name.forge != row.repository.forge || name.repository != row.repository.repository || pull.head != head {
            continue;
        }
        match name.what {
            What::Pull(number) if row.pull == Some(number) => return true,
            What::Pull(_) | What::Issue(_) | What::Branch(_) | What::Repository => {}
        }
    }
    for (name, observed) in &d.heads {
        if name.forge != row.repository.forge || name.repository != row.repository.repository || observed.commit != head
        {
            continue;
        }
        match &name.what {
            What::Branch(parts) if branch_eq(parts, &row.branch) || branch_eq(parts, &row.base) => return true,
            What::Branch(_) | What::Pull(_) | What::Issue(_) | What::Repository => {}
        }
    }
    false
}

/// Extra named resources and subscriptions can also be procedure inputs.
fn dependencies(d: &Domain, task: u64, hint: &Hint) -> bool {
    if let Some(names) = d.names.get(&task) {
        for name in names {
            if name.forge != hint.repository.forge || name.repository != hint.repository.repository {
                continue;
            }
            let named = match &name.what {
                What::Pull(number) | What::Issue(number) => match &hint.change {
                    Change::Item(item) => number == item,
                    Change::Commit(head) => match d.pulls.get(name) {
                        Some(pull) => pull.head == *head,
                        None => false,
                    },
                    Change::Branch(_) | Change::BranchMoved { .. } => false,
                },
                What::Branch(parts) => match &hint.change {
                    Change::Branch(branch) | Change::BranchMoved { branch, .. } => branch_eq(parts, branch),
                    Change::Commit(head) => match d.heads.get(name) {
                        Some(observed) => observed.commit == *head,
                        None => false,
                    },
                    Change::Item(_) => false,
                },
                What::Repository => false,
            };
            if named {
                return true;
            }
        }
    }
    for (_, subscriber) in &d.subscriptions {
        if subscriber.task != task {
            continue;
        }
        let (repository, named) = match &subscriber.topic {
            Topic::Pull { repository, number } | Topic::Participation { repository, number } => {
                let named = match &hint.change {
                    Change::Item(item) => number == item,
                    Change::Branch(_) | Change::BranchMoved { .. } | Change::Commit(_) => false,
                };
                (*repository, named)
            }
            Topic::Ci { repository, head } => {
                let named = match &hint.change {
                    Change::Commit(commit) => head == commit,
                    Change::Branch(_) | Change::BranchMoved { .. } | Change::Item(_) => false,
                };
                (*repository, named)
            }
            Topic::Landings { repository, branch } => {
                let named = match &hint.change {
                    Change::Branch(name) | Change::BranchMoved { branch: name, .. } => branch == name,
                    Change::Item(_) | Change::Commit(_) => false,
                };
                (*repository, named)
            }
        };
        if repository == hint.repository && named {
            return true;
        }
    }
    false
}

/// Compare a parsed resource name with a provider branch without allocating.
fn branch_eq(parts: &[Box<[u8]>], branch: &[u8]) -> bool {
    let mut rest = branch;
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            let Some(tail) = rest.strip_prefix(b"/") else { return false };
            rest = tail;
        }
        let Some(tail) = rest.strip_prefix(part.as_ref()) else { return false };
        rest = tail;
    }
    rest.is_empty()
}
