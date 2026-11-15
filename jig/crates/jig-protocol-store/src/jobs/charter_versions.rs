//! One root for charter versions (protocol/store/layouts.md, 2 and 6; jobs.md, 4).
use super::{JobAnswer, JobPage, JobRefusal, JobWrite};
use crate::settings_v3 as wire;
use alloc::boxed::Box;
use skein_codec::{
    schema::{self, DeclIx, FieldIx, Schema, Shape},
    value::{Node, Value},
};
use skein_lib::{List, Reader, Writer, length::usize_of};

pub(super) fn page(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    if page.rows.is_empty() {
        return Ok(JobAnswer { counts: Box::new([]), writes: Box::default(), through: None });
    }
    // The dispatch has no per-open state. Later pages name the same immutable
    // kept layout that this job's `from` fingerprint pins.
    let text = match &page.layout {
        Some(layout) => layout.text.as_ref(),
        None => crate::patches::kept::jig_store_settings::TEXTS.last().expect("settings kept text").as_bytes(),
    };
    let Ok(schema) = schema::parse(text) else {
        return Err(refusal(None, "invalid charter-version layout"));
    };
    let mut writes = List::uncapped();
    let mut bytes = 0_u64;
    let mut through = None;
    for row in &page.rows {
        let Some(value) = convert(&schema, &row.value) else {
            return Err(refusal(Some(row.key.clone()), "neither kept charter-version form"));
        };
        let write = JobWrite::Put { key: row.key.clone(), value };
        let total = bytes.saturating_add(write.bytes());
        if writes.len() == page.room.writes || total > page.room.bytes {
            if writes.is_empty() {
                return Err(refusal(Some(row.key.clone()), "room cannot hold one charter version"));
            }
            return Ok(JobAnswer { counts: Box::new([]), writes: writes.into_boxed(), through });
        }
        bytes = total;
        through = Some(row.key.clone());
        writes.push(write).expect("uncapped job writes");
    }
    Ok(JobAnswer { counts: Box::new([]), writes: writes.into_boxed(), through: None })
}
fn refusal(key: Option<Box<[u8]>>, why: &str) -> JobRefusal {
    JobRefusal { key, why: Box::from(why) }
}
fn convert(schema: &Schema, bytes: &[u8]) -> Option<Box<[u8]>> {
    // These records have no empty declarations: at most one structural node
    // per consuming node, so twice the bytes bounds their generic index.
    let nodes = u32::try_from(bytes.len()).ok()?.checked_mul(2)?;
    let mut value = Value::with_room(nodes);
    let plain = schema.find(b"CharterVersion")?;
    let overlaid = schema.find(b"OverlaidCharterVersion")?;
    let (root, decl, overlay) = match value.decode(schema, plain, headed(schema, bytes)?) {
        Ok(root) => (root, plain, None),
        Err(_) => {
            let root = value.decode(schema, overlaid, headed(schema, bytes)?).ok()?;
            let overlay = value.field(root, field(schema, overlaid, b"overlay")?).ok()?;
            (
                root,
                overlaid,
                Some(wire::Overlay::decode(&wire::CEILINGS, &mut Reader::new(span(&value, overlay)?)).ok()?),
            )
        }
    };
    let charter = value.field(root, field(schema, decl, b"charter")?).ok()?;
    let charter = match value.some(charter).ok()? {
        Some(node) => Some(wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(span(&value, node)?)).ok()?),
        None => None,
    };
    let row = wire::CharterVersion::new(&wire::CEILINGS, wire::CharterVersionParts { charter, overlay }).ok()?;
    let mut writer = Writer::new(usize_of(row.measure()));
    row.encode(&mut writer).ok()?;
    Some(writer.finish())
}
fn headed(schema: &Schema, bytes: &[u8]) -> Option<Box<[u8]>> {
    let mut writer = Writer::new(bytes.len());
    // A stored head is a layout id, not necessarily the kept family's version.
    writer.put(&schema.version().to_be_bytes()).ok()?;
    writer.put(bytes.get(2..)?).ok()?;
    Some(writer.finish())
}
fn field(schema: &Schema, decl: DeclIx, name: &[u8]) -> Option<FieldIx> {
    let (first, count) = match schema.declaration(decl).shape {
        Shape::Record { first, count, .. } => (first, count),
        Shape::Enum { .. } => return None,
    };
    for offset in 0..count {
        let field = FieldIx::new(first.index().checked_add(offset)?);
        if schema.spelled(schema.field(field).name) == name {
            return Some(field);
        }
    }
    None
}
fn span(value: &Value, node: Node) -> Option<&[u8]> {
    let span = value.span(node).ok()?;
    value.bytes().get(usize_of(span.start)..usize_of(span.end))
}
