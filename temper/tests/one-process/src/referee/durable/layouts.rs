//! The durable observer decodes registered rows through their retained layouts.
//! See temper/docs/design/testing/tiers.md, 1.5 and temper/docs/design/testing/checks.md, 1.
use jig_protocol_store::core_v6::Record as _;
use jig_protocol_store::{KeptFamily, Owner, ValueType, keys, registry_v1 as wire};
use skein_codec::{
    layout::{Misconverted, Plan, Workspace},
    schema::{self, Schema},
};
use skein_lib::{Buffer, Reader};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

struct Decoder {
    from: u16,
    to: u16,
    plan: Plan,
    text: [u8; 32],
    held: skein_lib::List<[u8; 32]>,
}

enum Parsed {
    Binary(&'static Schema),
    Registry(Box<Schema>),
}

impl AsRef<Schema> for Parsed {
    fn as_ref(&self) -> &Schema {
        match self {
            Self::Binary(schema) => schema,
            Self::Registry(schema) => schema,
        }
    }
}

#[derive(Clone, Default)]
struct Prepared {
    texts: BTreeMap<[u8; 32], Rc<Parsed>>,
    decoders: BTreeMap<Vec<u8>, Rc<Decoder>>,
}

#[derive(Clone, Default)]
pub(super) struct Layouts {
    // Pending observations share the registry bytes until a row changes.
    rows: Rc<BTreeMap<Vec<u8>, Box<[u8]>>>,
    // Tentative and committed observations of the same rows share preparation.
    // A changed row detaches the view and drops only the entries it fed.
    prepared: Rc<RefCell<Prepared>>,
}

impl Layouts {
    pub(super) fn observe(&mut self, ops: &[skein_kv::Op]) {
        for op in ops {
            let key = match op {
                skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. } => {
                    unreachable!("decoded commit frames contain map operations")
                }
                skein_kv::Op::Put { key, .. } | skein_kv::Op::Erase { key } => key,
            };
            let Some(registry @ (keys::Own::Layout { .. } | keys::Own::Text(_))) = keys::decode_own(key) else {
                continue;
            };
            let changed = match op {
                skein_kv::Op::Put { key, value } => self.rows.get(key.as_ref()) != Some(value),
                skein_kv::Op::Erase { key } => self.rows.contains_key(key.as_ref()),
                skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. } => {
                    unreachable!("decoded commit frames contain map operations")
                }
            };
            if !changed {
                continue;
            }
            let prepared = Rc::make_mut(&mut self.prepared).get_mut();
            match registry {
                keys::Own::Layout { .. } => {
                    prepared.decoders.remove(key.as_ref());
                }
                keys::Own::Text(digest) => {
                    prepared.texts.remove(&digest);
                    prepared
                        .decoders
                        .retain(|_, decoder| decoder.text != digest && !decoder.held.as_slice().contains(&digest));
                }
                _ => unreachable!("only registry text and layout rows are observed"),
            }
            let rows = Rc::make_mut(&mut self.rows);
            match op {
                skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. } => {
                    unreachable!("decoded commit frames contain map operations")
                }
                skein_kv::Op::Put { key, value } => {
                    rows.insert(key.to_vec(), value.clone());
                }
                skein_kv::Op::Erase { key } => {
                    rows.remove(key.as_ref());
                }
            }
        }
    }

    pub(super) fn clear_preparation(&mut self) {
        self.prepared = Rc::default();
    }

    fn text(&self, digest: [u8; 32]) -> Result<Rc<Parsed>, String> {
        if let Some(schema) = self.prepared.borrow().texts.get(&digest) {
            return Ok(schema.clone());
        }
        let key = keys::encode_own(&keys::Own::Text(digest), 1024).expect("registry text key");
        let bytes = self.rows.get(key.as_ref()).ok_or("registered text present")?;
        let text = wire::Text::decode(&wire::CEILINGS, &mut Reader::new(bytes))
            .map_err(|why| format!("registry text: {why:?}"))?;
        // A digest names a row; only its decoded bytes can establish equality with the binary text.
        let parsed = if let Some((_, schema)) =
            binary_schemas()?.iter().find(|(binary, _)| binary.as_bytes() == text.text.as_ref())
        {
            Parsed::Binary(schema)
        } else {
            Parsed::Registry(Box::new(schema::parse(&text.text).map_err(|why| format!("registry schema: {why:?}"))?))
        };
        let schema = Rc::new(parsed);
        self.prepared.borrow_mut().texts.insert(digest, schema.clone());
        Ok(schema)
    }

    fn decoder(&self, family: &KeptFamily, ty: &ValueType, row: &[u8]) -> Result<Decoder, String> {
        let layout = wire::Layout::decode(&wire::CEILINGS, &mut Reader::new(row))
            .map_err(|why| format!("registry layout: {why:?}"))?;
        let old = self.text(layout.text)?;
        let old_held = layout.held.iter().map(|digest| self.text(*digest)).collect::<Result<Vec<_>, _>>()?;
        let binary = binary_schemas()?;
        let new = &binary.iter().find(|(text, _)| *text == family.current).expect("binary family present").1;
        let new_held = binary.iter().map(|(_, schema)| schema).collect::<Vec<_>>();
        let old = old.as_ref().as_ref();
        let old_root = old.root(&layout.root, layout.variant.as_deref()).ok_or("stored root present")?;
        let new_root = new.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).ok_or("binary root present")?;
        let plan = Plan::derive(
            old,
            old_root,
            &old_held.iter().map(|schema| schema.as_ref().as_ref()).collect::<Vec<_>>(),
            new,
            new_root,
            &new_held,
        )
        .map_err(|why| format!("registered value readable: {why:?}"))?;
        Ok(Decoder { from: old.version(), to: new.version(), plan, text: layout.text, held: layout.held })
    }

    pub(super) fn serving(&mut self, key: &[u8], mut bytes: Box<[u8]>) -> Result<Box<[u8]>, String> {
        let Some((family, ty)) = resolve(key) else { return Ok(bytes) };
        let Some(id) = Reader::new(&bytes).u16() else { return Ok(bytes) };
        let key = keys::encode_own(&keys::Own::Layout { family: family.name.into(), number: ty.number, id }, 1024)
            .expect("registry layout key");
        let prepared = self.prepared.borrow().decoders.get(key.as_ref()).cloned();
        let decoder = if let Some(decoder) = prepared {
            decoder
        } else {
            // Unit snapshots without registry rows carry the binary's family version.
            let Some(row) = self.rows.get(key.as_ref()) else { return Ok(bytes) };
            let decoder = Rc::new(self.decoder(family, ty, row)?);
            self.prepared.borrow_mut().decoders.insert(key.to_vec(), decoder.clone());
            decoder
        };
        if decoder.plan.keeps() {
            bytes.get_mut(..2).expect("stored head").copy_from_slice(&decoder.to.to_be_bytes());
            return Ok(bytes);
        }
        bytes.get_mut(..2).expect("stored head").copy_from_slice(&decoder.from.to_be_bytes());
        let mut work = Workspace::new();
        if !work.fit(&decoder.plan) {
            return Err("registered workspace fits".into());
        }
        let mut capacity =
            u32::try_from(bytes.len()).map_err(|why| format!("registered value length: {why:?}"))?.max(1);
        loop {
            let mut out = Buffer::with_capacity(capacity);
            match decoder.plan.convert(&bytes, &mut out, &mut work) {
                Ok(()) => return Ok(out.as_slice().into()),
                Err(Misconverted::Room) => {
                    capacity = capacity.checked_mul(2).ok_or("registered conversion fits u32")?;
                }
                Err(why) => return Err(format!("registered conversion: {why:?}")),
            }
        }
    }
}

/// Only the binary's immutable schemas are shared between worlds, never stored registry rows.
fn binary_schemas() -> Result<&'static [(&'static str, Schema)], String> {
    type Schemas = Result<Box<[(&'static str, Schema)]>, String>;
    static SCHEMAS: std::sync::OnceLock<Schemas> = std::sync::OnceLock::new();
    SCHEMAS
        .get_or_init(|| {
            let mut schemas = Vec::new();
            for set in temper_engine_service::store::OWNER_SETS.sets {
                for family in set.families {
                    if !family.current.is_empty() {
                        let schema = schema::parse(family.current.as_bytes())
                            .map_err(|why| format!("binary schema: {why:?}"))?;
                        schemas.push((family.current, schema));
                    }
                }
            }
            Ok(schemas.into_boxed_slice())
        })
        .as_deref()
        .map_err(Clone::clone)
}

fn resolve(key: &[u8]) -> Option<(&'static KeptFamily, &'static ValueType)> {
    for set in temper_engine_service::store::OWNER_SETS.sets {
        let relative = match set.owner {
            Owner::Application => key.strip_prefix(&[keys::APPLICATION]),
            Owner::Skein | Owner::Smith | Owner::Jig => Some(key),
        };
        let Some(relative) = relative else { continue };
        for family in set.families {
            for ty in family.types {
                if ty.keys.iter().any(|range| relative >= range.from && relative < range.to) {
                    return Some((family, ty));
                }
            }
        }
    }
    None
}
#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::fixtures;
    use jig_domain_core as core;
    use jig_domain_tasks as tasks;
    use jig_protocol_store::codec;
    use skein_kv::Op;

    fn encoded_text(value: &str) -> Box<[u8]> {
        let value = wire::Text { text: value.as_bytes().into() };
        value.encode(&jig_protocol_store::registry_v1::CEILINGS).expect("measured text")
    }

    const ATTEMPT_LAYOUT: u16 = 91;
    const OTHER_LAYOUT: u16 = 92;

    pub(in crate::referee::durable) fn attempt() -> core::Record {
        core::Record::Tasks(tasks::Stored::Attempt(tasks::AttemptRow {
            task: 1,
            attempt: 2,
            claimed: None,
            started: None,
            ended: None,
            end: None,
            phases: None,
        }))
    }

    fn layout_key(key: &[u8], id: u16) -> Box<[u8]> {
        let (family, ty) = resolve(key).expect("registered task value");
        keys::encode_own(&keys::Own::Layout { family: family.name.into(), number: ty.number, id }, 1024)
            .expect("layout key")
    }

    fn digest(text: &str) -> [u8; 32] {
        let mut digest = skein_lib::sha256::Sha256::new();
        digest.update(text.as_bytes());
        digest.finish()
    }

    fn text_key(digest: [u8; 32]) -> Box<[u8]> {
        keys::encode_own(&keys::Own::Text(digest), 1024).expect("text key")
    }

    pub(in crate::referee::durable) fn registered_attempt() -> (Layouts, Box<[u8]>, Box<[u8]>) {
        let row = attempt();
        let key = keys::encode(&row.key(), 1024).expect("task key");
        let (family, _) = resolve(&key).expect("registered task value");
        let schema = schema::parse(family.current.as_bytes()).expect("binary schema");
        let fingerprint =
            schema.layout(schema.root(b"Row", Some(b"attempt")).expect("attempt root"), &[]).expect("layout").digest();
        let digest = digest(family.current);
        let layout = wire::Layout {
            root: b"Row".as_slice().into(),
            variant: Some(b"attempt".as_slice().into()),
            layout: fingerprint,
            text: digest,
            held: skein_lib::List::with_capacity(0),
        };
        let out = layout.encode(&jig_protocol_store::registry_v1::CEILINGS).expect("measured layout");
        let mut layouts = Layouts::default();
        layouts.observe(&[
            Op::Put { key: text_key(digest), value: encoded_text(family.current) },
            Op::Put { key: layout_key(&key, ATTEMPT_LAYOUT), value: out },
        ]);
        let limits = fixtures::codec();
        let mut bytes = codec::encode(&row, &limits, fixtures::store_names()).expect("task row");
        bytes.get_mut(..2).expect("head").copy_from_slice(&ATTEMPT_LAYOUT.to_be_bytes());
        (layouts, key, bytes)
    }

    pub(in crate::referee::durable) fn registry_ops(layouts: &Layouts) -> Vec<Op> {
        layouts.rows.iter().map(|(key, value)| Op::Put { key: key.as_slice().into(), value: value.clone() }).collect()
    }

    pub(in crate::referee::durable) fn another_layout(layouts: &Layouts, key: &[u8]) -> Op {
        Op::Put {
            key: layout_key(key, OTHER_LAYOUT),
            value: layouts.rows[layout_key(key, ATTEMPT_LAYOUT).as_ref()].clone(),
        }
    }

    fn prepared_decoder(layouts: &Layouts, key: &[u8]) -> Rc<Decoder> {
        layouts.prepared.borrow().decoders[key].clone()
    }

    pub(in crate::referee::durable) fn assert_same_decoder(before: &Layouts, after: &Layouts, key: &[u8]) {
        let key = layout_key(key, ATTEMPT_LAYOUT);
        assert!(Rc::ptr_eq(&prepared_decoder(before, &key), &prepared_decoder(after, &key)), "same attempt decoder");
    }

    pub(in crate::referee::durable) fn assert_empty_preparation(layouts: &Layouts) {
        let prepared = layouts.prepared.borrow();
        assert!(prepared.texts.is_empty() && prepared.decoders.is_empty(), "recovery starts empty");
    }

    #[test]
    fn stored_layout_heads_reach_the_durable_observer_as_family_versions() {
        let (mut layouts, key, bytes) = registered_attempt();
        let bytes = layouts.serving(&key, bytes).expect("read registered value");
        let row = attempt();
        assert_eq!(codec::decode(&row.key(), &bytes, &fixtures::codec(), fixtures::store_names()), Some(row));
    }

    #[test]
    fn only_byte_identical_registry_texts_share_the_binary_parse() {
        let (mut layouts, key, _) = registered_attempt();
        let (family, _) = resolve(&key).expect("registered task value");
        let binary = &binary_schemas()
            .expect("binary schemas")
            .iter()
            .find(|(text, _)| *text == family.current)
            .expect("attempt family")
            .1;
        let digest = digest(family.current);
        let edited = format!("\n{}", family.current);
        for (text, shares_binary) in [(family.current, true), (edited.as_str(), false)] {
            layouts.observe(&[Op::Put { key: text_key(digest), value: encoded_text(text) }]);
            let schema = layouts.text(digest).expect("stored text parses");
            let schema = schema.as_ref().as_ref();
            assert_eq!(std::ptr::eq(schema, binary), shares_binary);
            assert_eq!(schema.text(), text.as_bytes());
        }
    }

    #[test]
    fn a_registry_edit_detaches_preparation_from_the_unchanged_observation() {
        let (mut committed, key, bytes) = registered_attempt();
        // Learn the decoder through a tentative view, just as pending commits do.
        let mut pending = committed.clone();
        let current = pending.serving(&key, bytes.clone()).expect("pending registered value");
        assert_same_decoder(&pending, &committed, &key);
        assert_eq!(committed.serving(&key, bytes.clone()).expect("committed registered value"), current);
        for (registry_key, value) in committed.rows.as_ref().clone() {
            let mut pending = committed.clone();
            pending.observe(&[skein_kv::Op::Put { key: registry_key.clone().into(), value: value.clone() }]);
            assert_eq!(pending.serving(&key, bytes.clone()).expect("same registry row"), current);
            assert!(Rc::ptr_eq(&pending.rows, &committed.rows), "unchanged registry rows share storage");
            pending.observe(&[skein_kv::Op::Put { key: registry_key.clone().into(), value: Box::default() }]);
            assert!(!Rc::ptr_eq(&pending.rows, &committed.rows), "changed registry rows detach");
            assert!(pending.serving(&key, bytes.clone()).is_err(), "a changed registry row must be checked");
            assert_eq!(committed.serving(&key, bytes.clone()).expect("unchanged observation"), current);
            pending.observe(&[skein_kv::Op::Put { key: registry_key.clone().into(), value }]);
            assert_eq!(pending.serving(&key, bytes.clone()).expect("restored registry row"), current);
            pending.observe(&[skein_kv::Op::Erase { key: registry_key.clone().into() }]);
            if matches!(keys::decode_own(&registry_key), Some(keys::Own::Layout { .. })) {
                assert_eq!(pending.serving(&key, bytes.clone()).expect("unregistered value"), bytes);
            } else {
                assert!(pending.serving(&key, bytes.clone()).is_err(), "a missing schema cannot use the cached text");
            }
        }
        assert_same_decoder(&pending, &committed, &key);
        assert_eq!(Layouts::default().serving(&key, bytes.clone()).expect("fresh world"), bytes);
    }

    fn other_family(family: &KeptFamily) -> &'static KeptFamily {
        temper_engine_service::store::OWNER_SETS
            .sets
            .iter()
            .flat_map(|set| set.families)
            .find(|other| !other.current.is_empty() && other.current != family.current)
            .expect("another binary family text")
    }

    #[test]
    fn unrelated_registry_edits_keep_prepared_text_and_decoder() {
        let (mut layouts, key, bytes) = registered_attempt();
        let (family, _) = resolve(&key).expect("registered task value");
        let primary = digest(family.current);
        let other = other_family(family).current;
        let expected = layouts.serving(&key, bytes.clone()).expect("read registered value");
        let before = layouts.clone();
        let schema = before.text(primary).expect("primary text");
        let edits = [
            another_layout(&layouts, &key),
            Op::Erase { key: layout_key(&key, OTHER_LAYOUT) },
            Op::Put { key: text_key(digest(other)), value: encoded_text(other) },
            Op::Erase { key: text_key(digest(other)) },
        ];
        for edit in edits {
            layouts.observe(&[edit]);
            assert_eq!(layouts.serving(&key, bytes.clone()).expect("unchanged registered value"), expected);
            assert_same_decoder(&before, &layouts, &key);
            assert!(Rc::ptr_eq(&schema, &layouts.text(primary).expect("unchanged primary text")));
        }
    }

    fn with_held_texts(layouts: &Layouts, key: &[u8], held: [[u8; 32]; 2]) -> Op {
        let key = layout_key(key, ATTEMPT_LAYOUT);
        let mut layout = wire::Layout::decode(&wire::CEILINGS, &mut Reader::new(&layouts.rows[key.as_ref()]))
            .expect("attempt layout");
        layout.held = skein_lib::List::from_boxed(Box::new(held));
        Op::Put { key, value: layout.encode(&wire::CEILINGS).expect("layout with held texts") }
    }

    fn compare_with_fresh(layouts: &mut Layouts, key: &[u8], bytes: &[u8], step: &str) -> Result<Box<[u8]>, String> {
        let mut fresh = Layouts { rows: layouts.rows.clone(), ..Layouts::default() };
        let served = layouts.serving(key, bytes.into());
        assert_eq!(served, fresh.serving(key, bytes.into()), "{step}");
        served
    }

    #[test]
    fn registry_edits_and_recovery_match_an_empty_preparation() {
        let (registered, key, bytes) = registered_attempt();
        let (family, _) = resolve(&key).expect("registered task value");
        let other = other_family(family).current;
        let (primary, held) = (digest(family.current), digest(other));
        let layout = layout_key(&key, ATTEMPT_LAYOUT);
        let mut layouts = Layouts::default();
        let mut puts = registry_ops(&registered);
        puts.push(Op::Put { key: text_key(held), value: encoded_text(other) });
        puts.push(with_held_texts(&registered, &key, [primary, held]));
        let read = |layouts: &mut Layouts, step: &str| compare_with_fresh(layouts, &key, &bytes, step);
        layouts.observe(&puts);
        let current = read(&mut layouts, "put").expect("read registered value with held texts");
        let before = layouts.clone();
        layouts.observe(&registry_ops(&layouts));
        assert_eq!(read(&mut layouts, "unchanged put").expect("unchanged registered value"), current);
        assert_same_decoder(&before, &layouts, &key);

        for (input, name) in
            [(text_key(primary), "primary text"), (text_key(held), "held text"), (layout.clone(), "layout")]
        {
            let value = layouts.rows[input.as_ref()].clone();
            let changed = Op::Put { key: input.clone(), value: Box::default() };
            let erased = Op::Erase { key: input.clone() };
            // Erase both after the changed put and directly from a warm cache.
            let edits = [&[changed, erased.clone()][..], &[erased][..]];
            for edits in edits {
                let before = prepared_decoder(&layouts, &layout);
                let text = match keys::decode_own(&input) {
                    Some(keys::Own::Text(digest)) => Some((digest, layouts.text(digest).expect("warm text"))),
                    _ => None,
                };
                for edit in edits {
                    let erase = matches!(edit, Op::Erase { .. });
                    let step = format!("{} {name}", if erase { "erase" } else { "change" });
                    layouts.observe(std::slice::from_ref(edit));
                    let edited = read(&mut layouts, &step);
                    if erase && input == layout {
                        assert_eq!(edited.expect("missing layout leaves the bytes alone"), bytes);
                    } else {
                        assert!(edited.is_err(), "malformed layout or missing/malformed text must not be served");
                    }
                    assert!(!layouts.prepared.borrow().decoders.contains_key(layout.as_ref()));
                }
                layouts.observe(&[Op::Put { key: input.clone(), value: value.clone() }]);
                let served = read(&mut layouts, &format!("restore {name}")).expect("restored input");
                let row = attempt();
                assert_eq!(codec::decode(&row.key(), &served, &fixtures::codec(), fixtures::store_names()), Some(row));
                assert!(!Rc::ptr_eq(&before, &prepared_decoder(&layouts, &layout)));
                if let Some((digest, before)) = text {
                    assert!(!Rc::ptr_eq(&before, &layouts.text(digest).expect("restored text")));
                }
            }
        }

        let before = prepared_decoder(&layouts, &layout);
        let schema = layouts.text(primary).expect("primary text");
        let held_schema = layouts.text(held).expect("held text");
        layouts.observe(&[with_held_texts(&layouts, &key, [held, primary])]);
        assert_eq!(read(&mut layouts, "held reorder").expect("reordered held texts"), current);
        let reordered = prepared_decoder(&layouts, &layout);
        assert!(!Rc::ptr_eq(&before, &reordered));
        assert!(Rc::ptr_eq(&schema, &layouts.text(primary).expect("unchanged primary text")));
        assert!(Rc::ptr_eq(&held_schema, &layouts.text(held).expect("unchanged held text")));
        let mut pending = layouts.clone();
        pending.observe(&[with_held_texts(&pending, &key, [primary, held])]);
        assert_eq!(read(&mut pending, "pending reorder").expect("pending registry"), current);
        assert!(!Rc::ptr_eq(&reordered, &prepared_decoder(&pending, &layout)));
        drop(pending);
        assert_eq!(read(&mut layouts, "rollback").expect("committed registry"), current);
        assert!(Rc::ptr_eq(&reordered, &prepared_decoder(&layouts, &layout)));
        assert!(Rc::ptr_eq(&schema, &layouts.text(primary).expect("committed text")));

        let mut store = super::super::Store::new();
        store.layouts = layouts;
        store.recovering();
        assert_empty_preparation(&store.layouts);
        assert_eq!(read(&mut store.layouts, "recovery").expect("retained registry rows"), current);
        assert!(!Rc::ptr_eq(&reordered, &prepared_decoder(&store.layouts, &layout)));
        assert!(!Rc::ptr_eq(&schema, &store.layouts.text(primary).expect("retained text row")));
    }
}
