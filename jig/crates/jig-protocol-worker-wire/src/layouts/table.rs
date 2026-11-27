//! Kind numbers and bounds from the worker's text (jig/docs/design/protocol/channel/wires.md, 4).
use super::{plans::other_root, words};
use crate::{kinds, v22, wire};
use alloc::boxed::Box;
use skein_channel::{Direction, Kind};
use skein_codec::{
    layout::{Mode, Plan, Why},
    schema::{DeclIx, FieldIx, LimitLevel, Named, Schema, Shape, Type, TypeIx, VariantIx, limit_child, limit_name},
};
use skein_lib::List;

const UP: u16 = 0x0100;
pub(super) const DOWN: u16 = 0x0180;

pub(super) fn direction(mode: Mode) -> (u16, &'static str) {
    match mode {
        Mode::Read => (UP, "FromWorker"),
        Mode::Keep => (DOWN, "FromEngine"),
    }
}

pub(super) fn tag(base: u16, kind: u16) -> Option<u8> {
    u8::try_from(kind.checked_sub(base)?).ok()
}

pub(super) fn number(base: u16, tag: u8) -> u16 {
    base.checked_add(u16::from(tag)).expect("one byte after the frame kind base")
}

pub(super) fn current(limits: &v22::Limits) -> Result<List<Kind>, Box<[u8]>> {
    match kinds::current(limits) {
        Ok(table) => Ok(table),
        Err(crate::Overflow::Body) => Err(words::wire(wire::FAMILY, &[b": the engine's kind bounds do not fit"])),
    }
}

#[derive(Clone, Debug)]
pub(super) struct Frames {
    worker: Schema,
    from_worker: Variants,
    down_names: List<Box<str>>,
    read_tags: List<Option<u8>>,
    keep_tags: List<Option<u8>>,
    rules: List<Rule>,
}

#[derive(Clone, Copy, Debug)]
struct Rule {
    kind: u16,
    direction: Direction,
    record: DeclIx,
}

impl Frames {
    pub(super) fn new(
        engine: &Schema,
        worker: Schema,
        read: Option<&Plan>,
        keep_written: bool,
    ) -> Result<Self, Box<[u8]>> {
        let from_worker = Variants::peer(engine, &worker, Mode::Read)?;
        let from_engine = Variants::peer(engine, &worker, Mode::Keep)?;
        let own_engine = Variants::own(engine, Mode::Keep)?;
        let mut down_names = List::with_capacity(own_engine.count);
        for offset in 0..own_engine.count {
            let (name, _) = own_engine.at(engine, offset);
            down_names.push(Box::from(text(name))).expect("one name per engine tag");
        }
        let read_tags = if read.is_none() {
            from_worker.map(&worker, Variants::own(engine, Mode::Read)?, engine)
        } else {
            List::uncapped()
        };
        let keep_tags = if keep_written { own_engine.map(engine, from_engine, &worker) } else { List::uncapped() };
        let mut rules = List::uncapped();
        for (mode, variants, direction) in
            [(Mode::Read, from_worker, Direction::FromInitiator), (Mode::Keep, from_engine, Direction::FromResponder)]
        {
            let (base, declaration) = self::direction(mode);
            for offset in 0..variants.count {
                let tag = u8::try_from(offset).expect("a codec tag fits u8");
                if mode == Mode::Read {
                    let admitted = match read {
                        Some(plan) => plan.variant_tag(tag).is_ok(),
                        None => read_tags.get(offset).copied().flatten().is_some(),
                    };
                    if !admitted {
                        continue;
                    }
                }
                let (name, record) = variants.at(&worker, offset);
                let Some(record) = record else {
                    return Err(words::no_plan(
                        wire::FAMILY,
                        declaration,
                        mode,
                        Why::Payload,
                        declaration.as_bytes(),
                        Some(name),
                    ));
                };
                let kind = number(base, tag);
                for previous in &rules {
                    let previous: &Rule = previous;
                    if previous.kind == kind {
                        return Err(words::no_plan(
                            wire::FAMILY,
                            declaration,
                            mode,
                            Why::Reshaped,
                            declaration.as_bytes(),
                            Some(name),
                        ));
                    }
                }
                rules.push(Rule { kind, direction, record }).expect("bounded by the two enumerations");
            }
        }
        check_limits(&worker)?;
        Ok(Self { worker, from_worker, down_names, read_tags, keep_tags, rules })
    }

    pub(super) fn mapped(&self, mode: Mode, source: u8) -> Option<u8> {
        let tags = match mode {
            Mode::Read => &self.read_tags,
            Mode::Keep => &self.keep_tags,
        };
        tags.get(u32::from(source)).copied().flatten()
    }

    pub(super) fn table(&self, limits: &v22::Limits) -> List<Kind> {
        let values = limits.values();
        let bounds = self.worker.bounds_within(v22::LIMIT_NAMES, values.as_slice());
        let mut kinds = List::with_capacity(self.rules.len());
        for rule in &self.rules {
            // The parser checks every declaration at its ceilings; limits can only lower them.
            let largest = u32::try_from(bounds.at(rule.record).expect("one bound per declaration"))
                .expect("the parser checks the u32 bound");
            kinds.push(Kind { kind: rule.kind, direction: rule.direction, largest }).expect("one row per rule");
        }
        kinds
    }

    pub(super) fn source(&self, mode: Mode, tag: u8) -> &str {
        let offset = u32::from(tag);
        match mode {
            Mode::Read if offset < self.from_worker.count => text(self.from_worker.at(&self.worker, offset).0),
            Mode::Read => "",
            Mode::Keep => match self.down_names.get(offset) {
                Some(name) => name,
                None => "",
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Variants {
    first: VariantIx,
    count: u32,
}

impl Variants {
    fn own(schema: &Schema, mode: Mode) -> Result<Self, Box<[u8]>> {
        let (_, name) = direction(mode);
        Self::at_root(schema, schema.find(name.as_bytes()), mode)
    }

    fn peer(engine: &Schema, worker: &Schema, mode: Mode) -> Result<Self, Box<[u8]>> {
        let (_, name) = direction(mode);
        let root = engine.find(name.as_bytes()).expect("frames have been planned");
        Self::at_root(worker, other_root(engine, root, worker), mode)
    }

    fn at_root(schema: &Schema, root: Option<DeclIx>, mode: Mode) -> Result<Self, Box<[u8]>> {
        let (_, name) = direction(mode);
        if let Some(root) = root {
            match schema.declaration(root).shape {
                Shape::Enum { first, count } => return Ok(Self { first, count }),
                Shape::Record { .. } => {}
            }
        }
        Err(words::no_plan(wire::FAMILY, name, mode, Why::Reshaped, name.as_bytes(), None))
    }

    // A written frame still uses the channel's kind names and bounds. Match
    // names (including mutual `was` matches) without planning any payload.
    fn map(self, source: &Schema, target: Self, text: &Schema) -> List<Option<u8>> {
        let mut tags = List::with_capacity(self.count);
        for offset in 0..self.count {
            let mapped = if let Some(at) = target.preferred(text, self, source, offset)
                && self.preferred(source, target, text, at) == Some(offset)
            {
                Some(u8::try_from(at).expect("a codec tag fits u8"))
            } else {
                None
            };
            tags.push(mapped).expect("a slot per source tag");
        }
        tags
    }

    fn preferred(self, text: &Schema, source: Self, other: &Schema, offset: u32) -> Option<u32> {
        let (name, _) = source.at(other, offset);
        if let Some(at) = self.named(text, name) {
            return Some(at);
        }
        for &was in other.was(source.variant(offset)) {
            if let Some(at) = self.named(text, other.spelled(was)) {
                return Some(at);
            }
        }
        for at in 0..self.count {
            for &was in text.was(self.variant(at)) {
                if text.spelled(was) == name {
                    return Some(at);
                }
            }
        }
        None
    }

    #[expect(clippy::manual_find, reason = "step code uses no closures")]
    fn named(self, text: &Schema, name: &[u8]) -> Option<u32> {
        for at in 0..self.count {
            if self.at(text, at).0 == name {
                return Some(at);
            }
        }
        None
    }

    fn variant(self, offset: u32) -> Named {
        Named::Variant(VariantIx::new(add(self.first.index(), offset)))
    }

    fn at(self, schema: &Schema, tag: u32) -> (&[u8], Option<DeclIx>) {
        let variant = schema.variant(VariantIx::new(add(self.first.index(), tag)));
        (schema.spelled(variant.name), variant.record)
    }
}

fn check_limits(schema: &Schema) -> Result<(), Box<[u8]>> {
    for index in 0..schema.declarations() {
        let declaration = schema.declaration(DeclIx::new(index));
        let (first, count) = match declaration.shape {
            Shape::Record { first, count, .. } => (first, count),
            Shape::Enum { .. } => continue,
        };
        let name = schema.spelled(declaration.name);
        for at in first.index()..add(first.index(), count) {
            let field = schema.field(FieldIx::new(at));
            if let Some(limit) = missing(schema, field.ty, name, schema.spelled(field.name)) {
                return Err(words::wire(
                    wire::FAMILY,
                    &[b": the worker's text bounds ", name, b" by ", &limit, b", which the engine lacks"],
                ));
            }
        }
    }
    Ok(())
}

// bounds_within uses a text's ceiling for an unlisted name. An opening
// instead refuses that name, using the same spelling as the bound walk.
fn missing(schema: &Schema, mut at: TypeIx, record: &[u8], field: &[u8]) -> Option<Box<[u8]>> {
    let mut limit = match schema.ty(at) {
        Type::List { .. } | Type::Option(_) | Type::Text(_) | Type::Bytes { .. } | Type::Secret { .. } => {
            limit_name(record, field)
        }
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Named(_) => return None,
    };
    for _ in 0..schema.text().len() {
        match schema.ty(at) {
            Type::Option(item) => {
                limit = limit_child(&limit, LimitLevel::Some);
                at = item;
            }
            Type::List { item, .. } => {
                if !known(&limit) {
                    return Some(limit);
                }
                limit = limit_child(&limit, LimitLevel::Item);
                at = item;
            }
            Type::Text(_) | Type::Bytes { .. } | Type::Secret { .. } => {
                return if known(&limit) { None } else { Some(limit) };
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Named(_) => return None,
        }
    }
    unreachable!("a parsed type chain ends within its text")
}

fn known(limit: &[u8]) -> bool {
    for name in v22::LIMIT_NAMES {
        if name.as_bytes() == limit {
            return true;
        }
    }
    false
}

fn add(a: u32, b: u32) -> u32 {
    a.checked_add(b).expect("bounded schema indexes")
}

#[expect(clippy::disallowed_methods, reason = "the worker protocol validates its parsed schema identifiers")]
pub(super) fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).expect("schema identifiers are ASCII")
}
