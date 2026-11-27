//! Reach worklists and retained level conversions (data/codec/layouts.md, 2).
use super::compare::{
    Comparison, compare as compare_types, held as compare_held, leaf as compare_leaf, payload as payload_matches,
    width as integer_width,
};
use super::{
    Absent, DropField, Field, Inner, Kind, Mode, Narrowed, Node, Plan, PlanRefusal, Read, ReadKind, Refused, Source,
    Underived, Variant, Why, add,
};
use crate::schema::{
    DeclIx, FieldIx, Held, MAX_DECLARATIONS, Named, Root, Schema, Shape, Span, Type, TypeIx, VariantIx,
};
use alloc::boxed::Box;
use skein_lib::{List, Map};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum At {
    Decl(DeclIx),
    Type(TypeIx),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Location {
    text: u32,
    at: At,
    variant: Option<VariantIx>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    old: Location,
    new: Location,
    root: bool,
}
#[derive(Clone, Copy, Debug)]
struct Place {
    decl: Span,
    part: Option<Span>,
    old: bool,
    field: Option<FieldSite>,
}
#[derive(Clone, Copy, Debug)]
struct FieldSite {
    decl: Span,
    part: Span,
    outer: Option<FieldIx>,
}
impl Place {
    fn new(decl: Span) -> Self {
        Self { decl, part: None, old: false, field: None }
    }
    fn error(self, why: Why) -> Underived {
        Underived { why, decl: self.decl, part: self.part, old: self.old }
    }
    fn outer_default(self, schema: &Schema) -> Option<&[u8]> {
        schema.default(self.field?.outer?)
    }
    fn nested(mut self) -> Self {
        if let Some(field) = &mut self.field {
            field.outer = None;
        }
        self
    }
}
#[derive(Debug)]
struct Pair {
    old: u32,
    new: u32,
    matching: List<Option<DeclIx>>,
}
#[derive(Clone, Copy)]
pub(super) enum Policy {
    Strict,
    Between(Mode),
}

struct Builder {
    plan: Plan,
    policy: Policy,
    // Worklists assign first-discovery indices; the maps only find them again.
    keys: List<Key>,
    by_key: Map<Key, u32>,
    places: List<Place>,
    read_keys: List<Location>,
    by_read_key: Map<Location, u32>,
    read_places: List<Place>,
    pairs: List<Pair>,
    #[cfg(test)]
    passes: u32,
}

fn text<'a>(texts: &'a [&Schema], at: u32) -> &'a Schema {
    texts.get(skein_lib::usize_of(at)).expect("text index in the worklist")
}
fn location(text: u32, at: At) -> Location {
    Location { text, at, variant: None }
}
fn parts(shape: Shape) -> (u32, u32) {
    match shape {
        Shape::Record { first, count, .. } => (first.index(), count),
        Shape::Enum { first, count } => (first.index(), count),
    }
}
#[derive(Clone, Copy)]
struct Names<'a> {
    schema: &'a Schema,
    shape: Option<Shape>,
}

impl<'a> Names<'a> {
    fn range(self) -> core::ops::Range<u32> {
        let (first, count) = match self.shape {
            Some(shape) => parts(shape),
            None => (0, self.schema.declarations()),
        };
        first..add(first, count)
    }

    fn member(self, at: u32) -> Named {
        match self.shape {
            Some(Shape::Record { .. }) => Named::Field(FieldIx::new(at)),
            Some(Shape::Enum { .. }) => Named::Variant(VariantIx::new(at)),
            None => Named::Declaration(DeclIx::new(at)),
        }
    }

    fn spelled(self, member: Named) -> &'a [u8] {
        let span = match member {
            Named::Declaration(at) => self.schema.declaration(at).name,
            Named::Field(at) => self.schema.field(at).name,
            Named::Variant(at) => self.schema.variant(at).name,
        };
        self.schema.spelled(span)
    }

    #[expect(clippy::manual_map, reason = "step code enumerates options without callbacks")]
    fn current(self, name: &[u8]) -> Option<Named> {
        if self.shape.is_none() {
            return match self.schema.find(name) {
                Some(at) => Some(Named::Declaration(at)),
                None => None,
            };
        }
        for at in self.range() {
            let part = self.member(at);
            if self.spelled(part) == name {
                return Some(part);
            }
        }
        None
    }

    fn preferred(self, other: Self, part: Named, policy: Policy) -> Option<Named> {
        let name = other.spelled(part);
        if let Some(found) = self.current(name) {
            return Some(found);
        }
        for &was in other.schema.was(part) {
            if let Some(found) = self.current(other.schema.spelled(was)) {
                return Some(found);
            }
        }
        if let Policy::Between(_) = policy {
            for at in self.range() {
                let member = self.member(at);
                for &was in self.schema.was(member) {
                    if self.schema.spelled(was) == name {
                        return Some(member);
                    }
                }
            }
        }
        None
    }

    fn matched(self, other: Self, part: Named, policy: Policy) -> Option<Named> {
        let found = self.preferred(other, part, policy)?;
        // A confirmed rename works in both directions. If several live names
        // occur in a history, accept only the pairing both directions choose.
        if let Policy::Between(_) = policy
            && self.spelled(found) != other.spelled(part)
            && other.preferred(self, found, policy) != Some(part)
        {
            return None;
        }
        Some(found)
    }

    fn variant_for(self, old: Self, source: VariantIx, policy: Policy) -> Option<VariantIx> {
        for index in self.range() {
            let dest = VariantIx::new(index);
            if old.matched(self, Named::Variant(dest), policy) == Some(Named::Variant(source)) {
                return Some(dest);
            }
        }
        None
    }
}

fn tombstone_named(schema: &Schema, first: FieldIx, count: u32, name: &[u8]) -> Option<FieldIx> {
    for index in first.index()..add(first.index(), count) {
        let at = FieldIx::new(index);
        if schema.spelled(schema.field(at).name) == name {
            return Some(at);
        }
    }
    None
}

fn matched_tombstone(schema: &Schema, decl: DeclIx, live: &Schema, field: FieldIx) -> Option<FieldIx> {
    let (first, count) = schema.tombstones(decl);
    if count == 0 {
        return None;
    }
    let name = live.spelled(live.field(field).name);
    if let Some(found) = tombstone_named(schema, first, count, name) {
        return Some(found);
    }
    // Match the live field and tombstone the same way on the way out and back.
    for &was in live.was(Named::Field(field)) {
        if let Some(found) = tombstone_named(schema, first, count, live.spelled(was)) {
            return Some(found);
        }
    }
    for index in first.index()..add(first.index(), count) {
        let at = FieldIx::new(index);
        for &was in schema.was(Named::Field(at)) {
            if schema.spelled(was) == name {
                return Some(at);
            }
        }
    }
    None
}

fn default_tag(old: &Schema, a: DeclIx, new: &Schema, b: DeclIx, tag: u8, policy: Policy) -> Option<u8> {
    let old_shape = old.declaration(a).shape;
    let new_shape = new.declaration(b).shape;
    let (Shape::Enum { first: af, .. }, Shape::Enum { first: bf, .. }) = (old_shape, new_shape) else {
        return None;
    };
    let old_names = Names { schema: old, shape: Some(old_shape) };
    let new_names = Names { schema: new, shape: Some(new_shape) };
    let source = VariantIx::new(add(af.index(), u32::from(tag)));
    let dest = new_names.variant_for(old_names, source, policy)?;
    if new.variant(dest).record.is_some() {
        return None;
    }
    let offset = dest.index().checked_sub(bf.index()).expect("variant of new enum");
    Some(u8::try_from(offset).expect("schema tag fits"))
}

fn restored_default(
    live: &Schema,
    field: FieldIx,
    other: &Schema,
    tombstone: Option<FieldIx>,
    policy: Policy,
    place: Place,
) -> Result<Option<Box<[u8]>>, Underived> {
    // A written default wins; implicit none is the fallback without a tombstone.
    let at = match tombstone {
        Some(at) if live.default_span(field).is_none() => at,
        Some(_) | None => {
            return match live.default(field) {
                Some(bytes) => Ok(Some(Box::from(bytes))),
                None => Ok(None),
            };
        }
    };
    let mut bytes: Box<[u8]> = Box::from(other.default(at).expect("tombstones have defaults"));
    let mut a = other.field(at).ty;
    let mut b = live.field(field).ty;
    // The types compare equal. Defaults can reach an enum through present
    // options, but lists and pieces are empty and no default holds a record.
    for byte in &mut *bytes {
        match other.ty(a) {
            Type::Option(inner) => {
                if *byte == 0 {
                    break;
                }
                a = inner;
                b = option(live.ty(b)).expect("equal option types");
            }
            Type::Named(decl) => {
                let target = declaration(live.ty(b)).expect("equal named types");
                *byte = default_tag(other, decl, live, target, *byte, policy).ok_or(place.error(Why::Retyped))?;
                break;
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::Secret { .. }
            | Type::Bytes { .. }
            | Type::List { .. } => break,
        }
    }
    Ok(Some(bytes))
}

impl Builder {
    fn new(policy: Policy, head: Option<u16>, old_head: bool, tag: Option<u8>) -> Self {
        Self {
            plan: Plan {
                nodes: List::uncapped(),
                reads: List::uncapped(),
                refusals: List::with_capacity(MAX_DECLARATIONS),
                root: 0,
                depth: 0,
                fields: 0,
                head,
                old_head,
                tag,
            },
            policy,
            keys: List::uncapped(),
            by_key: Map::uncapped(),
            places: List::uncapped(),
            read_keys: List::uncapped(),
            by_read_key: Map::uncapped(),
            read_places: List::uncapped(),
            pairs: List::uncapped(),
            #[cfg(test)]
            passes: 0,
        }
    }
    fn complete(&mut self, olds: &[&Schema], news: &[&Schema]) -> Result<(), Underived> {
        for at in 0..u32::MAX {
            if at >= self.keys.len() {
                break;
            }
            self.build(olds, news, at)?;
        }
        for at in 0..u32::MAX {
            if at >= self.read_keys.len() {
                break;
            }
            self.build_read(olds, at);
        }
        finish(self)
    }
    fn pair(&mut self, olds: &[&Schema], news: &[&Schema], a: u32, b: u32) -> u32 {
        for at in 0..self.pairs.len() {
            let pair = self.pairs.get(at).expect("pair index");
            if pair.old == a && pair.new == b {
                return at;
            }
        }
        let old = text(olds, a);
        let new = text(news, b);
        let mut matching = List::with_capacity(new.declarations());
        let old_names = Names { schema: old, shape: None };
        let new_names = Names { schema: new, shape: None };
        for at in 0..new.declarations() {
            let part = Named::Declaration(DeclIx::new(at));
            let found = match old_names.matched(new_names, part, self.policy) {
                Some(Named::Declaration(at)) => Some(at),
                None => None,
                Some(Named::Field(_) | Named::Variant(_)) => unreachable!("declaration matching"),
            };
            matching.push(found).expect("one declaration match slot per text declaration");
        }
        let at = self.pairs.len();
        self.pairs.push(Pair { old: a, new: b, matching }).expect("text pairs bounded by the reach");
        at
    }
    fn request_read(&mut self, key: Location, place: Place) -> u32 {
        if let Some(&at) = self.by_read_key.get(&key) {
            return at;
        }
        let at = self.read_keys.len();
        self.read_keys.push(key).expect("read nodes bounded by reached texts");
        self.by_read_key.insert(key, at).expect("one lookup entry per reader");
        self.read_places.push(place).expect("one place per reader");
        self.plan.reads.push(Read { kind: ReadKind::Pending, minimum: 0, depth: 0 }).expect("one reader per key");
        at
    }
    fn request(&mut self, key: Key, place: Place) -> u32 {
        if let Some(&at) = self.by_key.get(&key) {
            return at;
        }
        let at = self.keys.len();
        let read = self.request_read(key.old, place);
        self.keys.push(key).expect("conversion nodes bounded by reached texts");
        self.by_key.insert(key, at).expect("one lookup entry per conversion");
        self.places.push(place).expect("one place per conversion");
        self.plan
            .nodes
            .push(Node { read, kind: Kind::Pending, keeps: false, empty: false, depth: 0 })
            .expect("one node per key");
        at
    }
    fn child(&mut self, key: Key, a: At, b: At, place: Place) -> u32 {
        self.request(
            Key { old: location(key.old.text, a), new: location(key.new.text, b), root: false },
            place.nested(),
        )
    }
    fn build(&mut self, olds: &[&Schema], news: &[&Schema], at: u32) -> Result<(), Underived> {
        let key = *self.keys.get(at).expect("conversion key");
        let place = *self.places.get(at).expect("conversion place");
        let pair = self.pair(olds, news, key.old.text, key.new.text);
        let kind = match (key.old.at, key.new.at) {
            (At::Decl(a), At::Decl(b)) => self.declaration(olds, news, pair, key, a, b)?,
            (At::Type(a), At::Type(b)) => self.ty(olds, news, pair, key, a, b, place)?,
            (At::Decl(_), At::Type(_)) | (At::Type(_), At::Decl(_)) => return Err(place.error(Why::Retyped)),
        };
        let slot = self.plan.nodes.get_mut(at).expect("conversion slot");
        slot.kind = kind;
        slot.empty = empty(text(news, key.new.text), key.new.at);
        Ok(())
    }
    #[expect(clippy::disallowed_methods, reason = "schema parser has validated these ASCII names")]
    fn refusal(&mut self, old: &Schema, decl: Span, part: Span, why: Refused) -> Result<u16, Underived> {
        let index = u16::try_from(self.plan.refusals.len()).expect("declaration bound fits u16");
        let names = PlanRefusal {
            why,
            decl: Box::from(core::str::from_utf8(old.spelled(decl)).expect("schema names are ASCII")),
            part: Box::from(core::str::from_utf8(old.spelled(part)).expect("schema names are ASCII")),
        };
        if self.plan.refusals.push(names).is_err() {
            return Err(Underived { why: Why::Deep, decl, part: Some(part), old: true });
        }
        Ok(index)
    }
    fn field_refusal(&mut self, old: &Schema, place: Place, why: Refused) -> Result<u16, Underived> {
        let field = place.field.expect("type conversion reached through a field");
        self.refusal(old, field.decl, field.part, why)
    }
    fn narrowed(&mut self, old: &Schema, place: Place, a: u32, b: u32) -> Result<Option<Narrowed>, Underived> {
        if b >= a {
            return Ok(None);
        }
        let refusal = self.field_refusal(old, place, Refused::Narrowed)?;
        Ok(Some(Narrowed { bound: b, refusal }))
    }
    fn check_drops(
        &mut self,
        old: &Schema,
        new: &Schema,
        pair: u32,
        a: DeclIx,
        b: DeclIx,
        used: &List<bool>,
    ) -> Result<List<DropField>, Underived> {
        let keep = match self.policy {
            Policy::Between(Mode::Keep) => true,
            Policy::Strict | Policy::Between(Mode::Read) => false,
        };
        let mut dropped = List::with_capacity(if keep { used.len() } else { 0 });
        let record = old.declaration(a);
        let (first, _) = parts(record.shape);
        for offset in 0..used.len() {
            if *used.get(offset).expect("one slot per source field") {
                continue;
            }
            let field = FieldIx::new(add(first, offset));
            let live = old.field(field);
            let part = Place { part: Some(live.name), old: true, ..Place::new(record.name) };
            let tombstone = matched_tombstone(new, b, old, field);
            let matching = &self.pairs.get(pair).expect("text pair").matching;
            if let Some(at) = tombstone
                && compare_types(matching, old, new, live.ty, new.field(at).ty) != Comparison::Equal
            {
                return Err(part.error(Why::Retyped));
            }
            if !keep || old.droppable(field) {
                continue;
            }
            let default = restored_default(old, field, new, tombstone, Policy::Between(Mode::Read), part)?;
            if let Some(bytes) = &default
                && bytes.is_empty()
                && empty(old, At::Type(live.ty))
            {
                continue; // Every value of this field is already at its default.
            }
            let refusal = self.refusal(old, record.name, live.name, Refused::Field)?;
            dropped.push(DropField { offset, default, refusal }).expect("one check per dropped field");
        }
        Ok(dropped)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "pairs fields and variants in discovery order (data/codec/layouts.md, 2)"
    )]
    fn declaration(
        &mut self,
        olds: &[&Schema],
        news: &[&Schema],
        pair: u32,
        key: Key,
        a: DeclIx,
        b: DeclIx,
    ) -> Result<Kind, Underived> {
        let old = text(olds, key.old.text);
        let new = text(news, key.new.text);
        let place = Place::new(new.declaration(b).name);
        let old_decl = old.declaration(a).name;
        if self.pairs.get(pair).expect("text pair").matching.get(b.index()).copied().flatten() != Some(a) {
            return Err(place.error(Why::Retyped));
        }
        let old_shape = old.declaration(a).shape;
        let new_shape = new.declaration(b).shape;
        let old_names = Names { schema: old, shape: Some(old_shape) };
        let new_names = Names { schema: new, shape: Some(new_shape) };
        match (old_shape, new_shape) {
            (
                Shape::Record { first: af, count: ac, versioned: av },
                Shape::Record { first: bf, count: bc, versioned: bv },
            ) => {
                let mut fields = List::with_capacity(bc);
                let track_used = match self.policy {
                    Policy::Between(Mode::Keep) => true,
                    Policy::Strict | Policy::Between(Mode::Read) => new.tombstones(b).1 != 0,
                };
                let mut used = List::with_capacity(if track_used { ac } else { 0 });
                if track_used {
                    for _ in 0..ac {
                        used.push(false).expect("one slot per old field");
                    }
                }
                let mut in_place = ac == bc && av == bv && (key.root || !av || old.version() == new.version());
                for offset in 0..bc {
                    let field = FieldIx::new(add(bf.index(), offset));
                    let part = Place { part: Some(new.field(field).name), ..place };
                    let source = match old_names.matched(new_names, Named::Field(field), self.policy) {
                        Some(Named::Field(at)) => Some(at),
                        None => None,
                        Some(Named::Declaration(_) | Named::Variant(_)) => unreachable!("field matching"),
                    };
                    let field = if let Some(source) = source {
                        let old_offset = source.index().checked_sub(af.index()).expect("field in matched record");
                        in_place &= old_offset == offset;
                        if let Some(used) = used.get_mut(old_offset) {
                            *used = true;
                        }
                        let part = Place {
                            field: Some(FieldSite { decl: old_decl, part: old.field(source).name, outer: Some(field) }),
                            ..part
                        };
                        let mut child = Key {
                            old: location(key.old.text, At::Type(old.field(source).ty)),
                            new: location(key.new.text, At::Type(new.field(field).ty)),
                            root: false,
                        };
                        if offset == 0 && old_offset == 0 {
                            child.old.variant = key.old.variant;
                            child.new.variant = key.new.variant;
                        }
                        let node = self.request(child, part);
                        Field { source: Source::Old(old_offset), node: Some(node) }
                    } else {
                        in_place = false;
                        let tombstone = matched_tombstone(old, a, new, field);
                        let matching = &self.pairs.get(pair).expect("text pair").matching;
                        if let Some(at) = tombstone
                            && compare_types(matching, old, new, old.field(at).ty, new.field(field).ty)
                                != Comparison::Equal
                        {
                            return Err(part.error(Why::Retyped));
                        }
                        let bytes = restored_default(new, field, old, tombstone, self.policy, part)?
                            .ok_or(part.error(Why::NoDefault))?;
                        Field { source: Source::Default(bytes), node: None }
                    };
                    fields.push(field).expect("one source per new field");
                }
                let dropped = self.check_drops(old, new, pair, a, b, &used)?;
                Ok(Kind::Record {
                    old_head: av,
                    head: if bv { Some(new.version()) } else { None },
                    fields,
                    dropped,
                    in_place,
                })
            }
            (Shape::Enum { first: af, count: ac }, Shape::Enum { first: bf, .. }) => {
                let mut variants = List::with_capacity(ac);
                for offset in 0..ac {
                    let source = VariantIx::new(add(af.index(), offset));
                    if let Some(only) = key.old.variant
                        && only != source
                    {
                        variants.push(None).expect("one slot per old tag");
                        continue;
                    }
                    let Some(dest) = new_names.variant_for(old_names, source, self.policy) else {
                        let part = old.variant(source).name;
                        if let Policy::Strict = self.policy {
                            return Err(Underived {
                                why: Why::VariantDropped,
                                decl: old_decl,
                                part: Some(part),
                                old: true,
                            });
                        }
                        let refusal = self.refusal(old, old_decl, part, Refused::Variant)?;
                        variants.push(Some(Variant::Refuse(refusal))).expect("one slot per old tag");
                        continue;
                    };
                    let part = Place { part: Some(new.variant(dest).name), ..place };
                    if key.new.variant.is_some() && key.new.variant != Some(dest) {
                        return Err(part.error(Why::Root));
                    }
                    let ar = old.variant(source).record;
                    let br = new.variant(dest).record;
                    if !payload_matches(&self.pairs.get(pair).expect("text pair").matching, ar, br) {
                        return Err(part.error(Why::Payload));
                    }
                    let payload = match (ar, br) {
                        (Some(a), Some(b)) => Some(self.child(key, At::Decl(a), At::Decl(b), part)),
                        (None, None) => None,
                        (Some(_), None) | (None, Some(_)) => unreachable!("payload judgment"),
                    };
                    let tag = dest.index().checked_sub(bf.index()).expect("variant of new enum");
                    variants
                        .push(Some(Variant::Kept {
                            tag: u8::try_from(tag).expect("schema tag fits"),
                            payload,
                            in_place: tag == offset,
                        }))
                        .expect("one slot per old tag");
                }
                Ok(Kind::Enum(variants))
            }
            (Shape::Enum { .. }, Shape::Record { .. }) | (Shape::Record { .. }, Shape::Enum { .. }) => {
                Err(place.error(Why::Reshaped))
            }
        }
    }
    #[expect(clippy::too_many_arguments, reason = "one type worklist pair and its refusal location")]
    fn ty(
        &mut self,
        olds: &[&Schema],
        news: &[&Schema],
        pair: u32,
        key: Key,
        a: TypeIx,
        b: TypeIx,
        place: Place,
    ) -> Result<Kind, Underived> {
        let old = text(olds, key.old.text);
        let new = text(news, key.new.text);
        let matching = &self.pairs.get(pair).expect("text pair").matching;
        // The store judges the entire field's type here, before queued declarations.
        if let Policy::Strict = self.policy
            && compare_types(matching, old, new, a, b) == Comparison::Retype
        {
            return Err(place.error(Why::Retyped));
        }
        let av = old.ty(a);
        let bv = new.ty(b);
        if let Some(inner) = option(bv) {
            if let Some(a) = option(av) {
                return Ok(Kind::Option(self.child(key, At::Type(a), At::Type(inner), place)));
            }
            return Ok(Kind::Wrap(self.child(key, At::Type(a), At::Type(inner), place)));
        }
        if let Some(inner) = option(av) {
            let default = match self.policy {
                Policy::Between(Mode::Read) => place.outer_default(new),
                Policy::Between(Mode::Keep) | Policy::Strict => None,
            };
            let absent = match default {
                Some(bytes) => Absent::Default(Box::from(bytes)),
                None => Absent::Refuse(self.field_refusal(old, place, Refused::Absent)?),
            };
            return Ok(Kind::Unwrap { child: self.child(key, At::Type(inner), At::Type(b), place), absent });
        }
        if let Some((bound, item)) = list(bv) {
            let (old_bound, a) = list(av).ok_or(place.error(Why::Retyped))?;
            let narrowed = self.narrowed(old, place, old_bound, bound)?;
            return Ok(Kind::List {
                bound: old_bound,
                item: self.child(key, At::Type(a), At::Type(item), place),
                narrowed,
            });
        }
        if let Some(old_width) = integer_width(av)
            && let Some(new_width) = integer_width(bv)
        {
            if old_width == new_width {
                return Ok(Kind::Copy);
            }
            let refusal =
                if new_width < old_width { Some(self.field_refusal(old, place, Refused::Narrowed)?) } else { None };
            return Ok(Kind::Integer { old: old_width, new: new_width, refusal });
        }
        // A tighter bound changes the pass, not which leaf types can be paired.
        let bound = content_bound(av).unwrap_or(0);
        if let Policy::Between(_) = self.policy
            && compare_leaf(matching, old, new, av, widen_content(bv, bound)) == Comparison::Retype
        {
            return Err(place.error(Why::Retyped));
        }
        match bv {
            Type::Named(b) => {
                let a = declaration(av).expect("compared named type");
                let node = self.request(
                    Key {
                        old: Location { at: At::Decl(a), ..key.old },
                        new: Location { at: At::Decl(b), ..key.new },
                        root: false,
                    },
                    place,
                );
                Ok(Kind::Named(node)) // A named level has no wire of its own.
            }
            Type::Bytes { bound, held } | Type::Secret { bound, held } => {
                self.content(olds, news, pair, key, av, held, bound, place)
            }
            Type::Text(to) => match self.narrowed(old, place, bound, to)? {
                Some(narrowed) => Ok(Kind::Content { bound: to, inner: None, narrowed: Some(narrowed) }),
                None => Ok(Kind::Copy),
            },
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration | Type::Fixed(_) => {
                Ok(Kind::Copy)
            }
            Type::List { .. } | Type::Option(_) => unreachable!("wrapper handled before leaf comparison"),
        }
    }
    #[expect(clippy::too_many_arguments, reason = "one terminal bytes conversion and its held-text context")]
    fn content(
        &mut self,
        olds: &[&Schema],
        news: &[&Schema],
        pair: u32,
        key: Key,
        av: Type,
        bh: Held,
        bound: u32,
        place: Place,
    ) -> Result<Kind, Underived> {
        let ah = match av {
            Type::Bytes { held, .. } | Type::Secret { held, .. } => held,
            Type::Text(_) => Held::Opaque,
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::List { .. }
            | Type::Option(_)
            | Type::Named(_) => unreachable!("compared content"),
        };
        let old = text(olds, key.old.text);
        let new = text(news, key.new.text);
        if compare_held(&self.pairs.get(pair).expect("text pair").matching, old, new, ah, bh) == Comparison::Retype {
            return Err(place.error(Why::Retyped));
        }
        let narrowed = self.narrowed(old, place, content_bound(av).expect("compared content bound"), bound)?;
        if let Some((af, an)) = held_record(ah)
            && let Some((bf, bn)) = held_record(bh)
        {
            let a = family(olds, old.spelled(af));
            let b = family(news, new.spelled(bf));
            match (a, b) {
                (None, None) => return Ok(opaque(bound, narrowed)),
                (Some(a), Some(b)) => {
                    let ad = text(olds, a).root(old.spelled(an), None).ok_or(place.error(Why::Held))?.decl;
                    let bd = text(news, b).root(new.spelled(bn), None).ok_or(place.error(Why::Held))?.decl;
                    let inner = self.request(
                        Key { old: location(a, At::Decl(ad)), new: location(b, At::Decl(bd)), root: false },
                        place,
                    );
                    return Ok(Kind::Content { bound, inner: Some(Inner::Held(inner)), narrowed });
                }
                (Some(_), None) | (None, Some(_)) => return Err(place.error(Why::Held)),
            }
        }
        let pair = &self.pairs.get(pair).expect("text pair").matching;
        let pieces = match (ah, bh) {
            (Held::Pieces(a), Held::Pieces(b)) => Some((a, b)),
            (Held::Pieces(a), Held::Opaque) => {
                let mut found = None;
                for at in 0..new.declarations() {
                    let b = DeclIx::new(at);
                    if pair.get(b.index()).copied().flatten() == Some(a) {
                        found = Some(b);
                        break;
                    }
                }
                Some((a, found.ok_or(place.error(Why::Pieces))?))
            }
            (Held::Opaque, Held::Pieces(b)) => {
                Some((pair.get(b.index()).copied().flatten().ok_or(place.error(Why::Pieces))?, b))
            }
            (Held::Opaque | Held::Record { .. }, Held::Opaque | Held::Record { .. }) => None,
            (Held::Pieces(_), Held::Record { .. }) | (Held::Record { .. }, Held::Pieces(_)) => {
                return Err(place.error(Why::Retyped));
            }
        };
        if let Some((a, b)) = pieces {
            let inner = self.child(key, At::Decl(a), At::Decl(b), place);
            return Ok(Kind::Content { bound, inner: Some(Inner::Pieces(inner)), narrowed });
        }
        Ok(opaque(bound, narrowed))
    }
    #[expect(clippy::manual_map, reason = "step code enumerates options without callbacks")]
    fn build_read(&mut self, olds: &[&Schema], at: u32) {
        let key = *self.read_keys.get(at).expect("reader key");
        let place = *self.read_places.get(at).expect("reader place");
        let schema = text(olds, key.text);
        let kind = match key.at {
            At::Decl(decl) => match schema.declaration(decl).shape {
                Shape::Record { versioned, first, count } => {
                    self.plan.fields = self.plan.fields.max(count);
                    let mut fields = List::with_capacity(count);
                    for offset in 0..count {
                        let field = schema.field(FieldIx::new(add(first.index(), offset)));
                        let mut child = location(key.text, At::Type(field.ty));
                        if offset == 0 {
                            child.variant = key.variant;
                        }
                        let reader = self.request_read(child, place);
                        fields.push(reader).expect("one reader per old field");
                    }
                    ReadKind::Record { head: versioned, fields }
                }
                Shape::Enum { first, count } => {
                    let mut variants = List::with_capacity(count);
                    for offset in 0..count {
                        let variant = VariantIx::new(add(first.index(), offset));
                        let payload = if key.variant.is_none() || key.variant == Some(variant) {
                            match schema.variant(variant).record {
                                Some(decl) => Some(self.request_read(location(key.text, At::Decl(decl)), place)),
                                None => None,
                            }
                        } else {
                            None
                        };
                        variants.push(payload).expect("one reader per old variant");
                    }
                    let only = match key.variant {
                        Some(at) => Some(
                            u8::try_from(at.index().checked_sub(first.index()).expect("restricted variant"))
                                .expect("schema tag"),
                        ),
                        None => None,
                    };
                    ReadKind::Enum { variants, only }
                }
            },
            At::Type(ty) => match schema.ty(ty) {
                Type::U8 => ReadKind::Fixed(1),
                Type::U16 => ReadKind::Fixed(2),
                Type::U32 => ReadKind::Fixed(4),
                Type::U64 | Type::Duration => ReadKind::Fixed(8),
                Type::Fixed(n) => ReadKind::Fixed(n),
                Type::Bool => ReadKind::Bool,
                Type::Text(bound) | Type::Bytes { bound, .. } | Type::Secret { bound, .. } => ReadKind::Content(bound),
                Type::Option(item) => ReadKind::Option(self.request_read(location(key.text, At::Type(item)), place)),
                Type::List { bound, item } => {
                    ReadKind::List { bound, item: self.request_read(location(key.text, At::Type(item)), place) }
                }
                Type::Named(decl) => ReadKind::Named(self.request_read(Location { at: At::Decl(decl), ..key }, place)),
            },
        };
        self.plan.reads.get_mut(at).expect("reader slot").kind = kind;
    }
}

fn opaque(bound: u32, narrowed: Option<Narrowed>) -> Kind {
    match narrowed {
        Some(narrowed) => Kind::Content { bound, inner: None, narrowed: Some(narrowed) },
        None => Kind::Copy,
    }
}
fn content_bound(ty: Type) -> Option<u32> {
    match ty {
        Type::Text(bound) | Type::Bytes { bound, .. } | Type::Secret { bound, .. } => Some(bound),
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::List { .. }
        | Type::Option(_)
        | Type::Named(_) => None,
    }
}
fn widen_content(ty: Type, minimum: u32) -> Type {
    match ty {
        Type::Text(bound) => Type::Text(bound.max(minimum)),
        Type::Bytes { bound, held } => Type::Bytes { bound: bound.max(minimum), held },
        Type::Secret { bound, held } => Type::Secret { bound: bound.max(minimum), held },
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::List { .. }
        | Type::Option(_)
        | Type::Named(_) => ty,
    }
}
fn empty(schema: &Schema, at: At) -> bool {
    match at {
        At::Type(at) => schema.fewest_bytes(at) == 0,
        At::Decl(at) => match schema.declaration(at).shape {
            Shape::Enum { .. } => false,
            Shape::Record { versioned, first, count } => {
                if versioned {
                    return false;
                }
                for offset in 0..count {
                    if schema.fewest_bytes(schema.field(FieldIx::new(add(first.index(), offset))).ty) != 0 {
                        return false;
                    }
                }
                true
            }
        },
    }
}
fn option(ty: Type) -> Option<TypeIx> {
    match ty {
        Type::Option(item) => Some(item),
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
        | Type::Named(_) => None,
    }
}
fn list(ty: Type) -> Option<(u32, TypeIx)> {
    match ty {
        Type::List { bound, item } => Some((bound, item)),
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
        | Type::Option(_)
        | Type::Named(_) => None,
    }
}
fn declaration(ty: Type) -> Option<DeclIx> {
    match ty {
        Type::Named(at) => Some(at),
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
        | Type::Option(_)
        | Type::List { .. } => None,
    }
}
fn held_record(held: Held) -> Option<(Span, Span)> {
    match held {
        Held::Record { family, name } => Some((family, name)),
        Held::Opaque | Held::Pieces(_) => None,
    }
}

fn family(texts: &[&Schema], name: &[u8]) -> Option<u32> {
    // Entry zero is the containing family's text, not an implicit held registration.
    for at in 1..u32::try_from(texts.len()).expect("held texts fit u32") {
        let schema = text(texts, at);
        if schema.spelled(schema.family()) == name {
            return Some(at);
        }
    }
    None
}
fn valid_root(schema: &Schema, root: Root) -> bool {
    if root.decl.index() >= schema.declarations() {
        return false;
    }
    let variant = match root.variant {
        Some(at) => {
            let leading = match schema.declaration(root.decl).shape {
                Shape::Enum { .. } => root.decl,
                Shape::Record { first, count, .. } => {
                    if count == 0 {
                        return false;
                    }
                    let Some(decl) = declaration(schema.ty(schema.field(first).ty)) else { return false };
                    decl
                }
            };
            let (first, count) = match schema.declaration(leading).shape {
                Shape::Enum { first, count } => (first, count),
                Shape::Record { .. } => return false,
            };
            if at.index() < first.index() || at.index() >= add(first.index(), count) {
                return false;
            }
            Some(schema.spelled(schema.variant(at).name))
        }
        None => None,
    };
    schema.root(schema.spelled(schema.declaration(root.decl).name), variant) == Some(root)
}
fn leading(schema: &Schema, root: Root) -> DeclIx {
    match schema.declaration(root.decl).shape {
        Shape::Enum { .. } => root.decl,
        Shape::Record { first, .. } => declaration(schema.ty(schema.field(first).ty)).expect("valid broad root"),
    }
}
fn root_tag(schema: &Schema, root: Root) -> Option<u8> {
    let variant = root.variant?;
    let leading = match schema.declaration(root.decl).shape {
        Shape::Enum { .. } => root.decl,
        Shape::Record { first, .. } => declaration(schema.ty(schema.field(first).ty)).expect("valid broad root"),
    };
    let first = match schema.declaration(leading).shape {
        Shape::Enum { first, .. } => first,
        Shape::Record { .. } => unreachable!("leading enum"),
    };
    Some(u8::try_from(variant.index().checked_sub(first.index()).expect("valid variant")).expect("schema tag"))
}

pub(super) fn plan(
    old: &Schema,
    old_root: Root,
    old_held: &[&Schema],
    new: &Schema,
    new_root: Root,
    new_held: &[&Schema],
    policy: Policy,
) -> Result<Plan, Underived> {
    let decl =
        if new_root.decl.index() < new.declarations() { new.declaration(new_root.decl).name } else { new.family() };
    let place = Place::new(decl);
    if !valid_root(old, old_root)
        || !valid_root(new, new_root)
        || old.spelled(old.family()) != new.spelled(new.family())
    {
        return Err(place.error(Why::Root));
    }
    let mut olds = List::uncapped();
    olds.push(old).expect("root text");
    for &text in old_held {
        olds.push(text).expect("held texts");
    }
    let mut news = List::uncapped();
    news.push(new).expect("root text");
    for &text in new_held {
        news.push(text).expect("held texts");
    }
    let headed = match old.declaration(old_root.decl).shape {
        Shape::Record { versioned, .. } => versioned,
        Shape::Enum { .. } => false,
    };
    let head = match new.declaration(new_root.decl).shape {
        Shape::Record { versioned: true, .. } => Some(new.version()),
        Shape::Record { versioned: false, .. } | Shape::Enum { .. } => None,
    };
    let mut builder = Builder::new(policy, head, headed, root_tag(old, old_root));
    let pair = builder.pair(olds.as_slice(), news.as_slice(), 0, 0);
    if builder.pairs.get(pair).expect("root pair").matching.get(new_root.decl.index()).copied().flatten()
        != Some(old_root.decl)
        || old_root.variant.is_some() != new_root.variant.is_some()
    {
        return Err(place.error(Why::Root));
    }
    if let Some(a) = old_root.variant {
        let b = new_root.variant.expect("both roots are broad");
        let old_enum = leading(old, old_root);
        let new_enum = leading(new, new_root);
        let old_names = Names { schema: old, shape: Some(old.declaration(old_enum).shape) };
        let new_names = Names { schema: new, shape: Some(new.declaration(new_enum).shape) };
        if builder.pairs.get(pair).expect("root pair").matching.get(new_enum.index()).copied().flatten()
            != Some(old_enum)
            || old_names.matched(new_names, Named::Variant(b), policy) != Some(Named::Variant(a))
        {
            return Err(Place { part: Some(new.variant(b).name), ..place }.error(Why::Root));
        }
    }
    builder.request(
        Key {
            old: Location { text: 0, at: At::Decl(old_root.decl), variant: old_root.variant },
            new: Location { text: 0, at: At::Decl(new_root.decl), variant: new_root.variant },
            root: true,
        },
        place,
    );
    builder.complete(olds.as_slice(), news.as_slice())?;
    Ok(builder.plan)
}

fn finish(builder: &mut Builder) -> Result<(), Underived> {
    for _ in 0..super::MAX_DEPTH {
        #[cfg(test)]
        {
            builder.passes = add(builder.passes, 1);
        }
        let mut changed = false;
        for at in 0..builder.plan.reads.len() {
            if builder.plan.read(at).depth == 0
                && let Some((depth, minimum)) = read_facts(&builder.plan, at)
            {
                let read = builder.plan.reads.get_mut(at).expect("reader slot");
                read.depth = depth;
                read.minimum = minimum;
                changed = true;
            }
        }
        for at in 0..builder.plan.nodes.len() {
            if builder.plan.node(at).depth == 0
                && let Some(facts) = node_facts(&builder.plan, at)
            {
                let node = builder.plan.nodes.get_mut(at).expect("conversion slot");
                node.depth = facts.depth;
                node.keeps = facts.keeps;
                node.empty = facts.empty;
                changed = true;
            }
        }
        // Facts depend only on settled children, whose nonzero depths never change.
        if !changed {
            break;
        }
    }
    for at in 0..builder.plan.reads.len() {
        let depth = builder.plan.read(at).depth;
        if depth == 0 || depth > super::MAX_DEPTH {
            return Err(builder.read_places.get(at).expect("reader place").error(Why::Deep));
        }
        builder.plan.depth = builder.plan.depth.max(depth);
    }
    for at in 0..builder.plan.nodes.len() {
        let depth = builder.plan.node(at).depth;
        if depth == 0 || depth > super::MAX_DEPTH {
            return Err(builder.places.get(at).expect("conversion place").error(Why::Deep));
        }
        builder.plan.depth = builder.plan.depth.max(depth);
    }
    Ok(())
}
fn read_facts(plan: &Plan, at: u32) -> Option<(u32, u32)> {
    let mut depth = 1;
    let minimum = match &plan.read(at).kind {
        ReadKind::Pending => return None,
        ReadKind::Fixed(n) => *n,
        ReadKind::Bool => 1,
        ReadKind::Content(_) => 4,
        ReadKind::Named(child) => {
            let read = plan.read(*child);
            if read.depth == 0 {
                return None;
            }
            return Some((read.depth, read.minimum));
        }
        ReadKind::Option(child) | ReadKind::List { item: child, .. } => {
            let read = plan.read(*child);
            if read.depth == 0 {
                return None;
            }
            depth = add(depth, read.depth);
            match plan.read(at).kind {
                ReadKind::Option(_) => 1,
                ReadKind::List { .. } => 4,
                ReadKind::Pending
                | ReadKind::Named(_)
                | ReadKind::Fixed(_)
                | ReadKind::Bool
                | ReadKind::Content(_)
                | ReadKind::Record { .. }
                | ReadKind::Enum { .. } => unreachable!("wrapper reader"),
            }
        }
        ReadKind::Record { head, fields } => {
            let mut minimum = if *head { 2 } else { 0 };
            for &field in fields {
                let read = plan.read(field);
                if read.depth == 0 {
                    return None;
                }
                depth = depth.max(add(1, read.depth));
                minimum = add(minimum, read.minimum);
            }
            minimum
        }
        ReadKind::Enum { variants, .. } => {
            let mut minimum = u32::MAX;
            for variant in variants {
                let min = match variant {
                    Some(child) => {
                        let read = plan.read(*child);
                        if read.depth == 0 {
                            return None;
                        }
                        depth = depth.max(add(1, read.depth));
                        add(1, read.minimum)
                    }
                    None => 1,
                };
                minimum = minimum.min(min);
            }
            minimum
        }
    };
    Some((depth, minimum))
}
struct NodeFacts {
    depth: u32,
    keeps: bool,
    empty: bool,
}
fn node_facts(plan: &Plan, at: u32) -> Option<NodeFacts> {
    let node = plan.node(at);
    let read = plan.read(node.read);
    let mut depth = read.depth;
    let mut empty = node.empty && read.minimum == 0;
    if depth == 0 {
        return None;
    }
    let keeps = match &node.kind {
        Kind::Pending => return None,
        Kind::Copy => true,
        Kind::Integer { .. } | Kind::Content { inner: None, .. } => false,
        Kind::Named(child) => {
            let child = plan.node(*child);
            if child.depth == 0 {
                return None;
            }
            depth = depth.max(child.depth);
            empty &= child.empty;
            child.keeps
        }
        Kind::Wrap(child)
        | Kind::Unwrap { child, .. }
        | Kind::Option(child)
        | Kind::List { item: child, .. }
        | Kind::Content { inner: Some(Inner::Held(child) | Inner::Pieces(child)), .. } => {
            let child = plan.node(*child);
            if child.depth == 0 {
                return None;
            }
            let extra = match node.kind {
                Kind::Content { inner: Some(Inner::Pieces(_)), .. } => 2,
                Kind::Wrap(_)
                | Kind::Unwrap { .. }
                | Kind::Option(_)
                | Kind::List { .. }
                | Kind::Content { inner: Some(Inner::Held(_)), .. } => 1,
                Kind::Pending
                | Kind::Named(_)
                | Kind::Copy
                | Kind::Integer { .. }
                | Kind::Record { .. }
                | Kind::Enum(_)
                | Kind::Content { inner: None, .. } => unreachable!("wrapper or content conversion"),
            };
            depth = depth.max(add(extra, child.depth));
            match node.kind {
                Kind::Wrap(_) | Kind::Unwrap { .. } => false,
                Kind::List { narrowed, .. } | Kind::Content { narrowed, .. } => narrowed.is_none() && child.keeps,
                Kind::Pending
                | Kind::Copy
                | Kind::Integer { .. }
                | Kind::Named(_)
                | Kind::Record { .. }
                | Kind::Enum(_) => unreachable!("wrapper or content conversion"),
                Kind::Option(_) => child.keeps,
            }
        }
        Kind::Record { fields, dropped, in_place, .. } => {
            // Even a record with no input or output bytes can refuse a dropped field.
            empty &= dropped.is_empty();
            let mut keeps = *in_place && dropped.is_empty();
            for field in fields {
                if let Some(child) = field.node {
                    let child = plan.node(child);
                    if child.depth == 0 {
                        return None;
                    }
                    depth = depth.max(add(1, child.depth));
                    empty &= child.empty;
                    keeps &= child.keeps;
                }
            }
            keeps
        }
        Kind::Enum(variants) => {
            let mut keeps = true;
            for variant in variants {
                match variant {
                    None => {}
                    Some(Variant::Refuse(_)) => keeps = false,
                    Some(Variant::Kept { in_place, payload, .. }) => {
                        keeps &= in_place;
                        if let Some(child) = payload {
                            let child = plan.node(*child);
                            if child.depth == 0 {
                                return None;
                            }
                            depth = depth.max(add(1, child.depth));
                            keeps &= child.keeps;
                        }
                    }
                }
            }
            keeps
        }
    };
    Some(NodeFacts { depth, keeps, empty })
}
