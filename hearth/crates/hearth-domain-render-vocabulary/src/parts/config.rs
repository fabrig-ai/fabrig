//! The static file tree's canonical record (hearth/docs/design/domain/render/spec.md, 1.2).
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
mod accounts;
pub use accounts::{Group, GroupKind, Member, User, UserKind};
mod directives;
pub use directives::{Action, Line, LineKind, Preset, Script, Tmpfile};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Config {
    pub presets: Box<[Preset]>,
    pub tmpfiles: Box<[Tmpfile]>,
    pub scripts: Box<[Script]>,
    pub groups: Box<[Group]>,
    pub users: Box<[User]>,
    pub members: Box<[Member]>,
    pub tree: Box<[TreeEntry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeEntry {
    pub path: Box<[u8]>,
    pub node: Node,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Node {
    Link { target: Box<[u8]>, owner: Box<[u8]>, group: Box<[u8]> },
    Directory { mode: u16, owner: Box<[u8]>, group: Box<[u8]> },
    File { content_digest: Digest, mode: u16, owner: Box<[u8]>, group: Box<[u8]> },
}
/// Paths changed in byte order (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub presets: Box<[PresetKeyed]>,
    pub tmpfiles: Box<[LineKeyed]>,
    pub scripts: Option<ScriptsChange>,
    pub groups: Box<[GroupKeyed]>,
    pub users: Box<[UserKeyed]>,
    pub members: Box<[MemberChange]>,
    pub tree: Box<[EntryChange]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EntryChange {
    Added { path: Box<[u8]>, node: Node },
    Removed { path: Box<[u8]>, node: Node },
    Changed { path: Box<[u8]>, before: Node, after: Node, sizes: Option<Sizes>, hunks: Option<crate::diff::Hunks> },
}
/// File sizes before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Sizes {
    pub before: u64,
    pub after: u64,
}
/// Changes in key order (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GroupKeyed {
    Added { key: Box<[u8]>, value: GroupKind },
    Removed { key: Box<[u8]>, value: GroupKind },
    Changed { key: Box<[u8]>, before: GroupKind, after: GroupKind },
}
/// Changes in key order (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UserKeyed {
    Added { key: Box<[u8]>, value: UserKind },
    Removed { key: Box<[u8]>, value: UserKind },
    Changed { key: Box<[u8]>, before: UserKind, after: UserKind },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MemberChange {
    Added(Member),
    Removed(Member),
}
/// Changes in key order (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PresetKeyed {
    Added { key: Box<[u8]>, value: Action },
    Removed { key: Box<[u8]>, value: Action },
    Changed { key: Box<[u8]>, before: Action, after: Action },
}
/// Changes in key order (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LineKeyed {
    Added { key: Box<[u8]>, value: Line },
    Removed { key: Box<[u8]>, value: Line },
    Changed { key: Box<[u8]>, before: Line, after: Line },
}
/// Scripts before and after (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ScriptsChange {
    pub before: Box<[Script]>,
    pub after: Box<[Script]>,
}
