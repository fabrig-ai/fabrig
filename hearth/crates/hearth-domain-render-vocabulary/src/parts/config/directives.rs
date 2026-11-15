//! Render values (hearth/docs/design/domain/render/README.md, 5.1).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Preset {
    pub unit: Box<[u8]>,
    pub action: Action,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Enable,
    Disable,
}
impl Action {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Enable => b"enable",
            Self::Disable => b"disable",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tmpfile {
    pub path: Box<[u8]>,
    pub line: Line,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub kind: LineKind,
    pub mode: Option<u16>,
    pub owner: Option<Box<[u8]>>,
    pub group: Option<Box<[u8]>>,
    pub age: Option<Box<[u8]>>,
    pub argument: Option<Box<[u8]>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LineKind {
    D,
    CapitalD,
    E,
    F,
    FPlus,
    W,
    WPlus,
    CapitalL,
    CapitalLPlus,
    CapitalC,
    Z,
    CapitalZ,
    R,
    CapitalR,
}
impl LineKind {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::D => b"d",
            Self::CapitalD => b"D",
            Self::E => b"e",
            Self::F => b"f",
            Self::FPlus => b"f+",
            Self::W => b"w",
            Self::WPlus => b"w+",
            Self::CapitalL => b"L",
            Self::CapitalLPlus => b"L+",
            Self::CapitalC => b"C",
            Self::Z => b"z",
            Self::CapitalZ => b"Z",
            Self::R => b"r",
            Self::CapitalR => b"R",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Script {
    pub task: Box<[u8]>,
    pub order: u16,
    pub content_digest: hearth_lib::encode::Digest,
}
