//! Resolution and layer keys (hearth/docs/design/domain/render/spec.md, 1.5, 2.1).
use hearth_lib::encode::Digest;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Keys {
    pub packages: Digest,
    pub software: Digest,
    pub config: Digest,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Resolution {
    Needed { key: Digest },
    Known { key: Digest, keys: Keys },
}
impl Resolution {
    #[must_use]
    pub const fn key(&self) -> Digest {
        match self {
            Self::Needed { key } | Self::Known { key, .. } => *key,
        }
    }
}
