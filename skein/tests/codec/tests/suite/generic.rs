//! Every schema's goldens agree with the generic decoder (data/codec/README.md, 8.3).

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
    #[expect(clippy::panic, reason = "a refused golden reports the generated path and reason")]
    fn every_golden_decodes_generically_to_its_own_bytes_and_tree() {
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
            let text = std::fs::read_to_string(&path).expect("schema readable");
            let schema = skein_codec::schema::parse(text.as_bytes()).expect("schema valid");
            let generated_schema = skein_codegen::parse(&text).expect("generator schema");
            let family = generated::families().into_iter().find(|family| family.text == text);
            for golden in skein_codegen::generate(&generated_schema).goldens {
                let root = generic::golden_root(&schema, &golden.name);
                let node =
                    value.decode(&schema, root, golden.bytes.clone().into_boxed_slice()).expect("golden decodes");
                assert_eq!(value.bytes(), golden.bytes, "{path:?}: {}", golden.name);
                if let Some(family) = &family {
                    let name = schema.spelled(schema.declaration(root).name);
                    let generated_root =
                        family.roots.iter().find(|root| root.name.as_bytes() == name).expect("generated decoder");
                    match (generated_root.decode)(&golden.bytes) {
                        generated::Decoded::Value { debug, bytes } => {
                            assert_eq!(
                                generic::render(&schema, &value, node, root),
                                debug,
                                "{path:?}: {}",
                                golden.name
                            );
                            assert_eq!(value.bytes(), bytes);
                        }
                        generated::Decoded::Problem { path, reason } => {
                            panic!("golden {}: {path}: {reason:?}", golden.name)
                        }
                    }
                    compared = compared.checked_add(1).expect("golden count");
                }
                count = count.checked_add(1).expect("golden count");
            }
        }
        // Five generated families contribute 84, the UI schema contributes 12; every-v2 contributes 28.
        assert_eq!(count, 124);
        assert_eq!(compared, 84);
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
            (Decoded::Value { debug, bytes: encoded }, Ok(node)) => {
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
