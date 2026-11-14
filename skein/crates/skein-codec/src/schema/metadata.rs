//! Former-name scopes and defaults after layout resolution (data/codec/README.md, 7.2–7.3).

use super::{DeclIx, FieldIx, Named, Schema, SchemaError, Shape, Span, VariantIx, Wrong, add, constant};
use alloc::boxed::Box;
use skein_lib::{List, Set, bytes};

fn part(schema: &Schema, named: Named) -> (Span, u32) {
    match named {
        Named::Declaration(at) => {
            let part = schema.declaration(at);
            (part.name, part.line)
        }
        Named::Field(at) => {
            let part = schema.field(at);
            (part.name, part.line)
        }
        Named::Variant(at) => {
            let part = schema.variant(at);
            (part.name, part.line)
        }
    }
}

fn former(schema: &Schema, names: &mut Set<(u32, Box<[u8]>)>, scope: u32, target: Named) -> Result<(), SchemaError> {
    let (_, line) = part(schema, target);
    for &at in schema.was(target) {
        if !names.insert((scope, bytes::copy_of(schema.spelled(at)))).expect("names occupy disjoint text spans") {
            return Err(SchemaError { line, at, wrong: Wrong::Former });
        }
    }
    Ok(())
}

fn member(shape: Shape, at: u32) -> Named {
    match shape {
        Shape::Record { .. } => Named::Field(FieldIx::new(at)),
        Shape::Enum { .. } => Named::Variant(VariantIx::new(at)),
    }
}

fn parts(shape: Shape) -> (u32, u32) {
    match shape {
        Shape::Record { first, count, .. } => (first.index(), count),
        Shape::Enum { first, count } => (first.index(), count),
    }
}

fn names(schema: &Schema) -> Result<(), SchemaError> {
    let capacity = u32::try_from(schema.text.len()).expect("schema text ceiling");
    let mut names = Set::with_capacity(capacity);
    for at in 0..schema.declarations() {
        let declaration = schema.declaration(DeclIx::new(at));
        names.insert((0, bytes::copy_of(schema.spelled(declaration.name)))).expect("text bounds names");
        let (first, count) = parts(declaration.shape);
        for index in first..add(first, count) {
            let (name, _) = part(schema, member(declaration.shape, index));
            names.insert((add(at, 1), bytes::copy_of(schema.spelled(name)))).expect("text bounds names");
        }
    }
    for at in 0..schema.declarations() {
        former(schema, &mut names, 0, Named::Declaration(DeclIx::new(at)))?;
        let shape = schema.declaration(DeclIx::new(at)).shape;
        let (first, count) = parts(shape);
        for index in first..add(first, count) {
            former(schema, &mut names, add(at, 1), member(shape, index))?;
        }
    }
    Ok(())
}

// Grammar has already checked each default. This cursor only skips separators
// between `some` and its value; quoted text keeps its spaces and comment marks.
fn next(text: &[u8], offset: &mut usize) -> Option<Span> {
    let mut comment = false;
    for &byte in text.get(*offset..).expect("cursor in default") {
        if comment {
            comment = byte != b'\n';
        } else if byte == b'#' {
            comment = true;
        } else if !byte.is_ascii_whitespace() {
            break;
        }
        *offset = offset.checked_add(1).expect("text ceiling");
    }
    let start = *offset;
    let &first = text.get(start)?;
    *offset = offset.checked_add(1).expect("text ceiling");
    for &byte in text.get(*offset..).expect("cursor in default") {
        if first == b'"' {
            *offset = offset.checked_add(1).expect("text ceiling");
            if byte == b'"' {
                break;
            }
        } else {
            if !byte.is_ascii_alphanumeric() && byte != b'_' && byte != b'-' {
                break;
            }
            *offset = offset.checked_add(1).expect("text ceiling");
        }
    }
    Some(Span {
        start: u32::try_from(start).expect("text ceiling"),
        end: u32::try_from(*offset).expect("text ceiling"),
    })
}

pub(super) fn check(schema: &mut Schema) -> Result<(), SchemaError> {
    names(schema)?;
    let mut pool = core::mem::replace(&mut schema.pool, List::with_capacity(0));
    for at in 0..schema.fields.len() {
        let field = schema.field(FieldIx::new(at));
        let Some(span) = schema.default_span(FieldIx::new(at)) else {
            continue;
        };
        let refusal = SchemaError { line: field.line, at: span, wrong: Wrong::Default };
        let start = pool.len();
        let text = schema.spelled(span);
        let mut offset = 0;
        let mut ty = field.ty;
        let mut complete = false;
        for _ in 0..text.len() {
            let Some(word) = next(text, &mut offset) else {
                break;
            };
            match constant::encode(&mut pool, schema, ty, super::spelled(text, word)) {
                Ok(Some(inner)) => ty = inner,
                Ok(None) => {
                    complete = true;
                    break;
                }
                Err(()) => return Err(refusal),
            }
        }
        if !complete {
            return Err(refusal);
        }
        *schema.encodings.get_mut(at).expect("one encoding per field") = Some(Span { start, end: pool.len() });
    }
    schema.pool = pool;
    Ok(())
}
