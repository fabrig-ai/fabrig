//! Decode stored heads by their own texts before the open plans any change
//! (protocol/store/opening.md, 5, and protocol/store/layouts.md, 4).
use super::{Layouts, head, rehead, rows};
use crate::{OpenRefusal, Store};
use alloc::boxed::Box;
use skein_codec::{schema::Shape, value::Value};

pub(crate) struct Decoder {
    layouts: Box<Layouts>,
}
impl Decoder {
    pub(crate) fn load(store: &mut Store) -> Result<Self, OpenRefusal> {
        let mut layouts = Box::new(Layouts::new());
        rows::load(store, &mut layouts)?;
        Ok(Self { layouts })
    }

    pub(crate) fn seeded(&self) -> bool {
        !self.layouts.rows.is_empty()
    }

    pub(crate) fn decode(&self, family: &str, number: u16, value: &mut Value, mut bytes: Box<[u8]>) -> Result<(), u16> {
        let id = head(&bytes).ok_or(0_u16)?;
        for ((name, ty, layout), row) in &self.layouts.rows {
            if name.as_ref() != family || *ty != number || *layout != id {
                continue;
            }
            let text = self.layouts.texts.get(row.value.text()).ok_or(id)?;
            let schema = text.schema.as_ref().ok_or(id)?;
            let root = schema.root(row.value.root(), row.value.variant().as_deref()).ok_or(id)?;
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
            return Ok(());
        }
        Err(id)
    }
}
