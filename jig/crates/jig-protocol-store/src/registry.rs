//! Layouts kept for the serving open (protocol/store/layouts.md, 3–7).
use crate::{Job, JobLayout, OpenRefusal, Owner, Store};
use alloc::boxed::Box;
use skein_codec::{
    layout::{Plan, Workspace},
    schema::Schema,
};
use skein_lib::{Buffer, List, Map, Queue};
mod check;
pub(crate) mod rehearsal;
mod rows;
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
    retired: Map<Box<[u8]>, Box<[u8]>>,
    written: Map<(u32, u16), &'static str>,
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
            retired: Map::uncapped(),
            written: Map::uncapped(),
            texts: Map::uncapped(),
            pending: Queue::uncapped(),
        }
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
        self.written = Map::uncapped();
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
    pub(crate) fn has_writes(&self) -> bool {
        !self.pending.is_empty()
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
    /// Settling reads as stored, refusing rather than handing malformed bytes on (jobs.md, 5.1).
    pub(crate) fn put_again(
        &self,
        key: &[u8],
        value: &[u8],
        work: &mut ReadWork,
    ) -> Result<Option<Box<[u8]>>, Box<str>> {
        let index = self.resolve_index(key).expect("settling ranges resolve to value types");
        let ty = self.types.get(index).expect("admitted value type");
        let Some(id) = head(value) else {
            return Err(reason(&[b"is at layout `0`, which the registry does not hold"]));
        };
        if id == ty.binary {
            return Ok(None);
        }
        if let Some(name) = self.written.get(&(index, id)) {
            return Err(reason(&[
                b"is at layout `",
                skein_lib::Decimal::of(u64::from(id)).as_bytes(),
                b"`, which the written job `",
                name.as_bytes(),
                b"` converts",
            ]));
        }
        if ty.keeping.as_slice().contains(&id) {
            let mut value = Box::from(value);
            rehead(&mut value, ty.version);
            return Ok(Some(value));
        }
        let Some(plan) = ty.plans.get(&id) else {
            return Err(reason(&[
                b"is at layout `",
                skein_lib::Decimal::of(u64::from(id)).as_bytes(),
                b"`, which the registry does not hold",
            ]));
        };
        match plan.convert(value, &mut work.buffer, &mut work.workspace) {
            Ok(()) => Ok(Some(Box::from(work.buffer.as_slice()))),
            Err(skein_codec::layout::Misconverted::Dropped(index)) => {
                let (enumeration, variant) = plan.dropped(index).expect("plan's dropped name");
                Err(reason(&[
                    b"holds the variant `",
                    variant.as_bytes(),
                    b"` of `",
                    enumeration.as_bytes(),
                    b"`, which this build no longer has",
                ]))
            }
            Err(why) => {
                use skein_codec::layout::Misconverted;
                let why = match why {
                    Misconverted::Short => b"Short".as_slice(),
                    Misconverted::Tag => b"Tag",
                    Misconverted::Bool => b"Bool",
                    Misconverted::Bound => b"Bound",
                    Misconverted::Trailing => b"Trailing",
                    Misconverted::Room => b"Room",
                    Misconverted::Dropped(_) => unreachable!("dropped variants name their enumeration"),
                };
                Err(reason(&[
                    b"does not convert from layout `",
                    skein_lib::Decimal::of(u64::from(id)).as_bytes(),
                    b"`: `",
                    why,
                    b"`",
                ]))
            }
        }
    }
    pub(crate) fn retire_set(&mut self, set: &crate::PatchSet, logs_pending: bool, bound: u32) -> Option<u64> {
        let mut retired = 0_u64;
        for family in set.families {
            for ty in family.types {
                if check::history_type(family.name, ty.number) && !logs_pending {
                    continue;
                }
                let from = whole(set.owner, ty.keys.first()?.from, bound)?;
                let index = self.ranges.get(&from)?.ty;
                let binary = self.types.get(index)?.binary;
                let mut keys = List::uncapped();
                for (key, row) in &self.rows {
                    if key.0.as_ref() == family.name && key.1 == ty.number && key.2 != binary && !row.retire {
                        keys.push(key.clone()).expect("uncapped retired rows");
                    }
                }
                for key in &keys {
                    self.rows.get_mut(key)?.retire = true;
                    retired = retired.checked_add(1)?;
                }
                let planned = self.types.get_mut(index)?;
                planned.keeping = List::uncapped();
                planned.plans = Map::uncapped();
            }
        }
        rows::operations(bound, self).ok()?;
        Some(retired)
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
    let mut layouts = Layouts::new();
    rows::load(store, &mut layouts)?;
    let seeded = !layouts.rows.is_empty();
    let mut registered = false;
    for set in store.patch_sets.sets {
        for family in set.families {
            registered |= !family.types.is_empty() || !family.retired.is_empty();
        }
    }
    if !seeded && !registered {
        *store.registry = Layouts::new();
        *store.layout_work = ReadWork::new();
        return Ok(false);
    }
    if let Err(why) = crate::patching::plan::bounds(&store.patch_sets) {
        return Err(OpenRefusal::Set { owner: why.owner, patch: why.patch });
    }
    if seeded {
        check::patches(store)?;
    }
    check::retired(store, &mut layouts)?;
    let mut binary = texts::Binary::new(&store.patch_sets, &mut layouts)?;
    if !seeded && !empty {
        rows::seed(store, &mut layouts, &mut binary)?;
    }
    let changed = rows::register(store, &mut layouts, &mut binary)?;
    let mut work = Workspace::new();
    check::plans(store, &mut layouts, &binary, &mut work)?;
    let writing = changed || (!seeded && !layouts.types.is_empty());
    rows::operations(store.limits.store.key, &mut layouts)?;
    if !writing && store.directory.patch.is_none() {
        // Retirement alone does not write: these erases wait for a writing open.
        layouts.pending = Queue::uncapped();
    }
    layouts.active = seeded && !changed;
    *store.layout_work = ReadWork::fit(&layouts, work, store.limits.store.value);
    *store.registry = layouts;
    Ok(writing)
}

/// Refusal parts are checked names, decimal numbers and literal UTF-8 (jobs.md, 5.1).
#[expect(clippy::disallowed_methods, reason = "refusal parts are valid UTF-8")]
pub(crate) fn reason(parts: &[&[u8]]) -> Box<str> {
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.checked_add(part.len()).expect("bounded refusal");
    }
    let mut writer = skein_lib::Writer::new(bytes);
    for part in parts {
        writer.put(part).expect("measured refusal");
    }
    Box::from(core::str::from_utf8(&writer.finish()).expect("UTF-8 refusal parts"))
}
