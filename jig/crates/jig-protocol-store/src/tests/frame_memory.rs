//! Frame capacities measured independently of the maps (protocol/store/transcripts.md, 6, protocol/store/README.md, 8).
use skein_codec::{Record, schema};
const REAL: &str = smith_transcript::SCHEMA;

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
