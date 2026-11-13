//! The visitor's reachable shape and bounds (temper/docs/design/client/forge/README.md, 9.4).
#[path = "public/bounds.rs"]
mod bounds;
#[path = "public/shape.rs"]
mod shape;

#[expect(clippy::disallowed_macros, reason = "the committed schema and shape are test inputs")]
const SCHEMA: &str = include_str!("../../schema/public-v1.schema");
#[expect(clippy::disallowed_macros, reason = "the committed schema and shape are test inputs")]
const PANELS: &str = include_str!("../../schema/client-wire-v1.schema");
#[expect(clippy::disallowed_macros, reason = "the committed schema and shape are test inputs")]
const FIELDS: &str = include_str!("../../schema/public-v1.fields");

#[test]
fn the_public_schema_names_nothing_from_elsewhere() {
    shape::closed(SCHEMA, PANELS).expect("the public schema is closed and apart from the panels");
}

#[test]
fn the_fields_golden_is_the_reachable_wire_shape() {
    let listing = shape::listing(SCHEMA);
    let path = std::path::Path::new("schema/public-v1.fields");
    if temper_test_env::rewrite() {
        std::fs::write(path, &listing).unwrap();
    }
    assert_eq!(listing, std::fs::read_to_string(path).unwrap());
}

#[test]
fn a_nested_field_added_changes_the_golden() {
    let member = SCHEMA.replace("record PublicMember {", "record PublicMember { extra: u32");
    assert_ne!(shape::listing(&member), FIELDS);
    let step = SCHEMA.replace("enum PublicStep {", "enum PublicStep { extra,");
    assert_ne!(shape::listing(&step), FIELDS);
    let unreachable =
        SCHEMA.replace("family temper-public-wire 1", "family temper-public-wire 1\nrecord Unreachable { extra: u32 }");
    assert_eq!(shape::listing(&unreachable), FIELDS);
}

#[test]
fn a_name_from_elsewhere_breaks_the_closure() {
    let foreign = SCHEMA.replace("head: option fixed 32", "head: option Pull");
    assert!(shape::closed(&foreign, PANELS).is_err());
    // Even copying the declaration locally must not admit a panel's record name.
    let copied = foreign.replace("enum PublicStep", "record Pull { address: bytes 2048 }\nenum PublicStep");
    skein_codegen::parse(&copied).expect("the copied panel record parses locally");
    assert!(shape::closed(&copied, PANELS).is_err());
}

#[test]
fn public_bounds_hold_the_largest_answers() {
    bounds::largest_answers();
}
