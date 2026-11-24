//! The serving walk and the operations it rides (jig/docs/design/protocol/store/settling.md, 2–5).
use super::{Digest, Layouts, ReadAs, RowKey, check, head, rehead, rows};
use crate::{KeptMap, Store, keys, pages};
use alloc::boxed::Box;
use core::ops::Bound::{Included, Unbounded};
use skein_kv::{Op, Range, Row};
use skein_lib::{List, Map, Queue, Set, len_u64};

const PUTS: u32 = 32;

#[derive(Debug)]
struct Candidate {
    key: RowKey,
    texts: Set<Digest>,
    refused: bool,
}

#[derive(Debug)]
struct Group {
    ty: u32,
    ids: Map<u16, Candidate>,
}

#[derive(Debug)]
struct Retirement {
    ty: u32,
    row: Candidate,
}

#[derive(Debug)]
struct Submitted {
    number: u64,
    ty: u32,
    id: u16,
}

/// Only the active group has a cursor and a page. Texts keep counts, not bytes.
#[derive(Debug)]
pub(super) struct Walk {
    groups: Queue<Group>,
    current: Option<Group>,
    next: Option<Range>,
    page: Queue<Row>,
    retiring: Queue<Retirement>,
    submitted: Queue<Submitted>,
    refs: Map<Digest, u32>,
}

impl Walk {
    pub(super) fn new(layouts: &Layouts) -> Option<Box<Self>> {
        let mut groups = Map::uncapped();
        let mut refs = Map::uncapped();
        let mut index = 0_u32;
        for owner in layouts.owners.sets {
            for family in owner.families {
                for declared in family.types {
                    let ty = index;
                    index = index.checked_add(1).expect("registered type count fits");
                    if family.map != KeptMap::Store
                        || declared.keys.is_empty()
                        || check::history_type(&layouts.owners, family.name, declared.number)
                    {
                        continue;
                    }
                    let edge = layouts.types.get(ty).expect("registered type");
                    if edge.keeping.is_empty() && edge.plans.is_empty() {
                        continue;
                    }
                    let mut group = Group { ty, ids: Map::uncapped() };
                    let lower = (Box::from(family.name), declared.number, u16::MIN);
                    let upper = (Box::from(family.name), declared.number, u16::MAX);
                    for (key, row) in layouts.rows.range(lower..=upper) {
                        let id = key.2;
                        if !(edge.keeping.as_slice().contains(&id) || edge.plans.contains_key(&id))
                            || written(owner.jobs, key, row.value.layout)
                        {
                            continue;
                        }
                        let texts = digests(&row.value);
                        for text in &texts {
                            refs.insert(*text, 0_u32).expect("uncapped text references");
                        }
                        group
                            .ids
                            .insert(id, Candidate { key: key.clone(), texts, refused: false })
                            .expect("uncapped layouts");
                    }
                    if !group.ids.is_empty() {
                        groups.insert((family.name, declared.number), group).expect("uncapped value types");
                    }
                }
            }
        }
        if groups.is_empty() {
            return None;
        }
        for (_, row) in &layouts.rows {
            if row.retire && !row.stored {
                continue;
            }
            for digest in &digests(&row.value) {
                if let Some(count) = refs.get_mut(digest) {
                    *count = count.checked_add(1).expect("registry row count fits");
                }
            }
        }
        let mut ordered = Queue::uncapped();
        for _ in 0..groups.len() {
            let (&key, _) = groups.first().expect("remaining value type");
            ordered.push(groups.remove(&key).expect("first value type"));
        }
        Some(Box::new(Self {
            groups: ordered,
            current: None,
            next: None,
            page: Queue::uncapped(),
            retiring: Queue::uncapped(),
            submitted: Queue::uncapped(),
            refs,
        }))
    }

    fn fill(&mut self, store: &mut Store) {
        if !self.page.is_empty() {
            return;
        }
        for _ in 0..=self.groups.len() {
            if let Some(range) = self.next.take() {
                let ty = self.current.as_ref().expect("cursor belongs to the active type").ty;
                let end = range.end.clone().expect("registered ranges have ends");
                #[cfg(test)]
                {
                    store.registry.walk_reads = store.registry.walk_reads.checked_add(1).expect("walk reads fit");
                }
                let Ok(page) = pages::reader::load_stored(store, range) else {
                    // This type's emptiness was not proved; nothing of it can retire.
                    self.current = None;
                    return;
                };
                for row in page.rows {
                    self.page.push(row);
                }
                self.next = match page.next {
                    Some(start) => Some(Range { start, end: Some(end) }),
                    None => range_of(&store.registry, ty, Some(&end)),
                };
                return;
            }
            let Some(group) = self.groups.pop() else { return };
            if may_hold(store, &group) {
                self.next = range_of(&store.registry, group.ty, None);
                self.current = Some(group);
            } else {
                self.ready_to_retire(group);
            }
        }
    }

    fn ready_to_retire(&mut self, mut group: Group) {
        for _ in 0..group.ids.len() {
            let (&id, _) = group.ids.first().expect("remaining layout");
            let row = group.ids.remove(&id).expect("first layout");
            if !row.refused {
                self.retiring.push(Retirement { ty: group.ty, row });
            }
        }
    }

    fn drain(&mut self, store: &mut Store, touched: &Set<&[u8]>, room: &mut Room, extras: &mut List<Op>) {
        let Some(group) = self.current.as_mut() else { return };
        let ty = store.registry.types.get(group.ty).expect("a serving type");
        let mut puts = 0_u32;
        for _ in 0..self.page.len() {
            if puts == PUTS || room.ops == 0 {
                break;
            }
            let row = self.page.iter().next().expect("unconsumed row");
            let Some(id) = head(&row.value) else {
                self.page.pop();
                continue;
            };
            if !group.ids.contains_key(&id)
                || touched.contains(row.key.as_ref())
                || !store.stores.store.unchanged(&row.key, &row.value)
            {
                // The only serving writes are at the binary's id, or erases.
                self.page.pop();
                continue;
            }
            let work = &mut store.layout_work;
            let read = if keys::logged(&row.key).is_some() { ReadAs::Stored } else { ty.read_as(id, &row.value, work) };
            let value = match read {
                ReadAs::Stored => {
                    // The read edge would not convert it. Keep the layout and all its texts.
                    group.ids.get_mut(&id).expect("settling layout").refused = true;
                    self.page.pop();
                    continue;
                }
                ReadAs::Reheaded => row.value.as_ref(),
                ReadAs::Converted => work.buffer.as_slice(),
            };
            let Some(bytes) = put_bytes(&row.key, value) else { break };
            if !room.take(1, bytes, len_u64(value.len()), len_u64(row.value.len())) {
                break;
            }
            let Row { key, mut value } = self.page.pop().expect("unconsumed row");
            if let ReadAs::Converted = read {
                value = Box::from(work.buffer.as_slice());
            } else {
                rehead(&mut value, ty.version);
            }
            store.registry.write(&key, &mut value).expect("a planned map row crosses the write edge");
            extras.push(Op::Put { key, value }).expect("settled put fits the commit");
            puts = puts.checked_add(1).expect("settled put count fits");
        }
        if self.page.is_empty() && self.next.is_none() {
            let group = self.current.take().expect("finished type");
            self.ready_to_retire(group);
        }
    }

    fn retire(&mut self, number: u64, bound: u32, touched: &Set<&[u8]>, room: &mut Room, extras: &mut List<Op>) {
        for _ in 0..self.retiring.len() {
            if room.ops == 0 {
                break;
            }
            let retiring = self.retiring.iter().next().expect("pending retirement");
            let Some(erases) = erases(&retiring.row, &self.refs, bound, touched, room) else { break };
            let retirement = self.retiring.pop().expect("pending retirement");
            for op in erases {
                extras.push(op).expect("retirement fits the commit");
            }
            for digest in &retirement.row.texts {
                let count = self.refs.get_mut(digest).expect("retiring row names the text");
                *count = count.checked_sub(1).expect("retiring row is counted");
                if *count == 0 {
                    self.refs.remove(digest);
                }
            }
            self.submitted.push(Submitted { number, ty: retirement.ty, id: retirement.row.key.2 });
        }
    }

    fn committed(&mut self, layouts: &mut Layouts, number: u64) {
        for _ in 0..self.submitted.len() {
            let waiting = self.submitted.iter().next().expect("submitted retirement");
            if waiting.number > number {
                break;
            }
            let Submitted { ty, id, .. } = self.submitted.pop().expect("durable retirement");
            let ty = layouts.types.get_mut(ty).expect("retired value type");
            ty.plans.remove(&id);
            let mut kept = List::uncapped();
            for &other in &ty.keeping {
                if other != id {
                    kept.push(other).expect("uncapped kept layouts");
                }
            }
            ty.keeping = List::from_boxed(kept.into_boxed());
        }
    }

    fn walking(&self) -> bool {
        !self.groups.is_empty() || self.current.is_some() || !self.retiring.is_empty()
    }
}

fn written(jobs: &[crate::Job], key: &RowKey, fingerprint: Digest) -> bool {
    for job in jobs {
        if let Some(of) = &job.converts
            && of.family == key.0.as_ref()
            && of.value_type == key.1
            && let crate::Layouts::One(named) = of.from
            && named == fingerprint
        {
            return true;
        }
    }
    false
}

fn digests(row: &crate::registry_v1::Layout) -> Set<Digest> {
    let mut digests = Set::with_capacity(row.held.len().checked_add(1).expect("bounded held texts"));
    digests.insert(row.text).expect("own text fits");
    for &digest in &row.held {
        digests.insert(digest).expect("held text fits");
    }
    digests
}

fn may_hold(store: &Store, group: &Group) -> bool {
    for (&id, _) in &group.ids {
        if check::count(store, &store.registry, group.ty, id) != Some(0) {
            return true;
        }
    }
    false
}

fn range_of(layouts: &Layouts, ty: u32, from: Option<&[u8]>) -> Option<Range> {
    let start = match from {
        Some(from) => Included(from),
        None => Unbounded,
    };
    for (from, range) in layouts.ranges.range::<[u8], _>((start, Unbounded)) {
        if range.ty == ty {
            return Some(Range { start: from.clone(), end: Some(range.to.clone()) });
        }
    }
    None
}

#[derive(Debug)]
struct Room {
    ops: u32,
    bytes: u64,
    growth: u64,
}
impl Room {
    fn take(&mut self, ops: u32, bytes: u64, after: u64, before: u64) -> bool {
        let growth = after.saturating_sub(before);
        let (Some(ops), Some(bytes), Some(spare)) =
            (self.ops.checked_sub(ops), self.bytes.checked_sub(bytes), self.growth.checked_sub(growth))
        else {
            return false;
        };
        self.ops = ops;
        self.bytes = bytes;
        self.growth = spare.saturating_add(before.saturating_sub(after));
        true
    }
}

fn put_bytes(key: &[u8], value: &[u8]) -> Option<u64> {
    Some(u64::from(skein_kv::put_bytes(u32::try_from(key.len()).ok()?, u32::try_from(value.len()).ok()?)?))
}

fn erases(
    row: &Candidate,
    refs: &Map<Digest, u32>,
    bound: u32,
    touched: &Set<&[u8]>,
    room: &mut Room,
) -> Option<Box<[Op]>> {
    let mut keys = List::with_capacity(room.ops.min(row.texts.len().checked_add(1)?));
    keys.push(rows::key(&row.key, bound)?).ok()?;
    for digest in &row.texts {
        if refs.get(digest) == Some(&1) {
            keys.push(keys::encode_own(&keys::Own::Text(*digest), bound)?).ok()?;
        }
    }
    let mut bytes = 0_u64;
    for key in &keys {
        if touched.contains(key.as_ref()) {
            return None;
        }
        bytes = bytes.checked_add(u64::from(skein_kv::erase_bytes(u32::try_from(key.len()).ok()?)?))?;
    }
    if !room.take(keys.len(), bytes, 0, 0) {
        return None;
    }
    let mut erases = List::with_capacity(keys.len());
    for key in keys.into_boxed() {
        erases.push(Op::Erase { key }).ok()?;
    }
    Some(erases.into_boxed())
}

pub(crate) fn ride(store: &mut Store, number: u64, root: Box<[Op]>, count: u32, frame: u64) -> Box<[Op]> {
    if !store.mark_durable() || crate::upgrade::switch::pending(store) || store.rehearsal.is_some() {
        return root;
    }
    let Some(mut walk) = store.registry.walk.take() else { return root };
    if !walk.walking() {
        store.registry.walk = Some(walk);
        return root;
    }
    let extras = carry(store, &mut walk, number, &root, count, frame);
    if walk.walking() || !walk.submitted.is_empty() {
        store.registry.walk = Some(walk);
    }
    if extras.is_empty() {
        return root;
    }
    let mut ops = List::from_boxed(root);
    ops.raise(store.limits.store.ops);
    for op in extras.into_boxed() {
        ops.push(op).expect("root and settled operations fit");
    }
    ops.into_boxed()
}

pub(super) fn committed(layouts: &mut Layouts, number: u64) {
    let Some(mut walk) = layouts.walk.take() else { return };
    // A queued retirement is ordered after all its puts, but reads still see
    // durable rows until that prefix commits.
    walk.committed(layouts, number);
    if walk.walking() || !walk.submitted.is_empty() {
        layouts.walk = Some(walk);
    }
}

fn carry(store: &mut Store, walk: &mut Walk, number: u64, required: &[Op], count: u32, frame: u64) -> List<Op> {
    let limits = store.limits.store;
    let remaining = limits.ops.saturating_sub(count);
    let mut extras = List::with_capacity(remaining);
    if remaining == 0 {
        return extras;
    }
    let Some((bytes, growth)) = store.stores.store.spare_for(&limits, required, frame) else { return extras };
    let mut room = Room { ops: remaining, bytes, growth };
    let mut keys = Set::with_capacity(count);
    for op in required {
        if let Op::Put { key, .. } | Op::Erase { key } = op {
            keys.insert(key.as_ref()).expect("root keys bounded by the commit");
        }
    }
    walk.retire(number, limits.key, &keys, &mut room, &mut extras);
    walk.fill(store);
    walk.drain(store, &keys, &mut room, &mut extras);
    extras
}
