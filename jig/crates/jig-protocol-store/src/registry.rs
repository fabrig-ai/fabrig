//! Layouts kept for the serving open (protocol/store/layouts.md, 3–7).
use crate::{Job, JobLayout, OpenRefusal, Owner, Store};
use alloc::boxed::Box;
use skein_codec::{
    layout::{Misconverted, Plan, Workspace},
    schema::Schema,
};
use skein_lib::{Buffer, List, Map, Queue};
mod check;
pub use check::{HeldLayout, judge_held_layout};
#[cfg(test)]
mod counting;
mod depth;
pub(crate) mod rehearsal;
mod rows;
pub(crate) mod settling;
#[cfg(test)]
mod tests;
mod texts;

/// Workspace fitted at open beside the plans; the buffer grows only on
/// converted reads, up to the value bound (protocol/store/layouts.md, 7).
#[derive(Debug)]
pub(crate) struct ReadWork {
    pub(crate) workspace: Workspace,
    buffer: Buffer,
    bound: u32,
}
impl ReadWork {
    pub(crate) fn new() -> Self {
        Self { workspace: Workspace::new(), buffer: Buffer::with_capacity(0), bound: 0 }
    }
    fn fit(workspace: Workspace, bound: u32) -> Self {
        Self { workspace, buffer: Buffer::with_capacity(0), bound }
    }
    fn convert(&mut self, plan: &Plan, value: &[u8]) -> Result<(), Misconverted> {
        if !self.workspace.fits(plan) {
            return Err(Misconverted::Room);
        }
        let length = u32::try_from(value.len()).unwrap_or(self.bound);
        let twice = length.checked_mul(2).unwrap_or(self.bound);
        let room = twice.checked_next_power_of_two().unwrap_or(self.bound).max(4096).min(self.bound);
        if self.buffer.capacity() < room {
            self.buffer = Buffer::with_capacity(room);
        }
        // Even a Room unrelated to output space stops at the bound: at most 32 doublings of a u32 capacity.
        for _ in 0..=u32::BITS {
            match plan.convert(value, &mut self.buffer, &mut self.workspace) {
                Err(Misconverted::Room) if self.buffer.capacity() < self.bound => {
                    let room = self.buffer.capacity().checked_mul(2).unwrap_or(self.bound).min(self.bound);
                    self.buffer = Buffer::with_capacity(room);
                }
                Ok(()) => return Ok(()),
                Err(
                    error @ (Misconverted::Short
                    | Misconverted::Tag
                    | Misconverted::Bool
                    | Misconverted::Bound
                    | Misconverted::Trailing
                    | Misconverted::Room
                    | Misconverted::Refused(_)),
                ) => return Err(error),
            }
        }
        Err(Misconverted::Room)
    }
}

type Digest = [u8; 32];
type RowKey = (Box<str>, u16, u16);
type OrderKey = (Box<str>, u16);
#[derive(Debug)]
struct Text {
    bytes: Box<[u8]>,
    schema: Option<texts::Cached>,
    verified: bool,
    stored: bool,
    current: bool,
}
impl Text {
    fn stored(bytes: Box<[u8]>) -> Self {
        Self { bytes, schema: None, verified: false, stored: true, current: false }
    }

    fn schema(&self) -> Option<&Schema> {
        match self.schema.as_ref()? {
            texts::Cached::Parsed(schema) => Some(schema),
            texts::Cached::Prepared(schema) => Some(schema),
        }
    }
}
#[derive(Debug)]
struct Row {
    value: crate::registry_v1::Layout,
    stored: bool,
    retire: bool,
}
#[derive(Debug)]
struct Order {
    value: crate::registry_v1::Order,
    dirty: bool,
}
#[derive(Debug)]
struct Type {
    version: u16,
    binary: u16,
    keeping: List<u16>,
    plans: Map<u16, Plan>,
}
/// Transcript records use two registry types with no map key ranges.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Frame {
    Body,
    Turn,
}
impl Frame {
    pub(crate) fn owner(owner: Owner) -> Option<Self> {
        match owner {
            Owner::Jig => Some(Self::Body),
            Owner::Smith => Some(Self::Turn),
            Owner::Skein | Owner::Application => None,
        }
    }
}
#[derive(Debug)]
struct FrameTypes {
    body: Option<u32>,
    turn: Option<u32>,
}
impl FrameTypes {
    fn get(&self, frame: Frame) -> Option<u32> {
        match frame {
            Frame::Body => self.body,
            Frame::Turn => self.turn,
        }
    }
    fn set(&mut self, frame: Frame, index: u32) {
        match frame {
            Frame::Body => self.body = Some(index),
            Frame::Turn => self.turn = Some(index),
        }
    }
}
enum ReadAs {
    Stored,
    Reheaded,
    Converted,
}
impl Type {
    fn read_as(&self, id: u16, value: &[u8], work: &mut ReadWork) -> ReadAs {
        if id == self.binary || self.keeping.as_slice().contains(&id) {
            return ReadAs::Reheaded;
        }
        if let Some(plan) = self.plans.get(&id)
            && work.convert(plan, value).is_ok()
        {
            return ReadAs::Converted;
        }
        ReadAs::Stored
    }
}

#[derive(Debug)]
struct Range {
    to: Box<[u8]>,
    ty: u32,
}
/// Built before the copy is made; active only after its rows are durable.
#[derive(Debug)]
pub(crate) struct Layouts {
    active: bool,
    types: List<Type>,
    introduced: List<(&'static str, u16)>,
    frame_types: FrameTypes,
    ranges: Map<Box<[u8]>, Range>,
    rows: Map<RowKey, Row>,
    orders: Map<OrderKey, Order>,
    retired: Map<Box<[u8]>, Box<[u8]>>,
    written: Map<RowKey, &'static str>,
    texts: Map<Digest, Text>,
    prepared_schemas: &'static [Schema],
    admitted_schemas: Option<List<Schema>>,
    pending: Queue<skein_kv::Op>,
    owners: crate::OwnerSets,
    walk: Option<Box<settling::Walk>>,
    #[cfg(test)]
    pub(crate) walk_reads: u32,
}
impl Layouts {
    pub(crate) fn new() -> Self {
        Self {
            active: false,
            types: List::uncapped(),
            introduced: List::uncapped(),
            frame_types: FrameTypes { body: None, turn: None },
            ranges: Map::uncapped(),
            rows: Map::uncapped(),
            orders: Map::uncapped(),
            retired: Map::uncapped(),
            written: Map::uncapped(),
            texts: Map::uncapped(),
            prepared_schemas: &[],
            admitted_schemas: None,
            pending: Queue::uncapped(),
            owners: crate::OwnerSets::NONE,
            walk: None,
            #[cfg(test)]
            walk_reads: 0,
        }
    }
    pub(crate) fn take_introduced(&mut self) -> List<(&'static str, u16)> {
        core::mem::replace(&mut self.introduced, List::uncapped())
    }

    fn resolve_index(&self, key: &[u8]) -> Option<u32> {
        use core::ops::Bound::{Included, Unbounded};
        let (_, range) = self.ranges.range::<[u8], _>((Unbounded, Included(key))).next_back()?;
        if key >= range.to.as_ref() {
            return None;
        }
        Some(range.ty)
    }
    fn resolve(&self, key: &[u8]) -> Option<&Type> {
        self.types.get(self.resolve_index(key)?)
    }
    pub(crate) fn write(&self, key: &[u8], value: &mut [u8]) -> Result<(), ()> {
        use core::ops::Bound::{Included, Unbounded};
        if let Some((_, to)) = self.retired.range::<[u8], _>((Unbounded, Included(key))).next_back()
            && key < to.as_ref()
        {
            return Err(());
        }
        if !self.active {
            return Ok(());
        }
        let Some(ty) = self.resolve(key) else {
            return Ok(());
        };
        if head(value) != Some(ty.version) {
            return Err(());
        }
        rehead(value, ty.binary);
        Ok(())
    }
    pub(crate) fn read<'a>(&self, key: &[u8], value: &'a mut [u8], work: &'a mut ReadWork) -> &'a [u8] {
        if !self.active {
            return value;
        }
        self.read_layout(key, value, work)
    }
    fn read_layout<'a>(&self, key: &[u8], value: &'a mut [u8], work: &'a mut ReadWork) -> &'a [u8] {
        let Some(ty) = self.resolve(key) else {
            return value;
        };
        let Some(id) = head(value) else {
            return value;
        };
        match ty.read_as(id, value, work) {
            ReadAs::Stored => value,
            ReadAs::Reheaded => {
                rehead(value, ty.version);
                value
            }
            ReadAs::Converted => work.buffer.as_slice(),
        }
    }
    fn frame(&self, frame: Frame) -> Option<&Type> {
        self.types.get(self.frame_types.get(frame)?)
    }
    pub(crate) fn has_frame(&self, frame: Frame) -> bool {
        self.frame(frame).is_some()
    }
    pub(crate) fn write_frame(&self, frame: Frame, value: &mut [u8]) -> Option<()> {
        if let Some(ty) = self.frame(frame) {
            if head(value)? != ty.version {
                return None;
            }
            rehead(value, ty.binary);
        }
        Some(())
    }
    /// A prefix can be decoded without a full conversion only at a keeping id.
    pub(crate) fn frame_version(&self, frame: Frame, id: u16) -> Result<Option<u16>, ()> {
        if !self.active {
            return Ok(Some(id));
        }
        let Some(ty) = self.frame(frame) else { return Ok(Some(id)) };
        if id == ty.binary || ty.keeping.as_slice().contains(&id) {
            Ok(Some(ty.version))
        } else if ty.plans.contains_key(&id) {
            Ok(None)
        } else {
            Err(())
        }
    }
    pub(crate) fn read_frame<'a>(&self, frame: Frame, value: &'a mut [u8], work: &'a mut ReadWork) -> Option<&'a [u8]> {
        if !self.has_frame(frame) {
            return Some(value);
        }
        let id = head(value)?;
        if let Some(version) = self.frame_version(frame, id).ok()? {
            rehead(value, version);
            return Some(value);
        }
        self.convert_frame(frame, id, value, work)
    }
    pub(crate) fn convert_frame<'a>(
        &self,
        frame: Frame,
        id: u16,
        value: &[u8],
        work: &'a mut ReadWork,
    ) -> Option<&'a [u8]> {
        let plan = self.frame(frame)?.plans.get(&id)?;
        work.convert(plan, value).ok()?;
        Some(work.buffer.as_slice())
    }
    pub(crate) fn activate(&mut self) {
        self.active = true;
    }
    /// Once registry writes and written jobs are durable, keep the read and
    /// write edges' tables and just the references the serving walk needs.
    /// Activation itself precedes written jobs.
    pub(crate) fn settle(&mut self) {
        if !self.active || self.rows.is_empty() {
            return;
        }
        assert!(self.pending.is_empty(), "registry writes finished before settling");
        self.walk = settling::Walk::new(self);
        self.owners = crate::OwnerSets::NONE;
        self.rows = Map::uncapped();
        self.texts = Map::uncapped();
        self.pending = Queue::uncapped();
        self.written = Map::uncapped();
        for ty in self.types.iter_mut() {
            let keeping = core::mem::replace(&mut ty.keeping, List::uncapped());
            ty.keeping = List::from_boxed(keeping.into_boxed());
        }
        let types = core::mem::replace(&mut self.types, List::uncapped());
        self.types = List::from_boxed(types.into_boxed());
    }
    pub(crate) fn committed(&mut self, number: u64) {
        settling::committed(self, number);
    }
    #[cfg(test)]
    pub(crate) fn assert_settled(&self) {
        assert!(self.active);
        assert!(self.rows.is_empty());
        assert!(self.texts.is_empty());
        assert!(self.pending.is_empty());
        assert_eq!(self.types.len(), self.types.capacity());
        for ty in &self.types {
            assert_eq!(ty.keeping.len(), ty.keeping.capacity());
            assert!(!ty.plans.contains_key(&ty.binary));
            for (_, plan) in &ty.plans {
                assert!(!plan.keeps());
            }
        }
    }
    fn next_op(&mut self) -> Option<skein_kv::Op> {
        let op = self.pending.pop()?;
        if let skein_kv::Op::Erase { key } = &op
            && key.starts_with(&[crate::keys::COMPONENT, 6])
            && let Some(crate::keys::Own::Layout { family, number, id }) = crate::keys::decode_own(key)
            && let Some(row) = self.rows.get_mut(&(family, number, id))
        {
            // The serving tables are built after the pending operations commit.
            row.stored = false;
        }
        Some(op)
    }
    pub(crate) fn first(&mut self, ops: &mut List<skein_kv::Op>) -> Option<()> {
        for _ in 0..self.pending.len() {
            ops.push(self.next_op()?).ok()?;
        }
        Some(())
    }
    pub(crate) fn batch(&mut self, limits: &crate::Limits) -> Option<Box<[skein_kv::Op]>> {
        let mut ops = List::with_capacity(limits.store.ops.min(limits.commits.writes));
        let mut bytes = u64::from(skein_kv::FRAME_BYTES);
        for _ in 0..self.pending.len() {
            let op = self.pending.iter().next()?;
            let size = rows::size(op)?;
            let total = bytes.checked_add(size)?;
            if total > u64::from(limits.store.commit) || ops.len() == ops.capacity() {
                break;
            }
            bytes = total;
            ops.push(self.next_op()?).ok()?;
        }
        if ops.is_empty() && !self.pending.is_empty() {
            return None;
        }
        Some(ops.into_boxed())
    }
    pub(crate) fn written(&self, job: &Job) -> Option<Queue<(u16, JobLayout)>> {
        let mut layouts = Queue::uncapped();
        let Some(converts) = job.converts.as_ref() else { return Some(layouts) };
        for (key, row) in &self.rows {
            let (family, number, id) = key;
            if family.as_ref() != converts.family || *number != converts.value_type {
                continue;
            }
            let takes = match converts.from {
                crate::Layouts::One(from) => row.value.layout == from,
                crate::Layouts::Unplanned => self.written.get(key) == Some(&job.name),
            };
            if !takes {
                continue;
            }
            let mut held = List::uncapped();
            for digest in &row.value.held {
                held.push(self.texts.get(digest)?.bytes.clone()).ok()?;
            }
            layouts.push((
                *id,
                JobLayout {
                    root: texts::string(&row.value.root)?,
                    variant: match &row.value.variant {
                        Some(v) => Some(texts::string(v)?),
                        None => None,
                    },
                    text: self.texts.get(&row.value.text)?.bytes.clone(),
                    held: held.into_boxed(),
                },
            ));
            match converts.from {
                crate::Layouts::One(_) => break,
                crate::Layouts::Unplanned => {}
            }
        }
        Some(layouts)
    }
}
/// Keep an owned row in place when only its head moves. A converted row owns
/// the conversion buffer's result before the next read reuses it.
pub(crate) fn read(store: &mut Store, key: &[u8], value: &mut Box<[u8]>) {
    owned(store, key, value, false);
}

/// The deployment header is read after plans are checked but before the
/// registry's writes, to establish the open's numbers (protocol/store/layouts.md, 5).
pub(crate) fn header(store: &mut Store, key: &[u8], value: &mut Box<[u8]>) {
    owned(store, key, value, true);
}

fn owned(store: &mut Store, key: &[u8], value: &mut Box<[u8]>, before_active: bool) {
    let original = value.as_ptr();
    let work = &mut store.layout_work;
    let mapped = if before_active {
        store.registry.read_layout(key, value, work)
    } else {
        store.registry.read(key, value, work)
    };
    if mapped.as_ptr() != original {
        *value = Box::from(mapped);
    }
}

pub(crate) fn head(value: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes(value.get(..2)?.try_into().ok()?))
}
fn variant(name: Option<&str>) -> Option<&[u8]> {
    match name {
        Some(name) => Some(name.as_bytes()),
        None => None,
    }
}
pub(crate) fn rehead(value: &mut [u8], id: u16) {
    if let Some(head) = value.get_mut(..2) {
        for (byte, next) in head.iter_mut().zip(id.to_be_bytes()) {
            *byte = next;
        }
    }
}
fn whole(owner: Owner, key: &[u8], bound: u32) -> Option<Box<[u8]>> {
    if owner == Owner::Application { crate::keys::application(key, bound) } else { Some(Box::from(key)) }
}
/// Plan before any writes, including before linking the new generation.
pub(crate) fn open(store: &mut Store, empty: bool) -> Result<bool, OpenRefusal> {
    #[cfg(test)]
    counting::reset();
    let mut layouts = Layouts::new();
    layouts.prepared_schemas = store.prepared_schemas;
    let registered = rows::open(store, &mut layouts, empty)?;
    let seeded = !layouts.rows.is_empty();
    if !seeded && !registered {
        *store.registry = Layouts::new();
        *store.layout_work = ReadWork::new();
        *store.frame_work = ReadWork::new();
        return Ok(false);
    }
    if let Err(why) = crate::upgrade::admission::checked(store) {
        return Err(OpenRefusal::Set { owner: why });
    }
    layouts.admitted_schemas = store.directory.admitted_schemas.take();
    check::retired(store, &mut layouts)?;
    let mut binary = texts::Binary::new(&store.owner_sets, &mut layouts)?;
    let changed = rows::register(store, &mut layouts, &mut binary)?;
    if empty {
        layouts.take_introduced();
    }
    let mut work = Workspace::new();
    let mut frames = Workspace::new();
    let later = check::plans(store, &mut layouts, &mut binary, &mut work, &mut frames)?;
    let mut work = ReadWork::fit(work, store.limits.store.value);
    later.run(store, &layouts, &mut work)?;
    let writing = changed || (!seeded && !layouts.types.is_empty());
    rows::operations(store.limits.store.key, &mut layouts)?;
    if !writing && store.directory.patch.is_none() {
        // Retirement alone does not write: these erases wait for a writing open.
        layouts.pending = Queue::uncapped();
    }
    layouts.active = seeded && !changed;
    work.buffer = Buffer::with_capacity(0);
    *store.layout_work = work;
    *store.frame_work = ReadWork::fit(frames, store.limits.store.page.bytes);
    *store.registry = layouts;
    Ok(writing)
}
