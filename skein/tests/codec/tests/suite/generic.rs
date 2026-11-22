//! Every schema's fixed-rule values agree with the generic decoder (data/codec/README.md, 8.3).

extern crate alloc;

use crate::generic_support as generic;
use crate::support::generated;

#[cfg(test)]
mod tests {
    use super::{generated, generic};
    use skein_codec::value::Value;

    #[test]
    fn oracle_disagreement_returns_the_case_context() {
        let family = generated::Family { name: "planted", text: "family planted 1\nrecord Byte { n: u8 }", roots: &[] };
        let root = generated::Root {
            name: "Byte",
            decode: |_| generated::Decoded::Problem { path: "ByteN".into(), reason: skein_codec::Reason::Short },
        };
        let schema = skein_codec::schema::parse(family.text.as_bytes()).expect("schema");
        let decl = schema.find(root.name.as_bytes()).expect("root");
        let mut value = Value::with_room(generic::ROOM);
        let mut oracle = generic::Oracle { schema: &schema, decl, family: &family, root: &root, value: &mut value };

        let finding = oracle.compare(0x1234, &[7], false).expect_err("generic accepts, generated refuses");

        assert!(finding.starts_with("planted.Byte, seed 0x0000000000001234, bytes [7]: generic Ok("), "{finding}");
        assert!(finding.ends_with(", generated Problem { path: \"ByteN\", reason: Short }"), "{finding}");
    }

    #[test]
    #[expect(clippy::panic, reason = "a refused fixed-rule value reports the generated path and reason")]
    fn every_fixed_value_decodes_generically_to_its_own_bytes_and_tree() {
        let mut paths = std::fs::read_dir("schema")
            .expect("schema directory")
            .map(|entry| entry.expect("schema entry").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "schema"))
            .collect::<Vec<_>>();
        paths.push("../../examples/ui/view/schema/notes-v1.schema".into());
        paths.sort();
        let mut value = Value::with_room(generic::ROOM);
        let mut count = 0_u32;
        let mut compared = 0_u32;
        for path in paths {
            let current = generated::current_text(path.to_str().expect("schema path"));
            let text = current.map_or_else(|| std::fs::read_to_string(&path).expect("schema readable"), str::to_owned);
            let schema = skein_codec::schema::parse(text.as_bytes()).expect("schema valid");
            let fixed_values = current.map_or_else(
                || skein_codegen::fixed_values(&skein_codegen::parse(&text).expect("generator schema")),
                |text| skein_codegen::values(text).fixed.clone(),
            );
            let family = generated::families().into_iter().find(|family| family.text == text);
            for fixed_value in fixed_values {
                let root = generic::named_root(&schema, &fixed_value.name);
                let node = value
                    .decode(&schema, root, fixed_value.bytes.clone().into_boxed_slice())
                    .expect("fixed-rule value decodes");
                assert_eq!(value.bytes(), fixed_value.bytes, "{path:?}: {}", fixed_value.name);
                if let Some(family) = &family {
                    let name = schema.spelled(schema.declaration(root).name);
                    let generated_root =
                        family.roots.iter().find(|root| root.name.as_bytes() == name).expect("generated decoder");
                    match (generated_root.decode)(&fixed_value.bytes) {
                        generated::Decoded::Value { debug, bytes, .. } => {
                            assert_eq!(
                                generic::render(&schema, &value, node, root),
                                debug,
                                "{path:?}: {}",
                                fixed_value.name
                            );
                            assert_eq!(value.bytes(), bytes);
                        }
                        generated::Decoded::Problem { path, reason } => {
                            panic!("fixed_value {}: {path}: {reason:?}", fixed_value.name)
                        }
                    }
                    compared = compared.checked_add(1).expect("fixed-rule value count");
                }
                count = count.checked_add(1).expect("fixed-rule value count");
            }
        }
        // The codec fixtures contribute 388 fixed-rule values, including mapped's 234 and
        // growth's two; the UI schema contributes 12.
        assert_eq!(count, 400);
        assert_eq!(compared, count);
    }
}

#[cfg(test)]
mod single_byte {
    use super::generated::{Decoded, Root};
    use super::{generated, generic};
    use skein_codec::schema::{self, Schema};
    use skein_codec::value::{Refusal, Value};

    fn compare(schema: &Schema, root: &Root, value: &mut Value, bytes: &[u8]) {
        let decl = schema.find(root.name.as_bytes()).expect("root");
        match ((root.decode)(bytes), value.decode(schema, decl, bytes.into())) {
            (Decoded::Value { debug, bytes: encoded, .. }, Ok(node)) => {
                assert_eq!(value.bytes(), encoded, "{}; input={bytes:02x?}", root.name);
                assert_eq!(generic::render(schema, value, node, decl), debug, "{}; input={bytes:02x?}", root.name);
            }
            (Decoded::Problem { path, reason }, Err(Refusal::Problem(problem))) => {
                assert_eq!(
                    (generic::path_name(schema, problem.path), problem.reason),
                    (path, reason),
                    "{}; input={bytes:02x?}",
                    root.name
                );
            }
            (expected, decoded) => {
                unreachable!("{}; input={bytes:02x?}; generated={expected:?}; generic={decoded:?}", root.name);
            }
        }
    }

    #[test]
    fn every_possible_single_byte_matches_all_generated_roots() {
        for family in generated::families() {
            let schema = schema::parse(family.text.as_bytes()).expect("runtime schema");
            let mut value = Value::with_room(generic::ROOM);
            for root in family.roots {
                for byte in u8::MIN..=u8::MAX {
                    compare(&schema, root, &mut value, &[byte]);
                }
            }
            assert_eq!(schema.spelled(schema.family()), family.name.as_bytes());
        }
    }
}
