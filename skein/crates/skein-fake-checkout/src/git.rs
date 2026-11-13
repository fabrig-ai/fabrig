//! Local git mechanics over `Checkout` (testing/fakes/checkout.md, section 4).
//! Keeps object-presence, checked-out head and merge-conflict metadata in the
//! checkout; commit graphs and trees live in the caller-supplied `Remote`.
//! Never knows a service's repository policy, transport, authorization, clock
//! or delivery lifecycle (testing-strategy.md, section 4.3).
//!
//! Clone/fetch bring both parent chains; checkout replaces non-git files;
//! commit snapshots them without moving a remote reference. Merge combines
//! small single-base histories and writes conflict markers; an explicit merge
//! commit records both parents even for an unchanged tree. Push delegates
//! policy and expected-head checks to `Remote`. All calls return synchronously.
//!
//! The fixture owns graph/tree sizes and acyclic increasing commit IDs.
//! Merge chooses the greatest common ID and allocates quadratic line-diff
//! scratch, so this is no full Git implementation or production memory bound
//! (programming-model.md, section 10.2; testing/fakes/checkout.md, section 4).

use std::collections::{BTreeMap, BTreeSet};

use crate::{Checkout, in_git};

/// Owned regular-file bytes keyed by path relative to a working-tree root.
/// Fixture supplies valid paths and sizes (testing/fakes/checkout.md, section 4).
pub type Tree = BTreeMap<Vec<u8>, Vec<u8>>;

/// World-supplied synchronous remote/store boundary. The kit forwards
/// repository and branch bytes; the implementation owns policy, availability
/// and fault answers. Commit IDs increase topologically in a finite acyclic
/// graph; all queried IDs must exist. No delivery state or clock is held here
/// (testing/fakes/checkout.md, section 4).
pub trait Remote {
    /// Metadata and the default tip for a bare, single-branch clone.
    fn clone_default(&mut self, _remote: &[u8]) -> Result<CloneInfo, Fault> {
        panic!("the world supplies metadata for a bare clone")
    }

    /// Clone asks for all repository branch tips, or a remote refusal. No
    /// checkout mutation occurs before this returns (testing/fakes/checkout.md, section 4).
    fn heads(&mut self, remote: &[u8]) -> Result<Vec<u64>, Fault>;

    /// Fetch asks for the selected tip, or a remote refusal. The implementation
    /// resolves the selector; the kit imports its missing ancestry afterward
    /// (testing/fakes/checkout.md, section 4).
    fn fetch(&mut self, remote: &[u8], want: Want<'_>) -> Result<u64, Fault>;

    /// Caller delegates branch creation at `commit`; an existing branch stays
    /// where it is. Implementation checks repository/commit policy
    /// (testing/fakes/checkout.md, section 4).
    fn create(&mut self, remote: &[u8], branch: &[u8], commit: u64) -> Result<Created, Fault>;

    /// Upload a store commit and create the branch only if absent. None means
    /// created; Some is the existing head, with no move or upload.
    fn push_new(&mut self, _remote: &[u8], _branch: &[u8], _commit: u64) -> Result<Option<u64>, Fault> {
        panic!("the world supplies create-by-push")
    }

    /// Caller delegates push policy. `None` supplies no old-head condition;
    /// `Some` requires that exact old head in addition to the implementation’s
    /// fast-forward rules. Rejection/refusal must leave the reference unchanged
    /// (testing/fakes/checkout.md, section 4).
    fn push(&mut self, remote: &[u8], branch: &[u8], commit: u64, expected: Option<u64>) -> Result<Pushed, Fault>;

    /// Pure first-parent lookup of an existing commit; `None` for its root
    /// (testing/fakes/checkout.md, section 4).
    fn parent(&self, commit: u64) -> Option<u64>;

    /// Pure second-parent lookup; `None` for a nonmerge commit
    /// (testing/fakes/checkout.md, section 4).
    fn merge_parent(&self, commit: u64) -> Option<u64>;

    /// Pure owned tree lookup of an existing commit, bounded by the fixture
    /// (testing/fakes/checkout.md, section 4).
    fn tree(&self, commit: u64) -> Tree;

    /// Pure owned message lookup of an existing commit. The opaque bytes are
    /// bounded by the fixture (testing/fakes/checkout.md, section 4).
    fn message(&self, commit: u64) -> Vec<u8>;

    /// Store the supplied tree with existing parent IDs. With no second parent,
    /// return `None` only when unchanged from `parent` and `always` is false; with a second parent,
    /// always issue a fresh ID recording both parents, even if unchanged.
    /// A new object records `message` verbatim; an unchanged tree drops it.
    /// No remote reference moves (testing/fakes/checkout.md, section 4).
    fn store(&mut self, parent: u64, merging: Option<u64>, always: bool, tree: Tree, message: &[u8]) -> Option<u64>;
}

/// Remote implementation’s terminal refusal to a synchronous call. The kit
/// propagates it without starting checkout changes (testing/fakes/checkout.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fault {
    /// Remote has no requested repository, branch or commit (testing/fakes/checkout.md, section 4).
    Missing(
        /// Missing remote name kind (testing/fakes/checkout.md, section 4).
        What,
    ),

    /// Remote refuses the requested operation (testing/fakes/checkout.md, section 4).
    Refused,

    /// Remote cannot be reached (testing/fakes/checkout.md, section 4).
    Unreachable,
}

/// Remote name kind that was not found (testing/fakes/checkout.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum What {
    /// Repository address (testing/fakes/checkout.md, section 4).
    Repository,

    /// Branch name (testing/fakes/checkout.md, section 4).
    Branch,

    /// Commit ID (testing/fakes/checkout.md, section 4).
    Commit,
}

/// Caller’s fetch selector; interpreted entirely by `Remote`
/// (testing/fakes/checkout.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Want<'a> {
    /// Named branch tip (testing/fakes/checkout.md, section 4).
    Branch(
        /// Borrowed branch bytes for this call (testing/fakes/checkout.md, section 4).
        &'a [u8],
    ),

    /// Specific commit ID (testing/fakes/checkout.md, section 4).
    Commit(
        /// Commit ID in the remote’s store (testing/fakes/checkout.md, section 4).
        u64,
    ),

    /// Remote’s default branch tip (testing/fakes/checkout.md, section 4).
    Default,
}

/// Remote’s branch-creation answer (testing/fakes/checkout.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Created {
    /// Absent branch was created at the requested commit (testing/fakes/checkout.md, section 4).
    Created,

    /// Branch already existed and was left unchanged (testing/fakes/checkout.md, section 4).
    Exists,
}

/// Remote’s push answer, including policy/expected-head rejection
/// (testing/fakes/checkout.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pushed {
    /// Reference now names the requested commit (testing/fakes/checkout.md, section 4).
    Pushed,

    /// Policy or old-head condition rejected the push without moving the reference (testing/fakes/checkout.md, section 4).
    Rejected,
}

/// Required head metadata or local object is absent. Checkout/commit/merge
/// returns this before changing the working tree (testing/fakes/checkout.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NotFetched;

/// World adapter clones into an absent global directory: all commits
/// reachable from every tip through both parents, recorded as local objects,
/// without a checked-out tree. Remote failure leaves the checkout unchanged;
/// an existing destination asserts (testing/fakes/checkout.md, section 4).
pub fn clone_repository(
    forge: &mut impl Remote,
    checkout: &mut Checkout,
    remote: &[u8],
    at: &[u8],
) -> Result<(), Fault> {
    let heads = forge.heads(remote)?;
    let mut reached = BTreeSet::new();
    let mut pending = heads;
    while let Some(commit) = pending.pop() {
        if !reached.insert(commit) {
            continue;
        }
        pending.extend(forge.parent(commit));
        pending.extend(forge.merge_parent(commit));
    }
    assert!(!checkout.exists(at), "a clone goes where nothing is");
    checkout.mkdir(&[at, b"/.git"].concat());
    for commit in reached {
        checkout.write(&object(checkout, at, commit), b"");
    }
    Ok(())
}

/// World adapter imports the selected commit and missing ancestry through
/// both parents into `at`; existing object versions stay unchanged. Does not
/// check out files or move remote references. Remote refusal leaves checkout
/// unchanged (testing/fakes/checkout.md, section 4).
pub fn fetch(
    forge: &mut impl Remote,
    checkout: &mut Checkout,
    remote: &[u8],
    at: &[u8],
    want: Want<'_>,
) -> Result<u64, Fault> {
    let fetched = forge.fetch(remote, want)?;
    let mut pending = vec![fetched];
    let mut reached = BTreeSet::new();
    while let Some(commit) = pending.pop() {
        if !reached.insert(commit) || object_present(checkout, at, commit) {
            continue;
        }
        checkout.write(&object(checkout, at, commit), b"");
        pending.extend(forge.parent(commit));
        pending.extend(forge.merge_parent(commit));
    }
    Ok(fetched)
}

/// World adapter forwards branch creation directly to `Remote`; no local
/// checkout or object-presence check (testing/fakes/checkout.md, section 4).
pub fn create(forge: &mut impl Remote, remote: &[u8], branch: &[u8], commit: u64) -> Result<Created, Fault> {
    forge.create(remote, branch, commit)
}

/// World adapter replaces non-git descendants with an existing local commit’s
/// tree, writes head metadata and clears merge metadata. Local object absence
/// returns `NotFetched` without mutation (testing/fakes/checkout.md, section 4).
pub fn check_out(forge: &impl Remote, checkout: &mut Checkout, at: &[u8], commit: u64) -> Result<(), NotFetched> {
    if !object_present(checkout, at, commit) {
        return Err(NotFetched);
    }
    checkout.replace_tree(at, &forge.tree(commit));
    checkout.write(&[at, b"/.git/temper-head"].concat(), &commit.to_le_bytes());
    checkout.remove(&[at, b"/.git/temper-conflicts"].concat());
    checkout.remove(&[at, b"/.git/MERGE_HEAD"].concat());
    Ok(())
}

/// World adapter snapshots non-git regular files on a locally held parent.
/// Unchanged tree returns `None`; a new commit records the opaque message,
/// local presence and head.
/// No remote reference moves; missing parent returns before mutation
/// (testing/fakes/checkout.md, section 4).
pub fn commit(
    forge: &mut impl Remote,
    checkout: &mut Checkout,
    at: &[u8],
    parent: u64,
    message: &[u8],
) -> Result<Option<u64>, NotFetched> {
    commit_with(forge, checkout, at, parent, false, message)
}

/// Store a fresh single-parent object even when its tree is unchanged (testing/fakes/checkout.md, section 4).
pub fn commit_always(
    forge: &mut impl Remote,
    checkout: &mut Checkout,
    at: &[u8],
    parent: u64,
    message: &[u8],
) -> Result<u64, NotFetched> {
    Ok(commit_with(forge, checkout, at, parent, true, message)?.expect("always stores an object"))
}

fn commit_with(
    forge: &mut impl Remote,
    checkout: &mut Checkout,
    at: &[u8],
    parent: u64,
    always: bool,
    message: &[u8],
) -> Result<Option<u64>, NotFetched> {
    if !object_present(checkout, at, parent) {
        return Err(NotFetched);
    }
    let mut tree = checkout.tree(at);
    tree.retain(|path, _| !in_git(path));
    let Some(commit) = forge.store(parent, None, always, tree, message) else {
        return Ok(None);
    };
    checkout.write(&object(checkout, at, commit), b"");
    checkout.write(&[at, b"/.git/temper-head"].concat(), &commit.to_le_bytes());
    Ok(Some(commit))
}

/// World adapter forwards a locally held commit to `Remote::push` with no
/// old-head condition. The remote owns fast-forward and creation policy;
/// missing local object asserts (testing/fakes/checkout.md, section 4).
pub fn push(
    forge: &mut impl Remote,
    checkout: &Checkout,
    remote: &[u8],
    at: &[u8],
    commit: u64,
    branch: &[u8],
) -> Result<Pushed, Fault> {
    assert!(object_present(checkout, at, commit), "a push is of a commit the working tree has");
    forge.push(remote, branch, commit, None)
}

/// Merge result after the working tree and merge metadata were written;
/// no commit or remote reference was created (testing/fakes/checkout.md, section 4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Merged {
    /// Sorted relative paths requiring resolution, each holding conflict
    /// markers. Count/bytes are bounded by the fixture’s trees
    /// (testing/fakes/checkout.md, section 4).
    pub conflicts: Vec<Vec<u8>>,
}

/// World adapter merges checked-out head and locally held `theirs`, using
/// the greatest-ID common ancestor. Preserves git nodes, writes markers for
/// conflicts and records merge metadata. Missing head/object returns before
/// mutation; malformed head bytes assert. No commit/reference moves
/// (testing/fakes/checkout.md, section 4).
pub fn merge(forge: &impl Remote, checkout: &mut Checkout, at: &[u8], theirs: u64) -> Result<Merged, NotFetched> {
    let raw = checkout.content(&[at, b"/.git/temper-head"].concat()).ok_or(NotFetched)?;
    let ours = u64::from_le_bytes(raw.try_into().expect("checkout stores an eight-byte head"));
    if !object_present(checkout, at, theirs) {
        return Err(NotFetched);
    }
    let ours_tree = forge.tree(ours);
    let theirs_tree = forge.tree(theirs);
    let base = common_ancestor(forge, ours, theirs).map_or_else(Tree::new, |commit| forge.tree(commit));
    let paths: BTreeSet<_> = base.keys().chain(ours_tree.keys()).chain(theirs_tree.keys()).cloned().collect();
    let mut tree = Tree::new();
    let mut conflicts = Vec::new();
    for path in paths {
        let old = base.get(&path);
        let left = ours_tree.get(&path);
        let right = theirs_tree.get(&path);
        let content = if left == right || right == old {
            left.cloned()
        } else if left == old {
            right.cloned()
        } else if let (Some(old), Some(left), Some(right)) = (old, left, right) {
            if let Some(merged) = merge_lines(old, left, right) {
                Some(merged)
            } else {
                conflicts.push(path.clone());
                Some(markers(left, right))
            }
        } else {
            conflicts.push(path.clone());
            Some(markers(left.map_or(&[], Vec::as_slice), right.map_or(&[], Vec::as_slice)))
        };
        if let Some(content) = content {
            tree.insert(path, content);
        }
    }
    checkout.replace_tree(at, &tree);
    checkout.write(&[at, b"/.git/MERGE_HEAD"].concat(), &theirs.to_le_bytes());
    checkout.remove(&[at, b"/.git/temper-conflicts"].concat());
    for (index, path) in conflicts.iter().enumerate() {
        checkout.write(&[at, format!("/.git/temper-conflicts/{index}").as_bytes()].concat(), path);
    }
    Ok(Merged { conflicts })
}

/// Merge commit refusal before creating an object or changing metadata
/// (testing/fakes/checkout.md, section 4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CommitFailure {
    /// One of the supplied parents is not locally held (testing/fakes/checkout.md, section 4).
    NotFetched,

    /// Originally conflicted paths still contain marker lines (testing/fakes/checkout.md, section 4).
    Unresolved {
        /// Relative paths still containing marker lines, in recorded conflict
        /// order; fixture-sized (testing/fakes/checkout.md, section 4).
        files: Vec<Vec<u8>>,
    },
}

/// World adapter commits the current non-git tree with two supplied locally
/// held parents, even if unchanged. Only originally conflicted paths are
/// checked for marker lines; deleting one resolves it. Success writes object
/// presence/head, records the opaque message and clears merge metadata,
/// without moving references.
/// `Remote::store` must return a merge ID or this asserts
/// (testing/fakes/checkout.md, section 4).
pub fn commit_merging(
    forge: &mut impl Remote,
    checkout: &mut Checkout,
    at: &[u8],
    parent: u64,
    merging: u64,
    message: &[u8],
) -> Result<u64, CommitFailure> {
    if !object_present(checkout, at, parent) || !object_present(checkout, at, merging) {
        return Err(CommitFailure::NotFetched);
    }
    let files: Vec<_> = checkout
        .tree(&[at, b"/.git/temper-conflicts"].concat())
        .into_values()
        .filter(|path| checkout.content(&[at, b"/", path].concat()).is_some_and(has_markers))
        .collect();
    if !files.is_empty() {
        return Err(CommitFailure::Unresolved { files });
    }
    let mut tree = checkout.tree(at);
    tree.retain(|path, _| !in_git(path));
    let commit = forge.store(parent, Some(merging), false, tree, message).expect("a merge always records both parents");
    checkout.write(&object(checkout, at, commit), b"");
    checkout.write(&[at, b"/.git/temper-head"].concat(), &commit.to_le_bytes());
    checkout.remove(&[at, b"/.git/temper-conflicts"].concat());
    checkout.remove(&[at, b"/.git/MERGE_HEAD"].concat());
    Ok(commit)
}

/// World adapter forwards a locally held commit and `Some(expected)` old
/// head to `Remote::push`. The implementation owns conditional fast-forward
/// policy; missing local object asserts (testing/fakes/checkout.md, section 4).
pub fn push_expected(
    forge: &mut impl Remote,
    checkout: &Checkout,
    remote: &[u8],
    at: &[u8],
    commit: u64,
    branch: &[u8],
    expected: u64,
) -> Result<Pushed, Fault> {
    assert!(object_present(checkout, at, commit), "a push is of a locally held commit");
    forge.push(remote, branch, commit, Some(expected))
}

fn ancestors(forge: &impl Remote, start: u64) -> BTreeSet<u64> {
    let mut reached = BTreeSet::new();
    let mut pending = vec![start];
    while let Some(commit) = pending.pop() {
        if reached.insert(commit) {
            pending.extend(forge.parent(commit));
            pending.extend(forge.merge_parent(commit));
        }
    }
    reached
}

fn common_ancestor(forge: &impl Remote, ours: u64, theirs: u64) -> Option<u64> {
    // Store names are increasing topological numbers. The latest common
    // ancestor suffices for the world's single-base histories.
    let left = ancestors(forge, ours);
    let right = ancestors(forge, theirs);
    left.intersection(&right).copied().max()
}

fn markers(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut result = b"<<<<<<< ours\n".to_vec();
    result.extend_from_slice(left);
    if !left.ends_with(b"\n") {
        result.push(b'\n');
    }
    result.extend_from_slice(b"=======\n");
    result.extend_from_slice(right);
    if !right.ends_with(b"\n") {
        result.push(b'\n');
    }
    result.extend_from_slice(b">>>>>>> theirs\n");
    result
}

fn has_markers(content: &[u8]) -> bool {
    content
        .split(|byte| *byte == b'\n')
        .any(|line| line.starts_with(b"<<<<<<< ") || line == b"=======" || line.starts_with(b">>>>>>> "))
}

#[derive(PartialEq, Eq)]
struct Edit<'a> {
    start: usize,
    end: usize,
    replacement: Vec<&'a [u8]>,
}

fn edits<'a>(base: &[&[u8]], changed: &[&'a [u8]]) -> Vec<Edit<'a>> {
    let prefix = base.iter().zip(changed).take_while(|(left, right)| left == right).count();
    let suffix = base[prefix..]
        .iter()
        .rev()
        .zip(changed[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let base = &base[prefix..base.len() - suffix];
    let changed = &changed[prefix..changed.len() - suffix];
    let mut lengths = vec![vec![0; changed.len() + 1]; base.len() + 1];
    for i in (0..base.len()).rev() {
        for j in (0..changed.len()).rev() {
            lengths[i][j] = if base[i] == changed[j] {
                lengths[i + 1][j + 1] + 1
            } else {
                lengths[i + 1][j].max(lengths[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut result = Vec::new();
    while i < base.len() || j < changed.len() {
        if i < base.len() && j < changed.len() && base[i] == changed[j] {
            i += 1;
            j += 1;
            continue;
        }
        let start = i;
        let mut replacement = Vec::new();
        while i < base.len() || j < changed.len() {
            if i < base.len() && j < changed.len() && base[i] == changed[j] {
                break;
            }
            if j < changed.len() && (i == base.len() || lengths[i][j + 1] >= lengths[i + 1][j]) {
                replacement.push(changed[j]);
                j += 1;
            } else {
                i += 1;
            }
        }
        result.push(Edit { start: prefix + start, end: prefix + i, replacement });
    }
    result
}

fn merge_lines(base: &[u8], left: &[u8], right: &[u8]) -> Option<Vec<u8>> {
    let base: Vec<_> = base.split_inclusive(|byte| *byte == b'\n').collect();
    let left: Vec<_> = left.split_inclusive(|byte| *byte == b'\n').collect();
    let right: Vec<_> = right.split_inclusive(|byte| *byte == b'\n').collect();
    let mut changes = edits(&base, &left);
    for edit in edits(&base, &right) {
        if changes.contains(&edit) {
            continue;
        }
        if changes.iter().any(|old| {
            (old.start < edit.end && edit.start < old.end)
                || (old.start == old.end && old.start >= edit.start && old.start <= edit.end)
                || (edit.start == edit.end && edit.start >= old.start && edit.start <= old.end)
        }) {
            return None;
        }
        changes.push(edit);
    }
    changes.sort_by_key(|edit| edit.start);
    let mut result = Vec::new();
    let mut cursor = 0;
    for edit in changes {
        for line in &base[cursor..edit.start] {
            result.extend_from_slice(line);
        }
        for line in edit.replacement {
            result.extend_from_slice(line);
        }
        cursor = edit.end;
    }
    for line in &base[cursor..] {
        result.extend_from_slice(line);
    }
    Some(result)
}

/// Initialize a repository without contacting a remote. Bare repositories keep
/// their objects directly under the directory (testing/fakes/checkout.md, 4).
pub fn init(checkout: &mut Checkout, at: &[u8], bare: bool) {
    let directory = if bare { at.to_vec() } else { [at, b"/.git"].concat() };
    checkout.mkdir(&[directory.as_slice(), b"/objects/info"].concat());
    checkout.mkdir(&[directory.as_slice(), b"/refs/heads"].concat());
    checkout.write(&[directory.as_slice(), b"/HEAD"].concat(), b"ref: refs/heads/master\n");
}
fn directory(checkout: &Checkout, at: &[u8]) -> Vec<u8> {
    let git = [at, b"/.git"].concat();
    if checkout.exists(&git) { git } else { at.to_vec() }
}
fn object(checkout: &Checkout, at: &[u8], commit: u64) -> Vec<u8> {
    [directory(checkout, at).as_slice(), format!("/objects/{commit}").as_bytes()].concat()
}
/// Object presence includes one level of alternates, relative to the object directory.
#[must_use]
pub fn object_present(checkout: &Checkout, at: &[u8], commit: u64) -> bool {
    if checkout.exists(&object(checkout, at, commit)) {
        return true;
    }
    let objects = [directory(checkout, at).as_slice(), b"/objects"].concat();
    let Some(bytes) = checkout.content(&[objects.as_slice(), b"/info/alternates"].concat()) else {
        return false;
    };
    alternate_directories(&objects, bytes)
        .iter()
        .any(|alternate| checkout.exists(&[alternate.as_slice(), format!("/{commit}").as_bytes()].concat()))
}
/// Resolve newline-separated alternates relative to an object directory, without host IO.
#[must_use]
pub fn alternate_directories(objects: &[u8], bytes: &[u8]) -> Vec<Vec<u8>> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let joined = if line.starts_with(b"/") { line.to_vec() } else { [objects, b"/", line].concat() };
            let mut parts = Vec::new();
            for part in joined.split(|byte| *byte == b'/') {
                match part {
                    b"" | b"." => {}
                    b".." => {
                        parts.pop()?;
                    }
                    part => parts.push(part),
                }
            }
            let result = parts.join(&b'/');
            Some(if joined.starts_with(b"/") { [b"/", result.as_slice()].concat() } else { result })
        })
        .collect()
}
/// Move a reference only to an available object, in a bare or ordinary repository.
pub fn update_ref(checkout: &mut Checkout, at: &[u8], reference: &[u8], commit: u64) -> Result<(), NotFetched> {
    if !object_present(checkout, at, commit) {
        return Err(NotFetched);
    }
    let path = [directory(checkout, at).as_slice(), b"/", reference].concat();
    checkout.write(&path, &commit.to_le_bytes());
    Ok(())
}

/// The object format a remote advertises and its default branch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjectFormat {
    Sha1,
    Sha256,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CloneInfo {
    pub format: ObjectFormat,
    pub default: Vec<u8>,
    pub tip: Option<u64>,
}
/// Make a bare single-branch clone, taking the remote's format even when empty.
pub fn clone_bare(forge: &mut impl Remote, checkout: &mut Checkout, remote: &[u8], at: &[u8]) -> Result<(), Fault> {
    let info = forge.clone_default(remote)?;
    init_with_format(checkout, at, true, info.format);
    if let Some(tip) = info.tip {
        let mut pending = vec![tip];
        let mut reached = BTreeSet::new();
        while let Some(commit) = pending.pop() {
            if !reached.insert(commit) {
                continue;
            }
            checkout.write(&object(checkout, at, commit), b"");
            pending.extend(forge.parent(commit));
            pending.extend(forge.merge_parent(commit));
        }
        update_ref(checkout, at, &[b"refs/heads/", info.default.as_slice()].concat(), tip).expect("clone brought tip");
    }
    checkout.write(&[at, b"/HEAD"].concat(), &[b"ref: refs/heads/", info.default.as_slice(), b"\n"].concat());
    checkout.write(&[at, b"/remote"].concat(), remote);
    Ok(())
}
/// Initialize with an explicit format, without reaching the remote.
pub fn init_with_format(checkout: &mut Checkout, at: &[u8], bare: bool, format: ObjectFormat) {
    init(checkout, at, bare);
    let bytes = match format {
        ObjectFormat::Sha1 => b"sha1\n".as_slice(),
        ObjectFormat::Sha256 => b"sha256\n",
    };
    checkout.write(&[directory(checkout, at).as_slice(), b"/object-format"].concat(), bytes);
}
#[must_use]
pub fn object_format(checkout: &Checkout, at: &[u8]) -> ObjectFormat {
    match checkout.content(&[directory(checkout, at).as_slice(), b"/object-format"].concat()) {
        Some(b"sha256\n") => ObjectFormat::Sha256,
        _ => ObjectFormat::Sha1,
    }
}
