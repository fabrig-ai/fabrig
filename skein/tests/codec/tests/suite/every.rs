//! The byte pin's committed goldens (data/codec/README.md, section 6).

#[cfg(test)]
mod tests {
    use crate::generic_support as generic;
    use crate::support::generated::{self, Decoded};
    use skein_codec::value::Value;

    #[test]
    #[expect(clippy::panic, reason = "each golden failure names its version and file")]
    fn every_committed_golden_round_trips_at_ceilings() {
        let mut versions = Vec::new();
        let mut value = Value::with_room(generic::ROOM);
        for family in generated::families().into_iter().filter(|family| family.name == "every") {
            let schema = skein_codec::schema::parse(family.text.as_bytes()).expect("every schema");
            let version = schema.version();
            versions.push(version);
            let mut paths = std::fs::read_dir(format!("golden/every-v{version}"))
                .expect("every golden directory")
                .map(|entry| entry.expect("golden entry").path())
                .collect::<Vec<_>>();
            paths.sort();
            assert!(!paths.is_empty(), "every-v{version}: no committed goldens");
            for path in paths {
                let name = path.file_name().expect("golden name").to_str().expect("UTF-8 golden name");
                let context = format!("every-v{version}/{name}");
                let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{context}: {error}"));
                let decl = generic::golden_root(&schema, name);
                let root = family
                    .roots
                    .iter()
                    .find(|root| root.name.as_bytes() == schema.spelled(schema.declaration(decl).name))
                    .expect("generated decoder for every declaration");
                match (root.decode)(&bytes) {
                    Decoded::Value { bytes: encoded, measured, .. } => {
                        assert_eq!(usize::try_from(measured).expect("measured length"), bytes.len(), "{context}: measure");
                        assert_eq!(encoded, bytes, "{context}: generated round trip");
                    }
                    Decoded::Problem { path, reason } => panic!("{context}: generated decode: {path}: {reason:?}"),
                }
                value
                    .decode(&schema, decl, bytes.clone().into_boxed_slice())
                    .unwrap_or_else(|error| panic!("{context}: generic decode: {error:?}"));
                assert_eq!(value.bytes(), bytes, "{context}: generic round trip");
                if version == 3 && name == "record_blank_smallest.bin" {
                    assert_eq!(bytes, [0, 3], "{context}: the blank is its version alone");
                }
            }
        }
        assert_eq!(versions, [1, 2, 3], "all versions of the byte pin");
    }
}
