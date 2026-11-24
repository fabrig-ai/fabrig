//! Frame capacities measured independently of the maps (protocol/store/transcripts.md, 6, protocol/store/README.md, 8).
#![expect(
    clippy::disallowed_macros,
    clippy::print_stderr,
    reason = "the measurement reads a pinned fixture and reports its numbers"
)]

use crate::transcripts::patches;
use skein_codec::{Record, patch::Patcher, schema};

// The current transcript schema and its owner set bound the frame machine.
const REAL: &str = smith_transcript::SCHEMA;
// Smith's real turn alone, as its set keeps it from the baseline on.
const REAL_SETS: crate::OwnerSets = crate::OwnerSets {
    sets: &[crate::OwnerSet {
        level: Some(0),
        families: &[crate::KeptFamily {
            current: REAL,
            name: smith_transcript::FAMILY,
            frames: Some(crate::Frames {
                root: smith_transcript::ROOT,
                texts: smith_transcript::patches::kept::smith_transcript::TEXTS,
                patches: smith_transcript::patches::PATCHES,
            }),
            ..super::frame_patches::SETS.sets[1].families[0]
        }],
        ..super::frame_patches::SETS.sets[1]
    }],
};
const CANDIDATES: [u32; 4] = [4096, 16_384, 65_536, 262_144];

// 21 envelope/message + 2*4 text + 16*6 calls + 16*14 read results.
const TYPICAL_NODES: u32 = 349;

#[test]
fn frame_capacities_cover_real_and_kept_turns() {
    let limits = super::fixtures::store_limits();
    let sets = super::frame_patches::SETS;
    for nodes in CANDIDATES {
        let mut capacity = patches::capacity(&sets.sets[1], 0, &limits, &[]).expect("turn capacity");
        capacity.nodes = nodes;
        eprintln!(
            "smith nodes {nodes}: {} bytes (versions {}, patches {}, text {}, bytes {})",
            Patcher::workspace_bound(capacity).expect("heap"),
            capacity.versions,
            capacity.patches,
            capacity.text,
            capacity.bytes
        );
    }
    let schema = schema::parse(REAL.as_bytes()).expect("smith's pinned schema");
    let root = schema.find(smith_transcript::ROOT.as_bytes()).expect("Turn");
    kept_turns_fit(&sets, &limits);
    let real = patches::capacity(&REAL_SETS.sets[0], 0, &limits, &[]).expect("real turn capacity");
    let maximum = maximum_decl(&schema, root);
    assert_eq!(real.nodes, maximum.0, "smith's machine reads every turn its kept schema allows");
    assert!(u64::from(maximum.1) <= u64::from(real.bytes), "the maximum-node turn fits the transcript's bytes");
    eprintln!("real smith Turn maximum nodes {}, achieved in {} bytes", real.nodes, maximum.1);
    let typical = typical_turn();
    let mut value = skein_codec::value::Value::with_room(TYPICAL_NODES);
    value.decode(&schema, root, typical.clone()).expect("typical tool-heavy turn fits its node count");
    let mut smaller = skein_codec::value::Value::with_room(TYPICAL_NODES.checked_sub(1).expect("nonempty"));
    assert_eq!(smaller.decode(&schema, root, typical.clone()), Err(skein_codec::value::Refusal::Room));
    eprintln!("typical smith 34-block tool-heavy turn {} bytes, {TYPICAL_NODES} nodes", typical.len());
    eprintln!("real smith machine heap {}", Patcher::workspace_bound(real).expect("real heap"));
}

fn kept_turns_fit(sets: &crate::OwnerSets, limits: &crate::Limits) {
    for set in sets.sets {
        if set.owner != crate::Owner::Smith {
            continue;
        }
        for (ix, family) in set.families.iter().enumerate() {
            let Some(frames) = &family.frames else { continue };
            let root = frames.root;
            let capacity = patches::capacity(set, ix, limits, &[]).expect("kept capacity");
            let meta = patches::kept_patcher(frames, capacity, &[]).expect("kept metadata");
            let last = schema::parse(frames.texts.last().expect("kept last").as_bytes()).expect("last text").version();
            let mut most = 0;
            for text in frames.texts {
                let schema = schema::parse(text.as_bytes()).expect("kept text");
                let root = patches::held_root(&meta, schema.version(), last, root.as_bytes()).expect("kept root");
                most = most.max(maximum_decl(&schema, root).0);
            }
            assert_eq!(capacity.nodes, most, "the machine's nodes are its kept texts' most");
        }
    }
}

#[test]
fn the_current_transcript_schema_matches_its_generated_bounds() {
    let schema = schema::parse(REAL.as_bytes()).expect("smith's pinned schema");
    check_generated_bounds(&schema);
    let root = schema.find(smith_transcript::ROOT.as_bytes()).expect("Turn");
    assert_eq!(
        Some(u64::from(schema.most_bytes(root))),
        smith_transcript::Turn::encoded_bound(&smith_transcript::CEILINGS),
        "the parser and generated codec agree on the maximum turn"
    );
    assert_eq!(u64::from(schema.version()), smith_transcript::Turn::FORMAT_VERSION);
    assert_eq!(schema.spelled(schema.family()), smith_transcript::FAMILY.as_bytes());
}

// Compare every bound in schema order, including bounds inside options and lists.
fn check_generated_bounds(schema: &schema::Schema) {
    use schema::{LimitLevel, Type};
    let ceilings = smith_transcript::CEILINGS.values();
    let names = smith_transcript::v8::LIMIT_NAMES;
    assert_eq!(names.len(), ceilings.as_slice().len(), "smith's generated limit names and values");
    let mut generated = names.iter().zip(ceilings.as_slice());
    for decl in 0..schema.declarations() {
        let declaration = schema.declaration(schema::DeclIx::new(decl));
        let schema::Shape::Record { first, count, .. } = declaration.shape else {
            continue;
        };
        for offset in 0..count {
            let field = schema.field(schema::FieldIx::new(first.index().checked_add(offset).expect("field")));
            let mut ty = field.ty;
            let mut name = schema::limit_name(schema.spelled(declaration.name), schema.spelled(field.name));
            for _ in 0..schema.text().len() {
                let (bound, next) = match schema.ty(ty) {
                    Type::Option(item) => {
                        ty = item;
                        name = schema::limit_child(&name, LimitLevel::Some);
                        continue;
                    }
                    Type::List { bound, item } => (bound, Some(item)),
                    Type::Bytes { bound, .. } | Type::Text(bound) | Type::Secret { bound, .. } => (bound, None),
                    Type::U8
                    | Type::U16
                    | Type::U32
                    | Type::U64
                    | Type::Bool
                    | Type::Duration
                    | Type::Fixed(_)
                    | Type::Named(_) => break,
                };
                let (&expected_name, &expected_bound) = generated.next().expect("no more bounds than smith's");
                assert_eq!(name.as_ref(), expected_name.as_bytes(), "copied limit: {expected_name}");
                assert_eq!(bound, expected_bound, "copied limit: {expected_name}");
                let Some(item) = next else { break };
                ty = item;
                name = schema::limit_child(&name, LimitLevel::Item);
            }
        }
    }
    assert_eq!(generated.next(), None, "the schema holds every generated limit");
}

fn maximum_decl(schema: &schema::Schema, decl: schema::DeclIx) -> (u32, u32) {
    use schema::{FieldIx, Shape, VariantIx};
    match schema.declaration(decl).shape {
        Shape::Record { versioned, first, count } => {
            let mut total = (1_u32, if versioned { 2_u32 } else { 0 });
            for offset in 0..count {
                let field = schema.field(FieldIx::new(first.index().checked_add(offset).expect("field")));
                let child = maximum_type(schema, field.ty);
                total.0 = total.0.checked_add(child.0).expect("nodes");
                total.1 = total.1.checked_add(child.1).expect("bytes");
            }
            total
        }
        Shape::Enum { first, count } => {
            let mut best = (0, 0);
            for offset in 0..count {
                if let Some(decl) =
                    schema.variant(VariantIx::new(first.index().checked_add(offset).expect("variant"))).record
                {
                    let child = maximum_decl(schema, decl);
                    if child.0 > best.0 || (child.0 == best.0 && child.1 < best.1) {
                        best = child;
                    }
                }
            }
            (best.0.checked_add(1).expect("enum"), best.1.checked_add(1).expect("tag"))
        }
    }
}
fn maximum_type(schema: &schema::Schema, ty: schema::TypeIx) -> (u32, u32) {
    use schema::Type;
    match schema.ty(ty) {
        Type::Named(decl) => maximum_decl(schema, decl),
        Type::List { bound, item } => {
            let child = maximum_type(schema, item);
            (
                child.0.checked_mul(bound).expect("nodes").checked_add(1).expect("list"),
                child.1.checked_mul(bound).expect("bytes").checked_add(4).expect("count"),
            )
        }
        Type::Option(item) => {
            let child = maximum_type(schema, item);
            (child.0.checked_add(1).expect("option"), child.1.checked_add(1).expect("presence"))
        }
        Type::Fixed(size) => (1, size),
        Type::U8 | Type::Bool => (1, 1),
        Type::U16 => (1, 2),
        Type::U32 | Type::Bytes { .. } | Type::Secret { .. } | Type::Text(_) => (1, 4),
        Type::U64 | Type::Duration => (1, 8),
    }
}
fn typical_turn() -> Box<[u8]> {
    // 48 envelope/message bytes + 2 * 11 text blocks + 16 * (27 call + 65 read-result).
    const TYPICAL_BYTES: usize = 1542;
    let mut bytes = skein_lib::Writer::new(TYPICAL_BYTES);
    bytes.put(&smith_transcript::wire::VERSION.to_be_bytes()).expect("version");
    bytes.put(&[0, 0]).expect("kind, opening");
    bytes.put(&0_u32.to_be_bytes()).expect("endpoint");
    bytes.put(&0_u32.to_be_bytes()).expect("model");
    bytes.put(&[0]).expect("effort");
    bytes.put(&0_u32.to_be_bytes()).expect("dialect");
    bytes.put(&1_u32.to_be_bytes()).expect("place");
    bytes.put(&[0; 5]).expect("usage");
    bytes.put(&0_u64.to_be_bytes()).expect("spent");
    bytes.put(&0_u32.to_be_bytes()).expect("jobs");
    bytes.put(&1_u32.to_be_bytes()).expect("messages");
    bytes.put(&[1]).expect("assistant");
    bytes.put(&34_u32.to_be_bytes()).expect("blocks");
    text_block(&mut bytes);
    for _ in 0_u8..16 {
        bytes.put(&[2]).expect("call block");
        content(&mut bytes, b"tool-id");
        content(&mut bytes, b"read");
        content(&mut bytes, b"{}");
        bytes.put(&[0]).expect("replay");
        bytes.put(&[3]).expect("result block");
        content(&mut bytes, b"tool-id");
        bytes.put(&[0, 0]).expect("returned outcome, read");
        content(&mut bytes, &[b'a'; 32]);
        for _ in 0_u8..3 {
            bytes.put(&1_u32.to_be_bytes()).expect("line fields");
        }
        bytes.put(&[1, 0, 0]).expect("end, whole, within");
    }
    text_block(&mut bytes);
    bytes.put(&[0]).expect("unavailable system context");
    bytes.finish()
}
fn text_block(bytes: &mut skein_lib::Writer) {
    bytes.put(&[0]).expect("text block");
    content(bytes, b"hello");
    bytes.put(&[0]).expect("no replay");
}
fn content(bytes: &mut skein_lib::Writer, text: &[u8]) {
    bytes.put(&u32::try_from(text.len()).expect("text").to_be_bytes()).expect("length");
    bytes.put(text).expect("content");
}
