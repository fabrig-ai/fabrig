//! Maps refusals before emission (data/codec/maps.md, 6).

use super::{Item, Mapping, Maps, Mode, Shape, error, name};
use crate::{Declaration, Error, Field, Record, Schema, Type};

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

fn value(schema: &Schema, maps: &Maps, ty: &Type, via: bool, line: usize) -> Result<(), Error> {
    match ty {
        Type::Option(ty) | Type::List(_, ty) => value(schema, maps, ty, via, line),
        Type::Named(name) => {
            if via {
                return Err(error(schema, line, format!("via on declaration {name}")));
            }
            let mapping = maps.mapping(name).ok_or_else(|| error(schema, line, format!("{name} has no maps line")))?;
            match mapping.mode {
                Mode::Payload => Err(error(schema, line, format!("payload {name} has no domain type"))),
                Mode::Hand if maps.hand.is_none() => Err(error(schema, line, format!("{name} needs a by hand module"))),
                Mode::Mirror(_) | Mode::Stands { .. } | Mode::Hand => Ok(()),
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

fn fields(schema: &Schema, maps: &Maps, record: &Record, mapping: &Mapping, overrides: &[Item]) -> Result<(), Error> {
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
        value(schema, maps, &field.ty, via.is_some(), line)?;
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
) -> Result<(), Error> {
    if record.fields.len() != 1 {
        return Err(error(schema, mapping.line, format!("{} stands for requires exactly one field", mapping.name)));
    }
    let field: &Field = record
        .fields
        .iter()
        .find(|candidate| candidate.name == field)
        .ok_or_else(|| error(schema, mapping.line, format!("{} has no field {field}", mapping.name)))?;
    if field.pieces {
        return Err(error(schema, mapping.line, format!("{}.{} holds pieces", mapping.name, field.name)));
    }
    value(schema, maps, &field.ty, via, mapping.line)
}

pub(super) fn check(schema: &Schema, maps: &Maps) -> Result<(), Error> {
    let mut previous = None;
    for mapping in &maps.declarations {
        let at = schema.declarations.iter().position(|declaration| name(declaration) == mapping.name);
        ordered(schema, mapping.line, &mapping.name, at, &mut previous)?;
        let declaration = schema.declarations.get(at.expect("ordered checked existence")).expect("declaration index");
        if mapping.enumeration != matches!(declaration, Declaration::Enum(_)) {
            return Err(error(schema, mapping.line, format!("{} has a different declaration kind", mapping.name)));
        }
        match declaration {
            Declaration::Record(record) => {
                field_items(schema, record, &mapping.items)?;
                match &mapping.mode {
                    Mode::Mirror(_) | Mode::Payload => fields(schema, maps, record, mapping, &[])?,
                    Mode::Stands { field, via } => stands(schema, maps, mapping, record, field, via.is_some())?,
                    Mode::Hand => {}
                }
            }
            Declaration::Enum(enumeration) => {
                if matches!(mapping.mode, Mode::Hand) {
                    continue;
                }
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
                            value(schema, maps, &Type::Named(payload.clone()), false, line)?;
                        } else {
                            let payload_mapping = maps
                                .mapping(payload)
                                .ok_or_else(|| error(schema, line, format!("{payload} has no maps line")))?;
                            if matches!(payload_mapping.mode, Mode::Hand) {
                                return Err(error(schema, line, format!("{payload} is by hand; use tuple")));
                            }
                            let record = record(schema, payload);
                            field_items(schema, record, block.unwrap_or_default())?;
                            fields(schema, maps, record, payload_mapping, block.unwrap_or_default())?;
                        }
                    } else if tuple || block.is_some() {
                        return Err(error(schema, line, format!("unit variant {} has tuple or block", variant.name)));
                    }
                }
            }
        }
    }
    Ok(())
}
