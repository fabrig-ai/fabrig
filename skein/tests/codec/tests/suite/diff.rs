//! Kept/current fixture stories (codec.md, 10.6).
//! Set `SKEIN_DIFF_WRITE=1` to rewrite `diff/<case>/diff.txt` from the schema pairs.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use skein_codec::schema;
    use skein_codegen::diff::{Kind, Rename, Version, diff};

    #[test]
    fn task_store_changes_tell_each_kind_and_mixed_landing() {
        let rows = [
            (
                "none",
                vec![
                    Rename::Declaration { old: "Task".into(), new: "WorkItem".into() },
                    Rename::Part { decl: "WorkItem".into(), old: "title".into(), new: "summary".into() },
                    Rename::Part { decl: "State".into(), old: "open".into(), new: "pending".into() },
                ],
                Kind::NoneNeeded,
                Version::Kept,
                vec![],
            ),
            ("derived", vec![], Kind::Derived, Version::Moved, vec![]),
            ("written", vec![], Kind::Written, Version::Moved, vec!["Key", "Task", "Event"]),
            ("mixed", vec![], Kind::Written, Version::Moved, vec!["Task"]),
        ];
        let write = std::env::var_os("SKEIN_DIFF_WRITE").is_some_and(|value| value == "1");
        for (case, renames, kind, version, written) in rows {
            let directory = Path::new("diff").join(case);
            let read = |name| fs::read(directory.join(name)).expect("story fixture readable");
            let kept = schema::parse(&read("kept.schema")).expect("valid kept story");
            let current = schema::parse(&read("current.schema")).expect("valid current story");
            let result = diff(&kept, &current, &renames).expect("same family and valid renames");
            assert_eq!(result.kind(), Some(kind), "{case}");
            assert_eq!(result.version(), version, "{case}");
            assert_eq!(result.written(), written, "{case}");
            let path = directory.join("diff.txt");
            if write {
                fs::write(path, result.to_string()).expect("write story diff");
            } else {
                let expected = fs::read_to_string(path).expect("story diff committed");
                assert_eq!(
                    result.to_string(),
                    expected,
                    "{case}: run SKEIN_DIFF_WRITE=1 heavy cargo nextest run -p skein-codec-tests --test suite -E 'test(/^diff::/)'"
                );
            }
        }
    }
}

#[cfg(test)]
mod wire {

    use crate::{generic_support as generic, support::generated};
    use skein_codec::schema::{self, Schema};
    use skein_codec::value::{Refusal, Value};
    use skein_codegen::diff::{self, Diff, Kind};
    use skein_lib::Rng;
    fn read(text: &str) -> Schema {
        schema::parse(text.as_bytes()).expect("wire schema")
    }

    fn lines(diff: &Diff) -> Vec<String> {
        diff.changes.iter().map(|change| format!("{} {change}", change.kind)).collect()
    }

    fn decoded(schema: &Schema, root: &str, bytes: &[u8]) -> Result<String, Refusal> {
        let decl = schema.find(root.as_bytes()).expect("QA root");
        let mut value = Value::with_room(generic::ROOM);
        let node = value.decode(schema, decl, bytes.into())?;
        Ok(generic::render(schema, &value, node, decl))
    }

    #[test]
    fn none_needed_bound_changes_preserve_generated_draws() {
        // Sample Field has text 4 and bytes 4. Both widen without changing bytes.
        const CURRENT: &str = "family sample 1 record Field { name: bytes 8 value: secret 8 }";
        const DRAWS: u64 = 64; // Tiny bounds: both empty and full draws, over fixed replay seeds.
        let family = generated::families().into_iter().find(|family| family.name == "sample").expect("sample family");
        let root = family.roots.iter().find(|root| root.name == "Field").expect("Field generated decoder");
        let kept = read(family.text);
        let current = read(CURRENT);
        let drawn_schema = skein_codegen::parse(family.text).expect("generated draw schema");
        let diff = diff::diff(&kept, &current, &[]).expect("same family");
        assert_eq!(diff.kind(), Some(Kind::NoneNeeded));
        for seed in 0..DRAWS {
            let bytes = generated::draw(&drawn_schema, root.name, &mut Rng::new(seed));
            let generated::Decoded::Value { bytes: encoded, .. } = (root.decode)(&bytes) else {
                unreachable!("a generated draw decodes with its generated decoder");
            };
            assert_eq!(encoded, bytes, "seed {seed}");
            let decl = current.find(b"Field").expect("current Field");
            let mut value = Value::with_room(generic::ROOM);
            let node =
                value.decode(&current, decl, bytes.clone().into_boxed_slice()).expect("none needed accepts kept bytes");
            let schema::Shape::Record { first, .. } = current.declaration(decl).shape else {
                unreachable!("Field is a record");
            };
            let name = value.field(node, first).expect("name field");
            let next = schema::FieldIx::new(first.index().checked_add(1).expect("value field index"));
            let secret = value.field(node, next).expect("value field");
            let mut old = Value::with_room(generic::ROOM);
            let old_node = old
                .decode(&kept, kept.find(b"Field").expect("kept Field"), bytes.into_boxed_slice())
                .expect("kept draw");
            let old_name = old.field(old_node, first).expect("kept name");
            let old_value = old.field(old_node, next).expect("kept value");
            assert_eq!(value.content(name).expect("bytes"), old.content(old_name).expect("text"), "seed {seed}");
            assert_eq!(value.expose(secret).expect("secret"), old.content(old_value).expect("bytes"), "seed {seed}");
        }
    }

    #[test]
    fn adding_unused_declaration_preserves_every_generated_root() {
        const DRAWS: u64 = 8; // Every fixture root, at eight replayable seeds.
        for family in generated::families() {
            let current_text = format!("{}\nrecord Unused {{ n: u8 }}\n", family.text);
            let kept = read(family.text);
            let current = read(&current_text);
            let diff = diff::diff(&kept, &current, &[]).expect("same family");
            assert_eq!(lines(&diff), ["none add Unused"], "{}", family.name);
            assert_ne!(kept.fingerprints().family(), current.fingerprints().family(), "{}", family.name);
            let drawn_schema = skein_codegen::parse(family.text).expect("generated draw schema");
            for root in family.roots {
                for seed in 0..DRAWS {
                    let bytes = generated::draw(&drawn_schema, root.name, &mut Rng::new(seed));
                    match (root.decode)(&bytes) {
                        generated::Decoded::Value { debug, bytes: encoded } => {
                            assert_eq!(encoded, bytes, "{} {}, seed {seed}", family.name, root.name);
                            assert_eq!(
                                decoded(&current, root.name, &bytes),
                                Ok(debug),
                                "{} {}, seed {seed}",
                                family.name,
                                root.name
                            );
                        }
                        generated::Decoded::Problem { path, reason } => {
                            unreachable!("a generated draw decodes: {path} {reason:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn enum_append_preserves_every_old_tag_and_meaning() {
        let kept = read("family f 1 enum E { a, b, c }");
        let current = read("family f 1 enum E { a, b, c, d }");
        let diff = diff::diff(&kept, &current, &[]).expect("same family");
        assert_eq!(lines(&diff), ["none add E.d"]);
        for (bytes, expected) in [([0], "A"), ([1], "B"), ([2], "C")] {
            assert_eq!(decoded(&current, "E", &bytes), Ok(expected.into()));
        }
    }

    #[test]
    fn derived_order_can_decode_bytes_with_a_different_meaning() {
        let kept = read("family f 1 record R { a: u8 b: u8 }");
        let current = read("family f 1 record R { b: u8 a: u8 }");
        let diff = diff::diff(&kept, &current, &[]).expect("same family");
        assert_eq!(kept.fingerprints().family(), current.fingerprints().family());
        assert_eq!(lines(&diff), ["derived order R b a"]);
        assert_eq!(decoded(&kept, "R", &[7, 9]), Ok("R { a: 7, b: 9 }".into()));
        assert_eq!(decoded(&current, "R", &[7, 9]), Ok("R { b: 7, a: 9 }".into()));
    }

    #[test]
    fn written_byte_to_text_retype_rejects_non_utf8_kept_value() {
        let kept = read("family f 1 record R { f: bytes 1 }");
        let current = read("family f 1 record R { f: text 1 }");
        let diff = diff::diff(&kept, &current, &[]).expect("same family");
        assert_eq!(lines(&diff), ["written retype R.f bytes 1 text 1"]);
        assert_eq!(decoded(&kept, "R", &[0, 0, 0, 1, 255]), Ok("R { f: [255] }".into()));
        assert!(decoded(&current, "R", &[0, 0, 0, 1, 255]).is_err(), "non-UTF-8 bytes cannot be text");
    }
}
