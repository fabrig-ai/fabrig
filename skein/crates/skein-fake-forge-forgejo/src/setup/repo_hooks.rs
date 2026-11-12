use super::{Store, absent, collaborators::repository, empty, encoded, full};
use crate::{Config, Limits, response::Reply};
use alloc::boxed::Box;
use skein_forge::forgejo::{
    document::Hook,
    request::{HookSpec, Page, Repo},
    server::Answer,
};
use skein_lib::{List, Secret};
/// A repository hook used for inspection and, when eligible, signed loopback deliveries.
#[derive(Debug)]
pub struct RepositoryHook {
    pub id: u64,
    pub repository: u32,
    pub url: Box<[u8]>,
    pub secret: Secret,
    pub events: Box<[Box<[u8]>]>,
    pub active: bool,
}
impl Store {
    /// Inspects a repository's hooks in id order, including their private scenario secrets.
    #[must_use]
    pub fn repository_hooks(&self, repository: u32) -> RepositoryHooks<'_> {
        RepositoryHooks { rows: self.repo_hooks.iter(), repository }
    }
    /// Seeds a repository hook without an HTTP exchange.
    pub fn seed_repository_hook(&mut self, repository: u32, hook: HookSpec) -> Option<u64> {
        let id = self.next_repo_hook;
        let next = id.checked_add(1)?;
        self.repo_hooks
            .push(RepositoryHook {
                id,
                repository,
                url: hook.url,
                secret: hook.secret,
                events: hook.events,
                active: hook.active,
            })
            .ok()?;
        self.next_repo_hook = next;
        Some(id)
    }
    /// Removes one scenario hook, leaving every other row in order.
    pub fn delete_repository_hook(&mut self, repository: u32, id: u64) -> bool {
        for index in 0..self.repo_hooks.len() {
            if let Some(hook) = self.repo_hooks.get(index)
                && hook.repository == repository
                && hook.id == id
            {
                super::remove(&mut self.repo_hooks, index);
                return true;
            }
        }
        false
    }
    pub(super) fn list_repo_hooks(&self, at: &Repo, page: Page, config: &Config, limits: &Limits) -> Reply {
        let Some(repository) = repository(config, at) else {
            return absent(limits);
        };
        let (page, cap) = crate::routes::page(config, page);
        let start = usize::try_from(page.saturating_sub(1).saturating_mul(cap)).unwrap_or(usize::MAX);
        let mut rows = List::with_capacity(cap);
        for hook in self.repository_hooks(repository).skip(start).take(usize::try_from(cap).unwrap_or(0)) {
            if rows.push(document(hook)).is_err() {
                return full(limits);
            }
        }
        encoded(200, Answer::Hooks(rows.into_boxed()), limits)
    }
    pub(super) fn create_repo_hook(&mut self, at: &Repo, hook: &HookSpec, config: &Config, limits: &Limits) -> Reply {
        let Some(repository) = repository(config, at) else {
            return absent(limits);
        };
        if self.repo_hooks.room() == 0 {
            return full(limits);
        }
        let id = self.next_repo_hook;
        let Some(next) = id.checked_add(1) else {
            return full(limits);
        };
        let row = RepositoryHook {
            id,
            repository,
            url: hook.url.clone(),
            secret: hook.secret.clone(),
            events: hook.events.clone(),
            active: hook.active,
        };
        let answer = document(&row);
        if self.repo_hooks.push(row).is_err() {
            return full(limits);
        }
        self.next_repo_hook = next;
        encoded(201, Answer::Hook(answer), limits)
    }
    pub(super) fn edit_repo_hook(
        &mut self,
        at: &Repo,
        id: u64,
        spec: &HookSpec,
        config: &Config,
        limits: &Limits,
    ) -> Reply {
        let Some(repository) = repository(config, at) else {
            return absent(limits);
        };
        for index in 0..self.repo_hooks.len() {
            if let Some(hook) = self.repo_hooks.get_mut(index)
                && hook.repository == repository
                && hook.id == id
            {
                // Forgejo keeps the secret the hook was made with (forge/setup/forgejo.md, 2).
                hook.url.clone_from(&spec.url);
                hook.events.clone_from(&spec.events);
                hook.active = spec.active;
                return encoded(200, Answer::Hook(document(hook)), limits);
            }
        }
        absent(limits)
    }
    pub(super) fn remove_repo_hook(&mut self, at: &Repo, id: u64, config: &Config, limits: &Limits) -> Reply {
        let Some(repository) = repository(config, at) else {
            return absent(limits);
        };
        for index in 0..self.repo_hooks.len() {
            if let Some(hook) = self.repo_hooks.get(index)
                && hook.repository == repository
                && hook.id == id
            {
                super::remove(&mut self.repo_hooks, index);
                return empty();
            }
        }
        absent(limits)
    }
}
fn document(hook: &RepositoryHook) -> Hook {
    Hook { id: hook.id, url: hook.url.clone(), events: hook.events.clone(), active: hook.active }
}

/// A repository's stored hooks, in creation id order for deterministic inspection.
#[derive(Debug)]
pub struct RepositoryHooks<'a> {
    rows: core::slice::Iter<'a, RepositoryHook>,
    repository: u32,
}
impl<'a> Iterator for RepositoryHooks<'a> {
    type Item = &'a RepositoryHook;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let hook = self.rows.next()?;
            if hook.repository == self.repository {
                return Some(hook);
            }
        }
    }
}
