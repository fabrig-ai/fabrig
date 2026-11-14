//! Software downloads and scripts; fetch hints stay outside the digest (hearth/docs/design/domain/render/spec.md, 1.2).
pub use super::config::Script;
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Software {
    pub downloads: Box<[Download]>,
    pub scripts: Box<[Script]>,
    pub fetch: Box<[Fetch]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Download {
    pub path: Box<[u8]>,
    pub sha256: Digest,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
    pub how: How,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum How {
    File { mode: u16 },
    Tar { strip: u32 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fetch {
    pub sha256: Digest,
    pub url: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub downloads: Box<[DownloadKeyed]>,
    pub scripts: Option<super::config::ScriptsChange>,
}
/// Keyed changes (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DownloadKeyed {
    Added { key: Box<[u8]>, value: Download },
    Removed { key: Box<[u8]>, value: Download },
    Changed { key: Box<[u8]>, before: Download, after: Download },
}
