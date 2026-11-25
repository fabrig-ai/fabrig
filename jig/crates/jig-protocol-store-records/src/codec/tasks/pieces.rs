//! Byte-summed pieces retain the domain's byte sum (skein/docs/design/data/codec/README.md, 3; protocol/store/records.md, 1).
use super::{Box, Reader, domain, wire};
use crate::generated::tasks as mapped;
use skein_lib::List;
use wire::Record;
fn admit(used: &mut usize, name: &[u8], extra: usize, limit: u32, path: wire::Path) -> Result<(), wire::Problem> {
    *used = used.saturating_add(name.len()).saturating_add(extra);
    if name.is_empty() || *used > skein_lib::length::usize_of(limit) {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    }
    Ok(())
}
fn split(reader: &mut Reader<'_>, path: wire::Path) -> Result<Option<Box<[u8]>>, wire::Problem> {
    match skein_codec::piece::split(reader) {
        Ok(value) => Ok(value),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}

pub(super) fn fields(
    values: &[domain::ResultField],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(u32::try_from(values.len()).expect("bounded field count"));
    let mut used = 0;
    for field in values {
        let domain::ResultField { name, value } = field;
        admit(&mut used, name, value.len(), limits.result_field_value, path)?;
        let field = mapped::result_field_to_record(field);
        let item = field.encode(limits)?;
        items.push(item).expect("same field count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_fields(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[domain::ResultField]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    let mut used = 0;
    while let Some(bytes) = split(&mut reader, path)? {
        let wire::ResultField { name, value } = wire::ResultField::decode(limits, &mut Reader::new(&bytes))?;
        admit(&mut used, &name, value.len(), limits.result_field_value, path)?;
        let value = mapped::result_field_to_domain(wire::ResultField { name, value });
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn rules(values: &[domain::FieldRule], limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(u32::try_from(values.len()).expect("bounded field count"));
    let mut used = 0;
    for field in values {
        let domain::FieldRule { name, max } = field;
        admit(&mut used, name, skein_lib::length::usize_of(*max), limits.field_rule_name, wire::paths::VERDICT_FIELDS)?;
        let field = mapped::field_rule_to_record(field);
        let item = field.encode(limits)?;
        items.push(item).expect("same field count");
    }
    match crate::codec::pieces::join(items, limits.verdict_fields) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path: wire::paths::VERDICT_FIELDS, reason }),
    }
}
pub(super) fn read_rules(bytes: Box<[u8]>, limits: &wire::Limits) -> Result<Box<[domain::FieldRule]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    let mut used = 0;
    while let Some(bytes) = split(&mut reader, wire::paths::VERDICT_FIELDS)? {
        let wire::FieldRule { name, max } = wire::FieldRule::decode(limits, &mut Reader::new(&bytes))?;
        admit(&mut used, &name, skein_lib::length::usize_of(max), limits.field_rule_name, wire::paths::VERDICT_FIELDS)?;
        let value = mapped::field_rule_to_domain(wire::FieldRule { name, max });
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn parameters(
    values: &[domain::Parameter],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
) -> Result<Box<[u8]>, wire::Problem> {
    let Ok(count) = u32::try_from(values.len()) else {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    };
    let mut items = List::with_capacity(count);
    for value in values {
        let value = mapped::parameter_to_record(value);
        let item = value.encode(limits)?;
        items.push(item).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_parameters(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[domain::Parameter]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    while let Some(bytes) = split(&mut reader, path)? {
        let value = wire::Parameter::decode(limits, &mut Reader::new(&bytes))?;
        let value = mapped::parameter_to_domain(value);
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn segments(
    values: &[Box<[u8]>],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
) -> Result<Box<[u8]>, wire::Problem> {
    let Ok(count) = u32::try_from(values.len()) else {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    };
    let mut items = List::with_capacity(count);
    for value in values {
        let value = wire::Segment { bytes: value.clone() };
        let item = value.encode(limits)?;
        items.push(item).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_segments(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[Box<[u8]>]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    while let Some(bytes) = split(&mut reader, path)? {
        let value = wire::Segment::decode(limits, &mut Reader::new(&bytes))?;
        let value = value.bytes;
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn part_segments(
    values: &[Box<[u8]>],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
) -> Result<Box<[u8]>, wire::Problem> {
    let Ok(count) = u32::try_from(values.len()) else {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    };
    let mut items = List::with_capacity(count);
    for value in values {
        let value = wire::PartSegment { bytes: value.clone() };
        let item = value.encode(limits)?;
        items.push(item).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_part_segments(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[Box<[u8]>]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    while let Some(bytes) = split(&mut reader, path)? {
        let value = wire::PartSegment::decode(limits, &mut Reader::new(&bytes))?;
        let value = value.bytes;
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn authority_segments(
    values: &[Box<[u8]>],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
) -> Result<Box<[u8]>, wire::Problem> {
    let Ok(count) = u32::try_from(values.len()) else {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    };
    let mut items = List::with_capacity(count);
    for value in values {
        let value = wire::AuthoritySegment { bytes: value.clone() };
        let item = value.encode(limits)?;
        items.push(item).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_authority_segments(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[Box<[u8]>]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    while let Some(bytes) = split(&mut reader, path)? {
        let value = wire::AuthoritySegment::decode(limits, &mut Reader::new(&bytes))?;
        let value = value.bytes;
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn grants(
    values: &[domain::Grant],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
) -> Result<Box<[u8]>, wire::Problem> {
    let Ok(count) = u32::try_from(values.len()) else {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    };
    let mut items = List::with_capacity(count);
    for value in values {
        let value = mapped::grant_to_record(value)?;
        let item = value.encode(limits)?;
        items.push(item).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_grants(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[domain::Grant]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    while let Some(bytes) = split(&mut reader, path)? {
        let value = wire::Grant::decode(limits, &mut Reader::new(&bytes))?;
        let value = mapped::grant_to_domain(value);
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn scopes(
    values: &[domain::ResourceScope],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
) -> Result<Box<[u8]>, wire::Problem> {
    let Ok(count) = u32::try_from(values.len()) else {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    };
    let mut items = List::with_capacity(count);
    for value in values {
        let value = mapped::resource_scope_to_record(value)?;
        let item = value.encode(limits)?;
        items.push(item).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_scopes(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[domain::ResourceScope]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    while let Some(bytes) = split(&mut reader, path)? {
        let value = wire::ResourceScope::decode(limits, &mut Reader::new(&bytes))?;
        let value = mapped::resource_scope_to_domain(value);
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn creations(
    values: &[domain::New],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
    moved: Option<&[u64; 64]>,
) -> Result<Box<[u8]>, wire::Problem> {
    let Ok(count) = u32::try_from(values.len()) else {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    };
    let mut items = List::with_capacity(count);
    for value in values {
        let value = super::phase::new(value, limits, moved)?;
        let item = value.encode(limits)?;
        items.push(item).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_creations(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[domain::New]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    while let Some(bytes) = split(&mut reader, path)? {
        let value = wire::Creation::decode(limits, &mut Reader::new(&bytes))?;
        let value = super::phase::read_new(value, limits)?;
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn members(
    values: &[domain::New],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
    moved: Option<&[u64; 64]>,
) -> Result<Box<[u8]>, wire::Problem> {
    let Ok(count) = u32::try_from(values.len()) else {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    };
    let mut items = List::with_capacity(count);
    for value in values {
        if value.recurring.is_some() {
            return Err(wire::Problem { path: wire::paths::CREATION_RECURRING, reason: skein_codec::Reason::Bound });
        }
        let value = super::phase::member(value, limits, moved)?;
        let item = value.encode(limits)?;
        items.push(item).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_members(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[domain::New]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    while let Some(bytes) = split(&mut reader, path)? {
        let value = wire::Member::decode(limits, &mut Reader::new(&bytes))?;
        let value = super::phase::read_member(value, limits)?;
        values.push(value).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}
