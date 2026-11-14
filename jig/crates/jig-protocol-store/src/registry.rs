//! Layouts kept for the serving open (protocol/store/layouts.md, 3–7).
use alloc::boxed::Box;
use skein_codec::{layout::{Plan, Workspace}, schema::Schema};
use skein_lib::{Buffer, List, Map, Queue};
use crate::{Job, JobLayout, OpenRefusal, Owner, Store};
mod check;
mod rows;
mod texts;

type Digest = [u8; 32];
type RowKey = (Box<str>, u16, u16);
#[derive(Debug)]
struct Text {
    bytes: Box<[u8]>,
    schema: Option<Schema>,
    stored: bool,
}
#[derive(Debug)]
struct Row {
    value: crate::registry_v1::Layout,
    stored: bool,
    retire: bool,
}
#[derive(Debug)]
struct Type {
    family: &'static str,
    number: u16,
    version: u16,
    binary: u16,
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
        Self { active: false, types: List::uncapped(), ranges: Map::uncapped(), rows: Map::uncapped(), texts: Map::uncapped(), pending: Queue::uncapped() }
    }
    fn resolve(&self, key: &[u8]) -> Option<&Type> {
        use core::ops::Bound::{Included, Unbounded};
        let (_, range) = self.ranges.range::<[u8], _>((Unbounded, Included(key))).next_back()?;
        if key >= range.to.as_ref() { return None; }
        self.types.get(range.ty)
    }
    #[cfg_attr(not(test), expect(dead_code, reason = "the next change wires the store's write edge"))]
    pub(crate) fn write(&self, key: &[u8], value: &mut [u8]) -> Result<(), ()> {
        if !self.active { return Ok(()); }
        let Some(ty) = self.resolve(key) else { return Ok(()); };
        if head(value) != Some(ty.version) { return Err(()); }
        rehead(value, ty.binary);
        Ok(())
    }
    #[cfg_attr(not(test), expect(dead_code, reason = "the next change wires the store's read edges"))]
    pub(crate) fn read<'a>(&self, key: &[u8], value: &'a mut [u8], out: &'a mut Buffer, work: &mut Workspace) -> &'a [u8] {
        if !self.active { return value; }
        let Some(ty) = self.resolve(key) else { return value; };
        let Some(id) = head(value) else { return value; };
        if id == ty.binary {
            rehead(value, ty.version);
            return value;
        }
        let Some(plan) = ty.plans.get(&id) else { return value; };
        if plan.keeps() {
            rehead(value, ty.version);
            return value;
        }
        if plan.convert(value, out, work).is_ok() { out.as_slice() } else { value }
    }
    pub(crate) fn activate(&mut self) { self.active = true; }
    pub(crate) fn first(&mut self, ops: &mut List<skein_kv::Op>) -> Option<()> {
        for _ in 0..self.pending.len() { ops.push(self.pending.pop()?).ok()?; }
        Some(())
    }
    pub(crate) fn batch(&mut self, limits: &crate::Limits) -> Option<Box<[skein_kv::Op]>> {
        let mut ops = List::with_capacity(limits.store.ops.min(limits.commits.writes));
        let mut bytes = u64::from(skein_kv::FRAME_BYTES);
        for _ in 0..self.pending.len() {
            let op = self.pending.iter().next()?;
            let size = rows::size(op)?;
            let total = bytes.checked_add(size)?;
            if total > u64::from(limits.store.commit) || ops.len() == ops.capacity() { break; }
            bytes = total;
            ops.push(self.pending.pop()?).ok()?;
        }
        if ops.is_empty() && !self.pending.is_empty() { return None; }
        Some(ops.into_boxed())
    }
    pub(crate) fn written(&self, job: &Job) -> Option<(u16, JobLayout)> {
        let converts = job.converts.as_ref()?;
        for ((family, number, id), row) in &self.rows {
            if family.as_ref() != converts.family || *number != converts.value_type || row.value.layout() != &converts.from { continue; }
            let mut held = List::uncapped();
            for digest in row.value.held() { held.push(self.texts.get(digest)?.bytes.clone()).ok()?; }
            return Some((*id, JobLayout {
                root: texts::string(row.value.root())?,
                variant: match row.value.variant() { Some(v) => Some(texts::string(v)?), None => None },
                text: self.texts.get(row.value.text())?.bytes.clone(), held: held.into_boxed(),
            }));
        }
        None
    }
}
pub(crate) fn head(value: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes(value.get(..2)?.try_into().ok()?))
}
fn variant(name: Option<&str>) -> Option<&[u8]> { match name { Some(name) => Some(name.as_bytes()), None => None } }
fn rehead(value: &mut [u8], id: u16) {
    if let Some(head) = value.get_mut(..2) { for (byte, next) in head.iter_mut().zip(id.to_be_bytes()) { *byte = next; } }
}
fn whole(owner: Owner, key: &[u8], bound: u32) -> Option<Box<[u8]>> {
    if owner == Owner::Application { crate::keys::application(key, bound) } else { Some(Box::from(key)) }
}
/// Plan before any writes, including before linking the new generation.
pub(crate) fn open(store: &mut Store, empty: bool) -> Result<bool, OpenRefusal> {
    let mut layouts = Layouts::new();
    rows::load(store, &mut layouts)?;
    let seeded = !layouts.rows.is_empty();
    if seeded { check::patches(store)?; }
    let mut binary = texts::Binary::new(&store.patch_sets, &mut layouts)?;
    if !seeded && !empty { rows::seed(store, &mut layouts, &mut binary)?; }
    let changed = rows::register(store, &mut layouts, &mut binary)?;
    let mut work = Workspace::new();
    check::plans(store, &mut layouts, &binary, &mut work)?;
    let writing = changed || (!seeded && !layouts.types.is_empty());
    rows::operations(store, &mut layouts)?;
    layouts.active = seeded && !changed;
    store.registry = layouts;
    store.layout_work = work;
    Ok(writing)
}
