//! Canonical filtered-tree keys (domain/landing.md, 6.1).
use crate::{Commit, Gate, Key, Repository};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
};
use skein_lib::{Writer, sha256::Sha256};

/// One forge tree entry.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub name: Box<[u8]>,
    pub mode: u32,
    pub kind: EntryKind,
    pub object: Commit,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EntryKind {
    Blob,
    Tree,
    Commit,
}
/// A whole tree: root named by candidate, directories by their own ids.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tree {
    pub object: Commit,
    pub entries: Box<[Entry]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Keyed {
    Need(Commit),
    Key(Key),
    TooLarge { bound: Option<KeyBound>, needed: u64 },
}
/// The configured filtered-tree capacity that refused the next entry.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KeyBound {
    Entries,
    PathBytes,
}
/// Bounds of the filtered entries and their total full-path bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct KeyLimits {
    pub entries: u32,
    pub path_bytes: u32,
}

/// Workflow, ignored-path count and paths, each length-prefixed by a
/// big-endian u16; then one byte for blocking and, only when true, base
/// (domain/landing.md, 6.1).
#[must_use]
pub fn definition(gate: &Gate) -> Box<[u8]> {
    let mut length = gate.workflow.len().checked_add(5).expect("definition header fits");
    length = length.checked_add(usize::from(gate.base)).expect("base flag fits");
    for path in &gate.ignored {
        length = length.checked_add(2).expect("path length prefix fits");
        length = length.checked_add(path.len()).expect("definition fits");
    }
    assert!(u16::try_from(length).is_ok(), "definition fits its encoded length");
    let mut out = Writer::new(length);
    put_word(&mut out, gate.workflow.len());
    out.put(&gate.workflow).expect("sized workflow");
    put_word(&mut out, gate.ignored.len());
    for path in &gate.ignored {
        put_word(&mut out, path.len());
        out.put(path).expect("sized ignored path");
    }
    out.put(&[u8::from(gate.blocking)]).expect("sized blocking flag");
    if gate.base {
        out.put(&[1]).expect("sized base flag");
    }
    out.finish()
}

fn put_word(out: &mut Writer, value: usize) {
    out.put(&u16::try_from(value).expect("definition length fits u16").to_be_bytes()).expect("sized word");
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pending {
    Directory(Commit),
    Entry { mode: u32, object: Commit },
}

/// Hash a filtered tree, requesting the first missing tree in path order.
#[must_use]
#[expect(clippy::too_many_arguments, reason = "a base tree is optional beside the candidate tree and key identity")]
pub fn key(
    repository: Repository,
    name: &[u8],
    definition: &[u8],
    ignored: &[Box<[u8]>],
    candidate: Commit,
    trees: &[Tree],
    base: Option<(Commit, &[Tree])>,
    limits: &KeyLimits,
) -> Keyed {
    let (Ok(name_len), Ok(definition_len)) = (u16::try_from(name.len()), u16::try_from(definition.len())) else {
        return Keyed::TooLarge { bound: None, needed: 0 };
    };
    let mut hash = Sha256::new();
    hash.update(&[if base.is_some() { 2 } else { 1 }]);
    hash.update(&u64::from(repository.forge).to_be_bytes());
    hash.update(&u64::from(repository.repository).to_be_bytes());
    hash.update(&name_len.to_be_bytes());
    hash.update(name);
    hash.update(&definition_len.to_be_bytes());
    hash.update(definition);
    if let Some((commit, trees)) = base {
        let entries = match walk(ignored, commit, trees, *limits) {
            Ok(entries) => entries,
            Err(problem) => return problem,
        };
        hash.update(&u32::try_from(entries.len()).expect("bounded filtered tree").to_be_bytes());
        hash_entries(&mut hash, entries);
    }
    match walk(ignored, candidate, trees, *limits) {
        Ok(entries) => {
            hash_entries(&mut hash, entries);
            Keyed::Key(hash.finish())
        }
        Err(problem) => problem,
    }
}

type Filtered = BTreeMap<Box<[u8]>, (u32, Commit)>;
fn hash_entries(hash: &mut Sha256, entries: Filtered) {
    for (path, (mode, object)) in entries {
        hash.update(&u16::try_from(path.len()).expect("admitted path length").to_be_bytes());
        hash.update(&path);
        hash.update(&mode.to_be_bytes());
        hash.update(&object);
    }
}

fn walk(ignored: &[Box<[u8]>], candidate: Commit, trees: &[Tree], limits: KeyLimits) -> Result<Filtered, Keyed> {
    let mut filtered = BTreeMap::new();
    let mut pending = BTreeMap::new();
    let mut read = BTreeSet::new();
    let replaced = pending.insert(Box::<[u8]>::from([]), Pending::Directory(candidate));
    assert!(replaced.is_none(), "root is the first entry");
    let mut entries = 0_u32;
    let mut path_bytes = 0_u32;
    // Each expanded directory consumes a distinct prefix of an ignored path.
    let mut budget = u64::from(limits.entries).saturating_add(1);
    for path in ignored {
        budget = budget.saturating_add(skein_lib::len_u64(path.len()));
    }
    for _ in 0..budget {
        let Some((path, item)) = pending.pop_first() else { return finished(filtered, &read, trees) };
        match item {
            Pending::Directory(object) => {
                let mut tree = None;
                for supplied in trees {
                    if supplied.object == object {
                        if tree.is_some() {
                            return Err(Keyed::TooLarge { bound: None, needed: 0 });
                        }
                        tree = Some(supplied);
                    }
                }
                let Some(tree) = tree else {
                    for supplied in trees {
                        if !read.contains(&supplied.object) {
                            return Err(Keyed::TooLarge { bound: None, needed: 0 });
                        }
                    }
                    return Err(Keyed::Need(object));
                };
                let _inserted = read.insert(object);
                expand(&path, tree, ignored, limits, &mut pending, &mut entries, &mut path_bytes)?;
            }
            Pending::Entry { mode, object } => {
                let replaced = filtered.insert(path, (mode, object));
                assert!(replaced.is_none(), "filtered paths are unique");
            }
        }
    }
    if pending.is_empty() { finished(filtered, &read, trees) } else { Err(Keyed::TooLarge { bound: None, needed: 0 }) }
}

fn finished(filtered: Filtered, read: &BTreeSet<Commit>, trees: &[Tree]) -> Result<Filtered, Keyed> {
    for tree in trees {
        if !read.contains(&tree.object) {
            return Err(Keyed::TooLarge { bound: None, needed: 0 });
        }
    }
    Ok(filtered)
}

fn expand(
    parent: &[u8],
    tree: &Tree,
    ignored: &[Box<[u8]>],
    limits: KeyLimits,
    pending: &mut BTreeMap<Box<[u8]>, Pending>,
    entries: &mut u32,
    path_bytes: &mut u32,
) -> Result<(), Keyed> {
    for entry in &tree.entries {
        let length = parent.len().saturating_add(usize::from(!parent.is_empty())).saturating_add(entry.name.len());
        if entry.name.is_empty() {
            return Err(Keyed::TooLarge { bound: None, needed: 0 });
        }
        let mut path = Writer::new(length);
        path.put(parent).expect("sized parent");
        if !parent.is_empty() {
            path.put(b"/").expect("sized separator");
        }
        path.put(&entry.name).expect("sized name");
        let path = path.finish();
        let mut excluded = false;
        let mut below = false;
        for ignore in ignored {
            if prefix(ignore, &path) {
                excluded = true;
            }
            if path.len() < ignore.len() && prefix(&path, ignore) {
                below = true;
            }
        }
        if excluded {
            continue;
        }
        let item = match entry.kind {
            EntryKind::Tree if below => Pending::Directory(entry.object),
            EntryKind::Tree | EntryKind::Blob | EntryKind::Commit => {
                if *entries >= limits.entries {
                    return Err(Keyed::TooLarge {
                        bound: Some(KeyBound::Entries),
                        needed: u64::from(*entries).saturating_add(1),
                    });
                }
                if length > usize::from(u16::MAX) {
                    return Err(Keyed::TooLarge { bound: None, needed: 0 });
                }
                *entries = entries.checked_add(1).expect("entry below limit");
                let total = u64::from(*path_bytes).saturating_add(skein_lib::len_u64(path.len()));
                if total > u64::from(limits.path_bytes) {
                    return Err(Keyed::TooLarge { bound: Some(KeyBound::PathBytes), needed: total });
                }
                *path_bytes = u32::try_from(total).expect("path bytes below limit");
                Pending::Entry { mode: entry.mode, object: entry.object }
            }
        };
        if pending.insert(path, item).is_some() {
            return Err(Keyed::TooLarge { bound: None, needed: 0 });
        }
    }
    Ok(())
}

fn prefix(prefix: &[u8], path: &[u8]) -> bool {
    path == prefix || (path.starts_with(prefix) && path.get(prefix.len()) == Some(&b'/'))
}
