//! Render values (hearth/docs/design/domain/render/README.md, 5.1).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Group {
    pub name: Box<[u8]>,
    pub kind: GroupKind,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GroupKind {
    Sysusers { gid: Option<u32> },
    Package,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub name: Box<[u8]>,
    pub kind: UserKind,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UserKind {
    Sysusers { uid: Option<u32>, group: Box<[u8]>, gecos: Box<[u8]>, home: Box<[u8]>, shell: Box<[u8]> },
    Package,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Member {
    pub user: Box<[u8]>,
    pub group: Box<[u8]>,
}
