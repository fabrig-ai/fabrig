//! A delivery preserves the checkout's ancestry (temper/docs/design/protocol/git.md, 4.7).
use crate::{
    invocation::{self, Command},
    sequence::{self, Next},
};
use skein_lib::bytes;
use temper_worker_domain_checkout::git;

pub(crate) fn enabled(op: &git::Op) -> bool {
    match op {
        git::Op::Commit { snapshot, .. } => snapshot.is_some(),
        git::Op::Store { .. }
        | git::Op::Make { .. }
        | git::Op::Look { .. }
        | git::Op::Clone { .. }
        | git::Op::Reds { .. }
        | git::Op::Point { .. }
        | git::Op::Fetch { .. }
        | git::Op::Create { .. }
        | git::Op::CheckOut { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::Brings { .. }
        | git::Op::IsAncestor { .. }
        | git::Op::Merge { .. }
        | git::Op::Push { .. } => false,
    }
}

fn id(stdout: &[u8]) -> Option<git::Commit> {
    invocation::object(stdout.strip_suffix(b"\n")?)
}

fn refused(message: &[u8]) -> Next {
    sequence::failed(git::Fault::Refused, message, 0)
}

pub(crate) fn after(op: &git::Op, command: &Command, code: u8, stdout: &[u8]) -> Option<Next> {
    match command {
        Command::SnapshotHead if code == 0 => {
            let line = stdout.strip_suffix(b"\n")?;
            let split = bytes::find(line, b" ")?;
            let head = invocation::object(line.get(..split)?)?;
            let parents = line.get(split.checked_add(1)?..)?;
            let second = match bytes::find(parents, b" ") {
                Some(split) => {
                    invocation::object(parents.get(..split)?)?;
                    let second = parents.get(split.checked_add(1)?..)?;
                    if second.contains(&b' ') {
                        return Some(refused(b"delivery supports at most two merge parents"));
                    }
                    Some(invocation::object(second)?)
                }
                None => {
                    if !parents.is_empty() {
                        invocation::object(parents)?;
                    }
                    None
                }
            };
            Some(Next::Run(Command::SnapshotMerge { head, second }))
        }
        Command::SnapshotMerge { head, second } if code <= 1 => {
            let (parent, merging, snapshot) = parameters(op);
            let merging = if code == 0 {
                if stdout.strip_suffix(b"\n")?.contains(&b'\n') {
                    return Some(refused(b"delivery supports at most two merge parents"));
                }
                Some(id(stdout)?)
            } else if stdout.is_empty() {
                if head == parent { *merging } else { None }
            } else {
                return None;
            };
            match merging.or(*second) {
                Some(second) => Some(Next::Run(Command::SnapshotFetch {
                    remote: snapshot.remote.clone(),
                    head: *head,
                    merging,
                    second,
                })),
                None => Some(Next::Snapshot { parent: *head, merging }),
            }
        }
        Command::SnapshotFetch { head, merging, second, .. } if code == 0 => {
            Some(Next::Run(Command::SnapshotReachable { head: *head, merging: *merging, second: *second }))
        }
        Command::SnapshotReachable { head, merging, .. } if code == 0 => {
            if stdout.is_empty() {
                Some(refused(b"delivery second parent is not reachable from the branch's remote"))
            } else {
                id(stdout)?;
                Some(Next::Snapshot { parent: *head, merging: *merging })
            }
        }
        Command::SnapshotReset { commit } if code == 0 => Some(Next::End(git::Done::Committed { commit: *commit })),
        Command::Reds(_)
        | Command::SnapshotHead
        | Command::SnapshotMerge { .. }
        | Command::SnapshotFetch { .. }
        | Command::SnapshotReachable { .. }
        | Command::SnapshotReset { .. }
        | Command::Version
        | Command::Origin { .. }
        | Command::Format
        | Command::Clone { .. }
        | Command::Remote { .. }
        | Command::Store { .. }
        | Command::UpdateRef { .. }
        | Command::Fetch { .. }
        | Command::Create { .. }
        | Command::CheckOut { .. }
        | Command::Clean
        | Command::Merge { .. }
        | Command::Unmerged
        | Command::Add
        | Command::DiffPaths { .. }
        | Command::DiffIndex { .. }
        | Command::WriteTree
        | Command::CommitTree { .. }
        | Command::MergeBases { .. }
        | Command::BringsList { .. }
        | Command::IsAncestor { .. }
        | Command::Push { .. } => None,
    }
}

fn parameters(op: &git::Op) -> (&git::Commit, &Option<git::Commit>, &git::Snapshot) {
    match op {
        git::Op::Commit { parent, merging, snapshot: Some(snapshot), .. } => (parent, merging, snapshot),
        git::Op::Commit { snapshot: None, .. }
        | git::Op::Store { .. }
        | git::Op::Make { .. }
        | git::Op::Look { .. }
        | git::Op::Clone { .. }
        | git::Op::Reds { .. }
        | git::Op::Point { .. }
        | git::Op::Fetch { .. }
        | git::Op::Create { .. }
        | git::Op::CheckOut { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::Brings { .. }
        | git::Op::IsAncestor { .. }
        | git::Op::Merge { .. }
        | git::Op::Push { .. } => unreachable!("snapshot sequence belongs to a delivery"),
    }
}
