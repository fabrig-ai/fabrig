//! Empty destinations and deferred copies (skein/docs/design/testing/fakes/forge.md, 2.6).

use crate::{
    Config, Domain,
    api::{Answer, Error, Permission, What},
    store::{Repository, fits},
};
use skein_lib::{Env, Id, Map, Set, Writer, bytes::copy_of};

/// Admits the destination without allocating any commit or moving a head.
pub(crate) fn start(
    domain: &mut Domain,
    env: &Env<Config>,
    source: &[u8],
    user: u64,
    owner: &[u8],
    name: &[u8],
) -> Result<(Id<Repository>, Id<Repository>, Answer), Error> {
    let source = *domain.names.get(source).ok_or(Error::Missing(What::Repository))?;
    let stored = domain.repositories.get(source).expect("named source");
    stored.require(user, Permission::Read)?;
    let limits = &env.limits.limits;
    fits(owner, limits.name_bytes)?;
    fits(name, limits.name_bytes)?;
    let length = owner.len().checked_add(1).ok_or(Error::TooLarge)?;
    let length = length.checked_add(name.len()).ok_or(Error::TooLarge)?;
    let mut full = Writer::new(length);
    full.put(owner).expect("sized owner");
    full.put(b"/").expect("sized separator");
    full.put(name).expect("sized name");
    let full = full.finish();
    fits(&full, limits.name_bytes)?;
    if domain.names.contains_key(&*full) {
        return Err(Error::Exists);
    }
    if domain.repositories.is_full() || stored.has.len() > limits.commits {
        return Err(Error::Full);
    }
    let default = copy_of(&stored.default);
    let mut last = 0;
    for (_, repository) in &domain.repositories {
        last = last.max(repository.id);
    }
    let id = last.checked_add(1).ok_or(Error::Full)?;
    let checks = env.limits.made_checks.clone();
    let mut destination = Repository::empty(limits, copy_of(&full), copy_of(&default), checks, None, false);
    destination.id = id;
    destination.permissions.insert(user, Permission::Admin).expect("owner permission room");
    let destination = domain.repositories.insert(destination).expect("checked repository room");
    domain.names.insert(full, destination).expect("one name per retained repository");
    Ok((source, destination, Answer::Copied { repository: id, default_branch: default }))
}

/// Fills a live destination from a live source, without CI, hooks or observations.
pub(crate) fn fill(domain: &mut Domain, source: Id<Repository>, destination: Id<Repository>) -> Result<Answer, Error> {
    let stored = domain.repositories.get(source).expect("retained source slot");
    if stored.gone {
        return Err(Error::Missing(What::Repository));
    }
    let mut branches = Map::with_capacity(stored.branches.capacity());
    for (name, &commit) in &stored.branches {
        branches.insert(copy_of(name), commit).expect("source branches fit");
    }
    let mut has = Set::with_capacity(stored.has.capacity());
    for &commit in &stored.has {
        has.insert(commit).expect("source commits fit");
    }
    let default = copy_of(&stored.default);
    let stored = domain.repositories.get_mut(destination).expect("retained destination slot");
    if stored.gone {
        return Err(Error::Missing(What::Repository));
    }
    stored.branches = branches;
    stored.has = has;
    stored.default = default;
    Ok(Answer::Copied { repository: stored.id, default_branch: copy_of(&stored.default) })
}

/// Keeps the slot for alarms and drops its public name.
pub(crate) fn delete(domain: &mut Domain, id: Id<Repository>, user: u64) -> Result<Answer, Error> {
    let stored = domain.repositories.get_mut(id).expect("named repository");
    stored.require(user, Permission::Admin)?;
    stored.gone = true;
    domain.names.remove(&*stored.name);
    Ok(Answer::Done)
}
