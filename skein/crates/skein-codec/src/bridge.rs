//! Kept wire versions, checked reverses and lossless translation (data/codec/bridges.md, 5).

use crate::patch::{Build, Capacity, Halt, KeepError, Old, PatchError, PatchWrong, Patcher, Site, Step};
use crate::schema::{DeclIx, Span};
use crate::value::Node;
use alloc::boxed::Box;
use skein_lib::{List, bytes};

/// One owner's text-numbered written dispatch, serving both directions.
pub type Sites = fn(Site, &Old, &mut Build, Node) -> Result<(), Halt>;

/// Why a family's bridge could not be made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BridgeError {
    /// A kept schema was refused.
    Keep(KeepError),
    /// A forward block was refused.
    Plan { patch: u32, error: PatchError },
    /// A reverse needed above the floor was refused.
    Reverse { version: u16, error: PatchError },
    /// The floor is outside the kept versions.
    Floor,
    /// The caller's capacity cannot hold the bridge.
    Room,
}

/// A family's kept schemas and plans; per-record workspace is reused.
#[derive(Debug)]
pub struct Bridge {
    patcher: Patcher,
    first: u16,
    floor: u16,
    current: u16,
    sites: Sites,
    nodes: u32,
}

impl Bridge {
    /// Keeps this family's texts and plans all forward blocks and the reverses its floor needs.
    #[expect(clippy::similar_names, reason = "the public signature names patches; patcher is their machine")]
    pub fn new(
        capacity: Capacity,
        texts: &[&str],
        patches: &[&str],
        floor: u16,
        sites: Sites,
    ) -> Result<Bridge, BridgeError> {
        if capacity.text > crate::schema::MAX_TEXT {
            return Err(BridgeError::Room);
        }
        let mut patcher = Patcher::new(capacity);
        let mut first = None;
        let mut current = None;
        for text in texts {
            let version = match patcher.keep(text.as_bytes()) {
                Ok(version) => version,
                Err(error) => return Err(BridgeError::Keep(error)),
            };
            if let Some(previous) = current {
                if u16::checked_add(previous, 1) != Some(version) {
                    return Err(BridgeError::Floor);
                }
            } else {
                first = Some(version);
            }
            current = Some(version);
        }
        let first = first.ok_or(BridgeError::Floor)?;
        let current = current.ok_or(BridgeError::Floor)?;
        if floor < first || floor > current {
            return Err(BridgeError::Floor);
        }
        for text in patches {
            let planned = match patcher.plan(text.as_bytes()) {
                Ok(plan) => plan,
                Err(error) => {
                    let patch = match crate::patch::header(text.as_bytes()) {
                        Ok(header) => header.level(),
                        Err(_) => 0,
                    };
                    return Err(BridgeError::Plan { patch, error });
                }
            };
            if let Some(plan) = planned {
                let version = patcher.versions(plan).to;
                if version > floor {
                    match patcher.reverse(text.as_bytes()) {
                        Ok(Some(_)) => {}
                        Ok(None) => return Err(BridgeError::Reverse { version, error: missing() }),
                        Err(error) => return Err(BridgeError::Reverse { version, error }),
                    }
                }
            }
        }
        for version in first..current {
            if patcher.plan_from(version).is_none() {
                return Err(BridgeError::Plan { patch: 0, error: missing() });
            }
        }
        Ok(Bridge { patcher, first, floor, current, sites, nodes: capacity.nodes })
    }

    /// The kept family's name.
    #[must_use]
    pub fn family(&self) -> &[u8] {
        let schema = self.patcher.schema(self.current).expect("bridge keeps current");
        schema.spelled(schema.family())
    }

    /// The oldest version this bridge reads.
    #[must_use]
    pub const fn first(&self) -> u16 {
        self.first
    }

    /// The oldest version this bridge speaks.
    #[must_use]
    pub const fn floor(&self) -> u16 {
        self.floor
    }

    /// The newest kept version.
    #[must_use]
    pub const fn current(&self) -> u16 {
        self.current
    }

    /// The family fingerprint of a kept version.
    #[must_use]
    pub fn layout(&self, version: u16) -> Option<[u8; 32]> {
        Some(self.patcher.schema(version)?.fingerprints().family().digest())
    }

    fn root_at(&self, version: u16, root: &[u8]) -> Option<DeclIx> {
        let mut decl = self.patcher.schema(self.current)?.find(root)?;
        for from in (version..self.current).rev() {
            decl = self.patcher.source(self.patcher.plan_from(from)?, decl)?;
        }
        Some(decl)
    }

    /// Converts a record, refusing a down translation unless its forward round trip is identical.
    pub fn translate(&mut self, from: u16, to: u16, root: &[u8], record: Box<[u8]>) -> Result<Box<[u8]>, Halt> {
        if from < self.first || from > self.current || to < self.floor || to > self.current {
            return Err(Halt::Order);
        }
        if from == to {
            return Ok(record);
        }
        let source = self.root_at(from, root).ok_or(Halt::Order)?;
        let comparison = if from > to { Some(bytes::copy_of(&record)) } else { None };
        self.passes(from, to, source, record)?;
        let result = bytes::copy_of(self.patcher.finish()?);
        if let Some(comparison) = comparison {
            let source = self.root_at(to, root).ok_or(Halt::Order)?;
            self.follow(to, from, source)?;
            if self.patcher.finish()? != comparison.as_ref() {
                return Err(Halt::Altered);
            }
        }
        Ok(result)
    }

    fn passes(&mut self, from: u16, to: u16, root: DeclIx, record: Box<[u8]>) -> Result<(), Halt> {
        let plan = if from < to { self.patcher.plan_from(from) } else { self.patcher.reverse_from(from) }
            .ok_or(Halt::Order)?;
        self.patcher.start(plan, root, record)?;
        self.answer()?;
        self.patcher.finish()?;
        let root = self.patcher.counterpart(plan, root).ok_or(Halt::Order)?;
        self.follow(self.patcher.versions(plan).to, to, root)
    }

    fn follow(&mut self, from: u16, to: u16, mut root: DeclIx) -> Result<(), Halt> {
        let mut version = from;
        for _ in self.first..=self.current {
            if version == to {
                return Ok(());
            }
            let plan = if version < to { self.patcher.plan_from(version) } else { self.patcher.reverse_from(version) }
                .ok_or(Halt::Order)?;
            self.patcher.next_pass(plan, root)?;
            self.answer()?;
            self.patcher.finish()?;
            root = self.patcher.counterpart(plan, root).ok_or(Halt::Order)?;
            version = self.patcher.versions(plan).to;
        }
        Err(Halt::Order)
    }

    fn answer(&mut self) -> Result<(), Halt> {
        // Every stop consumes at least one of the capacity-bounded nodes.
        for _ in 0..=self.nodes {
            match self.patcher.next()? {
                Step::Done => return Ok(()),
                Step::Written { site, at } => {
                    let (old, new) = self.patcher.parts();
                    (self.sites)(site, old, new, at)?;
                }
                Step::Held { at, .. } => {
                    let (old, new) = self.patcher.parts();
                    new.carry(old, at)?;
                }
            }
        }
        Err(Halt::Room)
    }

    /// Whether every reverse from current to `to` is total for this current-named root.
    #[must_use]
    pub fn total(&self, to: u16, root: &[u8]) -> Option<bool> {
        if to < self.floor || to > self.current {
            return None;
        }
        let mut root = self.patcher.schema(self.current)?.find(root)?;
        let mut total = true;
        for version in (to..self.current).rev() {
            let plan = self.patcher.reverse_from(version.checked_add(1)?)?;
            total &= self.patcher.total(plan, root);
            root = self.patcher.counterpart(plan, root)?;
        }
        Some(total)
    }
}

fn missing() -> PatchError {
    PatchError { line: 1, at: Span { start: 0, end: 0 }, wrong: PatchWrong::Unkept }
}

/// Family fingerprints in kept-version order from the floor through current.
pub fn layouts(texts: &[&str], floor: u16) -> Result<List<[u8; 32]>, BridgeError> {
    let Ok(versions) = u32::try_from(texts.len()) else {
        return Err(BridgeError::Room);
    };
    let mut patcher =
        Patcher::new(Capacity { versions, patches: 0, text: crate::schema::MAX_TEXT, bytes: 0, nodes: 0 });
    let mut result = List::with_capacity(versions);
    let mut previous = None;
    let mut found = false;
    for text in texts {
        let version = match patcher.keep(text.as_bytes()) {
            Ok(version) => version,
            Err(error) => return Err(BridgeError::Keep(error)),
        };
        if let Some(previous) = previous
            && u16::checked_add(previous, 1) != Some(version)
        {
            return Err(BridgeError::Floor);
        }
        previous = Some(version);
        found |= version == floor;
        if version >= floor {
            let schema = patcher.schema(version).expect("text just kept");
            result.push(schema.fingerprints().family().digest()).expect("kept version room");
        }
    }
    if !found {
        return Err(BridgeError::Floor);
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
