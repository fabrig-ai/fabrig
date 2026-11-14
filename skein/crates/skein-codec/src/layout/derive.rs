//! Reach worklists and retained level conversions (data/codec/layouts.md, 2).
use super::{Field, Inner, Kind, Node, Plan, Read, ReadKind, Source, Underived, Variant, Why, add};
use crate::patch::{Comparison, Target, compare_held, compare_types, payload_matches};
use crate::schema::{DeclIx, FieldIx, Held, Named, Root, Schema, Shape, Span, Type, TypeIx, VariantIx};
use alloc::boxed::Box;
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum At {
    Decl(DeclIx),
    Type(TypeIx),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    text: u32,
    at: At,
    variant: Option<VariantIx>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}
impl Place {
    fn error(self, why: Why) -> Underived {
        Underived { why, decl: self.decl, part: self.part, old: self.old }
    }
}
#[derive(Debug)]
struct Pair {
    old: u32,
    new: u32,
    matching: List<Option<DeclIx>>,
}
struct Builder {
    plan: Plan,
    keys: List<Key>,
    places: List<Place>,
    read_keys: List<Location>,
    read_places: List<Place>,
    pairs: List<Pair>,
}

fn text<'a>(texts: &'a [&Schema], at: u32) -> &'a Schema {
    texts.get(skein_lib::usize_of(at)).expect("text index in the worklist")
}
fn location(text: u32, at: At) -> Location {
    Location { text, at, variant: None }
}
fn former(old: &Schema, new: &Schema, at: DeclIx) -> Option<DeclIx> {
    if let Some(found) = old.find(new.spelled(new.declaration(at).name)) {
        return Some(found);
    }
    for &name in new.was(Named::Declaration(at)) {
        if let Some(found) = old.find(new.spelled(name)) {
            return Some(found);
        }
    }
    None
}
fn parts(shape: Shape) -> (u32, u32) {
    match shape {
        Shape::Record { first, count, .. } => (first.index(), count),
        Shape::Enum { first, count } => (first.index(), count),
    }
}
fn target(shape: Shape, at: u32) -> Target {
    match shape {
        Shape::Record { .. } => Target::Field(FieldIx::new(at)),
        Shape::Enum { .. } => Target::Variant(VariantIx::new(at)),
    }
}
fn name(schema: &Schema, at: Target) -> Span {
    match at {
        Target::Declaration(at) => schema.declaration(at).name,
        Target::Field(at) => schema.field(at).name,
        Target::Variant(at) => schema.variant(at).name,
    }
}
fn named_part(at: Target) -> Named {
    match at {
        Target::Declaration(at) => Named::Declaration(at),
        Target::Field(at) => Named::Field(at),
        Target::Variant(at) => Named::Variant(at),
    }
}
fn find_part(schema: &Schema, shape: Shape, spelled: &[u8]) -> Option<Target> {
    let (first, count) = parts(shape);
    for at in first..add(first, count) {
        let at = target(shape, at);
        if schema.spelled(name(schema, at)) == spelled {
            return Some(at);
        }
    }
    None
}
fn matched_part(old: &Schema, shape: Shape, new: &Schema, current: Target) -> Option<Target> {
    // A current name present in the old scope takes precedence over its former names.
    if let Some(found) = find_part(old, shape, new.spelled(name(new, current))) {
        return Some(found);
    }
    for &was in new.was(named_part(current)) {
        if let Some(found) = find_part(old, shape, new.spelled(was)) {
            return Some(found);
        }
    }
    None
}

impl Builder {
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
        for at in 0..new.declarations() {
            matching.push(former(old, new, DeclIx::new(at))).expect("one declaration match slot per text declaration");
        }
        let at = self.pairs.len();
        self.pairs.push(Pair { old: a, new: b, matching }).expect("text pairs bounded by the reach");
        at
    }
    fn request_read(&mut self, key: Location, place: Place) -> u32 {
        for at in 0..self.read_keys.len() {
            if self.read_keys.get(at) == Some(&key) {
                return at;
            }
        }
        let at = self.read_keys.len();
        self.read_keys.push(key).expect("read nodes bounded by reached texts");
        self.read_places.push(place).expect("one place per reader");
        self.plan.reads.push(Read { kind: ReadKind::Pending, minimum: 0, depth: 0 }).expect("one reader per key");
        at
    }
    fn request(&mut self, key: Key, place: Place) -> u32 {
        for at in 0..self.keys.len() {
            if self.keys.get(at) == Some(&key) {
                return at;
            }
        }
        let at = self.keys.len();
        let read = self.request_read(key.old, place);
        self.keys.push(key).expect("conversion nodes bounded by reached texts");
        self.places.push(place).expect("one place per conversion");
        self.plan
            .nodes
            .push(Node { read, kind: Kind::Pending, keeps: false, empty: false, depth: 0 })
            .expect("one node per key");
        at
    }
    fn child(&mut self, key: Key, a: At, b: At, place: Place) -> u32 {
        self.request(Key { old: location(key.old.text, a), new: location(key.new.text, b), root: false }, place)
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
        let place = Place { decl: new.declaration(b).name, part: None, old: false };
        if self.pairs.get(pair).expect("text pair").matching.get(b.index()).copied().flatten() != Some(a) {
            return Err(place.error(Why::Retyped));
        }
        match (old.declaration(a).shape, new.declaration(b).shape) {
            (
                sa @ Shape::Record { first: af, count: ac, versioned: av },
                Shape::Record { first: bf, count: bc, versioned: bv },
            ) => {
                let mut fields = List::with_capacity(bc);
                let mut in_place = ac == bc && av == bv && (key.root || !av || old.version() == new.version());
                for offset in 0..bc {
                    let field = FieldIx::new(add(bf.index(), offset));
                    let part = Place { part: Some(new.field(field).name), ..place };
                    let source = match matched_part(old, sa, new, Target::Field(field)) {
                        Some(Target::Field(at)) => Some(at),
                        None => None,
                        Some(Target::Declaration(_) | Target::Variant(_)) => unreachable!("field matching"),
                    };
                    let field = if let Some(source) = source {
                        let old_offset = source.index().checked_sub(af.index()).expect("field in matched record");
                        in_place &= old_offset == offset;
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
                        let bytes = new.default(field).ok_or(part.error(Why::NoDefault))?;
                        Field { source: Source::Default(Box::from(bytes)), node: None }
                    };
                    fields.push(field).expect("one source per new field");
                }
                Ok(Kind::Record { old_head: av, head: if bv { Some(new.version()) } else { None }, fields, in_place })
            }
            (sa @ Shape::Enum { first: af, count: ac }, sb @ Shape::Enum { first: bf, .. }) => {
                let mut variants = List::with_capacity(ac);
                for offset in 0..ac {
                    let source = VariantIx::new(add(af.index(), offset));
                    if let Some(only) = key.old.variant
                        && only != source
                    {
                        variants.push(None).expect("one slot per old tag");
                        continue;
                    }
                    let mut found = None;
                    let (first, count) = parts(sb);
                    for index in first..add(first, count) {
                        let dest = VariantIx::new(index);
                        if matched_part(old, sa, new, Target::Variant(dest)) == Some(Target::Variant(source)) {
                            found = Some(dest);
                            break;
                        }
                    }
                    let dest = found.ok_or(
                        Place { decl: old.declaration(a).name, part: Some(old.variant(source).name), old: true }
                            .error(Why::VariantDropped),
                    )?;
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
                        .push(Some(Variant {
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
        if compare_types(&self.pairs.get(pair).expect("text pair").matching, old, new, a, b) == Comparison::Retype {
            return Err(place.error(Why::Retyped));
        }
        let av = old.ty(a);
        let bv = new.ty(b);
        match bv {
            Type::Option(b) => {
                if let Some(a) = option(av) {
                    Ok(Kind::Option(self.child(key, At::Type(a), At::Type(b), place)))
                } else {
                    Ok(Kind::Wrap(self.child(key, At::Type(a), At::Type(b), place)))
                }
            }
            Type::List { item: b, .. } => {
                let (bound, a) = list(av).expect("compared list");
                Ok(Kind::List { bound, item: self.child(key, At::Type(a), At::Type(b), place) })
            }
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
                // A named level has no wire of its own.
                Ok(Kind::Named(node))
            }
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                let a = width(av);
                let b = width(bv);
                Ok(if a == b { Kind::Copy } else { Kind::Integer { old: a, new: b } })
            }
            Type::Bytes { bound, held } | Type::Secret { bound, held } => {
                self.content(olds, news, pair, key, av, held, bound, place)
            }
            Type::Bool | Type::Duration | Type::Fixed(_) | Type::Text(_) => Ok(Kind::Copy),
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
        if let Some((af, an)) = held_record(ah)
            && let Some((bf, bn)) = held_record(bh)
        {
            let a = family(olds, old.spelled(af));
            let b = family(news, new.spelled(bf));
            match (a, b) {
                (None, None) => return Ok(Kind::Copy),
                (Some(a), Some(b)) => {
                    let ad = text(olds, a).find(old.spelled(an)).ok_or(place.error(Why::Held))?;
                    let bd = text(news, b).find(new.spelled(bn)).ok_or(place.error(Why::Held))?;
                    if !is_record(text(olds, a), ad) || !is_record(text(news, b), bd) {
                        return Err(place.error(Why::Held));
                    }
                    let inner = self.request(
                        Key { old: location(a, At::Decl(ad)), new: location(b, At::Decl(bd)), root: false },
                        place,
                    );
                    return Ok(Kind::Content { bound, inner: Inner::Held(inner) });
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
            return Ok(Kind::Content { bound, inner: Inner::Pieces(inner) });
        }
        Ok(Kind::Copy)
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

fn width(ty: Type) -> u32 {
    match ty {
        Type::U8 => 1,
        Type::U16 => 2,
        Type::U32 => 4,
        Type::U64 => 8,
        Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Secret { .. }
        | Type::Text(_)
        | Type::Bytes { .. }
        | Type::List { .. }
        | Type::Option(_)
        | Type::Named(_) => unreachable!("compared integer"),
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
fn is_record(schema: &Schema, at: DeclIx) -> bool {
    match schema.declaration(at).shape {
        Shape::Record { .. } => true,
        Shape::Enum { .. } => false,
    }
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
) -> Result<Plan, Underived> {
    let decl =
        if new_root.decl.index() < new.declarations() { new.declaration(new_root.decl).name } else { new.family() };
    let place = Place { decl, part: None, old: false };
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
    let mut builder = Builder {
        plan: Plan {
            nodes: List::uncapped(),
            reads: List::uncapped(),
            root: 0,
            depth: 0,
            fields: 0,
            head,
            old_head: headed,
            tag: root_tag(old, old_root),
        },
        keys: List::uncapped(),
        places: List::uncapped(),
        read_keys: List::uncapped(),
        read_places: List::uncapped(),
        pairs: List::uncapped(),
    };
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
        if builder.pairs.get(pair).expect("root pair").matching.get(new_enum.index()).copied().flatten()
            != Some(old_enum)
            || matched_part(old, old.declaration(old_enum).shape, new, Target::Variant(b)) != Some(Target::Variant(a))
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
    for at in 0..u32::MAX {
        if at >= builder.keys.len() {
            break;
        }
        builder.build(olds.as_slice(), news.as_slice(), at)?;
    }
    for at in 0..u32::MAX {
        if at >= builder.read_keys.len() {
            break;
        }
        builder.build_read(olds.as_slice(), at);
    }
    finish(&mut builder)?;
    Ok(builder.plan)
}

fn finish(builder: &mut Builder) -> Result<(), Underived> {
    for _ in 0..super::MAX_DEPTH {
        for at in 0..builder.plan.reads.len() {
            if builder.plan.read(at).depth == 0
                && let Some((depth, minimum)) = read_facts(&builder.plan, at)
            {
                let read = builder.plan.reads.get_mut(at).expect("reader slot");
                read.depth = depth;
                read.minimum = minimum;
            }
        }
        for at in 0..builder.plan.nodes.len() {
            if builder.plan.node(at).depth == 0
                && let Some((depth, keeps)) = node_facts(&builder.plan, at)
            {
                let node = builder.plan.nodes.get_mut(at).expect("conversion slot");
                node.depth = depth;
                node.keeps = keeps;
                node.empty &= builder.plan.reads.get(node.read).expect("reader").minimum == 0;
            }
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
#[expect(clippy::manual_flatten, reason = "explicit variants retain the sparse broad-root table")]
fn node_facts(plan: &Plan, at: u32) -> Option<(u32, bool)> {
    let node = plan.node(at);
    let mut depth = plan.read(node.read).depth;
    if depth == 0 {
        return None;
    }
    let keeps = match &node.kind {
        Kind::Pending => return None,
        Kind::Copy => true,
        Kind::Integer { .. } => false,
        Kind::Named(child) => {
            let child = plan.node(*child);
            if child.depth == 0 {
                return None;
            }
            depth = depth.max(child.depth);
            child.keeps
        }
        Kind::Wrap(child)
        | Kind::Option(child)
        | Kind::List { item: child, .. }
        | Kind::Content { inner: Inner::Held(child) | Inner::Pieces(child), .. } => {
            let child = plan.node(*child);
            if child.depth == 0 {
                return None;
            }
            let extra = match node.kind {
                Kind::Content { inner: Inner::Pieces(_), .. } => 2,
                Kind::Wrap(_) | Kind::Option(_) | Kind::List { .. } | Kind::Content { inner: Inner::Held(_), .. } => 1,
                Kind::Pending
                | Kind::Named(_)
                | Kind::Copy
                | Kind::Integer { .. }
                | Kind::Record { .. }
                | Kind::Enum(_) => unreachable!("wrapper conversion"),
            };
            depth = depth.max(add(extra, child.depth));
            match node.kind {
                Kind::Wrap(_) => false,
                Kind::Pending
                | Kind::Copy
                | Kind::Integer { .. }
                | Kind::Named(_)
                | Kind::Option(_)
                | Kind::List { .. }
                | Kind::Record { .. }
                | Kind::Enum(_)
                | Kind::Content { .. } => child.keeps,
            }
        }
        Kind::Record { fields, in_place, .. } => {
            let mut keeps = *in_place;
            for field in fields {
                if let Some(child) = field.node {
                    let child = plan.node(child);
                    if child.depth == 0 {
                        return None;
                    }
                    depth = depth.max(add(1, child.depth));
                    keeps &= child.keeps;
                }
            }
            keeps
        }
        Kind::Enum(variants) => {
            let mut keeps = true;
            for variant in variants {
                if let Some(variant) = variant {
                    keeps &= variant.in_place;
                    if let Some(child) = variant.payload {
                        let child = plan.node(child);
                        if child.depth == 0 {
                            return None;
                        }
                        depth = depth.max(add(1, child.depth));
                        keeps &= child.keeps;
                    }
                }
            }
            keeps
        }
    };
    Some((depth, keeps))
}
