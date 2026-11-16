//! Atomic contents commits, including a repository's first head (skein/docs/design/testing/fakes/forge.md, 2.2, 2.3 and 4).
use crate::{
    Config, Domain,
    api::{Error, Permission, What},
    git::{self, Object},
    store::fits,
};
use alloc::boxed::Box;
use skein_lib::{Env, Map, bytes::copy_of};

#[derive(Debug)]
pub struct ContentsChange {
    pub path: Box<[u8]>,
    pub content: Box<[u8]>,
    pub expected: Option<[u8; 20]>,
}
/// Validates every change before storing one commit and starting its CI.
#[expect(clippy::too_many_arguments, reason = "one contents batch names its base and optional new branch")]
pub fn change_files(
    domain: &mut Domain,
    env: &Env<Config>,
    repository: &[u8],
    branch: &[u8],
    new_branch: Option<&[u8]>,
    message: &[u8],
    files: &[ContentsChange],
    by: u64,
) -> Result<u64, Error> {
    let limits = &env.limits.limits;
    let id = *domain.names.get(repository).ok_or(Error::Missing(What::Repository))?;
    let stored = domain.repositories.get(id).expect("named repository");
    stored.require(by, Permission::Write)?;
    if stored.refusing {
        return Err(Error::Refused);
    }
    let target = new_branch.unwrap_or(branch);
    if stored.is_protected(target) {
        return Err(Error::Protected);
    }
    fits(branch, limits.name_bytes)?;
    fits(target, limits.name_bytes)?;
    let message_bytes = limits.title_bytes.checked_add(limits.body_bytes).ok_or(Error::TooLarge)?;
    fits(message, message_bytes.checked_add(2).ok_or(Error::TooLarge)?)?;
    if branch.is_empty() || target.is_empty() || files.is_empty() {
        return Err(Error::Refused);
    }
    let tip = stored.branches.get(branch).copied();
    if new_branch.is_some() {
        if stored.branches.contains_key(target) {
            return Err(Error::Exists);
        }
        if tip.is_none() {
            return Err(Error::Missing(What::Branch));
        }
        if stored.branches.len() == stored.branches.capacity() {
            return Err(Error::Full);
        }
    }
    let mut tree = match tip {
        Some(tip) => git::copy_tree(limits, &domain.commits.get(&tip).expect("branch commit").tree),
        None => {
            if !stored.branches.is_empty() {
                return Err(Error::Missing(What::Branch));
            }
            if stored.branches.len() == stored.branches.capacity() {
                return Err(Error::Full);
            }
            Map::with_capacity(limits.files)
        }
    };
    for (index, file) in files.iter().enumerate() {
        fits(&file.path, limits.name_bytes)?;
        fits(&file.content, limits.content_bytes)?;
        valid_path(&file.path)?;
        for before in files.get(..index).expect("preceding changes") {
            if before.path == file.path {
                return Err(Error::Refused);
            }
        }
        match (tree.get(&file.path), file.expected) {
            (None, None) => {}
            (Some(content), Some(expected)) => {
                if crate::trees::blob_id(content) != expected {
                    return Err(Error::Conflict);
                }
            }
            (Some(_), None) => return Err(Error::Exists),
            (None, Some(_)) => return Err(Error::Missing(What::File)),
        }
        tree.insert(file.path.clone(), file.content.clone()).or(Err(Error::Full))?;
    }
    let commit = git::store(
        domain,
        Object { parent: tip, merge_parent: None, tree, message: copy_of(message), author: None, committer: None },
    )?;
    let stored = domain.repositories.get_mut(id).expect("named repository");
    stored.has.insert(commit).expect("room for every commit");
    stored.branches.insert(copy_of(target), commit).expect("branch room checked");
    git::moved(domain, env, id, target, if new_branch.is_some() { None } else { tip }, commit, by);
    Ok(commit)
}

fn valid_path(path: &[u8]) -> Result<(), Error> {
    let mut start = 0;
    for at in 0..=path.len() {
        if at == path.len() || path.get(at) == Some(&b'/') {
            let segment = path.get(start..at).expect("path positions");
            if segment.is_empty() || segment == b"." || segment == b".." {
                return Err(Error::Refused);
            }
            start = at.saturating_add(1);
        }
    }
    Ok(())
}
