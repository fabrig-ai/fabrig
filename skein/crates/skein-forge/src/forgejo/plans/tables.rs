//! Repository paths and user logins, evicted by least recent use (forge/README.md, 4.2).
use crate::api;
use crate::forgejo::{
    document::MergeStyle,
    id::{ObjectFormat, ObjectId},
    request::Repo,
};
use crate::users::Users;
use alloc::boxed::Box;
use skein_lib::Lru;
/// The repository fields needed to address calls and read their answers.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repository {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub object_format: ObjectFormat,
    pub merge_style: MergeStyle,
    pub html_url: Box<[u8]>,
    pub settings: api::Settings,
}
impl Repository {
    #[must_use]
    pub fn at(&self) -> Repo {
        Repo { owner: self.owner.clone(), name: self.name.clone() }
    }
    #[must_use]
    pub fn id(&self, bytes: api::Commit) -> ObjectId {
        ObjectId { bytes, format: self.object_format }
    }
}
/// The two bounded lookup tables. Forge starts hold their page caps beside settings' tokens.
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
