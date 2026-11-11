//! Fixed sequences, never retries (protocol/git.md, section 4).
use crate::{
    invocation::{self, Command, Source},
    output::{self, Status},
};
use alloc::boxed::Box;
use temper_worker_domain_checkout::{self as checkout, git};

/// The next step after an invocation has settled and closed.
#[derive(Clone, PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "bounded diagnostics stay inline as in git::Done")]
pub enum Next {
    Run(Command),
    Snapshot { parent: git::Commit, merging: Option<git::Commit> },
    Make,
    Store,
    InstallStore { directory: Box<[u8]>, done: Box<git::Done> },
    Alternates,
    Scan { files: Box<[Box<[u8]>]> },
    End(git::Done),
}
/// The first step depends only on the operation.
#[must_use]
pub fn first(op: &git::Op) -> Next {
    Next::Run(match op {
        git::Op::Make { .. } => return Next::Make,
        git::Op::Store { .. } => return Next::Store,
        git::Op::Look { at, .. } => Command::Origin { directory: invocation::place(at) },
        git::Op::Clone { .. } => Command::Format,
        git::Op::Fetch { at, remote, want, into, .. } => Command::Fetch {
            workspace: at.workspace,
            into: *into,
            remote: remote.clone(),
            source: match want {
                git::Want::Branch { branch } => Source::Branch(branch.clone()),
                git::Want::Commit { commit } => Source::Commit(*commit),
                git::Want::Default => Source::Default,
            },
        },
        git::Op::Create { remote, branch, commit, .. } => {
            Command::Create { remote: remote.clone(), branch: branch.clone(), commit: *commit }
        }
        git::Op::Point { into, commit, .. } => Command::UpdateRef { into: *into, commit: *commit },
        git::Op::CheckOut { commit, .. } => Command::CheckOut { commit: *commit },
        git::Op::IsAncestor { ancestor, commit, .. } => Command::IsAncestor { expected: *ancestor, commit: *commit },
        git::Op::DiffPaths { before, after, .. } => Command::DiffPaths { before: *before, after: *after },
        git::Op::Merge { theirs, .. } => Command::Merge { theirs: *theirs },
        git::Op::Commit { snapshot: Some(_), .. } => Command::SnapshotHead,
        git::Op::Commit { merging, snapshot: None, .. } => {
            if merging.is_some() {
                Command::Unmerged
            } else {
                Command::Add
            }
        }
        git::Op::Push { remote, commit, branch, expected, .. } => match expected {
            Some(expected) => Command::IsAncestor { expected: *expected, commit: *commit },
            None => Command::Push { remote: remote.clone(), commit: *commit, branch: branch.clone(), expected: None },
        },
    })
}
/// Fault and diagnostic from the invocation that failed.
#[must_use]
pub fn failed(fault: git::Fault, tail: &[u8], cut: u64) -> Next {
    Next::End(git::Done::FailedWithOutput { fault, diagnostic: git::PushDiagnostic::new(tail, cut) })
}
/// Decide by exit status and stdout first, otherwise by the failure tail.
#[must_use]
#[expect(clippy::too_many_lines, reason = "the invocation terminal table handles every operation together")]
pub fn after(
    op: &git::Op,
    ran: &Command,
    code: u8,
    stdout: &[u8],
    tail: &[u8],
    cut: u64,
    checkout: &checkout::Limits,
) -> Next {
    if let Some(next) = crate::snapshot::after(op, ran, code, stdout) {
        return next;
    }
    match ran {
        Command::Origin { .. } => match op {
            git::Op::Look { remote, .. } => match code {
                0 => {
                    return Next::End(git::Done::Looked { holds: stdout.strip_suffix(b"\n") == Some(remote.as_ref()) });
                }
                1 | 128 => return Next::End(git::Done::Looked { holds: false }),
                _ => {}
            },
            git::Op::Store { .. }
            | git::Op::Make { .. }
            | git::Op::Clone { .. }
            | git::Op::Point { .. }
            | git::Op::Fetch { .. }
            | git::Op::Create { .. }
            | git::Op::CheckOut { .. }
            | git::Op::DiffPaths { .. }
            | git::Op::IsAncestor { .. }
            | git::Op::Merge { .. }
            | git::Op::Commit { .. }
            | git::Op::Push { .. } => unreachable!("origin is a look's invocation"),
        },
        Command::Format if code == 0 => {
            let format = match stdout {
                b"sha1\n" => Some(invocation::Format::Sha1),
                b"sha256\n" => Some(invocation::Format::Sha256),
                other => {
                    let _length = other.len();
                    None
                }
            };
            if let Some(format) = format {
                let git::Op::Clone { at, .. } = op else { unreachable!("format is a clone's first invocation") };
                return Next::Run(Command::Clone { directory: at.repository.clone(), format });
            }
        }
        Command::Store { directory, .. } => {
            let done = if code == 0 {
                git::Done::Succeeded
            } else {
                git::Done::FailedWithOutput {
                    fault: output::classify(tail, git::Kind::Store),
                    diagnostic: git::PushDiagnostic::new(tail, cut),
                }
            };
            return Next::InstallStore { directory: directory.clone(), done: Box::new(done) };
        }
        Command::Clone { .. } if code == 0 => return Next::Alternates,
        Command::Remote { .. } | Command::Clean if code == 0 => return Next::End(git::Done::Succeeded),
        Command::Fetch { into, workspace, .. } if code == 0 => {
            if let Some(commit) = output::fetched(stdout, &invocation::store_reference(*workspace, *into)) {
                return Next::Run(Command::UpdateRef { into: *into, commit });
            }
        }
        Command::UpdateRef { commit, .. } if code == 0 => {
            return Next::End(if op.kind() == git::Kind::Point {
                git::Done::Succeeded
            } else {
                git::Done::Fetched { commit: *commit }
            });
        }
        Command::Create { branch, .. } => {
            if let Some(next) = pushed(stdout, branch, true, tail, cut) {
                return next;
            }
        }
        Command::Push { branch, .. } => {
            if let Some(next) = pushed(stdout, branch, false, tail, cut) {
                return next;
            }
        }
        Command::CheckOut { .. } if code == 0 => return checked_out(op),
        Command::Merge { .. } => match code {
            0 => return Next::End(git::Done::Merged),
            1 => return Next::Run(Command::Unmerged),
            _ => {}
        },
        Command::DiffPaths { .. } if code == 0 => {
            if let Some(files) = output::design_paths(stdout, checkout) {
                return Next::End(git::Done::Paths { files });
            }
        }
        Command::Unmerged if code == 0 => {
            if let Some(next) = unmerged(op, stdout, checkout) {
                return next;
            }
        }
        Command::Add if code == 0 => match op {
            git::Op::Commit { parent, merging, always, .. } => {
                return Next::Run(if merging.is_some() || *always {
                    Command::WriteTree
                } else {
                    Command::DiffIndex { parent: *parent }
                });
            }
            git::Op::Store { .. }
            | git::Op::Make { .. }
            | git::Op::Clone { .. }
            | git::Op::Point { .. }
            | git::Op::Fetch { .. }
            | git::Op::Create { .. }
            | git::Op::CheckOut { .. }
            | git::Op::Look { .. }
            | git::Op::DiffPaths { .. }
            | git::Op::IsAncestor { .. }
            | git::Op::Merge { .. }
            | git::Op::Push { .. } => {}
        },
        Command::DiffIndex { parent } => match code {
            0 => {
                return Next::End(if crate::snapshot::enabled(op) {
                    git::Done::Committed { commit: *parent }
                } else {
                    git::Done::Unchanged
                });
            }
            1 => return Next::Run(Command::WriteTree),
            _ => {}
        },
        Command::WriteTree if code == 0 => {
            if let Some(command) = commit_tree(op, stdout) {
                return Next::Run(command);
            }
        }
        Command::CommitTree { .. } if code == 0 => {
            if let Some(commit) = id(stdout) {
                return if crate::snapshot::enabled(op) {
                    Next::Run(Command::SnapshotReset { commit })
                } else {
                    Next::End(git::Done::Committed { commit })
                };
            }
        }
        Command::IsAncestor { .. } => match op {
            git::Op::IsAncestor { .. } => {
                if code <= 1 {
                    return Next::End(git::Done::IsAncestor { contains: code == 0 });
                }
                return failed(git::Fault::Broken, tail, cut);
            }
            git::Op::Push { remote, commit, branch, expected, .. } => match code {
                1 => return Next::End(git::Done::Rejected),
                0 => {
                    return Next::Run(Command::Push {
                        remote: remote.clone(),
                        commit: *commit,
                        branch: branch.clone(),
                        expected: *expected,
                    });
                }
                _ => {}
            },
            git::Op::DiffPaths { .. }
            | git::Op::Look { .. }
            | git::Op::Store { .. }
            | git::Op::Make { .. }
            | git::Op::Clone { .. }
            | git::Op::Point { .. }
            | git::Op::Fetch { .. }
            | git::Op::Create { .. }
            | git::Op::CheckOut { .. }
            | git::Op::Merge { .. }
            | git::Op::Commit { .. } => {}
        },
        Command::SnapshotHead
        | Command::SnapshotMerge { .. }
        | Command::SnapshotFetch { .. }
        | Command::SnapshotReachable { .. }
        | Command::SnapshotReset { .. }
        | Command::Version
        | Command::Format
        | Command::Clone { .. }
        | Command::Remote { .. }
        | Command::UpdateRef { .. }
        | Command::Fetch { .. }
        | Command::CheckOut { .. }
        | Command::Clean
        | Command::DiffPaths { .. }
        | Command::Unmerged
        | Command::Add
        | Command::WriteTree
        | Command::CommitTree { .. } => {}
    }
    let fault = if code == 0 || !remote(ran) { git::Fault::Broken } else { output::classify(tail, op.kind()) };
    failed(fault, tail, cut)
}
fn id(stdout: &[u8]) -> Option<git::Commit> {
    invocation::object(stdout.strip_suffix(b"\n")?)
}
fn pushed(stdout: &[u8], branch: &[u8], create: bool, tail: &[u8], cut: u64) -> Option<Next> {
    let status = output::status(stdout, branch)?;
    let done = match status {
        Status::NewBranch => git::Done::Succeeded,
        Status::UpToDate => {
            if create {
                git::Done::Exists
            } else {
                git::Done::Succeeded
            }
        }
        Status::StaleInfo => {
            if create {
                git::Done::Exists
            } else {
                git::Done::Rejected
            }
        }
        Status::FastForward if !create => git::Done::Succeeded,
        Status::NonFastForward | Status::FetchFirst if !create => git::Done::Rejected,
        Status::Declined => return Some(failed(git::Fault::Refused, tail, cut)),
        Status::FastForward | Status::NonFastForward | Status::FetchFirst | Status::Other => return None,
    };
    Some(Next::End(done))
}
fn unmerged(op: &git::Op, stdout: &[u8], checkout: &checkout::Limits) -> Option<Next> {
    match op {
        git::Op::Merge { .. } => {
            let files = output::conflict_paths(stdout, checkout)?;
            if files.is_empty() { None } else { Some(Next::End(git::Done::Conflicted { files })) }
        }
        git::Op::Commit { .. } => {
            let files = output::paths(stdout, checkout)?;
            Some(if files.is_empty() { Next::Run(Command::Add) } else { Next::Scan { files } })
        }
        git::Op::Store { .. }
        | git::Op::Make { .. }
        | git::Op::Clone { .. }
        | git::Op::Point { .. }
        | git::Op::Fetch { .. }
        | git::Op::Create { .. }
        | git::Op::CheckOut { .. }
        | git::Op::Look { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::IsAncestor { .. }
        | git::Op::Push { .. } => None,
    }
}
fn commit_tree(op: &git::Op, stdout: &[u8]) -> Option<Command> {
    let tree = id(stdout)?;
    match op {
        git::Op::Commit { parent, merging, .. } => {
            Some(Command::CommitTree { tree, parent: *parent, merging: *merging })
        }
        git::Op::Store { .. }
        | git::Op::Make { .. }
        | git::Op::Clone { .. }
        | git::Op::Point { .. }
        | git::Op::Fetch { .. }
        | git::Op::Create { .. }
        | git::Op::CheckOut { .. }
        | git::Op::Look { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::IsAncestor { .. }
        | git::Op::Merge { .. }
        | git::Op::Push { .. } => None,
    }
}

fn checked_out(op: &git::Op) -> Next {
    match op {
        git::Op::CheckOut { clean: false, .. } => Next::End(git::Done::Succeeded),
        git::Op::CheckOut { clean: true, .. }
        | git::Op::Store { .. }
        | git::Op::Make { .. }
        | git::Op::Clone { .. }
        | git::Op::Point { .. }
        | git::Op::Fetch { .. }
        | git::Op::Create { .. }
        | git::Op::Look { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::IsAncestor { .. }
        | git::Op::Merge { .. }
        | git::Op::Commit { .. }
        | git::Op::Push { .. } => Next::Run(Command::Clean),
    }
}

fn remote(command: &Command) -> bool {
    match command {
        Command::Store { .. } | Command::Fetch { .. } | Command::Create { .. } | Command::Push { .. } => true,
        Command::Format
        | Command::Version
        | Command::Origin { .. }
        | Command::Clone { .. }
        | Command::Remote { .. }
        | Command::UpdateRef { .. }
        | Command::SnapshotHead
        | Command::SnapshotMerge { .. }
        | Command::SnapshotFetch { .. }
        | Command::SnapshotReachable { .. }
        | Command::SnapshotReset { .. }
        | Command::CheckOut { .. }
        | Command::Clean
        | Command::Merge { .. }
        | Command::Unmerged
        | Command::Add
        | Command::DiffIndex { .. }
        | Command::DiffPaths { .. }
        | Command::WriteTree
        | Command::CommitTree { .. }
        | Command::IsAncestor { .. } => false,
    }
}
