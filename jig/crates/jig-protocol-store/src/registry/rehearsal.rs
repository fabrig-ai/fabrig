//! Decode stored heads by their own texts before the open plans any change
//! (protocol/store/opening-rehearsal.md, 1, and protocol/store/layouts.md, 4).
use super::{Layouts, head, rehead, rows, texts};
use crate::{OpenRefusal, Store};
use alloc::boxed::Box;
use skein_codec::{
    schema::{FieldIx, Held, Schema, Shape, Type},
    value::{Node, Value},
};

#[derive(Debug)]
pub(crate) struct Decoder {
    layouts: Box<Layouts>,
}
impl Decoder {
    pub(crate) fn load(store: &mut Store) -> Result<Self, OpenRefusal> {
        let mut layouts = Box::new(Layouts::new());
        layouts.prepared_schemas = store.prepared_schemas;
        rows::open(store, &mut layouts, false)?;
        Ok(Self { layouts })
    }

    pub(crate) fn family(&mut self) {
        texts::clear(&mut self.layouts);
    }

    pub(crate) fn decode(&mut self, family: &str, number: u16, value: &mut Value, bytes: Box<[u8]>) -> Result<(), u16> {
        self.decoded(family, number, value, bytes)?;
        Ok(())
    }

    pub(crate) fn seed_frames(&mut self, store: &Store) -> Result<(), OpenRefusal> {
        let mut binary = texts::Binary::new(&store.owner_sets, &mut self.layouts)?;
        for set in store.owner_sets.sets {
            for family in set.families {
                if let Some(frames) = &family.frames {
                    rows::seed_frames(&mut self.layouts, &mut binary, family, frames)
                        .ok_or(OpenRefusal::Set { owner: set.owner })?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn transcript(
        &mut self,
        family: &str,
        number: u16,
        smith: &str,
        value: &mut Value,
        bytes: Box<[u8]>,
    ) -> Result<Box<[u8]>, u16> {
        let (schema, decl, node) = self.decoded(family, number, value, bytes)?;
        let version = schema.version();
        let Shape::Record { first, count, .. } = schema.declaration(decl).shape else { return Err(version) };
        for offset in 0..count {
            let field = FieldIx::new(first.index().checked_add(offset).ok_or(version)?);
            if let Type::Bytes { held: Held::Record { family, .. }, .. } = schema.ty(schema.field(field).ty)
                && schema.spelled(family) == smith.as_bytes()
            {
                let node = value.field(node, field).or(Err(version))?;
                return Ok(Box::from(value.content(node).or(Err(version))?));
            }
        }
        Err(version)
    }

    pub(crate) fn decoded(
        &mut self,
        family: &str,
        number: u16,
        value: &mut Value,
        mut bytes: Box<[u8]>,
    ) -> Result<(&Schema, skein_codec::schema::DeclIx, Node), u16> {
        let id = head(&bytes).ok_or(0_u16)?;
        let key = (Box::from(family), number, id);
        if let Some(row) = self.layouts.rows.get(&key) {
            let digest = row.value.text;
            texts::parse(&mut self.layouts, digest).ok_or(id)?;
            let row = self.layouts.rows.get(&key).ok_or(id)?;
            let text = self.layouts.texts.get(&digest).ok_or(id)?;
            let schema = text.schema().ok_or(id)?;
            let root = schema.root(&row.value.root, row.value.variant.as_deref()).ok_or(id)?;
            // Generic decoders, like generated ones, read the family's version,
            // not the layout id. Only this owned copy's head is changed.
            rehead(&mut bytes, schema.version());
            let Ok(node) = value.decode(schema, root.decl, bytes) else { return Err(schema.version()) };
            if let Some(expected) = root.variant {
                let node = match schema.declaration(root.decl).shape {
                    Shape::Enum { .. } => node,
                    Shape::Record { first, .. } => match value.field(node, first) {
                        Ok(node) => node,
                        Err(_) => return Err(schema.version()),
                    },
                };
                if value.tag(node) != Ok(expected) {
                    return Err(schema.version());
                }
            }
            return Ok((schema, root.decl, node));
        }
        Err(id)
    }
}
