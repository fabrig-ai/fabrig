//! Application roots on the worker channel (jig/docs/design/protocol/channel/bridges.md, 8).
use crate::Spoken;
use skein_codec::{
    bridge::{Bridge, BridgeError, Sites},
    patch::Capacity,
    schema::{DeclIx, MAX_TEXT},
};
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carried {
    Items,
    Left,
    Saved,
    Preparation,
    JobSpec,
    JobResult,
    Grant,
    Ask,
    Answer,
}

#[derive(Clone, Copy, Debug)]
pub struct Root {
    pub carried: Carried,
    pub wire: &'static str,
    pub declaration: &'static str,
}

#[derive(Debug)]
pub struct Application {
    pub spoken: List<Spoken>,
    pub kept: List<Kept>,
    pub roots: List<Root>,
}
impl Application {
    /// An application whose opaque parts cross unchanged.
    #[must_use]
    pub fn opaque(spoken: List<Spoken>) -> Self {
        Self { spoken, kept: List::uncapped(), roots: List::uncapped() }
    }
}

/// A wire's kept versions and the limits used to size its bridge (bridges.md, 8).
#[derive(Clone, Debug)]
pub struct Kept {
    pub family: &'static str,
    pub texts: &'static [&'static str],
    pub patches: &'static [&'static str],
    pub floor: u16,
    pub sites: Sites,
    pub names: &'static [&'static str],
    pub values: List<u32>,
}
impl Kept {
    /// Plans the wire's bridge within its texts' largest declaration bound.
    pub fn bridge(&self) -> Result<Bridge, BridgeError> {
        let versions = u32::try_from(self.texts.len()).or(Err(BridgeError::Room))?;
        let mut schemas = List::with_capacity(versions);
        let mut bytes = 0_u32;
        for text in self.texts {
            let schema = skein_codec::schema::parse(text.as_bytes()).or(Err(BridgeError::Floor))?;
            let bounds = schema.bounds_within(self.names, self.values.as_slice());
            for declaration in 0..schema.declarations() {
                let bound = bounds.at(DeclIx::new(declaration));
                bytes = bytes.max(u32::try_from(bound.ok_or(BridgeError::Room)?).or(Err(BridgeError::Room))?);
            }
            schemas.push(schema).expect("one schema per kept text");
        }
        // Eight nodes per byte covers records, fields and list entries,
        // including their unencoded enclosing nodes.
        let capacity = Capacity {
            versions,
            patches: u32::try_from(self.patches.len())
                .or(Err(BridgeError::Room))?
                .checked_mul(2)
                .ok_or(BridgeError::Room)?,
            text: MAX_TEXT,
            bytes,
            nodes: bytes.checked_mul(8).ok_or(BridgeError::Room)?,
        };
        Bridge::from_schemas(capacity, schemas, self.patches, self.floor, self.sites)
    }
}
