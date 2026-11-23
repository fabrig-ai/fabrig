//! A store fetch shares its lock; Pack's store part holds it alone
//! (temper/docs/design/protocol/git-operations.md, 1.14, A store's lock).
use crate::invocation::Command;
use skein_lib::Token;
use temper_worker_domain_checkout::git;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Access {
    pub store: Token,
    pub shared: bool,
}
impl Access {
    pub fn conflicts(self, other: Access) -> bool {
        self.store == other.store && !(self.shared && other.shared)
    }
    /// Only store invocations retain an access; a repository invocation releases it.
    pub fn for_command(op: &git::Op, command: &Command) -> Option<Access> {
        match command {
            Command::CountObjects { store: Some(store) } | Command::Repack { store: Some(store) } => {
                Some(Access { store: *store, shared: false })
            }
            Command::Fetch { .. } => match op {
                git::Op::Fetch { store, .. } => Some(Access { store: *store, shared: true }),
                git::Op::Pack { .. }
                | git::Op::Reset { .. }
                | git::Op::Reds { .. }
                | git::Op::Keep { .. }
                | git::Op::Make { .. }
                | git::Op::Store { .. }
                | git::Op::Look { .. }
                | git::Op::Clone { .. }
                | git::Op::Point { .. }
                | git::Op::Create { .. }
                | git::Op::CheckOut { .. }
                | git::Op::IsAncestor { .. }
                | git::Op::Brings { .. }
                | git::Op::DiffPaths { .. }
                | git::Op::Merge { .. }
                | git::Op::Commit { .. }
                | git::Op::Push { .. } => unreachable!("store fetch belongs to Fetch"),
            },
            Command::CountObjects { store: None }
            | Command::Repack { store: None }
            | Command::Reds(_)
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
            | Command::Push { .. } => None,
        }
    }
}
