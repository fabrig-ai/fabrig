//! Repository rows and user logins, evicted by least recent use (forge/README.md, 4.2).
use crate::users::Users;
use crate::{api, github::document};
use alloc::boxed::Box;
use skein_lib::Lru;

/// The first merge method the repository allows (forge/github.md, 5.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Method {
    Merge,
    Squash,
    Rebase,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repository {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub owner_id: u64,
    pub organization: bool,
    /// `None` when no method is allowed; no disabled method is chosen as a default.
    pub merge: Option<Method>,
    pub html_url: Box<[u8]>,
    pub settings: api::Settings,
}
#[derive(Debug)]
pub struct Tables {
    repositories: Lru<api::Repository, Repository>,
    users: Users,
}
impl Tables {
    #[must_use]
    pub fn new(repositories: u32, users: u32) -> Self {
        Self { repositories: Lru::with_capacity(repositories), users: Users::with_capacity(users) }
    }
    #[must_use]
    pub fn repository(&mut self, key: api::Repository) -> Option<&Repository> {
        self.repositories.get(key)
    }
    pub fn keep_repository(&mut self, key: api::Repository, row: Repository) {
        drop(self.repositories.insert(key, row));
    }
    pub fn drop_repository(&mut self, key: api::Repository) {
        drop(self.repositories.remove(key));
    }
    #[must_use]
    pub fn user(&mut self, forge: u16, id: u64) -> Option<&[u8]> {
        Some(self.users.get((forge, id))?.as_ref())
    }
    pub fn keep_user(&mut self, forge: u16, id: u64, login: Box<[u8]>) {
        drop(self.users.insert((forge, id), login));
    }
}
/// Copies GitHub's routing and owner fields, choosing merge, then squash, then rebase.
#[must_use]
pub fn repository(row: &document::Repository) -> Repository {
    let merge = if row.merge {
        Some(Method::Merge)
    } else if row.squash {
        Some(Method::Squash)
    } else if row.rebase {
        Some(Method::Rebase)
    } else {
        None
    };
    Repository {
        owner: row.owner.clone(),
        name: row.name.clone(),
        owner_id: row.owner_id,
        organization: row.organization,
        merge,
        html_url: row.html_url.clone(),
        settings: api::Settings {
            default_branch: row.default_branch.clone(),
            merge: row.merge,
            squash: row.squash,
            rebase: row.rebase,
        },
    }
}
