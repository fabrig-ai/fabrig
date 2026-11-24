//! Planning takes each text from the shell or parses it, computes each layout
//! and finds each text's held texts once. It releases them when the plans are
//! made (jig/docs/design/protocol/store/layouts.md, 7).
use super::{Digest, Layouts, Text};
use crate::{OpenRefusal, OwnerSets, upgrade::schemas::find};
use alloc::boxed::Box;
use skein_codec::schema::{self, DeclIx, FieldIx, Held, Schema, Shape, Type, TypeIx};
use skein_lib::{List, Map};

pub(super) fn digest(bytes: &[u8]) -> Digest {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(bytes);
    hash.finish()
}
#[expect(clippy::disallowed_methods, reason = "registry root and family names are checked UTF-8 at decoding")]
pub(super) fn string(bytes: &[u8]) -> Option<Box<str>> {
    Some(Box::from(core::str::from_utf8(bytes).ok()?))
}
#[derive(Debug)]
pub(super) enum Cached {
    Parsed(Box<Schema>),
    Prepared(&'static Schema),
}
impl Cached {
    #[cfg_attr(not(test), expect(unused_variables, reason = "only the test parse tally needs the digest"))]
    fn read(prepared: &'static [Schema], bytes: &[u8], digest: Digest) -> Option<Self> {
        if let Some(schema) = find(prepared, bytes) {
            return Some(Self::Prepared(schema));
        }
        #[cfg(test)]
        super::counting::parsed(digest);
        Some(Self::Parsed(Box::new(schema::parse(bytes).ok()?)))
    }
}

type LayoutKey = (Digest, Box<[u8]>, Option<Box<[u8]>>, Box<[Digest]>);
#[derive(Debug)]
pub(super) struct Binary {
    names: Map<&'static str, Digest>,
    held: Map<Digest, List<Digest>>,
    layouts: Map<LayoutKey, Digest>,
    depths: Map<(Digest, Box<[Digest]>), List<u32>>,
}
impl Binary {
    pub(super) fn new(sets: &OwnerSets, layouts: &mut Layouts) -> Result<Self, OpenRefusal> {
        let mut names = Map::uncapped();
        for set in sets.sets {
            for family in set.families {
                let refusal = OpenRefusal::Set { owner: set.owner };
                let digest = keep(layouts, family.current.as_bytes(), true).ok_or(refusal)?;
                names.insert(family.name, digest).expect("uncapped names");
            }
        }
        Ok(Self { names, held: Map::uncapped(), layouts: Map::uncapped(), depths: Map::uncapped() })
    }
    pub(super) fn text(&self, family: &str) -> Option<Digest> {
        self.names.get(family).copied()
    }
    pub(super) fn held(&mut self, layouts: &mut Layouts, text: Digest) -> Option<List<Digest>> {
        if let Some(held) = self.held.get(&text) {
            return Some(held.clone());
        }
        #[cfg(test)]
        super::counting::held(text);
        let mut held = List::with_capacity(crate::registry_v1::CEILINGS.layout_held);
        let mut pending = List::uncapped();
        pending.push(text).ok()?;
        // Each family is entered once; its whole text describes the texts it may hold.
        for index in 0..=self.names.len() {
            let Some(digest) = pending.get(index) else { break };
            parse(layouts, *digest)?;
            let schema = layouts.texts.get(digest)?.schema()?;
            for decl in 0..schema.declarations() {
                let Shape::Record { first, count, .. } = schema.declaration(DeclIx::new(decl)).shape else { continue };
                for offset in 0..count {
                    let field = schema.field(FieldIx::new(first.index().checked_add(offset)?));
                    let Some(name) = holds(schema, field.ty) else { continue };
                    let Some(&digest) = self.names.get(name) else { continue };
                    if digest == text || held.as_slice().contains(&digest) {
                        continue;
                    }
                    held.push(digest).ok()?;
                    pending.push(digest).ok()?;
                }
            }
        }
        self.held.insert(text, held.clone()).expect("uncapped held texts");
        Some(held)
    }
    pub(super) fn shallow(&mut self, layouts: &Layouts, text: Digest, root: DeclIx, held: &List<Digest>) -> bool {
        let key = (text, Box::from(held.as_slice()));
        if !self.depths.contains_key(&key) {
            let Some(text) = layouts.texts.get(&text) else { return false };
            let Some(schema) = text.schema() else { return false };
            let Some(held) = schemas(layouts, held, false) else { return false };
            let depths = super::depth::heights(schema, held.as_slice());
            self.depths.insert(key.clone(), depths).expect("uncapped depth proofs");
        }
        let Some(depths) = self.depths.get(&key) else { return false };
        let Some(&depth) = depths.get(root.index()) else { return false };
        depth != 0
    }
    pub(super) fn layout(
        &mut self,
        layouts: &Layouts,
        text: Digest,
        root: &[u8],
        variant: Option<&[u8]>,
        held: &List<Digest>,
    ) -> Option<Digest> {
        let key = (
            text,
            Box::from(root),
            match variant {
                Some(variant) => Some(Box::from(variant)),
                None => None,
            },
            Box::from(held.as_slice()),
        );
        if let Some(digest) = self.layouts.get(&key) {
            return Some(*digest);
        }
        let schema = layouts.texts.get(&text)?.schema()?;
        let root = schema.root(root, variant)?;
        let held = schemas(layouts, held, false)?;
        #[cfg(test)]
        super::counting::layout();
        let digest = schema.layout(root, held.as_slice()).ok()?.digest();
        self.layouts.insert(key, digest).expect("uncapped computed layouts");
        Some(digest)
    }
}
#[expect(clippy::disallowed_methods, reason = "schema family tokens are ASCII names")]
fn holds(schema: &Schema, mut ty: TypeIx) -> Option<&str> {
    for _ in 0..schema.text().len() {
        match schema.ty(ty) {
            Type::List { item, .. } | Type::Option(item) => ty = item,
            Type::Bytes { held: Held::Record { family, .. }, .. }
            | Type::Secret { held: Held::Record { family, .. }, .. } => {
                return core::str::from_utf8(schema.spelled(family)).ok();
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::Named(_)
            | Type::Bytes { held: Held::Opaque | Held::Pieces(_), .. }
            | Type::Secret { held: Held::Opaque | Held::Pieces(_), .. } => return None,
        }
    }
    None
}
pub(super) fn keep(layouts: &mut Layouts, bytes: &[u8], current: bool) -> Option<Digest> {
    if bytes.len() > skein_lib::usize_of(crate::registry_v1::CEILINGS.text_text) {
        return None;
    }
    let digest = digest(bytes);
    if let Some(held) = layouts.texts.get_mut(&digest) {
        held.current |= current;
    } else {
        let schema = Cached::read(layouts.prepared_schemas, bytes, digest)?;
        layouts
            .texts
            .insert(digest, Text { bytes: Box::from(bytes), schema: Some(schema), stored: false, current })
            .expect("uncapped texts");
    }
    Some(digest)
}

pub(super) fn parse(layouts: &mut Layouts, digest: Digest) -> Option<()> {
    let text = layouts.texts.get_mut(&digest)?;
    if text.schema.is_none() {
        if self::digest(&text.bytes) != digest {
            return None;
        }
        text.schema = Some(Cached::read(layouts.prepared_schemas, &text.bytes, digest)?);
    }
    Some(())
}

pub(super) fn row(layouts: &mut Layouts, key: &super::RowKey) -> Option<()> {
    let row = layouts.rows.get(key)?;
    let text = row.value.text;
    let held = row.value.held.clone();
    parse(layouts, text)?;
    for digest in &held {
        parse(layouts, *digest)?;
    }
    Some(())
}

pub(super) fn clear(layouts: &mut Layouts) {
    clear_schemas(layouts, false);
}

pub(super) fn clear_all(layouts: &mut Layouts) {
    clear_schemas(layouts, true);
    layouts.prepared_schemas = &[];
}

fn clear_schemas(layouts: &mut Layouts, all: bool) {
    let mut digests = List::uncapped();
    for (digest, text) in &layouts.texts {
        if all || !text.current {
            digests.push(*digest).expect("uncapped digests");
        }
    }
    for digest in &digests {
        layouts.texts.get_mut(digest).expect("held text").schema = None;
    }
}

pub(super) fn schemas<'a>(layouts: &'a Layouts, digests: &List<Digest>, stored: bool) -> Option<List<&'a Schema>> {
    let mut schemas = List::with_capacity(digests.len());
    for digest in digests {
        let text = layouts.texts.get(digest)?;
        if stored && !text.stored {
            return None;
        }
        schemas.push(text.schema()?).ok()?;
    }
    Some(schemas)
}
