//! Schema-driven samples and a worker shim with the contiguous codec pass.
use super::wire;
use skein_codec::{
    layout::{Misconverted, Mode, Plan, Workspace},
    schema::{self, DeclIx, FieldIx, Schema, Shape, Type, TypeIx, VariantIx},
};
use skein_lib::{Buffer, Reader, Rng};
use wire::{KindRule, Record};

pub fn older(text: &str) -> String {
    replace(&replace(text, "    parts: bytes 134217728 = 0x\n", ""), ", part_busy: PartBusy", "")
}

pub fn richer(text: &str) -> String {
    let text = replace(text, "value: FactValue }", "value: FactValue worker_note: u32 = 19 }");
    let text = replace(
        &text,
        "enum FromWorker {\n    hello: Hello",
        "record WorkerOnly { value: u8 }\nenum FromWorker {\n    worker_only: WorkerOnly, announced was hello: Hello",
    );
    replace(&text, "assignment: Assignment, message: Inbound", "message: Inbound, assignment: Assignment")
}

pub fn replace(text: &str, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1, "edit one schema phrase: {old}");
    text.replace(old, new)
}

pub fn parse(text: &str) -> Schema {
    schema::parse(text.as_bytes()).expect("sample schema parses")
}

pub fn stated(schema: &Schema) -> wire::layouts::Theirs<'_> {
    wire::layouts::Theirs {
        name: core::str::from_utf8(schema.spelled(schema.family())).expect("family identifier"),
        layout: schema.fingerprints().family().digest(),
        text: schema.text(),
    }
}

pub struct Shim {
    plan: Plan,
    work: Workspace,
}

impl Shim {
    pub fn new(mode: Mode, source: &Schema, target: &Schema, root: &str) -> Self {
        let old = source.root(root.as_bytes(), None).expect("shim source root");
        let new = target.root(root.as_bytes(), None).expect("shim target root");
        let plan = Plan::between(mode, source, old, &[], target, new, &[]).expect("shim derives");
        let mut work = Workspace::new();
        assert!(work.fit(&plan));
        Self { plan, work }
    }

    pub fn convert(&mut self, bytes: &[u8], out: &mut Buffer) -> Result<(), Misconverted> {
        self.plan.convert(bytes, out, &mut self.work)
    }
}

pub const DOWN: u16 = 0x0180;
pub const UP: u16 = 0x0100;
pub const ROOM: u32 = 16_384;

pub fn frame(schema: &Schema, rule: KindRule, rng: &mut Rng, defaults: bool) -> Box<[u8]> {
    let (base, name) =
        if rule.kind >= DOWN { (DOWN, b"FromEngine".as_slice()) } else { (UP, b"FromWorker".as_slice()) };
    let decl = schema.find(name).expect("frame declaration");
    let Shape::Enum { first, .. } = schema.declaration(decl).shape else { panic!("frame enumeration") };
    let tag = u8::try_from(rule.kind.checked_sub(base).expect("application kind")).expect("frame tag");
    let variant = schema.variant(VariantIx::new(add(first.index(), u32::from(tag))));
    let mut out = Buffer::with_capacity(ROOM);
    put(&mut out, &[tag]);
    draw(schema, variant.record.expect("frame payload"), rng, &mut out, defaults);
    if base == DOWN {
        let record = wire::v22::FromEngine::decode(&wire::CEILINGS, &mut Reader::new(out.as_slice()))
            .expect("drawn engine value decodes");
        record.encode(&wire::CEILINGS).expect("engine value encodes")
    } else {
        let record = wire::v22::FromWorker::decode(&wire::CEILINGS, &mut Reader::new(out.as_slice()))
            .expect("drawn worker value decodes");
        record.encode(&wire::CEILINGS).expect("worker value encodes")
    }
}

// Draw canonical lengths, tags and field values from the schema, as in
// skein/docs/design/data/codec/run-time.md, 2.3. Small lists keep each sample lean.
pub fn draw(schema: &Schema, decl: DeclIx, rng: &mut Rng, out: &mut Buffer, defaults: bool) {
    match schema.declaration(decl).shape {
        Shape::Record { versioned, first, count } => {
            if versioned {
                put(out, &schema.version().to_be_bytes());
            }
            for index in first.index()..add(first.index(), count) {
                let at = FieldIx::new(index);
                if defaults && let Some(bytes) = schema.default(at) {
                    put(out, bytes);
                } else {
                    draw_type(schema, schema.field(at).ty, rng, out, defaults);
                }
            }
        }
        Shape::Enum { first, count } => {
            let tag = index(rng.below(u64::from(count)));
            put(out, &[u8::try_from(tag).expect("enum tag")]);
            if let Some(record) = schema.variant(VariantIx::new(add(first.index(), tag))).record {
                draw(schema, record, rng, out, defaults);
            }
        }
    }
}

fn draw_type(schema: &Schema, at: TypeIx, rng: &mut Rng, out: &mut Buffer, defaults: bool) {
    match schema.ty(at) {
        Type::U8 => put(out, rng.next_u64().to_be_bytes().first_chunk::<1>().expect("u8 bytes")),
        Type::U16 => put(out, rng.next_u64().to_be_bytes().first_chunk::<2>().expect("u16 bytes")),
        Type::U32 => put(out, rng.next_u64().to_be_bytes().first_chunk::<4>().expect("u32 bytes")),
        Type::U64 | Type::Duration => put(out, &rng.next_u64().to_be_bytes()),
        Type::Bool => put(out, &[u8::from(rng.below(2) != 0)]),
        Type::Fixed(length) => content(rng, out, length),
        Type::Text(bound) | Type::Secret { bound, .. } | Type::Bytes { bound, .. } => {
            let length = index(rng.between(0, u64::from(bound.min(8))));
            put(out, &length.to_be_bytes());
            content(rng, out, length);
        }
        Type::Option(item) => {
            let present = rng.below(2) != 0;
            put(out, &[u8::from(present)]);
            if present {
                draw_type(schema, item, rng, out, defaults);
            }
        }
        Type::List { bound, item } => {
            let count = index(rng.between(0, u64::from(bound.min(2))));
            put(out, &count.to_be_bytes());
            for _ in 0..count {
                draw_type(schema, item, rng, out, defaults);
            }
        }
        Type::Named(decl) => draw(schema, decl, rng, out, defaults),
    }
}

fn add(a: u32, b: u32) -> u32 {
    a.checked_add(b).expect("sample index fits")
}

fn index(value: u64) -> u32 {
    u32::try_from(value).expect("sample index fits")
}

fn content(rng: &mut Rng, out: &mut Buffer, length: u32) {
    for _ in 0..length {
        put(out, &[b'a'.checked_add(u8::try_from(rng.below(26)).expect("letter offset")).expect("ASCII letter")]);
    }
}

pub fn put(out: &mut Buffer, bytes: &[u8]) {
    out.put(bytes).expect("sample fits its buffer");
}
