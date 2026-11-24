//! Fixed git sequences and the run's disposable view (protocol/git-operations.md, section 1).
use crate::{
    invocation::{self, Command, Source},
    output::{self, Status},
};
use alloc::boxed::Box;
use temper_worker_domain_checkout::{self as checkout, git};

/// Pack bound (temper/docs/design/protocol/git-operations.md, 1.14).
const PACKS: u32 = 50;

/// The next step after an invocation has settled and closed.
#[derive(Clone, PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "bounded diagnostics stay inline as in git::Done")]
pub enum Next {
    Run(Command),
    Reds { directory: Box<[u8]>, content: Option<Box<[u8]>>, marker: bool, rerun: Option<u32> },
    Results { directory: Box<[u8]> },
    Snapshot { parent: git::Commit, merging: Option<git::Commit> },
    Keep,
    Make,
    Look { directory: Box<[u8]> },
    Store,
    InstallStore { directory: Box<[u8]>, done: Box<git::Done> },
    Alternates { format: invocation::Format },
    View { done: Box<git::Done>, retry: bool },
    InitView { done: Box<git::Done>, format: Option<invocation::Format> },
    RemakeView { done: Box<git::Done>, format: Option<invocation::Format> },
    Scan { files: Box<[Box<[u8]>]>, cut: bool },
    End(git::Done),
}
/// The first step depends only on the operation.
#[must_use]
pub fn first(op: &git::Op) -> Next {
    Next::Run(match op {
        git::Op::Reds { .. } => return crate::reds::first(op),
        git::Op::Keep { .. } => return Next::Keep,
        git::Op::Make { .. } => return Next::Make,
        git::Op::Store { .. } => return Next::Store,
        git::Op::Look { at, .. } => Command::Origin { directory: invocation::git_dir(at) },
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
        git::Op::Counts { at, remote, .. } => Command::FetchCounts { remote: remote.clone(), workspace: at.workspace },
        git::Op::Create { remote, branch, commit, .. } => {
            Command::Create { remote: remote.clone(), branch: branch.clone(), commit: *commit }
        }
        git::Op::Reset { commit, .. } => Command::SnapshotReset { commit: *commit },
        git::Op::Pack { store, .. } => Command::CountObjects { store: *store },
        git::Op::Point { into, commit, .. } => Command::UpdateRef { into: *into, commit: *commit },
        git::Op::CheckOut { commit, .. } => Command::CheckOut { commit: *commit },
        git::Op::Brings { head, upstream, .. } => Command::MergeBases { head: *head, upstream: *upstream },
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
    if let Command::Reds(check) = ran {
        return crate::reds::after(op, check, code, stdout);
    }
    if let Some(next) = crate::snapshot::after(op, ran, code, stdout) {
        return next;
    }
    match ran {
        Command::CountObjects { store } => {
            if code == 0
                && let Some(packs) = output::packs(stdout)
            {
                return if packs > PACKS { Next::Run(Command::Repack { store: *store }) } else { packed(*store) };
            }
        }
        Command::Repack { store } => {
            if code == 0 {
                return packed(*store);
            }
        }
        Command::Reds(_) => unreachable!("reds handled above"),
        Command::MergeBases { .. } => return merge_bases(op, code, stdout, tail, cut),
        Command::BringsList { .. } => {
            if code == 0 && (stdout.is_empty() || id(stdout).is_some()) {
                return Next::End(git::Done::Brings { brings: !stdout.is_empty() });
            }
            return failed(git::Fault::Broken, tail, cut);
        }
        Command::Origin { .. } => match op {
            git::Op::Look { at, remote, .. } => match code {
                0 => {
                    return Next::End(git::Done::Looked { holds: stdout.strip_suffix(b"\n") == Some(remote.as_ref()) });
                }
                1 => return Next::End(git::Done::Looked { holds: false }),
                128 => return Next::Look { directory: invocation::place(at) },
                _ => {}
            },
            git::Op::Store { .. }
            | git::Op::Keep { .. }
            | git::Op::Make { .. }
            | git::Op::Clone { .. }
            | git::Op::Reds { .. }
            | git::Op::Reset { .. }
            | git::Op::Pack { .. }
            | git::Op::Point { .. }
            | git::Op::Fetch { .. }
            | git::Op::Counts { .. }
            | git::Op::Create { .. }
            | git::Op::CheckOut { .. }
            | git::Op::DiffPaths { .. }
            | git::Op::Brings { .. }
            | git::Op::IsAncestor { .. }
            | git::Op::Merge { .. }
            | git::Op::Commit { .. }
            | git::Op::Push { .. } => unreachable!("origin is a look's invocation"),
        },
        Command::Format if code == 0 => {
            let format = format(stdout);
            if let Some(format) = format {
                let (git::Op::Clone { at, .. } | git::Op::Look { at, .. }) = op else {
                    unreachable!("format belongs to a worker's directory being made")
                };
                return Next::Run(Command::Clone { directory: invocation::git_dir(at), format, bare: true });
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
        Command::Clone { format, .. } if code == 0 => return Next::Alternates { format: *format },
        Command::Remote { format, .. } if code == 0 => {
            if let git::Op::Look { .. } = op {
                return Next::End(git::Done::Looked { holds: true });
            }
            return Next::RemakeView { done: Box::new(git::Done::Succeeded), format: Some(*format) };
        }
        Command::Clean | Command::DeleteCounts if code == 0 => return Next::End(git::Done::Succeeded),
        Command::Fetch { into, workspace, .. } if code == 0 => {
            if let Some(commit) = output::fetched(stdout, &invocation::store_reference(*workspace, *into)) {
                return Next::Run(Command::UpdateRef { into: *into, commit });
            }
        }
        Command::FetchCounts { workspace, .. } => {
            return if code == 0 {
                match output::fetched(stdout, &invocation::counts_store_reference(*workspace)) {
                    Some(commit) => Next::Run(Command::UpdateCounts { commit }),
                    None => failed(git::Fault::Broken, tail, cut),
                }
            } else {
                match output::counts_failure(tail, cut) {
                    Ok(()) => Next::Run(Command::DeleteCounts),
                    Err(fault) => failed(fault, tail, cut),
                }
            };
        }
        Command::UpdateCounts { commit } if code == 0 => return Next::End(git::Done::Fetched { commit: *commit }),
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
            let git::Op::DiffPaths { markdown, .. } = op else { unreachable!("path comparison") };
            if let Some(files) = output::design_paths(stdout, checkout, *markdown) {
                return Next::End(git::Done::Paths { files });
            }
        }
        Command::UpdateIndex { .. } if code == 0 => return Next::Run(Command::Unmerged),
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
            | git::Op::Keep { .. }
            | git::Op::Make { .. }
            | git::Op::Clone { .. }
            | git::Op::Reds { .. }
            | git::Op::Reset { .. }
            | git::Op::Pack { .. }
            | git::Op::Point { .. }
            | git::Op::Fetch { .. }
            | git::Op::Counts { .. }
            | git::Op::Create { .. }
            | git::Op::CheckOut { .. }
            | git::Op::Look { .. }
            | git::Op::DiffPaths { .. }
            | git::Op::Brings { .. }
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
            git::Op::Brings { .. }
            | git::Op::DiffPaths { .. }
            | git::Op::Look { .. }
            | git::Op::Store { .. }
            | git::Op::Keep { .. }
            | git::Op::Make { .. }
            | git::Op::Clone { .. }
            | git::Op::Reds { .. }
            | git::Op::Reset { .. }
            | git::Op::Pack { .. }
            | git::Op::Point { .. }
            | git::Op::Fetch { .. }
            | git::Op::Counts { .. }
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
        | Command::UpdateCounts { .. }
        | Command::DeleteCounts
        | Command::Fetch { .. }
        | Command::CheckOut { .. }
        | Command::Clean
        | Command::DiffPaths { .. }
        | Command::Unmerged
        | Command::UpdateIndex { .. }
        | Command::Add
        | Command::WriteTree
        | Command::CommitTree { .. } => {}
    }
    let fault = if code == 0 || !remote(ran) { git::Fault::Broken } else { output::classify(tail, op.kind()) };
    failed(fault, tail, cut)
}
fn packed(store: Option<skein_lib::Token>) -> Next {
    match store {
        Some(_) => Next::Run(Command::CountObjects { store: None }),
        None => Next::End(git::Done::Succeeded),
    }
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
    let files = output::conflict_paths(stdout, checkout)?;
    let mut kept = 0_usize;
    for file in &files {
        kept = kept.checked_add(file.len())?.checked_add(1)?;
    }
    let cut = kept != stdout.len();
    match op {
        git::Op::Merge { .. } => {
            if files.is_empty() {
                return None;
            }
            Some(Next::End(git::Done::Conflicted { cut, files }))
        }
        git::Op::Commit { .. } => {
            if files.is_empty() {
                return if cut { None } else { Some(Next::Run(Command::Add)) };
            }
            Some(Next::Scan { files, cut })
        }
        git::Op::Store { .. }
        | git::Op::Keep { .. }
        | git::Op::Make { .. }
        | git::Op::Clone { .. }
        | git::Op::Reds { .. }
        | git::Op::Reset { .. }
        | git::Op::Pack { .. }
        | git::Op::Point { .. }
        | git::Op::Fetch { .. }
        | git::Op::Counts { .. }
        | git::Op::Create { .. }
        | git::Op::CheckOut { .. }
        | git::Op::Look { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::Brings { .. }
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
        | git::Op::Keep { .. }
        | git::Op::Make { .. }
        | git::Op::Clone { .. }
        | git::Op::Reds { .. }
        | git::Op::Reset { .. }
        | git::Op::Pack { .. }
        | git::Op::Point { .. }
        | git::Op::Fetch { .. }
        | git::Op::Counts { .. }
        | git::Op::Create { .. }
        | git::Op::CheckOut { .. }
        | git::Op::Look { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::Brings { .. }
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
        | git::Op::Keep { .. }
        | git::Op::Make { .. }
        | git::Op::Clone { .. }
        | git::Op::Reds { .. }
        | git::Op::Reset { .. }
        | git::Op::Pack { .. }
        | git::Op::Point { .. }
        | git::Op::Fetch { .. }
        | git::Op::Counts { .. }
        | git::Op::Create { .. }
        | git::Op::Look { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::Brings { .. }
        | git::Op::IsAncestor { .. }
        | git::Op::Merge { .. }
        | git::Op::Commit { .. }
        | git::Op::Push { .. } => Next::Run(Command::Clean),
    }
}

fn remote(command: &Command) -> bool {
    match command {
        Command::Store { .. }
        | Command::Fetch { .. }
        | Command::FetchCounts { .. }
        | Command::Create { .. }
        | Command::Push { .. }
        | Command::CheckOut { .. }
        | Command::Merge { .. } => true,
        Command::Reds(_)
        | Command::CountObjects { .. }
        | Command::Repack { .. }
        | Command::Format
        | Command::Version
        | Command::Origin { .. }
        | Command::Clone { .. }
        | Command::Remote { .. }
        | Command::UpdateRef { .. }
        | Command::UpdateCounts { .. }
        | Command::DeleteCounts
        | Command::SnapshotHead
        | Command::SnapshotMerge { .. }
        | Command::SnapshotFetch { .. }
        | Command::SnapshotReachable { .. }
        | Command::SnapshotReset { .. }
        | Command::Clean
        | Command::Unmerged
        | Command::UpdateIndex { .. }
        | Command::Add
        | Command::DiffIndex { .. }
        | Command::DiffPaths { .. }
        | Command::WriteTree
        | Command::CommitTree { .. }
        | Command::MergeBases { .. }
        | Command::BringsList { .. }
        | Command::IsAncestor { .. } => false,
    }
}

fn merge_bases(op: &git::Op, code: u8, stdout: &[u8], tail: &[u8], cut: u64) -> Next {
    let git::Op::Brings { base, .. } = op else { unreachable!("merge bases belong to Brings") };
    if code == 1 && stdout.is_empty() {
        return Next::End(git::Done::Brings { brings: false });
    }
    if code == 0 && !stdout.is_empty() {
        let mut bases = skein_lib::List::uncapped();
        for line in stdout.split_inclusive(|byte| *byte == b'\n') {
            let Some(commit) = id(line) else { return failed(git::Fault::Broken, tail, cut) };
            bases.push(commit).expect("bounded stdout");
        }
        return Next::Run(Command::BringsList { bases: bases.into_boxed(), base: *base });
    }
    failed(git::Fault::Broken, tail, cut)
}

fn format(stdout: &[u8]) -> Option<invocation::Format> {
    match stdout {
        b"sha1\n" => Some(invocation::Format::Sha1),
        b"sha256\n" => Some(invocation::Format::Sha256),
        other => {
            let _length = other.len();
            None
        }
    }
}
/// The operations that move the worker's metadata (protocol/git-operations.md, 1.13).
pub(crate) fn view_place(op: &git::Op) -> Option<&git::Place> {
    match op {
        git::Op::Clone { at, .. }
        | git::Op::Fetch { at, .. }
        | git::Op::Counts { at, .. }
        | git::Op::Point { at, .. }
        | git::Op::CheckOut { at, .. }
        | git::Op::Merge { at, .. }
        | git::Op::Commit { at, .. }
        | git::Op::Reset { at, .. } => Some(at),
        git::Op::Pack { .. }
        | git::Op::Keep { .. }
        | git::Op::Make { .. }
        | git::Op::Store { .. }
        | git::Op::Look { .. }
        | git::Op::Create { .. }
        | git::Op::Push { .. }
        | git::Op::IsAncestor { .. }
        | git::Op::Brings { .. }
        | git::Op::DiffPaths { .. }
        | git::Op::Reds { .. } => None,
    }
}
/// Whether the terminal still has a view to write (protocol/git-operations.md, 1.13).
pub(crate) fn views(op: &git::Op, done: &git::Done) -> bool {
    match done {
        git::Done::Failed { .. } | git::Done::FailedWithOutput { .. } => false,
        git::Done::Results { .. }
        | git::Done::Paths { .. }
        | git::Done::ParentUnreachable { .. }
        | git::Done::Looked { .. }
        | git::Done::IsAncestor { .. }
        | git::Done::Brings { .. }
        | git::Done::Merged
        | git::Done::Conflicted { .. }
        | git::Done::Succeeded
        | git::Done::Fetched { .. }
        | git::Done::Committed { .. }
        | git::Done::Unchanged
        | git::Done::Exists
        | git::Done::Rejected => view_place(op).is_some(),
    }
}
/// Append the view once, only to a nonfailed terminal (protocol/git-operations.md, 1.13).
#[must_use]
pub(crate) fn ending(op: &git::Op, done: git::Done) -> Next {
    if views(op, &done) { Next::View { done: Box::new(done), retry: false } } else { Next::End(done) }
}

/// Reuse git's format and init commands while retaining the operation's terminal.
pub(crate) fn after_view(
    op: &git::Op,
    command: &Command,
    done: &git::Done,
    code: u8,
    stdout: &[u8],
    tail: &[u8],
    cut: u64,
) -> Next {
    match command {
        Command::Format if code == 0 => {
            if let Some(format) = format(stdout) {
                return Next::Run(Command::Clone {
                    directory: invocation::place(view_place(op).expect("view place")),
                    format,
                    bare: false,
                });
            }
        }
        Command::Clone { .. } if code == 0 => return Next::View { done: Box::new(done.clone()), retry: true },
        Command::Reds(_)
        | Command::CountObjects { .. }
        | Command::Repack { .. }
        | Command::Format
        | Command::SnapshotHead
        | Command::SnapshotMerge { .. }
        | Command::SnapshotFetch { .. }
        | Command::SnapshotReachable { .. }
        | Command::SnapshotReset { .. }
        | Command::Version
        | Command::Origin { .. }
        | Command::Clone { .. }
        | Command::Remote { .. }
        | Command::Store { .. }
        | Command::UpdateRef { .. }
        | Command::UpdateCounts { .. }
        | Command::DeleteCounts
        | Command::Fetch { .. }
        | Command::FetchCounts { .. }
        | Command::Create { .. }
        | Command::CheckOut { .. }
        | Command::Clean
        | Command::Merge { .. }
        | Command::Unmerged
        | Command::UpdateIndex { .. }
        | Command::Add
        | Command::DiffIndex { .. }
        | Command::DiffPaths { .. }
        | Command::WriteTree
        | Command::CommitTree { .. }
        | Command::MergeBases { .. }
        | Command::BringsList { .. }
        | Command::IsAncestor { .. }
        | Command::Push { .. } => {}
    }
    failed(git::Fault::Broken, tail, cut)
}
