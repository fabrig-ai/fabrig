//! The packages record and its four resolution inputs (hearth/docs/design/domain/render/spec.md, 1.2, 2.1).
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
mod change;
pub use change::*;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Packages {
    pub initramfs_file: Option<Digest>,
    pub sources: Sources,
    pub request: Box<[Box<[u8]>]>,
    pub variant: Variant,
    pub preseeds: Box<[Preseed]>,
    pub initramfs: Option<Initramfs>,
    pub apt: Apt,
    pub seeds: Seeds,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Seeds {
    pub users: Box<[SeedUser]>,
    pub groups: Box<[SeedGroup]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedUser {
    pub name: Box<[u8]>,
    pub uid: u32,
    pub gid: Option<u32>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedGroup {
    pub name: Box<[u8]>,
    pub gid: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sources {
    pub suite: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub components: Box<[Component]>,
    pub archive: Box<[u8]>,
    pub updates: bool,
    pub security: Option<Box<[u8]>>,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Component {
    Main,
    Contrib,
    NonFree,
    NonFreeFirmware,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Variant {
    Apt,
    Container,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Apt {
    pub install_recommends: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Preseed {
    pub question: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub kind: PreseedKind,
    pub value: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PreseedKind {
    Select,
    Multiselect,
    String,
    Boolean,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Initramfs {
    pub modules: Box<[Box<[u8]>]>,
    pub microcode: Microcode,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Microcode {
    None,
    Generic,
}
impl Component {
    pub const ALL: [Self; 4] = [Self::Main, Self::Contrib, Self::NonFree, Self::NonFreeFirmware];
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Main => b"main",
            Self::Contrib => b"contrib",
            Self::NonFree => b"non-free",
            Self::NonFreeFirmware => b"non-free-firmware",
        }
    }
}
impl Variant {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Apt => b"apt",
            Self::Container => b"container",
        }
    }
}
impl PreseedKind {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Select => b"select",
            Self::Multiselect => b"multiselect",
            Self::String => b"string",
            Self::Boolean => b"boolean",
        }
    }
}
impl Microcode {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::None => b"none",
            Self::Generic => b"generic",
        }
    }
}
