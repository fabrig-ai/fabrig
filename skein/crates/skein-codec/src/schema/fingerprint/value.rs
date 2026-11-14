//! Value-type layouts, encoding 2 (data/codec/README.md, 9.4).

use super::{Fingerprint, hash_name};
use crate::schema::{DeclIx, FieldIx, Held, MAX_DECLARATIONS, Schema, Shape, Type, TypeIx, VariantIx, add};
use skein_lib::{List, Stack, sha256::Sha256};

/// The most resolved held records on a layout's path.
pub const MAX_HELD: u32 = 4;

/// A value type's declaration and, for a broad root, its leading variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Root {
    /// An index belonging to the schema.
    pub decl: DeclIx,
    /// An index belonging to the root's leading enumeration, exactly when broad.
    pub variant: Option<VariantIx>,
}

/// A root or its resolved holds cannot name a value layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutError {
    /// The root or a resolved held record does not belong to the given text.
    Root,
    /// Resolved holds nest past `MAX_HELD`.
    Deep,
}

impl Schema {
    /// Finds a root by this text's own declaration and variant names (README.md, 9.4).
    #[must_use]
    pub fn root(&self, decl: &[u8], variant: Option<&[u8]>) -> Option<Root> {
        let decl = self.find(decl)?;
        let variant = match variant {
            None => None,
            Some(name) => {
                let (first, count) = leading(self, decl)?;
                let mut found = None;
                for at in first.index()..add(first.index(), count) {
                    let at = VariantIx::new(at);
                    if self.spelled(self.variant(at).name) == name {
                        found = Some(at);
                        break;
                    }
                }
                Some(found?)
            }
        };
        Some(Root { decl, variant })
    }

    /// Hashes this value type, resolving holds among the supplied texts (README.md, 9.4).
    pub fn layout(&self, root: Root, held: &[&Schema]) -> Result<Fingerprint, LayoutError> {
        validate(self, root)?;
        let mut tables = List::uncapped();
        tables.push(Table::new(None, self)).expect("first text table");
        let mut stack = Stack::with_capacity(add(MAX_HELD, 1).checked_mul(MAX_DECLARATIONS).expect("bounded path"));
        stack.push(Frame::new(0, self, root, 0)).expect("root frame");
        // A part is visited once, or twice when it first opens a dependency.
        // Each declaration finishes once per text; a held cycle is refused within five paths.
        let mut work = u64::try_from(self.text().len()).expect("bounded text");
        for text in held {
            work = work.saturating_add(u64::try_from(text.text().len()).expect("bounded text"));
        }
        work = work.saturating_mul(u64::from(add(MAX_HELD, 1))).saturating_mul(3);
        for _ in 0..work {
            let mut frame = stack.pop().expect("an unfinished layout has a frame");
            let schema = text(self, held, tables.get(frame.text).expect("frame text").source);
            let Some(part) = part(schema, frame.root, frame.offset) else {
                let computed = Computed { fingerprint: Fingerprint(frame.hash.finish()), height: frame.height };
                if stack.is_empty() {
                    return Ok(computed.fingerprint);
                }
                *tables
                    .get_mut(frame.text)
                    .expect("frame text")
                    .declarations
                    .get_mut(frame.root.decl.index())
                    .expect("frame declaration") = Some(computed);
                continue;
            };
            let target = dependency(self, held, &mut tables, frame.text, part)?;
            let mut child = None;
            if let Some(target) = target {
                let depth = add(frame.depth, u32::from(target.held));
                if depth > MAX_HELD {
                    return Err(LayoutError::Deep);
                }
                child = *tables
                    .get(target.text)
                    .expect("target text")
                    .declarations
                    .get(target.decl.index())
                    .expect("target declaration");
                match child {
                    None => {
                        let next = text(self, held, tables.get(target.text).expect("target text").source);
                        let next = Frame::new(target.text, next, Root { decl: target.decl, variant: None }, depth);
                        stack.push(frame).expect("bounded declaration path");
                        stack.push(next).expect("bounded declaration path");
                        continue;
                    }
                    Some(computed) => {
                        if add(depth, computed.height) > MAX_HELD {
                            return Err(LayoutError::Deep);
                        }
                        frame.height = frame.height.max(add(computed.height, u32::from(target.held)));
                    }
                }
            }
            hash_part(schema, part, child, &mut frame.hash);
            frame.offset = add(frame.offset, 1);
            stack.push(frame).expect("resuming frame");
        }
        unreachable!("each reachable declaration and part visited within the text bound")
    }
}

fn leading(schema: &Schema, decl: DeclIx) -> Option<(VariantIx, u32)> {
    match schema.declaration(decl).shape {
        Shape::Enum { first, count } => Some((first, count)),
        Shape::Record { first, count, .. } => {
            if count == 0 {
                return None;
            }
            match schema.ty(schema.field(first).ty) {
                Type::Named(at) => match schema.declaration(at).shape {
                    Shape::Enum { first, count } => Some((first, count)),
                    Shape::Record { .. } => None,
                },
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Bool
                | Type::Duration
                | Type::Fixed(_)
                | Type::Secret { .. }
                | Type::Text(_)
                | Type::Bytes { .. }
                | Type::List { .. }
                | Type::Option(_) => None,
            }
        }
    }
}

fn validate(schema: &Schema, root: Root) -> Result<(), LayoutError> {
    if root.decl.index() >= schema.declarations() {
        return Err(LayoutError::Root);
    }
    match (leading(schema, root.decl), root.variant) {
        (None, None) => Ok(()),
        (Some((first, count)), Some(at)) if at.index() >= first.index() && at.index() < add(first.index(), count) => {
            Ok(())
        }
        (None | Some(_), Some(_)) | (Some(_), None) => Err(LayoutError::Root),
    }
}

#[derive(Clone, Copy, Debug)]
struct Computed {
    fingerprint: Fingerprint,
    // The longest held path below this declaration, also checked on memoized reads.
    height: u32,
}

#[derive(Debug)]
struct Table {
    source: Option<usize>,
    declarations: List<Option<Computed>>,
}

impl Table {
    fn new(source: Option<usize>, schema: &Schema) -> Self {
        let mut declarations = List::with_capacity(schema.declarations());
        for _ in 0..schema.declarations() {
            declarations.push(None).expect("one entry per declaration");
        }
        Self { source, declarations }
    }
}

fn text<'a>(primary: &'a Schema, held: &[&'a Schema], source: Option<usize>) -> &'a Schema {
    match source {
        None => primary,
        Some(at) => held.get(at).expect("held text index"),
    }
}

#[derive(Debug)]
struct Frame {
    text: u32,
    root: Root,
    offset: u32,
    hash: Sha256,
    height: u32,
    depth: u32,
}

impl Frame {
    fn new(text: u32, schema: &Schema, root: Root, depth: u32) -> Self {
        let mut hash = Sha256::new();
        hash.update(&[2]);
        match schema.declaration(root.decl).shape {
            Shape::Record { versioned, count, .. } => {
                hash.update(&[1, u8::from(versioned)]);
                hash.update(&count.to_be_bytes());
            }
            Shape::Enum { first, count } => match root.variant {
                None => {
                    hash.update(&[2]);
                    hash.update(&count.to_be_bytes());
                }
                Some(at) => restricted(first, at, &mut hash),
            },
        }
        Self { text, root, offset: 0, hash, height: 0, depth }
    }
}

#[derive(Clone, Copy)]
enum Part {
    Type(TypeIx),
    Variant(VariantIx),
    Restricted { first: VariantIx, at: VariantIx },
}

fn part(schema: &Schema, root: Root, offset: u32) -> Option<Part> {
    match schema.declaration(root.decl).shape {
        Shape::Record { first, count, .. } => {
            if offset >= count {
                return None;
            }
            if offset == 0
                && let Some(at) = root.variant
            {
                let (first, _) = leading(schema, root.decl).expect("validated broad root");
                return Some(Part::Restricted { first, at });
            }
            Some(Part::Type(schema.field(FieldIx::new(add(first.index(), offset))).ty))
        }
        Shape::Enum { first, count } => match root.variant {
            Some(at) => {
                if offset == 0 {
                    Some(Part::Variant(at))
                } else {
                    None
                }
            }
            None => {
                if offset < count {
                    Some(Part::Variant(VariantIx::new(add(first.index(), offset))))
                } else {
                    None
                }
            }
        },
    }
}

#[derive(Clone, Copy)]
struct Target {
    text: u32,
    decl: DeclIx,
    held: bool,
}

fn dependency(
    primary: &Schema,
    held: &[&Schema],
    tables: &mut List<Table>,
    from: u32,
    part: Part,
) -> Result<Option<Target>, LayoutError> {
    let schema = text(primary, held, tables.get(from).expect("part text").source);
    let clause = match part {
        Part::Variant(at) | Part::Restricted { at, .. } => {
            let Some(decl) = schema.variant(at).record else { return Ok(None) };
            return Ok(Some(Target { text: from, decl, held: false }));
        }
        Part::Type(mut at) => {
            let mut clause = Held::Opaque;
            for _ in 0..schema.types.len() {
                match schema.ty(at) {
                    Type::Named(decl) => return Ok(Some(Target { text: from, decl, held: false })),
                    Type::List { item, .. } | Type::Option(item) => {
                        at = item;
                        continue;
                    }
                    Type::Bytes { held, .. } | Type::Secret { held, .. } => clause = held,
                    Type::U8
                    | Type::U16
                    | Type::U32
                    | Type::U64
                    | Type::Bool
                    | Type::Duration
                    | Type::Fixed(_)
                    | Type::Text(_) => {}
                }
                break;
            }
            clause
        }
    };
    match clause {
        Held::Opaque => Ok(None),
        Held::Pieces(decl) => Ok(Some(Target { text: from, decl, held: false })),
        Held::Record { family, name } => {
            for (source, next) in held.iter().enumerate() {
                if next.spelled(next.family()) != schema.spelled(family) {
                    continue;
                }
                let decl = next.find(schema.spelled(name)).ok_or(LayoutError::Root)?;
                match next.declaration(decl).shape {
                    Shape::Record { .. } => {}
                    Shape::Enum { .. } => return Err(LayoutError::Root),
                }
                for (at, table) in tables.iter().enumerate() {
                    if table.source == Some(source) {
                        return Ok(Some(Target {
                            text: u32::try_from(at).expect("text table index"),
                            decl,
                            held: true,
                        }));
                    }
                }
                let at = tables.len();
                tables.push(Table::new(Some(source), next)).expect("one table per reached text");
                return Ok(Some(Target { text: at, decl, held: true }));
            }
            Ok(None)
        }
    }
}

fn restricted(first: VariantIx, at: VariantIx, hash: &mut Sha256) {
    hash.update(&[4]);
    hash.update(&at.index().checked_sub(first.index()).expect("variant belongs to leading enumeration").to_be_bytes());
}

fn hash_part(schema: &Schema, part: Part, child: Option<Computed>, hash: &mut Sha256) {
    match part {
        Part::Type(at) => hash_type(schema, at, child, hash),
        Part::Variant(at) => hash_variant(schema, at, child, hash),
        Part::Restricted { first, at } => {
            hash.update(&[15]);
            restricted(first, at, hash);
            hash_variant(schema, at, child, hash);
        }
    }
}

fn hash_variant(schema: &Schema, at: VariantIx, child: Option<Computed>, hash: &mut Sha256) {
    match schema.variant(at).record {
        None => hash.update(&[0]),
        Some(_) => {
            hash.update(&[1]);
            reference(child, hash);
        }
    }
}

fn reference(child: Option<Computed>, hash: &mut Sha256) {
    hash.update(&child.expect("dependency computed before its referrer").fingerprint.digest());
}

fn hash_held(schema: &Schema, held: Held, child: Option<Computed>, hash: &mut Sha256) {
    match held {
        Held::Opaque => hash.update(&[0]),
        Held::Record { family, name } => match child {
            Some(_) => {
                hash.update(&[3]);
                reference(child, hash);
            }
            None => {
                hash.update(&[1]);
                hash_name(schema.spelled(family), hash);
                hash_name(schema.spelled(name), hash);
            }
        },
        Held::Pieces(_) => {
            hash.update(&[2]);
            reference(child, hash);
        }
    }
}

fn hash_type(schema: &Schema, mut at: TypeIx, child: Option<Computed>, hash: &mut Sha256) {
    for _ in 0..schema.types.len() {
        match schema.ty(at) {
            Type::U8 => hash.update(&[1]),
            Type::U16 => hash.update(&[2]),
            Type::U32 => hash.update(&[3]),
            Type::U64 => hash.update(&[4]),
            Type::Bool => hash.update(&[5]),
            Type::Duration => hash.update(&[6]),
            Type::Fixed(bound) => {
                hash.update(&[7]);
                hash.update(&bound.to_be_bytes());
            }
            Type::Secret { bound, held: Held::Opaque } => {
                hash.update(&[8]);
                hash.update(&bound.to_be_bytes());
            }
            Type::Text(bound) => {
                hash.update(&[9]);
                hash.update(&bound.to_be_bytes());
            }
            Type::Secret { bound, held: held @ (Held::Record { .. } | Held::Pieces(_)) } => {
                hash.update(&[14]);
                hash.update(&bound.to_be_bytes());
                hash_held(schema, held, child, hash);
            }
            Type::Bytes { bound, held } => {
                hash.update(&[10]);
                hash.update(&bound.to_be_bytes());
                hash_held(schema, held, child, hash);
            }
            Type::List { bound, item } => {
                hash.update(&[11]);
                hash.update(&bound.to_be_bytes());
                at = item;
                continue;
            }
            Type::Option(value) => {
                hash.update(&[12]);
                at = value;
                continue;
            }
            Type::Named(_) => {
                hash.update(&[13]);
                reference(child, hash);
            }
        }
        break;
    }
}
