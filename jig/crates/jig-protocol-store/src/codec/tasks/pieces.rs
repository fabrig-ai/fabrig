//! Byte-summed pieces retain the domain's byte sum (codec.md, 3; protocol/store/records.md, 1).
use super::{Box, Reader, Writer, domain, wire};
use skein_lib::List;
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
        let field =
            wire::ResultField::new(limits, wire::ResultFieldParts { name: name.clone(), value: value.clone() })?;
        let mut item = Writer::new(skein_lib::length::usize_of(field.measure()));
        field.encode(&mut item).expect("measured field");
        items.push(item.finish()).expect("same field count");
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
        let wire::ResultFieldParts { name, value } =
            wire::ResultField::decode(limits, &mut Reader::new(&bytes))?.into_parts();
        admit(&mut used, &name, value.len(), limits.result_field_value, path)?;
        values.push(domain::ResultField { name, value }).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn rules(values: &[domain::FieldRule], limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(u32::try_from(values.len()).expect("bounded field count"));
    let mut used = 0;
    for field in values {
        let domain::FieldRule { name, max } = field;
        admit(&mut used, name, skein_lib::length::usize_of(*max), limits.field_rule_name, wire::Path::VerdictFields)?;
        let field = wire::FieldRule::new(limits, wire::FieldRuleParts { name: name.clone(), max: *max })?;
        let mut item = Writer::new(skein_lib::length::usize_of(field.measure()));
        field.encode(&mut item).expect("measured field");
        items.push(item.finish()).expect("same field count");
    }
    match crate::codec::pieces::join(items, limits.verdict_fields) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path: wire::Path::VerdictFields, reason }),
    }
}
pub(super) fn read_rules(bytes: Box<[u8]>, limits: &wire::Limits) -> Result<Box<[domain::FieldRule]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    let mut used = 0;
    while let Some(bytes) = split(&mut reader, wire::Path::VerdictFields)? {
        let wire::FieldRuleParts { name, max } =
            wire::FieldRule::decode(limits, &mut Reader::new(&bytes))?.into_parts();
        admit(&mut used, &name, skein_lib::length::usize_of(max), limits.field_rule_name, wire::Path::VerdictFields)?;
        values.push(domain::FieldRule { name, max }).expect("piece bounds item count");
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
        let value = super::value::parameter(value, limits)?;
        let mut item = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
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
        let value = super::value::read_parameter(value);
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
        let value = wire::Segment::new(limits, wire::SegmentParts { bytes: value.clone() })?;
        let mut item = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
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
        let value = value.into_parts().bytes;
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
        let value = wire::PartSegment::new(limits, wire::PartSegmentParts { bytes: value.clone() })?;
        let mut item = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
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
        let value = value.into_parts().bytes;
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
        let value = wire::AuthoritySegment::new(limits, wire::AuthoritySegmentParts { bytes: value.clone() })?;
        let mut item = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
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
        let value = value.into_parts().bytes;
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
        let value = wire::Grant::new(
            limits,
            wire::GrantParts {
                connector: value.connector,
                kind: value.kind,
                pattern: super::value::pattern(&value.pattern, limits)?,
            },
        )?;
        let mut item = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
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
        let value = {
            let wire::GrantParts { connector, kind, pattern } = value.into_parts();
            domain::Grant { connector, kind, pattern: super::value::read_pattern(pattern) }
        };
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
        let value = wire::ResourceScope::new(
            limits,
            wire::ResourceScopeParts {
                connector: value.connector,
                pattern: super::value::pattern(&value.pattern, limits)?,
            },
        )?;
        let mut item = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
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
        let value = {
            let wire::ResourceScopeParts { connector, pattern } = value.into_parts();
            domain::ResourceScope { connector, pattern: super::value::read_pattern(pattern) }
        };
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
        let mut item = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
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
            return Err(wire::Problem { path: wire::Path::CreationRecurring, reason: skein_codec::Reason::Bound });
        }
        let value = super::phase::member(value, limits, moved)?;
        let mut item = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
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
