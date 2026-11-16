//! Each text parsed, layout computed and text's held texts found once in planning,
//! then dropped when the plans are made (jig/docs/design/protocol/store/layouts.md, 7).
use super::{Digest, Layouts, Text};
use crate::{OpenRefusal, PatchSets};
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
type LayoutKey = (Digest, Box<[u8]>, Option<Box<[u8]>>, Box<[Digest]>);
#[derive(Debug)]
pub(super) struct Binary {
    names: Map<&'static str, (Digest, Digest)>,
    held: Map<(Digest, bool), List<Digest>>,
    layouts: Map<LayoutKey, Digest>,
}
impl Binary {
    pub(super) fn new(sets: &PatchSets, layouts: &mut Layouts) -> Result<Self, OpenRefusal> {
        let mut names = Map::uncapped();
        for set in sets.sets {
            for family in set.families {
                let Some(seed) = family.texts.last() else { continue };
                let mut digests = List::with_capacity(2);
                for text in [*seed, family.current] {
                    let refusal = OpenRefusal::Set { owner: set.owner, patch: 0 };
                    if text.len() > skein_lib::usize_of(crate::registry_v1::CEILINGS.text_text) {
                        return Err(refusal);
                    }
                    let digest = digest(text.as_bytes());
                    let current = text == family.current;
                    if let Some(held) = layouts.texts.get_mut(&digest) {
                        held.current |= current;
                    } else {
                        // Keep Set's syntax refusal even for families with no value types.
                        #[cfg(test)]
                        super::counting::parsed(digest);
                        let Ok(parsed) = schema::parse(text.as_bytes()) else { return Err(refusal) };
                        let schema = Some(parsed);
                        layouts
                            .texts
                            .insert(digest, Text { bytes: Box::from(text.as_bytes()), schema, stored: false, current })
                            .expect("uncapped texts");
                    }
                    digests.push(digest).expect("two texts");
                }
                names
                    .insert(family.name, (*digests.get(0).expect("seed"), *digests.get(1).expect("current")))
                    .expect("uncapped names");
            }
        }
        Ok(Self { names, held: Map::uncapped(), layouts: Map::uncapped() })
    }
    pub(super) fn text(&self, family: &str, current: bool) -> Option<Digest> {
        let &(seed, binary) = self.names.get(family)?;
        Some(if current { binary } else { seed })
    }
    pub(super) fn held(&mut self, layouts: &mut Layouts, text: Digest, current: bool) -> Option<List<Digest>> {
        if let Some(held) = self.held.get(&(text, current)) {
            return Some(held.clone());
        }
        #[cfg(test)]
        super::counting::held(text, current);
        let mut held = List::with_capacity(crate::registry_v1::CEILINGS.layout_held);
        let mut pending = List::uncapped();
        pending.push(text).ok()?;
        // Each family is entered once; its whole text describes the texts it may hold.
        for index in 0..=self.names.len() {
            let Some(digest) = pending.get(index) else { break };
            parse(layouts, *digest)?;
            let schema = layouts.texts.get(digest)?.schema.as_ref()?;
            for decl in 0..schema.declarations() {
                let Shape::Record { first, count, .. } = schema.declaration(DeclIx::new(decl)).shape else { continue };
                for offset in 0..count {
                    let field = schema.field(FieldIx::new(first.index().checked_add(offset)?));
                    let Some(name) = holds(schema, field.ty) else { continue };
                    let Some(&pair) = self.names.get(name) else { continue };
                    let digest = if current { pair.1 } else { pair.0 };
                    if digest == text || held.as_slice().contains(&digest) {
                        continue;
                    }
                    held.push(digest).ok()?;
                    pending.push(digest).ok()?;
                }
            }
        }
        self.held.insert((text, current), held.clone()).expect("uncapped held texts");
        Some(held)
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
        let schema = layouts.texts.get(&text)?.schema.as_ref()?;
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
pub(super) fn parse(layouts: &mut Layouts, digest: Digest) -> Option<()> {
    let text = layouts.texts.get_mut(&digest)?;
    if text.schema.is_none() {
        if self::digest(&text.bytes) != digest {
            return None;
        }
        #[cfg(test)]
        super::counting::parsed(digest);
        text.schema = Some(schema::parse(&text.bytes).ok()?);
    }
    Some(())
}

pub(super) fn row(layouts: &mut Layouts, key: &super::RowKey) -> Option<()> {
    let row = layouts.rows.get(key)?;
    let text = *row.value.text();
    let held = row.value.held().clone();
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
        schemas.push(text.schema.as_ref()?).ok()?;
    }
    Some(schemas)
}
