//! Worker changes as kept in the scratch map (protocol/store/opening-rehearsal.md, 1).
use crate::{KeptWorker, Store, keys, pages, registry::rehearsal::Decoder};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{
    CoreKey, Key,
    settings::{Key as SettingKey, Setting},
};
use skein_codec::{
    schema::{DeclIx, FieldIx, Schema, Shape},
    value::{Node, Value},
};
use skein_lib::List;

pub(super) fn kept(store: &mut Store) -> Option<Box<[KeptWorker]>> {
    let mut range = keys::worker_changes();
    let mut workers = List::uncapped();
    let mut decoder = None;
    let mut value = Value::with_room(store.limits.store.value);
    for _ in 0..=store.stores.store.len() {
        let page = pages::load(store, range.clone()).ok()?;
        for row in page.rows {
            let Key::Core(CoreKey::Settings(SettingKey::Changed(Setting::Worker(number)))) = keys::decode(&row.key)?
            else {
                return None;
            };
            let decoder = match &mut decoder {
                Some(decoder) => decoder,
                slot @ None => slot.insert(Decoder::load(store).ok()?),
            };
            let (schema, decl, node) = decoder
                .decoded("jig-store-settings", crate::patch_set::CHANGED_SETTINGS, &mut value, row.value)
                .ok()?;
            workers.push(worker(number, &value, schema, decl, node)?).ok()?;
        }
        match page.next {
            Some(next) => range.start = next,
            None => return Some(workers.into_boxed()),
        }
    }
    None
}

fn worker(number: u32, value: &Value, schema: &Schema, decl: DeclIx, node: Node) -> Option<KeptWorker> {
    let item = value.field(node, field(schema, decl, b"value")?).ok()?;
    let Some(item) = value.some(item).ok()? else {
        // A removal without a value masks the default at this number.
        return Some(KeptWorker { number, name: Box::new([]), withdrawn: true });
    };
    let variant = schema.variant(value.tag(item).ok()?);
    if schema.spelled(variant.name) != b"worker" {
        return None;
    }
    let payload = value.payload(item).ok()??;
    let decl = variant.record?;
    let stored = value.uint(value.field(payload, field(schema, decl, b"number")?).ok()?).ok()?;
    if stored != u64::from(number) {
        return None;
    }
    let name = value.content(value.field(payload, field(schema, decl, b"name")?).ok()?).ok()?;
    let withdrawn = value.boolean(value.field(payload, field(schema, decl, b"withdrawn")?).ok()?).ok()?;
    Some(KeptWorker { number, name: Box::from(name), withdrawn })
}

fn field(schema: &Schema, decl: DeclIx, name: &[u8]) -> Option<FieldIx> {
    let Shape::Record { first, count, .. } = schema.declaration(decl).shape else { return None };
    for offset in 0..count {
        let at = FieldIx::new(first.index().checked_add(offset)?);
        if schema.spelled(schema.field(at).name) == name {
            return Some(at);
        }
    }
    None
}
