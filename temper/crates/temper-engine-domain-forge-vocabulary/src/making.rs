//! Repositories being made (temper/docs/design/domain/forge/adoption.md, 5).
use alloc::boxed::Box;
use skein_lib::Wall;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MakingKey {
    pub forge: u16,
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MakeFrom {
    Empty,
    Copy { address: Box<[u8]> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MakeInto {
    pub project: u32,
    pub home: bool,
    pub host: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MakeStep {
    Admin,
    Org,
    Repository,
    File,
    Adoption,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MakeFailure {
    Taken,
    Empty,
    TimedOut,
    Refused { status: u16 },
    NotAllowed,
    Other,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MakingFailed {
    pub step: MakeStep,
    pub reason: MakeFailure,
    pub words: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MakingState {
    Waiting,
    Making,
    Made { id: u64, branch: Box<[u8]> },
    Adopting { id: u64, branch: Box<[u8]> },
    Failed(MakingFailed),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepositoryMaking {
    pub key: MakingKey,
    pub from: MakeFrom,
    pub into: Option<MakeInto>,
    pub asked: Wall,
    pub copying: bool,
    pub state: MakingState,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MakeRefusal {
    NoSuchForge,
    NotOffered,
    NotOwner,
    Invalid,
    HomeHeld,
    Full,
    NotFailed,
}
