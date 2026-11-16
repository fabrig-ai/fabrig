//! Layouts kept for the serving open (protocol/store/layouts.md, 3–7).
use crate::{Job, JobLayout, OpenRefusal, Owner, Store};
use alloc::boxed::Box;
use skein_codec::{
    layout::{Plan, Workspace},
    schema::Schema,
};
use skein_lib::{Buffer, List, Map, Queue};
mod check;
#[cfg(test)]
mod counting;
pub(crate) mod rehearsal;
mod rows;
#[cfg(test)]
mod tests;
mod texts;

/// Fitted at open beside the plans (protocol/store/layouts.md, 7). A store
/// whose plans all keep needs no conversion buffer; its reads only rehead.
#[derive(Debug)]
pub(crate) struct ReadWork {
    pub(crate) workspace: Workspace,
    buffer: Buffer,
}
impl ReadWork {
    pub(crate) fn new() -> Self {
        Self { workspace: Workspace::new(), buffer: Buffer::with_capacity(0) }
    }
    fn fit(layouts: &Layouts, workspace: Workspace, bound: u32) -> Self {
        let mut converts = false;
        for ty in &layouts.types {
            converts |= !ty.plans.is_empty();
        }
        Self { workspace, buffer: Buffer::with_capacity(if converts { bound } else { 0 }) }
    }
}

type Digest = [u8; 32];
type RowKey = (Box<str>, u16, u16);
#[derive(Debug)]
struct Text {
    bytes: Box<[u8]>,
    schema: Option<Schema>,
    stored: bool,
    current: bool,
}
#[derive(Debug)]
struct Row {
    value: crate::registry_v1::Layout,
    stored: bool,
    retire: bool,
}
#[derive(Debug)]
struct Type {
    version: u16,
    binary: u16,
    keeping: List<u16>,
    plans: Map<u16, Plan>,
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
    ranges: Map<Box<[u8]>, Range>,
    rows: Map<RowKey, Row>,
    texts: Map<Digest, Text>,
    pending: Queue<skein_kv::Op>,
}
impl Layouts {
    pub(crate) fn new() -> Self {
        Self {
            active: false,
            types: List::uncapped(),
            ranges: Map::uncapped(),
            rows: Map::uncapped(),
            texts: Map::uncapped(),
            pending: Queue::uncapped(),
        }
    }
    fn resolve(&self, key: &[u8]) -> Option<&Type> {
        use core::ops::Bound::{Included, Unbounded};
        let (_, range) = self.ranges.range::<[u8], _>((Unbounded, Included(key))).next_back()?;
        if key >= range.to.as_ref() {
            return None;
        }
        self.types.get(range.ty)
    }
    pub(crate) fn write(&self, key: &[u8], value: &mut [u8]) -> Result<(), ()> {
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
    pub(crate) fn read<'a>(
        &self,
        key: &[u8],
        value: &'a mut [u8],
        out: &'a mut Buffer,
        work: &mut Workspace,
    ) -> &'a [u8] {
        if !self.active {
            return value;
        }
        let Some(ty) = self.resolve(key) else {
            return value;
        };
        let Some(id) = head(value) else {
            return value;
        };
        if id == ty.binary {
            rehead(value, ty.version);
            return value;
        }
        if ty.keeping.as_slice().contains(&id) {
            rehead(value, ty.version);
            return value;
        }
        let Some(plan) = ty.plans.get(&id) else {
            return value;
        };
        if plan.convert(value, out, work).is_ok() { out.as_slice() } else { value }
    }
    pub(crate) fn activate(&mut self) {
        self.active = true;
    }
    /// Once registry writes and written jobs are durable, only the read and
    /// write edges' tables survive. Activation itself precedes written jobs.
    pub(crate) fn settle(&mut self) {
        if !self.active {
            return;
        }
        assert!(self.pending.is_empty(), "registry writes finished before settling");
        self.rows = Map::uncapped();
        self.texts = Map::uncapped();
        self.pending = Queue::uncapped();
        for ty in self.types.iter_mut() {
            let keeping = core::mem::replace(&mut ty.keeping, List::uncapped());
            ty.keeping = List::from_boxed(keeping.into_boxed());
        }
        let types = core::mem::replace(&mut self.types, List::uncapped());
        self.types = List::from_boxed(types.into_boxed());
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
    pub(crate) fn first(&mut self, ops: &mut List<skein_kv::Op>) -> Option<()> {
        for _ in 0..self.pending.len() {
            ops.push(self.pending.pop()?).ok()?;
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
            ops.push(self.pending.pop()?).ok()?;
        }
        if ops.is_empty() && !self.pending.is_empty() {
            return None;
        }
        Some(ops.into_boxed())
    }
    pub(crate) fn written(&self, job: &Job) -> Option<(u16, JobLayout)> {
        let converts = job.converts.as_ref()?;
        for ((family, number, id), row) in &self.rows {
            if family.as_ref() != converts.family
                || *number != converts.value_type
                || row.value.layout() != &converts.from
            {
                continue;
            }
            let mut held = List::uncapped();
            for digest in row.value.held() {
                held.push(self.texts.get(digest)?.bytes.clone()).ok()?;
            }
            return Some((
                *id,
                JobLayout {
                    root: texts::string(row.value.root())?,
                    variant: match row.value.variant() {
                        Some(v) => Some(texts::string(v)?),
                        None => None,
                    },
                    text: self.texts.get(row.value.text())?.bytes.clone(),
                    held: held.into_boxed(),
                },
            ));
        }
        None
    }
}
/// Keep an owned row in place when only its head moves. A converted row owns
/// the conversion buffer's result before the next read reuses it.
pub(crate) fn read(store: &mut Store, key: &[u8], value: &mut Box<[u8]>) {
    let original = value.as_ptr();
    let work = &mut store.layout_work;
    let mapped = store.registry.read(key, value, &mut work.buffer, &mut work.workspace);
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
fn rehead(value: &mut [u8], id: u16) {
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
    rows::load(store, &mut layouts)?;
    let seeded = !layouts.rows.is_empty();
    let mut registered = false;
    for set in store.patch_sets.sets {
        for family in set.families {
            registered |= !family.types.is_empty();
        }
    }
    if !seeded && !registered {
        *store.registry = Layouts::new();
        *store.layout_work = ReadWork::new();
        return Ok(false);
    }
    if let Err(why) = crate::patching::plan::checked(store) {
        return Err(OpenRefusal::Set { owner: why.owner, patch: why.patch });
    }
    if seeded {
        check::patches(store)?;
    }
    let mut binary = texts::Binary::new(&store.patch_sets, &mut layouts)?;
    if !seeded && !empty {
        rows::seed(store, &mut layouts, &mut binary)?;
    }
    let changed = rows::register(store, &mut layouts, &mut binary)?;
    let mut work = Workspace::new();
    check::plans(store, &mut layouts, &mut binary, &mut work)?;
    let writing = changed || (!seeded && !layouts.types.is_empty());
    rows::operations(store, &mut layouts)?;
    if !writing && store.directory.patch.is_none() {
        // Retirement alone does not write: these erases wait for a writing open.
        layouts.pending = Queue::uncapped();
    }
    layouts.active = seeded && !changed;
    *store.layout_work = ReadWork::fit(&layouts, work, store.limits.store.value);
    *store.registry = layouts;
    Ok(writing)
}
