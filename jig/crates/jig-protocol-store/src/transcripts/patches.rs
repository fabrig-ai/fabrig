//! Resident frame machines, bounded independently of map records (protocol/store/transcripts.md, 6, protocol/store/README.md, 8).
use crate::{Frames, Limits, Owner, OwnerSets, codec, upgrade::schemas};
use alloc::boxed::Box;
use skein_codec::Record as _;
use skein_codec::{
    patch::{Capacity, Halt, Patcher},
    schema::{self, DeclIx, FieldIx, Schema, Shape, Type, TypeIx, VariantIx},
};
use skein_lib::List;

#[derive(Debug)]
pub(crate) struct Machine {
    pub patcher: Option<Box<Patcher>>,
    pub root: &'static str,
    pub last: u16,
    pub capacity: Capacity,
}
#[derive(Debug)]
pub(crate) struct Machines {
    pub body: Option<Machine>,
    pub turn: Option<Machine>,
}
impl Machines {
    pub(crate) fn new() -> Self {
        Self { body: None, turn: None }
    }
}

pub(crate) fn make(sets: &OwnerSets, limits: &Limits, prepared: &[Schema]) -> Result<Machines, Owner> {
    let mut machines = Machines::new();
    for set in sets.sets {
        for (ix, family) in set.families.iter().enumerate() {
            let Some(frames) = &family.frames else { continue };
            let root = frames.root;
            if frames.texts.is_empty() {
                continue;
            }
            let refusal = set.owner;
            let capacity = capacity(set, ix, limits, prepared).ok_or(refusal)?;
            let patcher = kept_patcher(frames, capacity, prepared).ok_or(refusal)?;
            let last = patcher.schemas().last().ok_or(refusal)?.version();
            let machine = Machine { patcher: Some(patcher), root, last, capacity };
            match set.owner {
                Owner::Jig => machines.body = Some(machine),
                Owner::Smith => machines.turn = Some(machine),
                Owner::Skein | Owner::Application => return Err(refusal),
            }
        }
    }
    Ok(machines)
}

pub(crate) fn capacity(set: &crate::OwnerSet, ix: usize, limits: &Limits, prepared: &[Schema]) -> Option<Capacity> {
    let family = set.families.get(ix)?;
    let frames = family.frames.as_ref()?;
    let root = frames.root;
    let codec = codec::limits(&limits.core)?;
    let bytes = match set.owner {
        Owner::Jig => u32::try_from(crate::core_v6::TurnRecord::encoded_bound(&codec.core)?).ok()?,
        Owner::Smith => codec.core.turn_record_transcript,
        Owner::Skein | Owner::Application => return None,
    };
    let mut capacity = frame_capacity(frames, Room { bytes, nodes: 0 })?;
    // Either owner's nodes are the most its kept texts allow from the
    // frames' root, so every frame those texts allow is read (protocol/store/transcripts.md 6).
    let meta = kept_patcher(frames, Capacity { bytes: 0, nodes: 0, ..capacity }, prepared)?;
    let last = meta.schemas().last()?.version();
    for schema in meta.schemas() {
        let root = held_root(&meta, schema.version(), last, root.as_bytes()).ok()?;
        capacity.nodes = capacity.nodes.max(node_bound(schema, root)?);
    }
    Some(capacity)
}

/// Schemas name earlier declarations. Dynamic programming counts each named
/// subtree once; a held field is one leaf, with no traversal into its bytes.
pub(crate) fn node_bound(schema: &Schema, root: DeclIx) -> Option<u32> {
    let mut nodes = List::with_capacity(schema.declarations());
    for index in 0..schema.declarations() {
        let size = match schema.declaration(DeclIx::new(index)).shape {
            Shape::Record { first, count, .. } => {
                let mut size = 1_u32;
                for offset in 0..count {
                    let field = schema.field(FieldIx::new(first.index().checked_add(offset)?));
                    size = size.checked_add(type_nodes(schema, field.ty, &nodes)?)?;
                }
                size
            }
            Shape::Enum { first, count } => {
                let mut payload = 0;
                for offset in 0..count {
                    if let Some(decl) = schema.variant(VariantIx::new(first.index().checked_add(offset)?)).record {
                        payload = payload.max(*nodes.get(decl.index())?);
                    }
                }
                payload.checked_add(1)?
            }
        };
        nodes.push(size).ok()?;
    }
    nodes.get(root.index()).copied()
}
fn type_nodes(schema: &Schema, first: TypeIx, nodes: &List<u32>) -> Option<u32> {
    let mut end = first.index();
    for _ in 0..schema.text().len() {
        match schema.ty(TypeIx::new(end)) {
            Type::List { item, .. } | Type::Option(item) => end = item.index(),
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Secret { .. }
            | Type::Text(_)
            | Type::Named(_)
            | Type::Bytes { .. } => break,
        }
    }
    let mut count = 0_u32;
    for index in (first.index()..=end).rev() {
        count = match schema.ty(TypeIx::new(index)) {
            Type::Option(_) => count.checked_add(1)?,
            Type::List { bound, .. } => count.checked_mul(bound)?.checked_add(1)?,
            Type::Named(decl) => *nodes.get(decl.index())?,
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Secret { .. }
            | Type::Text(_)
            | Type::Bytes { .. } => 1,
        };
    }
    Some(count)
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Room {
    pub bytes: u32,
    pub nodes: u32,
}

pub(crate) fn frame_capacity(frames: &Frames, room: Room) -> Option<Capacity> {
    let versions = u32::try_from(frames.texts.len()).ok()?;
    let patches = u32::try_from(frames.patches.len()).ok()?;
    let mut text = 0_usize;
    for kept in frames.texts {
        text = text.max(kept.len());
    }
    for patch in frames.patches {
        text = text.max(patch.len());
    }
    // Patcher::new requires the language's ceiling; keep/plan name the
    // offending kept text or patch if the actual text exceeds it.
    let text = u32::try_from(text).ok()?.min(schema::MAX_TEXT);
    Some(Capacity { versions, patches, text, bytes: room.bytes, nodes: room.nodes })
}

pub(crate) fn kept_patcher(frames: &Frames, capacity: Capacity, prepared: &[Schema]) -> Option<Box<Patcher>> {
    let mut machine = Box::new(Patcher::new(capacity));
    let mut previous = None;
    for text in frames.texts {
        let text = text.as_bytes();
        let version = if let Some(index) = schemas::matching(prepared, text) {
            machine.keep_schema(prepared.get(index)?.clone())
        } else {
            machine.keep(text)
        };
        let version = version.ok()?;
        if let Some(previous) = previous
            && Some(version) != u16::checked_add(previous, 1)
        {
            return None;
        }
        previous = Some(version);
    }
    for text in frames.patches {
        if machine.plan(text.as_bytes()).is_err() {
            return None;
        }
    }
    for schema in machine.schemas() {
        let version = schema.version();
        if Some(version) != previous && machine.plan_from(version).is_none() {
            return None;
        }
    }
    Some(machine)
}

pub(crate) fn held_root(patcher: &Patcher, version: u16, reached: u16, name: &[u8]) -> Result<DeclIx, Halt> {
    if let Some(schema) = patcher.schema(version)
        && let Some(root) = schema.find(name)
    {
        return Ok(root);
    }
    let mut root = patcher.schema(reached).ok_or(Halt::Order)?.find(name).ok_or(Halt::Order)?;
    // A held name may spell the reached declaration after a rename. Walk
    // sources back to the input version; forward passes use counterparts.
    let passes = reached.checked_sub(version).ok_or(Halt::Order)?;
    for offset in 0..passes {
        let from = reached.checked_sub(offset).expect("offset below reached").checked_sub(1).expect("previous version");
        let plan = patcher.plan_from(from).ok_or(Halt::Order)?;
        root = patcher.source(plan, root).ok_or(Halt::Order)?;
    }
    Ok(root)
}
