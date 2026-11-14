//! Names and places for the activator, never values (hearth/docs/design/domain/render/spec.md, 1.3).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Secrets {
    pub files: Box<[SecretFile]>,
    pub passwords: Box<[Password]>,
    pub identity: Box<[Identity]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SecretFile {
    pub name: Box<[u8]>,
    pub path: Box<[u8]>,
    pub mode: u16,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Password {
    pub user: Box<[u8]>,
    pub secret: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Identity {
    pub name: Box<[u8]>,
    pub kind: IdentityKind,
    pub path: Box<[u8]>,
    pub public_path: Option<Box<[u8]>>,
    pub mode: u16,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IdentityKind {
    SshHostKey { algorithm: Algorithm },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Algorithm {
    Ed25519,
    Ecdsa,
    Rsa,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub files: Box<[Named]>,
    pub passwords: Box<[Named]>,
    pub identity: Box<[Named]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Named {
    Added(Box<[u8]>),
    Removed(Box<[u8]>),
    Changed(Box<[u8]>),
}
