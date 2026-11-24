//! Between the checkout's operations and the fakes, as the protocol layer
//! and io would translate them: io names a workspace's directory by its token,
//! and a repository's by the workspace's and the repository's name; a commit
//! uses the fake forge's shared object spelling (skein's testing/fakes/forge.md, 3).

use skein_lib::Token;
use temper_fake_checkout::Checkout;
use temper_fake_checkout::git::{self, Pushed, Remote, What};
use temper_worker_domain_checkout::git::{Commit, Done, Fault, Missing, Op, Place};

mod machine;
pub use machine::{cut, perform_with_capacity, used, writes};

/// The domain's name for the fake's commit `fake`.
#[must_use]
pub fn commit(fake: u64) -> Commit {
    let mut raw = [0; 32];
    raw[..20].copy_from_slice(&skein_fake_forge_domain::object_bytes(fake));
    Commit::new(raw)
}

/// The fake's name for the domain's `commit`.
#[must_use]
pub fn fake(commit: Commit) -> u64 {
    let raw = commit.raw();
    let mut count = [0; 8];
    count.copy_from_slice(&raw[8..16]);
    let count = u64::from_be_bytes(count);
    assert_eq!(commit, self::commit(count), "a commit the forge named");
    count
}

/// Where io keeps the workspace `workspace`.
#[must_use]
pub fn dir(workspace: Token) -> Vec<u8> {
    temper_fake_checkout::layout::dir(workspace)
}

/// Where io keeps the repository at `place`.
#[must_use]
pub fn path(place: &Place) -> Vec<u8> {
    temper_fake_checkout::layout::path(place.workspace, &place.repository)
}

/// The workspace an operation is in.
#[must_use]
pub fn workspace(op: &Op) -> Option<Token> {
    match op {
        Op::Store { .. } => None,
        Op::Keep { workspace, .. } | Op::Make { workspace, .. } => Some(*workspace),
        Op::Clone { at, .. }
        | Op::Reds { at, .. }
        | Op::Reset { at, .. }
        | Op::Pack { at, .. }
        | Op::Point { at, .. }
        | Op::Fetch { at, .. }
        | Op::Counts { at, .. }
        | Op::Create { at, .. }
        | Op::CheckOut { at, .. }
        | Op::Look { at, .. }
        | Op::DiffPaths { at, .. }
        | Op::Brings { at, .. }
        | Op::IsAncestor { at, .. }
        | Op::Merge { at, .. }
        | Op::Commit { at, .. }
        | Op::Push { at, .. } => Some(at.workspace),
    }
}

/// The identity an operation acts as, if it names one.
#[must_use]
pub fn identity(op: &Op) -> Option<u32> {
    match op {
        Op::Store { identity, .. }
        | Op::CheckOut { identity, .. }
        | Op::Merge { identity, .. }
        | Op::Fetch { identity, .. }
        | Op::Counts { identity, .. }
        | Op::Create { identity, .. }
        | Op::Commit { identity, .. }
        | Op::Push { identity, .. } => Some(*identity),
        Op::Reds { .. }
        | Op::Reset { .. }
        | Op::Pack { .. }
        | Op::Point { .. }
        | Op::Clone { .. }
        | Op::Keep { .. }
        | Op::Make { .. }
        | Op::Look { .. }
        | Op::DiffPaths { .. }
        | Op::Brings { .. }
        | Op::IsAncestor { .. } => None,
    }
}

/// The repository a remote operation reaches, by the forge's address for it.
#[must_use]
pub fn remote(op: &Op) -> Option<&[u8]> {
    match op {
        Op::Store { remote, .. }
        | Op::Fetch { remote, .. }
        | Op::Counts { remote, .. }
        | Op::Create { remote, .. }
        | Op::Push { remote, .. } => Some(remote),
        Op::Commit { snapshot, .. } => snapshot.as_ref().map(|snapshot| snapshot.remote.as_ref()),
        Op::Reds { .. }
        | Op::Reset { .. }
        | Op::Pack { .. }
        | Op::Point { .. }
        | Op::Clone { .. }
        | Op::Keep { .. }
        | Op::Make { .. }
        | Op::CheckOut { .. }
        | Op::Merge { .. }
        | Op::Look { .. }
        | Op::DiffPaths { .. }
        | Op::Brings { .. }
        | Op::IsAncestor { .. } => None,
    }
}

/// Runs `op` on the fakes, as the protocol layer would run its git
/// invocation, git reaching the forge through `forge`, and says how it ended.
pub fn perform(forge: &mut impl temper_fake_git::counts::Remote, disk: &mut Checkout, op: Op) -> Done {
    perform_with_capacity(forge, disk, op, None)
}

#[expect(clippy::too_many_lines, reason = "the world translates every checkout operation in one table")]
fn perform_unchecked(forge: &mut impl temper_fake_git::counts::Remote, disk: &mut Checkout, op: Op) -> Done {
    match op {
        Op::Keep { workspace, mark } => {
            let path = format!("kept/{}", workspace.raw());
            if let Some(mark) = mark {
                disk.write(path.as_bytes(), format!("{mark:?}").as_bytes());
            } else {
                disk.remove(path.as_bytes());
            }
            Done::Succeeded
        }
        Op::Reset { at, commit: head } => {
            assert_cloned(disk, &at);
            assert!(git::object_present(disk, &path(&at), fake(head)));
            disk.write(&[path(&at).as_slice(), b"/.git/temper-head"].concat(), &fake(head).to_le_bytes());
            disk.remove(&[path(&at).as_slice(), b"/.git/MERGE_HEAD"].concat());
            disk.remove(&[path(&at).as_slice(), b"/.git/temper-conflicts"].concat());
            Done::Committed { commit: head }
        }
        Op::Reds { at, action } => {
            if !matches!(action, temper_worker_domain_checkout::git::Reds::Clear)
                && disk.content(&[path(&at).as_slice(), b"/.git/refs/temper/landing"].concat()).is_none()
            {
                return Done::Succeeded;
            }
            let directory = [path(&at).as_slice(), b"/.git/temper"].concat();
            let file = [directory.as_slice(), b"/reds"].concat();
            let results = [directory.as_slice(), b"/check-results"].concat();
            match action {
                temper_worker_domain_checkout::git::Reds::Prepare { wait_ms, rerun_processes } => {
                    disk.remove(&results);
                    disk.remove(&file);
                    disk.mkdir(&directory);
                    disk.write(
                        &[directory.as_slice(), b"/reds-wait"].concat(),
                        format!("temper-reds-wait 1 {wait_ms}\n").as_bytes(),
                    );
                    disk.write(
                        &[directory.as_slice(), b"/rerun"].concat(),
                        format!("temper-rerun 1 {rerun_processes}\n").as_bytes(),
                    );
                }
                temper_worker_domain_checkout::git::Reds::Clear => {
                    disk.remove(&file);
                    disk.remove(&results);
                }
                temper_worker_domain_checkout::git::Reds::Read { .. } => {
                    disk.remove(&file);
                    let bytes = disk.content(&results).map(|bytes| Box::from(&bytes[..bytes.len().min(262_145)]));
                    disk.remove(&results);
                    return Done::Results { bytes };
                }
                temper_worker_domain_checkout::git::Reds::Write { .. } => return Done::Failed { fault: Fault::Broken },
            }
            Done::Succeeded
        }
        Op::Make { workspace, stores } => {
            // Empty: whatever is there goes first.
            let dir = dir(workspace);
            disk.remove(&dir);
            disk.mkdir(&dir);
            for store in stores {
                disk.remove(
                    &[store_path(store).as_slice(), b"/refs/temper/", workspace.raw().to_string().as_bytes()].concat(),
                );
            }
            Done::Succeeded
        }
        Op::Look { at, remote, store: _ } => {
            let directory = path(&at);
            Done::Looked {
                holds: disk.content(&[directory.as_slice(), b"/.git/temper-remote"].concat()) == Some(remote.as_ref()),
            }
        }
        Op::Store { store, aside, remote, .. } => {
            if let Some(aside) = aside {
                disk.remove(&store_path(aside));
            }
            if !disk.exists(&store_path(store))
                && let Err(fault) = git::clone_bare(forge, disk, &remote, &store_path(store))
            {
                return failed(fault);
            }
            Done::Succeeded
        }
        Op::Clone { at, remote, store } => {
            assert!(disk.exists(&dir(at.workspace)), "a workspace made before its clone");
            let format = git::object_format(disk, &store_path(store));
            git::init_with_format(disk, &path(&at), false, format);
            let alternate = format!("../../../../../stores/{}/objects\n", store.raw());
            disk.write(&[path(&at).as_slice(), b"/.git/objects/info/alternates"].concat(), alternate.as_bytes());
            disk.write(&[path(&at).as_slice(), b"/.git/temper-remote"].concat(), &remote);
            Done::Succeeded
        }
        Op::Fetch { at, store, remote, want, into, identity: _, account: _ } => {
            assert_cloned(disk, &at);
            machine::fetch(forge, disk, &at, store, &remote, &want, into, false)
        }
        Op::Counts { at, store, remote, .. } => {
            assert_cloned(disk, &at);
            machine::counts(forge, disk, &at, store, &remote, false)
        }
        Op::Pack { at, store } => {
            if let Some(store) = store {
                machine::pack(disk, &store_path(store));
            }
            machine::pack(disk, &[path(&at).as_slice(), b"/.git"].concat());
            Done::Succeeded
        }
        Op::Point { at, into, commit } => {
            assert_cloned(disk, &at);
            let reference = machine::reference(into);
            match git::update_ref(disk, &path(&at), reference, fake(commit)) {
                Ok(()) => Done::Succeeded,
                Err(_) => Done::Failed { fault: Fault::Broken },
            }
        }
        Op::Create { at, remote, branch, commit, identity: _, account: _ } => {
            assert_cloned(disk, &at);
            match forge.push_new(&remote, &branch, fake(commit)) {
                Ok(None) => Done::Succeeded,
                Ok(Some(_)) => Done::Exists,
                Err(fault) => failed(fault),
            }
        }
        Op::CheckOut { at, commit, .. } => {
            assert_cloned(disk, &at);
            let checked_out = git::check_out(forge, disk, &path(&at), fake(commit));
            checked_out.expect("a checkout is of a commit fetched into the repository");
            machine::add_pack(disk, &[path(&at).as_slice(), b"/.git"].concat());
            Done::Succeeded
        }
        Op::Commit { at, mut parent, mut merging, mut always, title, body, identity: _, snapshot } => {
            assert_cloned(disk, &at);
            assert!(!title.is_empty(), "a commit has a title");
            if let Some(snapshot) = &snapshot {
                let head = local_head(disk, &at, b"temper-head").expect("a prepared checkout has HEAD");
                merging = local_head(disk, &at, b"MERGE_HEAD").or(if head == parent { merging } else { None });
                if head != parent {
                    always = false;
                }
                parent = head;
                if let Some(second) = merging.map(fake).or_else(|| forge.merge_parent(fake(head))) {
                    let heads = match forge.heads(&snapshot.remote) {
                        Ok(heads) => heads,
                        Err(fault) => return failed(fault),
                    };
                    if !heads.iter().any(|head| contains(forge, second, *head)) {
                        return Done::ParentUnreachable { head, merging, parent: commit(second) };
                    }
                }
            }
            let message =
                if body.is_empty() { title.to_vec() } else { [title.as_ref(), b"\n\n", body.as_ref()].concat() };
            match merging {
                None if always => Done::Committed {
                    commit: commit(
                        git::commit_always(forge, disk, &path(&at), fake(parent), &message).expect("parent fetched"),
                    ),
                },
                None => match git::commit(forge, disk, &path(&at), fake(parent), &message)
                    .expect("parent is locally fetched")
                {
                    Some(committed) => Done::Committed { commit: commit(committed) },
                    None => {
                        if snapshot.is_some() {
                            Done::Committed { commit: parent }
                        } else {
                            Done::Unchanged
                        }
                    }
                },
                Some(second) => {
                    match git::commit_merging(forge, disk, &path(&at), fake(parent), fake(second), &message) {
                        Ok(committed) => Done::Committed { commit: commit(committed) },
                        Err(git::CommitFailure::Unresolved { files }) => Done::Conflicted {
                            files: files.into_iter().map(Vec::into_boxed_slice).collect(),
                            cut: false,
                        },
                        Err(git::CommitFailure::NotFetched) => panic!("both merge parents are locally fetched"),
                    }
                }
            }
        }
        Op::DiffPaths { at, before, after, markdown } => {
            assert_cloned(disk, &at);
            let before = forge.tree(fake(before));
            let after = forge.tree(fake(after));
            let paths: std::collections::BTreeSet<_> = before.keys().chain(after.keys()).collect();
            Done::Paths {
                files: paths
                    .into_iter()
                    .filter(|path| before.get(*path) != after.get(*path))
                    .filter(|path| markdown || !path.ends_with(b".md"))
                    .take(1)
                    .map(|path| path.clone().into_boxed_slice())
                    .collect(),
            }
        }
        Op::Brings { at, head, base, upstream } => {
            assert_cloned(disk, &at);
            let shared = ancestors(forge, fake(head));
            let upstream = ancestors(forge, fake(upstream));
            let base = ancestors(forge, fake(base));
            Done::Brings { brings: shared.intersection(&upstream).any(|commit| !base.contains(commit)) }
        }
        Op::IsAncestor { at, ancestor, commit } => {
            assert_cloned(disk, &at);
            Done::IsAncestor { contains: contains(forge, fake(ancestor), fake(commit)) }
        }

        Op::Merge { at, theirs, .. } => {
            assert_cloned(disk, &at);
            let merged = git::merge(forge, disk, &path(&at), fake(theirs)).expect("merge parent is locally fetched");
            machine::add_pack(disk, &[path(&at).as_slice(), b"/.git"].concat());
            if merged.conflicts.is_empty() {
                Done::Merged
            } else {
                Done::Conflicted {
                    files: merged.conflicts.into_iter().map(Vec::into_boxed_slice).collect(),
                    cut: false,
                }
            }
        }
        Op::Push { at, remote, commit, branch, expected, identity: _, account: _ } => {
            assert_cloned(disk, &at);
            let pushed = match expected {
                Some(expected) => {
                    git::push_expected(forge, disk, &remote, &path(&at), fake(commit), &branch, fake(expected))
                }
                None => git::push(forge, disk, &remote, &path(&at), fake(commit), &branch),
            };
            match pushed {
                Ok(Pushed::Pushed) => Done::Succeeded,
                Ok(Pushed::Rejected) => Done::Rejected,
                Err(git::Fault::Refused) => Done::FailedWithOutput {
                    fault: Fault::Refused,
                    diagnostic: temper_worker_domain_checkout::git::PushDiagnostic::new(b"remote: push refused", 0),
                },
                Err(fault) => failed(fault),
            }
        }
    }
}

/// `perform`, with the git layer's rules on the checkout's `conflicts`
/// (temper/docs/design/protocol/git-operations.md, 1.6 and 1.7).
pub fn perform_within(
    forge: &mut impl temper_fake_git::counts::Remote,
    disk: &mut Checkout,
    op: Op,
    conflicts: u32,
) -> Done {
    perform_within_capacity(forge, disk, op, conflicts, None)
}

/// The same conflict bound, with the fake disk's optional byte capacity.
pub fn perform_within_capacity(
    forge: &mut impl temper_fake_git::counts::Remote,
    disk: &mut Checkout,
    op: Op,
    conflicts: u32,
    capacity: Option<u64>,
) -> Done {
    let merge = matches!(op, Op::Merge { .. });
    match perform_with_capacity(forge, disk, op, capacity) {
        Done::Conflicted { files, cut } if merge => {
            let most = usize::try_from(conflicts).expect("conflicts fits usize");
            let cut = cut || files.len() > most;
            Done::Conflicted { files: files.into_iter().take(most).collect(), cut }
        }
        done => done,
    }
}

fn failed(fault: git::Fault) -> Done {
    let fault = match fault {
        git::Fault::Missing(What::Repository) => Fault::Missing { missing: Missing::Repository },
        git::Fault::Missing(What::Branch) => Fault::Missing { missing: Missing::Branch },
        git::Fault::Missing(What::Commit) => Fault::Missing { missing: Missing::Commit },
        git::Fault::Refused => Fault::Refused,
        git::Fault::Unreachable => Fault::Unreachable,
    };
    Done::Failed { fault }
}

fn assert_cloned(disk: &Checkout, at: &Place) {
    let git = [path(at).as_slice(), b"/.git"].concat();
    assert!(disk.exists(&git), "an operation in a repository runs in a clone: {:?}", String::from_utf8_lossy(&git));
}

/// Local git ancestry, independent of the checkout's delivery decision.
fn contains(forge: &impl Remote, ancestor: u64, commit: u64) -> bool {
    let mut pending = vec![commit];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(head) = pending.pop() {
        if head == ancestor {
            return true;
        }
        if seen.insert(head) {
            pending.extend(forge.parent(head));
            pending.extend(forge.merge_parent(head));
        }
    }
    false
}

fn local_head(disk: &Checkout, at: &Place, name: &[u8]) -> Option<Commit> {
    let bytes = disk.content(&[path(at).as_slice(), b"/.git/", name].concat())?;
    Some(commit(u64::from_le_bytes(bytes.try_into().expect("fake git head"))))
}

/// A shared bare repository on the world's fake disk.
#[must_use]
pub fn store_path(store: Token) -> Vec<u8> {
    temper_fake_checkout::layout::store_path(store)
}

fn ancestors(forge: &mut impl Remote, head: u64) -> std::collections::BTreeSet<u64> {
    let mut seen = std::collections::BTreeSet::new();
    let mut pending = vec![head];
    while let Some(commit) = pending.pop() {
        if seen.insert(commit) {
            pending.extend(forge.parent(commit));
            pending.extend(forge.merge_parent(commit));
        }
    }
    seen
}
