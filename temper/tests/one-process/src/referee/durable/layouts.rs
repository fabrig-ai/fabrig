//! The durable observer decodes registered rows through their retained layouts.
use jig_protocol_store::{KeptFamily, Owner, ValueType, keys, registry_v1 as wire};
use skein_codec::{
    layout::{Misconverted, Plan, Workspace},
    schema::{self, Schema},
};
use skein_lib::{Buffer, Reader};
use std::collections::BTreeMap;

#[derive(Clone)]
struct Decoder {
    from: u16,
    to: u16,
    plan: Plan,
}

#[derive(Clone, Default)]
pub(super) struct Layouts {
    rows: BTreeMap<Vec<u8>, Box<[u8]>>,
    decoders: BTreeMap<Vec<u8>, Decoder>,
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
            if !matches!(keys::decode_own(key), Some(keys::Own::Layout { .. } | keys::Own::Text(_))) {
                continue;
            }
            self.decoders.clear();
            match op {
                skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. } => {
                    unreachable!("decoded commit frames contain map operations")
                }
                skein_kv::Op::Put { key, value } => {
                    self.rows.insert(key.to_vec(), value.clone());
                }
                skein_kv::Op::Erase { key } => {
                    self.rows.remove(key.as_ref());
                }
            }
        }
    }

    fn text(&self, digest: [u8; 32]) -> Result<Schema, String> {
        let key = keys::encode_own(&keys::Own::Text(digest), 1024).expect("registry text key");
        let bytes = self.rows.get(key.as_ref()).ok_or("registered text present")?;
        let text = wire::Text::decode(&wire::CEILINGS, &mut Reader::new(bytes))
            .map_err(|why| format!("registry text: {why:?}"))?;
        schema::parse(text.text()).map_err(|why| format!("registry schema: {why:?}"))
    }

    fn decoder(&self, family: &KeptFamily, ty: &ValueType, key: &[u8]) -> Result<Decoder, String> {
        let layout = wire::Layout::decode(&wire::CEILINGS, &mut Reader::new(self.rows.get(key).expect("found layout")))
            .map_err(|why| format!("registry layout: {why:?}"))?;
        let old = self.text(*layout.text())?;
        let old_held = layout.held().iter().map(|digest| self.text(*digest)).collect::<Result<Vec<_>, _>>()?;
        let new = schema::parse(family.current.as_bytes()).map_err(|why| format!("binary schema: {why:?}"))?;
        let mut new_held = Vec::new();
        for set in temper_engine_service::store::PATCH_SETS.sets {
            for family in set.families {
                if !family.current.is_empty() {
                    new_held.push(
                        schema::parse(family.current.as_bytes())
                            .map_err(|why| format!("held binary schema: {why:?}"))?,
                    );
                }
            }
        }
        let old_root = old.root(layout.root(), layout.variant().as_deref()).ok_or("stored root present")?;
        let new_root = new.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).ok_or("binary root present")?;
        let plan = Plan::derive(
            &old,
            old_root,
            &old_held.iter().collect::<Vec<_>>(),
            &new,
            new_root,
            &new_held.iter().collect::<Vec<_>>(),
        )
        .map_err(|why| format!("registered value readable: {why:?}"))?;
        Ok(Decoder { from: old.version(), to: new.version(), plan })
    }

    pub(super) fn serving(&mut self, key: &[u8], mut bytes: Box<[u8]>) -> Result<Box<[u8]>, String> {
        let Some((family, ty)) = resolve(key) else { return Ok(bytes) };
        let Some(id) = Reader::new(&bytes).u16() else { return Ok(bytes) };
        let key = keys::encode_own(&keys::Own::Layout { family: family.name.into(), number: ty.number, id }, 1024)
            .expect("registry layout key");
        // Unit snapshots without registry rows carry the binary's family version.
        if !self.rows.contains_key(key.as_ref()) {
            return Ok(bytes);
        }
        if !self.decoders.contains_key(key.as_ref()) {
            self.decoders.insert(key.to_vec(), self.decoder(family, ty, &key)?);
        }
        let decoder = self.decoders.get(key.as_ref()).expect("planned stored layout");
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

fn resolve(key: &[u8]) -> Option<(&'static KeptFamily, &'static ValueType)> {
    for set in temper_engine_service::store::PATCH_SETS.sets {
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
mod tests {
    use super::*;
    use crate::fixtures;
    use jig_domain_core as core;
    use jig_domain_tasks as tasks;
    use jig_protocol_store::codec;
    use skein_lib::Writer;

    fn encoded_text(value: &str) -> Box<[u8]> {
        let value =
            wire::Text::new(&wire::CEILINGS, wire::TextParts { text: value.as_bytes().into() }).expect("registry text");
        let mut out = Writer::new(usize::try_from(value.measure()).expect("text size"));
        value.encode(&mut out).expect("measured text");
        out.finish()
    }

    #[test]
    fn stored_layout_heads_reach_the_durable_observer_as_family_versions() {
        let row = core::Record::Tasks(tasks::Stored::Attempt(tasks::AttemptRow {
            task: 1,
            attempt: 2,
            claimed: None,
            started: None,
            ended: None,
            end: None,
            phases: None,
        }));
        let key = keys::encode(&row.key(), 1024).expect("task key");
        let (family, ty) = resolve(&key).expect("registered task value");
        let schema = schema::parse(family.current.as_bytes()).expect("binary schema");
        let fingerprint =
            schema.layout(schema.root(b"Row", Some(b"attempt")).expect("attempt root"), &[]).expect("layout").digest();
        let mut digest = skein_lib::sha256::Sha256::new();
        digest.update(family.current.as_bytes());
        let digest = digest.finish();
        let layout = wire::Layout::new(
            &wire::CEILINGS,
            wire::LayoutParts {
                root: b"Row".as_slice().into(),
                variant: Some(b"attempt".as_slice().into()),
                layout: fingerprint,
                text: digest,
                held: skein_lib::List::with_capacity(0),
            },
        )
        .expect("layout row");
        let mut out = Writer::new(usize::try_from(layout.measure()).expect("layout size"));
        layout.encode(&mut out).expect("measured layout");
        let mut layouts = Layouts::default();
        layouts.observe(&[
            skein_kv::Op::Put {
                key: keys::encode_own(&keys::Own::Text(digest), 1024).expect("text key"),
                value: encoded_text(family.current),
            },
            skein_kv::Op::Put {
                key: keys::encode_own(
                    &keys::Own::Layout { family: family.name.into(), number: ty.number, id: 91 },
                    1024,
                )
                .expect("layout key"),
                value: out.finish(),
            },
        ]);
        let limits = fixtures::codec();
        let mut bytes = codec::encode(&row, &limits, fixtures::store_names()).expect("task row");
        bytes.get_mut(..2).expect("head").copy_from_slice(&91_u16.to_be_bytes());
        let bytes = layouts.serving(&key, bytes).expect("read registered value");
        assert_eq!(codec::decode(&row.key(), &bytes, &limits, fixtures::store_names()), Some(row));
    }
}
