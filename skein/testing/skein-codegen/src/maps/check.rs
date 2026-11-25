//! Maps refusals before emission (data/codec/maps.md, 6).

use super::{Direction, Item, Mapping, Maps, Mode, Shape, error, name};
use crate::{Declaration, Enumeration, Error, Record, Schema, Type};

fn ordered(
    schema: &Schema,
    line: usize,
    name: &str,
    at: Option<usize>,
    previous: &mut Option<usize>,
) -> Result<(), Error> {
    let at = at.ok_or_else(|| error(schema, line, format!("unknown name {name}")))?;
    if previous.is_some_and(|previous| at <= previous) {
        return Err(error(schema, line, format!("{name} repeated or out of schema order")));
    }
    *previous = Some(at);
    Ok(())
}

pub(super) fn record<'a>(schema: &'a Schema, name: &str) -> &'a Record {
    schema
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Record(record) if record.name == name => Some(record),
            Declaration::Record(_) | Declaration::Enum(_) => None,
        })
        .expect("validated schema payload record")
}

pub(super) fn item<'a>(items: &'a [Item], name: &str) -> Option<&'a Item> {
    items.iter().find(|item| item.name == name)
}

fn field_items(schema: &Schema, record: &Record, items: &[Item]) -> Result<(), Error> {
    let mut previous = None;
    for item in items {
        ordered(
            schema,
            item.line,
            &item.name,
            record.fields.iter().position(|field| field.name == item.name),
            &mut previous,
        )?;
    }
    Ok(())
}

fn value(
    schema: &Schema,
    maps: &Maps,
    ty: &Type,
    via: bool,
    line: usize,
    direction: Option<Direction>,
) -> Result<(), Error> {
    match ty {
        Type::Option(ty) | Type::List(_, ty) => value(schema, maps, ty, via, line, direction),
        Type::Named(name) => {
            if via {
                return Err(error(schema, line, format!("via on declaration {name}")));
            }
            let mapping = maps.mapping(name).ok_or_else(|| error(schema, line, format!("{name} has no maps line")))?;
            match &mapping.mode {
                Mode::Payload => Err(error(schema, line, format!("payload {name} has no domain type"))),
                Mode::Hand if maps.hand.is_none() => Err(error(schema, line, format!("{name} needs a by hand module"))),
                Mode::Mirror(_) => {
                    if let Some(direction) = direction
                        && maps.in_direction(name, direction).is_none()
                    {
                        return Err(error(schema, line, format!("{name} is not mapped to {}", direction.target())));
                    }
                    Ok(())
                }
                Mode::Stands { field, via, .. } => {
                    // Stands-for records have no functions: their field is translated at the caller.
                    if direction.is_some() {
                        let record = record(schema, name);
                        let held = record
                            .fields
                            .iter()
                            .find(|candidate| candidate.name == *field)
                            .expect("earlier stands-for field checked");
                        value(schema, maps, &held.ty, via.is_some(), line, direction)?;
                    }
                    Ok(())
                }
                Mode::Hand => Ok(()),
            }
        }
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Bytes(_)
        | Type::Secret(_)
        | Type::Text(_) => Ok(()),
    }
}

fn fields(
    schema: &Schema,
    maps: &Maps,
    record: &Record,
    mapping: &Mapping,
    overrides: &[Item],
    direction: Option<Direction>,
) -> Result<(), Error> {
    for field in &record.fields {
        let default = item(&mapping.items, &field.name);
        let override_item = item(overrides, &field.name);
        let line = override_item.or(default).map_or(mapping.line, |item| item.line);
        if field.pieces {
            return Err(error(
                schema,
                line,
                format!("{}.{} holds pieces and must stay by hand", record.name, field.name),
            ));
        }
        let via =
            override_item.and_then(|item| item.via.as_ref()).or_else(|| default.and_then(|item| item.via.as_ref()));
        let shape =
            override_item.filter(|item| item.shape != Shape::Same).or(default).map_or(Shape::Same, |item| item.shape);
        field_shape(schema, maps, &field.ty, shape, line)?;
        value(schema, maps, &field.ty, via.is_some(), line, direction)?;
    }
    Ok(())
}

fn field_shape(schema: &Schema, maps: &Maps, ty: &Type, shape: Shape, line: usize) -> Result<(), Error> {
    if let Type::Option(inner) = ty {
        return field_shape(schema, maps, inner, shape, line);
    }
    let valid = match shape {
        Shape::Same => true,
        Shape::Slice => matches!(ty, Type::List(_, _)),
        Shape::Boxed => match ty {
            Type::Named(name) => {
                maps.mapping(name).is_some_and(|mapping| matches!(mapping.mode, Mode::Mirror(_) | Mode::Hand))
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Bytes(_)
            | Type::Secret(_)
            | Type::Text(_)
            | Type::List(_, _)
            | Type::Option(_) => false,
        },
    };
    if valid {
        Ok(())
    } else {
        Err(error(schema, line, "slice requires a list; boxed requires a mapped declaration's domain type"))
    }
}

fn stands(
    schema: &Schema,
    maps: &Maps,
    mapping: &Mapping,
    record: &Record,
    field: &str,
    via: bool,
    shape: Shape,
) -> Result<(), Error> {
    if record.fields.len() != 1 {
        return Err(error(schema, mapping.line, format!("{} stands for requires exactly one field", mapping.name)));
    }
    let field = record
        .fields
        .iter()
        .find(|candidate| candidate.name == field)
        .ok_or_else(|| error(schema, mapping.line, format!("{} has no field {field}", mapping.name)))?;
    if field.pieces {
        return Err(error(schema, mapping.line, format!("{}.{} holds pieces", mapping.name, field.name)));
    }
    if shape == Shape::Slice && !matches!(field.ty, Type::List(_, _)) {
        return Err(error(
            schema,
            mapping.line,
            format!("{}.{}: stands for slice requires a list", mapping.name, field.name),
        ));
    }
    value(schema, maps, &field.ty, via, mapping.line, None)
}

fn enumeration(
    schema: &Schema,
    maps: &Maps,
    enumeration: &Enumeration,
    mapping: &Mapping,
    direction: Direction,
) -> Result<(), Error> {
    let mut previous = None;
    for item in &mapping.items {
        ordered(
            schema,
            item.line,
            &item.name,
            enumeration.variants.iter().position(|variant| variant.name == item.name),
            &mut previous,
        )?;
    }
    for variant in &enumeration.variants {
        let settings = item(&mapping.items, &variant.name);
        let line = settings.map_or(mapping.line, |item| item.line);
        let tuple = settings.is_some_and(|item| item.tuple);
        let block = settings.and_then(|item| item.block.as_deref());
        if settings.is_some_and(|item| item.shape == Shape::Boxed) && !tuple {
            return Err(error(schema, line, "boxed variant requires tuple"));
        }
        if let Some(payload) = &variant.record {
            if tuple {
                if block.is_some() {
                    return Err(error(schema, line, "a tuple uses its payload's map, not a field block"));
                }
                value(schema, maps, &Type::Named(payload.clone()), false, line, Some(direction))?;
            } else {
                let payload_mapping = maps
                    .inline_payload(payload, direction)
                    .ok_or_else(|| error(schema, line, format!("{payload} has no maps line")))?;
                if matches!(payload_mapping.mode, Mode::Hand) {
                    return Err(error(schema, line, format!("{payload} is by hand; use tuple")));
                }
                let record = record(schema, payload);
                field_items(schema, record, block.unwrap_or_default())?;
                fields(schema, maps, record, payload_mapping, block.unwrap_or_default(), Some(direction))?;
            }
        } else if tuple || block.is_some() {
            return Err(error(schema, line, format!("unit variant {} has tuple or block", variant.name)));
        }
    }
    Ok(())
}

fn declarations<'a>(schema: &'a Schema, maps: &Maps) -> Result<Vec<&'a Declaration>, Error> {
    let mut result = Vec::new();
    let mut previous = None;
    let mut seen = [false; 2];
    for mapping in &maps.declarations {
        let (at, declaration) = schema
            .declarations
            .iter()
            .enumerate()
            .find(|(_, declaration)| name(declaration) == mapping.name)
            .ok_or_else(|| error(schema, mapping.line, format!("unknown name {}", mapping.name)))?;
        if previous.is_some_and(|previous| at < previous) {
            return Err(error(schema, mapping.line, format!("{} out of schema order", mapping.name)));
        }
        if previous != Some(at) {
            seen = [false; 2];
            previous = Some(at);
        }
        if mapping.enumeration != matches!(declaration, Declaration::Enum(_)) {
            return Err(error(schema, mapping.line, format!("{} has a different declaration kind", mapping.name)));
        }
        for (direction, seen) in Direction::ALL.into_iter().zip(&mut seen) {
            if mapping.applies(direction) && core::mem::replace(seen, true) {
                return Err(error(
                    schema,
                    mapping.line,
                    format!("{} already has a line to {}", mapping.name, direction.target()),
                ));
            }
        }
        result.push(declaration);
    }
    Ok(result)
}

pub(super) fn check(schema: &Schema, maps: &Maps) -> Result<(), Error> {
    let declarations = declarations(schema, maps)?;
    for (mapping, declaration) in maps.declarations.iter().zip(declarations) {
        match declaration {
            Declaration::Record(record) => {
                field_items(schema, record, &mapping.items)?;
                match &mapping.mode {
                    Mode::Mirror(_) => {
                        for direction in Direction::ALL.into_iter().filter(|direction| mapping.applies(*direction)) {
                            fields(schema, maps, record, mapping, &[], Some(direction))?;
                        }
                    }
                    Mode::Payload => fields(schema, maps, record, mapping, &[], None)?,
                    Mode::Stands { field, via, shape } => {
                        stands(schema, maps, mapping, record, field, via.is_some(), *shape)?;
                    }
                    Mode::Hand => {}
                }
            }
            Declaration::Enum(variants) => {
                if matches!(mapping.mode, Mode::Mirror(_)) {
                    for direction in Direction::ALL.into_iter().filter(|direction| mapping.applies(*direction)) {
                        enumeration(schema, maps, variants, mapping, direction)?;
                    }
                }
            }
        }
    }
    Ok(())
}
